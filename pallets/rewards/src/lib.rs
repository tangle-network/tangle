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

	/// Stores the user rewards for each user and asset combination
	#[pallet::storage]
	#[pallet::getter(fn user_rewards)]
	pub type UserRewards<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		Asset<T::AssetId>,
		UserRewardsOf<T>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn asset_rewards)]
	pub type AssetRewards<T: Config> =
		StorageMap<_, Blake2_128Concat, Asset<T::AssetId>, u128, ValueQuery>;

	/// Stores the whitelisted assets that can be used for rewards
	#[pallet::storage]
	#[pallet::getter(fn allowed_reward_assets)]
	pub type AllowedRewardAssets<T: Config> =
		StorageMap<_, Blake2_128Concat, Asset<T::AssetId>, bool, ValueQuery>;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Claim rewards for a specific asset and reward type
		#[pallet::weight(10_000)]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			asset: Asset<T::AssetId>,
			reward_type: RewardType,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
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
