// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Pallet to process claims from Ethereum addresses.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::all)]

use codec::MaxEncodedLen;
use frame_support::{
	ensure,
	traits::{Currency, Get, LockIdentifier, LockableCurrency, OnUnbalanced},
	CloneNoBound, EqNoBound, PalletId, PartialEqNoBound, RuntimeDebugNoBound,
};
pub use pallet::*;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{codec, traits::Zero};
use sp_std::{convert::TryInto, prelude::*, vec};
use tangle_primitives::{roles::RoleType, traits::roles::RolesHandler};
mod impls;
#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;
mod weights;
pub use weights::WeightInfo;

/// The ledger of a (bonded) stash.
#[derive(
	PartialEqNoBound,
	EqNoBound,
	CloneNoBound,
	Encode,
	Decode,
	RuntimeDebugNoBound,
	TypeInfo,
	MaxEncodedLen,
)]
#[scale_info(skip_type_params(T))]
pub struct RoleStakingLedger<T: Config> {
	/// The stash account whose balance is actually locked and at stake.
	pub stash: T::AccountId,
	/// The total amount of the stash's balance that we are currently accounting for.
	/// It's just `active` plus all the `unlocking` balances.
	#[codec(compact)]
	pub total_locked: BalanceOf<T>,
}

impl<T: Config> RoleStakingLedger<T> {
	/// Initializes the default object using the given `validator`.
	pub fn default_from(stash: T::AccountId) -> Self {
		Self { stash, total_locked: Zero::zero() }
	}

	pub fn is_empty(&self) -> bool {
		self.total_locked.is_zero()
	}
}

pub type CurrencyOf<T> = <T as pallet::Config>::Currency;
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

const ROLES_STAKING_ID: LockIdentifier = *b"rstaking";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: LockableCurrency<
			Self::AccountId,
			Moment = BlockNumberFor<Self>,
			Balance = Self::CurrencyBalance,
		>;
		/// Just the `Currency::Balance` type; we have this item to allow us to constrain it to
		/// `From<u64>`.
		type CurrencyBalance: sp_runtime::traits::AtLeast32BitUnsigned
			+ codec::FullCodec
			+ Copy
			+ MaybeSerializeDeserialize
			+ sp_std::fmt::Debug
			+ Default
			+ From<u64>
			+ TypeInfo
			+ MaxEncodedLen;

		/// Handler for the unbalanced reduction when slashing a staker.
		type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
		/// Role assigned to the validator.
		RoleAssigned { account: T::AccountId, role: RoleType },
		/// Removed validator from role.
		RoleRemoved { account: T::AccountId, role: RoleType },
		/// Funds bonded to become a validator.
		Bonded { account: T::AccountId, amount: BalanceOf<T> },
		/// Funds unbonded to stop being a validator.
		Unbonded { account: T::AccountId, amount: BalanceOf<T> },
		/// Slashed validator.
		Slashed { account: T::AccountId, amount: BalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Role has already been assigned to provided validator.
		RoleAlreadyAssigned,
		/// No role assigned to provided validator.
		RoleNotAssigned,
		/// Insufficient bond to become a validator.
		InsufficientBond,
		/// Stash controller account already added to Ledger
		AlreadyPaired,
		/// Stash controller account not found in ledger
		InvalidStashController,
	}

	/// Map from all "controller" accounts to the info regarding the staking.
	#[pallet::storage]
	#[pallet::getter(fn ledger)]
	pub type Ledger<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, RoleStakingLedger<T>>;

	#[pallet::storage]
	#[pallet::getter(fn account_role)]
	/// Mapping of resource to bridge index
	pub type AccountRolesMapping<T: Config> = StorageMap<_, Blake2_256, T::AccountId, RoleType>;

	/// The minimum active bond to become and maintain the role.
	#[pallet::storage]
	#[pallet::getter(fn min_active_bond)]
	pub(super) type MinActiveBond<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight({0})]
		#[pallet::call_index(0)]
		pub fn assign_role(
			origin: OriginFor<T>,
			#[pallet::compact] bond_value: BalanceOf<T>,
			role: RoleType,
		) -> DispatchResult {
			let stash_account = ensure_signed(origin)?;
			// check if role is already assigned.
			ensure!(
				!AccountRolesMapping::<T>::contains_key(&stash_account),
				Error::<T>::RoleAlreadyAssigned
			);
			// check if stash account is already paired.
			if <Ledger<T>>::contains_key(&stash_account) {
				return Err(Error::<T>::AlreadyPaired.into())
			}
			// check if min active bond is met.
			let min_active_bond = MinActiveBond::<T>::get();
			if bond_value < min_active_bond.into() {
				return Err(Error::<T>::InsufficientBond.into())
			}
			// Bond with stash account.
			let stash_balance = T::Currency::free_balance(&stash_account);
			let value = bond_value.min(stash_balance);

			// update ledger.
			let item = RoleStakingLedger { stash: stash_account.clone(), total_locked: value };
			Self::update_ledger(&stash_account, &item);

			Self::deposit_event(Event::<T>::Bonded {
				account: stash_account.clone(),
				amount: value,
			});

			// Add role mapping for the stash account.
			AccountRolesMapping::<T>::insert(&stash_account, role);
			Self::deposit_event(Event::<T>::RoleAssigned { account: stash_account.clone(), role });
			Ok(())
		}

		#[pallet::weight({0})]
		#[pallet::call_index(1)]
		pub fn clear_role(origin: OriginFor<T>, role: RoleType) -> DispatchResult {
			let stash_account = ensure_signed(origin)?;
			// check if role is assigned.
			ensure!(
				Self::validate_role(stash_account.clone(), role.clone()),
				Error::<T>::RoleNotAssigned
			);
			// TODO: Call jobs manager to remove the services.

			// On successful removal of services, remove the role from the mapping.
			// unbound locked funds.
			let ledger = Self::ledger(&stash_account).ok_or(Error::<T>::InvalidStashController)?;
			Self::unbond(&ledger)?;
			Self::deposit_event(Event::<T>::Unbonded {
				account: ledger.stash,
				amount: ledger.total_locked,
			});

			// Remove role from the mapping.
			AccountRolesMapping::<T>::remove(&stash_account);
			Self::deposit_event(Event::<T>::RoleRemoved { account: stash_account, role });

			Ok(())
		}
	}
}
