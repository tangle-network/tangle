use super::*;

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
	pub(crate) fn add_job_id_to_validator_lookup(
		validator: T::AccountId,
		job_id: JobId,
	) -> DispatchResult {
		ValidatorJobIdLookup::<T>::try_mutate(validator, |job_ids| -> DispatchResult {
			let job_ids = job_ids.get_or_insert_with(Default::default);
			job_ids.push(job_id);
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
}
