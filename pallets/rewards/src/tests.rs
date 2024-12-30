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
use crate::{functions, mock::Test, Error, Event, Pallet as Rewards, RewardType};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{traits::Zero, AccountId32};
use tangle_primitives::services::Asset;

#[test]
fn test_whitelist_asset() {
	new_test_ext().execute_with(|| {
		let asset = Asset::<u32>::Custom(Zero::zero());
		let account: AccountId32 = AccountId32::new([1; 32]);

		// Non-root cannot whitelist asset
		assert_noop!(
			Rewards::<Test>::whitelist_asset(RuntimeOrigin::signed(account.clone()), asset),
			sp_runtime::DispatchError::BadOrigin
		);

		// Root can whitelist asset
		assert_ok!(Rewards::<Test>::whitelist_asset(RuntimeOrigin::root(), asset));
		assert!(Rewards::<Test>::is_asset_whitelisted(asset));

		// Cannot whitelist same asset twice
		assert_noop!(
			Rewards::<Test>::whitelist_asset(RuntimeOrigin::root(), asset),
			Error::<Test>::AssetAlreadyWhitelisted
		);
	});
}

#[test]
fn test_remove_asset() {
	new_test_ext().execute_with(|| {
		let asset = Asset::<u32>::Custom(Zero::zero());
		let account: AccountId32 = AccountId32::new([1; 32]);

		// Cannot remove non-whitelisted asset
		assert_noop!(
			Rewards::<Test>::remove_asset(RuntimeOrigin::root(), asset),
			Error::<Test>::AssetNotWhitelisted
		);

		// Whitelist the asset first
		assert_ok!(Rewards::<Test>::whitelist_asset(RuntimeOrigin::root(), asset));

		// Non-root cannot remove asset
		assert_noop!(
			Rewards::<Test>::remove_asset(RuntimeOrigin::signed(account.clone()), asset),
			sp_runtime::DispatchError::BadOrigin
		);

		// Root can remove asset
		assert_ok!(Rewards::<Test>::remove_asset(RuntimeOrigin::root(), asset));
		assert!(!Rewards::<Test>::is_asset_whitelisted(asset));

		// Cannot remove already removed asset
		assert_noop!(
			Rewards::<Test>::remove_asset(RuntimeOrigin::root(), asset),
			Error::<Test>::AssetNotWhitelisted
		);
	});
}

#[test]
fn test_claim_rewards() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::new([1; 32]);
		let asset = Asset::<u32>::Custom(Zero::zero());
		let reward_type = RewardType::Restaking;

		// Cannot claim rewards for non-whitelisted asset
		assert_noop!(
			Rewards::<Test>::claim_rewards(
				RuntimeOrigin::signed(account.clone()),
				asset,
				reward_type
			),
			Error::<Test>::AssetNotWhitelisted
		);

		// Whitelist the asset
		assert_ok!(Rewards::<Test>::whitelist_asset(RuntimeOrigin::root(), asset));

		// Cannot claim rewards when none are available
		assert_noop!(
			Rewards::<Test>::claim_rewards(
				RuntimeOrigin::signed(account.clone()),
				asset,
				reward_type
			),
			Error::<Test>::NoRewardsAvailable
		);
	});
}

#[test]
fn test_calculate_user_score() {
	new_test_ext().execute_with(|| {
		let tnt_rewards = vec![(1, 100), (2, 200), (3, 300)];

		let other_rewards = vec![(4, 400), (5, 500), (6, 600)];

		// Test native asset rewards calculation (with 6x multiplier)
		let score = functions::calculate_user_score::<Test>(
			Asset::<u32>::Custom(Zero::zero()),
			&tnt_rewards,
		);
		assert_eq!(score, 3600); // (100 + 200 + 300) * 6

		// Test EVM asset rewards calculation (no multiplier)
		let score = functions::calculate_user_score::<Test>(
			Asset::<u32>::Erc20(sp_core::H160::zero()),
			&other_rewards,
		);
		assert_eq!(score, 1500); // 400 + 500 + 600
	});
}
