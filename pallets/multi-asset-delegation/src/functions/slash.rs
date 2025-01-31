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

use crate::{types::*, Config, Delegators, Error, Event, Operators, Pallet};
use frame_support::{
	ensure,
	traits::{
		fungibles::Mutate, tokens::Preservation, Currency, ExistenceRequirement, Get,
		ReservableCurrency,
	},
};
use sp_runtime::{traits::CheckedSub, DispatchError, Percent};
use tangle_primitives::{
	services::{Asset, EvmAddressMapping},
	BlueprintId, InstanceId,
};

impl<T: Config> Pallet<T> {
	pub fn slash_operator(
		operator: &T::AccountId,
		blueprint_id: BlueprintId,
		service_id: InstanceId,
		percentage: Percent,
	) -> Result<(), DispatchError> {
		Operators::<T>::try_mutate(operator, |maybe_operator| {
			let operator_data = maybe_operator.as_mut().ok_or(Error::<T>::NotAnOperator)?;
			ensure!(operator_data.status == OperatorStatus::Active, Error::<T>::NotActiveOperator);

			// Slash operator stake
			let amount = percentage.mul_floor(operator_data.stake);
			operator_data.stake = operator_data
				.stake
				.checked_sub(&amount)
				.ok_or(Error::<T>::InsufficientStakeRemaining)?;

			// Slash each delegator
			for delegator in operator_data.delegations.iter() {
				// Ignore errors from individual delegator slashing
				let _ =
					Self::slash_delegator(&delegator.delegator, operator, blueprint_id, percentage);
			}

			// transfer the slashed amount to the treasury
			T::Currency::unreserve(operator, amount);
			let _ = T::Currency::transfer(
				operator,
				&T::SlashedAmountRecipient::get(),
				amount,
				ExistenceRequirement::AllowDeath,
			);

			// emit event
			Self::deposit_event(Event::OperatorSlashed { who: operator.clone(), amount });

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

			// Sanity check the delegation type and blueprint_id. This shouldn't
			// ever fail, since `slash_delegator` is called from the service module,
			// which checks the blueprint_id before calling this function,
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

			// emit event
			Self::deposit_event(Event::DelegatorSlashed {
				who: delegator.clone(),
				amount: slash_amount,
			});

			Ok(())
		})
	}
}
