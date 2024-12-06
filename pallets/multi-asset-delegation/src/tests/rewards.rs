use super::mock::*;
use crate::{
	types::{LockMultiplier, RestakeDepositScore, RewardConfig, RewardConfigForAssetVault},
	Error, RewardConfigStorage, RewardVaults,
};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_runtime::Percent;
use sp_std::collections::btree_map::BTreeMap;

const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;
const VAULT_ID: u32 = 1;
const TNT_ASSET_ID: u32 = 0;
const DOT_ASSET_ID: u32 = 1;
const INITIAL_BALANCE: u64 = 1_000_000;

fn setup_reward_config() {
	let mut configs = BTreeMap::new();
	configs.insert(
		VAULT_ID,
		RewardConfigForAssetVault {
			apy: Percent::from_percent(10),
			cap: 1_000_000,
			tnt_boost_multiplier: 2,
		},
	);

	let reward_config = RewardConfig {
		configs,
		whitelisted_blueprint_ids: vec![1],
		lock_multipliers: vec![
			LockMultiplier { lock_months: 1, multiplier: 1 },
			LockMultiplier { lock_months: 3, multiplier: 2 },
			LockMultiplier { lock_months: 6, multiplier: 3 },
			LockMultiplier { lock_months: 12, multiplier: 4 },
		],
	};

	RewardConfigStorage::<Test>::put(reward_config);
}

fn setup_test_ext_with_rewards() -> sp_io::TestExternalities {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		// Fund accounts
		let _ = Balances::deposit_creating(&ALICE, INITIAL_BALANCE);
		let _ = Balances::deposit_creating(&BOB, INITIAL_BALANCE);
		let _ = Balances::deposit_creating(&CHARLIE, INITIAL_BALANCE);

		// Setup reward config
		setup_reward_config();

		// Add assets to vault
		assert_ok!(MultiAssetDelegation::add_asset_to_vault(&VAULT_ID, &TNT_ASSET_ID));
		assert_ok!(MultiAssetDelegation::add_asset_to_vault(&VAULT_ID, &DOT_ASSET_ID));

		// Set APY and cap
		assert_ok!(MultiAssetDelegation::set_incentive_apy_and_cap(
			RuntimeOrigin::root(),
			VAULT_ID,
			Percent::from_percent(10),
			1_000_000,
		));
	});
	ext
}

#[test]
fn test_reward_distribution_and_claiming() {
	setup_test_ext_with_rewards().execute_with(|| {
		// Setup initial stake
		assert_ok!(MultiAssetDelegation::set_lock_period(
			RuntimeOrigin::signed(ALICE),
			TNT_ASSET_ID,
			3
		));

		crate::RestakeDepositScore::<Test>::insert(
			&ALICE,
			TNT_ASSET_ID,
			RestakeDepositScore {
				base_score: 1000,
				lock_multiplier: 2,
				expiry: 1000,
				auto_compound: false,
			},
		);

		// Advance some blocks to accumulate rewards
		run_to_block(10);

		// Distribute rewards
		assert_ok!(MultiAssetDelegation::distribute_rewards(1));

		// Log the pending rewards for debugging
		let pending = MultiAssetDelegation::pending_rewards(&ALICE, TNT_ASSET_ID);
		println!("Pending rewards: {}", pending);
		assert!(pending > 0, "Expected non-zero pending rewards");

		// Claim rewards
		let initial_balance = Balances::free_balance(&ALICE);
		assert_ok!(MultiAssetDelegation::claim_rewards(RuntimeOrigin::signed(ALICE), TNT_ASSET_ID));

		let final_balance = Balances::free_balance(&ALICE);
		println!("Balance change: {} -> {}", initial_balance, final_balance);

		assert_eq!(MultiAssetDelegation::pending_rewards(&ALICE, TNT_ASSET_ID), 0);
		assert!(final_balance > initial_balance);
	});
}

#[test]
fn test_multiple_users_reward_distribution() {
	setup_test_ext_with_rewards().execute_with(|| {
		// Setup stakes for multiple users with different lock periods
		for (who, lock_months) in [(ALICE, 1), (BOB, 3), (CHARLIE, 12)].iter() {
			assert_ok!(MultiAssetDelegation::set_lock_period(
				RuntimeOrigin::signed(*who),
				TNT_ASSET_ID,
				*lock_months
			));

			let multiplier = match lock_months {
				1 => 1,
				3 => 2,
				12 => 4,
				_ => unreachable!(),
			};

			crate::RestakeDepositScore::<Test>::insert(
				who,
				TNT_ASSET_ID,
				RestakeDepositScore {
					base_score: 1000,
					lock_multiplier: multiplier,
					expiry: 1000,
					auto_compound: false,
				},
			);
		}

		run_to_block(10);
		assert_ok!(MultiAssetDelegation::distribute_rewards(1));

		// Check rewards are proportional to lock periods
		let alice_rewards = MultiAssetDelegation::pending_rewards(&ALICE, TNT_ASSET_ID);
		let bob_rewards = MultiAssetDelegation::pending_rewards(&BOB, TNT_ASSET_ID);
		let charlie_rewards = MultiAssetDelegation::pending_rewards(&CHARLIE, TNT_ASSET_ID);

		println!(
			"Rewards - Alice: {}, Bob: {}, Charlie: {}",
			alice_rewards, bob_rewards, charlie_rewards
		);

		assert!(bob_rewards > alice_rewards, "3-month lock should yield more rewards than 1-month");
		assert!(
			charlie_rewards > bob_rewards,
			"12-month lock should yield more rewards than 3-month"
		);
	});
}

