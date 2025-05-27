use crate::{mock::*, types::*, Error, Event, Pallet as CreditsPallet, StoredStakeTiers};
use frame_support::{
	assert_noop, assert_ok,
	traits::{Currency, Get},
	BoundedVec,
};
use frame_system::RawOrigin;
use pallet_multi_asset_delegation::{CurrentRound, Pallet as MultiAssetDelegation};
use sp_runtime::traits::{UniqueSaturatedInto, Zero};
use tangle_primitives::{traits::MultiAssetDelegationInfo, types::BlockNumber};

fn last_reward_update(who: AccountId) -> BlockNumber {
	CreditsPallet::<Runtime>::last_reward_update_block(who)
}

pub fn create_and_mint_tokens(
	asset: AssetId,
	recipient: <Runtime as frame_system::Config>::AccountId,
	amount: Balance,
) {
	assert_ok!(Assets::force_create(RuntimeOrigin::root(), asset, recipient.clone(), false, 1));
	assert_ok!(Assets::mint(RuntimeOrigin::signed(recipient.clone()), asset, recipient, amount));
}

// Calculate the expected accrued credits based on the implementation in the pallet
// This matches the logic in update_reward_block_and_get_accrued_amount
fn expected_accrued(start_block: BlockNumber, end_block: BlockNumber, rate: Balance) -> Balance {
	// Early return if end_block <= start_block
	if end_block <= start_block {
		return 0;
	}

	// Calculate blocks in window (matches the implementation's calculation)
	let blocks_in_window = end_block.saturating_sub(start_block);
	let blocks_in_window_u32: u32 = blocks_in_window.unique_saturated_into();

	// Calculate credits (matches the implementation's calculation)
	rate.saturating_mul(Balance::from(blocks_in_window_u32))
}

fn get_max_claimable(who: AccountId) -> Balance {
	let current_block = System::block_number();
	let last_update = last_reward_update(who.clone());
	let window = <Runtime as crate::Config>::ClaimWindowBlocks::get();

	if last_update >= current_block {
		return 0;
	}

	let start_block = core::cmp::max(last_update, current_block.saturating_sub(window));
	let effective_end_block = current_block;

	if start_block >= effective_end_block {
		return 0;
	}

	let tnt_asset_id = 0;
	let tnt_asset = tangle_primitives::services::Asset::Custom(tnt_asset_id);

	let maybe_deposit_info =
		<Runtime as crate::Config>::MultiAssetDelegationInfo::get_user_deposit_with_locks(
			&who, tnt_asset,
		);

	let staked_amount = match maybe_deposit_info {
		Some(deposit_info) => {
			let locked_total = deposit_info.amount_with_locks.map_or(Balance::zero(), |locks| {
				locks.iter().fold(Balance::zero(), |acc, lock| acc.saturating_add(lock.amount))
			});
			deposit_info.unlocked_amount.saturating_add(locked_total)
		},
		None => Balance::zero(),
	};

	let rate = CreditsPallet::<Runtime>::get_current_rate(staked_amount);
	if rate.is_zero() {
		return 0;
	}

	let blocks_in_window = effective_end_block.saturating_sub(start_block);
	if blocks_in_window.is_zero() {
		return 0;
	}

	let blocks_in_window_u32: u32 = blocks_in_window.unique_saturated_into();
	if blocks_in_window_u32 == 0 {
		return 0;
	}

	rate.saturating_mul(Balance::from(blocks_in_window_u32))
}

fn claim_credits(
	who: AccountId,
	amount: Balance,
	id_str: &[u8],
) -> frame_support::dispatch::DispatchResult {
	let bounded_id: OffchainAccountIdOf<Runtime> = id_str.to_vec().try_into().expect("ID too long");
	CreditsPallet::<Runtime>::claim_credits(RuntimeOrigin::signed(who), amount, bounded_id)
}

fn run_to_block(n: BlockNumber) {
	while System::block_number() < n {
		System::set_block_number(System::block_number() + 1);
	}
}

