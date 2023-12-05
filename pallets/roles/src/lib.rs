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
	traits::{Currency, Get, ValidatorSet, ValidatorSetWithIdentification},
	CloneNoBound, EqNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
use tangle_primitives::roles::ValidatorRewardDistribution;

pub use pallet::*;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{codec, traits::Zero, Saturating};
use sp_staking::offence::ReportOffence;
use sp_std::{convert::TryInto, prelude::*, vec};
use tangle_primitives::{
	roles::{RoleType, RoleTypeMetadata},
	traits::jobs::JobsHandler,
};
mod impls;
use sp_std::collections::btree_map::BTreeMap;
#[cfg(test)]
pub(crate) mod mock;
pub mod offences;
#[cfg(test)]
mod tests;
mod weights;
pub use weights::WeightInfo;

use sp_runtime::RuntimeAppPublic;

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
	pub roles: BTreeMap<RoleType, RoleStakingRecord<T>>,
}

/// The information regarding the re-staked amount for a particular role.
#[derive(PartialEqNoBound, EqNoBound, Encode, Decode, RuntimeDebugNoBound, TypeInfo, Clone)]
#[scale_info(skip_type_params(T))]
pub struct RoleStakingRecord<T: Config> {
	/// Metadata associated with the role.
	pub metadata: RoleTypeMetadata,
	/// The total amount of the stash's balance that is re-staked for selected role.
	#[codec(compact)]
	pub re_staked: BalanceOf<T>,
}

