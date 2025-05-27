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
//! - `apy`: Annual Perbillage Yield for the vault
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
pub mod weights;
pub use weights::*;
pub mod migrations;

use sp_std::vec::Vec;
use tangle_primitives::BlueprintId;
use tangle_primitives::services::types::{ServiceId, PricingModel};

/// The pallet's account ID.
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		PalletId,
		pallet_prelude::*,
		traits::{Currency, LockableCurrency, ReservableCurrency, ExistenceRequirement},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{Perbill, traits::{AccountIdConversion, Zero, Saturating}};
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
				Self::AssetId,
			>;

		/// The origin that can manage reward assets
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The max possible apy
		type MaxApy: Get<Perbill>;

		/// The max possible deposit cap
		type MaxDepositCap: Get<BalanceOf<Self>>;

		/// The max possible incentive cap
		type MaxIncentiveCap: Get<BalanceOf<Self>>;

		/// The min possible deposit cap
		type MinDepositCap: Get<BalanceOf<Self>>;

		/// The min possible incentive cap
		type MinIncentiveCap: Get<BalanceOf<Self>>;

		/// Weight information for the pallet
		type WeightInfo: WeightInfo;

		/// Max length for vault name
		#[pallet::constant]
		type MaxVaultNameLength: Get<u32>;

		/// Max length for vault logo URL/data
		#[pallet::constant]
		type MaxVaultLogoLength: Get<u32>;

		/// The origin that is allowed to set vault metadata.
		type VaultMetadataOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The maximum number of pending reward entries an operator can have.
		#[pallet::constant]
		type MaxPendingRewardsPerOperator: Get<u32>;
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
	pub type DecayRate<T: Config> = StorageValue<_, Perbill, ValueQuery>;

	/// Defines the structure for storing vault metadata.
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct VaultMetadata<T: Config> {
		pub name: BoundedVec<u8, T::MaxVaultNameLength>,
		pub logo: BoundedVec<u8, T::MaxVaultLogoLength>,
	}

	/// Storage for vault metadata.
	#[pallet::storage]
	#[pallet::getter(fn vault_metadata_store)]
	pub type VaultMetadataStore<T: Config> =
		StorageMap<_, Blake2_128Concat, T::VaultId, VaultMetadata<T>, OptionQuery>;

	/// Storage map from Operator AccountId to a list of pending rewards.
	/// Each reward entry is a tuple of (ServiceId, Amount).
	#[pallet::storage]
	#[pallet::getter(fn pending_operator_rewards)]
	pub type PendingOperatorRewards<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId, // Operator AccountId
		BoundedVec<(ServiceId, BalanceOf<T>), T::MaxPendingRewardsPerOperator>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Rewards have been claimed by an account
		RewardsClaimed { account: T::AccountId, asset: Asset<T::AssetId>, amount: BalanceOf<T> },
		/// Event emitted when an incentive APY and cap are set for a reward vault
		IncentiveAPYAndCapSet { vault_id: T::VaultId, apy: sp_runtime::Perbill, cap: BalanceOf<T> },
		/// Event emitted when a blueprint is whitelisted for rewards
		BlueprintWhitelisted { blueprint_id: BlueprintId },
		/// Asset has been updated to reward vault
		AssetUpdatedInVault { vault_id: T::VaultId, asset: Asset<T::AssetId>, action: AssetAction },
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
		DecayConfigUpdated { start_period: BlockNumberFor<T>, rate: Perbill },
		/// The number of blocks for APY calculation has been updated
		ApyBlocksUpdated { blocks: BlockNumberFor<T> },
		/// Metadata for a vault was set or updated.
		VaultMetadataSet {
			vault_id: T::VaultId,
			name: BoundedVec<u8, T::MaxVaultNameLength>,
			logo: BoundedVec<u8, T::MaxVaultLogoLength>,
		},
		/// Metadata for a vault was removed.
		VaultMetadataRemoved { vault_id: T::VaultId },
		/// Reward recorded
		RewardRecorded { operator: T::AccountId, service_id: ServiceId, amount: BalanceOf<T> },
		/// Operator rewards claimed
		OperatorRewardsClaimed { operator: T::AccountId, amount: BalanceOf<T> },
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
		/// Incentive cap is greater than max incentive cap
		IncentiveCapGreaterThanMaxIncentiveCap,
		/// Deposit cap is greater than max deposit cap
		DepositCapGreaterThanMaxDepositCap,
		/// Incentive cap is less than min incentive cap
		IncentiveCapLessThanMinIncentiveCap,
		/// Deposit cap is less than min deposit cap
		DepositCapLessThanMinDepositCap,
		/// Vault name exceeds the maximum allowed length.
		NameTooLong,
		/// Vault logo exceeds the maximum allowed length.
		LogoTooLong,
		/// Vault metadata not found for the given vault ID.
		VaultMetadataNotFound,
		/// Operator has no pending rewards to claim.
		NoRewardsToClaim,
		/// An arithmetic operation resulted in an overflow.
		ArithmeticOverflow,
		/// Failed to transfer funds.
		TransferFailed,
		/// Operator has too many pending rewards.
		TooManyPendingRewards,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
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
		#[pallet::weight(<T as Config>::WeightInfo::claim_rewards_other())]
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
		/// * `asset` - ID of the asset
		/// * `action` - Action to perform (Add/Remove)
		///
		/// # Errors
		///
		/// * [`Error::AssetAlreadyInVault`] - Asset already exists in vault
		/// * [`Error::AssetNotInVault`] - Asset does not exist in vault
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::manage_asset_reward_vault())]
		pub fn manage_asset_reward_vault(
			origin: OriginFor<T>,
			vault_id: T::VaultId,
			asset: Asset<T::AssetId>,
			action: AssetAction,
		) -> DispatchResult {
			let _who = T::ForceOrigin::ensure_origin(origin)?;

			match action {
				AssetAction::Add => Self::add_asset_to_vault(&vault_id, &asset)?,
				AssetAction::Remove => Self::remove_asset_from_vault(&vault_id, &asset)?,
			}

			Self::deposit_event(Event::AssetUpdatedInVault { vault_id, asset, action });

			Ok(())
		}

		/// Creates a new reward configuration for a specific vault.
		///
		/// # Arguments
		/// * `origin` - Origin of the call, must pass `ForceOrigin` check
		/// * `vault_id` - The ID of the vault to update
		/// * `new_config` - The new reward configuration containing:
		///   * `apy` - Annual Perbillage Yield for the vault
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
		#[pallet::weight(<T as Config>::WeightInfo::create_reward_vault())]
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
		///   * `apy` - Annual Perbillage Yield for the vault
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
		#[pallet::weight(<T as Config>::WeightInfo::update_vault_reward_config())]
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
		#[pallet::weight(<T as Config>::WeightInfo::update_decay_config())]
		pub fn update_decay_config(
			origin: OriginFor<T>,
			start_period: BlockNumberFor<T>,
			rate: Perbill,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;

			// Ensure rate is reasonable (max 10% decay)
			ensure!(rate <= Perbill::from_percent(10), Error::<T>::InvalidDecayRate);

			DecayStartPeriod::<T>::put(start_period);
			DecayRate::<T>::put(rate);

			Self::deposit_event(Event::DecayConfigUpdated { start_period, rate });
			Ok(())
		}

		/// Update the number of blocks used for APY calculation
		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::update_apy_blocks())]
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

		/// Set the metadata for a specific vault.
		///
		/// Parameters:
		/// - `origin`: The origin authorized to set metadata (e.g., root or a specific council).
		/// - `vault_id`: The account ID of the vault.
		/// - `name`: The name of the vault (bounded string).
		/// - `logo`: The logo URL or data for the vault (bounded string).
		///
		/// Emits `VaultMetadataSet` event on success.
		/// Requires `VaultMetadataOrigin`.
		#[pallet::call_index(8)]
		#[pallet::weight(<T as Config>::WeightInfo::set_vault_metadata())]
		pub fn set_vault_metadata(
			origin: OriginFor<T>,
			vault_id: T::VaultId,
			name: Vec<u8>,
			logo: Vec<u8>,
		) -> DispatchResult {
			T::VaultMetadataOrigin::ensure_origin(origin)?;

			let bounded_name: BoundedVec<u8, T::MaxVaultNameLength> =
				name.try_into().map_err(|_| Error::<T>::NameTooLong)?;
			let bounded_logo: BoundedVec<u8, T::MaxVaultLogoLength> =
				logo.try_into().map_err(|_| Error::<T>::LogoTooLong)?;

			let metadata =
				VaultMetadata::<T> { name: bounded_name.clone(), logo: bounded_logo.clone() };

			VaultMetadataStore::<T>::insert(vault_id, metadata);

			Self::deposit_event(Event::VaultMetadataSet {
				vault_id,
				name: bounded_name,
				logo: bounded_logo,
			});
			Ok(())
		}

		/// Remove the metadata associated with a specific vault.
		///
		/// Parameters:
		/// - `origin`: The origin authorized to remove metadata (e.g., root or a specific council).
		/// - `vault_id`: The account ID of the vault whose metadata should be removed.
		///
		/// Emits `VaultMetadataRemoved` event on success.
		/// Requires `VaultMetadataOrigin`.
		#[pallet::call_index(9)]
		#[pallet::weight(<T as Config>::WeightInfo::remove_vault_metadata())]
		pub fn remove_vault_metadata(origin: OriginFor<T>, vault_id: T::VaultId) -> DispatchResult {
			T::VaultMetadataOrigin::ensure_origin(origin)?;

			ensure!(
				VaultMetadataStore::<T>::contains_key(vault_id),
				Error::<T>::VaultMetadataNotFound
			);
			VaultMetadataStore::<T>::remove(vault_id);

			Self::deposit_event(Event::VaultMetadataRemoved { vault_id });
			Ok(())
		}

		/// Allows an operator to claim all their currently pending rewards.
		#[pallet::call_index(10)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
		pub fn claim_rewards(origin: OriginFor<T>) -> DispatchResult {
			let operator = ensure_signed(origin)?;

			// Retrieve and clear pending rewards for the operator.
			let pending_rewards = PendingOperatorRewards::<T>::take(&operator);
			ensure!(!pending_rewards.is_empty(), Error::<T>::NoRewardsToClaim);

			// Calculate the total amount to be claimed.
			let mut total_reward = BalanceOf::<T>::zero();
			for (_, amount) in pending_rewards.iter() {
				total_reward = total_reward.saturating_add(*amount);
			}

			// Transfer the total reward from the pallet account to the operator.
			T::Currency::transfer(
				&Self::account_id(),
				&operator,
				total_reward,
				ExistenceRequirement::KeepAlive, // Or AllowDeath depending on requirements
			).map_err(|_| Error::<T>::TransferFailed)?;

			// Emit an event.
			Self::deposit_event(Event::OperatorRewardsClaimed { operator, amount: total_reward });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account ID of the Rewards Pallet.
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}

	impl<T: Config> tangle_primitives::traits::RewardRecorder<T::AccountId, ServiceId, BalanceOf<T>> for Pallet<T> {
		type PricingModel = PricingModel<BlockNumberFor<T>, BalanceOf<T>>;

		fn record_reward(
			operator: &T::AccountId,
			service_id: ServiceId,
			amount: BalanceOf<T>,
			_model: &Self::PricingModel, // Model might be used later
		) -> DispatchResult {
			if amount == BalanceOf::<T>::zero() {
				return Ok(()); // No need to record zero rewards
			}

			// Attempt to append the new reward.
			// This handles the BoundedVec limit implicitly.
			let result = PendingOperatorRewards::<T>::try_mutate(operator, |rewards| {
				rewards.try_push((service_id, amount))
			});

			match result {
				Ok(_) => {
					// Emit event only if successful
					Self::deposit_event(Event::RewardRecorded { operator: operator.clone(), service_id, amount });
					Ok(())
				},
				Err(_) => {
					// Log an error or handle the case where the operator has too many pending rewards.
					// For now, we simply don't record the reward if the limit is reached.
					// Optionally, emit a specific event or error.
					log::warn!(
						"Failed to record reward for operator {:?}: Too many pending rewards.",
						operator
					);
					Ok(())
				}
			}
		}
	}
}