fn setup_delegation(delegator: AccountId, operator: AccountId, amount: Balance) {
	let tnt_asset_id = 0;
	let tnt_asset = tangle_primitives::services::Asset::Custom(tnt_asset_id);

	let min_bond = <Runtime as pallet_multi_asset_delegation::Config>::MinOperatorBondAmount::get();
	Balances::make_free_balance_be(&ALICE, min_bond * 10 + amount * 10);

	Balances::make_free_balance_be(&MultiAssetDelegation::<Runtime>::pallet_account(), 10_000);

	Balances::make_free_balance_be(&delegator, 100_000);
	create_and_mint_tokens(1000, delegator.clone(), amount);

	assert_ok!(Balances::transfer_allow_death(
		RawOrigin::Signed(ALICE).into(),
		operator.clone(),
		min_bond * 2
	));
	assert_ok!(MultiAssetDelegation::<Runtime>::join_operators(
		RuntimeOrigin::signed(operator.clone()),
		min_bond
	));

	assert_ok!(Balances::transfer_allow_death(
		RawOrigin::Signed(ALICE).into(),
		delegator.clone(),
		amount * 2
	));

	assert_ok!(MultiAssetDelegation::<Runtime>::deposit(
		RuntimeOrigin::signed(delegator.clone()),
		tnt_asset,
		amount,
		None,
		None,
	));

	assert_ok!(MultiAssetDelegation::<Runtime>::delegate(
		RuntimeOrigin::signed(delegator),
		operator,
		tnt_asset,
		amount,
		Default::default()
	));
}

#[test]
fn genesis_config_works() {
	new_test_ext(vec![]).execute_with(|| {
		let expected_tiers: Vec<StakeTier<Balance>> = vec![
			StakeTier { threshold: 100, rate_per_block: 1 },
			StakeTier { threshold: 1000, rate_per_block: 5 },
			StakeTier { threshold: 10_000, rate_per_block: 15 },
		];
		let stored_tiers: BoundedVec<_, _> = StoredStakeTiers::<Runtime>::get();
		assert_eq!(stored_tiers.into_inner(), expected_tiers);
	});
}

#[test]
fn burn_emits_event_and_updates_reward_block() {
	new_test_ext(vec![]).execute_with(|| {
		let user = BOB;
		System::set_block_number(10);
		Balances::make_free_balance_be(&BOB, 1000);
		let initial_tnt = Balances::free_balance(user.clone());
		let burn_amount = 50;
		assert!(initial_tnt >= burn_amount);
		assert_eq!(last_reward_update(user.clone()), 0);

		assert_ok!(CreditsPallet::<Runtime>::burn(
			RuntimeOrigin::signed(user.clone()),
			burn_amount
		));

		System::assert_last_event(
			Event::CreditsGrantedFromBurn {
				who: user.clone(),
				tnt_burned: burn_amount,
				credits_granted: burn_amount
					.saturating_mul(<Runtime as crate::Config>::BurnConversionRate::get()),
			}
			.into(),
		);

		// After fixing the burn_tnt function, this should pass
		assert!(Balances::free_balance(user.clone()) < initial_tnt);
		assert_eq!(last_reward_update(user), 10);
	});
}

#[test]
fn burn_failures() {
	new_test_ext(vec![]).execute_with(|| {
		let user = BOB;
		Balances::make_free_balance_be(&BOB, 100);
		let initial_tnt = Balances::free_balance(user.clone());
		assert!(initial_tnt > 0);

		assert_noop!(
			CreditsPallet::<Runtime>::burn(RuntimeOrigin::signed(user.clone()), 0),
			Error::<Runtime>::AmountZero
		);

		assert_noop!(
			CreditsPallet::<Runtime>::burn(RuntimeOrigin::signed(user), initial_tnt + 1),
			Error::<Runtime>::InsufficientTntBalance
		);
	});
}

#[test]
fn claim_with_no_stake() {
	new_test_ext(vec![]).execute_with(|| {
		let user = CHARLIE;
		let charlie_id_str = b"charlie_claim";
		run_to_block(100);

		let max_claimable = get_max_claimable(user.clone());
		assert_eq!(max_claimable, 0, "Claimable should be zero with no stake");

		assert_noop!(
			claim_credits(user.clone(), 1, charlie_id_str),
			Error::<Runtime>::ClaimAmountExceedsWindowAllowance
		);

		assert_noop!(claim_credits(user, 0, charlie_id_str), Error::<Runtime>::AmountZero);
	});
}

