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

use crate::AssetLookupRewardVaults;
use crate::BalanceOf;
use crate::Error;
use crate::RewardConfigStorage;
use crate::TotalRewardVaultScore;
use crate::UserServiceReward;
use crate::{Config, Pallet};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::traits::Saturating;
use sp_runtime::DispatchError;
use tangle_primitives::types::rewards::LockMultiplier;
use tangle_primitives::{services::Asset, traits::rewards::RewardsManager};

impl<T: Config> RewardsManager<T::AccountId, T::AssetId, BalanceOf<T>, BlockNumberFor<T>>
	for Pallet<T>
{
	type Error = DispatchError;

	fn record_deposit(
		_account_id: &T::AccountId,
		asset: Asset<T::AssetId>,
		amount: BalanceOf<T>,
		_lock_multiplier: Option<LockMultiplier>,
	) -> Result<(), Self::Error> {
		// find the vault for the asset id
		// if the asset is not in a reward vault, do nothing
		if let Some(vault_id) = AssetLookupRewardVaults::<T>::get(asset) {
			// Update the reward vault score
			let score = TotalRewardVaultScore::<T>::get(vault_id).saturating_add(amount);
			TotalRewardVaultScore::<T>::insert(vault_id, score);
		}
		Ok(())
	}

	fn record_withdrawal(
		_account_id: &T::AccountId,
		asset: Asset<T::AssetId>,
		amount: BalanceOf<T>,
	) -> Result<(), Self::Error> {
		// find the vault for the asset id
		// if the asset is not in a reward vault, do nothing
		if let Some(vault_id) = AssetLookupRewardVaults::<T>::get(asset) {
			// Update the reward vault score
			let score = TotalRewardVaultScore::<T>::get(vault_id).saturating_sub(amount);
			TotalRewardVaultScore::<T>::insert(vault_id, score);
		}
		Ok(())
	}

	fn record_service_reward(
		account_id: &T::AccountId,
		asset: Asset<T::AssetId>,
		amount: BalanceOf<T>,
	) -> Result<(), Self::Error> {
		// update the amount in the user service reward storage
		UserServiceReward::<T>::try_mutate(account_id, asset, |reward| {
			*reward = reward.saturating_add(amount);
			Ok(())
		})
	}

	fn get_asset_deposit_cap_remaining(
		asset: Asset<T::AssetId>,
	) -> Result<BalanceOf<T>, Self::Error> {
		// find the vault for the asset id
		// if the asset is not in a reward vault, do nothing
		if let Some(vault_id) = AssetLookupRewardVaults::<T>::get(asset) {
			if let Some(config) = RewardConfigStorage::<T>::get(vault_id) {
				let current_score = TotalRewardVaultScore::<T>::get(vault_id);
				Ok(config.deposit_cap.saturating_sub(current_score))
			} else {
				Err(Error::<T>::RewardConfigNotFound.into())
			}
		} else {
			Err(Error::<T>::AssetNotInVault.into())
		}
	}

	fn get_asset_incentive_cap(asset: Asset<T::AssetId>) -> Result<BalanceOf<T>, Self::Error> {
		// find the vault for the asset id
		// if the asset is not in a reward vault, do nothing
		if let Some(vault_id) = AssetLookupRewardVaults::<T>::get(asset) {
			if let Some(config) = RewardConfigStorage::<T>::get(vault_id) {
				Ok(config.incentive_cap)
			} else {
				Err(Error::<T>::RewardConfigNotFound.into())
			}
		} else {
			Err(Error::<T>::AssetNotInVault.into())
		}
	}
}
