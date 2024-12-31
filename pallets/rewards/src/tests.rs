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
use crate::{mock::*, Error, Pallet as Rewards, RewardType};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::AccountId32;
use tangle_primitives::services::Asset;

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
        boost_rewards: BoostInfo {
            amount: boost_amount,
            multiplier,
            expiry,
        },
    };
    UserRewards::<T>::insert(account, asset, rewards);
}

#[test]
fn test_whitelist_asset() {
    new_test_ext().execute_with(|| {
        let asset = Asset::Custom(0u32);
        let account: AccountId32 = AccountId32::new([1; 32]);

        // Non-root cannot whitelist asset
        assert_noop!(
            Rewards::<Runtime>::whitelist_asset(RuntimeOrigin::signed(account.clone()), asset),
            sp_runtime::DispatchError::BadOrigin
        );

        // Root can whitelist asset
        assert_ok!(Rewards::<Runtime>::whitelist_asset(RuntimeOrigin::root(), asset));
        assert!(Rewards::<Runtime>::is_asset_whitelisted(asset));

        // Cannot whitelist same asset twice
        assert_noop!(
            Rewards::<Runtime>::whitelist_asset(RuntimeOrigin::root(), asset),
            Error::<Runtime>::AssetAlreadyWhitelisted
        );
    });
}

#[test]
fn test_remove_asset() {
    new_test_ext().execute_with(|| {
        let asset = Asset::Custom(0u32);
        let account: AccountId32 = AccountId32::new([1; 32]);

        // Cannot remove non-whitelisted asset
        assert_noop!(
            Rewards::<Runtime>::remove_asset(RuntimeOrigin::root(), asset),
            Error::<Runtime>::AssetNotWhitelisted
        );

        // Whitelist the asset first
        assert_ok!(Rewards::<Runtime>::whitelist_asset(RuntimeOrigin::root(), asset));

        // Non-root cannot remove asset
        assert_noop!(
            Rewards::<Runtime>::remove_asset(RuntimeOrigin::signed(account.clone()), asset),
            sp_runtime::DispatchError::BadOrigin
        );

        // Root can remove asset
        assert_ok!(Rewards::<Runtime>::remove_asset(RuntimeOrigin::root(), asset));
        assert!(!Rewards::<Runtime>::is_asset_whitelisted(asset));

        // Cannot remove already removed asset
        assert_noop!(
            Rewards::<Runtime>::remove_asset(RuntimeOrigin::root(), asset),
            Error::<Runtime>::AssetNotWhitelisted
        );
    });
}

#[test]
fn test_claim_rewards() {
    new_test_ext().execute_with(|| {
        let account: AccountId32 = AccountId32::new([1; 32]);
        let asset = Asset::Custom(0u32);
        let reward_type = RewardType::Restaking;

        // Cannot claim rewards for non-whitelisted asset
        assert_noop!(
            Rewards::<Runtime>::claim_rewards(RuntimeOrigin::signed(account.clone()), asset, reward_type),
            Error::<Runtime>::AssetNotWhitelisted
        );

        // Whitelist the asset
        assert_ok!(Rewards::<Runtime>::whitelist_asset(RuntimeOrigin::root(), asset));

        // Cannot claim rewards when none are available
        assert_noop!(
            Rewards::<Runtime>::claim_rewards(RuntimeOrigin::signed(account.clone()), asset, reward_type),
            Error::<Runtime>::NoRewardsAvailable
        );
    });
}

#[test]
fn test_reward_score_calculation() {
    new_test_ext().execute_with(|| {
        let account: AccountId32 = AccountId32::new([1; 32]);
        let custom_asset = Asset::Custom(0u32);
        let erc20_asset = Asset::Erc20(mock_address(1));
        let reward_type = RewardType::Restaking;

        // Whitelist both assets
        assert_ok!(Rewards::<Runtime>::whitelist_asset(RuntimeOrigin::root(), custom_asset));
        assert_ok!(Rewards::<Runtime>::whitelist_asset(RuntimeOrigin::root(), erc20_asset));

        // Set up rewards for custom asset with boost
        setup_user_rewards::<Runtime>(
            account.clone(),
            custom_asset,
            100u128.into(),  // restaking rewards
            50u128.into(),   // service rewards
            200u128.into(),  // boost amount
            LockMultiplier::ThreeMonths,
            100u64.into(),  // expiry block
        );

        // Set up rewards for ERC20 asset without boost
        setup_user_rewards::<Runtime>(
            account.clone(),
            erc20_asset,
            100u128.into(),  // restaking rewards
            50u128.into(),   // service rewards
            0u128.into(),    // no boost
            LockMultiplier::OneMonth,
            0u64.into(),     // no expiry
        );

        // Test claiming rewards for both assets
        assert_ok!(Rewards::<Runtime>::claim_rewards(
            RuntimeOrigin::signed(account.clone()),
            custom_asset,
            reward_type
        ));

        assert_ok!(Rewards::<Runtime>::claim_rewards(
            RuntimeOrigin::signed(account.clone()),
            erc20_asset,
            reward_type
        ));

        // Verify rewards are cleared after claiming
        let custom_rewards = Rewards::<Runtime>::user_rewards(account.clone(), custom_asset);
        assert_eq!(custom_rewards.restaking_rewards, 0u128.into());

        let erc20_rewards = Rewards::<Runtime>::user_rewards(account.clone(), erc20_asset);
        assert_eq!(erc20_rewards.restaking_rewards, 0u128.into());
    });
}