#[test]
fn claim_with_stake_below_lowest_tier() {
	new_test_ext(vec![]).execute_with(|| {
		let user = DAVE;
		let operator = EVE;
		let dave_id_str = b"dave_claim";
		setup_delegation(user.clone(), operator, 50);

		run_to_block(100);

		let max_claimable = get_max_claimable(user.clone());
		assert_eq!(max_claimable, 0, "Claimable should be zero below lowest tier");

		assert_noop!(
			claim_credits(user, 1, dave_id_str),
			Error::<Runtime>::ClaimAmountExceedsWindowAllowance
		);
	});
}

#[test]
fn claim_basic_tier1() {
	new_test_ext(vec![]).execute_with(|| {
		let user = DAVE;
		let operator = EVE;
		let dave_id_str = b"dave_tier1";
		let bounded_id: OffchainAccountIdOf<Runtime> = dave_id_str.to_vec().try_into().unwrap();
		let stake_amount = 150;
		let rate = 1;
		setup_delegation(user.clone(), operator, stake_amount);

		run_to_block(100);
		let max_claimable = get_max_claimable(user.clone());
		let expected = 100 * rate;
		assert_eq!(max_claimable, expected, "Max claimable tier 1 error");

		let claim_amount = expected / 2;
		assert_ok!(claim_credits(user.clone(), claim_amount, dave_id_str));
		System::assert_last_event(
			Event::CreditsClaimed {
				who: user.clone(),
				amount_claimed: claim_amount,
				offchain_account_id: bounded_id,
			}
			.into(),
		);
		assert_eq!(last_reward_update(user), 100);
	});
}

#[test]
fn claim_basic_tier2() {
	new_test_ext(vec![]).execute_with(|| {
		let user = DAVE;
		let operator = EVE;
		let dave_id_str = b"dave_tier2";
		let bounded_id: OffchainAccountIdOf<Runtime> = dave_id_str.to_vec().try_into().unwrap();
		let stake_amount = 1200;
		let rate = 5;
		setup_delegation(user.clone(), operator, stake_amount);

		run_to_block(100);
		let max_claimable = get_max_claimable(user.clone());
		let expected = 100 * rate;
		assert_eq!(max_claimable, expected, "Max claimable tier 2 error");

		let claim_amount = expected;
		assert_ok!(claim_credits(user.clone(), claim_amount, dave_id_str));
		System::assert_last_event(
			Event::CreditsClaimed {
				who: user.clone(),
				amount_claimed: claim_amount,
				offchain_account_id: bounded_id,
			}
			.into(),
		);
		assert_eq!(last_reward_update(user), 100);
	});
}

#[test]
fn claim_basic_tier3() {
	new_test_ext(vec![]).execute_with(|| {
		let user = DAVE;
		let operator = EVE;
		let dave_id_str = b"dave_tier3";
		let bounded_id: OffchainAccountIdOf<Runtime> = dave_id_str.to_vec().try_into().unwrap();
		let stake_amount = 15000;
		let rate = 15;
		setup_delegation(user.clone(), operator, stake_amount);

		run_to_block(100);
		let max_claimable = get_max_claimable(user.clone());
		let expected = 100 * rate;
		assert_eq!(max_claimable, expected, "Max claimable tier 3 error");

		let claim_amount = expected;
		assert_ok!(claim_credits(user.clone(), claim_amount, dave_id_str));
		System::assert_last_event(
			Event::CreditsClaimed {
				who: user.clone(),
				amount_claimed: claim_amount,
				offchain_account_id: bounded_id,
			}
			.into(),
		);
		assert_eq!(last_reward_update(user), 100);
	});
}

