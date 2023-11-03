use super::*;
use sp_runtime::traits::Zero;
use tangle_primitives::jobs::{DKGJobType, JobKey, JobType, ZkSaasPhaseOneJobType};

impl<T: Config> Pallet<T> {
	/// Add a job ID to the validator lookup.
	///
	/// This function associates a job ID with a specific validator account.
	///
	/// # Parameters
	///
	/// - `validator`: The account ID of the validator.
	/// - `job_id`: The ID of the job to associate with the validator.
	///
	/// # Errors
	///
	/// Returns a `DispatchError` if the operation fails.
	pub(crate) fn add_job_to_validator_lookup(
		validator: T::AccountId,
		job_key: JobKey,
		job_id: JobId,
	) -> DispatchResult {
		ValidatorJobIdLookup::<T>::try_mutate(validator, |jobs| -> DispatchResult {
			let jobs = jobs.get_or_insert_with(Default::default);
			jobs.push((job_key, job_id));
			Ok(())
		})
	}

	/// Get the next available job ID.
	///
	/// This function returns the current job ID and increments the internal counter for the next
	/// job.
	///
	/// # Returns
	///
	/// Returns the next available job ID if successful.
	///
	/// # Errors
	///
	/// Returns a `DispatchError` if the operation fails.
	pub(crate) fn get_next_job_id() -> Result<JobId, DispatchError> {
		let current_job_id = NextJobId::<T>::get();
		NextJobId::<T>::try_mutate(|job_id| -> DispatchResult {
			*job_id += 1;
			Ok(())
		})?;
		Ok(current_job_id)
	}

	/// Record rewards to a validator.
	///
	/// This function records the rewards earned by a validator account.
	///
	/// # Parameters
	///
	/// - `validator`: The account ID of the validator.
	/// - `reward`: The amount of rewards to record.
	///
	/// # Errors
	///
	/// Returns a `DispatchError` if the operation fails.
	pub(crate) fn record_reward_to_validator(
		validator: T::AccountId,
		reward: BalanceOf<T>,
	) -> DispatchResult {
		ValidatorRewards::<T>::try_mutate(validator, |existing| -> DispatchResult {
			let existing = existing.get_or_insert_with(Default::default);
			*existing += reward;
			Ok(())
		})
	}

	/// Get the account ID of the rewards pot.
	///
	/// This function returns the account ID associated with the rewards pot.
	pub fn rewards_account_id() -> T::AccountId {
		T::PalletId::get().into_sub_account_truncating(0)
	}

	/// Try to remove a validator from a submitted job.
	///
	/// # Parameters
	///
	/// - `job_key`: A unique identifier for the job category.
	/// - `job_id`: A unique identifier for the job within the category.
	/// - `validator`: The account ID of the validator to be removed.
	///
	/// # Errors
	///
	/// This function can return an error if:
	///
	/// - The specified job is not found.
	/// - The phase one result is not found (for phase two jobs).
	/// - There are not enough validators after removal.
	/// - The threshold is zero.
	/// - The next job ID cannot be generated.
	/// - The fee transfer fails.
	///
	/// # Details
	///
	/// This function attempts to remove a validator from a submitted job. If the job is in phase
	/// two, it fetches the phase one result. It then adjusts the participants and threshold based
	/// on the removal of the validator. If there are not enough validators after removal, an error
	/// is returned. If the job is in phase two, a new job is generated with updated parameters and
	/// the fee is charged from validator. The original job's result is removed.
	pub fn try_validator_removal_from_job(
		job_key: JobKey,
		job_id: JobId,
		validator: T::AccountId,
	) -> DispatchResult {
		SubmittedJobs::<T>::try_mutate(job_key.clone(), job_id, |job_info| -> DispatchResult {
			let job_info = job_info.as_mut().ok_or(Error::<T>::JobNotFound)?;

			let phase1_result = if !job_info.job_type.is_phase_one() {
				Some(
					KnownResults::<T>::get(job_key.clone(), job_id)
						.ok_or(Error::<T>::PhaseOneResultNotFound)?,
				)
			} else {
				None
			};

			if job_info.job_type.is_phase_one() {
				let participants = job_info.job_type.clone().get_participants().unwrap();
				let mut threshold = job_info.job_type.clone().get_threshold().unwrap();

				let participants: Vec<T::AccountId> =
					participants.into_iter().filter(|x| x != &validator).collect();

				if participants.len() <= threshold.into() {
					threshold = threshold.saturating_sub(1);
				}

				ensure!(!threshold.is_zero(), Error::<T>::NotEnoughValidators);
			} else {
				// this phase1 result cannot be used
				let phase1 = phase1_result.as_ref().ok_or(Error::<T>::PhaseOneResultNotFound)?;

				// generate a job to generate new key
				let job_id = Self::get_next_job_id()?;

				match job_key {
					JobKey::DKGSignature => {
						let new_participants = phase1
							.participants
							.clone()
							.into_iter()
							.filter(|x| x != &validator)
							.collect();
						let new_threshold = phase1.threshold.unwrap().saturating_sub(1);
						ensure!(!new_threshold.is_zero(), Error::<T>::NotEnoughValidators);

						let job_type = JobType::DKG(DKGJobType {
							participants: new_participants,
							threshold: new_threshold,
						});

						// charge the validator fee for job submission
						let job = JobSubmissionOf::<T> {
							expiry: phase1.expiry,
							job_type: job_type.clone(),
						};

						let fee = T::JobToFee::job_to_fee(&job);
						T::Currency::transfer(
							&validator,
							&Self::rewards_account_id(),
							fee,
							ExistenceRequirement::KeepAlive,
						)?;

						let job_info = JobInfo {
							owner: phase1.owner.clone(),
							expiry: phase1.expiry,
							job_type,
							fee,
						};
						SubmittedJobs::<T>::insert(job_key.clone(), job_id, job_info);
					},
					JobKey::ZkSaasPhaseTwo => {
						let new_participants = phase1
							.participants
							.clone()
							.into_iter()
							.filter(|x| x != &validator)
							.collect();

						let job_type = JobType::ZkSaasPhaseOne(ZkSaasPhaseOneJobType {
							participants: new_participants,
						});

						// charge the validator fee for job submission
						let job = JobSubmissionOf::<T> {
							expiry: phase1.expiry,
							job_type: job_type.clone(),
						};

						let fee = T::JobToFee::job_to_fee(&job);
						T::Currency::transfer(
							&validator,
							&Self::rewards_account_id(),
							fee,
							ExistenceRequirement::KeepAlive,
						)?;

						let job_info = JobInfo {
							owner: phase1.owner.clone(),
							expiry: phase1.expiry,
							job_type,
							fee,
						};
						SubmittedJobs::<T>::insert(job_key.clone(), job_id, job_info);
					},
					_ => {},
				};

				KnownResults::<T>::remove(job_key, job_id);
			}
			Ok(())
		})
	}
}
