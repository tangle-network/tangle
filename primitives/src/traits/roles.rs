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

use crate::jobs::JobKey;

/// A trait that handles roles associated with job types.
pub trait RolesHandler<AccountId> {
	/// Returns true if the validator is permitted to work with this job type.
	///
	/// # Parameters
	///
	/// - `address`: The account ID of the validator.
	/// - `job_key`: The key representing the type of job.
	///
	/// # Returns
	///
	/// Returns `true` if the validator is permitted to work with this job type, otherwise `false`.
	fn is_validator(address: AccountId, job_key: JobKey) -> bool;
}
