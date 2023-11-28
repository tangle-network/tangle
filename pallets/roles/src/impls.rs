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

use super::*;
use frame_support::{
	log,
	pallet_prelude::DispatchResult,
	traits::{Currency, OneSessionHandler},
};
use sp_runtime::{traits::CheckedDiv, Percent, Saturating};

use tangle_primitives::{jobs::JobKey, traits::roles::RolesHandler};

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

	/// Slash validator stake for the reported offence. The function should be a best effort
	/// slashing, slash upto max possible by the offence type.
	///
	/// # Parameters
	/// - `address`: The account ID of the validator.
	/// - `offence`: The offence reported against the validator
	///
	/// # Returns
	/// DispatchResult emitting `Slashed` event if validator is slashed
	fn slash_validator(
		address: T::AccountId,
		_offence: tangle_primitives::jobs::ValidatorOffence,
	) -> sp_runtime::DispatchResult {
		// TODO: implement calculation of slash amount.
		let slash_amount = 1000u64;
		Self::do_slash(address, slash_amount.into())?;
		Ok(())
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
	pub fn add_role(account: T::AccountId, role: RoleType) -> DispatchResult {
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
	pub fn remove_role(account: T::AccountId, role: RoleType) -> DispatchResult {
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
	pub fn has_role(account: T::AccountId, role: RoleType) -> bool {
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
	pub fn can_exit(account: T::AccountId) -> bool {
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
	pub fn calculate_max_re_stake_amount(total_stake: BalanceOf<T>) -> BalanceOf<T> {
		// User can re-stake max 50% of the total stake
		Percent::from_percent(50) * total_stake
	}

	/// Get the total amount of the balance that is locked for the given stash.
	///
	/// # Parameters
	/// - `stash`: The stash account ID.
	///
	/// # Returns
	/// The total amount of the balance that can be slashed.
	pub fn slashable_balance_of(stash: &T::AccountId) -> BalanceOf<T> {
		// Weight note: consider making the stake accessible through stash.
		Self::ledger(&stash).map(|l| l.total).unwrap_or_default()
	}

	/// Slash the given amount from the stash account.
	///
	/// # Parameters
	/// - `address`: The stash account ID.
	/// - `slash_amount`: The amount to be slashed.
	pub(crate) fn do_slash(
		address: T::AccountId,
		slash_amount: T::CurrencyBalance,
	) -> sp_runtime::DispatchResult {
		let mut ledger = Self::ledger(&address).ok_or(Error::<T>::AccountNotPaired)?;
		let (_imbalance, _missing) = T::Currency::slash(&address, slash_amount.into());
		ledger.total = ledger.total.saturating_sub(slash_amount.into());
		Self::update_ledger(&address, &ledger);
		Self::deposit_event(Event::Slashed { account: address, amount: slash_amount });
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

	pub fn distribute_rewards() -> DispatchResult {
		let total_rewards = T::InflationRewardPerSession::get();

		let mut tss_validators: Vec<T::AccountId> = Default::default();
		let mut zksaas_validators: Vec<T::AccountId> = Default::default();

		for (acc, role_types) in AccountRolesMapping::<T>::iter() {
			if role_types.contains(&RoleType::Tss) {
				tss_validators.push(acc.clone())
			}

			if role_types.contains(&RoleType::ZkSaas) {
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
