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
use crate::{types::*, Pallet};
use frame_support::traits::fungibles::Mutate;
use frame_support::{ensure, pallet_prelude::DispatchResult};
use frame_support::{
	sp_runtime::traits::{AccountIdConversion, CheckedAdd, Zero},
	traits::{tokens::Preservation, Get},
};

impl<T: Config> Pallet<T> {
	/// Returns the account ID of the pallet.
	pub fn pallet_account() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	/// Processes the deposit of assets into the pallet.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	/// * `asset_id` - The optional asset ID of the assets to be deposited.
	/// * `amount` - The amount of assets to be deposited.
	///
	/// # Errors
	///
	/// Returns an error if the user is already a delegator, if the stake amount is too low, or if
	/// the transfer fails.
	pub fn process_deposit(
		who: T::AccountId,
		asset_id: T::AssetId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		ensure!(amount >= T::MinDelegateAmount::get(), Error::<T>::BondTooLow);

		// Transfer the amount to the pallet account
		T::Fungibles::transfer(
			asset_id,
			&who,
			&Self::pallet_account(),
			amount,
			Preservation::Expendable,
		)?;

		// Update storage
		Delegators::<T>::try_mutate(&who, |maybe_metadata| -> DispatchResult {
			let metadata = maybe_metadata.get_or_insert_with(Default::default);
			// Handle checked addition first to avoid ? operator in closure
			if let Some(existing) = metadata.deposits.get(&asset_id) {
				let new_amount =
					existing.checked_add(&amount).ok_or(Error::<T>::DepositOverflow)?;
				metadata.deposits.insert(asset_id, new_amount);
			} else {
				metadata.deposits.insert(asset_id, amount);
			}
			Ok(())
		})?;

		Ok(())
	}

	/// Schedules an withdraw request for a delegator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	/// * `asset_id` - The optional asset ID of the assets to be withdrawd.
	/// * `amount` - The amount of assets to be withdrawd.
	///
	/// # Errors
	///
	/// Returns an error if the user is not a delegator, if there is insufficient balance, or if the
	/// asset is not supported.
	pub fn process_schedule_withdraw(
		who: T::AccountId,
		asset_id: T::AssetId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Ensure there is enough deposited balance
			let balance =
				metadata.deposits.get_mut(&asset_id).ok_or(Error::<T>::InsufficientBalance)?;
			ensure!(*balance >= amount, Error::<T>::InsufficientBalance);

			// Reduce the balance in deposits
			*balance -= amount;
			if *balance == Zero::zero() {
				metadata.deposits.remove(&asset_id);
			}

			// Create the unstake request
			let current_round = Self::current_round();
			metadata.withdraw_requests.push(WithdrawRequest {
				asset_id,
				amount,
				requested_round: current_round,
			});

			Ok(())
		})
	}

	/// Executes an withdraw request for a delegator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	/// * `asset_id` - The asset ID of the unstake request to cancel.
	/// * `amount` - The amount of the unstake request to cancel.
	///
	/// # Errors
	///
	/// Returns an error if the user is not a delegator, if there are no withdraw requests, or if
	/// the withdraw request is not ready.
	pub fn process_execute_withdraw(who: T::AccountId) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Ensure there are outstanding withdraw requests
			ensure!(!metadata.withdraw_requests.is_empty(), Error::<T>::NowithdrawRequests);

			let current_round = Self::current_round();
			let delay = T::LeaveDelegatorsDelay::get();

			// Process all ready withdraw requests
			metadata.withdraw_requests.retain(|request| {
				if current_round >= delay + request.requested_round {
					// Transfer the amount back to the delegator
					T::Fungibles::transfer(
						request.asset_id,
						&Self::pallet_account(),
						&who,
						request.amount,
						Preservation::Expendable,
					)
					.expect("Transfer should not fail");

					false // Remove this request
				} else {
					true // Keep this request
				}
			});

			Ok(())
		})
	}

	/// Cancels an withdraw request for a delegator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	/// * `asset_id` - The asset ID of the withdraw request to cancel.
	/// * `amount` - The amount of the withdraw request to cancel.
	///
	/// # Errors
	///
	/// Returns an error if the user is not a delegator or if there is no matching withdraw request.
	pub fn process_cancel_withdraw(
		who: T::AccountId,
		asset_id: T::AssetId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Find and remove the matching withdraw request
			let request_index = metadata
				.withdraw_requests
				.iter()
				.position(|r| r.asset_id == asset_id && r.amount == amount)
				.ok_or(Error::<T>::NoMatchingwithdrawRequest)?;

			let withdraw_request = metadata.withdraw_requests.remove(request_index);

			// Add the amount back to the delegator's deposits
			metadata
				.deposits
				.entry(asset_id)
				.and_modify(|e| *e += withdraw_request.amount)
				.or_insert(withdraw_request.amount);

			// Update the status if no more delegations exist
			if metadata.delegations.is_empty() {
				metadata.status = DelegatorStatus::Active;
			}

			Ok(())
		})
	}
}
