// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
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

//! # Tangle Multi Asset Delegation Pallet
//!
//! This crate provides the delegation framework for the Tangle network, enabling the delegation of
//! assets to operators for earning rewards.
//!
//! ## Key Components
//!
//! - **Operators**: Operators are entities within the Tangle network that can receive delegated
//!   assets from delegators. They manage these assets, perform jobs and generate rewards for
//!   delegators.
//! - **Deposits**: Depositors must first reserve (deposit) their assets before they can delegate
//!   them to operators. This ensures that the assets are locked and available for delegation.
//! - **Delegation**: Delegation is the process where delegators assign their deposited assets to
//!   operators to earn rewards.
//!
//! ## Workflow for Delegators
//!
//! 1. **Deposit**: Before a delegator can delegate assets to an operator, they must first deposit
//!    the desired amount of assets. This reserves the assets in the delegator's account.
//! 2. **Delegate**: After depositing assets, the delegator can delegate these assets to an
//!    operator. The operator then manages these assets, and the delegator can earn rewards from the
//!    operator's activities.
//! 3. **Unstake**: If a delegator wants to reduce their delegation, they can schedule a unstake
//!    request. This request will be executed after a specified delay, ensuring network stability.
//! 4. **withdraw Request**: To completely remove assets from delegation, a delegator must submit an
//!    withdraw request. Similar to unstake requests, withdraw requests also have a delay before
//!    they can be executed.
//!
//! ## Workflow for Operators
//!
//! - **Join Operators**: An account can join as an operator by depositing a minimum stake amount.
//!   This stake is reserved and ensures that the operator has a stake in the network.
//! - **Leave Operators**: Operators can leave the network by scheduling a leave request. This
//!   request is subject to a delay, during which the operator's status changes to 'Leaving'.
//! - **Stake More**: Operators can increase their stake to strengthen their stake in the network.
//! - **Stake Less**: Operators can schedule a stake reduction request, which is executed after a
//!   delay.
//! - **Go Offline/Online**: Operators can change their status to offline if they need to
//!   temporarily stop participating in the network, and can come back online when ready.
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(any(test, feature = "fuzzing"))]
pub mod mock;

#[cfg(any(test, feature = "fuzzing"))]
pub mod mock_evm;

#[cfg(test)]
mod tests;

pub mod weights;

// #[cfg(feature = "runtime-benchmarks")]
// TODO(@1xstj): Fix benchmarking and re-enable
// mod benchmarking;

pub mod functions;
pub mod traits;
pub mod types;
pub use functions::*;

