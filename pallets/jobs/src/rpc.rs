use super::*;
pub use tangle_primitives::jobs::RpcResponseJobsData;
use tangle_primitives::{jobs::RpcResponsePhaseOneResult, roles::RoleType};

impl<T: Config> Pallet<T> {
	/// Queries jobs associated with a specific validator.
	///
	/// This function takes a `validator` parameter of type `T::AccountId` and attempts
	/// to retrieve a list of jobs associated with the provided validator. If successful,
	/// it constructs a vector of `RpcResponseJobsData<T::AccountId>` containing information
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
	) -> Option<Vec<RpcResponseJobsData<T::AccountId>>> {
		let mut jobs: Vec<RpcResponseJobsData<T::AccountId>> = vec![];
		if let Some(jobs_list) = ValidatorJobIdLookup::<T>::get(validator) {
			for (role_type, job_id) in jobs_list.iter() {
				if let Some(job_info) = SubmittedJobs::<T>::get(role_type, job_id) {
					if !job_info.job_type.is_phase_one() {
						let result = KnownResults::<T>::get(
							job_info.job_type.get_role_type(),
							job_info.job_type.clone().get_phase_one_id().unwrap(),
						)
						.unwrap();

						let info = RpcResponseJobsData::<T::AccountId> {
							job_id: *job_id,
							job_type: job_info.job_type,
							participants: result.participants(),
							threshold: result.threshold(),
							key: Some(result.result),
						};

						jobs.push(info);
					} else {
						let info = RpcResponseJobsData::<T::AccountId> {
							job_id: *job_id,
							job_type: job_info.job_type,
							participants: None,
							threshold: None,
							key: None,
						};

						jobs.push(info);
					}
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
	) -> Option<RpcResponseJobsData<T::AccountId>> {
		if let Some(job_info) = SubmittedJobs::<T>::get(role_type, job_id) {
			if !job_info.job_type.is_phase_one() {
				let result = KnownResults::<T>::get(
					job_info.job_type.get_role_type(),
					job_info.job_type.clone().get_phase_one_id().unwrap(),
				)
				.unwrap();

				let info = RpcResponseJobsData::<T::AccountId> {
					job_id,
					job_type: job_info.job_type,
					participants: result.participants(),
					threshold: result.threshold(),
					key: Some(result.result),
				};

				return Some(info)
			} else {
				let info = RpcResponseJobsData::<T::AccountId> {
					job_id,
					job_type: job_info.job_type,
					participants: None,
					threshold: None,
					key: None,
				};

				return Some(info)
			}
		}

		None
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
	/// `RpcResponsePhaseOneResult`.
	pub fn query_phase_one_by_id(
		role_type: RoleType,
		job_id: JobId,
	) -> Option<RpcResponsePhaseOneResult<T::AccountId>> {
		if let Some(job_info) = SubmittedJobs::<T>::get(role_type, job_id) {
			if !job_info.job_type.is_phase_one() {
				let result = KnownResults::<T>::get(
					job_info.job_type.get_role_type(),
					job_info.job_type.clone().get_phase_one_id().unwrap(),
				)
				.unwrap();

				let info = RpcResponsePhaseOneResult::<T::AccountId> {
					owner: job_info.owner,
					result: result.result,
					job_type: job_info.job_type,
				};

				return Some(info)
			}
		}

		None
	}

	/// Queries next job ID.
	///
	///  # Returns
	///  Next job ID.
	pub fn query_next_job_id() -> JobId {
		NextJobId::<T>::get()
	}
}