#[test]
fn test_lock_period_setting() {
	new_test_ext().execute_with(|| {
		// Try setting invalid lock period
		assert_noop!(
			MultiAssetDelegation::set_lock_period(RuntimeOrigin::signed(ALICE), TNT_ASSET_ID, 2),
			Error::<Test>::InvalidLockPeriod
		);

		// Set valid lock period
		assert_ok!(MultiAssetDelegation::set_lock_period(
			RuntimeOrigin::signed(ALICE),
			TNT_ASSET_ID,
			3
		));

		// Verify restake deposit score
		let points = MultiAssetDelegation::restake_deposit_score(&ALICE, TNT_ASSET_ID).unwrap();
		assert_eq!(points.lock_multiplier, 2); // 3 months = 2x multiplier
	});
}

#[test]
fn test_auto_compound_toggle() {
	new_test_ext().execute_with(|| {
		// Toggle auto-compound
		assert_ok!(MultiAssetDelegation::toggle_auto_compound(
			RuntimeOrigin::signed(ALICE),
			TNT_ASSET_ID
		));

		// Verify auto-compound is enabled
		let points = MultiAssetDelegation::restake_deposit_score(&ALICE, TNT_ASSET_ID).unwrap();
		assert!(points.auto_compound);

		// Toggle again
		assert_ok!(MultiAssetDelegation::toggle_auto_compound(
			RuntimeOrigin::signed(ALICE),
			TNT_ASSET_ID
		));

		// Verify auto-compound is disabled
		let points = MultiAssetDelegation::restake_deposit_score(&ALICE, TNT_ASSET_ID).unwrap();
		assert!(!points.auto_compound);
	});
}

#[test]
fn test_tnt_boost_multiplier() {
	setup_test_ext_with_rewards().execute_with(|| {
		// Set TNT boost multiplier
		assert_ok!(MultiAssetDelegation::set_tnt_boost_multiplier(
			RuntimeOrigin::root(),
			VAULT_ID,
			3
		));

		// Verify multiplier is set
		let config = MultiAssetDelegation::reward_config().unwrap();
		let vault_config = config.configs.get(&VAULT_ID).unwrap();
		assert_eq!(vault_config.tnt_boost_multiplier, 3);

		// Non-root cannot set multiplier
		assert_noop!(
			MultiAssetDelegation::set_tnt_boost_multiplier(
				RuntimeOrigin::signed(ALICE),
				VAULT_ID,
				4
			),
			sp_runtime::DispatchError::BadOrigin
		);
	});
}

#[test]
fn test_auto_compound_rewards() {
	new_test_ext().execute_with(|| {
		// Setup restake deposit score with auto-compound enabled
		assert_ok!(MultiAssetDelegation::toggle_auto_compound(
			RuntimeOrigin::signed(ALICE),
			TNT_ASSET_ID
		));

		crate::RestakeDepositScore::<Test>::insert(
			&ALICE,
			TNT_ASSET_ID,
			RestakeDepositScore {
				base_score: 1000,
				lock_multiplier: 1,
				expiry: 1000,
				auto_compound: true,
			},
		);

		// Distribute rewards
		assert_ok!(MultiAssetDelegation::distribute_rewards(1));

		// Verify rewards are auto-compounded
		let points = MultiAssetDelegation::restake_deposit_score(&ALICE, TNT_ASSET_ID).unwrap();
		assert!(points.base_score > 1000);
		assert_eq!(MultiAssetDelegation::pending_rewards(&ALICE, TNT_ASSET_ID), 0);
	});
}

