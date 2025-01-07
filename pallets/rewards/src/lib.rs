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

//! # Tangle Rewards Pallet
//!
//! This pallet provides a reward management system for the Tangle network, enabling users to
//! accumulate and claim rewards.
//!
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod mock_evm;

#[cfg(test)]
mod tests;

// pub mod weights;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
use scale_info::TypeInfo;
use sp_runtime::Saturating;
use sp_std::collections::btree_map::BTreeMap;
use tangle_primitives::services::Asset;
pub mod types;
pub use types::*;
pub mod functions;
pub use functions::*;
pub mod impls;
use tangle_primitives::BlueprintId;
use tangle_primitives::MultiAssetDelegationInfo;

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
	use sp_runtime::traits::{AccountIdConversion, Zero};

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

	/// Stores the total score for each asset
	#[pallet::storage]
	#[pallet::getter(fn total_reward_vault_score)]
	pub type TotalRewardVaultScore<T: Config> =
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
			who: T::AccountId,
			vault_id: T::VaultId,
			asset_id: Asset<T::AssetId>,
			action: AssetAction,
		},
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
		ArithmeticError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Claim rewards for a specific asset and reward type
		#[pallet::weight(10_000)]
		pub fn claim_rewards(origin: OriginFor<T>, asset: Asset<T::AssetId>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// calculate and payout rewards
			Self::calculate_and_payout_rewards(&who, asset)?;

			// Emit event

			Ok(())
		}

		/// Whitelists a blueprint for rewards.
		///
		/// # Permissions
		///
		/// * Must be called by the force origin
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `blueprint_id` - ID of blueprint to whitelist
		// #[pallet::call_index(19)]
		// #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		// pub fn whitelist_blueprint_for_rewards(
		// 	origin: OriginFor<T>,
		// 	blueprint_id: BlueprintId,
		// ) -> DispatchResult {
		// 	T::ForceOrigin::ensure_origin(origin)?;

		// 	RewardConfigStorage::<T>::mutate(|maybe_config| {
		// 		let mut config = maybe_config.take().unwrap_or_else(|| RewardConfig {
		// 			configs: BTreeMap::new(),
		// 			whitelisted_blueprint_ids: Vec::new(),
		// 		});

		// 		if !config.whitelisted_blueprint_ids.contains(&blueprint_id) {
		// 			config.whitelisted_blueprint_ids.push(blueprint_id);
		// 		}

		// 		*maybe_config = Some(config);
		// 	});

		// 	Self::deposit_event(Event::BlueprintWhitelisted { blueprint_id });

		// 	Ok(())
		// }

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
		#[pallet::call_index(20)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn manage_asset_in_vault(
			origin: OriginFor<T>,
			vault_id: T::VaultId,
			asset_id: Asset<T::AssetId>,
			action: AssetAction,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match action {
				AssetAction::Add => Self::add_asset_to_vault(&vault_id, &asset_id)?,
				AssetAction::Remove => Self::remove_asset_from_vault(&vault_id, &asset_id)?,
			}

			Self::deposit_event(Event::AssetUpdatedInVault { who, vault_id, asset_id, action });

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
