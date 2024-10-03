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

#[cfg(test)]
mod mock;

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
	use crate::types::*;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{fungibles, Currency, LockableCurrency, ReservableCurrency},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize};
	use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
	use tangle_primitives::{traits::ServiceManager, RoundIndex};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency type used for managing balances.
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;

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

		/// The asset ID type.
		type AssetId: AtLeast32BitUnsigned
			+ Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ Clone
			+ Copy
			+ PartialOrd
			+ MaxEncodedLen;

		/// The vault ID type.
		type VaultId: AtLeast32BitUnsigned
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
	pub type Operators<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, OperatorMetadataOf<T>, OptionQuery>;

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
	pub type Delegators<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, DelegatorMetadataOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn reward_vaults)]
	/// Storage for the reward vaults
	pub type RewardVaults<T: Config> =
		StorageMap<_, Twox64Concat, T::VaultId, Vec<T::AssetId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn asset_reward_vault_lookup)]
	/// Storage for the reward vaults
	pub type AssetLookupRewardVaults<T: Config> =
		StorageMap<_, Twox64Concat, T::AssetId, T::VaultId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn reward_config)]
	/// Storage for the reward configuration, which includes APY, cap for assets, and whitelisted
	/// blueprints.
	pub type RewardConfigStorage<T: Config> =
		StorageValue<_, RewardConfig<T::VaultId, BalanceOf<T>>, OptionQuery>;

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
		Deposited { who: T::AccountId, amount: BalanceOf<T>, asset_id: T::AssetId },
		/// An withdraw has been scheduled.
		Scheduledwithdraw { who: T::AccountId, amount: BalanceOf<T>, asset_id: T::AssetId },
		/// An withdraw has been executed.
		Executedwithdraw { who: T::AccountId },
		/// An withdraw has been cancelled.
		Cancelledwithdraw { who: T::AccountId },
		/// A delegation has been made.
		Delegated {
			who: T::AccountId,
			operator: T::AccountId,
			amount: BalanceOf<T>,
			asset_id: T::AssetId,
		},
		/// A delegator unstake request has been scheduled.
		ScheduledDelegatorBondLess {
			who: T::AccountId,
			operator: T::AccountId,
			amount: BalanceOf<T>,
			asset_id: T::AssetId,
		},
		/// A delegator unstake request has been executed.
		ExecutedDelegatorBondLess { who: T::AccountId },
		/// A delegator unstake request has been cancelled.
		CancelledDelegatorBondLess { who: T::AccountId },
		/// Event emitted when an incentive APY and cap are set for a reward vault
		IncentiveAPYAndCapSet { vault_id: T::VaultId, apy: sp_runtime::Percent, cap: BalanceOf<T> },
		/// Event emitted when a blueprint is whitelisted for rewards
		BlueprintWhitelisted { blueprint_id: u32 },
		/// Asset has been updated to reward vault
		AssetUpdatedInVault {
			who: T::AccountId,
			vault_id: T::VaultId,
			asset_id: T::AssetId,
			action: AssetAction,
		},
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
	}

	/// Hooks for the pallet.
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// The callable functions (extrinsics) of the pallet.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Allows an account to join as an operator by providing a stake.
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

		/// Allows an operator to increase their stake.
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
		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn execute_operator_unstake(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_execute_operator_unstake(&who)?;
			Self::deposit_event(Event::OperatorBondLessExecuted { who });
			Ok(())
		}

		/// Cancels a scheduled stake decrease for an operator.
		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn cancel_operator_unstake(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_cancel_operator_unstake(&who)?;
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
			asset_id: T::AssetId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_deposit(who.clone(), asset_id, amount)?;
			Self::deposit_event(Event::Deposited { who, amount, asset_id });
			Ok(())
		}

		/// Schedules an withdraw request.
		#[pallet::call_index(11)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn schedule_withdraw(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_schedule_withdraw(who.clone(), asset_id, amount)?;
			Self::deposit_event(Event::Scheduledwithdraw { who, amount, asset_id });
			Ok(())
		}

		/// Executes a scheduled withdraw request.
		#[pallet::call_index(12)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn execute_withdraw(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_execute_withdraw(who.clone())?;
			Self::deposit_event(Event::Executedwithdraw { who });
			Ok(())
		}

		/// Cancels a scheduled withdraw request.
		#[pallet::call_index(13)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn cancel_withdraw(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_cancel_withdraw(who.clone(), asset_id, amount)?;
			Self::deposit_event(Event::Cancelledwithdraw { who });
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

		/// Schedules a request to reduce a delegator's stake.
		#[pallet::call_index(15)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn schedule_delegator_unstake(
			origin: OriginFor<T>,
			operator: T::AccountId,
			asset_id: T::AssetId,
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
		#[pallet::call_index(16)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn execute_delegator_unstake(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_execute_delegator_unstake(who.clone())?;
			Self::deposit_event(Event::ExecutedDelegatorBondLess { who });
			Ok(())
		}

		/// Cancels a scheduled request to reduce a delegator's stake.
		#[pallet::call_index(17)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn cancel_delegator_unstake(
			origin: OriginFor<T>,
			operator: T::AccountId,
			asset_id: T::AssetId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::process_cancel_delegator_unstake(who.clone(), operator, asset_id, amount)?;
			Self::deposit_event(Event::CancelledDelegatorBondLess { who });
			Ok(())
		}

		/// Sets the APY and cap for a specific asset.
		#[pallet::call_index(19)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn set_incentive_apy_and_cap(
			origin: OriginFor<T>,
			vault_id: T::VaultId,
			apy: sp_runtime::Percent,
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

				config.configs.insert(vault_id, RewardConfigForAssetVault { apy, cap });

				*maybe_config = Some(config);
			});

			// Emit an event
			Self::deposit_event(Event::IncentiveAPYAndCapSet { vault_id, apy, cap });

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

		/// Manage asset id to vault rewards
		#[pallet::call_index(21)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn manage_asset_in_vault(
			origin: OriginFor<T>,
			vault_id: T::VaultId,
			asset_id: T::AssetId,
			action: AssetAction,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match action {
				AssetAction::Add => Self::add_asset_to_vault(&vault_id, &asset_id)?,
				AssetAction::Remove => Self::remove_asset_from_vault(&vault_id, &asset_id)?,
			}

			Self::deposit_event(Event::AssetUpdatedInVault { who, vault_id, asset_id, action });

			Ok(())
		}
	}
}
