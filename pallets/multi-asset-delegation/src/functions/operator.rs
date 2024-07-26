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
	/// Handles the deposit of bond amount and creation of an operator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the operator.
	/// * `bond_amount` - The amount to be bonded by the operator.
	///
	/// # Errors
	///
	/// Returns an error if the user is already an operator or if the bond amount is too low.
	pub fn handle_deposit_and_create_operator(
		who: T::AccountId,
		bond_amount: BalanceOf<T>,
	) -> DispatchResult {
		ensure!(!Operators::<T>::contains_key(&who), Error::<T>::AlreadyOperator);
		ensure!(bond_amount >= T::MinOperatorBondAmount::get(), Error::<T>::BondTooLow);
		T::Currency::reserve(&who, bond_amount)?;

		let operator_metadata = OperatorMetadata {
			bond: bond_amount,
			delegation_count: 0,
			request: None,
			delegations: Default::default(),
			status: OperatorStatus::Active,
		};

		Operators::<T>::insert(&who, operator_metadata);

		Ok(())
	}

	/// Processes the leave operation for an operator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the operator.
	///
	/// # Errors
	///
	/// Returns an error if the operator is not found, already leaving, or cannot exit.
	#[allow(clippy::single_match)]
	pub fn process_leave_operator(who: &T::AccountId) -> DispatchResult {
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;

		match operator.status {
			OperatorStatus::Leaving(_) => return Err(Error::<T>::AlreadyLeaving.into()),
			_ => {},
		};

		ensure!(T::ServiceManager::can_exit(who), Error::<T>::CannotExit);

		let current_round = Self::current_round();
		let leaving_time = current_round + T::LeaveOperatorsDelay::get();

		operator.status = OperatorStatus::Leaving(leaving_time);
		Operators::<T>::insert(who, operator);

		Ok(())
	}

	/// Cancels the leave operation for an operator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the operator.
	///
	/// # Errors
	///
	/// Returns an error if the operator is not found or not in leaving state.
	pub fn process_cancel_leave_operator(who: &T::AccountId) -> Result<(), DispatchError> {
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;

		match operator.status {
			OperatorStatus::Leaving(_) => {},
			_ => return Err(Error::<T>::NotLeavingOperator.into()),
		};

		operator.status = OperatorStatus::Active;
		Operators::<T>::insert(who, operator);

		Ok(())
	}

	/// Executes the leave operation for an operator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the operator.
	///
	/// # Errors
	///
	/// Returns an error if the operator is not found, not in leaving state, or the leaving round has not been reached.
	pub fn process_execute_leave_operators(who: &T::AccountId) -> Result<(), DispatchError> {
		let operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;
		let current_round = Self::current_round();

		match operator.status {
			OperatorStatus::Leaving(leaving_round) => {
				ensure!(current_round >= leaving_round, Error::<T>::NotLeavingRound);
			},
			_ => return Err(Error::<T>::NotLeavingOperator.into()),
		};

		T::Currency::unreserve(who, operator.bond);
		Operators::<T>::remove(who);

		Ok(())
	}

	/// Processes an additional bond for an operator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the operator.
	/// * `additional_bond` - The additional amount to be bonded by the operator.
	///
	/// # Errors
	///
	/// Returns an error if the operator is not found or if the reserve fails.
	pub fn process_operator_bond_more(
		who: &T::AccountId,
		additional_bond: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;
		T::Currency::reserve(who, additional_bond)?;

		operator.bond += additional_bond;
		Operators::<T>::insert(who, operator);

		Ok(())
	}

	/// Schedules a bond reduction for an operator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the operator.
	/// * `unstake_amount` - The amount to be reduced from the operator's bond.
	///
	/// # Errors
	///
	/// Returns an error if the operator is not found, has active services, or cannot exit.
	pub fn process_schedule_operator_unstake(
		who: &T::AccountId,
		unstake_amount: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;
		ensure!(T::ServiceManager::can_exit(who), Error::<T>::CannotExit);

		operator.request = Some(OperatorBondLessRequest {
			amount: unstake_amount,
			request_time: Self::current_round(),
		});
		Operators::<T>::insert(who, operator);

		Ok(())
	}

	/// Executes a scheduled bond reduction for an operator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the operator.
	///
	/// # Errors
	///
	/// Returns an error if the operator is not found, has no scheduled bond reduction, or the request is not satisfied.
	pub fn process_execute_operator_unstake(who: &T::AccountId) -> Result<(), DispatchError> {
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;
		let request = operator.request.as_ref().ok_or(Error::<T>::NoScheduledBondLess)?;
		let current_round = Self::current_round();

		ensure!(
			current_round >= T::OperatorBondLessDelay::get() + request.request_time,
			Error::<T>::BondLessRequestNotSatisfied
		);

		operator.bond -= request.amount;
		operator.request = None;
		Operators::<T>::insert(who, operator);

		Ok(())
	}

	/// Cancels a scheduled bond reduction for an operator.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the operator.
	///
	/// # Errors
	///
	/// Returns an error if the operator is not found or has no scheduled bond reduction.
	pub fn process_cancel_operator_unstake(who: &T::AccountId) -> Result<(), DispatchError> {
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;
		ensure!(operator.request.is_some(), Error::<T>::NoScheduledBondLess);

		operator.request = None;
		Operators::<T>::insert(who, operator);

		Ok(())
	}

	/// Sets the operator status to offline.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the operator.
	///
	/// # Errors
	///
	/// Returns an error if the operator is not found or not currently active.
	pub fn process_go_offline(who: &T::AccountId) -> Result<(), DispatchError> {
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;
		ensure!(operator.status == OperatorStatus::Active, Error::<T>::NotActiveOperator);

		operator.status = OperatorStatus::Inactive;
		Operators::<T>::insert(who, operator);

		Ok(())
	}

	/// Sets the operator status to online.
	///
	/// # Arguments
	///
	/// * `who` - The account ID of the operator.
	///
	/// # Errors
	///
	/// Returns an error if the operator is not found or not currently inactive.
	pub fn process_go_online(who: &T::AccountId) -> Result<(), DispatchError> {
		let mut operator = Operators::<T>::get(who).ok_or(Error::<T>::NotAnOperator)?;
		ensure!(operator.status == OperatorStatus::Inactive, Error::<T>::NotOfflineOperator);

		operator.status = OperatorStatus::Active;
		Operators::<T>::insert(who, operator);

		Ok(())
	}
}
