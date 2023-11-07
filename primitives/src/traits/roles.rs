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

use crate::jobs::{ValidatorOffence};
use crate::roles::RoleType;
use sp_runtime::DispatchResult;

/// A trait that handles roles associated with job types.
pub trait RolesHandler<AccountId> {
	/// Returns true if the validator is permitted to work with this job type.
	///
	/// # Parameters
	///
	/// - `address`: The account ID of the validator.
	/// - `role_type`: The type of role
	///
	/// # Returns
	///
	/// Returns `true` if the validator is permitted to work with this job type, otherwise `false`.
	fn is_validator(address: AccountId, role_type: RoleType) -> bool;

	/// Slash validator stake for the reported offence. The function should be a best effort
	/// slashing, slash upto max possible by the offence type.
	///
	/// # Parameters
	///
	/// - `address`: The account ID of the validator.
	/// - `offence`: The offence reported against the validator
	///
	/// # Returns
	///
	/// Returns Ok() if the address is a validator and was slashed
	fn slash_validator(address: AccountId, offence: ValidatorOffence) -> DispatchResult;
}
