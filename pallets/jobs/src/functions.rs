use super::*;
use crate::types::{
	DKGTSSKeyRotationResultOf, DKGTSSKeySubmissionResultOf, DKGTSSSignatureResultOf,
	ParticipantKeyOf, ParticipantKeysOf, ZkSaaSCircuitResultOf, ZkSaaSProofResultOf,
};
use sp_runtime::traits::Zero;
use tangle_primitives::{
	jobs::{
		DKGTSSKeyRefreshResult, DKGTSSPhaseOneJobType, FallbackOptions, JobType, JobWithResult,
		ZkSaaSPhaseOneJobType,
	},
	roles::RoleType,
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
	/// - `role_type`: An identifier for the role type of the job.
	///
	/// # Errors
	///
	/// Returns a `DispatchError` if the operation fails.
	pub(crate) fn add_job_to_validator_lookup(
		validator: T::AccountId,
		role_type: RoleType,
		job_id: JobId,
	) -> DispatchResult {
		ValidatorJobIdLookup::<T>::try_mutate(validator.clone(), |jobs| -> DispatchResult {
			let jobs = jobs.get_or_insert_with(Default::default);

			// ensure the max limit of validator is followed
			let max_allowed = T::RolesHandler::get_max_active_service_for_restaker(validator)
				.unwrap_or_else(Default::default);

			jobs.try_push((role_type, job_id))
				.map_err(|_| Error::<T>::TooManyJobsForValidator)?;

			ensure!(jobs.len() <= max_allowed as usize, Error::<T>::TooManyJobsForValidator);
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

	pub fn refund_job_creator(owner: &T::AccountId, fee: BalanceOf<T>) -> DispatchResult {
		// refund the job creator
		T::Currency::transfer(
			&Self::rewards_account_id(),
			owner,
			fee,
			ExistenceRequirement::AllowDeath,
		)
	}

	/// Try to remove a validator from a submitted job.
	///
	/// # Parameters
	///
	/// - `role_type`: An identifier for the role type of the job.
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
		role_type: RoleType,
		job_id: JobId,
		validator: T::AccountId,
	) -> DispatchResult {
		SubmittedJobs::<T>::try_mutate(role_type, job_id, |maybe_job_info| -> DispatchResult {
			let job_info = maybe_job_info.as_mut().ok_or(Error::<T>::JobNotFound)?;

			if job_info.job_type.is_phase_one() {
				match job_info.fallback {
					FallbackOptions::Destroy => {
						Self::refund_job_creator(&job_info.owner, job_info.fee)?;
						Self::deposit_event(Event::JobRefunded { job_id, role_type });
						// remove the job from storage
						*maybe_job_info = None;
						Ok(())
					},
					FallbackOptions::RegenerateWithThreshold(new_threshold) => {
						let participants = job_info.job_type.clone().get_participants().unwrap();

						let participants: Vec<T::AccountId> =
							participants.into_iter().filter(|x| x != &validator).collect();

						if participants.len() <= new_threshold as usize {
							return Err(Error::<T>::NotEnoughValidators.into());
						}

						ensure!(!new_threshold.is_zero(), Error::<T>::NotEnoughValidators);

						// set fallback to destroy since we already regenerated
						job_info.fallback = FallbackOptions::Destroy;

						Self::deposit_event(Event::JobParticipantsUpdated {
							job_id,
							role_type,
							details: job_info.clone(),
						});
						Ok(())
					},
				}
			} else {
				let phase_one_id = job_info
					.job_type
					.get_phase_one_id()
					.ok_or(Error::<T>::PhaseOneResultNotFound)?;

				let phase1 = KnownResults::<T>::take(role_type, phase_one_id)
					.ok_or(Error::<T>::PhaseOneResultNotFound)?;

				let new_participants: BoundedVec<_, _> = phase1
					.participants()
					.ok_or(Error::<T>::InvalidJobPhase)?
					.into_iter()
					.filter(|x| x != &validator)
					.collect::<Vec<_>>()
					.try_into()
					.map_err(|_| Error::<T>::TooManyParticipants)?;

				#[allow(clippy::collapsible_if)]
				match job_info.fallback {
					FallbackOptions::Destroy => {
						// if the role is TSS, then destory only if signing is impossible
						if matches!(role_type, RoleType::Tss(_)) {
							if new_participants.len()
								>= job_info
									.job_type
									.clone()
									.get_threshold()
									.expect("Should exist!")
									.into()
							{
								return Ok(());
							}
						}

						Self::refund_job_creator(&job_info.owner, job_info.fee)?;
						Self::deposit_event(Event::JobRefunded { job_id, role_type });
						// remove the job from storage
						*maybe_job_info = None;
						Ok(())
					},
					FallbackOptions::RegenerateWithThreshold(new_threshold) => {
						// Generate a job to generate new key
						let job_id = Self::get_next_job_id()?;

						ensure!(!new_threshold.is_zero(), Error::<T>::NotEnoughValidators);

						let job_type: JobType<
							T::AccountId,
							T::MaxParticipants,
							T::MaxSubmissionLen,
							T::MaxAdditionalParamsLen,
						> = match role_type {
							RoleType::Tss(role) => JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType {
								role_type: role,
								participants: new_participants,
								threshold: new_threshold,
								permitted_caller: phase1.clone().permitted_caller,
								hd_wallet: match phase1.job_type {
									JobType::DKGTSSPhaseOne(info) => info.hd_wallet,
									_ => false,
								},
							}),

							RoleType::ZkSaaS(role) => {
								let phase_one = SubmittedJobs::<T>::get(role_type, phase_one_id)
									.ok_or(Error::<T>::JobNotFound)?;
								let system = match phase_one.job_type {
									JobType::ZkSaaSPhaseOne(ref info) => info.system.clone(),
									_ => return Err(Error::<T>::JobNotFound.into()),
								};

								JobType::ZkSaaSPhaseOne(ZkSaaSPhaseOneJobType {
									role_type: role,
									participants: new_participants,
									system,
									permitted_caller: phase1.clone().permitted_caller,
								})
							},
							_ => todo!(),
						};

						// charge the validator fee for job submission
						let job = JobSubmissionOf::<T> {
							expiry: job_info.expiry,
							ttl: job_info.ttl,
							job_type: job_type.clone(),
							fallback: job_info.fallback,
						};

						let fee = T::JobToFee::job_to_fee(&job);

						T::Currency::transfer(
							&validator,
							&Self::rewards_account_id(),
							fee,
							ExistenceRequirement::KeepAlive,
						)?;

						let phase_one_job_info = JobInfo {
							owner: phase1.owner.clone(),
							expiry: job_info.expiry,
							ttl: job_info.ttl,
							job_type,
							fee,
							fallback: FallbackOptions::Destroy, /* set to destroy, we
							                                     * dont want to keep
							                                     * looping */
						};

						Self::deposit_event(Event::JobReSubmitted {
							job_id,
							role_type,
							details: phase_one_job_info.clone(),
						});

						SubmittedJobs::<T>::insert(role_type, job_id, phase_one_job_info);

						// update the current job with new phase-one id
						job_info.job_type.update_phase_one_id(job_id);

						// set fallback to destroy since we already regenerated
						job_info.fallback = FallbackOptions::Destroy;

						// the old results are not useful since a participant has left, remove
						// from storage
						KnownResults::<T>::remove(role_type, job_id);
						Ok(())
					},
				}
			}
		})
	}

	pub fn verify_dkg_job_result(
		role_type: RoleType,
		job_info: &JobInfoOf<T>,
		info: DKGTSSKeySubmissionResultOf<T>,
	) -> Result<PhaseResultOf<T>, DispatchError> {
		// sanity check, does job and result type match
		ensure!(role_type.is_dkg_tss(), Error::<T>::ResultNotExpectedType);

		// ensure the participants are the expected participants from job
		let participants = job_info
			.job_type
			.clone()
			.get_participants()
			.ok_or(Error::<T>::InvalidJobParams)?;
		let mut participant_keys: ParticipantKeysOf<T> = Default::default();

		for participant in participants.clone() {
			let key = T::RolesHandler::get_validator_role_key(participant);
			ensure!(key.is_some(), Error::<T>::ValidatorRoleKeyNotFound);
			let bounded_key: ParticipantKeyOf<T> = key
				.expect("Checked above!")
				.try_into()
				.map_err(|_| Error::<T>::ExceedsMaxKeySize)?;
			participant_keys
				.try_push(bounded_key)
				.map_err(|_| Error::<T>::TooManyParticipants)?;
		}

		let job_result = JobResult::DKGPhaseOne(DKGTSSKeySubmissionResultOf::<T> {
			key: info.key.clone(),
			chain_code: info.chain_code,
			signatures: info.signatures,
			participants: participant_keys,
			threshold: job_info.job_type.clone().get_threshold().expect("Checked before"),
			signature_scheme: info.signature_scheme.clone(),
		});

		T::MPCHandler::verify(JobWithResult {
			job_type: job_info.job_type.clone(),
			phase_one_job_type: None,
			result: job_result.clone(),
		})?;

		let result = PhaseResult {
			owner: job_info.owner.clone(),
			job_type: job_info.job_type.clone(),
			ttl: job_info.ttl,
			permitted_caller: job_info.job_type.clone().get_permitted_caller(),
			result: job_result,
		};
		Ok(result)
	}

	pub fn verify_dkg_signature_job_result(
		role_type: RoleType,
		job_info: &JobInfoOf<T>,
		info: DKGTSSSignatureResultOf<T>,
	) -> Result<PhaseResultOf<T>, DispatchError> {
		let now = <frame_system::Pallet<T>>::block_number();
		// sanity check, does job and result type match
		ensure!(role_type.is_dkg_tss(), Error::<T>::ResultNotExpectedType);

		let existing_result_id = job_info
			.job_type
			.clone()
			.get_phase_one_id()
			.ok_or(Error::<T>::InvalidJobPhase)?;
		// Ensure the result exists
		let phase_one_result =
			KnownResults::<T>::get(job_info.job_type.get_role_type(), existing_result_id)
				.ok_or(Error::<T>::PreviousResultNotFound)?;

		// Validate existing result
		ensure!(phase_one_result.ttl >= now, Error::<T>::ResultExpired);

		let verifying_key = match phase_one_result.result {
			JobResult::DKGPhaseOne(result) => result.key,
			_ => return Err(Error::<T>::InvalidJobPhase.into()),
		};
		let job_result = JobResult::DKGPhaseTwo(DKGTSSSignatureResultOf::<T> {
			signature: info.signature.clone(),
			data: info.data,
			verifying_key,
			signature_scheme: info.signature_scheme,
			derivation_path: info.derivation_path,
		});

		let phase_one_job_info = KnownResults::<T>::get(
			job_info.job_type.get_role_type(),
			job_info.job_type.get_phase_one_id().ok_or(Error::<T>::InvalidJobPhase)?,
		)
		.ok_or(Error::<T>::JobNotFound)?;
		T::MPCHandler::verify(JobWithResult {
			job_type: job_info.job_type.clone(),
			phase_one_job_type: Some(phase_one_job_info.job_type),
			result: job_result.clone(),
		})?;

		let result = PhaseResult {
			owner: job_info.owner.clone(),
			ttl: job_info.ttl,
			job_type: job_info.job_type.clone(),
			permitted_caller: job_info.job_type.clone().get_permitted_caller(),
			result: job_result,
		};
		Ok(result)
	}

	pub fn verify_dkg_key_refresh_job_result(
		role_type: RoleType,
		job_info: &JobInfoOf<T>,
		info: DKGTSSKeyRefreshResult,
	) -> Result<PhaseResultOf<T>, DispatchError> {
		let now = <frame_system::Pallet<T>>::block_number();
		// sanity check, does job and result type match
		ensure!(role_type.is_dkg_tss(), Error::<T>::ResultNotExpectedType);

		let existing_result_id = job_info
			.job_type
			.clone()
			.get_phase_one_id()
			.ok_or(Error::<T>::InvalidJobPhase)?;
		// Ensure the result exists
		let phase_one_result =
			KnownResults::<T>::get(job_info.job_type.get_role_type(), existing_result_id)
				.ok_or(Error::<T>::PreviousResultNotFound)?;

		// Validate existing result
		ensure!(phase_one_result.ttl >= now, Error::<T>::ResultExpired);

		// ensure the participants are the expected participants from job
		let mut participant_keys: Vec<sp_core::ecdsa::Public> = Default::default();

		let participants = phase_one_result.participants().ok_or(Error::<T>::InvalidJobPhase)?;
		for participant in participants {
			let key = T::RolesHandler::get_validator_role_key(participant);
			ensure!(key.is_some(), Error::<T>::ValidatorRoleKeyNotFound);
			let pub_key = sp_core::ecdsa::Public::from_slice(&key.expect("checked above")[0..33])
				.map_err(|_| Error::<T>::InvalidValidator)?;
			participant_keys.push(pub_key);
		}

		let job_result = JobResult::DKGPhaseThree(DKGTSSKeyRefreshResult {
			signature_scheme: info.signature_scheme.clone(),
		});

		let phase_one_job_info = KnownResults::<T>::get(
			job_info.job_type.get_role_type(),
			job_info.job_type.get_phase_one_id().ok_or(Error::<T>::InvalidJobPhase)?,
		)
		.ok_or(Error::<T>::JobNotFound)?;
		T::MPCHandler::verify(JobWithResult {
			job_type: job_info.job_type.clone(),
			phase_one_job_type: Some(phase_one_job_info.job_type),
			result: job_result.clone(),
		})?;

		let result = PhaseResult {
			owner: job_info.owner.clone(),
			job_type: job_info.job_type.clone(),
			ttl: job_info.ttl,
			permitted_caller: job_info.job_type.clone().get_permitted_caller(),
			result: job_result,
		};
		Ok(result)
	}

	pub fn verify_dkg_key_rotation_job_result(
		role_type: RoleType,
		job_info: &JobInfoOf<T>,
		info: DKGTSSKeyRotationResultOf<T>,
	) -> Result<PhaseResultOf<T>, DispatchError> {
		let now = <frame_system::Pallet<T>>::block_number();
		// sanity check, does job and result type match
		ensure!(role_type.is_dkg_tss(), Error::<T>::ResultNotExpectedType);

		let existing_result_id = job_info
			.job_type
			.clone()
			.get_phase_one_id()
			.ok_or(Error::<T>::InvalidJobPhase)?;
		// Ensure the result exists
		let phase_one_result =
			KnownResults::<T>::get(job_info.job_type.get_role_type(), existing_result_id)
				.ok_or(Error::<T>::PreviousResultNotFound)?;

		// Validate existing result
		ensure!(phase_one_result.ttl >= now, Error::<T>::ResultExpired);

		// ensure the participants are the expected participants from job
		let mut participant_keys: Vec<sp_core::ecdsa::Public> = Default::default();

		let participants = phase_one_result.participants().ok_or(Error::<T>::InvalidJobPhase)?;
		for participant in participants {
			let key = T::RolesHandler::get_validator_role_key(participant);
			ensure!(key.is_some(), Error::<T>::ValidatorRoleKeyNotFound);
			let pub_key = sp_core::ecdsa::Public::from_slice(&key.expect("checked above")[0..33])
				.map_err(|_| Error::<T>::InvalidValidator)?;
			participant_keys.push(pub_key);
		}

		let curr_key = match phase_one_result.result {
			JobResult::DKGPhaseOne(info) => info.key.clone(),
			_ => return Err(Error::<T>::InvalidJobPhase.into()),
		};

		let new_phase_one_job_id = match job_info.job_type {
			JobType::DKGTSSPhaseFour(ref info) => info.new_phase_one_id,
			_ => return Err(Error::<T>::InvalidJobPhase.into()),
		};

		let new_phase_one_job_info =
			KnownResults::<T>::get(job_info.job_type.get_role_type(), new_phase_one_job_id)
				.ok_or(Error::<T>::JobNotFound)?;

		let new_key = match new_phase_one_job_info.result {
			JobResult::DKGPhaseOne(info) => info.key.clone(),
			_ => return Err(Error::<T>::InvalidJobPhase.into()),
		};
		let job_result = JobResult::DKGPhaseFour(DKGTSSKeyRotationResultOf::<T> {
			phase_one_id: job_info
				.job_type
				.get_phase_one_id()
				.ok_or(Error::<T>::InvalidJobPhase)?,
			new_phase_one_id: new_phase_one_job_id,
			new_key,
			key: curr_key,
			signature: info.signature.clone(),
			signature_scheme: info.signature_scheme.clone(),
			derivation_path: None,
		});

		T::MPCHandler::verify(JobWithResult {
			job_type: job_info.job_type.clone(),
			phase_one_job_type: Some(phase_one_result.job_type),
			result: job_result.clone(),
		})?;

		let result = PhaseResult {
			owner: job_info.owner.clone(),
			job_type: job_info.job_type.clone(),
			ttl: job_info.ttl,
			permitted_caller: job_info.job_type.clone().get_permitted_caller(),
			result: job_result,
		};
		Ok(result)
	}

	pub fn verify_zksaas_circuit_job_result(
		role_type: RoleType,
		job_id: JobId,
		job_info: &JobInfoOf<T>,
		_info: ZkSaaSCircuitResultOf<T>,
	) -> Result<PhaseResultOf<T>, DispatchError> {
		// sanity check, does job and result type match
		ensure!(role_type.is_zksaas(), Error::<T>::ResultNotExpectedType);
		// ensure the participants are the expected participants from job

		let participants = job_info
			.job_type
			.clone()
			.get_participants()
			.ok_or(Error::<T>::InvalidJobParams)?;
		let mut participant_keys: BoundedVec<
			sp_core::ecdsa::Public,
			<T as Config>::MaxParticipants,
		> = Default::default();

		for participant in participants.clone() {
			let key = T::RolesHandler::get_validator_role_key(participant);
			ensure!(key.is_some(), Error::<T>::ValidatorRoleKeyNotFound);
			let pub_key = sp_core::ecdsa::Public::from_slice(&key.expect("checked above")[0..33])
				.map_err(|_| Error::<T>::InvalidValidator)?;
			participant_keys
				.try_push(pub_key)
				.map_err(|_| Error::<T>::TooManyParticipants)?;
		}

		let job_result = JobResult::ZkSaaSPhaseOne(ZkSaaSCircuitResultOf::<T> {
			job_id,
			participants: participant_keys,
		});

		T::MPCHandler::verify(JobWithResult {
			job_type: job_info.job_type.clone(),
			phase_one_job_type: None,
			result: job_result.clone(),
		})?;

		let result = PhaseResult {
			owner: job_info.owner.clone(),
			ttl: job_info.ttl,
			job_type: job_info.job_type.clone(),
			result: job_result,
			permitted_caller: job_info.job_type.clone().get_permitted_caller(),
		};
		Ok(result)
	}

	pub fn verify_zksaas_prove_job_result(
		role_type: RoleType,
		job_info: &JobInfoOf<T>,
		info: ZkSaaSProofResultOf<T>,
	) -> Result<PhaseResultOf<T>, DispatchError> {
		let now = <frame_system::Pallet<T>>::block_number();
		// sanity check, does job and result type match
		ensure!(role_type.is_zksaas(), Error::<T>::ResultNotExpectedType);
		ensure!(role_type == job_info.job_type.get_role_type(), Error::<T>::ResultNotExpectedType);

		let existing_result_id = job_info
			.job_type
			.clone()
			.get_phase_one_id()
			.ok_or(Error::<T>::InvalidJobPhase)?;
		// Ensure the result exists
		let phase_one_result =
			KnownResults::<T>::get(job_info.job_type.get_role_type(), existing_result_id)
				.ok_or(Error::<T>::PreviousResultNotFound)?;

		// Validate existing result
		ensure!(phase_one_result.ttl >= now, Error::<T>::ResultExpired);

		let job_result = JobResult::ZkSaaSPhaseTwo(info.clone());

		T::MPCHandler::verify(JobWithResult {
			job_type: job_info.job_type.clone(),
			phase_one_job_type: Some(phase_one_result.job_type),
			result: job_result.clone(),
		})?;

		let result = PhaseResult {
			owner: job_info.owner.clone(),
			ttl: job_info.ttl,
			job_type: job_info.job_type.clone(),
			result: job_result,
			permitted_caller: job_info.job_type.clone().get_permitted_caller(),
		};
		Ok(result)
	}
}
