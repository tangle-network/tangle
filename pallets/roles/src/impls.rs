use super::*;
use frame_support::{pallet_prelude::DispatchResult, traits::WithdrawReasons};
use sp_runtime::Saturating;
use tangle_primitives::{roles::RoleType, traits::roles::RolesHandler};

/// Implements RolesHandler for the pallet.
impl<T: Config> RolesHandler<T::AccountId> for Pallet<T> {
	fn validate_role(address: T::AccountId, role: RoleType) -> bool {
		let assigned_role = AccountRolesMapping::<T>::get(address);
		match assigned_role {
			Some(r) =>
				if r == role {
					return true
				},
			None => return false,
		}

		false
	}
	fn slash_validator(
		address: T::AccountId,
		_offence: tangle_primitives::jobs::ValidatorOffence,
	) -> sp_runtime::DispatchResult {
		// TODO: implement calculation of slash amount.
		let slash_amount = 1000u64;
		Self::do_slash(address, slash_amount.into())?;
		Ok(())
	}
}

/// Functions for the pallet.
impl<T: Config> Pallet<T> {
	/// The total balance that can be slashed from a stash account as of right now.
	pub fn slashable_balance_of(stash: &T::AccountId) -> BalanceOf<T> {
		// Weight note: consider making the stake accessible through stash.
		Self::ledger(&stash).map(|l| l.total_locked).unwrap_or_default()
	}

	/// Slash staker's balance by the given amount.
	pub(crate) fn do_slash(
		address: T::AccountId,
		slash_amount: T::CurrencyBalance,
	) -> sp_runtime::DispatchResult {
		let mut ledger = Self::ledger(&address).ok_or(Error::<T>::InvalidStashController)?;
		let (_imbalance, _missing) = T::Currency::slash(&address, slash_amount.into());
		ledger.total_locked = ledger.total_locked.saturating_sub(slash_amount.into());
		Self::update_ledger(&address, &ledger);
		Self::deposit_event(Event::Slashed { account: address, amount: slash_amount });
		Ok(())
	}

	/// Update the ledger for the staker.
	///
	/// This will also update the stash lock.
	pub(crate) fn update_ledger(staker: &T::AccountId, ledger: &RoleStakingLedger<T>) {
		T::Currency::set_lock(
			ROLES_STAKING_ID,
			&ledger.stash,
			ledger.total_locked,
			WithdrawReasons::all(),
		);
		<Ledger<T>>::insert(staker, ledger);
	}
	/// Clear stash account information from pallet.
	pub(crate) fn kill_stash(stash: &T::AccountId) -> DispatchResult {
		<Ledger<T>>::remove(&stash);
		Ok(())
	}

	/// Unbond the full amount of the stash.
	pub(super) fn unbond(ledger: &RoleStakingLedger<T>) -> DispatchResult {
		let stash = ledger.stash.clone();
		if ledger.total_locked > T::Currency::minimum_balance() {
			// Remove the lock.
			T::Currency::remove_lock(ROLES_STAKING_ID, &stash);
			// Kill the stash and related information
			Self::kill_stash(&stash)?;
		}
		Ok(())
	}
}
