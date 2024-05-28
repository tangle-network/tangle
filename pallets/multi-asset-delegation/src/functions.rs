/// Functions for the pallet.
use super::*;
use crate::{types::*};
use frame_support::traits::ReservableCurrency;
use frame_support::traits::Get;
use frame_support::ensure;
use frame_support::pallet_prelude::DispatchResult;

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
        
        // Emit an event
        Self::deposit_event(Event::OperatorJoined(who));
        
        Ok(())
    }

}