#[test]
fn test_claim_all_rewards() {
	new_test_ext().execute_with(|| {
		// Setup restake deposit score for multiple assets
		crate::RestakeDepositScore::<Test>::insert(
			&ALICE,
			TNT_ASSET_ID,
			RestakeDepositScore {
				base_score: 1000,
				lock_multiplier: 1,
				expiry: 1000,
				auto_compound: false,
			},
		);

		crate::RestakeDepositScore::<Test>::insert(
			&ALICE,
			DOT_ASSET_ID,
			RestakeDepositScore {
				base_score: 1000,
				lock_multiplier: 1,
				expiry: 1000,
				auto_compound: false,
			},
		);

		// Distribute rewards
		assert_ok!(MultiAssetDelegation::distribute_rewards(1));

		// Verify pending rewards exist for both assets
		assert!(MultiAssetDelegation::pending_rewards(&ALICE, TNT_ASSET_ID) > 0);
		assert!(MultiAssetDelegation::pending_rewards(&ALICE, DOT_ASSET_ID) > 0);

		// Claim all rewards
		let initial_balance = Balances::free_balance(&ALICE);
		assert_ok!(MultiAssetDelegation::claim_all_rewards(RuntimeOrigin::signed(ALICE)));

		// Verify all rewards are claimed
		assert_eq!(MultiAssetDelegation::pending_rewards(&ALICE, TNT_ASSET_ID), 0);
		assert_eq!(MultiAssetDelegation::pending_rewards(&ALICE, DOT_ASSET_ID), 0);
		assert!(Balances::free_balance(&ALICE) > initial_balance);
	});
}

#[test]
fn test_expired_points() {
	new_test_ext().execute_with(|| {
		// Setup expired restake deposit score
		crate::RestakeDepositScore::<Test>::insert(
			&ALICE,
			TNT_ASSET_ID,
			RestakeDepositScore {
				base_score: 1000,
				lock_multiplier: 1,
				expiry: 0, // Already expired
				auto_compound: false,
			},
		);

		// Distribute rewards
		assert_ok!(MultiAssetDelegation::distribute_rewards(1));

		// Verify no rewards are distributed for expired points
		assert_eq!(MultiAssetDelegation::pending_rewards(&ALICE, TNT_ASSET_ID), 0);
	});
}

#[test]
fn test_no_rewards_to_claim() {
	new_test_ext().execute_with(|| {
		// Try claiming with no rewards
		assert_noop!(
			MultiAssetDelegation::claim_rewards(RuntimeOrigin::signed(ALICE), TNT_ASSET_ID),
			Error::<Test>::NoRewardsToClaim
		);

		assert_noop!(
			MultiAssetDelegation::claim_all_rewards(RuntimeOrigin::signed(ALICE)),
			Error::<Test>::NoRewardsToClaim
		);
	});
}

#[test]
fn test_asset_to_score_conversion() {
	new_test_ext().execute_with(|| {
		// Test basic conversion
		assert_eq!(
			MultiAssetDelegation::asset_to_score(1000),
			1000,
			"Basic point conversion should return same value for now"
		);

		// Test zero conversion
		assert_eq!(
			MultiAssetDelegation::asset_to_score(0),
			0,
			"Zero asset amount should convert to zero points"
		);

		// Test large number conversion
		let large_amount = 1_000_000_000;
		assert_eq!(
			MultiAssetDelegation::asset_to_score(large_amount),
			large_amount,
			"Large numbers should convert correctly"
		);
	});
}

#[test]
fn test_reward_cap_enforcement() {
	setup_test_ext_with_rewards().execute_with(|| {
		// Set a low cap
		assert_ok!(MultiAssetDelegation::set_incentive_apy_and_cap(
			RuntimeOrigin::root(),
			VAULT_ID,
			Percent::from_percent(10),
			2000, // Low cap
		));

		// Setup stakes exceeding cap
		for who in [ALICE, BOB].iter() {
			crate::RestakeDepositScore::<Test>::insert(
				who,
				TNT_ASSET_ID,
				RestakeDepositScore {
					base_score: 2000, // Each user stakes the cap amount
					lock_multiplier: 1,
					expiry: 1000,
					auto_compound: false,
				},
			);
		}

		run_to_block(10);
		assert_ok!(MultiAssetDelegation::distribute_rewards(1));

		let alice_rewards = MultiAssetDelegation::pending_rewards(&ALICE, TNT_ASSET_ID);
		let bob_rewards = MultiAssetDelegation::pending_rewards(&BOB, TNT_ASSET_ID);

		println!("Capped rewards - Alice: {}, Bob: {}", alice_rewards, bob_rewards);

		// Total rewards should not exceed cap-based calculation
		let total_rewards = alice_rewards.saturating_add(bob_rewards);
		let max_rewards = 2000u64.saturating_mul(10) / 100; // 10% APY
		assert!(total_rewards <= max_rewards, "Total rewards exceeded cap");
	});
}

// Add helper function for block progression
fn run_to_block(n: u64) {
	while frame_system::Pallet::<Test>::block_number() < n {
		frame_system::Pallet::<Test>::set_block_number(
			frame_system::Pallet::<Test>::block_number() + 1,
		);
	}
}
