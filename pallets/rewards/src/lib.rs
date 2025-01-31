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

//! # Rewards Pallet
//!
//! A flexible reward distribution system that supports multiple vaults with configurable reward
//! parameters.
//!
//! ## Overview
//!
//! The Rewards pallet provides a mechanism for distributing rewards to users who deposit assets
//! into various vaults. Each vault can have its own reward configuration, including APY rates and
//! deposit caps. The system supports both unlocked deposits and locked deposits with multipliers
//! for longer lock periods.
//!
//! ## Reward Vaults
//!
//! Each vault is identified by a unique `VaultId` and has its own reward configuration:
//! - `apy`: Annual Percentage Yield for the vault
//! - `deposit_cap`: Maximum amount that can be deposited
//! - `incentive_cap`: Maximum amount of incentives that can be distributed
//! - `boost_multiplier`: Optional multiplier to boost rewards
//!
//! ## Reward Calculation
//!
//! Rewards are calculated based on several factors:
//!
//! 1. Base Rewards: ```text Base Reward = APY * (user_deposit / total_deposits) * (total_deposits /
//!    deposit_capacity) ```
//!
//! 2. Locked Deposits: For locked deposits, additional rewards are calculated using: ```text Lock
//!    Reward = Base Reward * lock_multiplier * (remaining_lock_time / total_lock_time) ```
//!
//! Lock multipliers increase rewards based on lock duration:
//! - One Month: 1.1x
//! - Two Months: 1.2x
//! - Three Months: 1.3x
//! - Six Months: 1.6x
//!
//! ## Notes
//!
//! - The reward vaults will consider all assets in parity, so only add the same type of asset in
//!   the same vault.
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod mock_evm;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use scale_info::TypeInfo;
use tangle_primitives::services::Asset;
pub mod types;
pub use types::*;
pub mod functions;
pub mod impls;

use sp_std::vec::Vec;
use tangle_primitives::BlueprintId;

