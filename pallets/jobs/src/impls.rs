use super::*;
use tangle_primitives::{jobs::traits::JobsHandler, roles::RoleType};

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
	fn get_active_jobs(validator: T::AccountId) -> Vec<(RoleType, JobId)> {
		if let Some(jobs) = ValidatorJobIdLookup::<T>::get(validator) {
			return jobs.to_vec()
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
	fn exit_from_known_set(
		validator: T::AccountId,
		role_type: RoleType,
		job_id: JobId,
	) -> DispatchResult {
		Self::try_validator_removal_from_job(role_type, job_id, validator)
	}
}
