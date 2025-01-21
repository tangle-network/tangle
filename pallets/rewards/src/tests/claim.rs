use super::*;
use crate::{Event, UserClaimedReward};
use frame_support::assert_ok;
use frame_support::traits::Currency;
use sp_runtime::Percent;
use tangle_primitives::rewards::{LockInfo, LockMultiplier, UserDepositWithLocks};

fn run_to_block(n: u64) {
	while System::block_number() < n {
		System::set_block_number(System::block_number() + 1);
	}
}

#[test]
fn test_claim_rewards_with_invalid_asset() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(vault_id as u128);

		// Try to claim rewards for an asset that doesn't exist in the vault
		assert_noop!(
			RewardsPallet::<Runtime>::claim_rewards(RuntimeOrigin::signed(account.clone()), asset),
			Error::<Runtime>::AssetNotInVault
		);

		// Configure the reward vault
		assert_ok!(RewardsPallet::<Runtime>::update_vault_reward_config(
			RuntimeOrigin::root(),
			vault_id,
			RewardConfigForAssetVault {
				apy: Percent::from_percent(10),
				deposit_cap: 1_000_000_000_000_000_000_000u128,
				incentive_cap: 1_000_000_000_000_000_000_000u128,
				boost_multiplier: Some(150),
			}
		));

		// Add asset to vault
		assert_ok!(RewardsPallet::<Runtime>::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			asset,
			AssetAction::Add,
		));

		// Try to claim rewards without any deposit
		assert_noop!(
			RewardsPallet::<Runtime>::claim_rewards(RuntimeOrigin::signed(account.clone()), asset),
			Error::<Runtime>::NoRewardsAvailable
		);
	});
}

#[test]
fn test_claim_rewards_with_no_deposit() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(vault_id as u128);

		// Configure the reward vault
		assert_ok!(RewardsPallet::<Runtime>::update_vault_reward_config(
			RuntimeOrigin::root(),
			vault_id,
			RewardConfigForAssetVault {
				apy: Percent::from_percent(10),
				deposit_cap: 1_000_000_000_000_000_000_000u128,
				incentive_cap: 1_000_000_000_000_000_000u128,
				boost_multiplier: Some(1),
			}
		));

		// Add asset to vault
		assert_ok!(RewardsPallet::<Runtime>::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			asset,
			AssetAction::Add,
		));

		// Try to claim rewards without any deposit
		assert_noop!(
			RewardsPallet::<Runtime>::claim_rewards(RuntimeOrigin::signed(account.clone()), asset),
			Error::<Runtime>::NoRewardsAvailable
		);
	});
}

#[test]
fn test_claim_rewards() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(vault_id as u128);
		let deposit = 100_000_000_000_000_000_000u128;
		let apy = Percent::from_percent(10);
		let deposit_cap = 1_000_000_000_000_000_000_000u128;
		let boost_multiplier = Some(1);
		let incentive_cap = 1_000_000_000_000_000_000u128;

		// Fund rewards pallet with initial balance
		let rewards_account = RewardsPallet::<Runtime>::account_id();
		let initial_funding = 1_000_000_000_000_000_000_000_000u128; // 1M tokens
		Balances::make_free_balance_be(&rewards_account, initial_funding);

		// Configure the reward vault
		assert_ok!(RewardsPallet::<Runtime>::update_vault_reward_config(
			RuntimeOrigin::root(),
			vault_id,
			RewardConfigForAssetVault { apy, deposit_cap, incentive_cap, boost_multiplier }
		));

		// Add asset to vault
		assert_ok!(RewardsPallet::<Runtime>::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			asset,
			AssetAction::Add,
		));

		// Mock deposit with UserDepositWithLocks
		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert(
				(account.clone(), asset),
				UserDepositWithLocks { unlocked_amount: deposit, amount_with_locks: None },
			);
		});

		// Initial balance should be 0
		assert_eq!(Balances::free_balance(&account), 0);

		// Claim rewards
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards(
			RuntimeOrigin::signed(account.clone()),
			asset
		));

		// Balance should be greater than 0 after claiming rewards
		assert!(Balances::free_balance(&account) > 0);

		// Check events
		System::assert_has_event(
			Event::RewardsClaimed {
				account: account.clone(),
				asset,
				amount: Balances::free_balance(&account),
			}
			.into(),
		);

		// Check storage
		assert!(UserClaimedReward::<Runtime>::contains_key(&account, vault_id));
	});
}

#[test]
fn test_claim_rewards_multiple_times() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(vault_id as u128);
		let deposit = 100_000_000_000_000_000_000u128;

		// Fund rewards pallet with initial balance
		let rewards_account = RewardsPallet::<Runtime>::account_id();
		let initial_funding = 1_000_000_000_000_000_000_000_000u128; // 1M tokens
		Balances::make_free_balance_be(&rewards_account, initial_funding);

		// Configure the reward vault
		assert_ok!(RewardsPallet::<Runtime>::update_vault_reward_config(
			RuntimeOrigin::root(),
			vault_id,
			RewardConfigForAssetVault {
				apy: Percent::from_percent(10),
				deposit_cap: 1_000_000_000_000_000_000_000u128,
				incentive_cap: 1_000_000_000_000_000_000u128,
				boost_multiplier: Some(1),
			}
		));

		// Add asset to vault
		assert_ok!(RewardsPallet::<Runtime>::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			asset,
			AssetAction::Add,
		));

		// Mock deposit
		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert(
				(account.clone(), asset),
				UserDepositWithLocks {
					unlocked_amount: deposit,
					amount_with_locks: Some(vec![LockInfo {
						amount: deposit,
						expiry_block: 3000_u64,
						lock_multiplier: LockMultiplier::SixMonths,
					}]),
				},
			);
		});

		// Initial balance should be 0
		assert_eq!(Balances::free_balance(&account), 0);

		// Run some blocks to accumulate initial rewards
		run_to_block(100);

		// Claim rewards first time
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards(
			RuntimeOrigin::signed(account.clone()),
			asset
		));

		let first_claim_balance = Balances::free_balance(&account);
		assert!(first_claim_balance > 0);

		// Run more blocks to accumulate more rewards
		run_to_block(1000);

		// Claim rewards second time
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards(
			RuntimeOrigin::signed(account.clone()),
			asset
		));
	});
}

