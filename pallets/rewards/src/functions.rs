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
use crate::Error;
use crate::Event;
use crate::RewardConfigForAssetVault;
use crate::RewardConfigStorage;
use crate::RewardVaults;
use crate::TotalRewardVaultScore;
use crate::UserClaimedReward;
use crate::{BalanceOf, Config, Pallet};
use frame_support::ensure;
use frame_support::traits::Currency;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::traits::{CheckedDiv, CheckedMul};
use sp_runtime::traits::{Saturating, Zero};
use sp_runtime::DispatchError;
use sp_runtime::DispatchResult;
use sp_std::vec::Vec;
use tangle_primitives::services::Asset;
use tangle_primitives::traits::MultiAssetDelegationInfo;
use tangle_primitives::types::rewards::UserDepositWithLocks;

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

	pub fn calculate_rewards(
		account_id: &T::AccountId,
		asset: Asset<T::AssetId>,
	) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
		// find the vault for the asset id
		// if the asset is not in a reward vault, do nothing
		let vault_id =
			AssetLookupRewardVaults::<T>::get(asset).ok_or(Error::<T>::AssetNotInVault)?;

		// lets read the user deposits from the delegation manager
		let deposit_info =
			T::DelegationManager::get_user_deposit_with_locks(&account_id.clone(), asset)
				.ok_or(Error::<T>::NoRewardsAvailable)?;

		// read the asset reward config
		let reward_config = RewardConfigStorage::<T>::get(vault_id);

		// find the total vault score
		let total_score = TotalRewardVaultScore::<T>::get(vault_id);

		// get the users last claim
		let last_claim = UserClaimedReward::<T>::get(account_id, vault_id);

		Self::calculate_deposit_rewards_with_lock_multiplier(
			total_score,
			deposit_info,
			reward_config.ok_or(Error::<T>::RewardConfigNotFound)?,
			last_claim,
		)
	}

	/// Calculates and pays out rewards for a given account and asset.
	///
	/// This function orchestrates the reward calculation and payout process by:
	/// 1. Finding the vault associated with the asset
	/// 2. Retrieving user deposit information including any locked amounts
	/// 3. Calculating rewards based on deposit amounts, lock periods, and APY
	///
	/// # Arguments
	/// * `account_id` - The account to calculate rewards for
	/// * `asset` - The asset to calculate rewards for
	///
	/// # Returns
	/// * `Ok(BalanceOf<T>)` - The total rewards calculated
	/// * `Err(DispatchError)` - If any of the following conditions are met:
	///   - Asset is not in a reward vault
	///   - No rewards are available for the account
	///   - Reward configuration is not found for the vault
	///   - Arithmetic overflow occurs during calculation
	///
	/// # Assumptions
	/// * The asset must be registered in a reward vault
	/// * The reward configuration must exist for the vault
	pub fn calculate_and_payout_rewards(
		account_id: &T::AccountId,
		asset: Asset<T::AssetId>,
	) -> Result<BalanceOf<T>, DispatchError> {
		// find the vault for the asset id
		// if the asset is not in a reward vault, do nothing
		let vault_id =
			AssetLookupRewardVaults::<T>::get(asset).ok_or(Error::<T>::AssetNotInVault)?;

		let (total_rewards, rewards_to_be_paid) = Self::calculate_rewards(account_id, asset)?;

		// mint new TNT rewards and trasnfer to the user
		let _ = T::Currency::deposit_creating(account_id, rewards_to_be_paid);

		// update the last claim
		UserClaimedReward::<T>::insert(
			account_id,
			vault_id,
			(frame_system::Pallet::<T>::block_number(), total_rewards),
		);

		Self::deposit_event(Event::RewardsClaimed {
			account: account_id.clone(),
			asset,
			amount: rewards_to_be_paid,
		});

		Ok(total_rewards)
	}

	/// Calculates rewards for deposits considering both unlocked amounts and locked amounts with their respective multipliers.
	///
	/// The reward calculation follows these formulas:
	/// 1. For unlocked amounts:
	///    ```text
	///    base_reward = APY * (user_deposit / total_deposits) * (total_deposits / deposit_capacity)
	///    ```
	///
	/// 2. For locked amounts:
	///    ```text
	///    lock_reward = amount * APY * lock_multiplier * (remaining_lock_time / total_lock_time)
	///    ```
	///
	/// # Arguments
	/// * `total_asset_score` - Total score for the asset across all deposits
	/// * `deposit` - User's deposit information including locked amounts
	/// * `reward` - Reward configuration for the asset vault
	/// * `last_claim` - Timestamp and amount of user's last reward claim
	///
	/// # Returns
	/// * `Ok(BalanceOf<T>)` - The calculated rewards
	/// * `Err(DispatchError)` - If any arithmetic operation overflows
	///
	/// # Assumptions and Constraints
	/// * Lock multipliers are fixed at: 1x (1 month), 2x (2 months), 3x (3 months), 6x (6 months)
	/// * APY is applied proportionally to the lock period remaining
	/// * Rewards scale with:
	///   - The proportion of user's deposit to total deposits
	///   - The proportion of total deposits to deposit capacity
	///   - The lock multiplier (if applicable)
	///   - The remaining time in the lock period
	///
	pub fn calculate_deposit_rewards_with_lock_multiplier(
		total_asset_score: BalanceOf<T>,
		deposit: UserDepositWithLocks<BalanceOf<T>, BlockNumberFor<T>>,
		reward: RewardConfigForAssetVault<BalanceOf<T>>,
		last_claim: Option<(BlockNumberFor<T>, BalanceOf<T>)>,
	) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
		// The formula for rewards:
		// Base Reward = APY * (user_deposit / total_deposits) * (total_deposits / deposit_capacity)
		// For locked amounts: Base Reward * lock_multiplier * (remaining_lock_time / total_lock_time)

		let asset_apy = reward.apy;
		let deposit_capacity = reward.deposit_cap;

		// Start with unlocked amount as base score
		let mut total_rewards = deposit.unlocked_amount;

		// Get the current block and last claim block
		let current_block = frame_system::Pallet::<T>::block_number();
		let last_claim_block = last_claim.map(|(block, _)| block).unwrap_or(current_block);

		// Calculate base reward rate
		// APY * (deposit / total_deposits) * (total_deposits / capacity)
		let base_reward_rate = if !total_asset_score.is_zero() {
			let deposit_ratio = total_rewards
				.checked_mul(&total_rewards)
				.and_then(|v| v.checked_div(&total_asset_score))
				.ok_or(Error::<T>::ArithmeticError)?;

			let capacity_ratio = total_asset_score
				.checked_div(&deposit_capacity)
				.ok_or(Error::<T>::ArithmeticError)?;

			asset_apy.mul_floor(deposit_ratio.saturating_mul(capacity_ratio))
		} else {
			Zero::zero()
		};

		total_rewards = total_rewards.saturating_add(base_reward_rate);

		// Add rewards for locked amounts if any exist
		if let Some(locks) = deposit.amount_with_locks {
			for lock in locks {
				if lock.expiry_block > last_claim_block {
					// Calculate remaining lock time as a ratio
					let blocks_remaining: u32 =
						TryInto::<u32>::try_into(lock.expiry_block.saturating_sub(current_block))
							.map_err(|_| Error::<T>::ArithmeticError)?;

					let total_lock_blocks = lock.lock_multiplier.get_blocks();
					let time_ratio = BalanceOf::<T>::from(blocks_remaining)
						.checked_div(&BalanceOf::<T>::from(total_lock_blocks))
						.ok_or(Error::<T>::ArithmeticError)?;

					// Calculate lock reward:
					// amount * APY * multiplier * time_ratio
					let multiplier = BalanceOf::<T>::from(lock.lock_multiplier.value());
					let lock_reward = asset_apy
						.mul_floor(lock.amount)
						.saturating_mul(multiplier)
						.saturating_mul(time_ratio);

					total_rewards = total_rewards.saturating_add(lock_reward);
				}
			}
		}

		// lets remove any already claimed rewards
		let rewards_to_be_paid = total_rewards
			.saturating_sub(last_claim.map(|(_, amount)| amount).unwrap_or(Zero::zero()));

		Ok((total_rewards, rewards_to_be_paid))
	}
}
