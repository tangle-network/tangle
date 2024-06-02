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
use frame_support::traits::Get;
use frame_support::traits::ReservableCurrency;
use sp_runtime::traits::Zero;
use sp_runtime::DispatchError;
use sp_std::collections::btree_map::BTreeMap;

impl<T: Config> Pallet<T> {
	pub fn distribute_rewards(round: RoundIndex) -> DispatchResult {
		let mut delegation_info: BTreeMap<
			T::AssetId,
			Vec<DelegatorBond<T::AccountId, BalanceOf<T>, T::AssetId>>,
		> = BTreeMap::new();

		// Iterate through all operator snapshots for the given round
		// TODO: Add limits on number of rewards to process/payout
		for (_, operator_snapshot) in AtStake::<T>::iter_prefix(round) {
			for delegation in &operator_snapshot.delegations {
				delegation_info.entry(delegation.asset_id).or_default().push(delegation.clone());
			}
		}

		// Get the reward configuration
		if let Some(reward_config) = RewardConfigStorage::<T>::get() {
			// Distribute rewards for each asset
			for (asset_id, delegations) in delegation_info.iter() {
				if let Some(config) = reward_config.configs.get(asset_id) {
					// Calculate total amount and distribute rewards
					let total_amount: BalanceOf<T> =
						delegations.iter().fold(Zero::zero(), |acc, d| acc + d.amount);
					let cap: BalanceOf<T> = config.cap;

					if total_amount >= cap {
						// Calculate the total reward based on the APY
						let total_reward = Self::calculate_total_reward(config.apy, total_amount)?;

						for delegation in delegations {
							let reward = total_reward * delegation.amount / total_amount;
							// Logic to distribute reward to the delegator (e.g., mint or transfer tokens)
							Self::distribute_reward_to_delegator(&delegation.delegator, reward)?;
						}
					}
				}
			}
		}

		Ok(())
	}

	fn calculate_total_reward(
		apy: u128,
		total_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		//let total_reward = total_amount as u128 * apy / 100u32.into();
		Ok(Default::default())
	}

	fn distribute_reward_to_delegator(
		delegator: &T::AccountId,
		reward: BalanceOf<T>,
	) -> DispatchResult {
		// TODO : Implement the logic to distribute reward to the delegator
		Ok(())
	}
}
