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
use super::*;
/// Functions for the pallet.
use super::*;
use crate::{types::*};
use frame_support::traits::ReservableCurrency;
use frame_support::traits::Get;
use frame_support::ensure;
use frame_support::pallet_prelude::DispatchResult;
use sp_runtime::DispatchError;

impl<T: Config> Pallet<T> {
	pub fn handle_deposit_and_create_operator(who: T::AccountId, bond_amount: BalanceOf<T>) -> DispatchResult {
        
        // Check if the user is already an operator
        ensure!(!Operators::<T>::contains_key(&who), Error::<T>::AlreadyOperator);

        ensure!(bond_amount >= T::MinOperatorBondAmount::get(), Error::<T>::BondTooLow);

        /// Ensure the user has enough balance to reserve the bond amount
        T::Currency::reserve(&who, bond_amount)?;

        let operator_metadata = OperatorMetadata {
            bond: bond_amount,
            delegation_count: 0,
            total_counted: bond_amount,
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
            OperatorStatus::Leaving(x) => { return Err(Error::<T>::AlreadyLeaving.into()); },
            _ => {}
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

}
