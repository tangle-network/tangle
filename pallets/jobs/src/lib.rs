// This file is part of Webb.
// Copyright (C) 2022 Webb Technologies Inc.
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
use sp_runtime::{traits::AccountIdConversion, DispatchResult};
use sp_std::{prelude::*, vec::Vec};
use tangle_primitives::{
	jobs::{JobId, JobInfo, JobKey, PhaseOneResult, ValidatorOffence},
	traits::{
		jobs::{JobResultVerifier, JobToFee},
		roles::RolesHandler,
	},
};

mod functions;
mod impls;
mod mock;
mod tests;
mod types;

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
		type JobResultVerifier: JobResultVerifier<
			Self::AccountId,
			BlockNumberFor<Self>,
			BalanceOf<Self>,
		>;

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
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::pause_transaction())]
		pub fn submit_job(origin: OriginFor<T>, job: JobSubmissionOf<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let now = <frame_system::Pallet<T>>::block_number();

			let job_id = Self::get_next_job_id()?;
			let job_key = job.job_type.get_job_key();

			// ensure the job can be processed
			if job.job_type.is_phase_one() {
				// ensure all the participants have valid roles
				let participants =
					job.job_type.clone().get_participants().ok_or(Error::<T>::InvalidJobPhase)?;

				for participant in participants {
					ensure!(
						T::RolesHandler::is_validator(participant.clone(), job_key.clone()),
						Error::<T>::InvalidValidator
					);

					// add record for easy lookup
					Self::add_job_to_validator_lookup(participant, job_key.clone(), job_id)?;
				}

				// sanity check ensure threshold is valid
				ensure!(job.job_type.sanity_check(), Error::<T>::InvalidJobParams);
			}
			// phase two validations
			else {
				let existing_result_id =
					job.job_type.clone().get_phase_one_id().ok_or(Error::<T>::InvalidJobPhase)?;
				// ensure the result exists
				let result = KnownResults::<T>::get(
					job.job_type.clone().get_previous_phase_job_key().unwrap(),
					existing_result_id,
				)
				.ok_or(Error::<T>::PreviousResultNotFound)?;

				// validate existing result
				ensure!(result.expiry >= now, Error::<T>::ResultExpired);

				// ensure the phase one participants are still validators
				for participant in result.participants {
					ensure!(
						T::RolesHandler::is_validator(participant.clone(), job_key.clone()),
						Error::<T>::InvalidValidator
					);

					// add record for easy lookup
					Self::add_job_to_validator_lookup(participant, job_key.clone(), job_id)?;
				}

				// ensure the caller generated the phase one result
				ensure!(result.owner == caller, Error::<T>::InvalidJobParams);
			}

			// basic sanity checks
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

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::pause_transaction())]
		pub fn submit_job_result(
			origin: OriginFor<T>,
			job_key: JobKey,
			job_id: JobId,
			result: Vec<u8>,
		) -> DispatchResult {
			let _caller = ensure_signed(origin)?;

			// ensure the job exists
			let job_info =
				SubmittedJobs::<T>::get(job_key.clone(), job_id).ok_or(Error::<T>::JobNotFound)?;

			let mut phase1_result: Option<PhaseOneResultOf<T>> = None;

			// if phase2, fetch phase1 result
			if !job_info.job_type.is_phase_one() {
				let result = KnownResults::<T>::get(job_key.clone(), job_id)
					.ok_or(Error::<T>::PhaseOneResultNotFound)?;
				phase1_result = Some(result);
			}

			// validate the result
			T::JobResultVerifier::verify(&job_info, phase1_result.clone(), result.clone())?;

			// if phase 1, store in known result
			if job_info.job_type.is_phase_one() {
				let result = PhaseOneResult {
					owner: job_info.owner,
					expiry: job_info.expiry,
					result,
					participants: job_info.job_type.clone().get_participants().unwrap(),
					threshold: job_info.job_type.clone().get_threshold(),
				};

				KnownResults::<T>::insert(job_key.clone(), job_id, result);
			}

			// distribute fee to all participants
			let participants = if job_info.job_type.is_phase_one() {
				job_info.job_type.clone().get_participants().unwrap()
			} else {
				phase1_result.unwrap().participants
			};

			let fee_per_participant = job_info.fee / (participants.len() as u32).into();

			// record reward to all participants
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

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::pause_transaction())]
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

		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::pause_transaction())]
		pub fn report_inactive_validator(
			origin: OriginFor<T>,
			job_key: JobKey,
			job_id: JobId,
			validator: T::AccountId,
			offence: ValidatorOffence,
			signatures: Vec<u8>,
		) -> DispatchResult {
			let _caller = ensure_signed(origin)?;

			// remove the validator from the job
			let job_info =
				SubmittedJobs::<T>::get(job_key.clone(), job_id).ok_or(Error::<T>::JobNotFound)?;

			let mut phase1_result: Option<PhaseOneResultOf<T>> = None;

			// if phase2, fetch phase1 result
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

			// validate the result
			T::JobResultVerifier::verify_validator_report(
				validator.clone(),
				offence.clone(),
				signatures,
			)?;

			// slash the validator
			T::RolesHandler::slash_validator(validator.clone(), offence)?;

			// trigger validator removal
			Self::try_validator_removal_from_job(job_key, job_id, validator)?;

			Ok(())
		}
	}
}
