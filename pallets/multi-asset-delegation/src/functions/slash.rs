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
use frame_support::{dispatch::DispatchResult, ensure, traits::Get, weights::Weight};
use sp_runtime::{traits::CheckedSub, DispatchError};
use tangle_primitives::{services::UnappliedSlash, traits::SlashManager};

impl<T: Config> Pallet<T> {
	/// Helper function to update operator storage for a slash
	pub(crate) fn do_slash_operator(
		unapplied_slash: &UnappliedSlash<T::AccountId, BalanceOf<T>, T::AssetId>,
	) -> Result<Weight, DispatchError> {
		let mut weight = T::DbWeight::get().reads(1);

		Operators::<T>::try_mutate(
			&unapplied_slash.operator,
			|maybe_operator| -> DispatchResult {
				let operator_data = maybe_operator.as_mut().ok_or(Error::<T>::NotAnOperator)?;
				ensure!(
					operator_data.status == OperatorStatus::Active,
					Error::<T>::NotActiveOperator
				);

				// Update operator's stake
				operator_data.stake = operator_data
					.stake
					.checked_sub(&unapplied_slash.own)
					.ok_or(Error::<T>::InsufficientStakeRemaining)?;

				weight += T::DbWeight::get().writes(1);
				Ok(())
			},
		)?;

		// Emit event for operator slash
		Self::deposit_event(Event::OperatorSlashed {
			who: unapplied_slash.operator.clone(),
			amount: unapplied_slash.own,
		});

		Ok(weight)
	}

	/// Helper function to update delegator storage for a slash
	pub(crate) fn do_slash_delegator(
		unapplied_slash: &UnappliedSlash<T::AccountId, BalanceOf<T>, T::AssetId>,
		delegator: &T::AccountId,
	) -> Result<Weight, DispatchError> {
		let mut weight = T::DbWeight::get().reads(1);

		let slash_amount = Delegators::<T>::try_mutate(
			delegator,
			|maybe_metadata| -> Result<BalanceOf<T>, DispatchError> {
				let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

				// Find the delegation to the slashed operator
				let delegation = metadata
					.delegations
					.iter_mut()
					.find(|d| &d.operator == &unapplied_slash.operator)
					.ok_or(Error::<T>::NoActiveDelegation)?;

				// Find the slash amount for this delegator from the unapplied slash
				let slash_amount = unapplied_slash
					.others
					.iter()
					.find(|(d, _, _)| d == delegator)
					.map(|(_, _, amount)| *amount)
					.ok_or(Error::<T>::NoActiveDelegation)?;

				// Update delegator's stake
				delegation.amount = delegation
					.amount
					.checked_sub(&slash_amount)
					.ok_or(Error::<T>::InsufficientStakeRemaining)?;

				weight += T::DbWeight::get().writes(1);
				Ok(slash_amount)
			},
		)?;

		// Emit event for delegator slash
		Self::deposit_event(Event::DelegatorSlashed {
			who: delegator.clone(),
			amount: slash_amount,
		});

		Ok(weight)
	}
}

impl<T: Config> SlashManager<T::AccountId, BalanceOf<T>, T::AssetId> for Pallet<T> {
	/// Updates operator storage to reflect a slash.
	/// This only updates the storage items and does not handle asset transfers.
	///
	/// # Arguments
	/// * `unapplied_slash` - The unapplied slash record containing slash details
	fn slash_operator(
		unapplied_slash: &UnappliedSlash<T::AccountId, BalanceOf<T>, T::AssetId>,
	) -> Result<Weight, DispatchError> {
		Self::do_slash_operator(unapplied_slash)
	}

	/// Updates delegator storage to reflect a slash.
	/// This only updates the storage items and does not handle asset transfers.
	///
	/// # Arguments
	/// * `unapplied_slash` - The unapplied slash record containing slash details
	/// * `delegator` - The account of the delegator being slashed
	fn slash_delegator(
		unapplied_slash: &UnappliedSlash<T::AccountId, BalanceOf<T>, T::AssetId>,
		delegator: &T::AccountId,
	) -> Result<Weight, DispatchError> {
		Self::do_slash_delegator(unapplied_slash, delegator)
	}
}
