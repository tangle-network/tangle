use super::*;
pub use tangle_primitives::jobs::RpcResponseJobsData;
use tangle_primitives::{jobs::PhaseResult, roles::RoleType};

impl<T: Config> Pallet<T> {
	/// Queries jobs associated with a specific validator.
	///
	/// This function takes a `validator` parameter of type `T::AccountId` and attempts
	/// to retrieve a list of jobs associated with the provided validator. If successful,
	/// it constructs a vector of `RpcResponseJobsData` containing information
	/// about the jobs and returns it as a `Result`.
	///
	/// # Arguments
	///
	/// * `validator` - The account ID of the validator whose jobs are to be queried.
	///
	/// # Returns
	///
	/// Returns a `Result` containing a vector of `RpcResponseJobsData<T::AccountId>` if the
	/// operation is successful, or an error message as a `String` if there is an issue.
	pub fn query_jobs_by_validator(
		validator: T::AccountId,
	) -> Option<Vec<RpcResponseJobsData<T::AccountId, BlockNumberFor<T>>>> {
		let mut jobs: Vec<RpcResponseJobsData<_, _>> = vec![];
		if let Some(jobs_list) = ValidatorJobIdLookup::<T>::get(validator) {
			for (role_type, job_id) in jobs_list.iter() {
				if let Some(job_info) = SubmittedJobs::<T>::get(role_type, job_id) {
					let info = RpcResponseJobsData::<T::AccountId, BlockNumberFor<T>> {
						job_id: *job_id,
						job_type: job_info.job_type,
						ttl: job_info.ttl,
						expiry: job_info.expiry,
					};
					jobs.push(info);
				} else {
					continue
				}
			}
		}

		Some(jobs)
	}

	/// Queries a job by its key and ID.
	///
	/// # Arguments
	///
	/// * `role_type` - The role of the job.
	/// * `job_id` - The ID of the job.
	///
	/// # Returns
	///
	/// An optional `RpcResponseJobsData` containing the account ID of the job.
	pub fn query_job_by_id(
		role_type: RoleType,
		job_id: JobId,
	) -> Option<RpcResponseJobsData<T::AccountId, BlockNumberFor<T>>> {
		SubmittedJobs::<T>::get(role_type, job_id).map(|job_info| RpcResponseJobsData {
			job_id,
			job_type: job_info.job_type,
			ttl: job_info.ttl,
			expiry: job_info.expiry,
		})
	}

	/// Queries the phase one result of a job by its key and ID.
	///
	/// # Arguments
	///
	/// * `role_type` - The role of the job.
	/// * `job_id` - The ID of the job.
	///
	/// # Returns
	///
	/// An `Option` containing the phase one result of the job, wrapped in an
	/// `PhaseResult`.
	pub fn query_job_result(
		role_type: RoleType,
		job_id: JobId,
	) -> Option<PhaseResult<T::AccountId, BlockNumberFor<T>>> {
		KnownResults::<T>::get(role_type, job_id)
	}

	/// Queries next job ID.
	///
	///  # Returns
	///  Next job ID.
	pub fn query_next_job_id() -> JobId {
		NextJobId::<T>::get()
	}
}