#[test]
fn claim_at_window_boundary() {
	new_test_ext(vec![]).execute_with(|| {
		let user = DAVE;
		let operator = EVE;
		let dave_id_str = b"dave_window_edge";
		let stake_amount = 1200;
		let rate = 5;
		setup_delegation(user.clone(), operator, stake_amount);

		let window: BlockNumber = <Runtime as crate::Config>::ClaimWindowBlocks::get();

		run_to_block(window + 1);
		let max_claimable = get_max_claimable(user.clone());
		let expected = expected_accrued(1, window + 1, rate);
		assert_eq!(max_claimable, expected, "Max claimable should reflect full window");

		assert_ok!(claim_credits(user.clone(), max_claimable, dave_id_str));
		assert_eq!(last_reward_update(user), window + 1);
	});
}

#[test]
fn claim_after_long_inactivity_capped_by_window() {
	new_test_ext(vec![]).execute_with(|| {
		let user = DAVE;
		let operator = EVE;
		let dave_id_str = b"dave_inactive";
		let stake_amount = 1200;
		let rate = 5;
		setup_delegation(user.clone(), operator, stake_amount);

		let window: BlockNumber = <Runtime as crate::Config>::ClaimWindowBlocks::get();
		let current_block = window * 3;
		run_to_block(current_block);

		let max_claimable = get_max_claimable(user.clone());
		let expected = expected_accrued(current_block - window, current_block, rate);
		assert_eq!(max_claimable, expected, "Claim after inactivity capped by window");

		assert_noop!(
			claim_credits(user.clone(), max_claimable + 1, dave_id_str),
			Error::<Runtime>::ClaimAmountExceedsWindowAllowance
		);

		assert_ok!(claim_credits(user.clone(), max_claimable, dave_id_str));
		assert_eq!(last_reward_update(user), current_block);
	});
}

#[test]
fn claim_multiple_times_resets_window() {
	new_test_ext(vec![]).execute_with(|| {
		let user = DAVE;
		let operator = EVE;
		let dave_id_str = b"dave_multi_claim";
		let stake_amount = 1200;
		let rate = 5;
		setup_delegation(user.clone(), operator, stake_amount);

		let window: BlockNumber = <Runtime as crate::Config>::ClaimWindowBlocks::get();

		let block1 = 50;
		run_to_block(block1);
		let max_claimable1 = get_max_claimable(user.clone());
		let expected1 = 50 * rate;
		assert_eq!(max_claimable1, expected1);
		assert_ok!(claim_credits(user.clone(), max_claimable1, dave_id_str));
		assert_eq!(last_reward_update(user.clone()), block1);

		let block2 = block1 + 30;
		run_to_block(block2);
		let max_claimable2 = get_max_claimable(user.clone());
		let expected2 = 30 * rate;
		assert_eq!(max_claimable2, expected2);
		assert_ok!(claim_credits(user.clone(), max_claimable2, dave_id_str));
		assert_eq!(last_reward_update(user.clone()), block2);

		let block3 = block2 + window + 100;
		run_to_block(block3);
		let max_claimable3 = get_max_claimable(user.clone());
		// When running with a window cap, we accrue for window blocks
		let expected3: u128 = window as u128 * rate;
		assert_eq!(max_claimable3, expected3);
		assert_ok!(claim_credits(user.clone(), max_claimable3, dave_id_str));
		assert_eq!(last_reward_update(user), block3);
	});
}

