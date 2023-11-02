use super::*;
use tangle_primitives::traits::jobs::JobsHandler;

/// A trait that handles various aspects of jobs for a validator.
impl<T: Config> JobsHandler<T::AccountId> for Pallet<T> {
	/// Returns a list of active jobs associated with a validator.
	///
	/// # Parameters
	///
	/// - `validator`: The account ID of the validator.
	///
	/// # Returns
	///
	/// Returns a vector of `JobId` representing the active jobs of the validator.
	fn get_active_jobs(validator: T::AccountId) -> Vec<JobId> {
		if let Some(jobs) = ValidatorJobIdLookup::<T>::get(validator) {
			return jobs
		}
		Default::default()
	}

	/// Exits a job from the known set of a validator.
	///
	/// # Parameters
	///
	/// - `validator`: The account ID of the validator.
	/// - `job_id`: The ID of the job to exit from the known set.
	///
	/// # Errors
	///
	/// Returns a `DispatchResult` indicating success or an error if the operation fails.
	fn exit_from_known_set(validator: T::AccountId, job_id: JobId) -> DispatchResult {
		todo!()
	}
}
