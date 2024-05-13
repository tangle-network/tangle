// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! Auto-compounding functionality for staking rewards

use crate::pallet::{
	AddGet, AutoCompoundingDelegations as AutoCompoundingDelegationsStorage, BalanceOf, Config,
	DelegatorState, Error, Event, OperatorInfo, Pallet, Total,
};
use crate::types::{Bond, BondAdjust, Delegator};
use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_support::ensure;
use frame_support::traits::Get;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::traits::Saturating;
use sp_runtime::{BoundedVec, Percent, RuntimeDebug};
use sp_std::prelude::*;

/// Represents the auto-compounding amount for a delegation.
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, PartialOrd, Ord)]
pub struct AutoCompoundConfig<AccountId> {
	pub delegator: AccountId,
	pub value: Percent,
}

/// Represents the auto-compounding [Delegations] for `T: Config`
#[derive(Clone, Eq, PartialEq, RuntimeDebug)]
pub struct AutoCompoundDelegations<T: Config>(
	BoundedVec<
		AutoCompoundConfig<T::AccountId>,
		AddGet<T::MaxTopDelegationsPerOperator, T::MaxBottomDelegationsPerOperator>,
	>,
);

impl<T> AutoCompoundDelegations<T>
where
	T: Config,
{
	/// Creates a new instance of [AutoCompoundingDelegations] from a vector of sorted_delegations.
	/// This is used for testing purposes only.
	#[cfg(test)]
	pub fn new(
		sorted_delegations: BoundedVec<
			AutoCompoundConfig<T::AccountId>,
			AddGet<T::MaxTopDelegationsPerOperator, T::MaxBottomDelegationsPerOperator>,
		>,
	) -> Self {
		Self(sorted_delegations)
	}

	pub fn get_auto_compounding_delegation_count(operator: &T::AccountId) -> usize {
		<AutoCompoundingDelegationsStorage<T>>::decode_len(operator).unwrap_or_default()
	}

	/// Retrieves an instance of [AutoCompoundingDelegations] storage as [AutoCompoundDelegations].
	pub fn get_storage(operator: &T::AccountId) -> Self {
		Self(<AutoCompoundingDelegationsStorage<T>>::get(operator))
	}

	/// Inserts the current state to [AutoCompoundingDelegations] storage.
	pub fn set_storage(self, operator: &T::AccountId) {
		<AutoCompoundingDelegationsStorage<T>>::insert(operator, self.0)
	}

	/// Retrieves the auto-compounding value for a delegation. The `delegations_config` must be a
	/// sorted vector for binary_search to work.
	pub fn get_for_delegator(&self, delegator: &T::AccountId) -> Option<Percent> {
		match self.0.binary_search_by(|d| d.delegator.cmp(&delegator)) {
			Ok(index) => Some(self.0[index].value),
			Err(_) => None,
		}
	}

	/// Sets the auto-compounding value for a delegation. The `delegations_config` must be a sorted
	/// vector for binary_search to work.
	pub fn set_for_delegator(
		&mut self,
		delegator: T::AccountId,
		value: Percent,
	) -> Result<bool, Error<T>> {
		match self.0.binary_search_by(|d| d.delegator.cmp(&delegator)) {
			Ok(index) => {
				if self.0[index].value == value {
					Ok(false)
				} else {
					self.0[index].value = value;
					Ok(true)
				}
			},
			Err(index) => {
				self.0
					.try_insert(index, AutoCompoundConfig { delegator, value })
					.map_err(|_| Error::<T>::ExceedMaxDelegationsPerDelegator)?;
				Ok(true)
			},
		}
	}

	/// Removes the auto-compounding value for a delegation.
	/// Returns `true` if the entry was removed, `false` otherwise. The `delegations_config` must be a
	/// sorted vector for binary_search to work.
	pub fn remove_for_delegator(&mut self, delegator: &T::AccountId) -> bool {
		match self.0.binary_search_by(|d| d.delegator.cmp(&delegator)) {
			Ok(index) => {
				self.0.remove(index);
				true
			},
			Err(_) => false,
		}
	}

	/// Returns the length of the inner vector.
	pub fn len(&self) -> u32 {
		self.0.len() as u32
	}

	/// Returns a reference to the inner vector.
	#[cfg(test)]
	pub fn inner(
		&self,
	) -> &BoundedVec<
		AutoCompoundConfig<T::AccountId>,
		AddGet<T::MaxTopDelegationsPerOperator, T::MaxBottomDelegationsPerOperator>,
	> {
		&self.0
	}

	/// Converts the [AutoCompoundDelegations] into the inner vector.
	#[cfg(test)]
	pub fn into_inner(
		self,
	) -> BoundedVec<
		AutoCompoundConfig<T::AccountId>,
		AddGet<T::MaxTopDelegationsPerOperator, T::MaxBottomDelegationsPerOperator>,
	> {
		self.0
	}

	// -- pallet functions --

	/// Delegates and sets the auto-compounding config. The function skips inserting auto-compound
	/// storage and validation, if the auto-compound value is 0%.
	pub(crate) fn delegate_with_auto_compound(
		operator: T::AccountId,
		delegator: T::AccountId,
		amount: BalanceOf<T>,
		auto_compound: Percent,
		operator_delegation_count_hint: u32,
		operator_auto_compounding_delegation_count_hint: u32,
		delegation_count_hint: u32,
	) -> DispatchResultWithPostInfo {
		// check that caller can lock the amount before any changes to storage
		ensure!(
			<Pallet<T>>::get_delegator_stakable_free_balance(&delegator) >= amount,
			Error::<T>::InsufficientBalance
		);
		ensure!(amount >= T::MinDelegation::get(), Error::<T>::DelegationBelowMin);

		let mut delegator_state = if let Some(mut state) = <DelegatorState<T>>::get(&delegator) {
			// delegation after first
			ensure!(
				delegation_count_hint >= state.delegations.0.len() as u32,
				Error::<T>::TooLowDelegationCountToDelegate
			);
			ensure!(
				(state.delegations.0.len() as u32) < T::MaxDelegationsPerDelegator::get(),
				Error::<T>::ExceedMaxDelegationsPerDelegator
			);
			ensure!(
				state.add_delegation(Bond { owner: operator.clone(), amount }),
				Error::<T>::AlreadyDelegatedOperator
			);
			state
		} else {
			// first delegation
			ensure!(!<Pallet<T>>::is_operator(&delegator), Error::<T>::OperatorExists);
			Delegator::new(delegator.clone(), operator.clone(), amount)
		};
		let mut operator_state =
			<OperatorInfo<T>>::get(&operator).ok_or(Error::<T>::OperatorDNE)?;
		ensure!(
			operator_delegation_count_hint >= operator_state.delegation_count,
			Error::<T>::TooLowOperatorDelegationCountToDelegate
		);

		if !auto_compound.is_zero() {
			ensure!(
				Self::get_auto_compounding_delegation_count(&operator) as u32
					<= operator_auto_compounding_delegation_count_hint,
				<Error<T>>::TooLowOperatorAutoCompoundingDelegationCountToDelegate,
			);
		}

		// add delegation to operator
		let (delegator_position, less_total_staked) = operator_state
			.add_delegation::<T>(&operator, Bond { owner: delegator.clone(), amount })?;

		// lock delegator amount
		delegator_state.adjust_bond_lock::<T>(BondAdjust::Increase(amount))?;

		// adjust total locked,
		// only is_some if kicked the lowest bottom as a consequence of this new delegation
		let net_total_increase =
			if let Some(less) = less_total_staked { amount.saturating_sub(less) } else { amount };
		let new_total_locked = <Total<T>>::get().saturating_add(net_total_increase);

		// set auto-compound config if the percent is non-zero
		if !auto_compound.is_zero() {
			let mut auto_compounding_state = Self::get_storage(&operator);
			auto_compounding_state.set_for_delegator(delegator.clone(), auto_compound.clone())?;
			auto_compounding_state.set_storage(&operator);
		}

		<Total<T>>::put(new_total_locked);
		<OperatorInfo<T>>::insert(&operator, operator_state);
		<DelegatorState<T>>::insert(&delegator, delegator_state);
		<Pallet<T>>::deposit_event(Event::Delegation {
			delegator,
			locked_amount: amount,
			operator,
			delegator_position,
			auto_compound,
		});

		Ok(().into())
	}

	/// Sets the auto-compounding value for a delegation. The config is removed if value is zero.
	pub(crate) fn set_auto_compound(
		operator: T::AccountId,
		delegator: T::AccountId,
		value: Percent,
		operator_auto_compounding_delegation_count_hint: u32,
		delegation_count_hint: u32,
	) -> DispatchResultWithPostInfo {
		let delegator_state =
			<DelegatorState<T>>::get(&delegator).ok_or(<Error<T>>::DelegatorDNE)?;
		ensure!(
			delegator_state.delegations.0.len() <= delegation_count_hint as usize,
			<Error<T>>::TooLowDelegationCountToAutoCompound,
		);
		ensure!(
			delegator_state.delegations.0.iter().any(|b| b.owner == operator),
			<Error<T>>::DelegationDNE,
		);

		let mut auto_compounding_state = Self::get_storage(&operator);
		ensure!(
			auto_compounding_state.len() <= operator_auto_compounding_delegation_count_hint,
			<Error<T>>::TooLowOperatorAutoCompoundingDelegationCountToAutoCompound,
		);
		let state_updated = if value.is_zero() {
			auto_compounding_state.remove_for_delegator(&delegator)
		} else {
			auto_compounding_state.set_for_delegator(delegator.clone(), value)?
		};
		if state_updated {
			auto_compounding_state.set_storage(&operator);
		}

		<Pallet<T>>::deposit_event(Event::AutoCompoundSet { operator, delegator, value });

		Ok(().into())
	}

	/// Removes the auto-compounding value for a delegation. This should be called when the
	/// delegation is revoked to cleanup storage. Storage is only written iff the entry existed.
	pub(crate) fn remove_auto_compound(operator: &T::AccountId, delegator: &T::AccountId) {
		let mut auto_compounding_state = Self::get_storage(operator);
		if auto_compounding_state.remove_for_delegator(delegator) {
			auto_compounding_state.set_storage(&operator);
		}
	}

	/// Returns the value of auto-compound, if it exists for a given delegation, zero otherwise.
	pub(crate) fn auto_compound(operator: &T::AccountId, delegator: &T::AccountId) -> Percent {
		let delegations_config = Self::get_storage(operator);
		delegations_config
			.get_for_delegator(&delegator)
			.unwrap_or_else(|| Percent::zero())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::Test;

	#[test]
	fn test_set_for_delegator_inserts_config_and_returns_true_if_entry_missing() {
		let mut delegations_config =
			AutoCompoundDelegations::<Test>::new(vec![].try_into().expect("must succeed"));
		assert_eq!(
			true,
			delegations_config
				.set_for_delegator(1, Percent::from_percent(50))
				.expect("must succeed")
		);
		assert_eq!(
			vec![AutoCompoundConfig { delegator: 1, value: Percent::from_percent(50) }],
			delegations_config.into_inner().into_inner(),
		);
	}

	#[test]
	fn test_set_for_delegator_updates_config_and_returns_true_if_entry_changed() {
		let mut delegations_config = AutoCompoundDelegations::<Test>::new(
			vec![AutoCompoundConfig { delegator: 1, value: Percent::from_percent(10) }]
				.try_into()
				.expect("must succeed"),
		);
		assert_eq!(
			true,
			delegations_config
				.set_for_delegator(1, Percent::from_percent(50))
				.expect("must succeed")
		);
		assert_eq!(
			vec![AutoCompoundConfig { delegator: 1, value: Percent::from_percent(50) }],
			delegations_config.into_inner().into_inner(),
		);
	}

	#[test]
	fn test_set_for_delegator_updates_config_and_returns_false_if_entry_unchanged() {
		let mut delegations_config = AutoCompoundDelegations::<Test>::new(
			vec![AutoCompoundConfig { delegator: 1, value: Percent::from_percent(10) }]
				.try_into()
				.expect("must succeed"),
		);
		assert_eq!(
			false,
			delegations_config
				.set_for_delegator(1, Percent::from_percent(10))
				.expect("must succeed")
		);
		assert_eq!(
			vec![AutoCompoundConfig { delegator: 1, value: Percent::from_percent(10) }],
			delegations_config.into_inner().into_inner(),
		);
	}

	#[test]
	fn test_remove_for_delegator_returns_false_if_entry_was_missing() {
		let mut delegations_config =
			AutoCompoundDelegations::<Test>::new(vec![].try_into().expect("must succeed"));
		assert_eq!(false, delegations_config.remove_for_delegator(&1),);
	}

	#[test]
	fn test_remove_delegation_config_returns_true_if_entry_existed() {
		let mut delegations_config = AutoCompoundDelegations::<Test>::new(
			vec![AutoCompoundConfig { delegator: 1, value: Percent::from_percent(10) }]
				.try_into()
				.expect("must succeed"),
		);
		assert_eq!(true, delegations_config.remove_for_delegator(&1));
	}
}
