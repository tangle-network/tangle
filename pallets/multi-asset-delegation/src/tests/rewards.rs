use super::mock::*;
use crate::{
	types::{LockMultiplier, RestakeDepositScore, RewardConfig, RewardConfigForAssetVault},
	Error,
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

fn setup_test_ext() -> sp_io::TestExternalities {
	let mut t = new_test_ext();
	t.execute_with(|| {
		// Fund accounts
		let _ = Balances::deposit_creating(&ALICE, INITIAL_BALANCE);
		let _ = Balances::deposit_creating(&BOB, INITIAL_BALANCE);
		let _ = Balances::deposit_creating(&CHARLIE, INITIAL_BALANCE);

		// Setup reward config
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

		MultiAssetDelegation::set_incentive_apy_and_cap(
			RuntimeOrigin::root(),
			VAULT_ID,
			Percent::from_percent(10),
			1_000_000,
		)
		.unwrap();

		// Add assets to vault
		MultiAssetDelegation::add_asset_to_vault(&VAULT_ID, &TNT_ASSET_ID).unwrap();
		MultiAssetDelegation::add_asset_to_vault(&VAULT_ID, &DOT_ASSET_ID).unwrap();
	});
	t
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
		let points = MultiAssetDelegation::stake_points(&ALICE, TNT_ASSET_ID).unwrap();
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
		let points = MultiAssetDelegation::stake_points(&ALICE, TNT_ASSET_ID).unwrap();
		assert!(points.auto_compound);

		// Toggle again
		assert_ok!(MultiAssetDelegation::toggle_auto_compound(
			RuntimeOrigin::signed(ALICE),
			TNT_ASSET_ID
		));

		// Verify auto-compound is disabled
		let points = MultiAssetDelegation::stake_points(&ALICE, TNT_ASSET_ID).unwrap();
		assert!(!points.auto_compound);
	});
}

#[test]
fn test_tnt_boost_multiplier() {
	new_test_ext().execute_with(|| {
		// Set TNT boost multiplier
		assert_ok!(MultiAssetDelegation::set_tnt_boost_multiplier(
			RuntimeOrigin::root(),
			VAULT_ID,
			3
		));

		// Verify multiplier is set
		let config = MultiAssetDelegation::reward_config_storage().unwrap();
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
fn test_reward_distribution_and_claiming() {
	new_test_ext().execute_with(|| {
		// Setup restake deposit score for ALICE
		assert_ok!(MultiAssetDelegation::set_lock_period(
			RuntimeOrigin::signed(ALICE),
			TNT_ASSET_ID,
			3
		));
		MultiAssetDelegation::restake_deposit_score_mut(&ALICE, TNT_ASSET_ID, |points| {
			*points = Some(RestakeDepositScore {
				base_score: 1000,
				lock_multiplier: 2,
				expiry: 1000,
				auto_compound: false,
			});
		});

		// Distribute rewards
		assert_ok!(MultiAssetDelegation::distribute_rewards(1));

		// Check pending rewards
		let pending = MultiAssetDelegation::pending_rewards(&ALICE, TNT_ASSET_ID);
		assert!(pending > 0);

		// Claim rewards
		let initial_balance = Balances::free_balance(&ALICE);
		assert_ok!(MultiAssetDelegation::claim_rewards(RuntimeOrigin::signed(ALICE), TNT_ASSET_ID));

		// Verify rewards are claimed
		assert_eq!(MultiAssetDelegation::pending_rewards(&ALICE, TNT_ASSET_ID), 0);
		assert!(Balances::free_balance(&ALICE) > initial_balance);
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
		MultiAssetDelegation::restake_deposit_score_mut(&ALICE, TNT_ASSET_ID, |points| {
			*points = Some(RestakeDepositScore {
				base_score: 1000,
				lock_multiplier: 1,
				expiry: 1000,
				auto_compound: true,
			});
		});

		// Distribute rewards
		assert_ok!(MultiAssetDelegation::distribute_rewards(1));

		// Verify rewards are auto-compounded
		let points = MultiAssetDelegation::stake_points(&ALICE, TNT_ASSET_ID).unwrap();
		assert!(points.base_score > 1000);
		assert_eq!(MultiAssetDelegation::pending_rewards(&ALICE, TNT_ASSET_ID), 0);
	});
}

#[test]
fn test_claim_all_rewards() {
	new_test_ext().execute_with(|| {
		// Setup restake deposit score for multiple assets
		MultiAssetDelegation::restake_deposit_score_mut(&ALICE, TNT_ASSET_ID, |points| {
			*points = Some(RestakeDepositScore {
				base_score: 1000,
				lock_multiplier: 1,
				expiry: 1000,
				auto_compound: false,
			});
		});
		MultiAssetDelegation::restake_deposit_score_mut(&ALICE, DOT_ASSET_ID, |points| {
			*points = Some(RestakeDepositScore {
				base_score: 1000,
				lock_multiplier: 1,
				expiry: 1000,
				auto_compound: false,
			});
		});

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
		MultiAssetDelegation::restake_deposit_score_mut(&ALICE, TNT_ASSET_ID, |points| {
			*points = Some(RestakeDepositScore {
				base_score: 1000,
				lock_multiplier: 1,
				expiry: 0, // Already expired
				auto_compound: false,
			});
		});

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
fn test_asset_to_points_conversion() {
	new_test_ext().execute_with(|| {
		// Test basic conversion
		assert_eq!(
			MultiAssetDelegation::asset_to_points(1000),
			1000,
			"Basic point conversion should return same value for now"
		);

		// Test zero conversion
		assert_eq!(
			MultiAssetDelegation::asset_to_points(0),
			0,
			"Zero asset amount should convert to zero points"
		);

		// Test large number conversion
		let large_amount = 1_000_000_000;
		assert_eq!(
			MultiAssetDelegation::asset_to_points(large_amount),
			large_amount,
			"Large numbers should convert correctly"
		);
	});
}
