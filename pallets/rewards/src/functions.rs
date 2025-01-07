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

use crate::{BalanceOf, Config, Pallet, UserRewardsOf};
use frame_support::traits::Currency;
use sp_runtime::{
	traits::{Saturating, Zero},
	Percent,
};
use tangle_primitives::{services::Asset, types::rewards::LockMultiplier};

/// Calculate the score for a user's rewards based on their lock period and amount
pub fn calculate_user_score<T: Config>(
	asset: Asset<T::AssetId>,
	rewards: &UserRewardsOf<T>,
) -> u128 {
	// Get the lock multiplier in months (1, 2, 3, or 6)
	let lock_period = match rewards.boost_rewards.multiplier {
		LockMultiplier::OneMonth => 1,
		LockMultiplier::TwoMonths => 2,
		LockMultiplier::ThreeMonths => 3,
		LockMultiplier::SixMonths => 6,
	};

	// Convert amount to u128 for calculation
	let amount: u128 = rewards.boost_rewards.amount.saturated_into();

	// Score = amount * lock_period
	amount.saturating_mul(lock_period as u128)
}

/// Calculate the APY distribution for a given asset and score
pub fn calculate_apy_distribution<T: Config>(
	asset: Asset<T::AssetId>,
	user_score: u128,
) -> Percent {
	let total_score = <crate::TotalAssetScore<T>>::get(asset);
	if total_score.is_zero() {
		return Percent::zero();
	}

	let total_deposited = <crate::TotalDeposited<T>>::get(asset);
	let capacity = <crate::AssetCapacity<T>>::get(asset);
	let apy_basis_points = <crate::AssetApy<T>>::get(asset);

	// Convert capacity and total_deposited to u128 for calculation
	let capacity: u128 = capacity.saturated_into();
	let total_deposited: u128 = total_deposited.saturated_into();

	if capacity.is_zero() {
		return Percent::zero();
	}

	// Calculate pro-rata score distribution
	let score_ratio = Percent::from_rational(user_score, total_score);

	// Calculate capacity utilization
	let capacity_ratio = Percent::from_rational(total_deposited, capacity);

	// Calculate final APY
	// APY = base_apy * (score/total_score) * (total_deposited/capacity)
	let base_apy = Percent::from_percent(apy_basis_points as u8);
	base_apy.saturating_mul(score_ratio).saturating_mul(capacity_ratio)
}

/// Update the total score for an asset
pub fn update_total_score<T: Config>(asset: Asset<T::AssetId>, old_score: u128, new_score: u128) {
	let current_total = <crate::TotalAssetScore<T>>::get(asset);
	let new_total = current_total.saturating_sub(old_score).saturating_add(new_score);
	<crate::TotalAssetScore<T>>::insert(asset, new_total);
}

/// Update the total deposited amount for an asset
pub fn update_total_deposited<T: Config>(
	asset: Asset<T::AssetId>,
	amount: BalanceOf<T>,
	is_deposit: bool,
) {
	let current_total = <crate::TotalDeposited<T>>::get(asset);
	let new_total = if is_deposit {
		current_total.saturating_add(amount)
	} else {
		current_total.saturating_sub(amount)
	};
	<crate::TotalDeposited<T>>::insert(asset, new_total);
}