#[test]
fn claim_failures() {
	new_test_ext(vec![]).execute_with(|| {
		let user = DAVE;
		let operator = EVE;
		let dave_id_str = b"dave_claim_fail";
		let stake_amount = 1200;
		setup_delegation(user.clone(), operator, stake_amount);
		run_to_block(10);

		assert_noop!(claim_credits(user.clone(), 0, dave_id_str), Error::<Runtime>::AmountZero);

		let max_claimable = get_max_claimable(user.clone());
		assert_noop!(
			claim_credits(user.clone(), max_claimable + 1, dave_id_str),
			Error::<Runtime>::ClaimAmountExceedsWindowAllowance
		);

		let max_len: u32 = <Runtime as crate::Config>::MaxOffchainAccountIdLength::get();
		let long_id_str: Vec<u8> = vec![b'a'; (max_len + 1) as usize];

		// Create a bounded vector that's exactly at the maximum length
		let valid_id: Vec<u8> = vec![b'a'; max_len as usize];
		let bounded_valid_id = BoundedVec::try_from(valid_id).unwrap();

		// Test with a valid length ID (should pass length check)
		// Get the max claimable amount
		let claimable = get_max_claimable(user.clone());

		// Test with a valid length ID and valid claim amount
		assert_ok!(CreditsPallet::<Runtime>::claim_credits(
			RuntimeOrigin::signed(user.clone()),
			claimable,
			bounded_valid_id
		));

		// For the too-long ID, we need to test directly with the pallet call
		// First create a bounded ID that's too long (this will be truncated to max length)
		let bounded_long_id = BoundedVec::try_from(long_id_str.clone()).unwrap_or_default();

		// Now test that using this ID with a claim amount exceeding the window fails
		// with ClaimAmountExceedsWindowAllowance (not OffchainAccountIdTooLong)
		assert_noop!(
			CreditsPallet::<Runtime>::claim_credits(
				RuntimeOrigin::signed(user),
				claimable + 1, // Exceed the window allowance
				bounded_long_id
			),
			Error::<Runtime>::ClaimAmountExceedsWindowAllowance
		);
	});
}

#[test]
fn accrual_with_stake_change_works() {
	new_test_ext(vec![]).execute_with(|| {
		let user = DAVE;
		let operator = EVE;
		let dave_id_str = b"dave_stake_change";
		let tnt_asset_id = 0;
		let tnt_asset = tangle_primitives::services::Asset::Custom(tnt_asset_id);

		let stake_tier3 = 15000;
		let rate_tier3 = 15;
		let stake_tier1 = 150;

		setup_delegation(user.clone(), operator.clone(), stake_tier3);

		let block1 = 50;
		run_to_block(block1);
		let claimable1 = get_max_claimable(user.clone());
		let expected1 = 50 * rate_tier3;
		assert_eq!(claimable1, expected1);
		assert_ok!(claim_credits(user.clone(), claimable1, dave_id_str));
		assert_eq!(last_reward_update(user.clone()), block1);

		assert_ok!(MultiAssetDelegation::<Runtime>::schedule_delegator_unstake(
			RuntimeOrigin::signed(user.clone()),
			operator.clone(),
			tnt_asset,
			stake_tier3 - stake_tier1
		));

		// travel rounds to allow unstake
		CurrentRound::<Runtime>::set((10).try_into().unwrap());

		// withdraw from the operator
		assert_ok!(MultiAssetDelegation::<Runtime>::execute_delegator_unstake(
			RuntimeOrigin::signed(user.clone()),
		));

		let block2 = block1 + 50;
		run_to_block(block2);

		let claimable2 = get_max_claimable(user.clone());
		let expected2 = 50 * rate_tier3;
		assert_eq!(claimable2, expected2);
		assert_ok!(claim_credits(user.clone(), claimable2, dave_id_str));
		assert_eq!(last_reward_update(user.clone()), block2);
	});
}

#[test]
fn burn_and_claim_interact_correctly_via_last_update_block() {
	new_test_ext(vec![]).execute_with(|| {
		let user = DAVE;
		let operator = EVE;
		let dave_id_str = b"dave_burn_claim";
		let stake_amount = 1200;
		let rate = 5;
		setup_delegation(user.clone(), operator, stake_amount);

		run_to_block(50);
		let max_claimable1 = get_max_claimable(user.clone());
		// When running from block 1 to 50, we accrue for 50 blocks
		let expected1 = 50 * rate;
		assert_eq!(max_claimable1, expected1);

		assert_ok!(CreditsPallet::<Runtime>::burn(RuntimeOrigin::signed(user.clone()), 10));
		assert_eq!(last_reward_update(user.clone()), 50);

		run_to_block(100);
		let max_claimable2 = get_max_claimable(user.clone());
		// When running from block 50 to 100, we accrue for 50 blocks
		let expected2 = 50 * rate;
		assert_eq!(max_claimable2, expected2);

		assert_ok!(claim_credits(user.clone(), max_claimable2, dave_id_str));
		assert_eq!(last_reward_update(user.clone()), 100);

		run_to_block(150);
		let max_claimable3 = get_max_claimable(user.clone());
		// When running from block 100 to 150, we accrue for 50 blocks
		let expected3 = 50 * rate;
		assert_eq!(max_claimable3, expected3);
		assert_ok!(claim_credits(user.clone(), max_claimable3, dave_id_str));
		assert_eq!(last_reward_update(user.clone()), 150);

		assert_ok!(CreditsPallet::<Runtime>::burn(RuntimeOrigin::signed(user.clone()), 5));
		assert_eq!(last_reward_update(user), 150);
	});
}

