// This file is part of Tangle.
// Copyright (C) 2022-2023 Webb Technologies Inc.
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

//! Pallet to process claims from Ethereum addresses.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::all)]

use codec::MaxEncodedLen;
use frame_support::{
	ensure,
	traits::{Currency, Get},
	CloneNoBound, EqNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};

pub use pallet::*;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{codec, traits::Zero};
use sp_std::{convert::TryInto, prelude::*, vec};
use tangle_primitives::roles::{ReStakingOption, RoleType};
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
	/// The total amount of the stash's balance that is re-staked for all selected roles.
	/// This re-staked balance we are currently accounting for new slashing conditions.
	#[codec(compact)]
	pub total: BalanceOf<T>,
	/// The list of roles and their re-staked amounts.
	pub roles: Vec<RoleStakeInfo<T>>,
}

/// The information regarding the re-staked amount for a particular role.
#[derive(PartialEqNoBound, EqNoBound, Encode, Decode, RuntimeDebugNoBound, TypeInfo, Clone)]
#[scale_info(skip_type_params(T))]
pub struct RoleStakeInfo<T: Config> {
	/// Role type
	pub role: RoleType,
	/// The total amount of the stash's balance that is re-staked for selected role.
	#[codec(compact)]
	pub re_staked: BalanceOf<T>,
}

impl<T: Config> RoleStakingLedger<T> {
	/// Initializes the default object using the given `validator`.
	pub fn default_from(stash: T::AccountId) -> Self {
		Self { stash, total: Zero::zero(), roles: vec![] }
	}

	/// Returns `true` if the stash account has no funds at all.
	pub fn is_empty(&self) -> bool {
		self.total.is_zero()
	}
}

