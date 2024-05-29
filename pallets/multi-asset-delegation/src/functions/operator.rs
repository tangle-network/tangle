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

/// Functions for the pallet.
use super::*;
use crate::types::*;
use crate::Pallet;
use frame_support::ensure;
use frame_support::pallet_prelude::DispatchResult;
use frame_support::traits::Get;
use frame_support::traits::ReservableCurrency;
use sp_runtime::DispatchError;

impl<T: Config> Pallet<T> {
	pub fn handle_deposit_and_create_operator(
		who: T::AccountId,
		bond_amount: BalanceOf<T>,
	) -> DispatchResult {
		// Check if the user is already an operator
		ensure!(!Operators::<T>::contains_key(&who), Error::<T>::AlreadyOperator);

		ensure!(bond_amount >= T::MinOperatorBondAmount::get(), Error::<T>::BondTooLow);

		/// Ensure the user has enough balance to reserve the bond amount
		T::Currency::reserve(&who, bond_amount)?;

		let operator_metadata = OperatorMetadata {
			bond: bond_amount,
			delegation_count: 0,
			request: None,
			status: OperatorStatus::Active,
		};

		// Add the user as an operator
		Operators::<T>::insert(&who, operator_metadata);

		Ok(())
	}

	pub fn process_leave_operator(who: &T::AccountId) -> DispatchResult {
		// Check if the operator exists and is active
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;

		match operator.status {
			OperatorStatus::Leaving(x) => {
				return Err(Error::<T>::AlreadyLeaving.into());
			},
			_ => {},
		};

		// Check if the operator can exit using the ServiceManager trait
		ensure!(T::ServiceManager::can_exit(who), Error::<T>::CannotExit);

		// Calculate the leaving time
		let current_round = Self::current_round();
		let leaving_time = current_round + T::LeaveOperatorsDelay::get();

		// Update the operator's status to Leaving
		operator.status = OperatorStatus::Leaving(leaving_time);

		Operators::<T>::insert(who, operator);

		Ok(())
	}

	pub fn process_cancel_leave_operator(who: &T::AccountId) -> Result<(), DispatchError> {
		// Check if the operator exists and is in leaving state
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;

		match operator.status {
			OperatorStatus::Leaving(x) => {},
			_ => {
				return Err(Error::<T>::NotLeavingOperator.into());
			},
		};

		// Update the operator's status to Active
		operator.status = OperatorStatus::Active;

		Operators::<T>::insert(who, operator);

		Ok(())
	}

	pub fn process_execute_leave_operators(who: &T::AccountId) -> Result<(), DispatchError> {
		// Check if the operator exists and is in leaving state
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;

		// Calculate the leaving time
		let current_round = Self::current_round();

		match operator.status {
			OperatorStatus::Leaving(leaving_round) => {
				ensure!(current_round <= leaving_round, Error::<T>::NotLeavingRound);
			},
			_ => {
				return Err(Error::<T>::NotLeavingOperator.into());
			},
		};

		// TODO : Put delegated funds back into pool

		/// unresrve the bond amount
		T::Currency::reserve(&who, operator.bond)?;

		Ok(())
	}

	pub fn process_operator_bond_more(
		who: &T::AccountId,
		additional_bond: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		// Check if the operator exists
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;

		// Reserve the additional bond amount
		T::Currency::reserve(who, additional_bond)?;

		// Update the operator's bond amount
		operator.bond += additional_bond;
		Operators::<T>::insert(who, operator);

		Ok(())
	}

	pub fn process_schedule_operator_bond_less(
		who: &T::AccountId,
		bond_less_amount: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		// Check if the operator exists and has no active services using TNT
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;

		// Check if the operator can exit using the ServiceManager trait
		ensure!(T::ServiceManager::can_exit(who), Error::<T>::CannotExit);

		// TODO : We should instead be checking the TNT stake usage

		// Schedule the bond less request
		operator.request = Some(OperatorBondLessRequest {
			amount: bond_less_amount,
			request_time: Self::current_round(),
		});
		Operators::<T>::insert(who, operator);

		Ok(())
	}

	pub fn process_execute_operator_bond_less(who: &T::AccountId) -> Result<(), DispatchError> {
		// Check if the operator exists and has a scheduled bond less request
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;
		let request = operator.request.as_ref().ok_or(Error::<T>::NoScheduledBondLess)?;

		// Ensure the bond less request is satisfied (current round >= request time)
		let current_round = Self::current_round();
		ensure!(
			current_round >= T::LeaveOperatorsDelay::get() + request.request_time,
			Error::<T>::BondLessRequestNotSatisfied
		);

		// Unreserve the bond less amount
		T::Currency::unreserve(who, request.amount);

		// Update the operator's bond amount
		operator.bond -= request.amount;
		operator.request = None;
		Operators::<T>::insert(who, operator);

		Ok(())
	}

	pub fn process_cancel_operator_bond_less(who: &T::AccountId) -> Result<(), DispatchError> {
		// Check if the operator exists and has a scheduled bond less request
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;
		ensure!(operator.request.is_some(), Error::<T>::NoScheduledBondLess);

		// Cancel the bond less request
		operator.request = None;
		Operators::<T>::insert(who, operator);

		Ok(())
	}

	pub fn process_go_offline(who: &T::AccountId) -> Result<(), DispatchError> {
		// Check if the operator exists and is currently active
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;
		ensure!(operator.status == OperatorStatus::Active, Error::<T>::NotActiveOperator);

		// Update the operator's status to Offline
		operator.status = OperatorStatus::Inactive;
		Operators::<T>::insert(who, operator);

		Ok(())
	}

	pub fn process_go_online(who: &T::AccountId) -> Result<(), DispatchError> {
		// Check if the operator exists and is currently offline
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;
		ensure!(operator.status == OperatorStatus::Inactive, Error::<T>::NotOfflineOperator);

		// Update the operator's status to Online
		operator.status = OperatorStatus::Active;
		Operators::<T>::insert(who, operator);

		Ok(())
	}
}
