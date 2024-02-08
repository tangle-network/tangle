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
use crate::offences::ValidatorOffence;
use frame_support::{pallet_prelude::DispatchResult, traits::OneSessionHandler};
use pallet_staking::{CurrentEra, ErasRewardPoints};
use sp_runtime::{
	traits::{CheckedDiv, Convert},
	Perbill, Percent,
};
use sp_staking::offence::Offence;
use sp_std::collections::btree_map::BTreeMap;
use tangle_primitives::{
	jobs::{traits::JobsHandler, JobId, ReportValidatorOffence},
	roles::traits::RolesHandler,
};

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
	fn record_reward_to_validator(
		validators: Vec<T::AccountId>,
		reward_per_validator: BalanceOf<T>,
	) -> DispatchResult {
		let mut validator_reward_map = ValidatorRewardsInSession::<T>::get();
		let default_points: BalanceOf<T> = 0u32.into();

		for validator in validators {
			let current_points: BalanceOf<T> =
				*validator_reward_map.get(&validator).unwrap_or(&default_points);
			let new_points = current_points.saturating_add(reward_per_validator.into());
			let _ = validator_reward_map.try_insert(validator, new_points);
		}

		ValidatorRewardsInSession::<T>::put(validator_reward_map);

		Ok(())
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

	/// This function retrieves the current era and the total rewards available for distribution
	/// from the runtime configuration. It then computes rewards for active validators and
	/// validators based on their role types. After computing rewards, it adds individual reward
	/// points to each eligible validator and records them in the staking pallet ErasRewardPoints
	/// storage for the current era.
	///
	/// The total reward is calculated as follows:
	/// 1. Fifty percent of the total rewards is divided among all "active" validators. Active
	///    validators
	/// are those who have completed at least one job in the last session. Within this group, the
	/// ratio of payout to each validator is determined by the rewards assigned to the validator
	/// compared to the total rewards in the system.
	///
	/// 2. The remaining fifty percent of the total rewards is divided among all validators. The
	///    ratio for
	/// each validator is determined by reading the storage `ValidatorRewardDistribution` that
	/// assigns rewards based on the role type of active validators.
	///
	/// # Errors
	///
	/// Returns an error if the distribution encounters any issue while depositing rewards.
	pub fn distribute_rewards() -> DispatchResult {
		let current_era = CurrentEra::<T>::get().ok_or(Error::<T>::CannotGetCurrentEra)?;

		let total_rewards = T::InflationRewardPerSession::get();

		let active_validator_rewards: BTreeMap<_, _> =
			Self::compute_active_validator_rewards(total_rewards / 2_u32.into());

		let role_type_validator_rewards: BTreeMap<_, _> =
			Self::compute_validator_rewards_by_role_type(total_rewards / 2_u32.into());

		let mut combined_validator_rewards: BTreeMap<_, _> = Default::default();
		for (validator, role_reward) in role_type_validator_rewards.iter() {
			if let Some(reward) = active_validator_rewards.get(&validator) {
				combined_validator_rewards
					.insert(validator.clone(), role_reward.saturating_add(*reward));
			} else {
				combined_validator_rewards.insert(validator.clone(), *role_reward);
			}
		}

		// record these rewards in the staking pallet storage
		let mut era_reward_points = <ErasRewardPoints<T>>::get(&current_era);
		let total_reward_points = total_rewards.saturating_add(era_reward_points.total.into());
		era_reward_points.total = total_reward_points.try_into().unwrap_or(0_u32);

		// add individual reward points to each eligible validator
		for (validator, reward) in combined_validator_rewards.iter() {
			let current_points = era_reward_points.individual.get(&validator).unwrap_or(&0_u32);
			let new_points = reward.saturating_add((*current_points).into());
			let new_points_as_u32: u32 = new_points.try_into().unwrap_or(0_u32);
			era_reward_points.individual.insert(validator.clone(), new_points_as_u32.into());
		}

		ErasRewardPoints::<T>::insert(current_era, era_reward_points);
		ValidatorRewardsInSession::<T>::take();
		Self::deposit_event(Event::<T>::RolesRewardPaid { total_rewards });

		Ok(())
	}

	/// Computes the rewards for active validators based on the total rewards available
	///
	/// Given the total rewards available, this function calculates the rewards for each active
	/// validator based on their share of the total rewards.
	///
	/// # Parameters
	///
	/// - `total_rewards`: Total rewards available for distribution among active validators.
	///
	/// # Returns
	///
	/// A `BTreeMap` containing the account IDs of active validators as keys and their respective
	/// reward amounts as values.
	pub fn compute_active_validator_rewards(
		total_rewards: BalanceOf<T>,
	) -> BTreeMap<T::AccountId, BalanceOf<T>> {
		// Log total rewards available for debugging
		log::debug!(
			"Compute_active_validator_rewards, total rewards available {:?}",
			total_rewards
		);

		// Retrieve active validators and their rewards from storage
		let active_validators = ValidatorRewardsInSession::<T>::get();

		// Initialize a map to store rewards for active validators
		let mut active_validator_reward_map: BTreeMap<_, _> = Default::default();

		// Calculate the total rewards for all active validators
		let total_rewards_accumlated: BalanceOf<T> =
			active_validators.values().rfold(0_u32.into(), |acc, &x| acc + x);

		// Distribute rewards to active validators based on their share
		for (validator, reward) in active_validators.iter() {
			// Calculate the share of rewards for the current validator
			let validator_share = Perbill::from_rational(reward.clone(), total_rewards_accumlated);

			// Calculate the actual reward amount for the current validator
			let validator_share_of_total_reward = validator_share.mul_floor(total_rewards);
			// Insert the validator's account ID and reward amount into the reward map
			active_validator_reward_map
				.insert(validator.clone(), validator_share_of_total_reward.saturating_add(*reward));
		}

		// Return the map containing rewards for active validators
		active_validator_reward_map
	}

	/// Computes the rewards for validators based on their role types and the total rewards
	/// available
	///
	/// Given the total rewards available and the distribution of rewards among different role
	/// types, this function calculates the rewards for validators based on their assigned role
	/// types.
	///
	/// # Parameters
	///
	/// - `total_rewards`: Total rewards available for distribution among validators.
	///
	/// # Returns
	///
	/// A `BTreeMap` containing the account IDs of validators as keys and their respective
	/// reward amounts as values.
	///
	/// # Remarks
	///
	/// The function retrieves the distribution of rewards among different role types from the
	/// runtime configuration. It iterates over all accounts and their associated role types to
	/// categorize validators into two groups:
	/// - TSS validators
	/// - ZkSaaS validators
	/// The rewards are then calculated based on the distribution of rewards and the number of
	/// validators in each group. The resulting rewards are stored in a map, where each validator's
	/// account ID is associated with its reward amount.
	pub fn compute_validator_rewards_by_role_type(
		total_rewards: BalanceOf<T>,
	) -> BTreeMap<T::AccountId, BalanceOf<T>> {
		let mut tss_validators: Vec<T::AccountId> = Default::default();
		let mut zksaas_validators: Vec<T::AccountId> = Default::default();
		let dist = T::ValidatorRewardDistribution::get().get_reward_distribution();

		// TODO : This is an unbounded query, potentially dangerous
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
					_ => (), // we dont care about other role types for inflation rewards
				}
			}
		}

		log::debug!("Found {:?} tss validators", tss_validators.len());
		log::debug!("Found {:?} zksaas validators", zksaas_validators.len());

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

		log::debug!("Reward per tss validator : {:?}", tss_validator_reward);
		log::debug!("Reward per zksaas validator : {:?}", zksaas_validators_reward);

		let mut validator_reward_map: BTreeMap<T::AccountId, BalanceOf<T>> = Default::default();

		for validator in tss_validators {
			validator_reward_map.insert(validator, tss_validator_reward);
		}

		for validator in zksaas_validators {
			let default_balance: BalanceOf<T> = 0u32.into();
			let current_rewards: BalanceOf<T> =
				*validator_reward_map.get(&validator).unwrap_or(&default_balance);
			validator_reward_map
				.insert(validator, current_rewards.saturating_add(zksaas_validators_reward));
		}

		validator_reward_map
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