impl<T: Config> RoleStakingLedger<T> {
	/// Initializes the default object using the given `validator`.
	pub fn default_from(stash: T::AccountId) -> Self {
		Self { stash, total: Zero::zero(), roles: Default::default() }
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
	use crate::offences::ValidatorOffence;

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use tangle_primitives::{
		jobs::{JobId, JobKey},
		traits::jobs::MPCHandler,
	};

	/// A type for representing the validator id in a session.
	pub type ValidatorId<T> = <<T as Config>::ValidatorSet as ValidatorSet<
		<T as frame_system::Config>::AccountId,
	>>::ValidatorId;

	/// A tuple of (ValidatorId, Identification) where `Identification` is the full identification
	/// of `ValidatorId`.
	pub type IdentificationTuple<T> = (
		ValidatorId<T>,
		<<T as Config>::ValidatorSet as ValidatorSetWithIdentification<
			<T as frame_system::Config>::AccountId,
		>>::Identification,
	);

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_staking::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The job manager mechanism.
		type JobsHandler: JobsHandler<Self::AccountId>;

		/// Max roles per account.
		#[pallet::constant]
		type MaxRolesPerAccount: Get<u32>;

		/// The config that verifies MPC related functions
		type MPCHandler: MPCHandler<Self::AccountId, BlockNumberFor<Self>, BalanceOf<Self>>;

		/// The inflation reward to distribute per era
		type InflationRewardPerSession: Get<BalanceOf<Self>>;

		/// The inflation distribution based on validator type
		type ValidatorRewardDistribution: Get<ValidatorRewardDistribution>;

		/// The type used to identify an authority
		type AuthorityId: RuntimeAppPublic + Decode;

		/// A type for retrieving the validators supposed to be online in a session.
		type ValidatorSet: ValidatorSetWithIdentification<Self::AccountId>;

		/// A type to submit offence reports against the validators.
		type ReportOffences: ReportOffence<
			Self::AccountId,
			IdentificationTuple<Self>,
			ValidatorOffence<IdentificationTuple<Self>>,
		>;

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
		/// Pending jobs,that cannot be opted out at the moment.
		PendingJobs { pending_jobs: Vec<(JobKey, JobId)> },
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
		ExceedsMaxReStakeValue,
		/// Re staking amount should be greater than minimum re-staking bond requirement.
		InsufficientReStakingBond,
		/// Stash controller account already added to Roles Ledger
		AccountAlreadyPaired,
		/// Stash controller account not found in Roles Ledger.
		AccountNotPaired,
		/// Role clear request failed due to pending jobs, which can't be opted out at the moment.
		RoleClearRequestFailed,
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
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<RoleType, T::MaxRolesPerAccount>,
		ValueQuery,
	>;

	/// The minimum re staking bond to become and maintain the role.
	#[pallet::storage]
	#[pallet::getter(fn min_active_bond)]
	pub(super) type MinReStakingBond<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// Assigns roles to the validator.
	///
	/// # Parameters
	///
	/// - `origin`: Origin of the transaction.
	/// - `records`: List of roles user is interested to re-stake.
	///
	/// This function will return error if
	/// - Account is not a validator account.
	/// - Role is already assigned to the validator.
	/// - Min re-staking bond is not met.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight({0})]
		#[pallet::call_index(0)]
		pub fn assign_roles(
			origin: OriginFor<T>,
			records: Vec<RoleStakingRecord<T>>,
		) -> DispatchResult {
			let stash_account = ensure_signed(origin)?;
			// Ensure stash account is a validator.
			ensure!(
				pallet_staking::Validators::<T>::contains_key(&stash_account),
				Error::<T>::NotValidator
			);

			let mut ledger = Ledger::<T>::get(&stash_account)
				.unwrap_or(RoleStakingLedger::<T>::default_from(stash_account.clone()));

			let staking_ledger =
				pallet_staking::Ledger::<T>::get(&stash_account).ok_or(Error::<T>::NotValidator)?;

			let max_re_staking_bond = Self::calculate_max_re_stake_amount(staking_ledger.active);

			// Validate role staking records.
			for record in records.clone() {
				let role = record.metadata.get_role_type();
				let re_stake_amount = record.re_staked;
				// Check if role is already assigned.
				ensure!(
					!Self::has_role(stash_account.clone(), role.clone()),
					Error::<T>::HasRoleAssigned
				);

				// validate the metadata
				T::MPCHandler::validate_authority_key(
					stash_account.clone(),
					record.metadata.get_authority_key(),
				)?;

				// Re-staking amount of record should meet min re-staking amount requirement.
				let min_re_staking_bond = MinReStakingBond::<T>::get();
				ensure!(
					re_stake_amount >= min_re_staking_bond,
					Error::<T>::InsufficientReStakingBond
				);

				// Total re_staking amount should not exceed  max_re_staking_amount.
				ensure!(
					ledger.total.saturating_add(re_stake_amount) <= max_re_staking_bond,
					Error::<T>::ExceedsMaxReStakeValue
				);

				ledger.total = ledger.total.saturating_add(re_stake_amount);
				ledger.roles.insert(record.metadata.get_role_type(), record);
			}

			// Now that records are validated we can add them and update ledger
			for record in records {
				let role = record.metadata.get_role_type();
				Self::add_role(stash_account.clone(), role.clone())?;
				Self::deposit_event(Event::<T>::RoleAssigned {
					account: stash_account.clone(),
					role,
				});
			}
			Self::update_ledger(&stash_account, &ledger);

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
			ensure!(
				Self::has_role(stash_account.clone(), role.clone()),
				Error::<T>::NoRoleAssigned
			);

			// Get active jobs for the role.
			let active_jobs = T::JobsHandler::get_active_jobs(stash_account.clone());
			let mut role_cleared = true;
			let mut pending_jobs = Vec::new();
			for job in active_jobs {
				let job_key = job.0;
				if job_key.get_role_type() == role {
					// Submit request to exit from the known set.
					let res = T::JobsHandler::exit_from_known_set(
						stash_account.clone(),
						job_key.clone(),
						job.1,
					);

					if res.is_err() {
						role_cleared = false;
						pending_jobs.push((job_key.clone(), job.1));
					}
				}
			}

			if !role_cleared {
				// Role clear request failed due to pending jobs, which can't be opted out at the
				// moment.
				Self::deposit_event(Event::<T>::PendingJobs { pending_jobs });
				return Err(Error::<T>::RoleClearRequestFailed.into())
			};

			// Remove role from the mapping.
			Self::remove_role(stash_account.clone(), role.clone())?;
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
