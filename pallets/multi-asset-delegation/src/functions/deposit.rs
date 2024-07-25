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
use frame_support::{ensure, pallet_prelude::DispatchResult};

use frame_support::traits::fungibles::Mutate;

use frame_support::{
	sp_runtime::traits::AccountIdConversion,
	traits::{tokens::Preservation, Get},
};
use sp_runtime::traits::Zero;

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
	/// Returns an error if the user is already a delegator, if the bond amount is too low, or if
	/// the transfer fails.
	pub fn process_deposit(
		who: T::AccountId,
		asset_id: Option<T::AssetId>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		ensure!(amount >= T::MinDelegateAmount::get(), Error::<T>::BondTooLow);

		// Transfer the amount to the pallet account
		if let Some(asset_id) = asset_id {
			// ensure the asset is whitelisted
			ensure!(
				WhitelistedAssets::<T>::get().contains(&asset_id),
				Error::<T>::AssetNotWhitelisted
			);

			T::Fungibles::transfer(
				asset_id,
				&who,
				&Self::pallet_account(),
				amount,
				Preservation::Expendable,
			)?; // Transfer the assets to the pallet account

			// Update storage
			Delegators::<T>::mutate(&who, |maybe_metadata| {
				let metadata = maybe_metadata.get_or_insert_with(Default::default);
				metadata.deposits.entry(asset_id).and_modify(|e| *e += amount).or_insert(amount);
			});
		} else {
			// TODO : handle if TNT deposit
			todo!();
		}

		Ok(())
	}

	/// Schedules an unstake request for a delegator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	/// * `asset_id` - The optional asset ID of the assets to be unstaked.
	/// * `amount` - The amount of assets to be unstaked.
	///
	/// # Errors
	///
	/// Returns an error if the user is not a delegator, if there is an existing unstake request,
	/// if there is insufficient balance, or if the asset is not supported.
	pub fn process_schedule_unstake(
		who: T::AccountId,
		asset_id: Option<T::AssetId>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			if asset_id.is_none() {
				todo!(); // Handle TNT deposit
			}

			let asset_id = asset_id.unwrap();

			// Ensure there is no outstanding withdraw request
			ensure!(metadata.unstake_request.is_none(), Error::<T>::WithdrawRequestAlreadyExists);

			// Ensure there is enough deposited balance
			let balance =
				metadata.deposits.get_mut(&asset_id).ok_or(Error::<T>::InsufficientBalance)?;
			ensure!(*balance >= amount, Error::<T>::InsufficientBalance);

			// Reduce the balance in deposits
			*balance -= amount;
			if *balance == Zero::zero() {
				metadata.deposits.remove(&asset_id);
			}

			// Create the withdraw request
			let current_round = Self::current_round();
			metadata.unstake_request =
				Some(UnstakeRequest { asset_id, amount, requested_round: current_round });

			Ok(())
		})
	}

	/// Executes an unstake request for a delegator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	///
	/// # Errors
	///
	/// Returns an error if the user is not a delegator, if there is no unstake request, or if the
	/// unstake request is not ready.
	pub fn process_execute_unstake(who: T::AccountId) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Ensure there is an outstanding withdraw request
			let unstake_request =
				metadata.unstake_request.as_ref().ok_or(Error::<T>::NoWithdrawRequest)?;

			// Check if the requested round has been reached
			ensure!(
				Self::current_round() >=
					T::LeaveDelegatorsDelay::get() + unstake_request.requested_round,
				Error::<T>::UnstakeNotReady
			);

			// Get the asset ID and amount from the withdraw request
			let asset_id = unstake_request.asset_id;
			let amount = unstake_request.amount;

			// Transfer the amount back to the delegator
			T::Fungibles::transfer(
				asset_id,
				&Self::pallet_account(),
				&who,
				amount,
				Preservation::Expendable,
			)?;

			// Clear the withdraw request
			metadata.unstake_request = None;

			Ok(())
		})
	}

	/// Cancels an unstake request for a delegator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	///
	/// # Errors
	///
	/// Returns an error if the user is not a delegator or if there is no unstake request.
	pub fn process_cancel_unstake(who: T::AccountId) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Ensure there is an outstanding withdraw request
			let unstake_request =
				metadata.unstake_request.take().ok_or(Error::<T>::NoWithdrawRequest)?;

			// Get the asset ID and amount from the withdraw request
			let asset_id = unstake_request.asset_id;
			let amount = unstake_request.amount;

			// Add the amount back to the delegator's deposits
			metadata.deposits.entry(asset_id).and_modify(|e| *e += amount).or_insert(amount);

			// Update the status if no more delegations exist
			if metadata.delegations.is_empty() {
				metadata.status = DelegatorStatus::Active;
			}

			Ok(())
		})
	}
}
