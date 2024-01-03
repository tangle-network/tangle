// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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
//! Runtime API definition for jobs pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::Codec;
use sp_runtime::{traits::MaybeDisplay, Serialize};
use sp_std::vec::Vec;
use tangle_primitives::{
	jobs::{JobId, RpcResponseJobsData, RpcResponsePhaseOneResult},
	roles::RoleType,
};

sp_api::decl_runtime_apis! {
	pub trait JobsApi<AccountId> where
		AccountId: Codec + MaybeDisplay + Serialize,
	{
		/// Query jobs associated with a specific validator.
		///
		/// This function takes a `validator` parameter of type `AccountId` and attempts
		/// to retrieve a list of jobs associated with the provided validator. If successful,
		/// it constructs a vector of `RpcResponseJobsData<AccountId>` containing information
		/// about the jobs and returns it as a `Result`.
		///
		/// # Arguments
		///
		/// * `validator` - The account ID of the validator whose jobs are to be queried.
		///
		/// # Returns
		///
		/// An optional vec of `RpcResponseJobsData` of jobs assigned to validator
		fn query_jobs_by_validator(validator: AccountId) -> Option<Vec<RpcResponseJobsData<AccountId>>>;
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
		fn query_job_by_id(role_type: RoleType, job_id: JobId) -> Option<RpcResponseJobsData<AccountId>>;

		/// Queries the phase one result of a job by its key and ID.
		///
		/// # Arguments
		///
		/// * `role_type` - The role of the job.
		/// * `job_id` - The ID of the job.
		///
		/// # Returns
		///
		/// An `Option` containing the phase one result of the job, wrapped in an `RpcResponsePhaseOneResult`.
		fn query_phase_one_by_id(role_type: RoleType, job_id: JobId) -> Option<RpcResponsePhaseOneResult<AccountId>>;

		/// Queries next job ID.
		///
		///  # Returns
		///  Next job ID.
		fn query_next_job_id() -> JobId;
	}
}
