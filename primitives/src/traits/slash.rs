// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
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

use crate::types::{BlueprintId, InstanceId};
use sp_runtime::Percent;

/// Trait for managing slashing in the Tangle network.
/// This trait provides functionality to slash operators and delegators.
pub trait SlashManager<AccountId> {
	type Error;

	/// Slash a delegator's stake for an offense.
	///
	/// # Parameters
	/// * `delegator` - The account of the delegator being slashed
	/// * `offending_operator` - The operator account associated with the offense
	/// * `blueprint_id` - The blueprint ID where the offense occurred
	/// * `service_id` - The service instance ID where the offense occurred
	/// * `percentage` - The percentage of stake to slash
	fn slash_delegator(
		delegator: &AccountId,
		offending_operator: &AccountId,
		blueprint_id: BlueprintId,
		service_id: InstanceId,
		percentage: Percent,
	) -> Result<(), Self::Error>;

	/// Slash an operator's stake for an offense.
	///
	/// # Parameters
	/// * `offending_operator` - The operator account being slashed
	/// * `blueprint_id` - The blueprint ID where the offense occurred
	/// * `service_id` - The service instance ID where the offense occurred
	/// * `percentage` - The percentage of stake to slash
	fn slash_operator(
		offending_operator: &AccountId,
		blueprint_id: BlueprintId,
		service_id: InstanceId,
		percentage: Percent,
	) -> Result<(), Self::Error>;
}

impl<AccountId> SlashManager<AccountId> for () {
	type Error = &'static str;

	fn slash_delegator(
		_delegator: &AccountId,
		_offending_operator: &AccountId,
		_blueprint_id: BlueprintId,
		_service_id: InstanceId,
		_percentage: Percent,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn slash_operator(
		_offending_operator: &AccountId,
		_blueprint_id: BlueprintId,
		_service_id: InstanceId,
		_percentage: Percent,
	) -> Result<(), Self::Error> {
		Ok(())
	}
}
