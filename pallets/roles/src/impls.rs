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
use frame_support::{
	log,
	pallet_prelude::DispatchResult,
	traits::{Currency, OneSessionHandler},
};
use sp_runtime::{
	traits::{CheckedDiv, Convert},
	Perbill, Percent,
};

use sp_staking::offence::Offence;
use tangle_primitives::{
	jobs::{JobKey, ReportValidatorOffence},
	traits::{jobs::MPCHandler, roles::RolesHandler},
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
				return ledger.profile.get_role_metadata(job_key.get_role_type())
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
	/// Validate updated profile for the given account.
	/// This function will validate the updated profile for the given account.
	///
	/// # Parameters
	/// - `account`: The account ID of the validator.
	/// - `updated_profile`: The updated profile.
	pub(crate) fn validate_updated_profile(
		account: T::AccountId,
		updated_profile: Profile<T>,
	) -> DispatchResult {
		let ledger = Self::ledger(&account).ok_or(Error::<T>::NoProfileFound)?;
		let removed_roles = ledger.profile.get_removed_roles(&updated_profile);
		if !removed_roles.is_empty() {
			let active_jobs = T::JobsHandler::get_active_jobs(account.clone());
			// Check removed roles has any active jobs.
			for role in removed_roles {
				for job in active_jobs.clone() {
					let job_key = job.0;
					if job_key.get_role_type() == role {
						return Err(Error::<T>::RoleCannotBeRemoved.into())
					}
				}
			}
		};

		let records = updated_profile.get_records();
		let min_re_staking_bond = MinReStakingBond::<T>::get();

		for record in records {
			if updated_profile.is_independent() {
				// Re-staking amount of record should meet min re-staking amount requirement.
				let record_re_stake = record.amount.unwrap_or_default();
				ensure!(
					record_re_stake >= min_re_staking_bond,
					Error::<T>::InsufficientReStakingBond
				);
			}
			// validate the metadata
			T::MPCHandler::validate_authority_key(
				account.clone(),
				record.metadata.get_authority_key(),
			)?;
		}
		Ok(())
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
	/// Calculate slash value for re-staked amount
	///
	/// # Parameters
	/// - slash_fraction: Slash fraction of total-stake
	/// - `total_stake`: Total stake of the validator
	///
	/// # Returns
	/// Returns the slash value
	pub(crate) fn calculate_re_stake_slash_value(
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
				Self::calculate_re_stake_slash_value(slash_fraction, staking_ledger.total);
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
			if role_types.contains(&RoleType::Tss) {
				tss_validators.push(acc.clone())
			}

			if role_types.contains(&RoleType::ZkSaaS) {
				zksaas_validators.push(acc)
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
}

impl<T: Config> sp_runtime::BoundToRuntimeAppPublic for Pallet<T> {
	type Public = T::AuthorityId;
}

impl<T: Config> OneSessionHandler<T::AccountId> for Pallet<T> {
	type Key = T::AuthorityId;

	fn on_genesis_session<'a, I: 'a>(_validators: I)
	where
		I: Iterator<Item = (&'a T::AccountId, T::AuthorityId)>,
	{
		// nothing to be done
	}

	fn on_new_session<'a, I: 'a>(_changed: bool, _validators: I, _queued_validators: I)
	where
		I: Iterator<Item = (&'a T::AccountId, T::AuthorityId)>,
	{
		// nothing to be done
	}

	fn on_disabled(_i: u32) {
		// ignore
	}

	// Distribute the inflation rewards
	fn on_before_session_ending() {
		let _ = Self::distribute_rewards();
	}
}