pub type CurrencyOf<T> = <T as pallet_staking::Config>::Currency;
pub type BalanceOf<T> =
	<CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
	pub trait Config: frame_system::Config + pallet_staking::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type MaxRolesPerAccount: Get<u32>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
		/// Role assigned to the validator.
		RoleAssigned { account: T::AccountId, role: RoleType },
		/// Removed validator from role.
		RoleRemoved { account: T::AccountId, role: RoleType },
		/// Slashed validator.
		Slashed { account: T::AccountId, amount: BalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Not a validator.
		NotValidator,
		/// Validator has active role assigned
		HasRoleAssigned,
		/// No role assigned to provided validator.
		NoRoleAssigned,
		/// Max role limit reached for the account.
		MaxRoles,
		/// Invalid Re-staking amount, should not exceed total staked amount.
		InvalidReStakingBond,
		/// Re staking amount should be greater than minimum re-staking bond requirement.
		InsufficientReStakingBond,
		/// Stash controller account already added to Roles Ledger
		AccountAlreadyPaired,
		/// Stash controller account not found in Roles Ledger.
		AccountNotPaired,
	}

	/// Map from all "controller" accounts to the info regarding the staking.
	#[pallet::storage]
	#[pallet::getter(fn ledger)]
	pub type Ledger<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, RoleStakingLedger<T>>;

	#[pallet::storage]
	#[pallet::getter(fn account_roles)]
	/// Mapping of resource to bridge index
	pub type AccountRolesMapping<T: Config> = StorageMap<
		_,
		Blake2_256,
		T::AccountId,
		BoundedVec<RoleType, T::MaxRolesPerAccount>,
		ValueQuery,
	>;

	/// The minimum re staking bond to become and maintain the role.
	#[pallet::storage]
	#[pallet::getter(fn min_active_bond)]
	pub(super) type MinReStakingBond<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// Assigns a role to the validator.
	///
	/// # Parameters
	///
	/// - `origin`: Origin of the transaction.
	/// - `role`: Role to assign to the validator.
	/// - `re_stake`: Amount of funds you want to re-stake.
	///
	/// This function will return error if
	/// - Account is not a validator account.
	/// - Role is already assigned to the validator.
	/// - Min re-staking bond is not met.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight({0})]
		#[pallet::call_index(0)]
		pub fn assign_role(
			origin: OriginFor<T>,
			role: RoleType,
			re_stake: ReStakingOption,
		) -> DispatchResult {
			let stash_account = ensure_signed(origin)?;
			// Ensure stash account is a validator.
			ensure!(
				pallet_staking::Validators::<T>::contains_key(&stash_account),
				Error::<T>::NotValidator
			);

			// Check if role is already assigned.
			ensure!(!Self::has_role(stash_account.clone(), role), Error::<T>::HasRoleAssigned);

			let staking_ledger =
				pallet_staking::Ledger::<T>::get(&stash_account).ok_or(Error::<T>::NotValidator)?;
			let re_stake_amount = match re_stake {
				ReStakingOption::Full => staking_ledger.active,
				ReStakingOption::Custom(x) => x.into(),
			};

			// Validate re-staking bond, should be greater than min re-staking bond requirement.
			let min_re_staking_bond = MinReStakingBond::<T>::get();
			ensure!(re_stake_amount >= min_re_staking_bond, Error::<T>::InsufficientReStakingBond);

			// Validate re-staking bond, should not exceed active staked bond.
			ensure!(staking_ledger.active >= re_stake_amount, Error::<T>::InvalidReStakingBond);

			// Check if account is already paired with ledger
			let maybe_ledger = Ledger::<T>::get(&stash_account);
			if maybe_ledger.is_none() {
				// Add stash account to ledger.
				let role_info = RoleStakeInfo { role, re_staked: re_stake_amount };
				let item = RoleStakingLedger {
					stash: stash_account.clone(),
					total: re_stake_amount,
					roles: vec![role_info],
				};
				Self::update_ledger(&stash_account, &item);
			} else {
				// Update ledger and add role info.
				let mut ledger = maybe_ledger.unwrap();
				ledger.total += re_stake_amount;
				let role_info = RoleStakeInfo { role, re_staked: re_stake_amount };
				ledger.roles.push(role_info);
				Self::update_ledger(&stash_account, &ledger);
			}

			// Add role mapping for the stash account.
			Self::add_role(stash_account.clone(), role)?;
			Self::deposit_event(Event::<T>::RoleAssigned { account: stash_account.clone(), role });
			Ok(())
		}

		/// Removes the role from the validator.
		///
		/// # Parameters
		///
		/// - `origin`: Origin of the transaction.
		/// - `role`: Role to remove from the validator.
		///
		/// This function will return error if
		/// - Account is not a validator account.
		/// - Role is not assigned to the validator.
		/// - All the jobs are not completed.
		#[pallet::weight({0})]
		#[pallet::call_index(1)]
		pub fn clear_role(origin: OriginFor<T>, role: RoleType) -> DispatchResult {
			let stash_account = ensure_signed(origin)?;
			// Ensure stash account is a validator.
			ensure!(
				pallet_staking::Validators::<T>::contains_key(&stash_account),
				Error::<T>::NotValidator
			);

			// check if role is assigned.
			ensure!(Self::has_role(stash_account.clone(), role), Error::<T>::HasRoleAssigned);

			// TODO: Call jobs manager to remove the services.
			// On successful removal of services, remove the role from the mapping.
			// Issue link for reference : https://github.com/webb-tools/tangle/issues/292

			// Remove role from the mapping.
			Self::remove_role(stash_account.clone(), role)?;
			// Remove stash account related info.
			Self::kill_stash(&stash_account);

			Self::deposit_event(Event::<T>::RoleRemoved { account: stash_account, role });

			Ok(())
		}

		/// Declare no desire to either validate or nominate.
		///
		/// If you have opted for any of the roles, please submit `clear_role` extrinsic to opt out
		/// of all the services. Once your role is cleared, your request will be processed.
		///
		/// # Parameters
		///
		/// - `origin`: Origin of the transaction.
		///
		/// This function will return error if
		/// - Account is not a validator account.
		/// - Role is assigned to the validator.
		#[pallet::weight({0})]
		#[pallet::call_index(2)]
		pub fn chill(origin: OriginFor<T>) -> DispatchResult {
			let account = ensure_signed(origin.clone())?;
			// Ensure no role is assigned to the account before chilling.
			ensure!(Self::can_exit(account), Error::<T>::HasRoleAssigned);

			// chill
			pallet_staking::Pallet::<T>::chill(origin)
		}

		/// Unbound funds from the stash account.
		/// This will allow user to unbound and later withdraw funds.
		/// If you have opted for any of the roles, please submit `clear_role` extrinsic to opt out
		/// of all the services. Once your role is cleared, you can unbound
		/// and withdraw funds.
		///
		/// # Parameters
		///
		/// - `origin`: Origin of the transaction.
		/// - `amount`: Amount of funds to unbound.
		///
		/// This function will return error if
		/// - If there is any active role assigned to the user.
		///  
		#[pallet::weight({0})]
		#[pallet::call_index(3)]
		pub fn unbound_funds(
			origin: OriginFor<T>,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResult {
			let account = ensure_signed(origin.clone())?;
			// Ensure no role is assigned to the account and is eligible to unbound.
			ensure!(Self::can_exit(account), Error::<T>::HasRoleAssigned);

			// Unbound funds.
			let res = pallet_staking::Pallet::<T>::unbond(origin, amount);
			match res {
				Ok(_) => Ok(()),
				Err(dispatch_post_info) => Err(dispatch_post_info.error),
			}
		}

		/// Withdraw unbound funds after un-bonding period has passed.
		///
		/// # Parameters
		///
		/// - `origin`: Origin of the transaction.
		///
		/// This function will return error if
		/// - If there is any active role assigned to the user.
		#[pallet::weight({0})]
		#[pallet::call_index(4)]
		pub fn withdraw_unbonded(origin: OriginFor<T>) -> DispatchResult {
			let account = ensure_signed(origin.clone())?;
			// Ensure no role is assigned to the account and is eligible to withdraw.
			ensure!(Self::can_exit(account), Error::<T>::HasRoleAssigned);

			// Withdraw unbound funds.
			let res = pallet_staking::Pallet::<T>::withdraw_unbonded(origin, 0);
			match res {
				Ok(_) => Ok(()),
				Err(dispatch_post_info) => Err(dispatch_post_info.error),
			}
		}
	}
}