/// The pallet's account ID.
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, LockableCurrency, ReservableCurrency},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{traits::AccountIdConversion, Percent};
	use tangle_primitives::rewards::LockMultiplier;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency type used for managing balances.
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;

		/// Type representing the unique ID of an asset.
		type AssetId: Parameter + Member + Copy + Ord + Default + MaxEncodedLen + TypeInfo;

		/// The pallet's account ID.
		type PalletId: Get<PalletId>;

		/// Type representing the unique ID of a vault.
		type VaultId: Parameter
			+ Member
			+ Copy
			+ MaybeSerializeDeserialize
			+ Ord
			+ Default
			+ MaxEncodedLen
			+ TypeInfo;

		/// Manager for getting operator stake and delegation info
		type DelegationManager: tangle_primitives::traits::MultiAssetDelegationInfo<
			Self::AccountId,
			BalanceOf<Self>,
			BlockNumberFor<Self>,
			AssetId = Self::AssetId,
		>;

		/// The origin that can manage reward assets
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Stores the total score for each vault
	/// The difference between this and total_reward_vault_deposit is that this includes locked
	/// deposits multiplied by the lock multiplier
	#[pallet::storage]
	#[pallet::getter(fn total_reward_vault_score)]
	pub type TotalRewardVaultScore<T: Config> =
		StorageMap<_, Blake2_128Concat, T::VaultId, BalanceOf<T>, ValueQuery>;

	/// Stores the total deposit for each vault
	#[pallet::storage]
	#[pallet::getter(fn total_reward_vault_deposit)]
	pub type TotalRewardVaultDeposit<T: Config> =
		StorageMap<_, Blake2_128Concat, T::VaultId, BalanceOf<T>, ValueQuery>;

	/// Stores the service reward for a given user
	#[pallet::storage]
	#[pallet::getter(fn user_reward_score)]
	pub type UserServiceReward<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		Asset<T::AssetId>,
		BalanceOf<T>,
		ValueQuery,
	>;

	/// Stores the service reward for a given user
	#[pallet::storage]
	#[pallet::getter(fn user_claimed_reward)]
	pub type UserClaimedReward<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::VaultId,
		(BlockNumberFor<T>, BalanceOf<T>),
	>;

	#[pallet::storage]
	#[pallet::getter(fn reward_vaults)]
	/// Storage for the reward vaults
	pub type RewardVaults<T: Config> =
		StorageMap<_, Blake2_128Concat, T::VaultId, Vec<Asset<T::AssetId>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn asset_reward_vault_lookup)]
	/// Storage for the reward vaults
	pub type AssetLookupRewardVaults<T: Config> =
		StorageMap<_, Blake2_128Concat, Asset<T::AssetId>, T::VaultId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn reward_config)]
	/// Storage for the reward configuration, which includes APY, cap for assets
	pub type RewardConfigStorage<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::VaultId,
		RewardConfigForAssetVault<BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn reward_vaults_pot_account)]
	/// Storage for the reward vaults
	pub type RewardVaultsPotAccount<T: Config> =
		StorageMap<_, Blake2_128Concat, T::VaultId, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn blocks_for_apy)]
	/// Storage for the reward configuration, which includes APY, cap for assets
	pub type ApyBlocks<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn decay_start_period)]
	/// Number of blocks after which decay starts (e.g., 432000 for 30 days with 6s blocks)
	pub type DecayStartPeriod<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn decay_rate)]
	/// Per-block decay rate in basis points (1/10000). e.g., 1 = 0.01% per block
	pub type DecayRate<T: Config> = StorageValue<_, Percent, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Rewards have been claimed by an account
		RewardsClaimed { account: T::AccountId, asset: Asset<T::AssetId>, amount: BalanceOf<T> },
		/// Event emitted when an incentive APY and cap are set for a reward vault
		IncentiveAPYAndCapSet { vault_id: T::VaultId, apy: sp_runtime::Percent, cap: BalanceOf<T> },
		/// Event emitted when a blueprint is whitelisted for rewards
		BlueprintWhitelisted { blueprint_id: BlueprintId },
		/// Asset has been updated to reward vault
		AssetUpdatedInVault {
			vault_id: T::VaultId,
			asset_id: Asset<T::AssetId>,
			action: AssetAction,
		},
		/// Vault reward config updated
		VaultRewardConfigUpdated {
			vault_id: T::VaultId,
			new_config: RewardConfigForAssetVault<BalanceOf<T>>,
		},
		/// Vault created
		RewardVaultCreated {
			vault_id: T::VaultId,
			new_config: RewardConfigForAssetVault<BalanceOf<T>>,
			pot_account: T::AccountId,
		},
		/// Total score in vault updated
		TotalScoreUpdated {
			vault_id: T::VaultId,
			asset: Asset<T::AssetId>,
			total_score: BalanceOf<T>,
			lock_multiplier: Option<LockMultiplier>,
		},
		/// Total deposit in vault updated
		TotalDepositUpdated {
			vault_id: T::VaultId,
			asset: Asset<T::AssetId>,
			total_deposit: BalanceOf<T>,
		},
		/// Decay configuration was updated
		DecayConfigUpdated { start_period: BlockNumberFor<T>, rate: Percent },
		/// The number of blocks for APY calculation has been updated
		ApyBlocksUpdated { blocks: BlockNumberFor<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// No rewards available to claim
		NoRewardsAvailable,
		/// Insufficient rewards balance in pallet account
		InsufficientRewardsBalance,
		/// Asset is not whitelisted for rewards
		AssetNotWhitelisted,
		/// Asset is already whitelisted
		AssetAlreadyWhitelisted,
		/// Invalid APY value
		InvalidAPY,
		/// Asset already exists in a reward vault
		AssetAlreadyInVault,
		/// Asset not found in reward vault
		AssetNotInVault,
		/// The reward vault does not exist
		VaultNotFound,
		/// Error returned when trying to add a blueprint ID that already exists.
		DuplicateBlueprintId,
		/// Error returned when trying to remove a blueprint ID that doesn't exist.
		BlueprintIdNotFound,
		/// Error returned when the reward configuration for the vault is not found.
		RewardConfigNotFound,
		/// Arithmetic operation caused an overflow
		CannotCalculatePropotionalApy,
		/// Error returned when trying to calculate reward per block
		CannotCalculateRewardPerBlock,
		/// Incentive cap is greater than deposit cap
		IncentiveCapGreaterThanDepositCap,
		/// Boost multiplier must be 1
		BoostMultiplierMustBeOne,
		/// Vault already exists
		VaultAlreadyExists,
		/// Total deposit is less than incentive cap
		TotalDepositLessThanIncentiveCap,
		/// Pot account not found
		PotAlreadyExists,
		/// Pot account not found
		PotAccountNotFound,
		/// Decay rate is too high
		InvalidDecayRate,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// The number of blocks used for APY calculation
		pub apy_blocks: BlockNumberFor<T>,
		/// Number of blocks after which decay starts
		pub decay_start_period: BlockNumberFor<T>,
		/// Per-block decay rate in basis points
		pub decay_rate: Percent,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				// Default to 1 year worth of blocks (assuming 6s block time)
				apy_blocks: BlockNumberFor::<T>::from(5_256_000u32),
				// Default to 30 days worth of blocks
				decay_start_period: BlockNumberFor::<T>::from(432000u32),
				// Default to 1% per block
				decay_rate: Percent::from_percent(1),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			ApyBlocks::<T>::put(self.apy_blocks);
			DecayStartPeriod::<T>::put(self.decay_start_period);
			DecayRate::<T>::put(self.decay_rate);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Claim rewards for a specific asset and reward type
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn claim_rewards(origin: OriginFor<T>, asset: Asset<T::AssetId>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// calculate and payout rewards
			Self::calculate_and_payout_rewards(&who, asset)?;

			Ok(())
		}

		/// Claim rewards for another account
		///
		/// The dispatch origin must be signed.
		///
		/// Parameters:
		/// - `who`: The account to claim rewards for
		/// - `asset`: The asset to claim rewards for
		///
		/// Emits `RewardsClaimed` event when successful.
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn claim_rewards_other(
			origin: OriginFor<T>,
			who: T::AccountId,
			asset: Asset<T::AssetId>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			// calculate and payout rewards for the specified account
			Self::calculate_and_payout_rewards(&who, asset)?;

			Ok(())
		}

		/// Manage asset id to vault rewards.
		///
		/// # Permissions
		///
		/// * Must be signed by an authorized account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `vault_id` - ID of the vault
		/// * `asset_id` - ID of the asset
		/// * `action` - Action to perform (Add/Remove)
		///
		/// # Errors
		///
		/// * [`Error::AssetAlreadyInVault`] - Asset already exists in vault
		/// * [`Error::AssetNotInVault`] - Asset does not exist in vault
		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn manage_asset_reward_vault(
			origin: OriginFor<T>,
			vault_id: T::VaultId,
			asset_id: Asset<T::AssetId>,
			action: AssetAction,
		) -> DispatchResult {
			let _who = T::ForceOrigin::ensure_origin(origin)?;

			match action {
				AssetAction::Add => Self::add_asset_to_vault(&vault_id, &asset_id)?,
				AssetAction::Remove => Self::remove_asset_from_vault(&vault_id, &asset_id)?,
			}

			Self::deposit_event(Event::AssetUpdatedInVault { vault_id, asset_id, action });

			Ok(())
		}

		/// Creates a new reward configuration for a specific vault.
		///
		/// # Arguments
		/// * `origin` - Origin of the call, must pass `ForceOrigin` check
		/// * `vault_id` - The ID of the vault to update
		/// * `new_config` - The new reward configuration containing:
		///   * `apy` - Annual Percentage Yield for the vault
		///   * `deposit_cap` - Maximum amount that can be deposited
		///   * `incentive_cap` - Maximum amount of incentives that can be distributed
		///   * `boost_multiplier` - Optional multiplier to boost rewards
		///
		/// # Events
		/// * `VaultRewardConfigUpdated` - Emitted when vault reward config is updated
		///
		/// # Errors
		/// * `BadOrigin` - If caller is not authorized through `ForceOrigin`
		/// * `IncentiveCapGreaterThanDepositCap` - If incentive cap is greater than deposit cap
		/// * `BoostMultiplierMustBeOne` - If boost multiplier is not 1
		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn create_reward_vault(
			origin: OriginFor<T>,
			vault_id: T::VaultId,
			new_config: RewardConfigForAssetVault<BalanceOf<T>>,
		) -> DispatchResult {
			let _who = T::ForceOrigin::ensure_origin(origin)?;

			// ensure vault does not already exist
			ensure!(
				!RewardConfigStorage::<T>::contains_key(vault_id),
				Error::<T>::VaultAlreadyExists
			);

			// Validate the new configuration
			Self::validate_reward_config(&new_config)?;

			// Initialize the vault pot for rewards
			let pot_account = Self::create_reward_vault_pot(vault_id)?;

			RewardConfigStorage::<T>::insert(vault_id, new_config.clone());
			Self::deposit_event(Event::RewardVaultCreated { vault_id, new_config, pot_account });
			Ok(())
		}

		/// Updates the reward configuration for a specific vault.
		///
		/// # Arguments
		/// * `origin` - Origin of the call, must pass `ForceOrigin` check
		/// * `vault_id` - The ID of the vault to update
		/// * `new_config` - The new reward configuration containing:
		///   * `apy` - Annual Percentage Yield for the vault
		///   * `deposit_cap` - Maximum amount that can be deposited
		///   * `incentive_cap` - Maximum amount of incentives that can be distributed
		///   * `boost_multiplier` - Optional multiplier to boost rewards
		///
		/// # Events
		/// * `VaultRewardConfigUpdated` - Emitted when vault reward config is updated
		///
		/// # Errors
		/// * `BadOrigin` - If caller is not authorized through `ForceOrigin`
		/// * `IncentiveCapGreaterThanDepositCap` - If incentive cap is greater than deposit cap
		/// * `BoostMultiplierMustBeOne` - If boost multiplier is not 1
		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn update_vault_reward_config(
			origin: OriginFor<T>,
			vault_id: T::VaultId,
			new_config: RewardConfigForAssetVault<BalanceOf<T>>,
		) -> DispatchResult {
			let _who = T::ForceOrigin::ensure_origin(origin)?;

			// Validate the new configuration
			Self::validate_reward_config(&new_config)?;

			RewardConfigStorage::<T>::try_mutate(vault_id, |config| -> DispatchResult {
				// ensure config exists
				ensure!(config.is_some(), Error::<T>::VaultNotFound);

				// update config
				*config = Some(new_config.clone());

				Self::deposit_event(Event::VaultRewardConfigUpdated { vault_id, new_config });

				Ok(())
			})
		}

		/// Update the decay configuration
		#[pallet::call_index(6)]
		#[pallet::weight(T::DbWeight::get().writes(2))]
		pub fn update_decay_config(
			origin: OriginFor<T>,
			start_period: BlockNumberFor<T>,
			rate: Percent,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;

			// Ensure rate is reasonable (max 10% decay)
			ensure!(rate <= Percent::from_percent(10), Error::<T>::InvalidDecayRate);

			DecayStartPeriod::<T>::put(start_period);
			DecayRate::<T>::put(rate);

			Self::deposit_event(Event::DecayConfigUpdated { start_period, rate });
			Ok(())
		}

		/// Update the number of blocks used for APY calculation
		#[pallet::call_index(7)]
		#[pallet::weight(T::DbWeight::get().writes(1))]
		pub fn update_apy_blocks(
			origin: OriginFor<T>,
			blocks: BlockNumberFor<T>,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;

			// Update the storage
			ApyBlocks::<T>::put(blocks);

			Self::deposit_event(Event::ApyBlocksUpdated { blocks });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account ID of the rewards pot.
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}
