use super::*;
use scale_info::prelude::string::String;
pub use tangle_primitives::jobs::RpcResponseJobsData;

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
	) -> Result<Vec<RpcResponseJobsData<T::AccountId>>, String> {
		if let Some(jobs_list) = ValidatorJobIdLookup::<T>::get(validator) {
			let mut jobs: Vec<RpcResponseJobsData<T::AccountId>> = vec![];

			for (job_key, job_id) in jobs_list.iter() {
				if let Some(job_info) = SubmittedJobs::<T>::get(job_key.clone(), job_id) {
					if !job_info.job_type.is_phase_one() {
						let result = KnownResults::<T>::get(
							job_info.job_type.get_previous_phase_job_key().unwrap(),
							job_info.job_type.clone().get_phase_one_id().unwrap(),
						)
						.unwrap();

						let info = RpcResponseJobsData::<T::AccountId> {
							job_id: *job_id,
							job_type: job_info.job_type,
							participants: Some(result.participants),
							threshold: result.threshold,
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
				}
			}
		}
		Ok(vec![])
	}
}
