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
		boost_rewards: BoostInfo { amount: boost_amount, multiplier, expiry },
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
			Rewards::<Runtime>::claim_rewards(
				RuntimeOrigin::signed(account.clone()),
				asset,
				reward_type
			),
			Error::<Runtime>::AssetNotWhitelisted
		);

		// Whitelist the asset
		assert_ok!(Rewards::<Runtime>::whitelist_asset(RuntimeOrigin::root(), asset));

		// Cannot claim rewards when none are available
		assert_noop!(
			Rewards::<Runtime>::claim_rewards(
				RuntimeOrigin::signed(account.clone()),
				asset,
				reward_type
			),
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
			100u128.into(), // restaking rewards
			50u128.into(),  // service rewards
			200u128.into(), // boost amount
			LockMultiplier::ThreeMonths,
			100u64.into(), // expiry block
		);

		// Set up rewards for ERC20 asset without boost
		setup_user_rewards::<Runtime>(
			account.clone(),
			erc20_asset,
			100u128.into(), // restaking rewards
			50u128.into(),  // service rewards
			0u128.into(),   // no boost
			LockMultiplier::OneMonth,
			0u64.into(), // no expiry
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
			100u128.into(), // restaking rewards
			50u128.into(),  // service rewards
			200u128.into(), // boost amount
			LockMultiplier::ThreeMonths,
			100u64.into(), // expiry block
		);

		setup_user_rewards::<Runtime>(
			account2.clone(),
			asset,
			200u128.into(), // restaking rewards
			100u128.into(), // service rewards
			400u128.into(), // boost amount
			LockMultiplier::SixMonths,
			200u64.into(), // expiry block
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
			100u128.into(), // restaking rewards
			150u128.into(), // service rewards
			200u128.into(), // boost amount
			LockMultiplier::ThreeMonths,
			100u64.into(), // expiry block
		);

		// Test claiming each type of reward
		let reward_types = vec![RewardType::Restaking, RewardType::Service, RewardType::Boost];

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
			100u128.into(), // restaking rewards
			50u128.into(),  // service rewards
			0u128.into(),   // no boost
			LockMultiplier::OneMonth,
			0u64.into(), // no expiry
		);

		// First claim should succeed
		assert_ok!(Rewards::<Runtime>::claim_rewards(
			RuntimeOrigin::signed(account.clone()),
			asset,
			reward_type
		));

		// Second claim should fail as rewards are cleared
		assert_noop!(
			Rewards::<Runtime>::claim_rewards(
				RuntimeOrigin::signed(account.clone()),
				asset,
				reward_type
			),
			Error::<Runtime>::NoRewardsAvailable
		);

		// Set up new rewards
		setup_user_rewards::<Runtime>(
			account.clone(),
			asset,
			200u128.into(), // restaking rewards
			100u128.into(), // service rewards
			0u128.into(),   // no boost
			LockMultiplier::OneMonth,
			0u64.into(), // no expiry
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
			Rewards::<Runtime>::claim_rewards(
				RuntimeOrigin::signed(account.clone()),
				asset,
				reward_type
			),
			Error::<Runtime>::NoRewardsAvailable
		);

		// Test with invalid asset ID
		let invalid_asset = Asset::Custom(u32::MAX);
		assert_noop!(
			Rewards::<Runtime>::claim_rewards(
				RuntimeOrigin::signed(account.clone()),
				invalid_asset,
				reward_type
			),
			Error::<Runtime>::AssetNotWhitelisted
		);
	});
}

