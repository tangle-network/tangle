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

use crate::jobs::ReportRestakerOffence;
use sp_runtime::DispatchResult;
use sp_std::vec::Vec;

use super::RoleType;

/// A trait that handles roles associated with job types.
pub trait RolesHandler<AccountId> {
	type Balance;

	/// Returns true if the validator is permitted to work with this job type.
	///
	/// # Parameters
	///
	/// - `address`: The account ID of the validator.
	/// - `role_type`: The type of role.
	///
	/// # Returns
	///
	/// Returns `true` if the validator is permitted to work with this job type, otherwise `false`.
	fn is_restaker(address: AccountId, role_type: RoleType) -> bool;

	/// Report offence for the given validator.
	/// This function will report validators for committing offence.
	///
	/// # Parameters
	/// - `offence_report`: The offence report.
	///
	/// # Returns
	///
	/// Returns Ok() if validator offence report is submitted successfully.
	fn report_offence(offence_report: ReportRestakerOffence<AccountId>) -> DispatchResult;

	/// Retrieves role key associated with given validator
	///
	/// # Arguments
	///
	/// * `address` - The account ID of the validator for which role key is to be retrieved.
	///
	/// # Returns
	///
	/// Returns an `Option<Vec<u8>>` containing role key information for the specified
	/// validator, or `None` if no role key is found.
	fn get_validator_role_key(address: AccountId) -> Option<Vec<u8>>;

	/// Record rewards to a validator.
	///
	/// This function records a job completed by the given validators
	///
	/// # Parameters
	///
	/// - `validators`: The account ID of the validators.
	///
	/// # Errors
	///
	/// Returns a `DispatchError` if the operation fails.
	fn record_job_by_validators(validators: Vec<AccountId>) -> DispatchResult;
}
