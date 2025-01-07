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
use tangle_primitives::services::Asset;
pub mod types;
pub use types::*;
pub mod functions;
pub use functions::*;
pub mod impls;

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

		/// The maximum amount of rewards that can be claimed per asset per user.
		type MaxUniqueServiceRewards: Get<u32> + MaxEncodedLen + TypeInfo;

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
		StorageMap<_, Blake2_128Concat, T::VaultId, u128, ValueQuery>;

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
	/// Storage for the reward configuration, which includes APY, cap for assets, and whitelisted
	/// blueprints.
	pub type RewardConfigStorage<T: Config> =
		StorageValue<_, RewardConfig<T::VaultId, BalanceOf<T>>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Rewards have been added for an account
		RewardsAdded {
			account: T::AccountId,
			asset: Asset<T::AssetId>,
			amount: BalanceOf<T>,
			reward_type: RewardType,
		},
		/// Rewards have been claimed by an account
		RewardsClaimed {
			account: T::AccountId,
			asset: Asset<T::AssetId>,
			amount: BalanceOf<T>,
			reward_type: RewardType,
		},
		/// Asset has been whitelisted for rewards
		AssetWhitelisted { asset: Asset<T::AssetId> },
		/// Asset has been removed from whitelist
		AssetRemoved { asset: Asset<T::AssetId> },
		/// Asset rewards have been updated
		AssetRewardsUpdated { asset: Asset<T::AssetId>, total_score: u128, users_updated: u32 },
		/// Asset APY has been updated
		AssetApyUpdated { asset: Asset<T::AssetId>, apy_basis_points: u32 },
	}

	/// Type of reward being added or claimed
	#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
	pub enum RewardType {
		Restaking,
		Boost,
		Service,
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
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Claim rewards for a specific asset and reward type
		#[pallet::weight(10_000)]
		pub fn claim_rewards(origin: OriginFor<T>, asset: Asset<T::AssetId>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure the asset is whitelisted
			ensure!(Self::is_asset_whitelisted(asset), Error::<T>::AssetNotWhitelisted);

			// Get user rewards snapshot
			let rewards = Self::user_rewards(&who, asset);

			// Calculate user's score based on their stake and lock period
			let user_score = functions::calculate_user_score::<T>(asset, &rewards);

			// Calculate APY distribution based on user's score
			let apy = functions::calculate_apy_distribution::<T>(asset, user_score);

			// Calculate reward amount based on APY and elapsed time
			let reward_amount = match reward_type {
				RewardType::Boost => {
					// For boost rewards, calculate based on locked amount and APY
					let locked_amount = rewards.boost_rewards.amount;
					let elapsed_time = frame_system::Pallet::<T>::block_number()
						.saturating_sub(rewards.boost_rewards.expiry);

					// Convert APY to per-block rate (assuming 6 second blocks)
					// APY / (blocks per year) = reward rate per block
					// blocks per year = (365 * 24 * 60 * 60) / 6 = 5,256,000
					let blocks_per_year = T::BlocksPerYear::get();
					let reward_rate = apy.mul_floor(locked_amount) / blocks_per_year.into();

					reward_rate.saturating_mul(elapsed_time.into())
				},
				RewardType::Service => {
					// For service rewards, use the accumulated service rewards
					rewards.service_rewards
				},
				RewardType::Restaking => {
					// For restaking rewards, use the accumulated restaking rewards
					rewards.restaking_rewards
				},
			};

			// Ensure there are rewards to claim
			ensure!(!.is_zero(), Error::<T>::NoRewardsAvailable);

			// Transfer rewards to user
			// Note: This assumes the pallet account has sufficient balance
			let pallet_account = Self::account_id();
			T::Currency::transfer(
				&pallet_account,
				&who,
				reward_amount,
				frame_support::traits::ExistenceRequirement::KeepAlive,
			)?;

			// Reset the claimed reward type
			match reward_type {
				RewardType::Boost => {
					// For boost rewards, update the expiry to current block
					Self::update_user_rewards(
						&who,
						asset,
						UserRewards {
							boost_rewards: BoostInfo {
								expiry: frame_system::Pallet::<T>::block_number(),
								..rewards.boost_rewards
							},
							..rewards
						},
					);
				},
				RewardType::Service => {
					// Reset service rewards to zero
					Self::update_user_rewards(
						&who,
						asset,
						UserRewards { service_rewards: Zero::zero(), ..rewards },
					);
				},
				RewardType::Restaking => {
					// Reset restaking rewards to zero
					Self::update_user_rewards(
						&who,
						asset,
						UserRewards { restaking_rewards: Zero::zero(), ..rewards },
					);
				},
			}

			// Emit event
			Self::deposit_event(Event::RewardsClaimed {
				account: who,
				asset,
				amount: reward_amount,
				reward_type,
			});

			Ok(())
		}

		/// Update APY for an asset
		#[pallet::call_index(6)]
		#[pallet::weight(10_000)]
		pub fn update_asset_apy(
			origin: OriginFor<T>,
			asset: Asset<T::AssetId>,
			apy_basis_points: u32,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(Self::is_asset_whitelisted(asset), Error::<T>::AssetNotWhitelisted);
			ensure!(apy_basis_points <= 10000, Error::<T>::InvalidAPY); // Max 100%

			// Update APY
			AssetApy::<T>::insert(asset, apy_basis_points);

			// Emit event
			Self::deposit_event(Event::AssetApyUpdated { asset, apy_basis_points });

			Ok(())
		}

		/// Sets the APY and cap for a specific asset.
		///
		/// # Permissions
		///
		/// * Must be called by the force origin
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `vault_id` - ID of the vault
		/// * `apy` - Annual percentage yield (max 10%)
		/// * `cap` - Required deposit amount for full APY
		///
		/// # Errors
		///
		/// * [`Error::APYExceedsMaximum`] - APY exceeds 10% maximum
		/// * [`Error::CapCannotBeZero`] - Cap amount cannot be zero
		#[pallet::call_index(18)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn set_incentive_apy_and_cap(
			origin: OriginFor<T>,
			vault_id: T::VaultId,
			apy: sp_runtime::Percent,
			cap: BalanceOf<T>,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;

			ensure!(apy <= sp_runtime::Percent::from_percent(10), Error::<T>::APYExceedsMaximum);
			ensure!(!cap.is_zero(), Error::<T>::CapCannotBeZero);

			RewardConfigStorage::<T>::mutate(|maybe_config| {
				let mut config = maybe_config.take().unwrap_or_else(|| RewardConfig {
					configs: BTreeMap::new(),
					whitelisted_blueprint_ids: Vec::new(),
				});

				config.configs.insert(vault_id, RewardConfigForAssetVault { apy, cap });

				*maybe_config = Some(config);
			});

			Self::deposit_event(Event::IncentiveAPYAndCapSet { vault_id, apy, cap });

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
		#[pallet::call_index(19)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn whitelist_blueprint_for_rewards(
			origin: OriginFor<T>,
			blueprint_id: BlueprintId,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;

			RewardConfigStorage::<T>::mutate(|maybe_config| {
				let mut config = maybe_config.take().unwrap_or_else(|| RewardConfig {
					configs: BTreeMap::new(),
					whitelisted_blueprint_ids: Vec::new(),
				});

				if !config.whitelisted_blueprint_ids.contains(&blueprint_id) {
					config.whitelisted_blueprint_ids.push(blueprint_id);
				}

				*maybe_config = Some(config);
			});

			Self::deposit_event(Event::BlueprintWhitelisted { blueprint_id });

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

		/// Check if an asset is whitelisted for rewards
		pub fn is_asset_whitelisted(asset: Asset<T::AssetId>) -> bool {
			AllowedRewardAssets::<T>::get(&asset)
		}
	}
}
