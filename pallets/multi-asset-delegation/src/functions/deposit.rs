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
use frame_support::traits::fungibles::Mutate;
use frame_support::{ensure, pallet_prelude::DispatchResult};
use frame_support::{
	sp_runtime::traits::{AccountIdConversion, CheckedAdd, Zero},
	traits::{tokens::Preservation, Get},
};
use sp_core::H160;
use tangle_primitives::services::Asset;
use tangle_primitives::EvmAddressMapping;

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
		evm_sender: Option<H160>,
	) -> DispatchResult {
		match asset_id {
			Asset::Custom(asset_id) => {
				T::Fungibles::transfer(
					asset_id,
					&sender,
					&Self::pallet_account(),
					amount,
					Preservation::Expendable,
				);
			},
			Asset::Erc20(asset_address) => {
				let sender = evm_sender.ok_or(Error::<T>::ERC20TransferFailed)?;
				let (success, _weight) = Self::erc20_transfer(
					asset_address,
					&sender,
					Self::pallet_evm_account(),
					amount,
				)
				.map_err(|e| Error::<T>::ERC20TransferFailed)?;
				ensure!(success, Error::<T>::ERC20TransferFailed);
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
		evm_address: Option<H160>,
	) -> DispatchResult {
		ensure!(amount >= T::MinDelegateAmount::get(), Error::<T>::BondTooLow);

		// Transfer the amount to the pallet account
		Self::handle_transfer_to_pallet(&who, asset_id, amount, evm_address)?;

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
		asset_id: Asset<T::AssetId>,
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
			let mut withdraw_requests = metadata.withdraw_requests.clone();
			withdraw_requests
				.try_push(WithdrawRequest { asset_id, amount, requested_round: current_round })
				.map_err(|_| Error::<T>::MaxWithdrawRequestsExceeded)?;
			metadata.withdraw_requests = withdraw_requests;

			Ok(())
		})
	}

	/// Executes an withdraw request for a delegator.
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
			metadata.withdraw_requests.retain(|request| {
				if current_round >= delay + request.requested_round {
					match request.asset_id {
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
					}
				} else {
					true
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
		asset_id: Asset<T::AssetId>,
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
