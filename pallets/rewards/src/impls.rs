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
use crate::{Config, Pallet, UserRewards, UserRewardsOf};
use frame_support::traits::Currency;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::traits::{Saturating, Zero};
use tangle_primitives::{services::Asset, traits::rewards::RewardsManager};

impl<T: Config> RewardsManager<T::AccountId, T::AssetId, BalanceOf<T>, BlockNumberFor<T>>
	for Pallet<T>
{
	fn record_deposit(
		account_id: &T::AccountId,
		asset: Asset<T::AssetId>,
		amount: BalanceOf<T>,
		lock_multiplier: Option<LockMultiplier>,
	) -> Result<(), &'static str> {
		Ok(())
	}

	fn record_withdrawal(
		account_id: &T::AccountId,
		asset: Asset<T::AssetId>,
		amount: BalanceOf<T>,
	) -> Result<(), &'static str> {
		Ok(())
	}

	fn record_service_reward(
		account_id: &T::AccountId,
		asset: Asset<T::AssetId>,
		amount: BalanceOf<T>,
	) -> Result<(), &'static str> {
		// TODO : Handle service rewards later
		Ok(())
	}

	fn query_total_deposit(
		account_id: &T::AccountId,
		asset: Asset<T::AssetId>,
	) -> Result<(BalanceOf<T>, BalanceOf<T>), &'static str> {
		todo!()
	}
}
