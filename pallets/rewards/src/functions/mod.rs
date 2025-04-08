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
	AssetLookupRewardVaults, Config, Error, Pallet, RewardVaults, RewardVaultsPotAccount,
	SubaccountType,
};
use frame_support::{ensure, traits::Get};
use sp_runtime::{DispatchError, DispatchResult, traits::AccountIdConversion};
use sp_std::vec::Vec;
use tangle_primitives::services::Asset;

pub mod rewards;
pub mod services;

impl<T: Config> Pallet<T> {
	pub fn remove_asset_from_vault(
		vault_id: &T::VaultId,
		asset_id: &Asset<T::AssetId>,
	) -> DispatchResult {
		// Update RewardVaults storage
		RewardVaults::<T>::try_mutate(vault_id, |maybe_assets| -> DispatchResult {
			let assets = maybe_assets.as_mut().ok_or(Error::<T>::VaultNotFound)?;

			// Ensure the asset is in the vault
			ensure!(assets.contains(asset_id), Error::<T>::AssetNotInVault);

			assets.retain(|id| id != asset_id);

			Ok(())
		})?;

		// Update AssetLookupRewardVaults storage
		AssetLookupRewardVaults::<T>::remove(asset_id);

		Ok(())
	}

	pub fn add_asset_to_vault(
		vault_id: &T::VaultId,
		asset_id: &Asset<T::AssetId>,
	) -> DispatchResult {
		// Ensure the asset is not already associated with any vault
		ensure!(
			!AssetLookupRewardVaults::<T>::contains_key(asset_id),
			Error::<T>::AssetAlreadyInVault
		);

		// Update RewardVaults storage
		RewardVaults::<T>::try_mutate(vault_id, |maybe_assets| -> DispatchResult {
			let assets = maybe_assets.get_or_insert_with(Vec::new);

			// Ensure the asset is not already in the vault
			ensure!(!assets.contains(asset_id), Error::<T>::AssetAlreadyInVault);

			assets.push(*asset_id);

			Ok(())
		})?;

		// Update AssetLookupRewardVaults storage
		AssetLookupRewardVaults::<T>::insert(asset_id, vault_id);

		Ok(())
	}

	/// Creates a new pot account for a reward vault
	pub fn create_reward_vault_pot(vault_id: T::VaultId) -> Result<T::AccountId, DispatchError> {
		// Initialize the vault pot for rewards
		let pot_account_for_vault: T::AccountId =
			T::PalletId::get().into_sub_account_truncating((SubaccountType::RewardPot, vault_id));
		// Ensure the pot does not already exist
		ensure!(RewardVaultsPotAccount::<T>::get(vault_id).is_none(), Error::<T>::PotAlreadyExists);
		// Store the pot
		RewardVaultsPotAccount::<T>::insert(vault_id, pot_account_for_vault.clone());
		Ok(pot_account_for_vault)
	}
}