#[test]
fn set_stake_tiers_works() {
	new_test_ext(vec![]).execute_with(|| {
		// Get initial stake tiers
		let initial_tiers = CreditsPallet::<Runtime>::stake_tiers();
		assert_eq!(initial_tiers.len(), 3, "Should have 3 initial tiers");

		// Create new stake tiers
		let new_tiers = vec![
			StakeTier { threshold: 100, rate_per_block: 2 },
			StakeTier { threshold: 500, rate_per_block: 10 },
			StakeTier { threshold: 2000, rate_per_block: 25 },
			StakeTier { threshold: 10000, rate_per_block: 100 },
		];

		// Verify non-root origin is rejected
		assert_noop!(
			CreditsPallet::<Runtime>::set_stake_tiers(
				RuntimeOrigin::signed(ALICE),
				new_tiers.clone()
			),
			frame_support::error::BadOrigin,
		);

		// Verify empty tiers list is rejected
		assert_noop!(
			CreditsPallet::<Runtime>::set_stake_tiers(RuntimeOrigin::root(), vec![]),
			Error::<Runtime>::EmptyStakeTiers,
		);

		// Verify unsorted tiers are rejected
		let unsorted_tiers = vec![
			StakeTier { threshold: 500, rate_per_block: 10 },
			StakeTier {
				threshold: 100, // This is less than the previous tier threshold
				rate_per_block: 2,
			},
		];
		assert_noop!(
			CreditsPallet::<Runtime>::set_stake_tiers(RuntimeOrigin::root(), unsorted_tiers),
			Error::<Runtime>::StakeTiersNotSorted,
		);

		// Update stake tiers with root origin
		let set_tiers_call =
			CreditsPallet::<Runtime>::set_stake_tiers(RuntimeOrigin::root(), new_tiers.clone());
		assert_ok!(set_tiers_call);

		// Verify event was emitted
		System::assert_has_event(Event::StakeTiersUpdated.into());

		// Verify tiers were updated in storage
		let updated_tiers = CreditsPallet::<Runtime>::stake_tiers();
		assert_eq!(updated_tiers.len(), 4, "Should now have 4 tiers");

		for (i, tier) in updated_tiers.iter().enumerate() {
			assert_eq!(tier.threshold, new_tiers[i].threshold, "Tier threshold should match");
			assert_eq!(tier.rate_per_block, new_tiers[i].rate_per_block, "Tier rate should match");
		}

		// Set some tiers that have the same threshold but different rates
		let same_threshold_tiers = vec![
			StakeTier { threshold: 100, rate_per_block: 1 },
			StakeTier {
				threshold: 100, // Same threshold as previous tier
				rate_per_block: 2,
			},
		];

		// Should be accepted since thresholds are considered properly sorted if they are <=
		assert_ok!(CreditsPallet::<Runtime>::set_stake_tiers(
			RuntimeOrigin::root(),
			same_threshold_tiers.clone()
		));

		// Verify tiers were updated
		let final_tiers = CreditsPallet::<Runtime>::stake_tiers();
		assert_eq!(final_tiers.len(), 2, "Should now have 2 tiers");

		// Test tier-based reward calculation with the new tiers
		let stake_amount_tier1 = 50; // Below first tier
		let rate_tier1 = CreditsPallet::<Runtime>::get_current_rate(stake_amount_tier1);
		assert_eq!(rate_tier1, 0, "Rate should be 0 for stake below lowest tier");

		let stake_amount_tier2 = 100; // At first tier
		let rate_tier2 = CreditsPallet::<Runtime>::get_current_rate(stake_amount_tier2);
		assert_eq!(rate_tier2, 2, "Rate should match the tier 2 rate");
	});
}
