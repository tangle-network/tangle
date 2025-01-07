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
use frame_support::{
	ensure,
	pallet_prelude::DispatchResult,
	sp_runtime::traits::AccountIdConversion,
	traits::{fungibles::Mutate, tokens::Preservation, Get},
};
use sp_core::H160;
use tangle_primitives::services::{Asset, EvmAddressMapping};
use tangle_primitives::types::rewards::LockMultiplier;

impl<T: Config> Pallet<T> {
	/// Returns the account ID of the pallet.
	pub fn pallet_account() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	/// Returns the EVM account id of the pallet.
	///
	/// This function retrieves the account id associated with the pallet by converting
	/// the pallet evm address to an account id.
	///
	/// # Returns
	/// * `T::AccountId` - The account id of the pallet.
	pub fn pallet_evm_account() -> H160 {
		T::EvmAddressMapping::into_address(Self::pallet_account())
	}

	pub fn handle_transfer_to_pallet(
		sender: &T::AccountId,
		asset_id: Asset<T::AssetId>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		match asset_id {
			Asset::Custom(asset_id) => {
				T::Fungibles::transfer(
					asset_id,
					sender,
					&Self::pallet_account(),
					amount,
					Preservation::Expendable,
				)?;
			},
			Asset::Erc20(_) => {
				// Handled by the Precompile
			},
		}
		Ok(())
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
		asset_id: Asset<T::AssetId>,
		amount: BalanceOf<T>,
		lock_multiplier: Option<LockMultiplier>,
	) -> DispatchResult {
		ensure!(amount >= T::MinDelegateAmount::get(), Error::<T>::BondTooLow);

		// Transfer the amount to the pallet account
		Self::handle_transfer_to_pallet(&who, asset_id, amount)?;

		let now = <frame_system::Pallet<T>>::block_number();

		// Update storage
		Delegators::<T>::try_mutate(&who, |maybe_metadata| -> DispatchResult {
			let metadata = maybe_metadata.get_or_insert_with(Default::default);
			// If there's an existing deposit, increase it
			if let Some(existing) = metadata.deposits.get_mut(&asset_id) {
				existing
					.increase_deposited_amount(amount, lock_multiplier, now)
					.map_err(|_| Error::<T>::InsufficientBalance)?;
			} else {
				// Create a new deposit if none exists
				let new_deposit = Deposit::new(amount, lock_multiplier, now);
				metadata.deposits.insert(asset_id, new_deposit);
			}
			Ok(())
		})?;

		Ok(())
	}

	/// Schedules a withdraw request for a delegator.
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
		asset_id: Asset<T::AssetId>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			let now = <frame_system::Pallet<T>>::block_number();

			// Ensure there is enough deposited balance
			let deposit =
				metadata.deposits.get_mut(&asset_id).ok_or(Error::<T>::InsufficientBalance)?;
			deposit
				.decrease_deposited_amount(amount, now)
				.map_err(|_| Error::<T>::InsufficientBalance)?;

			// Create the unstake request
			let current_round = Self::current_round();
			let mut withdraw_requests = metadata.withdraw_requests.clone();
			withdraw_requests
				.try_push(WithdrawRequest { asset_id, amount, requested_round: current_round })
				.map_err(|_| Error::<T>::MaxWithdrawRequestsExceeded)?;
			metadata.withdraw_requests = withdraw_requests;

			Ok(())
		})
	}

	/// Executes a withdraw request for a delegator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the delegator.
	///
	/// # Errors
	///
	/// Returns an error if the user is not a delegator, if there are no withdraw requests, or if
	/// the withdraw request is not ready.
	pub fn process_execute_withdraw(
		who: T::AccountId,
		evm_address: Option<H160>,
	) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Ensure there are outstanding withdraw requests
			ensure!(!metadata.withdraw_requests.is_empty(), Error::<T>::NowithdrawRequests);

			let current_round = Self::current_round();
			let delay = T::LeaveDelegatorsDelay::get();

			// Process all ready withdraw requests
			let mut i = 0;
			while i < metadata.withdraw_requests.len() {
				let request = &metadata.withdraw_requests[i];
				if current_round >= delay + request.requested_round {
					let transfer_success = match request.asset_id {
						Asset::Custom(asset_id) => T::Fungibles::transfer(
							asset_id,
							&Self::pallet_account(),
							&who,
							request.amount,
							Preservation::Expendable,
						)
						.is_ok(),
						Asset::Erc20(asset_address) => {
							if let Some(evm_addr) = evm_address {
								if let Ok((success, _weight)) = Self::erc20_transfer(
									asset_address,
									&Self::pallet_evm_account(),
									evm_addr,
									request.amount,
								) {
									success
								} else {
									false
								}
							} else {
								false
							}
						},
					};

					if transfer_success {
						// Remove the completed request
						metadata.withdraw_requests.remove(i);
					} else {
						// Only increment if we didn't remove the request
						i += 1;
					}
				} else {
					i += 1;
				}
			}

			Ok(())
		})
	}

	/// Cancels a withdraw request for a delegator.
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
		asset_id: Asset<T::AssetId>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;
			let now = <frame_system::Pallet<T>>::block_number();

			// Find and remove the matching withdraw request
			let request_index = metadata
				.withdraw_requests
				.iter()
				.position(|r| r.asset_id == asset_id && r.amount == amount)
				.ok_or(Error::<T>::NoMatchingwithdrawRequest)?;

			let withdraw_request = metadata.withdraw_requests.remove(request_index);

			// Add the amount back to the delegator's deposits
			if let Some(deposit) = metadata.deposits.get_mut(&withdraw_request.asset_id) {
				deposit
					.increase_deposited_amount(withdraw_request.amount, None, now)
					.map_err(|_| Error::<T>::InsufficientBalance)?;
			} else {
				// we are only able to withdraw from existing deposits without any locks
				// so when we add back, add it without any locks
				let new_deposit = Deposit::new(withdraw_request.amount, None, now);
				metadata.deposits.insert(withdraw_request.asset_id, new_deposit);
			}

			// Update the status if no more delegations exist
			if metadata.delegations.is_empty() {
				metadata.status = DelegatorStatus::Active;
			}

			Ok(())
		})
	}
}