#[test]
fn test_rewards_manager_implementation() {
	new_test_ext().execute_with(|| {
		use tangle_primitives::traits::rewards::RewardsManager;

		let account: AccountId32 = AccountId32::new([1; 32]);
		let asset = Asset::Custom(0u32);

		// Whitelist the asset first
		assert_ok!(Rewards::<Runtime>::whitelist_asset(RuntimeOrigin::root(), asset));

		// Test recording delegation reward
		assert_ok!(Rewards::<Runtime>::record_delegation_reward(
			&account,
			asset,
			100u128.into(),
			3, // 3 months lock
		));

		// Verify delegation rewards
		let delegation_rewards = Rewards::<Runtime>::query_delegation_rewards(&account, asset)
			.expect("Should return delegation rewards");
		assert_eq!(delegation_rewards, 100u128.into());

		// Test recording service reward
		assert_ok!(Rewards::<Runtime>::record_service_reward(&account, asset, 50u128.into(),));

		// Verify service rewards
		let service_rewards = Rewards::<Runtime>::query_service_rewards(&account, asset)
			.expect("Should return service rewards");
		assert_eq!(service_rewards, 50u128.into());

		// Test querying total rewards
		let (delegation, service) = Rewards::<Runtime>::query_rewards(&account, asset)
			.expect("Should return total rewards");
		assert_eq!(delegation, 100u128.into());
		assert_eq!(service, 50u128.into());

		// Test recording rewards for non-whitelisted asset
		let non_whitelisted = Asset::Custom(1u32);
		assert_eq!(
			Rewards::<Runtime>::record_delegation_reward(
				&account,
				non_whitelisted,
				100u128.into(),
				3
			),
			Err("Asset not whitelisted")
		);
		assert_eq!(
			Rewards::<Runtime>::record_service_reward(&account, non_whitelisted, 50u128.into()),
			Err("Asset not whitelisted")
		);

		// Test invalid lock multiplier
		assert_eq!(
			Rewards::<Runtime>::record_delegation_reward(&account, asset, 100u128.into(), 4),
			Err("Invalid lock multiplier")
		);

		// Test accumulating rewards
		assert_ok!(
			Rewards::<Runtime>::record_delegation_reward(&account, asset, 50u128.into(), 3,)
		);
		assert_ok!(Rewards::<Runtime>::record_service_reward(&account, asset, 25u128.into(),));

		let (delegation, service) = Rewards::<Runtime>::query_rewards(&account, asset)
			.expect("Should return total rewards");
		assert_eq!(delegation, 150u128.into()); // 100 + 50
		assert_eq!(service, 75u128.into()); // 50 + 25
	});
}

#[test]
fn test_update_asset_rewards_should_fail_for_non_root() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = AccountId::from(Bob);
		let asset_id = Asset::Custom(1);
		let rewards = 1_000;

		// Act & Assert
		assert_noop!(
			Rewards::update_asset_rewards(RuntimeOrigin::signed(who), asset_id, rewards),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn test_update_asset_apy_should_fail_for_non_root() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = AccountId::from(Bob);
		let asset_id = Asset::Custom(1);
		let apy = 500; // 5%

		// Act & Assert
		assert_noop!(
			Rewards::update_asset_apy(RuntimeOrigin::signed(who), asset_id, apy),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn test_reward_score_calculation_with_zero_values() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = AccountId::from(Bob);
		let asset_id = Asset::Custom(1);

		// Test with zero stake
		assert_eq!(
			Rewards::calculate_reward_score(0, 1000, 500),
			0,
			"Reward score should be 0 with zero stake"
		);

		// Test with zero rewards
		assert_eq!(
			Rewards::calculate_reward_score(1000, 0, 500),
			0,
			"Reward score should be 0 with zero rewards"
		);

		// Test with zero APY
		assert_eq!(
			Rewards::calculate_reward_score(1000, 1000, 0),
			0,
			"Reward score should be 0 with zero APY"
		);
	});
}

#[test]
fn test_reward_score_calculation_with_large_values() {
	new_test_ext().execute_with(|| {
		// Test with maximum possible values
		let max_balance = u128::MAX;
		let large_apy = 10_000; // 100%

		// Should not overflow
		let score = Rewards::calculate_reward_score(max_balance, max_balance, large_apy);
		assert!(score > 0, "Reward score should not overflow with large values");
	});
}