#[test]
fn test_calculate_deposit_rewards_with_lock_multiplier() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(vault_id as u128);
		let deposit = 100_000_000_000_000_000_000u128;
		let apy = Percent::from_percent(10);
		let deposit_cap = 1_000_000_000_000_000_000_000u128;
		let boost_multiplier = Some(1);
		let incentive_cap = 1_000_000_000_000_000_000u128;

		// Fund rewards pallet with initial balance
		let rewards_account = RewardsPallet::<Runtime>::account_id();
		let initial_funding = 1_000_000_000_000_000_000_000_000u128; // 1M tokens
		Balances::make_free_balance_be(&rewards_account, initial_funding);

		// Configure the reward vault
		assert_ok!(RewardsPallet::<Runtime>::update_vault_reward_config(
			RuntimeOrigin::root(),
			vault_id,
			RewardConfigForAssetVault { apy, deposit_cap, incentive_cap, boost_multiplier }
		));

		// Add asset to vault
		assert_ok!(RewardsPallet::<Runtime>::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			asset,
			AssetAction::Add,
		));

		// Mock deposit with locked amounts
		let lock_expiry = 3000_u64;
		let lock_info = LockInfo {
			amount: deposit,
			expiry_block: lock_expiry,
			lock_multiplier: LockMultiplier::SixMonths,
		};

		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert(
				(account.clone(), asset),
				UserDepositWithLocks {
					unlocked_amount: deposit,
					amount_with_locks: Some(vec![lock_info.clone()]),
				},
			);
		});

		// Calculate rewards with no previous claim
		let total_score = BalanceOf::<Runtime>::from(200_000_000_000_000_000_000u128);
		let deposit_info = UserDepositWithLocks {
			unlocked_amount: deposit,
			amount_with_locks: Some(vec![lock_info]),
		};
		let reward_config =
			RewardConfigForAssetVault { apy, deposit_cap, incentive_cap, boost_multiplier };
		let last_claim = None;

		let total_rewards =
			RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
				total_score,
				total_score,
				deposit_info.clone(),
				reward_config.clone(),
				last_claim,
			)
			.unwrap();

		// Verify rewards are greater than 0
		assert!(total_rewards > 0);

		// Test with previous claim
		let previous_claim_amount = total_rewards / 2;
		let last_claim = Some((1u64, previous_claim_amount));

		let total_rewards_2 =
			RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
				total_score,
				total_score,
				deposit_info,
				reward_config,
				last_claim,
			)
			.unwrap();

		// Verify rewards calculation with previous claim
		assert_eq!(total_rewards, total_rewards_2);
	});
}

#[test]
fn test_calculate_deposit_rewards_with_expired_locks() {
	new_test_ext().execute_with(|| {
		let vault_id = 1u32;
		let asset = Asset::Custom(vault_id as u128);
		let deposit = 100_000_000_000_000_000_000u128;
		let apy = Percent::from_percent(10);
		let deposit_cap = 1_000_000_000_000_000_000_000u128;
		let boost_multiplier = Some(1);
		let incentive_cap = 1_000_000_000_000_000_000u128;

		// Fund rewards pallet with initial balance
		let rewards_account = RewardsPallet::<Runtime>::account_id();
		let initial_funding = 1_000_000_000_000_000_000_000_000u128; // 1M tokens
		Balances::make_free_balance_be(&rewards_account, initial_funding);

		// Configure the reward vault
		assert_ok!(RewardsPallet::<Runtime>::update_vault_reward_config(
			RuntimeOrigin::root(),
			vault_id,
			RewardConfigForAssetVault { apy, deposit_cap, incentive_cap, boost_multiplier }
		));

		// Add asset to vault
		assert_ok!(RewardsPallet::<Runtime>::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			asset,
			AssetAction::Add,
		));

		let total_score = BalanceOf::<Runtime>::from(200_000_000_000_000_000_000u128);
		let reward_config =
			RewardConfigForAssetVault { apy, deposit_cap, incentive_cap, boost_multiplier };

		// Test with expired lock
		let expired_lock = LockInfo {
			amount: deposit,
			expiry_block: 50_u64, // Expired block
			lock_multiplier: LockMultiplier::SixMonths,
		};

		let deposit_info = UserDepositWithLocks {
			unlocked_amount: deposit,
			amount_with_locks: Some(vec![expired_lock]),
		};

		// Run to block after expiry
		run_to_block(100);

		let total_rewards =
			RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
				total_score,
				total_score,
				deposit_info,
				reward_config,
				None,
			)
			.unwrap();

		// Verify only base rewards are calculated (no lock multiplier)
		assert!(total_rewards > 0);
	});
}