#[frame_support::pallet]
pub mod pallet {
	use crate::types::{delegator::DelegatorBlueprintSelection, AssetAction, *};
	use frame_support::{
		pallet_prelude::*,
		traits::{tokens::fungibles, Currency, Get, LockableCurrency, ReservableCurrency},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_core::H160;
	use sp_runtime::traits::{MaybeSerializeDeserialize, Member, Zero};
	use sp_std::{collections::btree_map::BTreeMap, fmt::Debug, prelude::*, vec::Vec};
	use tangle_primitives::types::rewards::LockMultiplier;
	use tangle_primitives::{services::Asset, traits::ServiceManager, BlueprintId, RoundIndex};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency type used for managing balances.
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;

		/// Type representing the unique ID of an asset.
		type AssetId: Parameter
			+ Member
			+ Copy
			+ MaybeSerializeDeserialize
			+ Ord
			+ Default
			+ MaxEncodedLen
			+ Encode
			+ Decode
			+ TypeInfo;

		/// The maximum number of blueprints a delegator can have in Fixed mode.
		#[pallet::constant]
		type MaxDelegatorBlueprints: Get<u32> + TypeInfo + MaxEncodedLen + Clone + Debug + PartialEq;

		/// The maximum number of blueprints an operator can support.
		#[pallet::constant]
		type MaxOperatorBlueprints: Get<u32> + TypeInfo + MaxEncodedLen + Clone + Debug + PartialEq;

		/// The maximum number of withdraw requests a delegator can have.
		#[pallet::constant]
		type MaxWithdrawRequests: Get<u32> + TypeInfo + MaxEncodedLen + Clone + Debug + PartialEq;

		/// The maximum number of delegations a delegator can have.
		#[pallet::constant]
		type MaxDelegations: Get<u32> + TypeInfo + MaxEncodedLen + Clone + Debug + PartialEq;

		/// The maximum number of unstake requests a delegator can have.
		#[pallet::constant]
		type MaxUnstakeRequests: Get<u32> + TypeInfo + MaxEncodedLen + Clone + Debug + PartialEq;

		/// The minimum amount of stake required for an operator.
		#[pallet::constant]
		type MinOperatorBondAmount: Get<BalanceOf<Self>>;

		/// The minimum amount of stake required for a delegate.
		#[pallet::constant]
		type MinDelegateAmount: Get<BalanceOf<Self>>;

		/// The duration for which the stake is locked.
		#[pallet::constant]
		type BondDuration: Get<RoundIndex>;

		/// The service manager that manages active services.
		type ServiceManager: ServiceManager<Self::AccountId, BalanceOf<Self>>;

		/// Number of rounds that operators remain bonded before the exit request is executable.
		#[pallet::constant]
		type LeaveOperatorsDelay: Get<RoundIndex>;

		/// Number of rounds operator requests to decrease self-stake must wait to be executable.
		#[pallet::constant]
		type OperatorBondLessDelay: Get<RoundIndex>;

		/// Number of rounds that delegators remain bonded before the exit request is executable.
		#[pallet::constant]
		type LeaveDelegatorsDelay: Get<RoundIndex>;

		/// Number of rounds that delegation unstake requests must wait before being executable.
		#[pallet::constant]
		type DelegationBondLessDelay: Get<RoundIndex>;

		/// The fungibles trait used for managing fungible assets.
		type Fungibles: fungibles::Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = BalanceOf<Self>>
			+ fungibles::Mutate<Self::AccountId, AssetId = Self::AssetId>;

		/// The pallet's account ID.
		type PalletId: Get<PalletId>;

		/// The origin with privileged access
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The address that receives slashed funds
		type SlashedAmountRecipient: Get<Self::AccountId>;

		/// A type that implements the `EvmRunner` trait for the execution of EVM
		/// transactions.
		type EvmRunner: tangle_primitives::services::EvmRunner<Self>;

		/// A type that implements the `EvmGasWeightMapping` trait for the conversion of EVM gas to
		/// Substrate weight and vice versa.
		type EvmGasWeightMapping: tangle_primitives::services::EvmGasWeightMapping;

		/// A type that implements the `EvmAddressMapping` trait for the conversion of EVM address
		type EvmAddressMapping: tangle_primitives::services::EvmAddressMapping<Self::AccountId>;

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
	pub type Operators<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, OperatorMetadataOf<T>>;

	/// Storage for the current round.
	#[pallet::storage]
	#[pallet::getter(fn current_round)]
	pub type CurrentRound<T: Config> = StorageValue<_, RoundIndex, ValueQuery>;

	/// Snapshot of collator delegation stake at the start of the round.
	#[pallet::storage]
	#[pallet::getter(fn at_stake)]
	pub type AtStake<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		RoundIndex,
		Blake2_128Concat,
		T::AccountId,
		OperatorSnapshotOf<T>,
		OptionQuery,
	>;

	/// Storage for delegator information.
	#[pallet::storage]
	#[pallet::getter(fn delegators)]
	pub type Delegators<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, DelegatorMetadataOf<T>>;

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
		/// An operator has increased their stake.
		OperatorBondMore { who: T::AccountId, additional_bond: BalanceOf<T> },
		/// An operator has scheduled to decrease their stake.
		OperatorBondLessScheduled { who: T::AccountId, unstake_amount: BalanceOf<T> },
		/// An operator has executed their stake decrease.
		OperatorBondLessExecuted { who: T::AccountId },
		/// An operator has cancelled their stake decrease request.
		OperatorBondLessCancelled { who: T::AccountId },
		/// An operator has gone offline.
		OperatorWentOffline { who: T::AccountId },
		/// An operator has gone online.
		OperatorWentOnline { who: T::AccountId },
		/// A deposit has been made.
		Deposited { who: T::AccountId, amount: BalanceOf<T>, asset_id: Asset<T::AssetId> },
		/// An withdraw has been scheduled.
		Scheduledwithdraw { who: T::AccountId, amount: BalanceOf<T>, asset_id: Asset<T::AssetId> },
		/// An withdraw has been executed.
		Executedwithdraw { who: T::AccountId },
		/// An withdraw has been cancelled.
		Cancelledwithdraw { who: T::AccountId },
		/// A delegation has been made.
		Delegated {
			who: T::AccountId,
			operator: T::AccountId,
			amount: BalanceOf<T>,
			asset_id: Asset<T::AssetId>,
		},
		/// A delegator unstake request has been scheduled.
		ScheduledDelegatorBondLess {
			who: T::AccountId,
			operator: T::AccountId,
			amount: BalanceOf<T>,
			asset_id: Asset<T::AssetId>,
		},
		/// A delegator unstake request has been executed.
		ExecutedDelegatorBondLess { who: T::AccountId },
		/// A delegator unstake request has been cancelled.
		CancelledDelegatorBondLess { who: T::AccountId },
		/// Operator has been slashed
		OperatorSlashed { who: T::AccountId, amount: BalanceOf<T> },
		/// Delegator has been slashed
		DelegatorSlashed { who: T::AccountId, amount: BalanceOf<T> },
		/// EVM execution reverted with a reason.
		EvmReverted { from: H160, to: H160, data: Vec<u8>, reason: Vec<u8> },
	}

