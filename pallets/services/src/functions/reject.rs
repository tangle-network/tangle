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
use crate::{Config, Error, Event, Pallet, ServiceRequests, StagingServicePayments};
use frame_support::{
	pallet_prelude::*,
	traits::{fungibles::Mutate, tokens::Preservation, Currency, ExistenceRequirement},
};
use sp_runtime::traits::Zero;
use tangle_primitives::services::{ApprovalState, Asset};

impl<T: Config> Pallet<T> {
	/// Process a rejection of a service request by an operator.
	///
	/// This function handles the rejection workflow including:
	/// - Updating the operator's approval state to rejected
	/// - Refunding any staged payments
	/// - Emitting appropriate events
	///
	/// # Arguments
	///
	/// * `operator` - The account ID of the operator rejecting the request
	/// * `request_id` - The ID of the service request being rejected
	///
	/// # Returns
	///
	/// Returns a DispatchResult indicating success or the specific error that occurred
	pub fn do_reject(operator: T::AccountId, request_id: u64) -> DispatchResult {
		let mut request = Self::service_requests(request_id)?;

		// First check if the operator exists and what their current state is
		let operator_index =
			request.operators_with_approval_state.iter().position(|(v, _)| v == &operator);

		// Make sure operator is listed in the request
		let operator_index = operator_index.ok_or(Error::<T>::ApprovalNotRequested)?;

		// Check if the operator has already rejected the request
		if matches!(
			request.operators_with_approval_state[operator_index].1,
			ApprovalState::Rejected
		) {
			return Err(Error::<T>::ApprovalNotRequested.into());
		}

		// Set the operator's state to Rejected
		request.operators_with_approval_state[operator_index].1 = ApprovalState::Rejected;

		let blueprint_id = request.blueprint;
		let (_, blueprint) = Self::blueprints(blueprint_id)?;
		let prefs = Self::operators(blueprint_id, operator.clone())?;

		let (allowed, _weight) = Self::on_reject_hook(&blueprint, blueprint_id, &prefs, request_id)
			.map_err(|_| Error::<T>::OnRejectFailure)?;
		ensure!(allowed, Error::<T>::RejectionInterrupted);

		Self::deposit_event(Event::ServiceRequestRejected {
			operator,
			blueprint_id: request.blueprint,
			request_id,
		});

		// Refund the payment if it exists
		if let Some(payment) = Self::service_payment(request_id) {
			match payment.asset {
				Asset::Custom(asset_id) if asset_id == Zero::zero() => {
					// For native currency, we expect an AccountId
					let refund_to = payment
						.refund_to
						.try_into_account_id()
						.map_err(|_| Error::<T>::ExpectedAccountId)?;
					T::Currency::transfer(
						&Self::pallet_account(),
						&refund_to,
						payment.amount,
						ExistenceRequirement::AllowDeath,
					)?;
				},
				Asset::Custom(asset_id) => {
					let refund_to = payment
						.refund_to
						.try_into_account_id()
						.map_err(|_| Error::<T>::ExpectedAccountId)?;
					T::Fungibles::transfer(
						asset_id,
						&Self::pallet_account(),
						&refund_to,
						payment.amount,
						Preservation::Expendable,
					)?;
				},
				Asset::Erc20(token) => {
					let refund_to = payment
						.refund_to
						.try_into_address()
						.map_err(|_| Error::<T>::ExpectedEVMAddress)?;
					let (success, _weight) = Self::erc20_transfer(
						token,
						Self::pallet_evm_account(),
						refund_to,
						payment.amount,
					)
					.map_err(|_| Error::<T>::ERC20TransferFailed)?;
					ensure!(success, Error::<T>::ERC20TransferFailed);
				},
			}
			StagingServicePayments::<T>::remove(request_id);
		}

		// Store the updated request back to storage
		ServiceRequests::<T>::insert(request_id, request);

		Ok(())
	}
}
