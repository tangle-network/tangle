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

use crate::offences::ValidatorOffence;

use super::*;
use frame_support::pallet_prelude::DispatchResult;
use sp_runtime::{traits::Convert, Percent};
use tangle_primitives::{
	jobs::{JobKey, ReportValidatorOffence},
	traits::roles::RolesHandler,
};

/// Implements RolesHandler for the pallet.
impl<T: Config> RolesHandler<T::AccountId> for Pallet<T> {
	/// Validates if the given address has the given role.
	///
	/// # Parameters
	/// - `address`: The account ID of the validator.
	/// - `job`: The key representing the type of job.
	///
	/// # Returns
	/// Returns `true` if the validator is permitted to work with this job type, otherwise `false`.
	fn is_validator(address: T::AccountId, job_key: JobKey) -> bool {
		let assigned_roles = AccountRolesMapping::<T>::get(address);
		let job_role = job_key.get_role_type();
		assigned_roles.contains(&job_role)
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
		offence_report: ReportValidatorOffence<T::AccountId>,
	) -> sp_runtime::DispatchResult {
		Self::report_offence(offence_report)
	}

	fn get_validator_metadata(address: T::AccountId, job_key: JobKey) -> Option<RoleTypeMetadata> {
		if Self::is_validator(address.clone(), job_key.clone()) {
			let ledger = Self::ledger(&address);
			if let Some(ledger) = ledger {
				return match ledger.roles.get(&job_key.get_role_type()) {
					Some(stake) => Some(stake.metadata.clone()),
					None => None,
				}
			} else {
				return None
			}
		} else {
			return None
		}
	}
}

/// Functions for the pallet.
impl<T: Config> Pallet<T> {
	/// Add new role to the given account.
	///
	/// # Parameters
	/// - `account`: The account ID of the validator.
	/// - `role`: Selected role type.
	pub(crate) fn add_role(account: T::AccountId, role: RoleType) -> DispatchResult {
		AccountRolesMapping::<T>::try_mutate(&account, |roles| {
			if !roles.contains(&role) {
				roles.try_push(role.clone()).map_err(|_| Error::<T>::MaxRoles)?;

				Ok(())
			} else {
				Err(Error::<T>::HasRoleAssigned.into())
			}
		})
	}

	/// Remove role from the given account.
	///
	/// # Parameters
	/// - `account`: The account ID of the validator.
	/// - `role`: Selected role type.
	pub(crate) fn remove_role(account: T::AccountId, role: RoleType) -> DispatchResult {
		AccountRolesMapping::<T>::try_mutate(&account, |roles| {
			if roles.contains(&role) {
				roles.retain(|r| r != &role);

				Ok(())
			} else {
				Err(Error::<T>::NoRoleAssigned.into())
			}
		})
	}

	/// Check if the given account has the given role.
	///
	/// # Parameters
	/// - `account`: The account ID of the validator.
	/// - `role`: Selected role type.
	///
	/// # Returns
	/// Returns `true` if the validator is permitted to work with this job type, otherwise `false`.
	pub(crate) fn has_role(account: T::AccountId, role: RoleType) -> bool {
		let assigned_roles = AccountRolesMapping::<T>::get(account);
		match assigned_roles.iter().find(|r| **r == role) {
			Some(_) => true,
			None => false,
		}
	}

	/// Check if account can chill, unbound and withdraw funds.
	///
	/// # Parameters
	/// - `account`: The account ID of the validator.
	///
	/// # Returns
	/// Returns boolean value.
	pub(crate) fn can_exit(account: T::AccountId) -> bool {
		let assigned_roles = AccountRolesMapping::<T>::get(account);
		if assigned_roles.is_empty() {
			// Role is cleared, account can chill, unbound and withdraw funds.
			return true
		}
		false
	}

	/// Calculate max re-stake amount for the given account.
	///
	/// # Parameters
	/// - `total_stake`: Total stake of the validator
	///
	/// # Returns
	/// Returns the max re-stake amount.
	pub(crate) fn calculate_max_re_stake_amount(total_stake: BalanceOf<T>) -> BalanceOf<T> {
		// User can re-stake max 50% of the total stake
		Percent::from_percent(50) * total_stake
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
	pub(crate) fn report_offence(
		offence_report: ReportValidatorOffence<T::AccountId>,
	) -> sp_runtime::DispatchResult {
		let offenders = offence_report
			.offenders
			.into_iter()
			.enumerate()
			.filter_map(|(_, id)| {
				// Get validator id from account id.
				let id = <<T as Config>::ValidatorSet as ValidatorSet<
					<T as frame_system::Config>::AccountId,
				>>::ValidatorIdOf::convert(id)?;
				<T::ValidatorSet as ValidatorSetWithIdentification<T::AccountId>>::IdentificationOf::convert(
					id.clone(),
				).map(|full_id| (id, full_id))
			})
			.collect::<Vec<IdentificationTuple<T>>>();

		let offence = ValidatorOffence {
			session_index: offence_report.session_index,
			validator_set_count: offence_report.validator_set_count,
			offenders,
			offence_type: offence_report.offence_type,
		};
		let _ = T::ReportOffences::report_offence(sp_std::vec![], offence.clone());
		Ok(())
	}

	/// Update the ledger for the given stash account.
	///
	/// # Parameters
	/// - `staker`: The stash account ID.
	/// - `ledger`: The new ledger.
	///
	/// # Note
	/// This function will set a lock on the stash account.
	pub(crate) fn update_ledger(staker: &T::AccountId, ledger: &RoleStakingLedger<T>) {
		<Ledger<T>>::insert(staker, ledger);
	}

	/// Kill the stash account and remove all related information.
	pub(crate) fn kill_stash(stash: &T::AccountId) {
		<Ledger<T>>::remove(&stash);
	}
}
