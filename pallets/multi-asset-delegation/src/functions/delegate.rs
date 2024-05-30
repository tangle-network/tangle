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
use frame_support::traits::ReservableCurrency;
use sp_runtime::traits::Zero;
use sp_runtime::DispatchError;

impl<T: Config> Pallet<T> {
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
			metadata.delegations.push(Bond { operator, amount, asset_id });

			// Update the status
			metadata.status = DelegatorStatus::Active;

			Ok(())
		})
	}

	pub fn process_schedule_delegator_bond_less(
		who: T::AccountId,
		asset_id: T::AssetId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Ensure no active services on this operator using this asset
			ensure!(
				metadata.delegations.iter().all(|d| d.asset_id != asset_id),
				Error::<T>::ActiveServicesUsingAsset
			);

			// Ensure no outstanding bond less request
			ensure!(
				metadata.delegator_bond_less_request.is_none(),
				Error::<T>::BondLessRequestAlreadyExists
			);

			// Create the bond less request
			let current_round = Self::current_round();
			metadata.delegator_bond_less_request =
				Some(BondLessRequest { asset_id, amount, requested_round: current_round });

			Ok(())
		})
	}

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
				Self::current_round() >= bond_less_request.requested_round,
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

			// Add the amount back to the delegator's deposits
			metadata.deposits.entry(asset_id).and_modify(|e| *e += amount).or_insert(amount);

			Ok(())
		})
	}
}
