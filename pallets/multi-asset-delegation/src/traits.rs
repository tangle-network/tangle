// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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
use crate::{types::*, Config};

use parity_scale_codec::{Decode, Encode};
use scale_info::prelude::{vec, vec::Vec};

use sp_runtime::{traits::Zero, RuntimeDebug};

pub trait ServiceManager<AccountId, Balance> {
	/// List active services for the given account ID.
	fn list_active_services(account: &AccountId) -> Vec<Service>;

	/// List service rewards for the given account ID.
	fn list_service_reward(account: &AccountId) -> Balance;

	/// Check if the given account ID can exit.
	fn can_exit(account: &AccountId) -> bool;
}

// Example struct representing a Service
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct Service {
	pub service_id: u32,
	pub name: Vec<u8>,
	pub status: ServiceStatus,
}

// Example enum representing Service Status
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub enum ServiceStatus {
	Active,
	Inactive,
}

pub trait MultiAssetDelegationInfo<AccountId, AssetId, PoolId, Balance> {
	/// Get the current round index.
	fn get_current_round() -> RoundIndex;

	/// Get the reward configuration for a specific asset ID.
	fn get_reward_config(pool_id: &PoolId) -> Option<RewardConfigForAssetPool<Balance>>;

	/// Get the total delegation amount for a specific delegator and asset ID.
	fn get_total_delegation(delegator: &AccountId, asset_id: &AssetId) -> Balance;

	/// Get the delegations for a specific operator.
	fn get_delegations_for_operator(
		operator: &AccountId,
	) -> Vec<DelegatorBond<AccountId, Balance, AssetId>>;
}

impl<T: Config> MultiAssetDelegationInfo<T::AccountId, T::AssetId, T::PoolId, BalanceOf<T>>
	for Pallet<T>
{
	fn get_current_round() -> RoundIndex {
		Self::current_round()
	}

	fn get_reward_config(pool_id: &T::PoolId) -> Option<RewardConfigForAssetPool<BalanceOf<T>>> {
		RewardConfigStorage::<T>::get().and_then(|config| config.configs.get(pool_id).cloned())
	}

	fn get_total_delegation(delegator: &T::AccountId, asset_id: &T::AssetId) -> BalanceOf<T> {
		Delegators::<T>::get(delegator).map_or(Zero::zero(), |metadata| {
			metadata
				.delegations
				.iter()
				.filter(|stake| &stake.asset_id == asset_id)
				.fold(Zero::zero(), |acc, stake| acc + stake.amount)
		})
	}

	fn get_delegations_for_operator(
		operator: &T::AccountId,
	) -> Vec<DelegatorBond<T::AccountId, BalanceOf<T>, T::AssetId>> {
		Operators::<T>::get(operator).map_or(vec![], |metadata| metadata.delegations)
	}
}