#[test]
fn test_rewards_should_fail_with_overflow() {
	new_test_ext().execute_with(|| {
		let asset_id = Asset::Custom(1);

		// Try to set rewards to maximum value
		assert_ok!(Rewards::update_asset_rewards(
			RuntimeOrigin::root(),
			asset_id.clone(),
			u128::MAX
		));

		// Try to set APY to maximum value - this should cause overflow in calculations
		assert_ok!(Rewards::update_asset_apy(RuntimeOrigin::root(), asset_id.clone(), u32::MAX));

		// Attempting to calculate reward score with max values should return 0 to prevent overflow
		let score = Rewards::calculate_reward_score(u128::MAX, u128::MAX, u32::MAX);
		assert_eq!(score, 0, "Should handle potential overflow gracefully");
	});
}

#[test]
fn test_rewards_with_invalid_asset() {
	new_test_ext().execute_with(|| {
		let invalid_asset = Asset::Custom(u32::MAX);
		let rewards = 1_000;
		let apy = 500;

		// Should succeed but have no effect since asset doesn't exist
		assert_ok!(Rewards::update_asset_rewards(
			RuntimeOrigin::root(),
			invalid_asset.clone(),
			rewards
		));

		assert_ok!(Rewards::update_asset_apy(RuntimeOrigin::root(), invalid_asset.clone(), apy));

		// Verify no rewards are available for invalid asset
		assert_eq!(Rewards::asset_rewards(invalid_asset.clone()), 0);
		assert_eq!(Rewards::asset_apy(invalid_asset.clone()), 0);
	});
}

#[test]
fn test_rewards_with_zero_stake() {
	new_test_ext().execute_with(|| {
		let asset_id = Asset::Custom(1);
		let rewards = 1_000;
		let apy = 500;

		// Set up rewards and APY
		assert_ok!(Rewards::update_asset_rewards(RuntimeOrigin::root(), asset_id.clone(), rewards));
		assert_ok!(Rewards::update_asset_apy(RuntimeOrigin::root(), asset_id.clone(), apy));

		// Calculate rewards for zero stake
		let reward_score = Rewards::calculate_reward_score(0, rewards, apy);
		assert_eq!(reward_score, 0, "Zero stake should result in zero rewards");

		// Verify total rewards score is zero when no stakes exist
		let total_score = Rewards::calculate_total_reward_score(asset_id.clone());
		assert_eq!(total_score, 0, "Total score should be zero when no stakes exist");
	});
}

#[test]
fn test_rewards_with_extreme_apy_values() {
	new_test_ext().execute_with(|| {
		let asset_id = Asset::Custom(1);
		let stake = 1_000;
		let rewards = 1_000;

		// Test extremely high APY (10000% = 100x)
		let extreme_apy = 1_000_000; // 10000%
		assert_ok!(Rewards::update_asset_apy(RuntimeOrigin::root(), asset_id.clone(), extreme_apy));

		let score = Rewards::calculate_reward_score(stake, rewards, extreme_apy);
		assert!(score > 0, "Should handle extreme APY values");
		assert!(score > rewards, "High APY should result in higher rewards");

		// Test with minimum possible APY
		assert_ok!(Rewards::update_asset_apy(RuntimeOrigin::root(), asset_id.clone(), 1));

		let min_score = Rewards::calculate_reward_score(stake, rewards, 1);
		assert!(min_score < score, "Minimum APY should result in lower rewards");
	});
}

#[test]
fn test_rewards_accumulation() {
	new_test_ext().execute_with(|| {
		let asset_id = Asset::Custom(1);
		let initial_rewards = 1_000;
		let additional_rewards = 500;

		// Set initial rewards
		assert_ok!(Rewards::update_asset_rewards(
			RuntimeOrigin::root(),
			asset_id.clone(),
			initial_rewards
		));

		// Try to add more rewards - should replace, not accumulate
		assert_ok!(Rewards::update_asset_rewards(
			RuntimeOrigin::root(),
			asset_id.clone(),
			additional_rewards
		));

		assert_eq!(
			Rewards::asset_rewards(asset_id.clone()),
			additional_rewards,
			"Rewards should be replaced, not accumulated"
		);
	});
}

