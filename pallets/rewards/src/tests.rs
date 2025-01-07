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
// along with Tangle. If not, see <http://www.gnu.org/licenses/>.
use crate::{
	mock::*, types::*, AssetAction, Error, Event, Pallet as RewardsPallet, UserClaimedReward,
};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{AccountId32, Percent};
use tangle_primitives::types::rewards::LockInfo;
use tangle_primitives::types::rewards::LockMultiplier;
use tangle_primitives::{
	services::Asset,
	types::rewards::{UserDepositWithLocks, UserRewards},
};

fn run_to_block(n: u64) {
	while System::block_number() < n {
		System::set_block_number(System::block_number() + 1);
	}
}

#[test]
fn test_claim_rewards() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(vault_id as u128);
		let deposit = 100;
		let apy = Percent::from_percent(10);
		let deposit_cap = 1000;
		let boost_multiplier = Some(150);
		let incentive_cap = 1000;

		// Configure the reward vault
		assert_ok!(RewardsPallet::<Runtime>::udpate_vault_reward_config(
			RuntimeOrigin::root(),
			vault_id,
			RewardConfigForAssetVault { apy, deposit_cap, incentive_cap, boost_multiplier }
		));

		// Add asset to vault
		assert_ok!(RewardsPallet::<Runtime>::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			asset.clone(),
			AssetAction::Add,
		));

		// Mock deposit with UserDepositWithLocks
		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert(
				(account.clone(), asset.clone()),
				UserDepositWithLocks { unlocked_amount: deposit, amount_with_locks: None },
			);
		});

		// Initial balance should be 0
		assert_eq!(Balances::free_balance(&account), 0);

		// Claim rewards
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards(
			RuntimeOrigin::signed(account.clone()),
			asset.clone()
		));

		// Balance should be greater than 0 after claiming rewards
		assert!(Balances::free_balance(&account) > 0);

		// Check events
		System::assert_has_event(
			Event::RewardsClaimed {
				account: account.clone(),
				asset: asset.clone(),
				amount: Balances::free_balance(&account),
			}
			.into(),
		);

		// Check storage
		assert!(UserClaimedReward::<Runtime>::contains_key(&account, vault_id));
	});
}

#[test]
fn test_claim_rewards_with_invalid_asset() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(vault_id as u128);

		// Try to claim rewards for an asset that doesn't exist in the vault
		assert_noop!(
			RewardsPallet::<Runtime>::claim_rewards(
				RuntimeOrigin::signed(account.clone()),
				asset.clone()
			),
			Error::<Runtime>::AssetNotInVault
		);

		// Configure the reward vault
		assert_ok!(RewardsPallet::<Runtime>::udpate_vault_reward_config(
			RuntimeOrigin::root(),
			vault_id,
			RewardConfigForAssetVault {
				apy: Percent::from_percent(10),
				deposit_cap: 1000,
				incentive_cap: 1000,
				boost_multiplier: Some(150),
			}
		));

		// Add asset to vault
		assert_ok!(RewardsPallet::<Runtime>::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			asset.clone(),
			AssetAction::Add,
		));

		// Try to claim rewards without any deposit
		assert_noop!(
			RewardsPallet::<Runtime>::claim_rewards(
				RuntimeOrigin::signed(account.clone()),
				asset.clone()
			),
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
		assert_ok!(RewardsPallet::<Runtime>::udpate_vault_reward_config(
			RuntimeOrigin::root(),
			vault_id,
			RewardConfigForAssetVault {
				apy: Percent::from_percent(10),
				deposit_cap: 1000,
				incentive_cap: 1000,
				boost_multiplier: Some(150),
			}
		));

		// Add asset to vault
		assert_ok!(RewardsPallet::<Runtime>::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			asset.clone(),
			AssetAction::Add,
		));

		// Try to claim rewards without any deposit
		assert_noop!(
			RewardsPallet::<Runtime>::claim_rewards(
				RuntimeOrigin::signed(account.clone()),
				asset.clone()
			),
			Error::<Runtime>::NoRewardsAvailable
		);
	});
}

#[test]
fn test_claim_rewards_multiple_times() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(vault_id as u128);
		let deposit = 100;

		// Configure the reward vault
		assert_ok!(RewardsPallet::<Runtime>::udpate_vault_reward_config(
			RuntimeOrigin::root(),
			vault_id,
			RewardConfigForAssetVault {
				apy: Percent::from_percent(10),
				deposit_cap: 1000,
				incentive_cap: 1000,
				boost_multiplier: Some(150),
			}
		));

		// Add asset to vault
		assert_ok!(RewardsPallet::<Runtime>::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			asset.clone(),
			AssetAction::Add,
		));

		// Mock deposit
		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert(
				(account.clone(), asset.clone()),
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
			asset.clone()
		));

		let first_claim_balance = Balances::free_balance(&account);
		assert!(first_claim_balance > 0);

		// Run more blocks to accumulate more rewards
		run_to_block(1000);

		// Claim rewards second time
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards(
			RuntimeOrigin::signed(account.clone()),
			asset.clone()
		));
	});
}
