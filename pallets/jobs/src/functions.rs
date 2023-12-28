use super::*;
use sp_runtime::traits::Zero;
use tangle_primitives::jobs::{
	DKGTSSPhaseOneJobType, DKGTSSSignatureResult, JobKey, JobType, JobWithResult,
	ZkSaaSCircuitResult, ZkSaaSPhaseOneJobType, ZkSaaSProofResult,
};

impl<T: Config> Pallet<T> {
	/// Add a job ID to the validator lookup.
	///
	/// This function associates a job ID with a specific validator account.
	///
	/// # Parameters
	///
	/// - `validator`: The account ID of the validator.
	/// - `job_id`: The ID of the job to associate with the validator.
	/// - `job_key`: The job key of the job
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
		SubmittedJobs::<T>::try_mutate(job_key, job_id, |job_info| -> DispatchResult {
			let job_info = job_info.as_mut().ok_or(Error::<T>::JobNotFound)?;

			let phase1_result = if !job_info.job_type.is_phase_one() {
				Some(
					KnownResults::<T>::get(job_key, job_id)
						.ok_or(Error::<T>::PhaseOneResultNotFound)?,
				)
			} else {
				None
			};

			// If the job type is in the phase one.
			// If it is, adjusts the participants and threshold accordingly.
			// Ensures that the threshold is not zero after adjustment.
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
					// Case for JobKey::DKGSignature
					// - Extract information from 'phase1'
					// - Create a new 'job_type' of DKGJobType with adjusted parameters (remove the
					//   reported validator and reduce threshold)
					// - Charge the validator fee for job submission
					// - Store information about the submitted job in 'SubmittedJobs'
					JobKey::DKGSignature => {
						let new_participants = phase1
							.participants()
							.ok_or(Error::<T>::InvalidJobPhase)?
							.into_iter()
							.filter(|x| x != &validator)
							.collect();
						let new_threshold = phase1
							.threshold()
							.ok_or(Error::<T>::InvalidJobPhase)?
							.saturating_sub(1);
						ensure!(!new_threshold.is_zero(), Error::<T>::NotEnoughValidators);

						let job_type = JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType {
							key_type: phase1.key_type.clone().expect("Checked above"),
							participants: new_participants,
							threshold: new_threshold,
							permitted_caller: phase1.clone().permitted_caller,
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
						SubmittedJobs::<T>::insert(job_key, job_id, job_info);
					},
					// Case for JobKey::ZkSaasProve
					// - Extract information from 'phase1'
					// - Create a new 'job_type' of ZkSaasPhaseOneJobType with adjusted parameters
					//   (remove the reported validator)
					// - Charge the validator fee for job submission
					// - Store information about the submitted job in 'SubmittedJobs'
					JobKey::ZkSaaSProve => {
						let new_participants = phase1
							.participants()
							.ok_or(Error::<T>::InvalidJobPhase)?
							.into_iter()
							.filter(|x| x != &validator)
							.collect();
						let phase_one_id = job_info
							.job_type
							.get_phase_one_id()
							.ok_or(Error::<T>::PhaseOneResultNotFound)?;
						let phase_one =
							SubmittedJobs::<T>::get(JobKey::ZkSaaSCircuit, phase_one_id)
								.ok_or(Error::<T>::JobNotFound)?;
						let system = match phase_one.job_type {
							JobType::ZkSaaSPhaseOne(ref info) => info.system.clone(),
							_ => return Err(Error::<T>::JobNotFound.into()),
						};

						let job_type = JobType::ZkSaaSPhaseOne(ZkSaaSPhaseOneJobType {
							participants: new_participants,
							system,
							permitted_caller: phase1.clone().permitted_caller,
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
						SubmittedJobs::<T>::insert(job_key, job_id, job_info);
					},
					_ => {
						// The phase one cases are handled above
					},
				};

				// the old results are not useful since a participant has left, remove from storage
				KnownResults::<T>::remove(job_key, job_id);
			}
			Ok(())
		})
	}

	pub fn verify_dkg_job_result(
		job_key: JobKey,
		job_info: &JobInfoOf<T>,
		info: DKGTSSKeySubmissionResult,
	) -> Result<PhaseOneResultOf<T>, DispatchError> {
		// sanity check, does job and result type match
		ensure!(job_key == JobKey::DKG, Error::<T>::ResultNotExpectedType);

		// ensure the participants are the expected participants from job
		let participants = job_info
			.job_type
			.clone()
			.get_participants()
			.ok_or(Error::<T>::InvalidJobParams)?;
		let mut participant_keys: Vec<Vec<u8>> = Default::default();

		for participant in participants.clone() {
			let key = T::RolesHandler::get_validator_metadata(participant, job_key);
			ensure!(key.is_some(), Error::<T>::ValidatorMetadataNotFound);
			participant_keys.push(key.expect("checked above").get_authority_key());
		}

		let job_result = JobResult::DKGPhaseOne(DKGTSSKeySubmissionResult {
			key: info.key.clone(),
			signatures: info.signatures,
			participants: participant_keys,
			threshold: job_info.job_type.clone().get_threshold().expect("Checked before"),
			key_type: info.key_type.clone(),
		});

		T::MPCHandler::verify(JobWithResult {
			job_type: job_info.job_type.clone(),
			phase_one_job_type: None,
			result: job_result,
		})?;

		let result = PhaseOneResult {
			owner: job_info.owner.clone(),
			expiry: job_info.expiry,
			job_type: job_info.job_type.clone(),
			permitted_caller: job_info.job_type.clone().get_permitted_caller(),
			result: info.key,
			key_type: Some(info.key_type),
		};
		Ok(result)
	}

