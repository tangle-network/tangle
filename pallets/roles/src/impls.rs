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

use crate::offences::ValidatorOffence;

use super::*;
use frame_support::{
	pallet_prelude::DispatchResult,
	traits::{Currency, OneSessionHandler},
};
use sp_runtime::{
	traits::{CheckedDiv, Convert},
	Perbill, Percent,
};

use sp_staking::offence::Offence;
use tangle_primitives::{
	jobs::{traits::JobsHandler, JobId, ReportValidatorOffence},
	roles::traits::RolesHandler,
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
	fn is_validator(address: T::AccountId, role_type: RoleType) -> bool {
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
		offence_report: ReportValidatorOffence<T::AccountId>,
	) -> sp_runtime::DispatchResult {
		Self::report_offence(offence_report)
	}

	fn get_validator_role_key(address: T::AccountId) -> Option<Vec<u8>> {
		let maybe_ledger = Self::ledger(&address);
		if let Some(ledger) = maybe_ledger {
			Some(ledger.role_key.to_vec())
		} else {
			return None
		}
	}
}

/// Functions for the pallet.
impl<T: Config> Pallet<T> {
	/// Validate updated profile for the given account.
	/// This function will validate the updated profile for the given account by
	/// checking if the account has any active jobs for the removed roles. If the
	/// account has any active jobs for the removed roles, then it will return
	/// the error `RoleCannotBeRemoved`.
	///
	/// # Parameters
	/// - `account`: The account ID of the validator.
	/// - `updated_profile`: The updated profile.
	pub(crate) fn validate_updated_profile(
		account: T::AccountId,
		updated_profile: Profile<T>,
	) -> DispatchResult {
		let current_ledger = Self::ledger(&account).ok_or(Error::<T>::NoProfileFound)?;
		let active_jobs: Vec<(RoleType, JobId)> = T::JobsHandler::get_active_jobs(account.clone());
		// Check if the account has any active jobs for the removed roles.
		let removed_roles = current_ledger.profile.get_removed_roles(&updated_profile);
		if !removed_roles.is_empty() {
			for role in removed_roles {
				for job in active_jobs.clone() {
					let role_type = job.0;
					if role_type == role {
						return Err(Error::<T>::RoleCannotBeRemoved.into())
					}
				}
			}
		};

		// Get all roles for which there are active jobs
		let roles_with_active_jobs: Vec<RoleType> =
			active_jobs.iter().map(|job| job.0).fold(Vec::new(), |mut acc, role| {
				if !acc.contains(&role) {
					acc.push(role);
				}
				acc
			});

		// If there are active jobs, changing a current independent profile to shared profile
		// is allowed if and only if the shared restaking amount is at least as much as the
		// sum of the restaking amounts of the current profile. This is because we require
		// the total amount staked to only increase or remain the same across active roles.
		if updated_profile.is_shared() && current_ledger.profile.is_independent() {
			if active_jobs.len() > 0 {
				let mut active_role_restaking_sum = Zero::zero();
				for role in roles_with_active_jobs.iter() {
					let current_role_restaking_amount = current_ledger
						.profile
						.get_records()
						.iter()
						.find_map(|record| if record.role == *role { record.amount } else { None })
						.unwrap_or_else(|| Zero::zero());
					active_role_restaking_sum += current_role_restaking_amount;
				}

				ensure!(
					updated_profile.get_total_profile_restake() >= active_role_restaking_sum,
					Error::<T>::InsufficientRestakingBond
				);
			}
		}

		// Changing a current shared profile to an independent profile is allowed if there are
		// active jobs as long as the stake allocated to the active roles is at least as much as
		// the shared profile restaking amount. This is because the shared restaking profile for an
		// active role is entirely allocated to that role (as it is shared between all selected
		// roles). Thus, we allow the user to change to an independent profile as long as the
		// restaking amount for the active roles is at least as much as the shared restaking amount.
		if updated_profile.is_independent() && current_ledger.profile.is_shared() {
			// For each role with an active job, ensure its stake is greater than or equal to the
			// existing ledger's shared restaking amount.
			for role in roles_with_active_jobs.iter() {
				let updated_role_restaking_amount = updated_profile
					.get_records()
					.iter()
					.find_map(|record| if record.role == *role { record.amount } else { None })
					.unwrap_or_else(|| Zero::zero());
				ensure!(
					updated_role_restaking_amount >=
						current_ledger.profile.get_total_profile_restake(),
					Error::<T>::InsufficientRestakingBond
				);
			}

			return Ok(())
		}
		// For each role with an active job, ensure its stake is greater than or equal to the
		// existing ledger's restaking amount for that role. If it's a shared profile, then the
		// restaking amount for that role is the entire shared restaking amount.
		let min_restaking_bond = MinRestakingBond::<T>::get();
		for record in updated_profile.clone().get_records() {
			match updated_profile.clone() {
				Profile::Independent(_) =>
					if roles_with_active_jobs.contains(&record.role) {
						ensure!(
							record.amount.unwrap_or_default() >= min_restaking_bond,
							Error::<T>::InsufficientRestakingBond
						);
						ensure!(
							record.amount.unwrap_or_default() >=
								current_ledger.restake_for(&record.role),
							Error::<T>::InsufficientRestakingBond
						);
					},
				Profile::Shared(profile) =>
					if roles_with_active_jobs.contains(&record.role) {
						ensure!(
							profile.amount >= current_ledger.profile.get_total_profile_restake(),
							Error::<T>::InsufficientRestakingBond
						);
					},
			}
		}
		Ok(())
	}