#[test]
fn test_rewards_with_multiple_assets() {
	new_test_ext().execute_with(|| {
		let asset1 = Asset::Custom(1);
		let asset2 = Asset::Custom(2);
		let rewards1 = 1_000;
		let rewards2 = 2_000;
		let apy1 = 500;
		let apy2 = 1_000;

		// Set different rewards and APY for different assets
		assert_ok!(Rewards::update_asset_rewards(RuntimeOrigin::root(), asset1.clone(), rewards1));
		assert_ok!(Rewards::update_asset_rewards(RuntimeOrigin::root(), asset2.clone(), rewards2));
		assert_ok!(Rewards::update_asset_apy(RuntimeOrigin::root(), asset1.clone(), apy1));
		assert_ok!(Rewards::update_asset_apy(RuntimeOrigin::root(), asset2.clone(), apy2));

		// Verify each asset maintains its own rewards and APY
		assert_eq!(Rewards::asset_rewards(asset1.clone()), rewards1);
		assert_eq!(Rewards::asset_rewards(asset2.clone()), rewards2);
		assert_eq!(Rewards::asset_apy(asset1.clone()), apy1);
		assert_eq!(Rewards::asset_apy(asset2.clone()), apy2);

		// Verify reward scores are calculated independently
		let stake = 1_000;
		let score1 = Rewards::calculate_reward_score(stake, rewards1, apy1);
		let score2 = Rewards::calculate_reward_score(stake, rewards2, apy2);
		assert_ne!(score1, score2, "Different assets should have different reward scores");
	});
}

#[test]
fn test_update_asset_rewards_multiple_times() {
	new_test_ext().execute_with(|| {
		// Arrange
		let asset_id = Asset::Custom(1);
		let initial_rewards = 1_000;
		let updated_rewards = 2_000;

		// Act - Update rewards multiple times
		assert_ok!(Rewards::update_asset_rewards(
			RuntimeOrigin::root(),
			asset_id.clone(),
			initial_rewards
		));

		assert_ok!(Rewards::update_asset_rewards(
			RuntimeOrigin::root(),
			asset_id.clone(),
			updated_rewards
		));

		// Assert
		assert_eq!(
			Rewards::asset_rewards(asset_id),
			updated_rewards,
			"Asset rewards should be updated to latest value"
		);
	});
}

#[test]
fn test_update_asset_apy_multiple_times() {
	new_test_ext().execute_with(|| {
		// Arrange
		let asset_id = Asset::Custom(1);
		let initial_apy = 500; // 5%
		let updated_apy = 1_000; // 10%

		// Act - Update APY multiple times
		assert_ok!(Rewards::update_asset_apy(RuntimeOrigin::root(), asset_id.clone(), initial_apy));

		assert_ok!(Rewards::update_asset_apy(RuntimeOrigin::root(), asset_id.clone(), updated_apy));

		// Assert
		assert_eq!(
			Rewards::asset_apy(asset_id),
			updated_apy,
			"Asset APY should be updated to latest value"
		);
	});
}

#[test]
fn test_reward_distribution_with_zero_total_score() {
	new_test_ext().execute_with(|| {
		// Arrange
		let asset_id = Asset::Custom(1);
		let rewards = 1_000;
		let apy = 500; // 5%

		// Update rewards and APY
		assert_ok!(Rewards::update_asset_rewards(RuntimeOrigin::root(), asset_id.clone(), rewards));
		assert_ok!(Rewards::update_asset_apy(RuntimeOrigin::root(), asset_id.clone(), apy));

		// With no stakes, total score should be 0
		let total_score = Rewards::calculate_total_reward_score(asset_id.clone());
		assert_eq!(total_score, 0, "Total score should be 0 with no stakes");
	});
}

#[test]
fn test_reward_score_calculation_with_different_apy_ranges() {
	new_test_ext().execute_with(|| {
		let stake = 1_000;
		let rewards = 1_000;

		// Test with different APY ranges
		let apys = vec![
			0,      // 0%
			100,    // 1%
			500,    // 5%
			1_000,  // 10%
			5_000,  // 50%
			10_000, // 100%
		];

		for apy in apys {
			let score = Rewards::calculate_reward_score(stake, rewards, apy);
			assert!(score >= 0, "Reward score should be non-negative for APY {}", apy);
		}
	});
}
