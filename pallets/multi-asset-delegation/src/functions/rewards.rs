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
use crate::{
	types::{DelegatorBond, *},
	Pallet,
};
use frame_support::{ensure, pallet_prelude::DispatchResult, traits::Currency};
use sp_runtime::{traits::Zero, DispatchError, Saturating};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
use tangle_primitives::RoundIndex;

impl<T: Config> Pallet<T> {
	#[allow(clippy::type_complexity)]
	pub fn distribute_rewards(round: RoundIndex) -> DispatchResult {
		let mut delegation_info: BTreeMap<
			T::AssetId,
			Vec<DelegatorBond<T::AccountId, BalanceOf<T>, T::AssetId>>,
		> = BTreeMap::new();

		// Iterate through all operator snapshots for the given round
		// TODO: Could be dangerous with many operators
		for (_, operator_snapshot) in AtStake::<T>::iter_prefix(round) {
			for delegation in &operator_snapshot.delegations {
				delegation_info.entry(delegation.asset_id).or_default().push(delegation.clone());
			}
		}

		// Get the reward configuration
		if let Some(reward_config) = RewardConfigStorage::<T>::get() {
			// Distribute rewards for each asset
			for (asset_id, delegations) in delegation_info.iter() {
				// We only reward asset in a reward vault
				if let Some(vault_id) = AssetLookupRewardVaults::<T>::get(asset_id) {
					if let Some(config) = reward_config.configs.get(&vault_id) {
						// Calculate total amount and distribute rewards
						let total_amount: BalanceOf<T> =
							delegations.iter().fold(Zero::zero(), |acc, d| acc + d.amount);
						let cap: BalanceOf<T> = config.cap;

						if total_amount >= cap {
							// Calculate the total reward based on the APY
							let total_reward =
								Self::calculate_total_reward(config.apy, total_amount)?;

							for delegation in delegations {
								// Calculate the percentage of the cap that the user is staking
								let staking_percentage =
									delegation.amount.saturating_mul(100u32.into()) / cap;
								// Calculate the reward based on the staking percentage
								let reward =
									total_reward.saturating_mul(staking_percentage) / 100u32.into();
								// Distribute the reward to the delegator
								Self::distribute_reward_to_delegator(
									&delegation.delegator,
									reward,
								)?;
							}
						}
					}
				}
			}
		}

		Ok(())
	}

	fn calculate_total_reward(
		apy: sp_runtime::Percent,
		total_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let total_reward = apy.mul_floor(total_amount);
		Ok(total_reward)
	}

	fn distribute_reward_to_delegator(
		delegator: &T::AccountId,
		reward: BalanceOf<T>,
	) -> DispatchResult {
		// mint rewards to delegator
		let _ = T::Currency::deposit_creating(delegator, reward);
		Ok(())
	}

	pub fn add_asset_to_vault(vault_id: &T::VaultId, asset_id: &T::AssetId) -> DispatchResult {
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

	pub fn remove_asset_from_vault(vault_id: &T::VaultId, asset_id: &T::AssetId) -> DispatchResult {
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
}
