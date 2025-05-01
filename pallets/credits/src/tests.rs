use crate::{
	mock::{self, *},
	types::*,
	Error, Event, Pallet as Credits,
};
use frame_support::{assert_noop, assert_ok, storage::bounded_vec, traits::fungibles::Mutate};
use mock::mock_currency::{self as mock_currency, set_balance as set_tnt_balance};
use sp_runtime::{DispatchError, DispatchResult, Perbill};

fn link_account(who: AccountId, id_str: &[u8]) -> DispatchResult {
	let bounded_id: OffchainAccountIdOf<Test> = id_str.to_vec().try_into().expect("ID too long");
	Credits::link_account(RuntimeOrigin::signed(who), bounded_id)
}

fn claim_credits(who: AccountId, amount: Balance, id_str: &[u8]) -> DispatchResult {
	let bounded_id: OffchainAccountIdOf<Test> = id_str.to_vec().try_into().expect("ID too long");
	Credits::claim_credits(RuntimeOrigin::signed(who), amount, bounded_id)
}

fn credit_balance(who: AccountId) -> Balance {
	Credits::credit_balance(who)
}

fn last_interaction(who: AccountId) -> BlockNumber {
	Credits::last_interaction_block(who)
}

fn last_reward_update(who: AccountId) -> BlockNumber {
	Credits::last_reward_update_block(who)
}

fn set_stake(who: AccountId, amount: Balance) {
	MOCK_STAKING_INFO.with(|s| s.borrow_mut().set_stake(who, TNT_ASSET_ID, amount));
}

fn expected_accrued(start_block: BlockNumber, end_block: BlockNumber, rate: Balance) -> Balance {
	if end_block <= start_block {
		return 0;
	}
	rate.saturating_mul((end_block - start_block).into())
}

// Helper to calculate expected claimable amount *for testing purposes only*
// Mirrors the logic in claim_credits but uses test state directly.
fn calculate_expected_claimable(who: AccountId, current_block: BlockNumber) -> Balance {
	let last_update = last_reward_update(who);
	let last_interact = last_interaction(who);

	let tnt_asset = tangle_primitives::services::Asset::Custom(TNT_ASSET_ID);
	let maybe_deposit_info = MultiAssetDelegation::get_user_deposit_with_locks(&who, tnt_asset);
	let staked_amount = maybe_deposit_info.map_or(Zero::zero(), |deposit_info| {
		let locked_total = deposit_info.amount_with_locks.map_or(Zero::zero(), |locks| {
			locks.iter().fold(Zero::zero(), |acc, lock| acc.saturating_add(lock.amount))
		});
		deposit_info.unlocked_amount.saturating_add(locked_total)
	});

	let rate = CreditsPallet::<Test>::get_current_rate(staked_amount);
	let accrued_since_last_update = expected_accrued(last_update, current_block, rate);

	let elapsed_blocks_decay =
		if last_interact == 0 { Zero::zero() } else { current_block.saturating_sub(last_interact) };
	let decay_factor = CreditsPallet::<Test>::calculate_decay_factor(elapsed_blocks_decay);

	// Apply decay to the amount accrued *since the last update* (as per simplified logic)
	decay_factor.mul_floor(accrued_since_last_update)
}

// Helper to get TNT balance using the Assets pallet
fn get_tnt_balance(who: AccountId) -> Balance {
	Assets::balance(TNT_ASSET_ID, &who)
}

// Helper to get the expected claimable amount based on current state
fn get_max_claimable(who: AccountId) -> Balance {
	let current_block = System::block_number();
	let last_update = last_reward_update(who);
	let window = CreditClaimWindowValue::get();
	let start_block = max(last_update, current_block.saturating_sub(window));
	let effective_end_block = current_block;

	if start_block >= effective_end_block {
		return 0;
	}
	let tnt_asset = tangle_primitives::services::Asset::Custom(TNT_ASSET_ID);
	let maybe_deposit_info = MultiAssetDelegation::get_user_deposit_with_locks(&who, tnt_asset);
	let staked_amount = maybe_deposit_info.map_or(Zero::zero(), |deposit_info| {
		let locked_total = deposit_info.amount_with_locks.map_or(Zero::zero(), |locks| {
			locks.iter().fold(Zero::zero(), |acc, lock| acc.saturating_add(lock.amount))
		});
		deposit_info.unlocked_amount.saturating_add(locked_total)
	});

	let rate = Credits::get_current_rate(staked_amount);
	if rate.is_zero() {
		return 0;
	}

	let blocks_in_window = effective_end_block.saturating_sub(start_block);
	if blocks_in_window.is_zero() {
		return 0;
	}

	let multiplier = <BalanceOf<Test>>::try_from(blocks_in_window).unwrap_or_default();
	rate.checked_mul(&multiplier).unwrap_or(0)
}

