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

//! # Tangle Multi Asset Delegation Pallet
//!
//! This crate provides the delegation framework for the Tangle network, enabling the delegation of assets to operators for earning rewards.
//!
//! ## Key Components
//!
//! - **Operators**: Operators are entities within the Tangle network that can receive delegated assets from delegators. They manage these assets, perform jobs and generate rewards for delegators.
//! - **Deposits**: Depositors must first reserve (deposit) their assets before they can delegate them to operators. This ensures that the assets are locked and available for delegation.
//! - **Delegation**: Delegation is the process where delegators assign their deposited assets to operators to earn rewards.
//!
//! ## Workflow for Delegators
//!
//! 1. **Deposit**: Before a delegator can delegate assets to an operator, they must first deposit the desired amount of assets. This reserves the assets in the delegator's account.
//! 2. **Delegate**: After depositing assets, the delegator can delegate these assets to an operator. The operator then manages these assets, and the delegator can earn rewards from the operator's activities.
//! 3. **Bond Less Request**: If a delegator wants to reduce their delegation, they can schedule a bond less request. This request will be executed after a specified delay, ensuring network stability.
//! 4. **Unstake Request**: To completely remove assets from delegation, a delegator must submit an unstake request. Similar to bond less requests, unstake requests also have a delay before they can be executed.
//!
//! ## Workflow for Operators
//!
//! - **Join Operators**: An account can join as an operator by depositing a minimum bond amount. This bond is reserved and ensures that the operator has a stake in the network.
//! - **Leave Operators**: Operators can leave the network by scheduling a leave request. This request is subject to a delay, during which the operator's status changes to 'Leaving'.
//! - **Bond More**: Operators can increase their bond to strengthen their stake in the network.
//! - **Bond Less**: Operators can schedule a bond reduction request, which is executed after a delay.
//! - **Go Offline/Online**: Operators can change their status to offline if they need to temporarily stop participating in the network, and can come back online when ready.
//!
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
		traits::{Currency, LockableCurrency, ReservableCurrency},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize};
	use sp_std::collections::btree_map::BTreeMap;
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
		#[pallet::constant]
		type MinOperatorBondAmount: Get<BalanceOf<Self>>;

		/// The minimum amount of bond required for a delegate.
		#[pallet::constant]
		type MinDelegateAmount: Get<BalanceOf<Self>>;

		/// The duration for which the bond is locked.
		#[pallet::constant]
		type BondDuration: Get<RoundIndex>;

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

		/// The origin with privileged access
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

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
		StorageMap<_, Twox64Concat, T::AccountId, OperatorMetadataOf<T>, OptionQuery>;

	/// Storage for the current round.
	#[pallet::storage]
	#[pallet::getter(fn current_round)]
	pub type CurrentRound<T: Config> = StorageValue<_, RoundIndex, ValueQuery>;

	/// Whitelisted assets that are allowed to be deposited
	#[pallet::storage]
	#[pallet::getter(fn whitelisted_assets)]
	pub type WhitelistedAssets<T: Config> = StorageValue<_, Vec<T::AssetId>, ValueQuery>;

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

	#[pallet::storage]
	#[pallet::getter(fn reward_config)]
	/// Storage for the reward configuration, which includes APY, cap for assets, and whitelisted blueprints.
	pub type RewardConfigStorage<T: Config> =
		StorageValue<_, RewardConfig<T::AssetId, BalanceOf<T>>, OptionQuery>;

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
		/// New whitelisted assets set
		WhitelistedAssetsSet { assets: Vec<T::AssetId> },
		/// Event emitted when an incentive APY and cap are set for an asset
		IncentiveAPYAndCapSet { asset_id: T::AssetId, apy: u128, cap: BalanceOf<T> },
		/// Event emitted when a blueprint is whitelisted for rewards
		BlueprintWhitelisted { blueprint_id: u32 },
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
		/// There is not active delegation
		NoActiveDelegation,
		/// The asset is not whitelisted
		AssetNotWhitelisted,
		/// The origin is not authorized to perform this action
		NotAuthorized,
		/// The asset ID is not found
		AssetNotFound,
		/// The blueprint ID is already whitelisted
		BlueprintAlreadyWhitelisted,
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
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
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
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn execute_unstake(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_execute_unstake(who.clone())?;
			Self::deposit_event(Event::ExecutedUnstake { who });
			Ok(())
		}

		/// Cancels a scheduled unstake request.
		#[pallet::call_index(13)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn cancel_unstake(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_cancel_unstake(who.clone())?;
			Self::deposit_event(Event::CancelledUnstake { who });
			Ok(())
		}

		/// Allows a user to delegate an amount of an asset to an operator.
		#[pallet::call_index(14)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
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
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn schedule_delegator_bond_less(
			origin: OriginFor<T>,
			operator: T::AccountId,
			asset_id: T::AssetId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_schedule_delegator_bond_less(
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

		/// Executes a scheduled request to reduce a delegator's bond.
		#[pallet::call_index(16)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn execute_delegator_bond_less(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_execute_delegator_bond_less(who.clone())?;
			Self::deposit_event(Event::ExecutedDelegatorBondLess { who });
			Ok(())
		}

		/// Cancels a scheduled request to reduce a delegator's bond.
		#[pallet::call_index(17)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn cancel_delegator_bond_less(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_cancel_delegator_bond_less(who.clone())?;
			Self::deposit_event(Event::CancelledDelegatorBondLess { who });
			Ok(())
		}

		/// Set the whitelisted assets allowed for delegation
		#[pallet::call_index(18)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn set_whitelisted_assets(
			origin: OriginFor<T>,
			assets: Vec<T::AssetId>,
		) -> DispatchResult {
			// Ensure that the origin is authorized
			T::ForceOrigin::ensure_origin(origin)?;

			// Set the whitelisted assets
			WhitelistedAssets::<T>::put(assets.clone());

			// Emit an event
			Self::deposit_event(Event::WhitelistedAssetsSet { assets });

			Ok(())
		}

		/// Sets the APY and cap for a specific asset.
		#[pallet::call_index(19)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn set_incentive_apy_and_cap(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			apy: u128,
			cap: BalanceOf<T>,
		) -> DispatchResult {
			// Ensure that the origin is authorized
			T::ForceOrigin::ensure_origin(origin)?;

			// Initialize the reward config if not already initialized
			RewardConfigStorage::<T>::mutate(|maybe_config| {
				let mut config = maybe_config.take().unwrap_or_else(|| RewardConfig {
					configs: BTreeMap::new(),
					whitelisted_blueprint_ids: Vec::new(),
				});

				config.configs.insert(asset_id, RewardConfigForAsset { apy, cap });

				*maybe_config = Some(config);
			});

			// Emit an event
			Self::deposit_event(Event::IncentiveAPYAndCapSet { asset_id, apy, cap });

			Ok(())
		}

		/// Whitelists a blueprint for rewards.
		#[pallet::call_index(20)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn whitelist_blueprint_for_rewards(
			origin: OriginFor<T>,
			blueprint_id: u32,
		) -> DispatchResult {
			// Ensure that the origin is authorized
			T::ForceOrigin::ensure_origin(origin)?;

			// Initialize the reward config if not already initialized
			RewardConfigStorage::<T>::mutate(|maybe_config| {
				let mut config = maybe_config.take().unwrap_or_else(|| RewardConfig {
					configs: BTreeMap::new(),
					whitelisted_blueprint_ids: Vec::new(),
				});

				if !config.whitelisted_blueprint_ids.contains(&blueprint_id) {
					config.whitelisted_blueprint_ids.push(blueprint_id);
				}

				*maybe_config = Some(config);
			});

			// Emit an event
			Self::deposit_event(Event::BlueprintWhitelisted { blueprint_id });

			Ok(())
		}
	}
}
