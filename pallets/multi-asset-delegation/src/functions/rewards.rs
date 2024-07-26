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
use crate::types::*;
use crate::Pallet;
use frame_support::ensure;
use frame_support::pallet_prelude::DispatchResult;
use frame_support::traits::Currency;
use sp_runtime::traits::Zero;
use sp_runtime::DispatchError;
use sp_runtime::Saturating;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
	#[allow(clippy::type_complexity)]
	pub fn distribute_rewards(round: RoundIndex) -> DispatchResult {
		let mut delegation_info: BTreeMap<
			T::AssetId,
			Vec<DelegatorBond<T::AccountId, BalanceOf<T>, T::AssetId>>,
		> = BTreeMap::new();

		// Iterate through all operator snapshots for the given round
		// TODO : Could be dangerous with many operators
		for (_, operator_snapshot) in AtStake::<T>::iter_prefix(round) {
			for delegation in &operator_snapshot.delegations {
				delegation_info.entry(delegation.asset_id).or_default().push(delegation.clone());
			}
		}

		// Get the reward configuration
		if let Some(reward_config) = RewardConfigStorage::<T>::get() {
			// Distribute rewards for each asset
			for (asset_id, delegations) in delegation_info.iter() {
				// we only reward asset in a reward pool
				if let Some(pool_id) = AssetLookupRewardPools::<T>::get(asset_id) {
					if let Some(config) = reward_config.configs.get(&pool_id) {
						// Calculate total amount and distribute rewards
						let total_amount: BalanceOf<T> =
							delegations.iter().fold(Zero::zero(), |acc, d| acc + d.amount);
						let cap: BalanceOf<T> = config.cap;

						if total_amount >= cap {
							// Calculate the total reward based on the APY
							let total_reward =
								Self::calculate_total_reward(config.apy, total_amount)?;

							for delegation in delegations {
								let reward = total_reward * delegation.amount / total_amount;
								// Logic to distribute reward to the delegator (e.g., mint or transfer tokens)
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
		apy: u32,
		total_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		// Assume APY is given as a percentage (e.g., 5 for 5%)
		let total_reward = total_amount.saturating_mul(apy.into()) / 100u32.into();
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

	pub fn add_asset_to_pool(pool_id: &T::PoolId, asset_id: &T::AssetId) -> DispatchResult {
		// Ensure the asset is not already associated with any pool
		ensure!(
			!AssetLookupRewardPools::<T>::contains_key(asset_id),
			Error::<T>::AssetAlreadyInPool
		);

		// Update RewardPools storage
		RewardPools::<T>::try_mutate(pool_id, |maybe_assets| -> DispatchResult {
			let assets = maybe_assets.get_or_insert_with(Vec::new);

			// Ensure the asset is not already in the pool
			ensure!(!assets.contains(asset_id), Error::<T>::AssetAlreadyInPool);

			assets.push(*asset_id);

			Ok(())
		})?;

		// Update AssetLookupRewardPools storage
		AssetLookupRewardPools::<T>::insert(asset_id, pool_id);

		Ok(())
	}

	pub fn remove_asset_from_pool(pool_id: &T::PoolId, asset_id: &T::AssetId) -> DispatchResult {
		// Update RewardPools storage
		RewardPools::<T>::try_mutate(pool_id, |maybe_assets| -> DispatchResult {
			let assets = maybe_assets.as_mut().ok_or(Error::<T>::PoolNotFound)?;

			// Ensure the asset is in the pool
			ensure!(assets.contains(asset_id), Error::<T>::AssetNotInPool);

			assets.retain(|id| id != asset_id);

			Ok(())
		})?;

		// Update AssetLookupRewardPools storage
		AssetLookupRewardPools::<T>::remove(asset_id);

		Ok(())
	}
}