	/// Errors emitted by the pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// The account is already an operator.
		AlreadyOperator,
		/// The stake amount is too low.
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
		/// Leaving round not reached
		LeavingRoundNotReached,
		/// There is no scheduled unstake request.
		NoScheduledBondLess,
		/// The unstake request is not satisfied.
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
		/// There is no unstake request.
		NoBondLessRequest,
		/// The unstake request is not ready.
		BondLessNotReady,
		/// A unstake request already exists.
		BondLessRequestAlreadyExists,
		/// There are active services using the asset.
		ActiveServicesUsingAsset,
		/// There is not active delegation
		NoActiveDelegation,
		/// The asset is not whitelisted
		AssetNotWhitelisted,
		/// The origin is not authorized to perform this action
		NotAuthorized,
		/// Maximum number of blueprints exceeded
		MaxBlueprintsExceeded,
		/// The asset ID is not found
		AssetNotFound,
		/// The blueprint ID is already whitelisted
		BlueprintAlreadyWhitelisted,
		/// No withdraw requests found
		NowithdrawRequests,
		/// No matching withdraw reqests found
		NoMatchingwithdrawRequest,
		/// Asset already exists in a reward vault
		AssetAlreadyInVault,
		/// Asset not found in reward vault
		AssetNotInVault,
		/// The reward vault does not exist
		VaultNotFound,
		/// Error returned when trying to add a blueprint ID that already exists.
		DuplicateBlueprintId,
		/// Error returned when trying to remove a blueprint ID that doesn't exist.
		BlueprintIdNotFound,
		/// Error returned when trying to add/remove blueprint IDs while not in Fixed mode.
		NotInFixedMode,
		/// Error returned when the maximum number of delegations is exceeded.
		MaxDelegationsExceeded,
		/// Error returned when the maximum number of unstake requests is exceeded.
		MaxUnstakeRequestsExceeded,
		/// Error returned when the maximum number of withdraw requests is exceeded.
		MaxWithdrawRequestsExceeded,
		/// Deposit amount overflow
		DepositOverflow,
		/// Unstake underflow
		UnstakeAmountTooLarge,
		/// Overflow while adding stake
		StakeOverflow,
		/// Underflow while reducing stake
		InsufficientStakeRemaining,
		/// APY exceeds maximum allowed by the extrinsic
		APYExceedsMaximum,
		/// Cap cannot be zero
		CapCannotBeZero,
		/// Cap exceeds total supply of asset
		CapExceedsTotalSupply,
		/// An unstake request is already pending
		PendingUnstakeRequestExists,
		/// The blueprint is not selected
		BlueprintNotSelected,
		/// Erc20 transfer failed
		ERC20TransferFailed,
		/// EVM encode error
		EVMAbiEncode,
		/// EVM decode error
		EVMAbiDecode,
		/// Cannot unstake with locks
		LockViolation,
	}

	/// Hooks for the pallet.
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// The callable functions (extrinsics) of the pallet.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Allows an account to join as an operator by staking the required bond amount.
		///
		/// # Permissions
		///
		/// * Must be signed by the account joining as operator
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `bond_amount` - Amount to stake as operator bond
		///
		/// # Errors
		///
		/// * [`Error::DepositOverflow`] - Bond amount would overflow deposit tracking
		/// * [`Error::StakeOverflow`] - Bond amount would overflow stake tracking
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn join_operators(origin: OriginFor<T>, bond_amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::handle_deposit_and_create_operator(who.clone(), bond_amount)?;
			Self::deposit_event(Event::OperatorJoined { who });
			Ok(())
		}

		/// Schedules an operator to leave the system.
		///
		/// # Permissions
		///
		/// * Must be signed by the operator account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		///
		/// # Errors
		///
		/// * [`Error::NotOperator`] - Account is not registered as an operator
		/// * [`Error::PendingUnstakeRequestExists`] - Operator already has a pending unstake request
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn schedule_leave_operators(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_leave_operator(&who)?;
			Self::deposit_event(Event::OperatorLeavingScheduled { who });
			Ok(())
		}

		/// Cancels a scheduled leave for an operator.
		///
		/// # Permissions
		///
		/// * Must be signed by the operator account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		///
		/// # Errors
		///
		/// * [`Error::NotOperator`] - Account is not registered as an operator
		/// * [`Error::NoUnstakeRequestExists`] - No pending unstake request exists
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn cancel_leave_operators(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_cancel_leave_operator(&who)?;
			Self::deposit_event(Event::OperatorLeaveCancelled { who });
			Ok(())
		}

		/// Executes a scheduled leave for an operator.
		///
		/// # Permissions
		///
		/// * Must be signed by the operator account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		///
		/// # Errors
		///
		/// * [`Error::NotOperator`] - Account is not registered as an operator
		/// * [`Error::NoUnstakeRequestExists`] - No pending unstake request exists
		/// * [`Error::UnstakePeriodNotElapsed`] - Unstake period has not elapsed yet
		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn execute_leave_operators(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_execute_leave_operators(&who)?;
			Self::deposit_event(Event::OperatorLeaveExecuted { who });
			Ok(())
		}

		/// Allows an operator to increase their stake.
		///
		/// # Permissions
		///
		/// * Must be signed by the operator account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `additional_bond` - Additional amount to stake
		///
		/// # Errors
		///
		/// * [`Error::NotOperator`] - Account is not registered as an operator
		/// * [`Error::StakeOverflow`] - Additional bond would overflow stake tracking
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

		/// Schedules an operator to decrease their stake.
		///
		/// # Permissions
		///
		/// * Must be signed by the operator account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `unstake_amount` - Amount to unstake
		///
		/// # Errors
		///
		/// * [`Error::NotOperator`] - Account is not registered as an operator
		/// * [`Error::PendingUnstakeRequestExists`] - Operator already has a pending unstake request
		/// * [`Error::InsufficientBalance`] - Operator has insufficient stake to unstake
		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn schedule_operator_unstake(
			origin: OriginFor<T>,
			unstake_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_schedule_operator_unstake(&who, unstake_amount)?;
			Self::deposit_event(Event::OperatorBondLessScheduled { who, unstake_amount });
			Ok(())
		}

		/// Executes a scheduled stake decrease for an operator.
		///
		/// # Permissions
		///
		/// * Must be signed by the operator account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		///
		/// # Errors
		///
		/// * [`Error::NotOperator`] - Account is not registered as an operator
		/// * [`Error::NoUnstakeRequestExists`] - No pending unstake request exists
		/// * [`Error::UnstakePeriodNotElapsed`] - Unstake period has not elapsed yet
		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn execute_operator_unstake(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_execute_operator_unstake(&who)?;
			Self::deposit_event(Event::OperatorBondLessExecuted { who });
			Ok(())
		}

		/// Cancels a scheduled stake decrease for an operator.
		///
		/// # Permissions
		///
		/// * Must be signed by the operator account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		///
		/// # Errors
		///
		/// * [`Error::NotOperator`] - Account is not registered as an operator
		/// * [`Error::NoUnstakeRequestExists`] - No pending unstake request exists
		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn cancel_operator_unstake(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_cancel_operator_unstake(&who)?;
			Self::deposit_event(Event::OperatorBondLessCancelled { who });
			Ok(())
		}

		/// Allows an operator to go offline.
		///
		/// # Permissions
		///
		/// * Must be signed by the operator account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		///
		/// # Errors
		///
		/// * [`Error::NotOperator`] - Account is not registered as an operator
		/// * [`Error::AlreadyOffline`] - Operator is already offline
		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn go_offline(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_go_offline(&who)?;
			Self::deposit_event(Event::OperatorWentOffline { who });
			Ok(())
		}

		/// Allows an operator to go online.
		///
		/// # Permissions
		///
		/// * Must be signed by the operator account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		///
		/// # Errors
		///
		/// * [`Error::NotOperator`] - Account is not registered as an operator
		/// * [`Error::AlreadyOnline`] - Operator is already online
		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn go_online(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_go_online(&who)?;
			Self::deposit_event(Event::OperatorWentOnline { who });
			Ok(())
		}

		/// Allows a user to deposit an asset.
		///
		/// # Permissions
		///
		/// * Must be signed by the depositor account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `asset_id` - ID of the asset to deposit
		/// * `amount` - Amount to deposit
		/// * `evm_address` - Optional EVM address
		///
		/// # Errors
		///
		/// * [`Error::DepositOverflow`] - Deposit would overflow tracking
		/// * [`Error::InvalidAsset`] - Asset is not supported
		#[pallet::call_index(10)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn deposit(
			origin: OriginFor<T>,
			asset_id: Asset<T::AssetId>,
			amount: BalanceOf<T>,
			evm_address: Option<H160>,
			lock_multiplier: Option<LockMultiplier>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_deposit(who.clone(), asset_id, amount, evm_address, lock_multiplier)?;
			Self::deposit_event(Event::Deposited { who, amount, asset_id });
			Ok(())
		}

		/// Schedules a withdraw request.
		///
		/// # Permissions
		///
		/// * Must be signed by the withdrawer account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `asset_id` - ID of the asset to withdraw
		/// * `amount` - Amount to withdraw
		///
		/// # Errors
		///
		/// * [`Error::InsufficientBalance`] - Insufficient balance to withdraw
		/// * [`Error::PendingWithdrawRequestExists`] - Pending withdraw request exists
		#[pallet::call_index(11)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn schedule_withdraw(
			origin: OriginFor<T>,
			asset_id: Asset<T::AssetId>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_schedule_withdraw(who.clone(), asset_id, amount)?;
			Self::deposit_event(Event::Scheduledwithdraw { who, amount, asset_id });
			Ok(())
		}

		/// Executes a scheduled withdraw request.
		///
		/// # Permissions
		///
		/// * Must be signed by the withdrawer account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `evm_address` - Optional EVM address
		///
		/// # Errors
		///
		/// * [`Error::NoWithdrawRequestExists`] - No pending withdraw request exists
		/// * [`Error::WithdrawPeriodNotElapsed`] - Withdraw period has not elapsed
		#[pallet::call_index(12)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn execute_withdraw(origin: OriginFor<T>, evm_address: Option<H160>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_execute_withdraw(who.clone(), evm_address)?;
			Self::deposit_event(Event::Executedwithdraw { who });
			Ok(())
		}

		/// Cancels a scheduled withdraw request.
		///
		/// # Permissions
		///
		/// * Must be signed by the withdrawer account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `asset_id` - ID of the asset withdrawal to cancel
		/// * `amount` - Amount of the withdrawal to cancel
		///
		/// # Errors
		///
		/// * [`Error::NoWithdrawRequestExists`] - No pending withdraw request exists
		#[pallet::call_index(13)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn cancel_withdraw(
			origin: OriginFor<T>,
			asset_id: Asset<T::AssetId>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_cancel_withdraw(who.clone(), asset_id, amount)?;
			Self::deposit_event(Event::Cancelledwithdraw { who });
			Ok(())
		}

		/// Allows a user to delegate an amount of an asset to an operator.
		///
		/// # Permissions
		///
		/// * Must be signed by the delegator account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `operator` - Operator to delegate to
		/// * `asset_id` - ID of asset to delegate
		/// * `amount` - Amount to delegate
		/// * `blueprint_selection` - Blueprint selection strategy
		///
		/// # Errors
		///
		/// * [`Error::NotOperator`] - Target account is not an operator
		/// * [`Error::InsufficientBalance`] - Insufficient balance to delegate
		/// * [`Error::MaxDelegationsExceeded`] - Would exceed max delegations
		#[pallet::call_index(14)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn delegate(
			origin: OriginFor<T>,
			operator: T::AccountId,
			asset_id: Asset<T::AssetId>,
			amount: BalanceOf<T>,
			blueprint_selection: DelegatorBlueprintSelection<T::MaxDelegatorBlueprints>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_delegate(
				who.clone(),
				operator.clone(),
				asset_id,
				amount,
				blueprint_selection,
			)?;
			Self::deposit_event(Event::Delegated { who, operator, asset_id, amount });
			Ok(())
		}

		/// Schedules a request to reduce a delegator's stake.
		///
		/// # Permissions
		///
		/// * Must be signed by the delegator account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `operator` - Operator to unstake from
		/// * `asset_id` - ID of asset to unstake
		/// * `amount` - Amount to unstake
		///
		/// # Errors
		///
		/// * [`Error::NotDelegator`] - Account is not a delegator
		/// * [`Error::InsufficientDelegation`] - Insufficient delegation to unstake
		/// * [`Error::PendingUnstakeRequestExists`] - Pending unstake request exists
		#[pallet::call_index(15)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn schedule_delegator_unstake(
			origin: OriginFor<T>,
			operator: T::AccountId,
			asset_id: Asset<T::AssetId>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_schedule_delegator_unstake(
				who.clone(),
				operator.clone(),
				asset_id,
				amount,
			)?;
			Self::deposit_event(Event::ScheduledDelegatorBondLess {
				who,
				asset_id,
				operator,
				amount,
			});
			Ok(())
		}

		/// Executes a scheduled request to reduce a delegator's stake.
		///
		/// # Permissions
		///
		/// * Must be signed by the delegator account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		///
		/// # Errors
		///
		/// * [`Error::NotDelegator`] - Account is not a delegator
		/// * [`Error::NoUnstakeRequestExists`] - No pending unstake request exists
		/// * [`Error::UnstakePeriodNotElapsed`] - Unstake period has not elapsed
		#[pallet::call_index(16)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn execute_delegator_unstake(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_execute_delegator_unstake(who.clone())?;
			Self::deposit_event(Event::ExecutedDelegatorBondLess { who });
			Ok(())
		}

		/// Cancels a scheduled request to reduce a delegator's stake.
		///
		/// # Permissions
		///
		/// * Must be signed by the delegator account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `operator` - Operator to cancel unstake from
		/// * `asset_id` - ID of asset unstake to cancel
		/// * `amount` - Amount of unstake to cancel
		///
		/// # Errors
		///
		/// * [`Error::NotDelegator`] - Account is not a delegator
		/// * [`Error::NoUnstakeRequestExists`] - No pending unstake request exists
		#[pallet::call_index(17)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn cancel_delegator_unstake(
			origin: OriginFor<T>,
			operator: T::AccountId,
			asset_id: Asset<T::AssetId>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_cancel_delegator_unstake(who.clone(), operator, asset_id, amount)?;
			Self::deposit_event(Event::CancelledDelegatorBondLess { who });
			Ok(())
		}

		/// Adds a blueprint ID to a delegator's selection.
		///
		/// # Permissions
		///
		/// * Must be signed by the delegator account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `blueprint_id` - ID of blueprint to add
		///
		/// # Errors
		///
		/// * [`Error::NotDelegator`] - Account is not a delegator
		/// * [`Error::DuplicateBlueprintId`] - Blueprint ID already exists
		/// * [`Error::MaxBlueprintsExceeded`] - Would exceed max blueprints
		/// * [`Error::NotInFixedMode`] - Not in fixed blueprint selection mode
		#[pallet::call_index(22)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn add_blueprint_id(origin: OriginFor<T>, blueprint_id: BlueprintId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut metadata = Self::delegators(&who).ok_or(Error::<T>::NotDelegator)?;

			for delegation in metadata.delegations.iter_mut() {
				match delegation.blueprint_selection {
					DelegatorBlueprintSelection::Fixed(ref mut ids) => {
						ensure!(!ids.contains(&blueprint_id), Error::<T>::DuplicateBlueprintId);
						ids.try_push(blueprint_id)
							.map_err(|_| Error::<T>::MaxBlueprintsExceeded)?;
					},
					_ => return Err(Error::<T>::NotInFixedMode.into()),
				}
			}

			Delegators::<T>::insert(&who, metadata);
			Ok(())
		}

		/// Removes a blueprint ID from a delegator's selection.
		///
		/// # Permissions
		///
		/// * Must be signed by the delegator account
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `blueprint_id` - ID of blueprint to remove
		///
		/// # Errors
		///
		/// * [`Error::NotDelegator`] - Account is not a delegator
		/// * [`Error::BlueprintIdNotFound`] - Blueprint ID not found
		/// * [`Error::NotInFixedMode`] - Not in fixed blueprint selection mode
		#[pallet::call_index(23)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn remove_blueprint_id(
			origin: OriginFor<T>,
			blueprint_id: BlueprintId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut metadata = Self::delegators(&who).ok_or(Error::<T>::NotDelegator)?;

			for delegation in metadata.delegations.iter_mut() {
				match delegation.blueprint_selection {
					DelegatorBlueprintSelection::Fixed(ref mut ids) => {
						let pos = ids
							.iter()
							.position(|&id| id == blueprint_id)
							.ok_or(Error::<T>::BlueprintIdNotFound)?;
						ids.remove(pos);
					},
					_ => return Err(Error::<T>::NotInFixedMode.into()),
				}
			}

			Delegators::<T>::insert(&who, metadata);
			Ok(())
		}
	}
}
