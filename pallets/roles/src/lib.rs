// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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

use frame_support::{
	ensure,
	traits::{Currency, Get, ValidatorSet, ValidatorSetWithIdentification},
	CloneNoBound, EqNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
use tangle_primitives::roles::ValidatorRewardDistribution;

pub use pallet::*;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::ecdsa;
use sp_runtime::{
	traits::{Convert, OpaqueKeys, Zero},
	Saturating,
};
use sp_staking::offence::ReportOffence;
use sp_std::{collections::btree_map::BTreeMap, convert::TryInto, prelude::*, vec};
use tangle_crypto_primitives::ROLE_KEY_TYPE;
use tangle_primitives::roles::{RoleType, RoleTypeMetadata};

mod impls;
mod profile;
use profile::{Profile, Record};

#[cfg(test)]
pub(crate) mod mock;
pub mod offences;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;
pub use weights::WeightInfo;

use sp_runtime::RuntimeAppPublic;

/// The ledger of a (bonded) stash.
#[derive(
	PartialEqNoBound, EqNoBound, CloneNoBound, Encode, Decode, RuntimeDebugNoBound, TypeInfo,
)]
#[scale_info(skip_type_params(T))]
pub struct RoleStakingLedger<T: Config> {
	/// The stash account whose balance is actually locked and at stake.
	pub stash: T::AccountId,
	/// The total amount of the stash's balance that is restaked for all selected roles.
	/// This restaked balance we are currently accounting for new slashing conditions.
	#[codec(compact)]
	pub total: BalanceOf<T>,
	/// Restaking Profile
	pub profile: Profile<T>,
	/// Roles map with their respective records.
	pub roles: BTreeMap<RoleType, Record<T>>,
}

impl<T: Config> RoleStakingLedger<T> {
	/// New staking ledger for a stash account.
	pub fn new(stash: T::AccountId, profile: Profile<T>) -> Self {
		let total_restake = profile.get_total_profile_restake();
		let roles = profile
			.get_records()
			.into_iter()
			.map(|record| (record.metadata.get_role_type(), record))
			.collect::<BTreeMap<_, _>>();
		Self { stash, total: total_restake.into(), profile, roles }
	}

