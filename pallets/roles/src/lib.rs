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
	BoundedBTreeMap, BoundedVec, CloneNoBound, EqNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
pub use pallet::*;
use pallet_staking::{ActiveEraInfo, EraRewardPoints};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::ecdsa;
use sp_runtime::{
	traits::{Convert, OpaqueKeys, Zero},
	DispatchError, Percent, Saturating,
};
use sp_staking::{offence::ReportOffence, EraIndex};
use sp_std::{convert::TryInto, fmt::Debug, prelude::*, vec};
use tangle_crypto_primitives::ROLE_KEY_TYPE;
use tangle_primitives::roles::{RoleType, ValidatorRewardDistribution};

mod functions;
mod impls;
mod types;

pub mod profile;
use profile::{Profile, Record};
pub use types::*;

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

#[frame_support::pallet]
pub mod pallet {
	use crate::offences::ValidatorOffence;

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use tangle_primitives::jobs::{traits::JobsHandler, JobId};

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

		/// The inflation distribution based on validator type
		type ValidatorRewardDistribution: Get<ValidatorRewardDistribution>;

		/// A type for retrieving the validators supposed to be online in a session.
		type ValidatorSet: ValidatorSetWithIdentification<Self::AccountId>;

		/// The max length for validator key
		type MaxKeyLen: Get<u32>;

		/// The max roles a validator is allowed to have
		type MaxRolesPerValidator: Get<u32>;

		/// The max validators allowed in the pallet
		type MaxValidators: Get<u32>;

		/// A type to submit offence reports against the validators.
		type ReportOffences: ReportOffence<
			Self::AccountId,
			IdentificationTuple<Self>,
			ValidatorOffence<IdentificationTuple<Self>>,
		>;

		/// The maximum active jobs per validator
		type MaxActiveJobsPerValidator: Get<u32> + Clone + TypeInfo + Debug + Eq + PartialEq;

		/// The max permitted restake for a restaker
		type MaxRestake: Get<Percent>;

