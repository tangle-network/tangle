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
use crate::types::{BalanceOf, OperatorStatus};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{traits::Zero, Percent};
use sp_std::prelude::*;
use tangle_primitives::types::rewards::UserDepositWithLocks;
use tangle_primitives::{
	services::Asset, traits::MultiAssetDelegationInfo, BlueprintId, RoundIndex,
};

impl<T: crate::Config> MultiAssetDelegationInfo<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>
	for crate::Pallet<T>
{
	type AssetId = T::AssetId;

	fn get_current_round() -> RoundIndex {
		Self::current_round()
	}

	fn is_operator(operator: &T::AccountId) -> bool {
		Operators::<T>::get(operator).is_some()
	}

	fn is_operator_active(operator: &T::AccountId) -> bool {
		Operators::<T>::get(operator)
			.is_some_and(|metadata| matches!(metadata.status, OperatorStatus::Active))
	}

	fn get_operator_stake(operator: &T::AccountId) -> BalanceOf<T> {
		Operators::<T>::get(operator).map_or(Zero::zero(), |metadata| metadata.stake)
	}

	fn get_total_delegation_by_asset_id(
		operator: &T::AccountId,
		asset_id: &Asset<T::AssetId>,
	) -> BalanceOf<T> {
		Operators::<T>::get(operator).map_or(Zero::zero(), |metadata| {
			metadata
				.delegations
				.iter()
				.filter(|stake| &stake.asset_id == asset_id)
				.fold(Zero::zero(), |acc, stake| acc + stake.amount)
		})
	}

	fn get_delegators_for_operator(
		operator: &T::AccountId,
	) -> Vec<(T::AccountId, BalanceOf<T>, Asset<Self::AssetId>)> {
		Operators::<T>::get(operator).map_or(Vec::new(), |metadata| {
			metadata
				.delegations
				.iter()
				.map(|stake| (stake.delegator.clone(), stake.amount, stake.asset_id))
				.collect()
		})
	}

	fn slash_operator(operator: &T::AccountId, blueprint_id: BlueprintId, percentage: Percent) {
		let _ = Pallet::<T>::slash_operator(operator, blueprint_id, percentage);
	}

	fn get_user_deposit_with_locks(
		who: &T::AccountId,
		asset_id: Asset<T::AssetId>,
	) -> Option<UserDepositWithLocks<BalanceOf<T>, BlockNumberFor<T>>> {
		Delegators::<T>::get(who).and_then(|metadata| {
			metadata.deposits.get(&asset_id).map(|deposit| UserDepositWithLocks {
				unlocked_amount: deposit.amount,
				amount_with_locks: deposit.locks.as_ref().map(|locks| locks.to_vec()),
			})
		})
	}
}