	/// Returns the total amount of the stash's balance that is restaked for all selected roles.
	pub fn total_restake(&self) -> BalanceOf<T> {
		self.total
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
	use tangle_primitives::jobs::{
		traits::{JobsHandler, MPCHandler},
		JobId,
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
	pub trait Config:
		frame_system::Config + pallet_staking::Config + pallet_session::Config
	{
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Authority identifier type
		type RoleKeyId: Member
			+ Parameter
			+ RuntimeAppPublic
			+ MaybeSerializeDeserialize
			+ AsRef<[u8]>
			+ Into<ecdsa::Public>
			+ From<ecdsa::Public>
			+ MaxEncodedLen;

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
		/// New profile created.
		ProfileCreated {
			account: T::AccountId,
			total_profile_restake: BalanceOf<T>,
			roles: Vec<RoleType>,
		},
		/// Profile updated.
		ProfileUpdated {
			account: T::AccountId,
			total_profile_restake: BalanceOf<T>,
			roles: Vec<RoleType>,
		},
		/// Profile deleted.
		ProfileDeleted { account: T::AccountId },
		/// Pending jobs,that cannot be opted out at the moment.
		PendingJobs { pending_jobs: Vec<(RoleType, JobId)> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Not a validator.
		NotValidator,
		/// Validator has active role assigned.
		HasRoleAssigned,
		/// Given role is not assigned to the validator.
		RoleNotAssigned,
		/// Max role limit reached for the account.
		MaxRoles,
		/// Role cannot due to pending jobs, which can't be opted out at the moment.
		RoleCannotBeRemoved,
		/// Restaking amount cannot be lowered if there are any pending jobs. You can only add more
		RestakingAmountCannotBeUpdated,
		/// Invalid Restaking amount, should not exceed total staked amount.
		ExceedsMaxRestakeValue,
		/// Re staking amount should be greater than minimum Restaking bond requirement.
		InsufficientRestakingBond,
		/// Profile Update failed.
		ProfileUpdateFailed,
		/// Profile already exists for given validator account.
		ProfileAlreadyExists,
		/// Stash controller account not found in Roles Ledger.
		NoProfileFound,
		/// Profile delete request failed due to pending jobs, which can't be opted out at the
		/// moment.
		ProfileDeleteRequestFailed,
		/// SessionKeys not provided
		SessionKeysNotProvided,
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
	pub(super) type MinRestakingBond<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// Create profile for the validator.
	/// Validator can choose roles he is interested to opt-in and restake tokens for it.
	/// Staking can be done shared or independently for each role.
	///
	/// # Parameters
	///
	/// - `origin`: Origin of the transaction.
	/// - `profile`: Profile to be created
	///
	/// This function will return error if
	/// - Account is not a validator account.
	/// - Profile already exists for the validator.
	/// - Min Restaking bond is not met.
	/// - Restaking amount is exceeds max Restaking value.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create_profile())]
		#[pallet::call_index(0)]
		pub fn create_profile(origin: OriginFor<T>, profile: Profile<T>) -> DispatchResult {
			let stash_account = ensure_signed(origin)?;

			let validator_id =
				<T as pallet_session::Config>::ValidatorIdOf::convert(stash_account.clone())
					.ok_or(Error::<T>::NotValidator)?;

			let session_keys = pallet_session::NextKeys::<T>::get(validator_id)
				.ok_or(Error::<T>::SessionKeysNotProvided)?;
			let _role_key = OpaqueKeys::get_raw(&session_keys, ROLE_KEY_TYPE);

			// Ensure stash account is a validator.
			ensure!(
				pallet_staking::Validators::<T>::contains_key(&stash_account),
				Error::<T>::NotValidator
			);

			// Ensure no profile is assigned to the validator.
			ensure!(!Ledger::<T>::contains_key(&stash_account), Error::<T>::ProfileAlreadyExists);
			let ledger = RoleStakingLedger::<T>::new(stash_account.clone(), profile.clone());
			let total_profile_restake = profile.get_total_profile_restake();

			// Restaking amount of profile should meet min Restaking amount requirement.
			let min_restaking_bond = MinRestakingBond::<T>::get();
			ensure!(
				total_profile_restake >= min_restaking_bond,
				Error::<T>::InsufficientRestakingBond
			);

			// Total restaking amount should not exceed  max_restaking_amount.
			let staking_ledger =
				pallet_staking::Ledger::<T>::get(&stash_account).ok_or(Error::<T>::NotValidator)?;
			let max_restaking_bond = Self::calculate_max_restake_amount(staking_ledger.active);
			ensure!(
				total_profile_restake <= max_restaking_bond,
				Error::<T>::ExceedsMaxRestakeValue
			);

			// Validate role staking records.
			let records = profile.get_records();
			for record in records {
				if profile.is_independent() {
					// Restaking amount of record should meet min Restaking amount requirement.
					let record_restake = record.amount.unwrap_or_default();
					ensure!(
						record_restake >= min_restaking_bond,
						Error::<T>::InsufficientRestakingBond
					);
				}
				// validate the metadata
				T::MPCHandler::validate_authority_key(
					stash_account.clone(),
					record.metadata.get_authority_key(),
				)?;
			}

			let profile_roles: BoundedVec<RoleType, T::MaxRolesPerAccount> =
				BoundedVec::try_from(profile.get_roles()).map_err(|_| Error::<T>::MaxRoles)?;

			AccountRolesMapping::<T>::insert(&stash_account, profile_roles);

			Self::update_ledger(&stash_account, &ledger);
			Self::deposit_event(Event::<T>::ProfileCreated {
				account: stash_account.clone(),
				total_profile_restake,
				roles: profile.get_roles(),
			});

			Ok(())
		}

		/// Update profile of the validator.
		/// This function will update the profile of the validator.
		/// If user wants to remove any role, please ensure that all the jobs associated with the
		/// role are completed else this tx will fail.
		/// If user wants to add any role, please ensure that the Restaking amount is greater than
		/// required min Restaking bond.
		///
		/// # Parameters
		/// - `origin`: Origin of the transaction.
		/// - `profile`: Updated profile.
		///
		/// This function will return error if
		/// - Account is not a validator account.
		/// - Profile is not assigned to the validator.
		/// - If there are any pending jobs for the role which user wants to remove.
		/// - Restaking amount is exceeds max Restaking value.
		/// - Restaking amount is less than min Restaking bond.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::update_profile())]
		#[pallet::call_index(1)]
		pub fn update_profile(origin: OriginFor<T>, updated_profile: Profile<T>) -> DispatchResult {
			let stash_account = ensure_signed(origin)?;
			// Ensure stash account is a validator.
			ensure!(
				pallet_staking::Validators::<T>::contains_key(&stash_account),
				Error::<T>::NotValidator
			);
			let mut ledger = Ledger::<T>::get(&stash_account).ok_or(Error::<T>::NoProfileFound)?;

			let total_profile_restake = updated_profile.get_total_profile_restake();
			// Restaking amount of record should meet min Restaking amount requirement.
			let min_restaking_bond = MinRestakingBond::<T>::get();
			ensure!(
				total_profile_restake >= min_restaking_bond,
				Error::<T>::InsufficientRestakingBond
			);

			let staking_ledger =
				pallet_staking::Ledger::<T>::get(&stash_account).ok_or(Error::<T>::NotValidator)?;

			let max_restaking_bond = Self::calculate_max_restake_amount(staking_ledger.active);
			// Total restaking amount should not exceed  max_restaking_amount.
			ensure!(
				total_profile_restake <= max_restaking_bond,
				Error::<T>::ExceedsMaxRestakeValue
			);

			Self::validate_updated_profile(stash_account.clone(), updated_profile.clone())?;
			ledger.profile = updated_profile.clone();
			ledger.total = total_profile_restake;

			let profile_roles: BoundedVec<RoleType, T::MaxRolesPerAccount> =
				BoundedVec::try_from(updated_profile.get_roles())
					.map_err(|_| Error::<T>::MaxRoles)?;

			AccountRolesMapping::<T>::insert(&stash_account, profile_roles);
			Self::update_ledger(&stash_account, &ledger);

			Self::deposit_event(Event::<T>::ProfileUpdated {
				account: stash_account.clone(),
				total_profile_restake,
				roles: updated_profile.get_roles(),
			});

			Ok(())
		}

		/// Delete profile of the validator.
		/// This function will submit the request to exit from all the services and will fails if
		/// all the job are not completed.
		///
		///
		/// # Parameters
		/// - `origin`: Origin of the transaction.
		///
		/// This function will return error if
		/// - Account is not a validator account.
		/// - Profile is not assigned to the validator.
		/// - All the jobs are not completed.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::delete_profile())]
		#[pallet::call_index(2)]
		pub fn delete_profile(origin: OriginFor<T>) -> DispatchResult {
			let stash_account = ensure_signed(origin)?;
			// Ensure stash account is a validator.
			ensure!(
				pallet_staking::Validators::<T>::contains_key(&stash_account),
				Error::<T>::NotValidator
			);
			let mut ledger = Ledger::<T>::get(&stash_account).ok_or(Error::<T>::NoProfileFound)?;

			// Submit request to exit from all the services.
			let active_jobs = T::JobsHandler::get_active_jobs(stash_account.clone());
			let mut pending_jobs = Vec::new();
			for job in active_jobs {
				let role_type = job.0;
				// Submit request to exit from the known set.
				let res =
					T::JobsHandler::exit_from_known_set(stash_account.clone(), role_type, job.1);

				if res.is_err() {
					pending_jobs.push((role_type, job.1));
				} else {
					// Remove role from the profile.
					ledger.profile.remove_role_from_profile(role_type);
				}
			}

			if !pending_jobs.is_empty() {
				// Update account roles mapping.
				let profile_roles: BoundedVec<RoleType, T::MaxRolesPerAccount> =
					BoundedVec::try_from(ledger.profile.get_roles())
						.map_err(|_| Error::<T>::MaxRoles)?;

				AccountRolesMapping::<T>::insert(&stash_account, profile_roles);

				// Profile delete request failed due to pending jobs, which can't be opted out at
				// the moment.
				Self::deposit_event(Event::<T>::PendingJobs { pending_jobs });
				return Err(Error::<T>::ProfileDeleteRequestFailed.into())
			};

			Self::deposit_event(Event::<T>::ProfileDeleted { account: stash_account.clone() });

			// Remove entry from ledger.
			Ledger::<T>::remove(&stash_account);
			// Remove entry from account roles mapping.
			AccountRolesMapping::<T>::remove(&stash_account);

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
		#[pallet::weight(<T as pallet::Config>::WeightInfo::chill())]
		#[pallet::call_index(3)]
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
		#[pallet::weight(<T as pallet::Config>::WeightInfo::unbound_funds())]
		#[pallet::call_index(4)]
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
		#[pallet::weight(<T as pallet::Config>::WeightInfo::withdraw_unbonded())]
		#[pallet::call_index(5)]
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
