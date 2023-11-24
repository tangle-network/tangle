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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use crate::types::{JobInfoOf, JobSubmissionOf, PhaseOneResultOf};
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, ReservableCurrency},
	PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
	traits::{AccountIdConversion, Zero},
	DispatchResult,
};
use sp_std::{prelude::*, vec::Vec};
use tangle_primitives::{
	jobs::{JobId, JobInfo, JobKey, PhaseOneResult, ValidatorOffenceType},
	traits::{
		jobs::{JobToFee, MPCHandler},
		roles::RolesHandler,
	},
};

mod functions;
mod impls;
mod mock;
mod rpc;
mod tests;
mod types;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
use crate::types::BalanceOf;

pub use module::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The job to fee converter
		type JobToFee: JobToFee<Self::AccountId, BlockNumberFor<Self>, Balance = BalanceOf<Self>>;

		/// The roles manager mechanism
		type RolesHandler: RolesHandler<Self::AccountId>;

		/// The job result verifying mechanism
		type MPCHandler: MPCHandler<Self::AccountId, BlockNumberFor<Self>, BalanceOf<Self>>;

		/// The origin which may set filter.
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// `PalletId` for the jobs pallet.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// invalid phase provided
		InvalidJobPhase,
		/// Given validator not valid for job type
		InvalidValidator,
		/// invalid params, cannot execute jobs
		InvalidJobParams,
		/// cannot find phase 1 result
		PreviousResultNotFound,
		/// The previous result expired
		ResultExpired,
		/// Invalid job expiry input
		JobAlreadyExpired,
		/// The requested job was not found
		JobNotFound,
		/// P1 result not found
		PhaseOneResultNotFound,
		/// no rewards found for validator
		NoRewards,
		/// Not enough validators to exit
		NotEnoughValidators,
		/// empty result
		EmptyResult,
		/// empty job
		EmptyJob,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new job has been submitted
		JobSubmitted { job_id: JobId, job_key: JobKey, details: JobSubmissionOf<T> },
		/// A new job result has been submitted
		JobResultSubmitted { job_id: JobId, job_key: JobKey },
		/// validator has earned reward
		ValidatorRewarded { id: T::AccountId, reward: BalanceOf<T> },
	}

	/// The paused transaction map
	#[pallet::storage]
	#[pallet::getter(fn submitted_jobs)]
	pub type SubmittedJobs<T: Config> =
		StorageDoubleMap<_, Twox64Concat, JobKey, Blake2_128Concat, JobId, JobInfoOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn known_results)]
	pub type KnownResults<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		JobKey,
		Blake2_128Concat,
		JobId,
		PhaseOneResult<T::AccountId, BlockNumberFor<T>>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn validator_job_id_lookup)]
	pub type ValidatorJobIdLookup<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Vec<(JobKey, JobId)>>;

	#[pallet::storage]
	#[pallet::getter(fn validator_rewards)]
	pub type ValidatorRewards<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>>;

	/// The job-id storage
	#[pallet::storage]
	#[pallet::getter(fn next_job_id)]
	pub type NextJobId<T: Config> = StorageValue<_, JobId, ValueQuery>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Submit a job for processing.
		///
		/// # Parameters
		///
		/// - `origin`: The origin of the call (typically a signed account).
		/// - `job`: The details of the job to be submitted.
		///
		/// # Errors
		///
		/// This function can return an error if:
		///
		/// - The caller is not authorized.
		/// - The job type is invalid or has invalid participants.
		/// - The threshold or parameters for phase one jobs are invalid.
		/// - The result for phase two jobs does not exist or is expired.
		/// - The phase one participants are not valid validators.
		/// - The caller did not generate the phase one result for phase two jobs.
		/// - The job has already expired.
		/// - The fee transfer fails.
		///
		/// # Details
		///
		/// This function allows a caller to submit a job for processing. For phase one jobs, it
		/// ensures that all participants have valid roles and performs a sanity check on the
		/// threshold. For phase two jobs, it validates the existence and expiration of the phase
		/// one result, as well as the validity of phase one participants. It also verifies that the
		/// caller generated the phase one result. The user is charged a fee based on job params.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::submit_job())]
		pub fn submit_job(origin: OriginFor<T>, job: JobSubmissionOf<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let now = <frame_system::Pallet<T>>::block_number();

			let job_id = Self::get_next_job_id()?;
			let job_key = job.job_type.get_job_key();

			// Ensure the job can be processed
			if job.job_type.is_phase_one() {
				// Ensure all the participants have valid roles
				let participants =
					job.job_type.clone().get_participants().ok_or(Error::<T>::InvalidJobPhase)?;

				ensure!(!participants.len().is_zero(), Error::<T>::InvalidJobPhase);

				for participant in participants {
					ensure!(
						T::RolesHandler::is_validator(participant.clone(), job_key.clone()),
						Error::<T>::InvalidValidator
					);

					// Add record for easy lookup
					Self::add_job_to_validator_lookup(participant, job_key.clone(), job_id)?;
				}

				// Sanity check ensure threshold is valid
				ensure!(job.job_type.sanity_check(), Error::<T>::InvalidJobParams);
			}
			// phase two validations
			else {
				let existing_result_id =
					job.job_type.clone().get_phase_one_id().ok_or(Error::<T>::InvalidJobPhase)?;
				// Ensure the result exists
				let result = KnownResults::<T>::get(
					job.job_type.clone().get_previous_phase_job_key().unwrap(),
					existing_result_id,
				)
				.ok_or(Error::<T>::PreviousResultNotFound)?;

				// Validate existing result
				ensure!(result.expiry >= now, Error::<T>::ResultExpired);

				// Ensure the phase one participants are still validators
				for participant in result.participants {
					ensure!(
						T::RolesHandler::is_validator(participant.clone(), job_key.clone()),
						Error::<T>::InvalidValidator
					);

					// add record for easy lookup
					Self::add_job_to_validator_lookup(participant, job_key.clone(), job_id)?;
				}

				// Ensure the caller generated the phase one result
				ensure!(result.owner == caller, Error::<T>::InvalidJobParams);
			}

			// Basic sanity checks
			ensure!(job.expiry > now, Error::<T>::JobAlreadyExpired);

			// charge the user fee for job submission
			let fee = T::JobToFee::job_to_fee(&job);
			T::Currency::transfer(
				&caller,
				&Self::rewards_account_id(),
				fee,
				ExistenceRequirement::KeepAlive,
			)?;

			// store the job to pallet
			let job_info = JobInfo {
				owner: caller.clone(),
				expiry: job.expiry,
				job_type: job.job_type.clone(),
				fee,
			};
			SubmittedJobs::<T>::insert(job_key.clone(), job_id, job_info);

			Self::deposit_event(Event::JobSubmitted { job_id, job_key, details: job });

			Ok(())
		}

		/// Submit the result of a job
		///
		/// # Parameters
		///
		/// - `origin`: The origin of the call (typically a signed account).
		/// - `job_key`: A unique identifier for the job category.
		/// - `job_id`: A unique identifier for the job within the category.
		/// - `result`: A vector containing the result of the job.
		///
		/// # Errors
		///
		/// This function can return an error if:
		///
		/// - The caller is not authorized.
		/// - The specified job is not found.
		/// - The phase one result is not found (for phase two jobs).
		/// - The result fails verification.
		/// - The fee distribution or reward recording fails.
		///
		/// # Details
		///
		/// This function allows a caller to submit the result of a job. The function validates the
		/// result using the result verifier config. If the result is valid, it proceeds to store
		/// the result in known results for phase one jobs. It distributes the fee among the
		/// participants and records rewards for each participant. Finally, it removes the job from
		/// the submitted jobs storage.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::submit_job_result())]
		pub fn submit_job_result(
			origin: OriginFor<T>,
			job_key: JobKey,
			job_id: JobId,
			result: Vec<u8>,
		) -> DispatchResult {
			let _caller = ensure_signed(origin)?;

			// sanity check
			ensure!(result.is_empty(), Error::<T>::EmptyResult);

			// Ensure the job exists
			let job_info =
				SubmittedJobs::<T>::get(job_key.clone(), job_id).ok_or(Error::<T>::JobNotFound)?;

			let phase1_result = if !job_info.job_type.is_phase_one() {
				Some(
					KnownResults::<T>::get(
						job_info
							.job_type
							.get_previous_phase_job_key()
							.ok_or(Error::<T>::InvalidJobPhase)?,
						job_info
							.job_type
							.clone()
							.get_phase_one_id()
							.ok_or(Error::<T>::InvalidJobPhase)?,
					)
					.ok_or(Error::<T>::PhaseOneResultNotFound)?,
				)
			} else {
				None
			};

			// Validate the result
			T::MPCHandler::verify(&job_info, phase1_result.clone(), result.clone())?;

			// If phase 1, store in known result
			if job_info.job_type.is_phase_one() {
				let result = PhaseOneResult {
					owner: job_info.owner,
					expiry: job_info.expiry,
					result,
					participants: job_info
						.job_type
						.clone()
						.get_participants()
						.ok_or(Error::<T>::InvalidJobPhase)?,
					threshold: job_info.job_type.clone().get_threshold(),
				};

				KnownResults::<T>::insert(job_key.clone(), job_id, result);
			}

			// Record fee rewards for all job participants
			let participants = if job_info.job_type.is_phase_one() {
				job_info.job_type.clone().get_participants().unwrap()
			} else {
				phase1_result.unwrap().participants
			};

			let fee_per_participant = job_info.fee / (participants.len() as u32).into();

			for participant in participants {
				Self::record_reward_to_validator(participant.clone(), fee_per_participant)?;
				Self::deposit_event(Event::ValidatorRewarded {
					id: participant,
					reward: fee_per_participant,
				});
			}

			SubmittedJobs::<T>::remove(job_key.clone(), job_id);

			Self::deposit_event(Event::JobResultSubmitted { job_id, job_key });

			Ok(())
		}

		/// Withdraw rewards accumulated by the caller.
		///
		/// # Parameters
		///
		/// - `origin`: The origin of the call (typically a signed account).
		///
		/// # Errors
		///
		/// This function can return an error if:
		///
		/// - The caller is not authorized.
		/// - No rewards are available for the caller.
		/// - The reward transfer operation fails.
		///
		/// # Details
		///
		/// This function allows a caller to withdraw rewards that have been accumulated in their
		/// account.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::withdraw_rewards())]
		pub fn withdraw_rewards(origin: OriginFor<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			let reward_available =
				ValidatorRewards::<T>::get(caller.clone()).ok_or(Error::<T>::NoRewards)?;

			T::Currency::transfer(
				&Self::rewards_account_id(),
				&caller,
				reward_available,
				ExistenceRequirement::KeepAlive,
			)?;

			ValidatorRewards::<T>::remove(caller);

			Ok(())
		}

		/// Report an inactive validator and take appropriate actions.
		///
		/// # Parameters
		///
		/// - `origin`: The origin of the call (typically a signed account).
		/// - `job_key`: A unique identifier for the job category.
		/// - `job_id`: A unique identifier for the job within the category.
		/// - `validator`: The account ID of the inactive validator.
		/// - `offence`: The type of offence committed by the validator.
		/// - `signatures`: A vector of signatures related to the report.
		///
		/// # Errors
		///
		/// This function can return an error if:
		///
		/// - The caller is not authorized.
		/// - The specified job is not found.
		/// - The phase one result is not found (for phase two jobs).
		/// - The specified validator is not part of the job participants.
		/// - The validator report is not valid or fails verification.
		/// - Slashing the validator fails.
		/// - Trying to remove the validator from the job fails.
		///
		/// # Details
		///
		/// This function allows a caller to report an inactive validator.
		/// It ensures that the specified validator is part of the job participants. The function
		/// then validates the report using the Result verifier config. If the report is valid, it
		/// will slash the validator.
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::report_inactive_validator())]
		pub fn report_inactive_validator(
			origin: OriginFor<T>,
			job_key: JobKey,
			job_id: JobId,
			validator: T::AccountId,
			offence: ValidatorOffenceType,
			signatures: Vec<u8>,
		) -> DispatchResult {
			let _caller = ensure_signed(origin)?;

			// Remove the validator from the job
			let job_info =
				SubmittedJobs::<T>::get(job_key.clone(), job_id).ok_or(Error::<T>::JobNotFound)?;

			let mut phase1_result: Option<PhaseOneResultOf<T>> = None;

			// If phase2, fetch phase1 result
			if !job_info.job_type.is_phase_one() {
				let result = KnownResults::<T>::get(job_key.clone(), job_id)
					.ok_or(Error::<T>::PhaseOneResultNotFound)?;
				phase1_result = Some(result);
			}

			let participants = if job_info.job_type.is_phase_one() {
				job_info.job_type.clone().get_participants().unwrap()
			} else {
				phase1_result.unwrap().participants
			};

			ensure!(participants.contains(&validator), Error::<T>::JobNotFound);

			// Validate the result
			T::MPCHandler::verify_validator_report(validator.clone(), offence.clone(), signatures)?;

			// Slash the validator
			T::RolesHandler::slash_validator(validator.clone(), offence)?;

			// Trigger validator removal
			Self::try_validator_removal_from_job(job_key, job_id, validator)?;

			Ok(())
		}
	}
}
