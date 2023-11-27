// This file is part of Tangle.
// Copyright (C) 2022-2023 Webb Technologies Inc.
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

use crate::jobs::{JobId, JobKey, JobResult, JobSubmission, ValidatorOffence};
use frame_support::dispatch::Vec;
use sp_arithmetic::traits::{BaseArithmetic, Unsigned};
use sp_runtime::DispatchResult;

/// A trait that describes the job to fee calculation.
pub trait JobToFee<AccountId, BlockNumber> {
	/// The type that is returned as result from calculation.
	type Balance: BaseArithmetic + From<u32> + Copy + Unsigned;

	/// Calculates the fee from the passed `job`.
	///
	/// # Parameters
	///
	/// - `job`: A reference to the job submission information containing `AccountId` and
	///   `BlockNumber`.
	///
	/// # Returns
	///
	/// Returns the calculated fee as `Self::Balance`.
	fn job_to_fee(job: &JobSubmission<AccountId, BlockNumber>) -> Self::Balance;
}

/// A trait that describes the job result verification.
pub trait MPCHandler<AccountId, BlockNumber, Balance> {
	/// Verifies the result of a job.
	///
	/// # Parameters
	///
	/// - `data`: Details of the job to verify
	///
	/// # Errors
	///
	/// Returns a `DispatchResult` indicating success or an error if verification fails.
	fn verify(data: JobResult) -> DispatchResult;

	// Verify a validator report
	///
	/// This function is responsible for verifying a report against a specific validator's
	/// offence and taking appropriate actions based on the report.
	///
	/// # Arguments
	///
	/// - `validator`: The account ID of the validator being reported.
	/// - `offence`: Details of the offence reported against the validator.
	/// - `report`: The report data provided by the reporting entity.
	fn verify_validator_report(
		validator: AccountId,
		offence: ValidatorOffence,
		signatures: Vec<Vec<u8>>,
	) -> DispatchResult;

	/// Validate the authority key associated with a specific validator.
	///
	/// This function is responsible for validating the authority key associated with a given
	/// validator.
	///
	/// # Arguments
	///
	/// - `validator`: The account ID of the validator whose authority key is to be validated.
	/// - `authority_key`: The authority key to be validated.
	fn validate_authority_key(validator: AccountId, authority_key: Vec<u8>) -> DispatchResult;
}

/// A trait that handles various aspects of jobs for a validator.
pub trait JobsHandler<AccountId> {
	/// Returns a list of active jobs associated with a validator.
	///
	/// # Parameters
	///
	/// - `validator`: The account ID of the validator.
	///
	/// # Returns
	///
	/// Returns a vector of `JobId` representing the active jobs of the validator.
	fn get_active_jobs(validator: AccountId) -> Vec<(JobKey, JobId)>;

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
	fn exit_from_known_set(validator: AccountId, job_key: JobKey, job_id: JobId) -> DispatchResult;
}
