#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, LockableCurrency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, StaticLookup};
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;
		type BondAmount: Get<BalanceOf<Self>>;
		type BondDuration: Get<Self::BlockNumber>;
	}

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::storage]
	#[pallet::getter(fn operators)]
	pub type Operators<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Operator<T>>;

	#[pallet::storage]
	#[pallet::getter(fn operator_scheduled_leaves)]
	pub type OperatorScheduledLeaves<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::BlockNumber>;

	#[pallet::storage]
	#[pallet::getter(fn operator_bonds)]
	pub type OperatorBonds<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn delegations)]
	pub type Delegations<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		BalanceOf<T>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn delegation_scheduled_reductions)]
	pub type DelegationScheduledReductions<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		(BalanceOf<T>, T::BlockNumber),
	>;

	#[pallet::storage]
	#[pallet::getter(fn incentives)]
	pub type Incentives<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Incentive<BalanceOf<T>>>;

	#[pallet::storage]
	#[pallet::getter(fn deposit_caps)]
	pub type DepositCaps<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		OperatorJoined(T::AccountId),
		OperatorScheduledLeave(T::AccountId, T::BlockNumber),
		OperatorLeaveExecuted(T::AccountId),
		OperatorLeaveCancelled(T::AccountId),
		OperatorBondedMore(T::AccountId, BalanceOf<T>),
		OperatorBondReductionScheduled(T::AccountId, BalanceOf<T>, T::BlockNumber),
		OperatorBondReductionExecuted(T::AccountId, BalanceOf<T>),
		OperatorBondReductionCancelled(T::AccountId),
		OperatorWentOffline(T::AccountId),
		OperatorWentOnline(T::AccountId),
		AssetDeposited(T::AccountId, BalanceOf<T>),
		UnstakeScheduled(T::AccountId, BalanceOf<T>, T::BlockNumber),
		UnstakeExecuted(T::AccountId, BalanceOf<T>),
		UnstakeCancelled(T::AccountId),
		Delegated(T::AccountId, T::AccountId, BalanceOf<T>),
		DelegatorBondReductionScheduled(T::AccountId, T::AccountId, BalanceOf<T>, T::BlockNumber),
		DelegatorBondReductionExecuted(T::AccountId, T::AccountId, BalanceOf<T>),
		DelegatorBondReductionCancelled(T::AccountId, T::AccountId),
		IncentiveAPYSet(T::AccountId, BalanceOf<T>),
		DepositCapSet(T::AccountId, BalanceOf<T>),
		ServiceWhitelistedForRewards(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		AlreadyOperator,
		NotAnOperator,
		BondTooLow,
		NotEnoughBond,
		NoActiveServices,
		NotScheduledToLeave,
		NotScheduledToReduceBond,
		NoActiveDelegation,
		NotScheduledToUnstake,
		IncentiveAlreadySet,
		CapExceeded,
		ServiceNotWhitelisted,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn join_operators(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!Operators::<T>::contains_key(&who), Error::<T>::AlreadyOperator);
			T::Currency::reserve(&who, T::BondAmount::get())?;
			Operators::<T>::insert(&who, Operator::default());
			Self::deposit_event(Event::OperatorJoined(who));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn schedule_leave_operators(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Operators::<T>::contains_key(&who), Error::<T>::NotAnOperator);
			ensure!(Self::no_active_services(&who), Error::<T>::NoActiveServices);
			let leave_block = frame_system::Pallet::<T>::block_number() + T::BondDuration::get();
			OperatorScheduledLeaves::<T>::insert(&who, leave_block);
			Self::deposit_event(Event::OperatorScheduledLeave(who, leave_block));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn execute_leave_operators(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(
				OperatorScheduledLeaves::<T>::contains_key(&who),
				Error::<T>::NotScheduledToLeave
			);
			let leave_block =
				OperatorScheduledLeaves::<T>::get(&who).ok_or(Error::<T>::NotScheduledToLeave)?;
			ensure!(
				frame_system::Pallet::<T>::block_number() >= leave_block,
				Error::<T>::BondTooLow
			);
			OperatorScheduledLeaves::<T>::remove(&who);
			Operators::<T>::remove(&who);
			T::Currency::unreserve(&who, T::BondAmount::get());
			Self::deposit_event(Event::OperatorLeaveExecuted(who));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn cancel_leave_operators(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(
				OperatorScheduledLeaves::<T>::contains_key(&who),
				Error::<T>::NotScheduledToLeave
			);
			OperatorScheduledLeaves::<T>::remove(&who);
			Self::deposit_event(Event::OperatorLeaveCancelled(who));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn operator_bond_more(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Operators::<T>::contains_key(&who), Error::<T>::NotAnOperator);
			T::Currency::reserve(&who, amount)?;
			let new_bond = OperatorBonds::<T>::get(&who).unwrap_or_default() + amount;
			OperatorBonds::<T>::insert(&who, new_bond);
			Self::deposit_event(Event::OperatorBondedMore(who, amount));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn schedule_operator_bond_less(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Operators::<T>::contains_key(&who), Error::<T>::NotAnOperator);
			ensure!(Self::no_active_services(&who), Error::<T>::NoActiveServices);
			let current_bond = OperatorBonds::<T>::get(&who).ok_or(Error::<T>::NotEnoughBond)?;
			ensure!(current_bond >= amount, Error::<T>::NotEnoughBond);
			let leave_block = frame_system::Pallet::<T>::block_number() + T::BondDuration::get();
			OperatorBonds::<T>::insert(&who, current_bond - amount);
			Self::deposit_event(Event::OperatorBondReductionScheduled(who, amount, leave_block));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn execute_operator_bond_less(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(OperatorBonds::<T>::contains_key(&who), Error::<T>::NotScheduledToReduceBond);
			let leave_block =
				OperatorBonds::<T>::get(&who).ok_or(Error::<T>::NotScheduledToReduceBond)?;
			ensure!(
				frame_system::Pallet::<T>::block_number() >= leave_block,
				Error::<T>::BondTooLow
			);
			let current_bond = OperatorBonds::<T>::get(&who).ok_or(Error::<T>::NotEnoughBond)?;
			ensure!(current_bond >= amount, Error::<T>::NotEnoughBond);
			OperatorBonds::<T>::insert(&who, current_bond - amount);
			T::Currency::unreserve(&who, amount);
			Self::deposit_event(Event::OperatorBondReductionExecuted(who, amount));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn cancel_operator_bond_less(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(OperatorBonds::<T>::contains_key(&who), Error::<T>::NotScheduledToReduceBond);
			OperatorBonds::<T>::remove(&who);
			Self::deposit_event(Event::OperatorBondReductionCancelled(who));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn go_offline(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Operators::<T>::contains_key(&who), Error::<T>::NotAnOperator);
			// Implement logic to mark operator as offline
			Self::deposit_event(Event::OperatorWentOffline(who));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn go_online(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Operators::<T>::contains_key(&who), Error::<T>::NotAnOperator);
			// Implement logic to mark operator as online
			Self::deposit_event(Event::OperatorWentOnline(who));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn deposit(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			T::Currency::reserve(&who, amount)?;
			// Implement logic to handle deposit
			Self::deposit_event(Event::AssetDeposited(who, amount));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn schedule_unstake(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Implement logic to schedule unstake
			let unstake_block = frame_system::Pallet::<T>::block_number() + T::BondDuration::get();
			Self::deposit_event(Event::UnstakeScheduled(who, amount, unstake_block));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn execute_unstake(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Implement logic to execute unstake
			Self::deposit_event(Event::UnstakeExecuted(who, amount));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn cancel_unstake(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Implement logic to cancel unstake
			Self::deposit_event(Event::UnstakeCancelled(who));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn delegate(
			origin: OriginFor<T>,
			operator: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Operators::<T>::contains_key(&operator), Error::<T>::NotAnOperator);
			// Implement logic to delegate
			Self::deposit_event(Event::Delegated(who, operator, amount));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn schedule_delegator_bond_less(
			origin: OriginFor<T>,
			operator: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Operators::<T>::contains_key(&operator), Error::<T>::NotAnOperator);
			// Implement logic to schedule delegator bond less
			let reduction_block =
				frame_system::Pallet::<T>::block_number() + T::BondDuration::get();
			Self::deposit_event(Event::DelegatorBondReductionScheduled(
				who,
				operator,
				amount,
				reduction_block,
			));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn execute_delegator_bond_less(
			origin: OriginFor<T>,
			operator: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Operators::<T>::contains_key(&operator), Error::<T>::NotAnOperator);
			// Implement logic to execute delegator bond less
			Self::deposit_event(Event::DelegatorBondReductionExecuted(who, operator, amount));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn cancel_delegator_bond_less(
			origin: OriginFor<T>,
			operator: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Operators::<T>::contains_key(&operator), Error::<T>::NotAnOperator);
			// Implement logic to cancel delegator bond less
			Self::deposit_event(Event::DelegatorBondReductionCancelled(who, operator));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn set_incentive_apy(
			origin: OriginFor<T>,
			pool: T::AccountId,
			apy: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Incentives::<T>::contains_key(&pool), Error::<T>::IncentiveAlreadySet);
			// Implement logic to set incentive APY
			Self::deposit_event(Event::IncentiveAPYSet(who, apy));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn set_deposit_cap(
			origin: OriginFor<T>,
			pool: T::AccountId,
			cap: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Implement logic to set deposit cap
			Self::deposit_event(Event::DepositCapSet(who, cap));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn whitelist_blueprint_for_rewards(
			origin: OriginFor<T>,
			blueprint: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Implement logic to whitelist blueprint for rewards
			Self::deposit_event(Event::ServiceWhitelistedForRewards(blueprint));
			Ok(())
		}
	}
}
