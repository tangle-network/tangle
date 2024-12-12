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
use super::*;
use crate::{types::*, Pallet};
use tangle_primitives::services::Asset;
use frame_support::traits::fungibles::Mutate;
use frame_support::traits::tokens::Preservation;
use frame_support::{ensure, pallet_prelude::DispatchResult, traits::Get};
use sp_runtime::traits::{CheckedSub, Zero};
use sp_runtime::DispatchError;
use sp_runtime::Percent;
use sp_std::vec::Vec;
use tangle_primitives::BlueprintId;

impl<T: Config> Pallet<T> {
	/// Processes the delegation of an amount of an asset to an operator.
	/// Creates a new delegation for the delegator and updates their status to active, the deposit
	/// of the delegator is moved to delegation.
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	/// * `operator` - The account ID of the operator.
	/// * `asset_id` - The ID of the asset to be delegated.
	/// * `amount` - The amount to be delegated.
	///
	/// # Errors
	///
	/// Returns an error if the delegator does not have enough deposited balance,
	/// or if the operator is not found.
	pub fn process_delegate(
		who: T::AccountId,
		operator: T::AccountId,
		asset_id: Asset<T::AssetId>,
		amount: BalanceOf<T>,
		blueprint_selection: DelegatorBlueprintSelection<T::MaxDelegatorBlueprints>,
	) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Ensure enough deposited balance
			let balance =
				metadata.deposits.get_mut(&asset_id).ok_or(Error::<T>::InsufficientBalance)?;
			ensure!(*balance >= amount, Error::<T>::InsufficientBalance);

			// Reduce the balance in deposits
			*balance = balance.checked_sub(&amount).ok_or(Error::<T>::InsufficientBalance)?;
			if *balance == Zero::zero() {
				metadata.deposits.remove(&asset_id);
			}

			// Check if the delegation exists and update it, otherwise create a new delegation
			if let Some(delegation) = metadata
				.delegations
				.iter_mut()
				.find(|d| d.operator == operator && d.asset_id == asset_id)
			{
				delegation.amount += amount;
			} else {
				// Create the new delegation
				let new_delegation = BondInfoDelegator {
					operator: operator.clone(),
					amount,
					asset_id,
					blueprint_selection,
				};

				// Create a mutable copy of delegations
				let mut delegations = metadata.delegations.clone();
				delegations
					.try_push(new_delegation)
					.map_err(|_| Error::<T>::MaxDelegationsExceeded)?;
				metadata.delegations = delegations;

				// Update the status
				metadata.status = DelegatorStatus::Active;
			}

			// Update the operator's metadata
			if let Some(mut operator_metadata) = Operators::<T>::get(&operator) {
				// Check if the operator has capacity for more delegations
				ensure!(
					operator_metadata.delegation_count < T::MaxDelegations::get(),
					Error::<T>::MaxDelegationsExceeded
				);

				// Create and push the new delegation bond
				let delegation = DelegatorBond { delegator: who.clone(), amount, asset_id };

				let mut delegations = operator_metadata.delegations.clone();

				// Check if delegation already exists
				if let Some(existing_delegation) =
					delegations.iter_mut().find(|d| d.delegator == who && d.asset_id == asset_id)
				{
					existing_delegation.amount += amount;
				} else {
					delegations
						.try_push(delegation)
						.map_err(|_| Error::<T>::MaxDelegationsExceeded)?;
					operator_metadata.delegation_count =
						operator_metadata.delegation_count.saturating_add(1);
				}

				operator_metadata.delegations = delegations;

				// Update storage
				Operators::<T>::insert(&operator, operator_metadata);
			} else {
				return Err(Error::<T>::NotAnOperator.into());
			}

