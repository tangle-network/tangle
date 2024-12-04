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
use super::RestakeDepositScore;
use super::*;
use crate::{
	types::{DelegatorBond, *},
	Pallet,
};
use frame_support::{ensure, pallet_prelude::DispatchResult, traits::Get};
use sp_runtime::{traits::Zero, DispatchError, Saturating};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
use tangle_primitives::RoundIndex;

impl<T: Config> Pallet<T> {
	/// Convert asset amount to score. Currently returns the same value, to be updated later.
	/// TODO : Once we have an oracle for asset price, we can use it to convert to score
	pub fn asset_to_score(amount: BalanceOf<T>) -> BalanceOf<T> {
		amount
	}

	#[allow(clippy::type_complexity)]
	pub fn distribute_rewards(round: RoundIndex) -> DispatchResult {
		let mut delegation_info: BTreeMap<
			T::AssetId,
			Vec<DelegatorBond<T::AccountId, BalanceOf<T>, T::AssetId>>,
		> = BTreeMap::new();

		// Iterate through all operator snapshots for the given round
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
						// Calculate total score and distribute rewards
						let current_block = frame_system::Pallet::<T>::block_number();
						let mut total_score = BalanceOf::<T>::zero();
						let mut delegation_score = Vec::new();

						// Calculate score for each delegation
						for delegation in delegations {
							if let Some(restake_score) =
								<RestakeDepositScore<T>>::get(&delegation.delegator, asset_id)
							{
								// Skip if score has expired
								if restake_score.expiry <= current_block {
									continue;
								}

								// Convert asset amount to score
								let base_score = Self::asset_to_score(delegation.amount);

								// Calculate score with multipliers
								let mut score = base_score;
								score = score.saturating_mul(restake_score.lock_multiplier.into());

								// Apply TNT boost multiplier if the asset is TNT
								if asset_id == &T::NativeAssetId::get() {
									score =
										score.saturating_mul(config.tnt_boost_multiplier.into());
								}

								total_score = total_score.saturating_add(score);
								delegation_score.push((delegation.delegator.clone(), score));
							}
						}

						if !total_score.is_zero() {
							// Calculate the total reward based on the APY
							let total_reward =
								Self::calculate_total_reward(config.apy, total_score)?;

							// Store rewards for each delegator
							for (delegator, score) in delegation_score {
								let reward = total_reward.saturating_mul(score) / total_score;

								// Handle auto-compounding if enabled
								if let Some(restake_score) =
									<RestakeDepositScore<T>>::get(&delegator, asset_id)
								{
									if restake_score.auto_compound {
										Self::auto_compound_reward(&delegator, reward, asset_id)?;
									} else {
										// Store reward for later claiming
										PendingRewards::<T>::mutate(
											&delegator,
											asset_id,
											|pending| {
												*pending = pending.saturating_add(reward);
											},
										);
										// Update total unclaimed rewards
										TotalUnclaimedRewards::<T>::mutate(asset_id, |total| {
											*total = total.saturating_add(reward);
										});
									}
								}
							}

							// Emit event for rewards distribution
							Self::deposit_event(Event::RewardsDistributed {
								asset_id: *asset_id,
								total_reward,
								round,
							});
						}
					}
				}
			}
		}

		Ok(())
	}

	fn auto_compound_reward(
		delegator: &T::AccountId,
		reward: BalanceOf<T>,
		asset_id: &T::AssetId,
	) -> DispatchResult {
		// If TNT, restake directly
		if asset_id == &T::NativeAssetId::get() {
			// Add reward to existing stake
			if let Some(mut restake_score) = <RestakeDepositScore<T>>::get(delegator, asset_id) {
				restake_score.base_score = restake_score.base_score.saturating_add(reward);
				<RestakeDepositScore<T>>::insert(delegator, asset_id, restake_score);

				// Emit auto-compound event
				Self::deposit_event(Event::RewardAutoCompounded {
					who: delegator.clone(),
					asset_id: *asset_id,
					amount: reward,
				});
			}
		} else {
			// For other assets, store as pending reward
			PendingRewards::<T>::mutate(delegator, asset_id, |pending| {
				*pending = pending.saturating_add(reward);
			});
			TotalUnclaimedRewards::<T>::mutate(asset_id, |total| {
				*total = total.saturating_add(reward);
			});
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
