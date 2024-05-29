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

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod functions;
pub mod traits;
pub mod types;
pub use functions::*;
pub use traits::*;

#[frame_support::pallet]
pub mod pallet {
	use crate::traits::ServiceManager;
	use crate::types::*;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, LockableCurrency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, StaticLookup};
	use sp_std::vec::Vec;

	use frame_support::traits::fungibles;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;

		type MinOperatorBondAmount: Get<BalanceOf<Self>>;

		type BondDuration: Get<BlockNumberFor<Self>>;

		type ServiceManager: ServiceManager<Self::AccountId, BalanceOf<Self>>;

		/// Number of rounds that operator remain bonded before exit request is executable
		#[pallet::constant]
		type LeaveOperatorsDelay: Get<RoundIndex>;
		/// Number of rounds Operator requests to decrease self-bond must wait to be executable
		#[pallet::constant]
		type OperatorBondLessDelay: Get<RoundIndex>;
		/// Number of rounds that delegators remain bonded before exit request is executable
		#[pallet::constant]
		type LeaveDelegatorsDelay: Get<RoundIndex>;
		/// Number of rounds that delegations remain bonded before revocation request is executable
		#[pallet::constant]
		type RevokeDelegationDelay: Get<RoundIndex>;
		/// Number of rounds that delegation less requests must wait before executable
		#[pallet::constant]
		type DelegationBondLessDelay: Get<RoundIndex>;

		type Fungibles: fungibles::Inspect<Self::AccountId> + fungibles::Mutate<Self::AccountId>;

		type AssetId: Parameter + Member + Copy;

		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: crate::weights::WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn operator_info)]
	pub(crate) type Operators<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, OperatorMetadata<BalanceOf<T>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn current_round)]
	pub type CurrentRound<T: Config> = StorageValue<_, RoundIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn at_stake)]
	/// Snapshot of collator delegation stake at the start of the round
	pub type AtStake<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		RoundIndex,
		Twox64Concat,
		T::AccountId,
		OperatorSnapshotOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn delegator_info)]
	pub(crate) type Delegators<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		DelegatorMetadata<T::AssetId, BalanceOf<T>>,
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		OperatorJoined { who: T::AccountId },

		OperatorLeavingScheduled { who: T::AccountId },

		OperatorLeaveCancelled { who: T::AccountId },

		OperatorLeaveExecuted { who: T::AccountId },

		OperatorBondMore { who: T::AccountId, additional_bond: BalanceOf<T> },

		OperatorBondLessScheduled { who: T::AccountId, bond_less_amount: BalanceOf<T> },

		OperatorBondLessExecuted { who: T::AccountId },

		OperatorBondLessCancelled { who: T::AccountId },

		OperatorWentOffline { who: T::AccountId },
		OperatorWentOnline { who: T::AccountId },
		Deposited { who: T::AccountId, amount: BalanceOf<T>, asset_id: Option<T::AssetId> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		AlreadyOperator,
		/// Errors should have helpful documentation associated with them.
		BondTooLow,
		NotAnOperator,
		CannotExit,
		AlreadyLeaving,
		NotLeavingOperator,
		NotLeavingRound,
		NoScheduledBondLess,
		BondLessRequestNotSatisfied,
		NotActiveOperator,
		NotOfflineOperator,
		AlreadyDelegator,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn join_operators(origin: OriginFor<T>, bond_amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::handle_deposit_and_create_operator(who.clone(), bond_amount)?;

			Self::deposit_event(Event::OperatorJoined { who });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn schedule_leave_operators(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::process_leave_operator(&who)?;

			// Emit an event
			Self::deposit_event(Event::OperatorLeavingScheduled { who });

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn cancel_leave_operators(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::process_cancel_leave_operator(&who)?;

			// Emit an event
			Self::deposit_event(Event::OperatorLeaveCancelled { who });

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn execute_leave_operators(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::process_execute_leave_operators(&who)?;

			// Emit an event
			Self::deposit_event(Event::OperatorLeaveExecuted { who });

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn operator_bond_more(
			origin: OriginFor<T>,
			additional_bond: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::process_operator_bond_more(&who, additional_bond)?;

			// Emit an event
			Self::deposit_event(Event::OperatorBondMore { who, additional_bond });

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn schedule_operator_bond_less(
			origin: OriginFor<T>,
			bond_less_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::process_schedule_operator_bond_less(&who, bond_less_amount)?;

			// Emit an event
			Self::deposit_event(Event::OperatorBondLessScheduled { who, bond_less_amount });

			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn execute_operator_bond_less(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::process_execute_operator_bond_less(&who)?;

			// Emit an event
			Self::deposit_event(Event::OperatorBondLessExecuted { who });

			Ok(())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn cancel_operator_bond_less(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::process_cancel_operator_bond_less(&who)?;

			// Emit an event
			Self::deposit_event(Event::OperatorBondLessCancelled { who });

			Ok(())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn go_offline(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::process_go_offline(&who)?;

			// Emit an event
			Self::deposit_event(Event::OperatorWentOffline { who });

			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn go_online(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::process_go_online(&who)?;

			// Emit an event
			Self::deposit_event(Event::OperatorWentOnline { who });

			Ok(())
		}

		#[pallet::call_index(10)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn deposit(
			origin: OriginFor<T>,
			asset_id: Option<T::AssetId>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::process_deposit(&who, asset_id, amount)?;

			// Emit an event
			Self::deposit_event(Event::Deposited { who, amount, asset_id });

			Ok(())
		}

		#[pallet::call_index(11)]
		#[pallet::weight(10_000)]
		pub fn schedule_unstake(
			origin: OriginFor<T>,
			asset_id: Option<T::AssetId>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_schedule_unstake(&who, asset_id, amount)?;
			Self::deposit_event(Event::ScheduledUnstake(who, asset_id, amount));
			Ok(())
		}

		#[pallet::call_index(12)]
		#[pallet::weight(10_000)]
		pub fn execute_unstake(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_execute_unstake(&who)?;
			Self::deposit_event(Event::ExecutedUnstake(who));
			Ok(())
		}

		#[pallet::call_index(13)]
		#[pallet::weight(10_000)]
		pub fn cancel_unstake(origin: OriginFor<T>, asset_id: T::AssetId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_cancel_unstake(&who, asset_id)?;
			Self::deposit_event(Event::CancelledUnstake(who, asset_id, amount));
			Ok(())
		}
	}
}