	pub fn verify_dkg_signature_job_result(
		job_key: JobKey,
		job_info: &JobInfoOf<T>,
		info: DKGTSSSignatureResult,
	) -> DispatchResult {
		let now = <frame_system::Pallet<T>>::block_number();
		// sanity check, does job and result type match
		ensure!(job_key == JobKey::DKGSignature, Error::<T>::ResultNotExpectedType);

		let existing_result_id = job_info
			.job_type
			.clone()
			.get_phase_one_id()
			.ok_or(Error::<T>::InvalidJobPhase)?;
		// Ensure the result exists
		let phase_one_result = KnownResults::<T>::get(
			job_info.job_type.clone().get_previous_phase_job_key().unwrap(),
			existing_result_id,
		)
		.ok_or(Error::<T>::PreviousResultNotFound)?;

		// Validate existing result
		ensure!(phase_one_result.expiry >= now, Error::<T>::ResultExpired);

		// ensure the participants are the expected participants from job
		let mut participant_keys: Vec<sp_core::ecdsa::Public> = Default::default();

		let participants = phase_one_result.participants().ok_or(Error::<T>::InvalidJobPhase)?;
		for participant in participants {
			let key = T::RolesHandler::get_validator_metadata(participant, job_key);
			ensure!(key.is_some(), Error::<T>::ValidatorMetadataNotFound);
			let pub_key = sp_core::ecdsa::Public::from_slice(
				&key.expect("checked above").get_authority_key()[0..33],
			)
			.map_err(|_| Error::<T>::InvalidValidator)?;
			participant_keys.push(pub_key);
		}

		let job_result = JobResult::DKGPhaseTwo(DKGTSSSignatureResult {
			signature: info.signature,
			data: info.data,
			signing_key: phase_one_result.result,
			key_type: info.key_type,
		});

		let phase_one_job_info = KnownResults::<T>::get(
			job_info
				.job_type
				.get_previous_phase_job_key()
				.ok_or(Error::<T>::InvalidJobPhase)?,
			job_info.job_type.get_phase_one_id().ok_or(Error::<T>::InvalidJobPhase)?,
		)
		.ok_or(Error::<T>::JobNotFound)?;
		T::MPCHandler::verify(JobWithResult {
			job_type: job_info.job_type.clone(),
			phase_one_job_type: Some(phase_one_job_info.job_type),
			result: job_result,
		})?;
		Ok(())
	}

	pub fn verify_zksaas_circuit_job_result(
		job_key: JobKey,
		job_id: JobId,
		job_info: &JobInfoOf<T>,
		_info: ZkSaaSCircuitResult,
	) -> Result<PhaseOneResultOf<T>, DispatchError> {
		// sanity check, does job and result type match
		ensure!(job_key == JobKey::ZkSaaSCircuit, Error::<T>::ResultNotExpectedType);
		// ensure the participants are the expected participants from job

		let participants = job_info
			.job_type
			.clone()
			.get_participants()
			.ok_or(Error::<T>::InvalidJobParams)?;
		let mut participant_keys: Vec<sp_core::ecdsa::Public> = Default::default();

		for participant in participants.clone() {
			let key = T::RolesHandler::get_validator_metadata(participant, job_key);
			ensure!(key.is_some(), Error::<T>::ValidatorMetadataNotFound);
			let pub_key = sp_core::ecdsa::Public::from_slice(
				&key.expect("checked above").get_authority_key()[0..33],
			)
			.map_err(|_| Error::<T>::InvalidValidator)?;
			participant_keys.push(pub_key);
		}

		let job_result = JobResult::ZkSaaSPhaseOne(ZkSaaSCircuitResult {
			job_id,
			participants: participant_keys,
		});

		T::MPCHandler::verify(JobWithResult {
			job_type: job_info.job_type.clone(),
			phase_one_job_type: None,
			result: job_result,
		})?;

		let result = PhaseOneResult {
			owner: job_info.owner.clone(),
			expiry: job_info.expiry,
			job_type: job_info.job_type.clone(),
			// No data in the result
			result: Vec::new(),
			permitted_caller: job_info.job_type.clone().get_permitted_caller(),
			key_type: None,
		};
		Ok(result)
	}

	pub fn verify_zksaas_prove_job_result(
		job_key: JobKey,
		job_info: &JobInfoOf<T>,
		info: ZkSaaSProofResult,
	) -> DispatchResult {
		let now = <frame_system::Pallet<T>>::block_number();
		// sanity check, does job and result type match
		ensure!(job_key == JobKey::DKGSignature, Error::<T>::ResultNotExpectedType);

		let existing_result_id = job_info
			.job_type
			.clone()
			.get_phase_one_id()
			.ok_or(Error::<T>::InvalidJobPhase)?;
		// Ensure the result exists
		let phase_one_result = KnownResults::<T>::get(
			job_info.job_type.clone().get_previous_phase_job_key().unwrap(),
			existing_result_id,
		)
		.ok_or(Error::<T>::PreviousResultNotFound)?;

		// Validate existing result
		ensure!(phase_one_result.expiry >= now, Error::<T>::ResultExpired);

		let job_result = JobResult::ZkSaaSPhaseTwo(info);

		let phase_one_job_info = SubmittedJobs::<T>::get(
			job_info
				.job_type
				.get_previous_phase_job_key()
				.ok_or(Error::<T>::InvalidJobPhase)?,
			job_info.job_type.get_phase_one_id().ok_or(Error::<T>::InvalidJobPhase)?,
		)
		.ok_or(Error::<T>::JobNotFound)?;
		T::MPCHandler::verify(JobWithResult {
			job_type: job_info.job_type.clone(),
			phase_one_job_type: Some(phase_one_job_info.job_type),
			result: job_result,
		})?;
		Ok(())
	}
}
