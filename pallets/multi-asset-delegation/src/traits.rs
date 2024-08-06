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
use crate::types::BalanceOf;
use crate::types::OperatorStatus;
use parity_scale_codec::{Decode, Encode};
use scale_info::prelude::vec;
use scale_info::prelude::vec::Vec;
use sp_runtime::traits::Zero;
use sp_runtime::RuntimeDebug;
use tangle_primitives::traits::MultiAssetDelegationInfo;
use tangle_primitives::RoundIndex;

impl<T: crate::Config> MultiAssetDelegationInfo<T::AccountId, BalanceOf<T>> for crate::Pallet<T> {
	type AssetId = T::AssetId;

	fn get_current_round() -> RoundIndex {
		Self::current_round()
	}

	fn is_operator(operator: &T::AccountId) -> bool {
		Operators::<T>::get(operator).is_some()
	}

	fn is_operator_active(operator: &T::AccountId) -> bool {
		Operators::<T>::get(operator).map_or(false, |metadata| match metadata.status {
			OperatorStatus::Active => true,
			_ => false,
		})
	}

	fn get_operator_stake(operator: &T::AccountId) -> BalanceOf<T> {
		Operators::<T>::get(operator).map_or(Zero::zero(), |metadata| metadata.stake)
	}

	fn get_total_delegation_by_asset_id(
		operator: &T::AccountId,
		asset_id: &T::AssetId,
	) -> BalanceOf<T> {
		Operators::<T>::get(operator).map_or(Zero::zero(), |metadata| {
			metadata
				.delegations
				.iter()
				.filter(|stake| &stake.asset_id == asset_id)
				.fold(Zero::zero(), |acc, stake| acc + stake.amount)
		})
	}
}