			Ok(())
		})
	}

	/// Schedules a stake reduction for a delegator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	/// * `operator` - The account ID of the operator.
	/// * `asset_id` - The ID of the asset to be reduced.
	/// * `amount` - The amount to be reduced.
	///
	/// # Errors
	///
	/// Returns an error if the delegator has no active delegation,
	/// or if the unstake amount is greater than the current delegation amount.
	pub fn process_schedule_delegator_unstake(
		who: T::AccountId,
		operator: T::AccountId,
		asset_id: Asset<T::AssetId>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Ensure the delegator has an active delegation with the operator for the given asset
			let delegation_index = metadata
				.delegations
				.iter()
				.position(|d| d.operator == operator && d.asset_id == asset_id)
				.ok_or(Error::<T>::NoActiveDelegation)?;

			// Get the delegation and clone necessary data
			let blueprint_selection =
				metadata.delegations[delegation_index].blueprint_selection.clone();
			let delegation = &mut metadata.delegations[delegation_index];
			ensure!(delegation.amount >= amount, Error::<T>::InsufficientBalance);

			delegation.amount -= amount;

			// Create the unstake request
			let current_round = Self::current_round();
			let mut unstake_requests = metadata.delegator_unstake_requests.clone();
			unstake_requests
				.try_push(BondLessRequest {
					operator: operator.clone(),
					asset_id,
					amount,
					requested_round: current_round,
					blueprint_selection,
				})
				.map_err(|_| Error::<T>::MaxUnstakeRequestsExceeded)?;
			metadata.delegator_unstake_requests = unstake_requests;

			// Remove the delegation if the remaining amount is zero
			if delegation.amount.is_zero() {
				metadata.delegations.remove(delegation_index);
			}

			// Update the operator's metadata
			Operators::<T>::try_mutate(&operator, |maybe_operator_metadata| -> DispatchResult {
				let operator_metadata =
					maybe_operator_metadata.as_mut().ok_or(Error::<T>::NotAnOperator)?;

				// Ensure the operator has a matching delegation
				let operator_delegation_index = operator_metadata
					.delegations
					.iter()
					.position(|d| d.delegator == who && d.asset_id == asset_id)
					.ok_or(Error::<T>::NoActiveDelegation)?;

				let operator_delegation =
					&mut operator_metadata.delegations[operator_delegation_index];

				// Reduce the amount in the operator's delegation
				ensure!(operator_delegation.amount >= amount, Error::<T>::InsufficientBalance);
				operator_delegation.amount -= amount;

				// Remove the delegation if the remaining amount is zero
				if operator_delegation.amount.is_zero() {
					operator_metadata.delegations.remove(operator_delegation_index);
					operator_metadata.delegation_count -= 1;
				}

				Ok(())
			})?;

			Ok(())
		})
	}

	/// Executes scheduled stake reductions for a delegator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	///
	/// # Errors
	///
	/// Returns an error if the delegator has no unstake requests or if none of the unstake requests
	/// are ready.
	pub fn process_execute_delegator_unstake(who: T::AccountId) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Ensure there are outstanding unstake requests
			ensure!(!metadata.delegator_unstake_requests.is_empty(), Error::<T>::NoBondLessRequest);

			let current_round = Self::current_round();

			// Process all ready unstake requests
			let mut executed_requests = Vec::new();
			metadata.delegator_unstake_requests.retain(|request| {
				let delay = T::DelegationBondLessDelay::get();
				if current_round >= delay + request.requested_round {
					// Add the amount back to the delegator's deposits
					metadata
						.deposits
						.entry(request.asset_id)
						.and_modify(|e| *e += request.amount)
						.or_insert(request.amount);
					executed_requests.push(request.clone());
					false // Remove this request
				} else {
					true // Keep this request
				}
			});

			// If no requests were executed, return an error
			ensure!(!executed_requests.is_empty(), Error::<T>::BondLessNotReady);

			Ok(())
		})
	}

	/// Cancels a scheduled stake reduction for a delegator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	/// * `asset_id` - The ID of the asset for which to cancel the unstake request.
	/// * `amount` - The amount of the unstake request to cancel.
	///
	/// # Errors
	///
	/// Returns an error if the delegator has no matching unstake request or if there is no active
	/// delegation.
	pub fn process_cancel_delegator_unstake(
		who: T::AccountId,
		operator: T::AccountId,
		asset_id: Asset<T::AssetId>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Find and remove the matching unstake request
			let request_index = metadata
				.delegator_unstake_requests
				.iter()
				.position(|r| {
					r.asset_id == asset_id && r.amount == amount && r.operator == operator
				})
				.ok_or(Error::<T>::NoBondLessRequest)?;

			let unstake_request = metadata.delegator_unstake_requests.remove(request_index);

			// Update the operator's metadata
			Operators::<T>::try_mutate(
				&unstake_request.operator,
				|maybe_operator_metadata| -> DispatchResult {
					let operator_metadata =
						maybe_operator_metadata.as_mut().ok_or(Error::<T>::NotAnOperator)?;

					// Find the matching delegation and increase its amount, or insert a new
					// delegation if not found
					let mut delegations = operator_metadata.delegations.clone();
					if let Some(delegation) = delegations
						.iter_mut()
						.find(|d| d.asset_id == asset_id && d.delegator == who.clone())
					{
						delegation.amount += amount;
					} else {
						delegations
							.try_push(DelegatorBond { delegator: who.clone(), amount, asset_id })
							.map_err(|_| Error::<T>::MaxDelegationsExceeded)?;

						// Increase the delegation count only when a new delegation is added
						operator_metadata.delegation_count += 1;
					}
					operator_metadata.delegations = delegations;

					Ok(())
				},
			)?;

			// Update the delegator's metadata
			let mut delegations = metadata.delegations.clone();

			// If a similar delegation exists, increase the amount
			if let Some(delegation) = delegations.iter_mut().find(|d| {
				d.operator == unstake_request.operator && d.asset_id == unstake_request.asset_id
			}) {
				delegation.amount += unstake_request.amount;
			} else {
				// Create a new delegation
				delegations
					.try_push(BondInfoDelegator {
						operator: unstake_request.operator.clone(),
						amount: unstake_request.amount,
						asset_id: unstake_request.asset_id,
						blueprint_selection: unstake_request.blueprint_selection,
					})
					.map_err(|_| Error::<T>::MaxDelegationsExceeded)?;
			}
			metadata.delegations = delegations;

			Ok(())
		})
	}

	/// Slashes a delegator's stake.
	///
	/// # Arguments
	///
	/// * `delegator` - The account ID of the delegator.
	/// * `operator` - The account ID of the operator.
	/// * `blueprint_id` - The ID of the blueprint.
	/// * `percentage` - The percentage of the stake to slash.
	///
	/// # Errors
	///
	/// Returns an error if the delegator is not found, or if the delegation is not active.
	pub fn slash_delegator(
		delegator: &T::AccountId,
		operator: &T::AccountId,
		blueprint_id: BlueprintId,
		percentage: Percent,
	) -> Result<(), DispatchError> {
		Delegators::<T>::try_mutate(delegator, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			let delegation = metadata
				.delegations
				.iter_mut()
				.find(|d| &d.operator == operator)
				.ok_or(Error::<T>::NoActiveDelegation)?;

			// Check delegation type and blueprint_id
			match &delegation.blueprint_selection {
				DelegatorBlueprintSelection::Fixed(blueprints) => {
					// For fixed delegation, ensure the blueprint_id is in the list
					ensure!(blueprints.contains(&blueprint_id), Error::<T>::BlueprintNotSelected);
				},
				DelegatorBlueprintSelection::All => {
					// For "All" type, no need to check blueprint_id
				},
			}

			// Calculate and apply slash
			let slash_amount = percentage.mul_floor(delegation.amount);
			delegation.amount = delegation
				.amount
				.checked_sub(&slash_amount)
				.ok_or(Error::<T>::InsufficientStakeRemaining)?;

			// Transfer slashed amount to the treasury
			let _ = T::Fungibles::transfer(
				delegation.asset_id,
				&Self::pallet_account(),
				&T::SlashedAmountRecipient::get(),
				slash_amount,
				Preservation::Expendable,
			);

			// emit event
			Self::deposit_event(Event::DelegatorSlashed {
				who: delegator.clone(),
				amount: slash_amount,
			});

			Ok(())
		})
	}
}
