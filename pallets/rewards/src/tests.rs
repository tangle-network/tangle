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
use super::*;
use crate::{mock::*, Error, Event, RewardType};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::Zero;
use tangle_primitives::services::Asset;

#[test]
fn test_whitelist_asset() {
	new_test_ext().execute_with(|| {
		let asset = Asset::<u32>::Native(0);

		// Only root can whitelist assets
		assert_noop!(
			Rewards::whitelist_asset(RuntimeOrigin::signed(1), asset),
			sp_runtime::DispatchError::BadOrigin
		);

		// Root can whitelist asset
		assert_ok!(Rewards::whitelist_asset(RuntimeOrigin::root(), asset));
		assert!(Rewards::is_asset_whitelisted(asset));

		// Cannot whitelist same asset twice
		assert_noop!(
			Rewards::whitelist_asset(RuntimeOrigin::root(), asset),
			Error::<Test>::AssetAlreadyWhitelisted
		);
	});
}

#[test]
fn test_remove_asset() {
	new_test_ext().execute_with(|| {
		let asset = Asset::<u32>::Native(0);

		// Whitelist asset first
		assert_ok!(Rewards::whitelist_asset(RuntimeOrigin::root(), asset));

		// Only root can remove assets
		assert_noop!(
			Rewards::remove_asset(RuntimeOrigin::signed(1), asset),
			sp_runtime::DispatchError::BadOrigin
		);

		// Root can remove asset
		assert_ok!(Rewards::remove_asset(RuntimeOrigin::root(), asset));
		assert!(!Rewards::is_asset_whitelisted(asset));

		// Cannot remove non-whitelisted asset
		assert_noop!(
			Rewards::remove_asset(RuntimeOrigin::root(), asset),
			Error::<Test>::AssetNotWhitelisted
		);
	});
}

#[test]
fn test_claim_rewards() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let asset = Asset::<u32>::Native(0);
		let reward_type = RewardType::Restaking;

		// Cannot claim from non-whitelisted asset
		assert_noop!(
			Rewards::claim_rewards(RuntimeOrigin::signed(user), asset, reward_type),
			Error::<Test>::AssetNotWhitelisted
		);

		// Whitelist asset
		assert_ok!(Rewards::whitelist_asset(RuntimeOrigin::root(), asset));

		// Cannot claim when no rewards available
		assert_noop!(
			Rewards::claim_rewards(RuntimeOrigin::signed(user), asset, reward_type),
			Error::<Test>::NoRewardsAvailable
		);

		// TODO: Add test for successful claim once we have a way to add rewards
	});
}

#[test]
fn test_calculate_user_score() {
	new_test_ext().execute_with(|| {
		use crate::types::{BoostInfo, LockMultiplier, UserRewards};

		// Test TNT asset scoring
		let tnt_rewards = UserRewards {
			restaking_rewards: 1000u32.into(),
			boost_rewards: BoostInfo {
				amount: 500u32.into(),
				multiplier: LockMultiplier::ThreeMonths,
				expiry: Zero::zero(),
			},
			service_rewards: 200u32.into(),
		};

		let tnt_score =
			crate::functions::calculate_user_score::<Test>(Asset::<u32>::Native(0), &tnt_rewards);
		// Expected: 1000 (restaking) + 200 (service) + (500 * 3) (boosted) = 2700
		assert_eq!(tnt_score, 2700);

		// Test non-TNT asset scoring
		let other_rewards = UserRewards {
			restaking_rewards: 1000u32.into(),
			boost_rewards: BoostInfo {
				amount: 500u32.into(),
				multiplier: LockMultiplier::ThreeMonths,
				expiry: Zero::zero(),
			},
			service_rewards: 200u32.into(),
		};

		let other_score =
			crate::functions::calculate_user_score::<Test>(Asset::<u32>::Evm(1), &other_rewards);
		// Expected: Only service rewards count for non-TNT assets
		assert_eq!(other_score, 200);
	});
}