#[test]
fn test_reward_distribution() {
    new_test_ext().execute_with(|| {
        let account1: AccountId32 = AccountId32::new([1; 32]);
        let account2: AccountId32 = AccountId32::new([2; 32]);
        let asset = Asset::Custom(0u32);
        let reward_type = RewardType::Restaking;

        // Whitelist the asset
        assert_ok!(Rewards::<Runtime>::whitelist_asset(RuntimeOrigin::root(), asset));

        // Set up different rewards for each account
        setup_user_rewards::<Runtime>(
            account1.clone(),
            asset,
            100u128.into(),  // restaking rewards
            50u128.into(),   // service rewards
            200u128.into(),  // boost amount
            LockMultiplier::ThreeMonths,
            100u64.into(),  // expiry block
        );

        setup_user_rewards::<Runtime>(
            account2.clone(),
            asset,
            200u128.into(),  // restaking rewards
            100u128.into(),  // service rewards
            400u128.into(),  // boost amount
            LockMultiplier::SixMonths,
            200u64.into(),  // expiry block
        );

        // Both accounts should be able to claim their rewards
        assert_ok!(Rewards::<Runtime>::claim_rewards(
            RuntimeOrigin::signed(account1.clone()),
            asset,
            reward_type
        ));

        assert_ok!(Rewards::<Runtime>::claim_rewards(
            RuntimeOrigin::signed(account2.clone()),
            asset,
            reward_type
        ));

        // Verify rewards are cleared after claiming
        let account1_rewards = Rewards::<Runtime>::user_rewards(account1, asset);
        assert_eq!(account1_rewards.restaking_rewards, 0u128.into());

        let account2_rewards = Rewards::<Runtime>::user_rewards(account2, asset);
        assert_eq!(account2_rewards.restaking_rewards, 0u128.into());
    });
}

#[test]
fn test_different_reward_types() {
    new_test_ext().execute_with(|| {
        let account: AccountId32 = AccountId32::new([1; 32]);
        let asset = Asset::Custom(0u32);
        
        // Whitelist the asset
        assert_ok!(Rewards::<Runtime>::whitelist_asset(RuntimeOrigin::root(), asset));

        // Set up rewards for different reward types
        setup_user_rewards::<Runtime>(
            account.clone(),
            asset,
            100u128.into(),  // restaking rewards
            150u128.into(),  // service rewards
            200u128.into(),  // boost amount
            LockMultiplier::ThreeMonths,
            100u64.into(),  // expiry block
        );

        // Test claiming each type of reward
        let reward_types = vec![
            RewardType::Restaking,
            RewardType::Service,
            RewardType::Boost,
        ];

        for reward_type in reward_types {
            assert_ok!(Rewards::<Runtime>::claim_rewards(
                RuntimeOrigin::signed(account.clone()),
                asset,
                reward_type
            ));
        }

        // Verify all rewards are cleared
        let rewards = Rewards::<Runtime>::user_rewards(account, asset);
        assert_eq!(rewards.restaking_rewards, 0u128.into());
        assert_eq!(rewards.service_rewards, 0u128.into());
        assert_eq!(rewards.boost_rewards.amount, 0u128.into());
    });
}

#[test]
fn test_multiple_claims() {
    new_test_ext().execute_with(|| {
        let account: AccountId32 = AccountId32::new([1; 32]);
        let asset = Asset::Custom(0u32);
        let reward_type = RewardType::Restaking;

        // Whitelist the asset
        assert_ok!(Rewards::<Runtime>::whitelist_asset(RuntimeOrigin::root(), asset));

        // Set up initial rewards
        setup_user_rewards::<Runtime>(
            account.clone(),
            asset,
            100u128.into(),  // restaking rewards
            50u128.into(),   // service rewards
            0u128.into(),    // no boost
            LockMultiplier::OneMonth,
            0u64.into(),     // no expiry
        );

        // First claim should succeed
        assert_ok!(Rewards::<Runtime>::claim_rewards(
            RuntimeOrigin::signed(account.clone()),
            asset,
            reward_type
        ));

        // Second claim should fail as rewards are cleared
        assert_noop!(
            Rewards::<Runtime>::claim_rewards(RuntimeOrigin::signed(account.clone()), asset, reward_type),
            Error::<Runtime>::NoRewardsAvailable
        );

        // Set up new rewards
        setup_user_rewards::<Runtime>(
            account.clone(),
            asset,
            200u128.into(),  // restaking rewards
            100u128.into(),  // service rewards
            0u128.into(),    // no boost
            LockMultiplier::OneMonth,
            0u64.into(),     // no expiry
        );

        // Should be able to claim again with new rewards
        assert_ok!(Rewards::<Runtime>::claim_rewards(
            RuntimeOrigin::signed(account.clone()),
            asset,
            reward_type
        ));
    });
}

#[test]
fn test_edge_cases() {
    new_test_ext().execute_with(|| {
        let account: AccountId32 = AccountId32::new([1; 32]);
        let asset = Asset::Custom(0u32);
        let reward_type = RewardType::Restaking;

        // Test with zero rewards
        assert_ok!(Rewards::<Runtime>::whitelist_asset(RuntimeOrigin::root(), asset));
        assert_noop!(
            Rewards::<Runtime>::claim_rewards(RuntimeOrigin::signed(account.clone()), asset, reward_type),
            Error::<Runtime>::NoRewardsAvailable
        );

        // Test with invalid asset ID
        let invalid_asset = Asset::Custom(u32::MAX);
        assert_noop!(
            Rewards::<Runtime>::claim_rewards(RuntimeOrigin::signed(account.clone()), invalid_asset, reward_type),
            Error::<Runtime>::AssetNotWhitelisted
        );
    });
}
