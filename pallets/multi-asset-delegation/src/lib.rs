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
	use frame_support::traits::fungibles;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		sp_runtime::traits::AccountIdConversion,
		traits::{Currency, LockableCurrency, ReservableCurrency},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, StaticLookup};
	use sp_std::vec::Vec;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency type used for managing balances.
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;

		/// The minimum amount of bond required for an operator.
		type MinOperatorBondAmount: Get<BalanceOf<Self>>;

		/// The minimum amount of bond required for a delegate.
		type MinDelegateAmount: Get<BalanceOf<Self>>;

		/// The duration for which the bond is locked.
		type BondDuration: Get<BlockNumberFor<Self>>;

		/// The service manager that manages active services.
		type ServiceManager: ServiceManager<Self::AccountId, BalanceOf<Self>>;

		/// Number of rounds that operators remain bonded before the exit request is executable.
		#[pallet::constant]
		type LeaveOperatorsDelay: Get<RoundIndex>;

		/// Number of rounds operator requests to decrease self-bond must wait to be executable.
		#[pallet::constant]
		type OperatorBondLessDelay: Get<RoundIndex>;

		/// Number of rounds that delegators remain bonded before the exit request is executable.
		#[pallet::constant]
		type LeaveDelegatorsDelay: Get<RoundIndex>;

		/// Number of rounds that delegations remain bonded before the revocation request is executable.
		#[pallet::constant]
		type RevokeDelegationDelay: Get<RoundIndex>;

		/// Number of rounds that delegation bond less requests must wait before being executable.
		#[pallet::constant]
		type DelegationBondLessDelay: Get<RoundIndex>;

		/// The fungibles trait used for managing fungible assets.
		type Fungibles: fungibles::Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = BalanceOf<Self>>
			+ fungibles::Mutate<Self::AccountId, AssetId = Self::AssetId>;

		/// The asset ID type.
		type AssetId: AtLeast32BitUnsigned
			+ Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ Clone
			+ Copy
			+ PartialOrd
			+ MaxEncodedLen;

		/// The pallet's account ID.
		type PalletId: Get<PalletId>;

		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: crate::weights::WeightInfo;
	}

	/// The pallet struct.
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Storage for operator information.
	#[pallet::storage]
	#[pallet::getter(fn operator_info)]
	pub(crate) type Operators<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, OperatorMetadata<BalanceOf<T>>, OptionQuery>;

	/// Storage for the current round.
	#[pallet::storage]
	#[pallet::getter(fn current_round)]
	pub type CurrentRound<T: Config> = StorageValue<_, RoundIndex, ValueQuery>;

	/// Snapshot of collator delegation stake at the start of the round.
	#[pallet::storage]
	#[pallet::getter(fn at_stake)]
	pub type AtStake<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		RoundIndex,
		Twox64Concat,
		T::AccountId,
		OperatorSnapshotOf<T>,
		OptionQuery,
	>;

	/// Storage for delegator information.
	#[pallet::storage]
	#[pallet::getter(fn delegators)]
	pub(crate) type Delegators<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, DelegatorMetadataOf<T>, OptionQuery>;

	/// Events emitted by the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An operator has joined.
		OperatorJoined { who: T::AccountId },
		/// An operator has scheduled to leave.
		OperatorLeavingScheduled { who: T::AccountId },
		/// An operator has cancelled their leave request.
		OperatorLeaveCancelled { who: T::AccountId },
		/// An operator has executed their leave request.
		OperatorLeaveExecuted { who: T::AccountId },
		/// An operator has increased their bond.
		OperatorBondMore { who: T::AccountId, additional_bond: BalanceOf<T> },
		/// An operator has scheduled to decrease their bond.
		OperatorBondLessScheduled { who: T::AccountId, bond_less_amount: BalanceOf<T> },
		/// An operator has executed their bond decrease.
		OperatorBondLessExecuted { who: T::AccountId },
		/// An operator has cancelled their bond decrease request.
		OperatorBondLessCancelled { who: T::AccountId },
		/// An operator has gone offline.
		OperatorWentOffline { who: T::AccountId },
		/// An operator has gone online.
		OperatorWentOnline { who: T::AccountId },
		/// A deposit has been made.
		Deposited { who: T::AccountId, amount: BalanceOf<T>, asset_id: Option<T::AssetId> },
		/// An unstake has been scheduled.
		ScheduledUnstake { who: T::AccountId, amount: BalanceOf<T>, asset_id: Option<T::AssetId> },
		/// An unstake has been executed.
		ExecutedUnstake { who: T::AccountId },
		/// An unstake has been cancelled.
		CancelledUnstake { who: T::AccountId },
		/// A delegation has been made.
		Delegated {
			who: T::AccountId,
			operator: T::AccountId,
			amount: BalanceOf<T>,
			asset_id: T::AssetId,
		},
		/// A delegator bond less request has been scheduled.
		ScheduledDelegatorBondLess {
			who: T::AccountId,
			operator: T::AccountId,
			amount: BalanceOf<T>,
			asset_id: T::AssetId,
		},
		/// A delegator bond less request has been executed.
		ExecutedDelegatorBondLess { who: T::AccountId },
		/// A delegator bond less request has been cancelled.
		CancelledDelegatorBondLess { who: T::AccountId },
	}

	/// Errors emitted by the pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// The account is already an operator.
		AlreadyOperator,
		/// The bond amount is too low.
		BondTooLow,
		/// The account is not an operator.
		NotAnOperator,
		/// The account cannot exit.
		CannotExit,
		/// The operator is already leaving.
		AlreadyLeaving,
		/// The account is not leaving as an operator.
		NotLeavingOperator,
		/// The round does not match the scheduled leave round.
		NotLeavingRound,
		/// There is no scheduled bond less request.
		NoScheduledBondLess,
		/// The bond less request is not satisfied.
		BondLessRequestNotSatisfied,
		/// The operator is not active.
		NotActiveOperator,
		/// The operator is not offline.
		NotOfflineOperator,
		/// The account is already a delegator.
		AlreadyDelegator,
		/// The account is not a delegator.
		NotDelegator,
		/// A withdraw request already exists.
		WithdrawRequestAlreadyExists,
		/// The account has insufficient balance.
		InsufficientBalance,
		/// There is no withdraw request.
		NoWithdrawRequest,
		/// The unstake is not ready.
		UnstakeNotReady,
		/// There is no bond less request.
		NoBondLessRequest,
		/// The bond less request is not ready.
		BondLessNotReady,
		/// A bond less request already exists.
		BondLessRequestAlreadyExists,
		/// There are active services using the asset.
		ActiveServicesUsingAsset,
	}

	/// Hooks for the pallet.
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// The callable functions (extrinsics) of the pallet.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Allows an account to join as an operator by providing a bond.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn join_operators(origin: OriginFor<T>, bond_amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::handle_deposit_and_create_operator(who.clone(), bond_amount)?;
			Self::deposit_event(Event::OperatorJoined { who });
			Ok(())
		}

		/// Schedules an operator to leave.
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn schedule_leave_operators(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_leave_operator(&who)?;
			Self::deposit_event(Event::OperatorLeavingScheduled { who });
			Ok(())
		}

		/// Cancels a scheduled leave for an operator.
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn cancel_leave_operators(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_cancel_leave_operator(&who)?;
			Self::deposit_event(Event::OperatorLeaveCancelled { who });
			Ok(())
		}

		/// Executes a scheduled leave for an operator.
		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn execute_leave_operators(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_execute_leave_operators(&who)?;
			Self::deposit_event(Event::OperatorLeaveExecuted { who });
			Ok(())
		}

		/// Allows an operator to increase their bond.
		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn operator_bond_more(
			origin: OriginFor<T>,
			additional_bond: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_operator_bond_more(&who, additional_bond)?;
			Self::deposit_event(Event::OperatorBondMore { who, additional_bond });
			Ok(())
		}

		/// Schedules an operator to decrease their bond.
		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn schedule_operator_bond_less(
			origin: OriginFor<T>,
			bond_less_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_schedule_operator_bond_less(&who, bond_less_amount)?;
			Self::deposit_event(Event::OperatorBondLessScheduled { who, bond_less_amount });
			Ok(())
		}

		/// Executes a scheduled bond decrease for an operator.
		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn execute_operator_bond_less(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_execute_operator_bond_less(&who)?;
			Self::deposit_event(Event::OperatorBondLessExecuted { who });
			Ok(())
		}

		/// Cancels a scheduled bond decrease for an operator.
		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn cancel_operator_bond_less(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_cancel_operator_bond_less(&who)?;
			Self::deposit_event(Event::OperatorBondLessCancelled { who });
			Ok(())
		}

		/// Allows an operator to go offline.
		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn go_offline(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_go_offline(&who)?;
			Self::deposit_event(Event::OperatorWentOffline { who });
			Ok(())
		}

		/// Allows an operator to go online.
		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn go_online(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_go_online(&who)?;
			Self::deposit_event(Event::OperatorWentOnline { who });
			Ok(())
		}

		/// Allows a user to deposit an asset.
		#[pallet::call_index(10)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn deposit(
			origin: OriginFor<T>,
			asset_id: Option<T::AssetId>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_deposit(who.clone(), asset_id, amount)?;
			Self::deposit_event(Event::Deposited { who, amount, asset_id });
			Ok(())
		}

		/// Schedules an unstake request.
		#[pallet::call_index(11)]
		#[pallet::weight(10_000)]
		pub fn schedule_unstake(
			origin: OriginFor<T>,
			asset_id: Option<T::AssetId>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_schedule_unstake(who.clone(), asset_id, amount)?;
			Self::deposit_event(Event::ScheduledUnstake { who, amount, asset_id });
			Ok(())
		}

		/// Executes a scheduled unstake request.
		#[pallet::call_index(12)]
		#[pallet::weight(10_000)]
		pub fn execute_unstake(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_execute_unstake(who.clone())?;
			Self::deposit_event(Event::ExecutedUnstake { who });
			Ok(())
		}

		/// Cancels a scheduled unstake request.
		#[pallet::call_index(13)]
		#[pallet::weight(10_000)]
		pub fn cancel_unstake(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_cancel_unstake(who.clone())?;
			Self::deposit_event(Event::CancelledUnstake { who });
			Ok(())
		}

		/// Allows a user to delegate an amount of an asset to an operator.
		#[pallet::call_index(14)]
		#[pallet::weight(10_000)]
		pub fn delegate(
			origin: OriginFor<T>,
			operator: T::AccountId,
			asset_id: T::AssetId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_delegate(who.clone(), operator.clone(), asset_id, amount)?;
			Self::deposit_event(Event::Delegated { who, operator, asset_id, amount });
			Ok(())
		}

		/// Schedules a request to reduce a delegator's bond.
		#[pallet::call_index(15)]
		#[pallet::weight(10_000)]
		pub fn schedule_delegator_bond_less(
			origin: OriginFor<T>,
			operator: T::AccountId,
			asset_id: T::AssetId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_schedule_delegator_bond_less(who.clone(), asset_id, amount)?;
			Self::deposit_event(Event::ScheduledDelegatorBondLess {
				who,
				asset_id,
				operator,
				amount,
			});
			Ok(())
		}

		/// Executes a scheduled request to reduce a delegator's bond.
		#[pallet::call_index(16)]
		#[pallet::weight(10_000)]
		pub fn execute_delegator_bond_less(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_execute_delegator_bond_less(who.clone())?;
			Self::deposit_event(Event::ExecutedDelegatorBondLess { who });
			Ok(())
		}

		/// Cancels a scheduled request to reduce a delegator's bond.
		#[pallet::call_index(17)]
		#[pallet::weight(10_000)]
		pub fn cancel_delegator_bond_less(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_cancel_delegator_bond_less(who.clone())?;
			Self::deposit_event(Event::CancelledDelegatorBondLess { who });
			Ok(())
		}
	}
}
