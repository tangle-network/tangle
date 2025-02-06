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
use tangle_primitives::{
	services::{Asset, UnappliedSlash},
	traits::SlashManager,
};

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
		operator: &T::AccountId,
		delegator: &T::AccountId,
		asset_id: Asset<T::AssetId>,
		slash_amount: BalanceOf<T>,
	) -> Result<Weight, DispatchError> {
		let mut weight = T::DbWeight::get().reads(1);

		Delegators::<T>::try_mutate(delegator, |maybe_metadata| -> DispatchResult {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Find the delegation to the slashed operator
			let delegation = metadata
				.delegations
				.iter_mut()
				.find(|d| &d.operator == operator && d.asset_id == asset_id)
				.ok_or(Error::<T>::NoActiveDelegation)?;

			// Update delegator's stake
			delegation.amount = delegation
				.amount
				.checked_sub(&slash_amount)
				.ok_or(Error::<T>::InsufficientStakeRemaining)?;

			weight += T::DbWeight::get().writes(1);
			Ok(())
		})?;

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
		let mut total_weight = Self::do_slash_operator(unapplied_slash)?;

		// Also slash all delegators in the unapplied_slash.others list
		for (delegator, asset_id, amount) in &unapplied_slash.others {
			total_weight = total_weight.saturating_add(Self::do_slash_delegator(
				&unapplied_slash.operator,
				delegator,
				*asset_id,
				*amount,
			)?);
		}

		Ok(total_weight)
	}
}
