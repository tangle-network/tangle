// This file is part of Webb.

// Copyright (C) 2021 Webb Technologies Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//!
//! # Collator Rewards Pallet
//!
//! A simple module for managing collator rewards. The collator rewards are initially paid out from
//! the collator reward pot alloted at the time of genesis, every block `RewardAmount` is paid out
//! to the block author along with the fees and tips. Once the initial genesis supply for collator
//! rewards has been exhausted, the pallet will issue new amount of currency every block to continue
//! rewarding the collators
//!
//! ## Overview
//!
//! The collator rewards pallet provides function to handle distribution of rewards for collators of
//! EGG network
//!
//! The supported dispatchable functions are documented in the [`Call`] enum.
//!
//! ## Interface
//!
//! ### Permissioned Functions
//!
//! * `force_set_reward_amount`: Set the reward amount to be paid out every block to collators
#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		sp_runtime::traits::AccountIdConversion,
		traits::{Currency, ExistenceRequirement, WithdrawReasons},
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::NegativeImbalance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency implementation of the runtime
		type Currency: Currency<Self::AccountId>;
		/// The origin permitted to set the reward amount
		type ForceOrigin: EnsureOrigin<Self::Origin>;
		/// PalletId for this pallet
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	/// The RewardAmount to be paid out every block to author
	#[pallet::storage]
	#[pallet::getter(fn reward_amount)]
	pub type RewardAmount<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Reward Amount set to value
		RewardAmountSet(BalanceOf<T>),
		/// Reward paid to AccountId
		CollatorRewarded { amount: BalanceOf<T>, account: T::AccountId },
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the reward amount to be paid out every block to collators
		/// Can only be called by the ForceOrigin
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn force_set_reward_amount(
			origin: OriginFor<T>,
			reward_amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			T::ForceOrigin::ensure_origin(origin)?;
			// Update storage.
			<RewardAmount<T>>::put(reward_amount);
			// Emit an event.
			Self::deposit_event(Event::RewardAmountSet(reward_amount));
			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Withdraw the reward to pay to the collator account.
		/// This function returns a NegativeImbalance as the result of withdrawal/issuance. The
		/// pallet account is checked for balance to process this withdrawal. If the withdrawal from
		/// pallet account fails, new issuance is created.
		pub fn withdraw_reward() -> Result<NegativeImbalanceOf<T>, DispatchError> {
			// check if the pallet account has the balance to withdraw the amount
			// else we issue the newly created amount
			match T::Currency::withdraw(
				&Self::account_id(),
				RewardAmount::<T>::get(),
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
			) {
				Ok(imbalance) => Ok(imbalance),
				Err(_) => {
					// Issue the amount to reward collators
					let issued_amount = T::Currency::issue(RewardAmount::<T>::get());
					Ok(issued_amount)
				},
			}
		}

		/// the pallet account_id
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}