	/// Check if account can chill, unbond and withdraw funds.
	///
	/// # Parameters
	/// - `account`: The account ID of the validator.
	///
	/// # Returns
	/// Returns boolean value.
	pub(crate) fn can_exit(account: T::AccountId) -> bool {
		let assigned_roles = AccountRolesMapping::<T>::get(account);
		if assigned_roles.is_empty() {
			// Role is cleared, account can chill, unbond and withdraw funds.
			return true
		}
		false
	}

	/// Calculate max restake amount for the given account.
	///
	/// # Parameters
	/// - `total_stake`: Total stake of the validator
	///
	/// # Returns
	/// Returns the max restake amount.
	pub(crate) fn calculate_max_restake_amount(total_stake: BalanceOf<T>) -> BalanceOf<T> {
		// User can restake max 50% of the total stake
		Percent::from_percent(50) * total_stake
	}
	/// Calculate slash value for restaked amount
	///
	/// # Parameters
	/// - `slash_fraction`: Slash fraction of total-stake
	/// - `total_stake`: Total stake of the validator
	///
	/// # Returns
	/// Returns the slash value
	pub(crate) fn calculate_restake_slash_value(
		slash_fraction: Perbill,
		total_stake: BalanceOf<T>,
	) -> BalanceOf<T> {
		// Slash value for given slash fraction
		slash_fraction * total_stake
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
			.clone()
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
			role_type: offence_report.role_type,
			offence_type: offence_report.offence_type,
		};
		let _ = T::ReportOffences::report_offence(sp_std::vec![], offence.clone());
		// Update and apply slash on ledger for all offenders
		let slash_fraction = offence.slash_fraction(offence.validator_set_count);
		for offender in offence_report.offenders {
			let mut profile_ledger =
				<Ledger<T>>::get(&offender).ok_or(Error::<T>::NoProfileFound)?;
			let staking_ledger =
				pallet_staking::Ledger::<T>::get(&offender).ok_or(Error::<T>::NotValidator)?;
			let slash_value =
				Self::calculate_restake_slash_value(slash_fraction, staking_ledger.total);
			// apply slash
			profile_ledger.total = profile_ledger.total.saturating_sub(slash_value);
			Self::update_ledger(&offender, &profile_ledger);
		}

		Ok(())
	}

	/// Update the ledger for the given stash account.
	///
	/// # Parameters
	/// - `staker`: The stash account ID.
	/// - `ledger`: The new ledger.
	pub(crate) fn update_ledger(staker: &T::AccountId, ledger: &RoleStakingLedger<T>) {
		<Ledger<T>>::insert(staker, ledger);
	}

	pub fn distribute_rewards() -> DispatchResult {
		let total_rewards = T::InflationRewardPerSession::get();

		let mut tss_validators: Vec<T::AccountId> = Default::default();
		let mut zksaas_validators: Vec<T::AccountId> = Default::default();

		for (acc, role_types) in AccountRolesMapping::<T>::iter() {
			for role_type in role_types.iter() {
				match role_type {
					RoleType::Tss(_) =>
						if !tss_validators.contains(&acc) {
							tss_validators.push(acc.clone());
						},
					RoleType::ZkSaaS(_) =>
						if !zksaas_validators.contains(&acc) {
							zksaas_validators.push(acc.clone());
						},
					_ => (),
				}
			}
		}

		log::debug!("Found {:?} tss validators", tss_validators.len());
		log::debug!("Found {:?} zksaas validators", zksaas_validators.len());

		let reward_distribution = T::ValidatorRewardDistribution::get();

		let dist = reward_distribution.get_reward_distribution();

		let tss_validator_reward = dist
			.0
			.mul_floor(total_rewards)
			.checked_div(&(tss_validators.len() as u32).into())
			.unwrap_or(Zero::zero());
		let zksaas_validators_reward = dist
			.1
			.mul_floor(total_rewards)
			.checked_div(&(zksaas_validators.len() as u32).into())
			.unwrap_or(Zero::zero());

		log::debug!("Reward for tss validator : {:?}", tss_validator_reward);
		log::debug!("Reward for zksaas validator : {:?}", zksaas_validators_reward);

		for validator in tss_validators {
			T::Currency::deposit_creating(&validator, tss_validator_reward);
		}

		for validator in zksaas_validators {
			T::Currency::deposit_creating(&validator, zksaas_validators_reward);
		}

		Ok(())
	}

	pub fn update_ledger_role_key(staker: &T::AccountId, role_key: Vec<u8>) -> DispatchResult {
		let mut ledger = Ledger::<T>::get(staker).ok_or(Error::<T>::NoProfileFound)?;
		let bounded_role_key: BoundedVec<u8, T::MaxKeyLen> =
			role_key.try_into().map_err(|_| Error::<T>::KeySizeExceeded)?;
		ledger.role_key = bounded_role_key;
		Self::update_ledger(staker, &ledger);
		Ok(())
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
		let _ = Self::distribute_rewards();
	}
}
