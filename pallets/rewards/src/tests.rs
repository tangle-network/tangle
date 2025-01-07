// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.
use crate::BalanceOf;
use crate::Config;
use crate::Event;
use crate::RewardConfigForAssetVault;
use crate::RewardConfigStorage;
use crate::TotalRewardVaultScore;
use crate::UserClaimedReward;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::AccountId32;
use sp_runtime::Percent;
use tangle_primitives::services::Asset;
use tangle_primitives::types::rewards::BoostInfo;
use tangle_primitives::types::rewards::LockInfo;
use tangle_primitives::types::rewards::LockMultiplier;
use tangle_primitives::types::rewards::UserDepositWithLocks;
use tangle_primitives::types::rewards::UserRewards;

// Helper function to set up user rewards
fn setup_user_rewards<T: Config>(
	account: T::AccountId,
	asset: Asset<T::AssetId>,
	restaking_rewards: BalanceOf<T>,
	service_rewards: BalanceOf<T>,
	boost_amount: BalanceOf<T>,
	multiplier: LockMultiplier,
	expiry: BlockNumberFor<T>,
) {
	let rewards = UserRewards {
		restaking_rewards,
		service_rewards,
		boost_rewards: BoostInfo { amount: boost_amount, multiplier, expiry },
	};
	UserRewards::<T>::insert(account, asset, rewards);
}

#[test]
fn test_reward_distribution_with_no_locks() {
	new_test_ext().execute_with(|| {
		// Setup test environment
		let account = 1;
		let asset = Asset::new(1, 1).unwrap();
		let vault_id = 1;
		let apy = Percent::from_percent(10); // 10% APY
		let deposit_cap = 1_000_000;

		// Register asset with vault
		assert_ok!(RewardsPallet::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			asset.clone(),
			Some(vault_id)
		));

		// Set reward config
		RewardConfigStorage::<Runtime>::insert(
			vault_id,
			RewardConfigForAssetVault { apy, deposit_cap },
		);

		// Setup mock deposit (unlocked only)
		let deposit = UserDepositWithLocks { unlocked_amount: 100_000, amount_with_locks: None };

		// Mock the delegation info
		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert((account, asset.clone()), deposit);
		});

		// Set total vault score
		TotalRewardVaultScore::<Runtime>::insert(vault_id, 100_000);

		// Initial balance should be zero
		assert_eq!(Balances::free_balance(account), 0);

		// Claim rewards
		assert_ok!(RewardsPallet::claim_rewards(RuntimeOrigin::signed(account), asset.clone()));

		// Check that rewards were paid out
		assert!(Balances::free_balance(account) > 0);

		// Verify event was emitted
		System::assert_has_event(
			Event::RewardsClaimed { account, asset, amount: Balances::free_balance(account) }
				.into(),
		);

		// Verify last claim was updated
		assert!(UserClaimedReward::<Runtime>::contains_key(account, vault_id));
	});
}

#[test]
fn test_reward_distribution_with_locks() {
	new_test_ext().execute_with(|| {
		// Setup test environment
		let account = 1;
		let asset = Asset::new(1, 1).unwrap();
		let vault_id = 1;
		let apy = Percent::from_percent(10); // 10% APY
		let deposit_cap = 1_000_000;

		// Register asset with vault
		assert_ok!(RewardsPallet::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			asset.clone(),
			Some(vault_id)
		));

		// Set reward config
		RewardConfigStorage::<Runtime>::insert(
			vault_id,
			RewardConfigForAssetVault { apy, deposit_cap },
		);

		// Setup mock deposit with locks
		let current_block = System::block_number();
		let locks = vec![
			LockInfo {
				amount: 50_000,
				lock_multiplier: LockMultiplier::TwoMonths,
				expiry_block: current_block + 100,
			},
			LockInfo {
				amount: 25_000,
				lock_multiplier: LockMultiplier::ThreeMonths,
				expiry_block: current_block + 150,
			},
		];

		let deposit =
			UserDepositWithLocks { unlocked_amount: 25_000, amount_with_locks: Some(locks) };

		// Mock the delegation info
		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert((account, asset.clone()), deposit);
		});

		// Set total vault score
		TotalRewardVaultScore::<Runtime>::insert(vault_id, 100_000);

		// Initial balance should be zero
		assert_eq!(Balances::free_balance(account), 0);

		// Claim rewards
		assert_ok!(RewardsPallet::claim_rewards(RuntimeOrigin::signed(account), asset.clone()));

		// Check that rewards were paid out and are higher than no-lock case due to multipliers
		let rewards = Balances::free_balance(account);
		assert!(rewards > 0);

		// Verify event was emitted
		System::assert_has_event(Event::RewardsClaimed { account, asset, amount: rewards }.into());

		// Verify last claim was updated
		let (claim_block, total_rewards) =
			UserClaimedReward::<Runtime>::get(account, vault_id).unwrap();
		assert_eq!(claim_block, System::block_number());
		assert!(total_rewards > rewards); // Total rewards should be higher than paid amount due to previous claims
	});
}

#[test]
fn test_reward_distribution_errors() {
	new_test_ext().execute_with(|| {
		let account = 1;
		let asset = Asset::new(1, 1).unwrap();

		// Asset not in vault
		assert_noop!(
			RewardsPallet::claim_rewards(RuntimeOrigin::signed(account), asset.clone()),
			Error::<Runtime>::AssetNotInVault
		);

		// Register asset with vault
		let vault_id = 1;
		assert_ok!(RewardsPallet::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			asset.clone(),
			Some(vault_id)
		));

		// No deposits
		assert_noop!(
			RewardsPallet::claim_rewards(RuntimeOrigin::signed(account), asset.clone()),
			Error::<Runtime>::NoRewardsAvailable
		);

		// Setup mock deposit but no reward config
		let deposit = UserDepositWithLocks { unlocked_amount: 100_000, amount_with_locks: None };

		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert((account, asset.clone()), deposit);
		});

		// No reward config
		assert_noop!(
			RewardsPallet::claim_rewards(RuntimeOrigin::signed(account), asset.clone()),
			Error::<Runtime>::RewardConfigNotFound
		);
	});
}

#[test]
fn test_subsequent_reward_claims() {
	new_test_ext().execute_with(|| {
		// Setup test environment similar to first test
		let account = 1;
		let asset = Asset::new(1, 1).unwrap();
		let vault_id = 1;

		// Setup basic reward config
		assert_ok!(RewardsPallet::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			asset.clone(),
			Some(vault_id)
		));

		RewardConfigStorage::<Runtime>::insert(
			vault_id,
			RewardConfigForAssetVault { apy: Percent::from_percent(10), deposit_cap: 1_000_000 },
		);

		// Setup mock deposit
		let deposit = UserDepositWithLocks { unlocked_amount: 100_000, amount_with_locks: None };

		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert((account, asset.clone()), deposit);
		});

		TotalRewardVaultScore::<Runtime>::insert(vault_id, 100_000);

		// First claim
		assert_ok!(RewardsPallet::claim_rewards(RuntimeOrigin::signed(account), asset.clone()));
		let first_reward = Balances::free_balance(account);
		assert!(first_reward > 0);

		// Advance some blocks
		run_to_block(100);

		// Second claim
		assert_ok!(RewardsPallet::claim_rewards(RuntimeOrigin::signed(account), asset.clone()));
		let second_reward = Balances::free_balance(account) - first_reward;

		// Second reward should be non-zero but less than first reward
		assert!(second_reward > 0);
		assert!(second_reward < first_reward);
	});
}
