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

			// Create a new delegation
			metadata.delegations.push(BondInfoDelegator {
				operator: operator.clone(),
				amount,
				asset_id,
			});

			// Update the status
			metadata.status = DelegatorStatus::Active;

			// Update the operator's metadata
			Operators::<T>::try_mutate(&operator, |maybe_operator_metadata| -> DispatchResult {
				let operator_metadata =
					maybe_operator_metadata.as_mut().ok_or(Error::<T>::NotAnOperator)?;

				// Increase the delegation count
				operator_metadata.delegation_count += 1;

				// Add the new delegation
				operator_metadata.delegations.push(DelegatorBond {
					delegator: who.clone(),
					amount,
					asset_id,
				});

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
	/// if there is an existing bond less request, or if the bond less amount is greater than the current delegation amount.
	pub fn process_schedule_delegator_bond_less(
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

			// Ensure there is no outstanding bond less request
			ensure!(
				metadata.delegator_bond_less_request.is_none(),
				Error::<T>::BondLessRequestAlreadyExists
			);

			// Ensure the amount to bond less is not greater than the current delegation amount
			ensure!(delegation.amount >= amount, Error::<T>::InsufficientBalance);

			// Create the bond less request
			let current_round = Self::current_round();
			metadata.delegator_bond_less_request =
				Some(BondLessRequest { asset_id, amount, requested_round: current_round });

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

	/// Executes a scheduled bond reduction for a delegator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	///
	/// # Errors
	///
	/// Returns an error if the delegator has no bond less request or if the bond less request is not ready.
	pub fn process_execute_delegator_bond_less(who: T::AccountId) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Ensure there is an outstanding bond less request
			let bond_less_request = metadata
				.delegator_bond_less_request
				.as_ref()
				.ok_or(Error::<T>::NoBondLessRequest)?;

			// Check if the requested round has been reached
			ensure!(
				Self::current_round()
					>= T::DelegationBondLessDelay::get() + bond_less_request.requested_round,
				Error::<T>::BondLessNotReady
			);

			// Get the asset ID and amount from the bond less request
			let asset_id = bond_less_request.asset_id;
			let amount = bond_less_request.amount;

			// Add the amount back to the delegator's deposits
			metadata.deposits.entry(asset_id).and_modify(|e| *e += amount).or_insert(amount);

			// Clear the bond less request
			metadata.delegator_bond_less_request = None;

			Ok(())
		})
	}

	/// Cancels a scheduled bond reduction for a delegator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	///
	/// # Errors
	///
	/// Returns an error if the delegator has no bond less request or if there is no active delegation.
	pub fn process_cancel_delegator_bond_less(who: T::AccountId) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Ensure there is an outstanding bond less request
			let bond_less_request = metadata
				.delegator_bond_less_request
				.take()
				.ok_or(Error::<T>::NoBondLessRequest)?;

			// Get the asset ID and amount from the bond less request
			let asset_id = bond_less_request.asset_id;
			let amount = bond_less_request.amount;

			// Find the operator associated with the bond less request
			let operator = metadata
				.delegations
				.iter()
				.find(|d| d.asset_id == asset_id && d.amount >= amount)
				.ok_or(Error::<T>::NoActiveDelegation)?
				.operator
				.clone();

			// Add the amount back to the delegator's deposits
			metadata.deposits.entry(asset_id).and_modify(|e| *e += amount).or_insert(amount);

			// Update the operator's metadata
			Operators::<T>::try_mutate(&operator, |maybe_operator_metadata| -> DispatchResult {
				let operator_metadata =
					maybe_operator_metadata.as_mut().ok_or(Error::<T>::NotAnOperator)?;

				// Increase the delegation count
				operator_metadata.delegation_count += 1;

				// Add the new delegation
				operator_metadata.delegations.push(DelegatorBond {
					delegator: who.clone(),
					amount,
					asset_id,
				});

				Ok(())
			})?;

			Ok(())
		})
	}
}
