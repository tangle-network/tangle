/// Functions for the pallet.
use super::*;
use crate::{offences::ValidatorOffence, types::*};
use frame_support::{
	pallet_prelude::DispatchResult,
	traits::{DefensiveResult, Imbalance, OnUnbalanced},
};

use pallet_staking::ActiveEra;
use sp_runtime::{traits::Convert, Perbill};
use sp_staking::offence::Offence;
use sp_std::collections::btree_map::BTreeMap;
use tangle_primitives::jobs::{traits::JobsHandler, JobId, ReportRestakerOffence};

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
						return Err(Error::<T>::RoleCannotBeRemoved.into());
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
					updated_role_restaking_amount
						>= current_ledger.profile.get_total_profile_restake(),
					Error::<T>::InsufficientRestakingBond
				);
			}

			return Ok(());
		}
		// For each role with an active job, ensure its stake is greater than or equal to the
		// existing ledger's restaking amount for that role. If it's a shared profile, then the
		// restaking amount for that role is the entire shared restaking amount.
		let min_restaking_bond = MinRestakingBond::<T>::get();
		for record in updated_profile.clone().get_records() {
			match updated_profile.clone() {
				Profile::Independent(_) => {
					if roles_with_active_jobs.contains(&record.role) {
						ensure!(
							record.amount.unwrap_or_default() >= min_restaking_bond,
							Error::<T>::InsufficientRestakingBond
						);
						ensure!(
							record.amount.unwrap_or_default()
								>= current_ledger.restake_for(&record.role),
							Error::<T>::InsufficientRestakingBond
						);
					}
				},
				Profile::Shared(profile) => {
					if roles_with_active_jobs.contains(&record.role) {
						ensure!(
							profile.amount >= current_ledger.profile.get_total_profile_restake(),
							Error::<T>::InsufficientRestakingBond
						);
					}
				},
			}
		}
		Ok(())
	}

	/// Check if account can chill, unbond and withdraw funds.
	///
	/// # Parameters
	/// - `account`: The account ID of the restaker.
	///
	/// # Returns
	/// Returns boolean value.
	pub(crate) fn can_exit(account: T::AccountId) -> bool {
		let assigned_roles = AccountRolesMapping::<T>::get(account);
		if assigned_roles.is_empty() {
			// Role is cleared, account can chill, unbond and withdraw funds.
			return true;
		}
		false
	}

	/// Calculate max restake amount for the given account.
	///
	/// # Parameters
	/// - `total_stake`: Total stake of the restaker
	///
	/// # Returns
	/// Returns the max restake amount.
	pub(crate) fn calculate_max_restake_amount(total_stake: BalanceOf<T>) -> BalanceOf<T> {
		// User can restake max 50% of the total stake
		T::MaxRestake::get() * total_stake
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

	/// Report offence for the given restaker.
	/// This function will report restakers for committing offence.
	///
	/// # Parameters
	/// - `offence_report`: The offence report.
	///
	/// # Returns
	///
	/// Returns Ok() if restaker offence report is submitted successfully.
	pub(crate) fn report_offence(
		offence_report: ReportRestakerOffence<T::AccountId>,
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
	/// - `restaker`: The stash account ID.
	/// - `ledger`: The new ledger.
	pub(crate) fn update_ledger(restaker: &T::AccountId, ledger: &RestakingLedger<T>) {
		<Ledger<T>>::insert(restaker, ledger);
	}

	/// Computes rewards for validators in the current era.
	///
	/// This function computes rewards for validators in the current era. It divides the total
	/// rewards available for the era into two parts: one for active validators and the other for
	/// validators based on their restake amount. The rewards are then combined, and individual
	/// reward points are assigned to each eligible validator. Finally, the reward points are stored
	/// for the current era, and relevant storage items are updated accordingly.
	///
	/// # Arguments
	///
	/// * `current_era_index` - The index of the current era.
	///
	/// # Errors
	///
	/// Returns an error if any dispatch operation fails.
	pub fn compute_rewards(current_era_index: EraIndex) -> DispatchResult {
		let total_rewards = T::InflationRewardPerSession::get();

		let active_validator_rewards: BTreeMap<_, _> =
			Self::compute_active_validator_rewards(total_rewards / 2_u32.into());

		let role_type_validator_rewards: BTreeMap<_, _> =
			Self::compute_validator_rewards_by_restake(
				total_rewards / 2_u32.into(),
				current_era_index,
			);

		let mut combined_validator_rewards: BTreeMap<_, _> = Default::default();
		for (validator, role_reward) in role_type_validator_rewards.iter() {
			if let Some(reward) = active_validator_rewards.get(&validator) {
				combined_validator_rewards
					.insert(validator.clone(), role_reward.saturating_add(*reward));
			} else {
				combined_validator_rewards.insert(validator.clone(), *role_reward);
			}
		}

		// set the era restaker reward point storage
		let mut era_reward_points = <ErasRestakeRewardPoints<T>>::get(&current_era_index);
		era_reward_points.total = total_rewards.try_into().unwrap_or(0_u32);

		// add individual reward points to each eligible validator
		for (validator, reward) in combined_validator_rewards.into_iter() {
			let reward_as_u32: u32 = reward.try_into().unwrap_or(0_u32);
			era_reward_points.individual.insert(validator.clone(), reward_as_u32.into());
		}

		ErasRestakeRewardPoints::<T>::insert(current_era_index, era_reward_points);
		ValidatorJobsInEra::<T>::take();
		Self::deposit_event(Event::<T>::RolesRewardSet { total_rewards });

		Ok(())
	}

	/// Computes rewards for active validators based on their completed jobs.
	///
	/// This function calculates rewards for validators who have completed jobs in the current era.
	/// The rewards are distributed among active validators proportionally to the number of jobs
	/// they have completed relative to the total number of jobs completed by all active validators.
	///
	/// # Arguments
	///
	/// * `total_rewards` - Total rewards available to distribute among active validators.
	///
	/// # Returns
	///
	/// A `BTreeMap` containing the account IDs of active validators as keys and their corresponding
	/// rewards as values.
	pub fn compute_active_validator_rewards(
		total_rewards: BalanceOf<T>,
	) -> BTreeMap<T::AccountId, BalanceOf<T>> {
		// Log total rewards available for debugging
		log::debug!(
			target: "pallet-roles",
			"Compute_active_validator_rewards, total rewards available {:?}",
			total_rewards
		);

		// Retrieve active validators and their rewards from storage
		let active_validators = ValidatorJobsInEra::<T>::get();

		// Initialize a map to store rewards for active validators
		let mut active_validator_reward_map: BTreeMap<_, _> = Default::default();

		// Calculate the total jobs for all active validators
		let total_jobs_completed: u32 =
			active_validators.values().rfold(0_u32.into(), |acc, &x| acc + x);

		// Distribute rewards to active validators based on their share
		for (validator, jobs_completed) in active_validators.iter() {
			// Calculate the share of rewards for the current validator
			let validator_share =
				Perbill::from_rational(jobs_completed.clone(), total_jobs_completed);

			// Calculate the actual reward amount for the current validator
			let validator_share_of_total_reward = validator_share.mul_floor(total_rewards);
			// Insert the validator's account ID and reward amount into the reward map
			active_validator_reward_map.insert(
				validator.clone(),
				validator_share_of_total_reward.saturating_add((*jobs_completed).into()),
			);
		}

		// Return the map containing rewards for active validators
		active_validator_reward_map
	}

	/// Computes validator rewards based on inflation.
	///
	/// This function calculates rewards for validators based on the amount they have staked
	/// in the system. The rewards are distributed proportionally to the amount of staked
	/// tokens each validator holds relative to the total staked tokens in the system.
	///
	/// # Arguments
	///
	/// * `total_rewards` - Total rewards available to distribute among validators.
	///
	/// # Returns
	///
	/// A `BTreeMap` containing validator accounts as keys and their corresponding rewards as
	/// values.
	pub fn compute_validator_rewards_by_restake(
		total_rewards: BalanceOf<T>,
		era_index: EraIndex,
	) -> BTreeMap<T::AccountId, BalanceOf<T>> {
		let mut total_restake: BalanceOf<T> = Default::default();
		let mut restakers_with_restake: Vec<(T::AccountId, BalanceOf<T>)> = Default::default();

		let total_stake_in_system: BalanceOf<T> =
			pallet_staking::Pallet::<T>::eras_total_stake(era_index);

		// TODO : This is an unbounded query, potentially dangerous
		for (restaker, ledger) in Ledger::<T>::iter() {
			restakers_with_restake.push((restaker, ledger.total_restake()));
			total_restake += ledger.total_restake();
		}

		let restake_to_stake_ratio = Perbill::from_rational(total_restake, total_stake_in_system);
		log::debug!(
			target: "pallet-roles",
			"EraIndex {:?} restake_to_stake_ratio {:?}",
			era_index, restake_to_stake_ratio
		);

		let missing_restake_ratio =
			Perbill::from_percent(T::MaxRestake::get().deconstruct().into())
				- restake_to_stake_ratio;
		log::debug!(
			target: "pallet-roles",
			"EraIndex {:?} missing_restake_ratio {:?}",
			era_index, missing_restake_ratio
		);

		let mut normalized_total_rewards = total_rewards;

		if !missing_restake_ratio.is_zero() {
			normalized_total_rewards =
				(Perbill::from_percent(T::MaxRestake::get().deconstruct().into())
					.saturating_sub(missing_restake_ratio))
				.mul_floor(total_rewards);
		}

		log::debug!(
			target: "pallet-roles",
			"EraIndex {:?} total_rewards {:?} normalized_total_rewards {:?}",
			era_index, total_rewards, normalized_total_rewards
		);

		let mut validator_reward_map: BTreeMap<T::AccountId, BalanceOf<T>> = Default::default();

		for (restaker, restake_amount) in restakers_with_restake {
			let restaker_reward_share = Perbill::from_rational(restake_amount, total_restake);
			let restaker_reward = restaker_reward_share.mul_floor(normalized_total_rewards);
			validator_reward_map.insert(restaker, restaker_reward);
		}

		validator_reward_map
	}

	/// Updates the role key associated with a restaker's ledger.
	///
	/// This function allows updating the role key associated with a restaker's ledger entry.
	/// It takes the restaker's account ID and the new role key as input parameters.
	///
	/// # Arguments
	///
	/// * `restaker` - The account ID of the restaker whose ledger's role key is to be updated.
	/// * `role_key` - The new role key to be associated with the restaker's ledger.
	///
	/// # Errors
	///
	/// Returns an error if no ledger entry is found for the given restaker or if the provided
	/// role key exceeds the maximum allowed key length.
	pub fn update_ledger_role_key(restaker: &T::AccountId, role_key: Vec<u8>) -> DispatchResult {
		let mut ledger = Ledger::<T>::get(restaker).ok_or(Error::<T>::NoProfileFound)?;
		let bounded_role_key: BoundedVec<u8, T::MaxKeyLen> =
			role_key.try_into().map_err(|_| Error::<T>::KeySizeExceeded)?;
		ledger.role_key = bounded_role_key;
		Self::update_ledger(restaker, &ledger);
		Ok(())
	}

	pub fn do_payout_stakers(validator_stash: T::AccountId, era: EraIndex) -> DispatchResult {
		// Validate input data
		let current_era =
			ActiveEra::<T>::get().ok_or_else(|| Error::<T>::InvalidEraToReward)?.index;

		let history_depth = <T as pallet_staking::Config>::HistoryDepth::get();

		ensure!(
			era <= current_era && era >= current_era.saturating_sub(history_depth),
			Error::<T>::InvalidEraToReward
		);

		let mut ledger = <Ledger<T>>::get(&validator_stash).ok_or(Error::<T>::NoProfileFound)?;

		ledger
			.claimed_rewards
			.retain(|&x| x >= current_era.saturating_sub(history_depth));

		match ledger.claimed_rewards.binary_search(&era) {
			Ok(_) => return Err(Error::<T>::AlreadyClaimed.into()),
			Err(pos) => ledger
				.claimed_rewards
				.try_insert(pos, era)
				// Since we retain era entries in `claimed_rewards` only upto
				// `HistoryDepth`, following bound is always expected to be
				// satisfied.
				.defensive_map_err(|_| Error::<T>::BoundNotMet)?,
		}

		let exposure = <pallet_staking::ErasStakersClipped<T>>::get(&era, &ledger.stash);

		// Input data seems good, no errors allowed after this point
		<Ledger<T>>::insert(&validator_stash, &ledger);

		let era_reward_points = <ErasRestakeRewardPoints<T>>::get(&era);
		let restaker_reward_points = era_reward_points
			.individual
			.get(&ledger.stash)
			.copied()
			.unwrap_or_else(Zero::zero);

		// Nothing to do if they have no reward points.
		if restaker_reward_points.is_zero() {
			return Ok(());
		}

		// This is how much validator + nominators are entitled to.
		let validator_total_payout = restaker_reward_points;

		let validator_prefs =
			pallet_staking::Pallet::<T>::eras_validator_prefs(&era, &validator_stash);
		// Validator first gets a cut off the top.
		let validator_commission = validator_prefs.commission;
		let validator_commission_payout = validator_commission * validator_total_payout;

		let validator_leftover_payout = validator_total_payout - validator_commission_payout;
		// Now let's calculate how this is split to the validator.
		let validator_exposure_part = Perbill::from_rational(exposure.own, exposure.total);
		let validator_staking_payout = validator_exposure_part * validator_leftover_payout;

		Self::deposit_event(Event::<T>::PayoutStarted {
			era_index: era,
			validator_stash: ledger.stash.clone(),
		});

		let mut total_imbalance = PositiveImbalanceOf::<T>::zero();
		// We can now make total validator payout
		// we use the staking pallet payout function since it handles the reward destination
		if let Some(imbalance) = Self::make_payout(
			&ledger.stash,
			(validator_staking_payout + validator_commission_payout).into(),
		) {
			Self::deposit_event(Event::<T>::Rewarded {
				stash: ledger.stash,
				amount: imbalance.peek(),
			});
			total_imbalance.subsume(imbalance);
		}

		// Track the number of payout ops to nominators. Note:
		// `WeightInfo::payout_stakers_alive_staked` always assumes at least a validator is paid
		// out, so we do not need to count their payout op.
		let mut nominator_payout_count: u32 = 0;

		// Lets now calculate how this is split to the nominators.
		// Reward only the clipped exposures. Note this is not necessarily sorted.
		for nominator in exposure.others.iter() {
			let nominator_exposure_part = Perbill::from_rational(nominator.value, exposure.total);

			let nominator_reward: BalanceOf<T> =
				(nominator_exposure_part * validator_leftover_payout).into();
			// We can now make nominator payout:
			if let Some(imbalance) = Self::make_payout(&nominator.who, nominator_reward) {
				// Note: this logic does not count payouts for `RewardDestination::None`.
				nominator_payout_count += 1;
				let e =
					Event::<T>::Rewarded { stash: nominator.who.clone(), amount: imbalance.peek() };
				Self::deposit_event(e);
				total_imbalance.subsume(imbalance);
			}
		}

		T::Reward::on_unbalanced(total_imbalance);
		Ok(())
	}

	/// Actually make a payment to a staker. This uses the currency's reward function
	/// to pay the right payee for the given staker account.
	fn make_payout(stash: &T::AccountId, amount: BalanceOf<T>) -> Option<PositiveImbalanceOf<T>> {
		// TODO : Consider handling RewardDestination config
		Some(T::Currency::deposit_creating(&stash, amount))
	}
}
