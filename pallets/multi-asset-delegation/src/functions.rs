/// Functions for the pallet.
use super::*;
use crate::{offences::ValidatorOffence, types::*};
use frame_support::traits::DefensiveSaturating;
use frame_support::traits::UnixTime;
use frame_support::{
	pallet_prelude::DispatchResult,
	traits::{DefensiveResult, Imbalance, OnUnbalanced},
};
use pallet_staking::ActiveEra;
use pallet_staking::EraPayout;
use sp_runtime::SaturatedConversion;
use sp_runtime::Saturating;
use sp_runtime::{traits::Convert, Perbill};
use sp_staking::offence::Offence;
use sp_std::collections::btree_map::BTreeMap;
use tangle_primitives::jobs::{traits::JobsHandler, JobId, ReportRestakerOffence};

impl<T: Config> Pallet<T> {
	pub fn join_operators(origin: OriginFor<T>) -> DispatchResult {
        let who = ensure_signed(origin)?;
        
        // Check if the user is already an operator
        ensure!(!Operators::<T>::contains_key(&who), Error::<T>::AlreadyOperator);
        
        // Call the function to handle deposit and staking check
        Self::handle_deposit_and_staking(&who)?;

        // Add the user as an operator
        Operators::<T>::insert(&who, Operator::default());
        
        // Emit an event
        Self::deposit_event(Event::OperatorJoined(who));
        
        Ok(())
    }

    fn handle_deposit_and_staking(who: &T::AccountId) -> Result<(), DispatchError> {
        // Ensure the user has enough balance to reserve the bond amount
        T::Currency::reserve(who, T::BondAmount::get())?;
                
        Ok(())
    }
}
