// This file is part of Webb.
// Copyright (C) 2022 Webb Technologies Inc.
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

use crate::jobs::{JobSubmission, JobInfo, PhaseOneResult, JobId};
use sp_arithmetic::{
	traits::{BaseArithmetic, SaturatedConversion, Unsigned},
	Perbill,
};
use sp_runtime::DispatchResult;

/// A trait that describes the job to fee calculation.
pub trait JobToFee<AccountId, BlockNumber> {
    /// The type that is returned as result from calculation.
    type Balance: BaseArithmetic + From<u32> + Copy + Unsigned;

    /// Calculates the fee from the passed `job`.
    ///
    /// # Parameters
    ///
    /// - `job`: A reference to the job submission information containing `AccountId` and `BlockNumber`.
    ///
    /// # Returns
    ///
    /// Returns the calculated fee as `Self::Balance`.
    fn job_to_fee(job: &JobSubmission<AccountId, BlockNumber>) -> Self::Balance;
}

/// A trait that describes the job result verification.
pub trait JobResultVerifier<AccountId, BlockNumber, Balance> {
    /// Verifies the result of a job.
    ///
    /// # Parameters
    ///
    /// - `job`: A reference to the job information containing `AccountId`, `BlockNumber`, and `Balance`.
    /// - `phase_one_data`: An optional result of phase one of the job, if applicable.
    /// - `result`: The result data of the job as a vector of bytes.
    ///
    /// # Errors
    ///
    /// Returns a `DispatchResult` indicating success or an error if verification fails.
    fn verify(
        job: &JobInfo<AccountId, BlockNumber, Balance>,
        phase_one_data: Option<PhaseOneResult<AccountId, BlockNumber>>,
        result: Vec<u8>,
    ) -> DispatchResult;
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
    fn get_active_jobs(validator: AccountId) -> Vec<JobId>;

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
    fn exit_from_known_set(validator: AccountId, job_id: JobId) -> DispatchResult;
}