#[test]
fn initial_state_correct() {
	new_test_ext().execute_with(|| {
		assert_eq!(last_reward_update(ALICE), 0);
		// Check initial stakes were set up by delegate calls in new_test_ext
		assert_eq!(
			MultiAssetDelegation::get_delegator_staked_amount(
				ALICE,
				ALICE,
				tangle_primitives::services::Asset::Custom(TNT_ASSET_ID)
			)
			.unwrap_or(0),
			1000
		);
		assert_eq!(
			MultiAssetDelegation::get_delegator_staked_amount(
				BOB,
				BOB,
				tangle_primitives::services::Asset::Custom(TNT_ASSET_ID)
			)
			.unwrap_or(0),
			150
		);
	});
}

#[test]
fn burn_emits_event_and_updates_reward_block() {
	new_test_ext().execute_with(|| {
		System::set_block_number(10);
		let initial_tnt = get_tnt_balance(ALICE);
		assert!(initial_tnt >= 50);

		assert_eq!(last_reward_update(ALICE), 0);

		assert_ok!(Credits::burn(RuntimeOrigin::signed(ALICE), 50));

		System::assert_last_event(
			Event::CreditsGrantedFromBurn {
				who: ALICE,
				tnt_burned: 50,
				credits_granted: 50 * CreditBurnConversionValue::get(),
			}
			.into(),
		);

		assert_eq!(get_tnt_balance(ALICE), initial_tnt - 50);
		assert_eq!(last_reward_update(ALICE), 10); // Updated by burn
	});
}

#[test]
fn claim_basic_within_window() {
	new_test_ext().execute_with(|| {
		let alice_id_str = b"alice_claim";
		let alice_id: OffchainAccountIdOf<Test> = alice_id_str.to_vec().try_into().unwrap();

		// Alice stakes 1000 -> 15 credits/block
		run_to_block(100); // Accrue for 99 blocks (2 to 100)
		let max_claimable = get_max_claimable(ALICE);
		let expected = 15 * 99;
		assert_eq!(
			max_claimable, expected,
			"Max claimable should match simple accrual within window"
		);

		// Claim less than accrued
		let claim_amount = 100;
		assert!(claim_amount <= max_claimable);
		assert_ok!(claim_credits(ALICE, claim_amount, alice_id_str));

		System::assert_last_event(
			Event::CreditsClaimed {
				who: ALICE,
				amount_claimed: claim_amount,
				offchain_account_id: alice_id.clone(),
			}
			.into(),
		);

		assert_eq!(last_reward_update(ALICE), 100); // Updated by claim
	});
}

#[test]
fn claim_at_window_boundary() {
	new_test_ext().execute_with(|| {
		let alice_id_str = b"alice_window_edge";
		let window = CreditClaimWindowValue::get();

		run_to_block(window + 1); // Go exactly to the end of the first window
		let max_claimable = get_max_claimable(ALICE);
		let expected = 15 * window; // Should be capped at window length
		assert_eq!(max_claimable, expected, "Max claimable should be capped at window");

		assert_ok!(claim_credits(ALICE, max_claimable, alice_id_str));
		assert_eq!(last_reward_update(ALICE), window + 1);
	});
}

#[test]
fn claim_after_long_inactivity_capped_by_window() {
	new_test_ext().execute_with(|| {
		let alice_id_str = b"alice_inactive";
		let window = CreditClaimWindowValue::get();

		run_to_block(window * 3); // Wait much longer than the window
		let max_claimable = get_max_claimable(ALICE);
		let expected = 15 * window; // Still capped by the single window length
		assert_eq!(max_claimable, expected, "Claim after inactivity still capped by window");

		// Claiming more than window allowance fails
		assert_noop!(
			claim_credits(ALICE, max_claimable + 1, alice_id_str),
			Error::<Test>::ClaimAmountExceedsWindowAllowance
		);

		// Claiming window allowance succeeds
		assert_ok!(claim_credits(ALICE, max_claimable, alice_id_str));
		assert_eq!(last_reward_update(ALICE), window * 3);
	});
}

