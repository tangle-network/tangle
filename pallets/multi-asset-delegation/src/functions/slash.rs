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
	dispatch::DispatchResult,
	ensure,
	traits::{
		fungibles::Mutate, tokens::Preservation, Currency, ExistenceRequirement, Get,
		ReservableCurrency,
	},
	weights::Weight,
};
use parity_scale_codec::Encode;
use sp_runtime::{traits::CheckedSub, DispatchError};
use tangle_primitives::{
	services::{Asset, EvmAddressMapping, UnappliedSlash},
	traits::SlashManager,
};

impl<T: Config> Pallet<T> {
	/// Helper function to update operator storage for a slash
	pub(crate) fn do_slash_operator(
		unapplied_slash: &UnappliedSlash<T::AccountId>,
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
				// Slash operator stake
				let amount = unapplied_slash.slash_percent.mul_floor(operator_data.stake);
				operator_data.stake = operator_data
					.stake
					.checked_sub(&amount)
					.ok_or(Error::<T>::InsufficientStakeRemaining)?;

				// transfer the slashed amount to the treasury
				T::Currency::unreserve(&unapplied_slash.operator, amount);
				let _ = T::Currency::transfer(
					&unapplied_slash.operator,
					&T::SlashRecipient::get(),
					amount,
					ExistenceRequirement::AllowDeath,
				);

				// Emit event for operator slash
				Self::deposit_event(Event::OperatorSlashed {
					operator: unapplied_slash.operator.clone(),
					amount,
					service_id: unapplied_slash.service_id,
					blueprint_id: unapplied_slash.blueprint_id,
					era: unapplied_slash.era,
				});

				// Slash each delegator
				for delegator in operator_data.delegations.iter() {
					// Ignore errors from individual delegator slashing
					let _ = Self::do_slash_delegator(unapplied_slash, &delegator.delegator);
				}

				weight += T::DbWeight::get().writes(1);
				Ok(())
			},
		)?;

		Ok(weight)
	}

	/// Helper function to update delegator storage for a slash
	pub(crate) fn do_slash_delegator(
		unapplied_slash: &UnappliedSlash<T::AccountId>,
		delegator: &T::AccountId,
	) -> Result<Weight, DispatchError> {
		let mut weight = T::DbWeight::get().reads(1);

		Delegators::<T>::try_mutate(delegator, |maybe_metadata| -> DispatchResult {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Find the delegation to the slashed operator
			let delegation = metadata
				.delegations
				.iter_mut()
				.find(|d| {
					d.operator == unapplied_slash.operator &&
						d.blueprint_selection.contains(&unapplied_slash.blueprint_id)
				})
				.ok_or(Error::<T>::NoActiveDelegation)?;

			// Update delegator's stake
			let slash_amount = unapplied_slash.slash_percent.mul_floor(delegation.amount);
			delegation.amount = delegation
				.amount
				.checked_sub(&slash_amount)
				.ok_or(Error::<T>::InsufficientStakeRemaining)?;

			if delegation.is_nomination {
				Self::apply_nominated_delegation_slash(
					delegator,
					&unapplied_slash.operator,
					slash_amount,
				)?;
			} else {
				Self::handle_asset_transfer(delegation.asset, slash_amount)?;
			}

			match delegation.asset {
				Asset::Erc20(address) => {
					let (_, _weight) = Self::call_slash_alert(
						Self::pallet_evm_account(),
						address,
						unapplied_slash.blueprint_id,
						unapplied_slash.service_id,
						unapplied_slash.operator.encode().try_into().unwrap_or_default(),
						slash_amount,
						500_000,
					)
					.map_err(|_| Error::<T>::SlashAlertFailed)?;
					weight += _weight;
				},
				Asset::Custom(_) => {
					// No custom asset handling for now
				},
			}

			Self::deposit_event(Event::DelegatorSlashed {
				delegator: delegator.clone(),
				asset: delegation.asset,
				amount: slash_amount,
				service_id: unapplied_slash.service_id,
				blueprint_id: unapplied_slash.blueprint_id,
				era: unapplied_slash.era,
			});
			Ok(())
		})?;

		Ok(weight)
	}

	/// Apply a slash for native asset delegations (both nominated and non-nominated)
	fn apply_nominated_delegation_slash(
		_delegator: &T::AccountId,
		_operator: &T::AccountId,
		_slash_amount: BalanceOf<T>,
	) -> Result<Weight, DispatchError> {
		let weight: Weight = Weight::zero();

		// TODO: Slash the nomination

		Ok(weight)
	}

	/// Apply a slash for non-native asset delegations (custom assets and ERC20)
	fn handle_asset_transfer(
		asset: Asset<T::AssetId>,
		slash_amount: BalanceOf<T>,
	) -> Result<Weight, DispatchError> {
		let mut weight: Weight = Weight::zero();

		match asset {
			Asset::Custom(asset_id) => {
				// Transfer slashed amount to the treasury
				let _ = T::Fungibles::transfer(
					asset_id,
					&Self::pallet_account(),
					&T::SlashRecipient::get(),
					slash_amount,
					Preservation::Expendable,
				);
			},
			Asset::Erc20(address) => {
				let slashed_amount_recipient_evm =
					T::EvmAddressMapping::into_address(T::SlashRecipient::get());
				let (success, _weight) = Self::erc20_transfer(
					address,
					&Self::pallet_evm_account(),
					slashed_amount_recipient_evm,
					slash_amount,
				)
				.map_err(|_| Error::<T>::ERC20TransferFailed)?;
				ensure!(success, Error::<T>::ERC20TransferFailed);

				weight += _weight;
			},
		}

		Ok(weight)
	}
}

impl<T: Config> SlashManager<T::AccountId> for Pallet<T> {
	/// Updates operator storage to reflect a slash.
	/// This only updates the storage items and does not handle asset transfers.
	///
	/// # Arguments
	/// * `unapplied_slash` - The unapplied slash record containing slash details
	fn slash_operator(
		unapplied_slash: &UnappliedSlash<T::AccountId>,
	) -> Result<Weight, DispatchError> {
		Self::do_slash_operator(unapplied_slash)
	}
}