		/// The origin for privileged calls
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The restaker payout per era
		type RestakerEraPayout: pallet_staking::EraPayout<BalanceOf<Self>>;

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
		/// Roles inflation reward paid for era
		RolesRewardSet { total_rewards: BalanceOf<T> },
		/// The re-stakers' rewards are getting paid.
		PayoutStarted { era_index: EraIndex, validator_stash: T::AccountId },
		/// The re-staker has been rewarded by this amount.
		Rewarded { stash: T::AccountId, amount: BalanceOf<T> },
		/// The min restaking bond amount has been updated
		MinRestakingBondUpdated { value: BalanceOf<T> },
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
		/// Key size exceeded
		KeySizeExceeded,
		/// Cannot find Current era
		CannotGetCurrentEra,
		/// Invalid era info
		InvalidEraToReward,
		/// Out of bounds input
		BoundNotMet,
		/// Rewards already claimed
		AlreadyClaimed,
		/// Unlock chunks already filled
		NoMoreChunks,
	}

	/// Map from all "controller" accounts to the info regarding the staking.
	#[pallet::storage]
	#[pallet::getter(fn ledger)]
	pub type Ledger<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, RestakingLedger<T>>;

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

	#[pallet::type_value]
	pub fn DefaultMinRestakingBond<T: Config>() -> BalanceOf<T> {
		1000u32.into()
	}

	/// The minimum re staking bond to become and maintain the role.
	#[pallet::storage]
	#[pallet::getter(fn min_active_bond)]
	pub(super) type MinRestakingBond<T: Config> =
		StorageValue<_, BalanceOf<T>, ValueQuery, DefaultMinRestakingBond<T>>;

	/// The number of jobs completed by a validator in era
	#[pallet::storage]
	#[pallet::getter(fn validator_points_per_session)]
	pub type ValidatorJobsInEra<T: Config> =
		StorageValue<_, BoundedBTreeMap<T::AccountId, u32, T::MaxValidators>, ValueQuery>;

	/// Rewards for the last `HISTORY_DEPTH` eras.
	/// If reward hasn't been set or has been removed then 0 reward is returned.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn eras_reward_points)]
	pub type ErasRestakeRewardPoints<T: Config> =
		StorageMap<_, Twox64Concat, EraIndex, EraRewardPoints<T::AccountId>, ValueQuery>;

	/// The active era information, it holds index and start.
	///
	/// The active era is the era being currently rewarded.
	#[pallet::storage]
	#[pallet::getter(fn active_restaker_era)]
	pub type ActiveRestakerEra<T> = StorageValue<_, ActiveEraInfo>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
			// just return the weight of the on_finalize.
			T::DbWeight::get().reads_writes(1, 1)
		}

		fn on_finalize(_n: BlockNumberFor<T>) {
			// if we already have restaker era information, check if the
			// staking pallet has advanced in active era
			if let Some(restaker_era) = Self::active_restaker_era() {
				if let Some(staking_era) = pallet_staking::Pallet::<T>::active_era() {
					// if we staking pallet has advanced a session, then we
					// compute rewards with last session restakers
					// and set new era
					if staking_era.index > restaker_era.index {
						let _ = Self::compute_rewards(staking_era.clone(), restaker_era.index);
						ActiveRestakerEra::<T>::put(staking_era);
					}
				}
			}
			// Set the start of the first era.
			else {
				if let Some(staking_era) = pallet_staking::Pallet::<T>::active_era() {
					ActiveRestakerEra::<T>::put(staking_era);
				}
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
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
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create_profile())]
		#[pallet::call_index(0)]
		pub fn create_profile(
			origin: OriginFor<T>,
			profile: Profile<T>,
			max_active_services: Option<u32>,
		) -> DispatchResult {
			let restaker_account = ensure_signed(origin)?;

			// Ensure stash account is a validator.
			ensure!(
				pallet_staking::Validators::<T>::contains_key(&restaker_account),
				Error::<T>::NotValidator
			);

			// Get Role key of validator.
			let validator_id =
				<T as pallet_session::Config>::ValidatorIdOf::convert(restaker_account.clone())
					.ok_or(Error::<T>::NotValidator)?;

			let session_keys = pallet_session::NextKeys::<T>::get(validator_id)
				.ok_or(Error::<T>::SessionKeysNotProvided)?;
			let role_key = OpaqueKeys::get_raw(&session_keys, ROLE_KEY_TYPE);

			// Ensure no profile is assigned to the validator.
			ensure!(
				!Ledger::<T>::contains_key(&restaker_account),
				Error::<T>::ProfileAlreadyExists
			);
			let ledger = RestakingLedger::<T>::try_new(
				restaker_account.clone(),
				profile.clone(),
				role_key.to_vec(),
				max_active_services,
			)?;
			let total_profile_restake = profile.get_total_profile_restake();

			// Restaking amount of profile should meet min Restaking amount requirement.
			let min_restaking_bond = MinRestakingBond::<T>::get();
			ensure!(
				total_profile_restake >= min_restaking_bond,
				Error::<T>::InsufficientRestakingBond
			);

			// Total restaking amount should not exceed  max_restaking_amount.
			let staking_ledger = pallet_staking::Ledger::<T>::get(&restaker_account)
				.ok_or(Error::<T>::NotValidator)?;
			let max_restaking_bond = Self::calculate_max_restake_amount(staking_ledger.active);
			ensure!(
				total_profile_restake <= max_restaking_bond,
				Error::<T>::ExceedsMaxRestakeValue
			);

			// Validate role re-staking records.
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
			}

			let profile_roles: BoundedVec<RoleType, T::MaxRolesPerAccount> =
				BoundedVec::try_from(profile.get_roles()).map_err(|_| Error::<T>::MaxRoles)?;

			AccountRolesMapping::<T>::insert(&restaker_account, profile_roles);

			Self::update_ledger(&restaker_account, &ledger);
			Self::deposit_event(Event::<T>::ProfileCreated {
				account: restaker_account.clone(),
				total_profile_restake,
				roles: profile.get_roles(),
			});

			Ok(())
		}

		/// Update profile of the restaker.
		/// This function will update the profile of the restaker.
		/// If user wants to remove any role, ensure that all the jobs associated with the
		/// role are completed else this tx will fail.
		/// If user wants to add any role, ensure that the Restaking amount is greater than
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
			let restaker_account = ensure_signed(origin)?;
			// Ensure stash account is a validator.
			ensure!(
				pallet_staking::Validators::<T>::contains_key(&restaker_account),
				Error::<T>::NotValidator
			);
			let mut ledger =
				Ledger::<T>::get(&restaker_account).ok_or(Error::<T>::NoProfileFound)?;

			let profile_before_update = ledger.profile;

			// Restaking amount of record should meet min restaking amount requirement.
			match updated_profile.clone() {
				Profile::Shared(profile) => {
					ensure!(
						profile.amount >= MinRestakingBond::<T>::get(),
						Error::<T>::InsufficientRestakingBond
					);
				},
				Profile::Independent(profile) => {
					for record in profile.records.iter() {
						let record_restake = record.amount.unwrap_or_default();
						ensure!(
							record_restake >= MinRestakingBond::<T>::get(),
							Error::<T>::InsufficientRestakingBond
						);
					}
				},
			};

			// Total restaking amount should not exceed `max_restaking_amount`.
			let staking_ledger = pallet_staking::Ledger::<T>::get(&restaker_account)
				.ok_or(Error::<T>::NotValidator)?;
			let max_restaking_bond = Self::calculate_max_restake_amount(staking_ledger.active);
			ensure!(
				updated_profile.get_total_profile_restake() <= max_restaking_bond,
				Error::<T>::ExceedsMaxRestakeValue
			);
			// Validate additional rules for profile update.
			Self::validate_updated_profile(restaker_account.clone(), updated_profile.clone())?;
			ledger.profile = updated_profile.clone();
			ledger.total = updated_profile.get_total_profile_restake().into();

			// if the total restake was reduced, we record that in unlock data
			if profile_before_update.get_total_profile_restake()
				> updated_profile.get_total_profile_restake()
			{
				let value = profile_before_update
					.get_total_profile_restake()
					.saturating_sub(updated_profile.get_total_profile_restake());
				let era = Self::active_restaker_era().ok_or(Error::<T>::InvalidEraToReward)?.index
					+ T::BondingDuration::get();

				ledger
					.unlocking
					.try_push(UnlockChunk { value, era })
					.map_err(|_| Error::<T>::NoMoreChunks)?;
			}

			let profile_roles: BoundedVec<RoleType, T::MaxRolesPerAccount> =
				BoundedVec::try_from(updated_profile.get_roles())
					.map_err(|_| Error::<T>::MaxRoles)?;

			AccountRolesMapping::<T>::insert(&restaker_account, profile_roles);
			Self::update_ledger(&restaker_account, &ledger);

			Self::deposit_event(Event::<T>::ProfileUpdated {
				account: restaker_account.clone(),
				total_profile_restake: updated_profile.get_total_profile_restake().into(),
				roles: updated_profile.get_roles(),
			});

			Ok(())
		}

		/// Delete profile of the restaker.
		/// This function will submit the request to exit from all the services and will fails if
		/// all the job are not completed.
		///
		///
		/// # Parameters
		/// - `origin`: Origin of the transaction.
		///
		/// This function will return error if
		/// - Account is not a validator account.
		/// - Profile is not assigned to the restaker.
		/// - All the jobs are not completed.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::delete_profile())]
		#[pallet::call_index(2)]
		pub fn delete_profile(origin: OriginFor<T>) -> DispatchResult {
			let restaker_account = ensure_signed(origin)?;
			// Ensure stash account is a validator.
			ensure!(
				pallet_staking::Validators::<T>::contains_key(&restaker_account),
				Error::<T>::NotValidator
			);
			let ledger = Ledger::<T>::get(&restaker_account).ok_or(Error::<T>::NoProfileFound)?;

			// Submit request to exit from all the services.
			let active_jobs = T::JobsHandler::get_active_jobs(restaker_account.clone());
			let mut pending_jobs = Vec::new();
			for job in active_jobs {
				let role_type = job.0;
				pending_jobs.push((role_type, job.1));
			}

			if !pending_jobs.is_empty() {
				// Update account roles mapping.
				let profile_roles: BoundedVec<RoleType, T::MaxRolesPerAccount> =
					BoundedVec::try_from(ledger.profile.get_roles())
						.map_err(|_| Error::<T>::MaxRoles)?;

				AccountRolesMapping::<T>::insert(&restaker_account, profile_roles);

				// Profile delete request failed due to pending jobs, which can't be opted out at
				// the moment.
				return Err(Error::<T>::ProfileDeleteRequestFailed.into());
			}

			Self::deposit_event(Event::<T>::ProfileDeleted { account: restaker_account.clone() });

			// Remove entry from ledger.
			Ledger::<T>::remove(&restaker_account);
			// Remove entry from account roles mapping.
			AccountRolesMapping::<T>::remove(&restaker_account);

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
			let restaker_account = ensure_signed(origin.clone())?;
			// Ensure no role is assigned to the restaker_account before chilling.
			ensure!(Self::can_exit(restaker_account), Error::<T>::HasRoleAssigned);

			// chill
			pallet_staking::Pallet::<T>::chill(origin)
		}

		/// Unbond funds from the stash account.
		/// This will allow user to unbond and later withdraw funds.
		/// If you have opted for any of the roles, please submit `clear_role` extrinsic to opt out
		/// of all the services. Once your role is cleared, you can unbond
		/// and withdraw funds.
		///
		/// # Parameters
		///
		/// - `origin`: Origin of the transaction.
		/// - `amount`: Amount of funds to unbond.
		///
		/// This function will return error if
		/// - If there is any active role assigned to the user.
		///  
		/// NOTE : This call wraps pallet_staking.unbond, so the pallet_staking.unbond call should
		/// be blocked in runtime
		#[pallet::weight(<T as pallet::Config>::WeightInfo::unbond_funds())]
		#[pallet::call_index(4)]
		pub fn unbond_funds(
			origin: OriginFor<T>,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResult {
			let restaker_account = ensure_signed(origin.clone())?;
			// Ensure no role is assigned to the restaker_account and is eligible to unbond.
			ensure!(Self::can_exit(restaker_account), Error::<T>::HasRoleAssigned);

			// Unbond funds.
			let res = pallet_staking::Pallet::<T>::unbond(origin, amount);
			match res {
				Ok(_) => Ok(()),
				Err(dispatch_post_info) => Err(dispatch_post_info.error),
			}
		}

		/// Withdraw unbond funds after un-bonding period has passed.
		///
		/// # Parameters
		///
		/// - `origin`: Origin of the transaction.
		///
		/// This function will return error if
		/// - If there is any active role assigned to the user.
		/// NOTE : This call wraps pallet_staking.withdraw_unbonded, so the
		/// pallet_staking.withdraw_unbonded call should be blocked in runtime
		#[pallet::weight(<T as pallet::Config>::WeightInfo::withdraw_unbonded())]
		#[pallet::call_index(5)]
		pub fn withdraw_unbonded(origin: OriginFor<T>) -> DispatchResult {
			let restaker_account = ensure_signed(origin.clone())?;
			// Ensure no role is assigned to the restaker_account and is eligible to withdraw.
			ensure!(Self::can_exit(restaker_account.clone()), Error::<T>::HasRoleAssigned);

			// if any unlocking restake, then remove any eligible restake
			let mut ledger =
				Ledger::<T>::get(&restaker_account.clone()).ok_or(Error::<T>::NoProfileFound)?;

			if !ledger.unlocking.is_empty() {
				let current_era =
					Self::active_restaker_era().ok_or(Error::<T>::InvalidEraToReward)?.index;

				ledger.unlocking.retain(|x| x.era >= current_era);
				Ledger::<T>::insert(restaker_account, ledger);
			}

			// Withdraw unbond funds.
			let res = pallet_staking::Pallet::<T>::withdraw_unbonded(origin, 0);
			match res {
				Ok(_) => Ok(()),
				Err(dispatch_post_info) => Err(dispatch_post_info.error),
			}
		}

		/// Pay out all the stakers behind a single restaker for a single era.
		///
		/// - `validator_stash` is the stash account of the restaker. Their nominators, up to
		///   `T::MaxNominatorRewardedPerValidator`, will also receive their rewards.
		/// - `era` may be any era between `[current_era - history_depth; current_era]`.
		///
		/// The origin of this call must be _Signed_. Any account can call this function, even if
		/// it is not one of the stakers.
		/// Note : This will only payout the restaker rewards, the validator/restaker will have to
		/// call staking.payout_stakers to claim staking rewards
		///
		/// ## Complexity
		/// - At most O(MaxNominatorRewardedPerValidator).
		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::withdraw_unbonded())]
		pub fn payout_stakers(
			origin: OriginFor<T>,
			validator_stash: T::AccountId,
			era: EraIndex,
		) -> DispatchResult {
			ensure_signed(origin)?;
			Self::do_payout_stakers(validator_stash, era)
		}

		#[pallet::call_index(7)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::withdraw_unbonded())]
		pub fn set_min_restaking_bond(
			origin: OriginFor<T>,
			min_restaking_bond: BalanceOf<T>,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;
			MinRestakingBond::<T>::put(min_restaking_bond.clone());
			Self::deposit_event(Event::<T>::MinRestakingBondUpdated { value: min_restaking_bond });
			Ok(())
		}
	}
}