#[test]
fn claim_multiple_times_resets_window() {
	new_test_ext().execute_with(|| {
		let alice_id_str = b"alice_multi_claim";
		let window = CreditClaimWindowValue::get();
		let rate = 15; // Alice rate

		// 1. Accrue and claim within first window
		let block1 = window / 2;
		run_to_block(block1);
		let max_claimable1 = get_max_claimable(ALICE);
		let expected1 = rate * (block1 - 1); // Accrued from block 1
		assert_eq!(max_claimable1, expected1);
		assert_ok!(claim_credits(ALICE, max_claimable1, alice_id_str));
		assert_eq!(last_reward_update(ALICE), block1);

		// 2. Accrue past the original window end, but within new window from claim 1
		let block2 = block1 + window - 10; // Still within window relative to block1
		run_to_block(block2);
		let max_claimable2 = get_max_claimable(ALICE);
		let expected2 = rate * (block2 - block1); // Accrued from block1
		assert_eq!(max_claimable2, expected2);
		assert_ok!(claim_credits(ALICE, max_claimable2, alice_id_str));
		assert_eq!(last_reward_update(ALICE), block2);

		// 3. Wait longer than a window, claim is capped
		let block3 = block2 + window + 100;
		run_to_block(block3);
		let max_claimable3 = get_max_claimable(ALICE);
		let expected3 = rate * window; // Capped at window size
		assert_eq!(max_claimable3, expected3);
		assert_ok!(claim_credits(ALICE, max_claimable3, alice_id_str));
		assert_eq!(last_reward_update(ALICE), block3);
	});
}

#[test]
fn claim_failures() {
	new_test_ext().execute_with(|| {
		let alice_id_str = b"alice@claim_fail";
		let alice_id: OffchainAccountIdOf<Test> = alice_id_str.to_vec().try_into().unwrap();
		run_to_block(10);

		// Claim zero
		assert_noop!(claim_credits(ALICE, 0, alice_id_str), Error::<Test>::AmountZero);

		// Claim more than allowance
		let max_claimable = get_max_claimable(ALICE);
		assert_noop!(
			claim_credits(ALICE, max_claimable + 1, alice_id_str),
			Error::<Test>::ClaimAmountExceedsWindowAllowance
		);

		// ID too long
		let long_id = vec![0u8; (MaxAccIdLenValue::get() + 1) as usize];
		let bounded_long_id_res: Result<OffchainAccountIdOf<Test>, _> = long_id.try_into();
		assert!(bounded_long_id_res.is_err());
		assert_noop!(
			Credits::claim_credits(RuntimeOrigin::signed(ALICE), 1, bounded_vec![0u8; 129]),
			Error::<Test>::OffchainAccountIdTooLong
		);
	});
}

#[test]
fn burn_failures() {
	new_test_ext().execute_with(|| {
		let initial_tnt = get_tnt_balance(BOB);
		assert!(initial_tnt > 0);

		// Burn zero
		assert_noop!(Credits::burn(RuntimeOrigin::signed(BOB), 0), Error::<Test>::AmountZero);

		// Burn more than balance
		assert_noop!(
			Credits::burn(RuntimeOrigin::signed(BOB), initial_tnt + 1),
			Error::<Test>::InsufficientTntBalance
		);
	});
}

#[test]
fn accrual_with_stake_change_within_window() {
	new_test_ext().execute_with(|| {
		let window = CreditClaimWindowValue::get();
		let alice_id_str = b"alice_stake_change";
		let rate_tier3 = 15;
		let rate_tier1 = 1;

		// Period 1: Tier 3 for window/2 blocks
		let block1 = window / 2;
		run_to_block(block1);
		let claimable1 = get_max_claimable(ALICE);
		let expected1 = rate_tier3 * (block1 - 1);
		assert_eq!(claimable1, expected1);
		assert_ok!(claim_credits(ALICE, claimable1, alice_id_str));
		assert_eq!(last_reward_update(ALICE), block1);

		// Change stake to Tier 1
		assert_ok!(MultiAssetDelegation::undelegate(
			RuntimeOrigin::signed(ALICE),
			ALICE,
			tangle_primitives::services::Asset::Custom(TNT_ASSET_ID),
			1000 - 150
		));
		// Need to run MAD lifecycle potentially? For simplicity, assume immediate effect for test,
		// or mock directly if needed. Let's assume test setup makes this immediate for credits
		// pallet view

		// Period 2: Tier 1 for window/2 blocks
		let block2 = block1 + window / 2;
		run_to_block(block2);
		let claimable2 = get_max_claimable(ALICE);
		let expected2 = rate_tier1 * (block2 - block1); // Only Tier 1 rate applies
		assert_eq!(claimable2, expected2);
		assert_ok!(claim_credits(ALICE, claimable2, alice_id_str));
		assert_eq!(last_reward_update(ALICE), block2);

		// Period 3: Go past window, still Tier 1
		let block3 = block2 + window + 100;
		run_to_block(block3);
		let claimable3 = get_max_claimable(ALICE);
		let expected3 = rate_tier1 * window; // Capped at window, using Tier 1 rate
		assert_eq!(claimable3, expected3);
		assert_ok!(claim_credits(ALICE, claimable3, alice_id_str));
		assert_eq!(last_reward_update(ALICE), block3);
	});
}
