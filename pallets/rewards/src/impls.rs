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

use crate::{
	AssetLookupRewardVaults, BalanceOf, Config, Error, Event, Pallet, RewardConfigStorage,
	TotalRewardVaultDeposit, TotalRewardVaultScore, UserClaimedReward, UserServiceReward,
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{traits::Saturating, DispatchError};
use tangle_primitives::{
	services::Asset, traits::rewards::RewardsManager, types::rewards::LockMultiplier,
};

impl<T: Config> RewardsManager<T::AccountId, T::AssetId, BalanceOf<T>, BlockNumberFor<T>>
	for Pallet<T>
{
	type Error = DispatchError;

	fn record_deposit(
		account_id: &T::AccountId,
		asset: Asset<T::AssetId>,
		amount: BalanceOf<T>,
		lock_multiplier: Option<LockMultiplier>,
	) -> Result<(), Self::Error> {
		// find the vault for the asset id
		// if the asset is not in a reward vault, do nothing
		if let Some(vault_id) = AssetLookupRewardVaults::<T>::get(asset) {
			// Update the reward vault deposit
			let deposit = TotalRewardVaultDeposit::<T>::get(vault_id).saturating_add(amount);
			TotalRewardVaultDeposit::<T>::insert(vault_id, deposit);

			// emit event
			Self::deposit_event(Event::TotalDepositUpdated {
				vault_id,
				asset,
				total_deposit: deposit,
			});

			// Update the reward vault score
			let score = if let Some(lock_multiplier) = lock_multiplier {
				amount.saturating_mul(lock_multiplier.value().into())
			} else {
				amount
			};

			let new_score = TotalRewardVaultScore::<T>::get(vault_id).saturating_add(score);
			TotalRewardVaultScore::<T>::insert(vault_id, new_score);

			// emit event
			Self::deposit_event(Event::TotalScoreUpdated {
				vault_id,
				total_score: new_score,
				asset,
				lock_multiplier,
			});

			// If this user has never claimed rewards, create an entry
			// this will give us a starting point for reward claim
			if !UserClaimedReward::<T>::contains_key(account_id, vault_id) {
				let current_block = frame_system::Pallet::<T>::block_number();
				let default_balance: BalanceOf<T> = 0_u32.into();
				UserClaimedReward::<T>::insert(
					account_id,
					vault_id,
					(current_block, default_balance),
				);
			}
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
			// Update the reward vault deposit
			let deposit = TotalRewardVaultDeposit::<T>::get(vault_id).saturating_sub(amount);
			TotalRewardVaultDeposit::<T>::insert(vault_id, deposit);

			// emit event
			Self::deposit_event(Event::TotalDepositUpdated {
				vault_id,
				asset,
				total_deposit: deposit,
			});

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
