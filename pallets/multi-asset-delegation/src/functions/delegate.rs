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
use crate::types::*;
use crate::Pallet;
use frame_support::ensure;
use frame_support::pallet_prelude::DispatchResult;
use frame_support::traits::Get;
use sp_runtime::traits::Zero;
use sp_std::vec::Vec;

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
		asset_id: T::AssetId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Ensure enough deposited balance
			let balance =
				metadata.deposits.get_mut(&asset_id).ok_or(Error::<T>::InsufficientBalance)?;
			ensure!(*balance >= amount, Error::<T>::InsufficientBalance);

			// Reduce the balance in deposits
			*balance -= amount;
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
				metadata.delegations.push(BondInfoDelegator {
					operator: operator.clone(),
					amount,
					asset_id,
				});
			}

			// Update the status
			metadata.status = DelegatorStatus::Active;

			// Update the operator's metadata
			Operators::<T>::try_mutate(&operator, |maybe_operator_metadata| -> DispatchResult {
				let operator_metadata =
					maybe_operator_metadata.as_mut().ok_or(Error::<T>::NotAnOperator)?;

				// Check if the delegation exists and update it, otherwise create a new delegation
				if let Some(delegation) = operator_metadata
					.delegations
					.iter_mut()
					.find(|d| d.delegator == who && d.asset_id == asset_id)
				{
					delegation.amount += amount;
				} else {
					operator_metadata.delegations.push(DelegatorBond {
						delegator: who.clone(),
						amount,
						asset_id,
					});
					// Increase the delegation count only when a new delegation is added
					operator_metadata.delegation_count += 1;
				}

				Ok(())
			})?;

			Ok(())
		})
	}

	/// Schedules a bond reduction for a delegator.
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
	/// or if the bond less amount is greater than the current delegation amount.
	pub fn process_schedule_delegator_unstake(
		who: T::AccountId,
		operator: T::AccountId,
		asset_id: T::AssetId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Ensure the delegator has an active delegation with the operator for the given asset
			let delegation = metadata
				.delegations
				.iter()
				.find(|d| d.operator == operator && d.asset_id == asset_id)
				.ok_or(Error::<T>::NoActiveDelegation)?;

			// Ensure the amount to bond less is not greater than the current delegation amount
			ensure!(delegation.amount >= amount, Error::<T>::InsufficientBalance);

			// Create the bond less request
			let current_round = Self::current_round();
			metadata.delegator_unstake_requests.push(BondLessRequest {
				operator: delegation.operator.clone(),
				asset_id,
				amount,
				requested_round: current_round,
			});

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

	/// Executes scheduled bond reductions for a delegator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	///
	/// # Errors
	///
	/// Returns an error if the delegator has no bond less requests or if none of the bond less requests are ready.
	pub fn process_execute_delegator_unstake(who: T::AccountId) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Ensure there are outstanding bond less requests
			ensure!(!metadata.delegator_unstake_requests.is_empty(), Error::<T>::NoBondLessRequest);

			let current_round = Self::current_round();
			let delay = T::DelegationBondLessDelay::get();

			// Process all ready bond less requests
			let mut executed_requests = Vec::new();
			metadata.delegator_unstake_requests.retain(|request| {
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

	/// Cancels a scheduled bond reduction for a delegator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	/// * `asset_id` - The ID of the asset for which to cancel the bond less request.
	/// * `amount` - The amount of the bond less request to cancel.
	///
	/// # Errors
	///
	/// Returns an error if the delegator has no matching bond less request or if there is no active delegation.
	pub fn process_cancel_delegator_unstake(
		who: T::AccountId,
		asset_id: T::AssetId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Find and remove the matching bond less request
			let request_index = metadata
				.delegator_unstake_requests
				.iter()
				.position(|r| r.asset_id == asset_id && r.amount == amount)
				.ok_or(Error::<T>::NoBondLessRequest)?;

			let unstake_request = metadata.delegator_unstake_requests.remove(request_index);

			// Update the operator's metadata
			Operators::<T>::try_mutate(
				&unstake_request.operator,
				|maybe_operator_metadata| -> DispatchResult {
					let operator_metadata =
						maybe_operator_metadata.as_mut().ok_or(Error::<T>::NotAnOperator)?;

					// Find the matching delegation and increase its amount, or insert a new delegation if not found
					if let Some(delegation) = operator_metadata
						.delegations
						.iter_mut()
						.find(|d| d.asset_id == asset_id && d.delegator == who.clone())
					{
						delegation.amount += amount;
					} else {
						operator_metadata.delegations.push(DelegatorBond {
							delegator: who.clone(),
							amount,
							asset_id,
						});

						// Increase the delegation count
						operator_metadata.delegation_count += 1;
					}

					Ok(())
				},
			)?;

			// Create a new delegation
			metadata.delegations.push(BondInfoDelegator {
				operator: unstake_request.operator,
				amount,
				asset_id,
			});

			Ok(())
		})
	}
}
