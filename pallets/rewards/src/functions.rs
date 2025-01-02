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
	// let base_score = match asset {
	// 	Asset::Native => {
	// 		// For TNT, include both restaking and boost rewards
	// 		let restaking_score = rewards
	// 			.restaking_rewards
	// 			.saturating_add(rewards.service_rewards)
	// 			.saturated_into::<u128>();

	// 		// Apply lock multiplier to boost rewards
	// 		let boost_score = rewards
	// 			.boost_rewards
	// 			.amount
	// 			.saturated_into::<u128>()
	// 			.saturating_mul(rewards.boost_rewards.multiplier.value() as u128);

	// 		restaking_score.saturating_add(boost_score)
	// 	},
	// 	_ => {
	// 		// For non-TNT assets, only consider service rewards
	// 		rewards.service_rewards.saturated_into::<u128>()
	// 	},
	// };

	// base_score
	todo!();
}
