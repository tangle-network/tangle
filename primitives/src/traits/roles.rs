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

use crate::{
	jobs::{JobKey, ReportValidatorOffence},
	roles::RoleTypeMetadata,
};
use sp_runtime::DispatchResult;

/// A trait that handles roles associated with job types.
pub trait RolesHandler<AccountId> {
	/// Returns true if the validator is permitted to work with this job type.
	///
	/// # Parameters
	///
	/// - `address`: The account ID of the validator.
	/// - `job_key`: The type of job
	///
	/// # Returns
	///
	/// Returns `true` if the validator is permitted to work with this job type, otherwise `false`.
	fn is_validator(address: AccountId, job_key: JobKey) -> bool;

	/// Report offence for the given validator.
	/// This function will report validators for committing offence.
	///
	/// # Parameters
	/// - `offence_report`: The offence report.
	///
	/// # Returns
	///
	/// Returns Ok() if validator offence report is submitted successfully.
	fn report_offence(offence_report: ReportValidatorOffence<AccountId>) -> DispatchResult;

	/// Retrieves metadata information for a validator associated with a specific job key.
	///
	/// # Arguments
	///
	/// * `address` - The account ID of the validator for which metadata is to be retrieved.
	/// * `job_key` - The unique identifier for the job associated with the validator.
	///
	/// # Returns
	///
	/// Returns an `Option<RoleTypeMetadata>` containing metadata information for the specified
	/// validator, or `None` if no metadata is found.
	fn get_validator_metadata(address: AccountId, job_key: JobKey) -> Option<RoleTypeMetadata>;
}
