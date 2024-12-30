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

use crate::{Config, UserRewardsOf};
use frame_support::traits::Currency;
use sp_runtime::{traits::Zero, Saturating};
use tangle_primitives::services::Asset;

/// Calculate a user's score based on their staked assets and lock multipliers.
/// Score is calculated as follows:
/// - For TNT assets:
///   - Base score = staked amount
///   - Multiplier based on lock period (1x to 6x)
/// - For other assets:
///   - Base score = staked amount
///   - No additional multiplier
pub fn calculate_user_score<T: Config>(
	asset: Asset<T::AssetId>,
	rewards: &UserRewardsOf<T>,
) -> u128 {
	let base_score = match asset {
		Asset::Native => {
			// For TNT, include both restaking and boost rewards
			let restaking_score = rewards
				.restaking_rewards
				.saturating_add(rewards.service_rewards)
				.saturated_into::<u128>();

			// Apply lock multiplier to boost rewards
			let boost_score = rewards
				.boost_rewards
				.amount
				.saturated_into::<u128>()
				.saturating_mul(rewards.boost_rewards.multiplier.value() as u128);

			restaking_score.saturating_add(boost_score)
		},
		_ => {
			// For non-TNT assets, only consider service rewards
			rewards.service_rewards.saturated_into::<u128>()
		},
	};

	base_score
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{BoostInfo, LockMultiplier, UserRewards};
	use sp_runtime::traits::Zero;

	// Helper function to create UserRewards with specific values
	fn create_user_rewards<T: Config>(
		restaking: u128,
		boost_amount: u128,
		boost_multiplier: LockMultiplier,
		service: u128,
	) -> UserRewardsOf<T> {
		UserRewards {
			restaking_rewards: restaking.saturated_into(),
			boost_rewards: BoostInfo {
				amount: boost_amount.saturated_into(),
				multiplier: boost_multiplier,
				expiry: Zero::zero(),
			},
			service_rewards: service.saturated_into(),
		}
	}

	#[test]
	fn test_calculate_user_score_tnt() {
		// Test TNT asset with different lock multipliers
		let rewards = create_user_rewards::<T>(
			1000,                        // restaking rewards
			500,                         // boost amount
			LockMultiplier::ThreeMonths, // 3x multiplier
			200,                         // service rewards
		);

		let score = calculate_user_score::<T>(Asset::Native, &rewards);

		// Expected: 1000 (restaking) + 200 (service) + (500 * 3) (boosted) = 2700
		assert_eq!(score, 2700);
	}

	#[test]
	fn test_calculate_user_score_other_asset() {
		// Test non-TNT asset
		let rewards = create_user_rewards::<T>(
			1000,                      // restaking rewards
			500,                       // boost amount
			LockMultiplier::SixMonths, // 6x multiplier (should not affect non-TNT)
			200,                       // service rewards
		);

		let score = calculate_user_score::<T>(Asset::Evm(1), &rewards);

		// Expected: only service rewards are counted
		assert_eq!(score, 200);
	}
}
