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

use super::*;

use frame_support::{pallet_prelude::DispatchResult, traits::OneSessionHandler};

use tangle_primitives::{jobs::ReportRestakerOffence, roles::traits::RolesHandler};

/// Implements RolesHandler for the pallet.
impl<T: Config> RolesHandler<T::AccountId> for Pallet<T> {
	type Balance = BalanceOf<T>;

	/// Validates if the given address has the given role.
	///
	/// # Parameters
	/// - `address`: The account ID of the validator.
	/// - `job`: The key representing the type of job.
	///
	/// # Returns
	/// Returns `true` if the validator is permitted to work with this job type, otherwise `false`.
	fn is_restaker(address: T::AccountId, role_type: RoleType) -> bool {
		let assigned_roles = AccountRolesMapping::<T>::get(address);
		assigned_roles.contains(&role_type)
	}

	/// Report offence for the given validator.
	/// This function will report validators for committing offence.
	///
	/// # Parameters
	/// - `offence_report`: The offence report.
	///
	/// # Returns
	///
	/// Returns Ok() if validator offence report is submitted successfully.
	fn report_offence(
		offence_report: ReportRestakerOffence<T::AccountId>,
	) -> sp_runtime::DispatchResult {
		Self::report_offence(offence_report)
	}

	/// Retrieves the role key associated with the given validator address.
	///
	/// Returns `Some(Vec<u8>)` containing the role key if the address has an associated ledger
	/// entry; otherwise, returns `None`.
	///
	/// # Arguments
	///
	/// * `address` - The account identifier of the validator whose role key is to be retrieved.
	fn get_validator_role_key(address: T::AccountId) -> Option<Vec<u8>> {
		let maybe_ledger = Self::ledger(&address);
		if let Some(ledger) = maybe_ledger {
			Some(ledger.role_key.to_vec())
		} else {
			return None
		}
	}

	/// Record rewards to a validator.
	///
	/// This function records the rewards earned by a validator account.
	///
	/// # Parameters
	///
	/// - `validators`: The account ID of the validators.
	/// - `reward_per_validator`: The amount of rewards to record per validator, all validators are
	///   rewarded equally for a job
	///
	/// # Errors
	///
	/// Returns a `DispatchError` if the operation fails.
	fn record_job_by_validators(validators: Vec<T::AccountId>) -> DispatchResult {
		let mut validator_job_map = ValidatorJobsInEra::<T>::get();
		for validator in validators {
			let current_job_count: u32 =
				*validator_job_map.get(&validator).unwrap_or(&Default::default());
			let new_job_count = current_job_count.saturating_add(1u32.into());
			let _ = validator_job_map.try_insert(validator, new_job_count);
		}

		ValidatorJobsInEra::<T>::put(validator_job_map);

		Ok(())
	}

	fn get_max_active_service_for_restaker(restaker: T::AccountId) -> Option<u32> {
		let maybe_ledger = Self::ledger(&restaker);
		if let Some(ledger) = maybe_ledger {
			Some(ledger.max_active_services)
		} else {
			return None
		}
	}
}

impl<T: Config> sp_runtime::BoundToRuntimeAppPublic for Pallet<T> {
	type Public = T::RoleKeyId;
}

impl<T: Config> OneSessionHandler<T::AccountId> for Pallet<T> {
	type Key = T::RoleKeyId;

	fn on_genesis_session<'a, I: 'a>(validators: I)
	where
		I: Iterator<Item = (&'a T::AccountId, T::RoleKeyId)>,
	{
		validators
			.into_iter()
			.filter(|(acc, _)| Ledger::<T>::contains_key(acc))
			.for_each(|(acc, role_key)| {
				match Self::update_ledger_role_key(acc, role_key.encode()) {
					Ok(_) => (),
					Err(e) => log::error!("Error updating ledger role key: {:?}", e),
				}
			});
	}

	fn on_new_session<'a, I: 'a>(_changed: bool, validators: I, _queued_validators: I)
	where
		I: Iterator<Item = (&'a T::AccountId, T::RoleKeyId)>,
	{
		validators
			.into_iter()
			.filter(|(acc, _)| Ledger::<T>::contains_key(acc))
			.for_each(|(acc, role_key)| {
				match Self::update_ledger_role_key(acc, role_key.encode()) {
					Ok(_) => (),
					Err(e) => log::error!("Error updating ledger role key: {:?}", e),
				}
			});
	}

	fn on_disabled(_i: u32) {
		// ignore
	}

	// Distribute the inflation rewards
	fn on_before_session_ending() {
		// ignore
	}
}
