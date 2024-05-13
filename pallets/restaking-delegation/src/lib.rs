// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! # Operator Delegated Staking
//! Minimal staking pallet that implements multi-asset delegation to operators.
//! The main difference between this pallet and `frame/pallet-staking` is that this pallet
//! uses direct delegation. Delegators choose exactly who they delegate and with what stake.
//! This is different from `frame/pallet-staking` where delegators approval vote and run Phragmen.
//! This pallet also doesn't de-facto implement any selection logic. It mainly provides the
//! primitive to delegate and manage delegations to operators for the Blueprints they operate
//! Instances for.
//!
//! ### Rules
//! There is a new round every `<Round<T>>::get().length` blocks.
//!
//! At the start of every round,
//! * issuance is calculated for operators (and their delegators) for block authoring
//! `T::RewardPaymentDelay` rounds ago
//! * a new set of operators is chosen from the operators
//!
//! Immediately following a round change, payments are made once-per-block until all payments have
//! been made. In each such block, one operator is chosen for a rewards payment and is paid along
//! with each of its top `T::MaxTopDelegationsPerOperator` delegators.
//!
//! To join the set of operators, call `join_operators` with `bond >= MinOperatorStk`.
//! To leave the set of operators, call `schedule_leave_operators`. If the call succeeds,
//! the operator is removed from the pool of operators so they cannot be selected for future
//! operator sets, but they are not unbonded until their exit request is executed. Any signed
//! account may trigger the exit `T::LeaveOperatorsDelay` rounds after the round in which the
//! original request was made.
//!
//! To join the set of delegators, call `delegate` and pass in an account that is
//! already an operator operator and `bond >= MinDelegation`. Each delegator can delegate up to
//! `T::MaxDelegationsPerDelegator` operator operators by calling `delegate`.
//!
//! To revoke a delegation, call `revoke_delegation` with the operator operator's account.
//! To leave the set of delegators and revoke all delegations, call `leave_delegators`. Leaving
//! delegations is only possible if no Blueprint Instance is renting the delegator's stake for
//! economic security.

#![cfg_attr(not(feature = "std"), no_std)]

mod auto_compound;
mod delegation_requests;
pub mod traits;
pub mod types;
pub mod weights;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;
#[cfg(test)]
mod mock;
mod set;
#[cfg(test)]
mod tests;

use frame_support::{pallet, traits::OneSessionHandler};
pub use weights::WeightInfo;

pub use auto_compound::{AutoCompoundConfig, AutoCompoundDelegations};
pub use delegation_requests::{CancelledScheduledRequest, DelegationAction, ScheduledRequest};
pub use pallet::*;
pub use traits::*;
pub use types::*;
pub use RoundIndex;

#[pallet]
pub mod pallet {
	use crate::delegation_requests::{
		CancelledScheduledRequest, DelegationAction, ScheduledRequest,
	};
	use crate::{set::BoundedOrderedSet, traits::*, types::*, WeightInfo};
	use crate::{AutoCompoundConfig, AutoCompoundDelegations};
	use frame_support::fail;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{
		tokens::WithdrawReasons, Currency, Get, Imbalance, LockIdentifier, LockableCurrency,
		ReservableCurrency,
	};
	use frame_system::pallet_prelude::*;
	use sp_core::ecdsa;
	use sp_runtime::RuntimeAppPublic;
	use sp_runtime::{
		traits::{Saturating, Zero},
		DispatchErrorWithPostInfo, Perbill, Percent,
	};
	use sp_std::{collections::btree_map::BTreeMap, prelude::*};

	/// Pallet for restaking delegation
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	pub type RoundIndex = u32;
	type RewardPoint = u32;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub const OPERATOR_LOCK_ID: LockIdentifier = *b"stkngcol";
	pub const DELEGATOR_LOCK_ID: LockIdentifier = *b"stkngdel";

	/// A hard limit for weight computation purposes for the max operators that _could_
	/// theoretically exist.
	pub const MAX_OPERATORS: u32 = 10000;

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The currency type
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;

		/// Authority identifier type
		type RoleKeyId: Member
			+ Parameter
			+ RuntimeAppPublic
			+ MaybeSerializeDeserialize
			+ AsRef<[u8]>
			+ Into<ecdsa::Public>
			+ From<ecdsa::Public>
			+ MaxEncodedLen;

		/// The origin for monetary governance
		type MonetaryGovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Minimum number of blocks per round
		#[pallet::constant]
		type MinBlocksPerRound: Get<u32>;
		/// If an operator doesn't produce any block on this number of rounds, it is notified as inactive.
		/// This value must be less than or equal to RewardPaymentDelay.
		#[pallet::constant]
		type MaxOfflineRounds: Get<u32>;
		/// Number of rounds that operators remain bonded before exit request is executable
		#[pallet::constant]
		type LeaveOperatorsDelay: Get<RoundIndex>;
		/// Number of rounds operator requests to decrease self-bond must wait to be executable
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
		/// Number of rounds after which block authors are rewarded
		#[pallet::constant]
		type RewardPaymentDelay: Get<RoundIndex>;
		/// Minimum number of selected operators every round
		#[pallet::constant]
		type MinSelectedOperators: Get<u32>;
		/// Maximum top delegations counted per operator
		#[pallet::constant]
		type MaxTopDelegationsPerOperator: Get<u32>;
		/// Maximum bottom delegations (not counted) per operator
		#[pallet::constant]
		type MaxBottomDelegationsPerOperator: Get<u32>;
		/// Maximum delegations per delegator
		#[pallet::constant]
		type MaxDelegationsPerDelegator: Get<u32>;
		/// Minimum stake required for any account to be an operator operator
		#[pallet::constant]
		type MinOperatorStk: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to delegate
		#[pallet::constant]
		type MinDelegation: Get<BalanceOf<Self>>;

		/// Handler to notify the runtime when an operator is paid.
		/// If you don't need it, you can specify the type `()`.
		type OnOperatorPayout: OnOperatorPayout<Self::AccountId, BalanceOf<Self>>;
		/// Handler to distribute an operator's reward.
		/// To use the default implementation of minting rewards, specify the type `()`.
		type PayoutOperatorReward: PayoutOperatorReward<Self>;
		/// Handler to notify the runtime when an operator is inactive.
		/// The default behavior is to mark the operator as offline.
		/// If you need to use the default implementation, specify the type `()`.
		type OnInactiveOperator: OnInactiveOperator<Self>;

		/// Get the slot duration in milliseconds
		#[pallet::constant]
		type SlotDuration: Get<u64>;
		/// Get the average time beetween 2 blocks in milliseconds
		#[pallet::constant]
		type BlockTime: Get<u64>;
		/// Maximum operators
		#[pallet::constant]
		type MaxOperators: Get<u32>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		DelegatorDNE,
		DelegatorDNEinTopNorBottom,
		DelegatorDNEInDelegatorSet,
		OperatorDNE,
		DelegationDNE,
		DelegatorExists,
		OperatorExists,
		OperatorBondBelowMin,
		InsufficientBalance,
		DelegatorBondBelowMin,
		DelegationBelowMin,
		AlreadyOffline,
		AlreadyActive,
		DelegatorAlreadyLeaving,
		DelegatorNotLeaving,
		DelegatorCannotLeaveYet,
		CannotDelegateIfLeaving,
		OperatorAlreadyLeaving,
		OperatorNotLeaving,
		OperatorCannotLeaveYet,
		CannotGoOnlineIfLeaving,
		ExceedMaxDelegationsPerDelegator,
		AlreadyDelegatedOperator,
		InvalidSchedule,
		CannotSetBelowMin,
		RoundLengthMustBeGreaterThanTotalSelectedOperators,
		NoWritingSameValue,
		TooLowOperatorCountWeightHintJoinOperators,
		TooLowOperatorCountWeightHintCancelLeaveOperators,
		TooLowOperatorCountToLeaveOperators,
		TooLowDelegationCountToDelegate,
		TooLowOperatorDelegationCountToDelegate,
		TooLowOperatorDelegationCountToLeaveOperators,
		TooLowDelegationCountToLeaveDelegators,
		PendingOperatorRequestsDNE,
		PendingOperatorRequestAlreadyExists,
		PendingOperatorRequestNotDueYet,
		PendingDelegationRequestDNE,
		PendingDelegationRequestAlreadyExists,
		PendingDelegationRequestNotDueYet,
		CannotDelegateLessThanOrEqualToLowestBottomWhenFull,
		PendingDelegationRevoke,
		TooLowDelegationCountToAutoCompound,
		TooLowOperatorAutoCompoundingDelegationCountToAutoCompound,
		TooLowOperatorAutoCompoundingDelegationCountToDelegate,
		TooLowOperatorCountToNotifyAsInactive,
		CannotBeNotifiedAsInactive,
		TooLowOperatorAutoCompoundingDelegationCountToLeaveOperators,
		TooLowOperatorCountWeightHint,
		TooLowOperatorCountWeightHintGoOffline,
		OperatorLimitReached,
		CannotSetAboveMaxOperators,
		RemovedCall,
		MarkingOfflineNotEnabled,
		CurrentRoundTooLow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Started new round.
		NewRound {
			starting_block: BlockNumberFor<T>,
			round: RoundIndex,
			selected_operators_number: u32,
			total_balance: BalanceOf<T>,
		},
		/// Account joined the set of operator operators.
		JoinedOperatorOperators {
			account: T::AccountId,
			amount_locked: BalanceOf<T>,
			new_total_amt_locked: BalanceOf<T>,
		},
		/// Operator selected for operators. Total Exposed Amount includes all delegations.
		OperatorChosen {
			round: RoundIndex,
			operator_account: T::AccountId,
			total_exposed_amount: BalanceOf<T>,
		},
		/// Operator requested to decrease a self bond.
		OperatorBondLessRequested {
			operator: T::AccountId,
			amount_to_decrease: BalanceOf<T>,
			execute_round: RoundIndex,
		},
		/// Operator has increased a self bond.
		OperatorBondedMore {
			operator: T::AccountId,
			amount: BalanceOf<T>,
			new_total_bond: BalanceOf<T>,
		},
		/// Operator has decreased a self bond.
		OperatorBondedLess { operator: T::AccountId, amount: BalanceOf<T>, new_bond: BalanceOf<T> },
		/// Operator temporarily leave the set of operator operators without unbonding.
		OperatorWentOffline { operator: T::AccountId },
		/// Operator rejoins the set of operator operators.
		OperatorBackOnline { operator: T::AccountId },
		/// Operator has requested to leave the set of operators.
		OperatorScheduledExit {
			exit_allowed_round: RoundIndex,
			operator: T::AccountId,
			scheduled_exit: RoundIndex,
		},
		/// Cancelled request to leave the set of operators.
		CancelledOperatorExit { operator: T::AccountId },
		/// Cancelled request to decrease operator's bond.
		CancelledOperatorBondLess {
			operator: T::AccountId,
			amount: BalanceOf<T>,
			execute_round: RoundIndex,
		},
		/// Operator has left the set of operators.
		OperatorLeft {
			ex_operator: T::AccountId,
			unlocked_amount: BalanceOf<T>,
			new_total_amt_locked: BalanceOf<T>,
		},
		/// Delegator requested to decrease a bond for the operator operator.
		DelegationDecreaseScheduled {
			delegator: T::AccountId,
			operator: T::AccountId,
			amount_to_decrease: BalanceOf<T>,
			execute_round: RoundIndex,
		},
		// Delegation increased.
		DelegationIncreased {
			delegator: T::AccountId,
			operator: T::AccountId,
			amount: BalanceOf<T>,
			in_top: bool,
		},
		// Delegation decreased.
		DelegationDecreased {
			delegator: T::AccountId,
			operator: T::AccountId,
			amount: BalanceOf<T>,
			in_top: bool,
		},
		/// Delegator requested to leave the set of delegators.
		DelegatorExitScheduled {
			round: RoundIndex,
			delegator: T::AccountId,
			scheduled_exit: RoundIndex,
		},
		/// Delegator requested to revoke delegation.
		DelegationRevocationScheduled {
			round: RoundIndex,
			delegator: T::AccountId,
			operator: T::AccountId,
			scheduled_exit: RoundIndex,
		},
		/// Delegator has left the set of delegators.
		DelegatorLeft { delegator: T::AccountId, unstaked_amount: BalanceOf<T> },
		/// Delegation revoked.
		DelegationRevoked {
			delegator: T::AccountId,
			operator: T::AccountId,
			unstaked_amount: BalanceOf<T>,
		},
		/// Delegation kicked.
		DelegationKicked {
			delegator: T::AccountId,
			operator: T::AccountId,
			unstaked_amount: BalanceOf<T>,
		},
		/// Cancelled a pending request to exit the set of delegators.
		DelegatorExitCancelled { delegator: T::AccountId },
		/// Cancelled request to change an existing delegation.
		CancelledDelegationRequest {
			delegator: T::AccountId,
			cancelled_request: CancelledScheduledRequest<BalanceOf<T>>,
			operator: T::AccountId,
		},
		/// New delegation (increase of the existing one).
		Delegation {
			delegator: T::AccountId,
			locked_amount: BalanceOf<T>,
			operator: T::AccountId,
			delegator_position: DelegatorAdded<BalanceOf<T>>,
			auto_compound: Percent,
		},
		/// Delegation from operator state has been remove.
		DelegatorLeftOperator {
			delegator: T::AccountId,
			operator: T::AccountId,
			unstaked_amount: BalanceOf<T>,
			total_operator_staked: BalanceOf<T>,
		},
		/// Paid the account (delegator or operator) the balance as liquid rewards.
		Rewarded { account: T::AccountId, rewards: BalanceOf<T> },
		/// Set total selected operators to this value.
		TotalSelectedSet { old: u32, new: u32 },
		/// Set operator commission to this value.
		OperatorCommissionSet { old: Perbill, new: Perbill },
		/// Auto-compounding reward percent was set for a delegation.
		AutoCompoundSet { operator: T::AccountId, delegator: T::AccountId, value: Percent },
		/// Compounded a portion of rewards towards the delegation.
		Compounded { operator: T::AccountId, delegator: T::AccountId, amount: BalanceOf<T> },
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			let mut weight = <T as Config>::WeightInfo::base_on_initialize();

			let mut round = <Round<T>>::get();
			if round.should_update(n) {
				// fetch current slot number
				// TODO: FIX
				let current_slot: u64 = 0u64;

				// // account for SlotProvider read
				// weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));

				// Compute round duration in slots
				let round_duration = (current_slot.saturating_sub(round.first_slot))
					.saturating_mul(T::SlotDuration::get());

				// mutate round
				round.update(n, current_slot);
				// // notify that new round begin
				// weight = weight.saturating_add(T::OnNewRound::on_new_round(round.current));
				// pay all stakers for T::RewardPaymentDelay rounds ago
				weight =
					weight.saturating_add(Self::prepare_staking_payouts(round, round_duration));

				// select top operator operators for next round
				let (extra_weight, operator_count, _delegation_count, total_staked) =
					Self::select_top_operators(round.current);
				weight = weight.saturating_add(extra_weight);
				// start next round
				<Round<T>>::put(round);
				Self::deposit_event(Event::NewRound {
					starting_block: round.first,
					round: round.current,
					selected_operators_number: operator_count,
					total_balance: total_staked,
				});
				// account for Round write
				weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));
			} else {
				weight = weight.saturating_add(Self::handle_delayed_payouts(round.current));
			}

			// add on_finalize weight
			//   read:  Author, Points, AwardedPts
			//   write: Points, AwardedPts
			weight = weight.saturating_add(T::DbWeight::get().reads_writes(3, 2));
			weight
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn operator_commission)]
	/// Commission percent taken off of rewards for all operators
	type OperatorCommission<T: Config> = StorageValue<_, Perbill, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_selected)]
	/// The total operators selected every round
	pub(crate) type TotalSelected<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn round)]
	/// Current round index and next round scheduled transition
	pub type Round<T: Config> = StorageValue<_, RoundInfo<BlockNumberFor<T>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn delegator_state)]
	/// Get delegator state associated with an account if account is delegating else None
	pub(crate) type DelegatorState<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Delegator<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn operator_info)]
	/// Get operator operator info associated with an account if account is operator else None
	pub(crate) type OperatorInfo<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, OperatorMetadata<BalanceOf<T>>, OptionQuery>;

	pub struct AddGet<T, R> {
		_phantom: PhantomData<(T, R)>,
	}
	impl<T, R> Get<u32> for AddGet<T, R>
	where
		T: Get<u32>,
		R: Get<u32>,
	{
		fn get() -> u32 {
			T::get() + R::get()
		}
	}

	/// Stores outstanding delegation requests per operator.
	#[pallet::storage]
	#[pallet::getter(fn delegation_scheduled_requests)]
	pub(crate) type DelegationScheduledRequests<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<
			ScheduledRequest<T::AccountId, BalanceOf<T>>,
			AddGet<T::MaxTopDelegationsPerOperator, T::MaxBottomDelegationsPerOperator>,
		>,
		ValueQuery,
	>;

	/// Stores auto-compounding configuration per operator.
	#[pallet::storage]
	#[pallet::getter(fn auto_compounding_delegations)]
	pub(crate) type AutoCompoundingDelegations<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<
			AutoCompoundConfig<T::AccountId>,
			AddGet<T::MaxTopDelegationsPerOperator, T::MaxBottomDelegationsPerOperator>,
		>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn top_delegations)]
	/// Top delegations for operator operator
	pub(crate) type TopDelegations<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Delegations<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn bottom_delegations)]
	/// Bottom delegations for operator operator
	pub(crate) type BottomDelegations<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Delegations<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn selected_operator)]
	/// The operator operators selected for the current round
	type SelectedOperators<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxOperators>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total)]
	/// Total capital locked by this staking pallet
	pub(crate) type Total<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn operator_pool)]
	/// The pool of operator operators, each with their total backing stake
	pub(crate) type OperatorPool<T: Config> = StorageValue<
		_,
		BoundedOrderedSet<Bond<T::AccountId, BalanceOf<T>>, T::MaxOperators>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn at_stake)]
	/// Snapshot of operator delegation stake at the start of the round
	pub type AtStake<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		RoundIndex,
		Twox64Concat,
		T::AccountId,
		OperatorSnapshot<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn delayed_payouts)]
	/// Delayed payouts
	pub type DelayedPayouts<T: Config> =
		StorageMap<_, Twox64Concat, RoundIndex, DelayedPayout<BalanceOf<T>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn points)]
	/// Total points awarded to operators for block production in the round
	pub type Points<T: Config> = StorageMap<_, Twox64Concat, RoundIndex, RewardPoint, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn awarded_pts)]
	/// Points for each operator per round
	pub type AwardedPts<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		RoundIndex,
		Twox64Concat,
		T::AccountId,
		RewardPoint,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn marking_offline)]
	/// Killswitch to enable/disable marking offline feature.
	pub type EnableMarkingOffline<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Initialize balance and register all as operators: `(operator AccountId, balance Amount)`
		pub operators: Vec<(T::AccountId, BalanceOf<T>)>,
		/// Initialize balance and make delegations:
		/// `(delegator AccountId, operator AccountId, delegation Amount, auto-compounding Percent)`
		pub delegations: Vec<(T::AccountId, T::AccountId, BalanceOf<T>, Percent)>,
		/// Default fixed percent an operator takes off the top of due rewards
		pub operator_commission: Perbill,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { delegations: vec![], operators: vec![], operator_commission: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			let mut operator_count = 0u32;
			// Initialize the operators
			for &(ref operator, balance) in &self.operators {
				assert!(
					<Pallet<T>>::get_operator_stakable_free_balance(operator) >= balance,
					"Account does not have enough balance to bond as a operator."
				);
				if let Err(error) = <Pallet<T>>::join_operators(
					T::RuntimeOrigin::from(Some(operator.clone()).into()),
					balance,
					operator_count,
				) {
					log::warn!("Join operators failed in genesis with error {:?}", error);
				} else {
					operator_count = operator_count.saturating_add(1u32);
				}
			}

			let mut col_delegator_count: BTreeMap<T::AccountId, u32> = BTreeMap::new();
			let mut col_auto_compound_delegator_count: BTreeMap<T::AccountId, u32> =
				BTreeMap::new();
			let mut del_delegation_count: BTreeMap<T::AccountId, u32> = BTreeMap::new();
			// Initialize the delegations
			for &(ref delegator, ref target, balance, auto_compound) in &self.delegations {
				assert!(
					<Pallet<T>>::get_delegator_stakable_free_balance(delegator) >= balance,
					"Account does not have enough balance to place delegation."
				);
				let cd_count =
					if let Some(x) = col_delegator_count.get(target) { *x } else { 0u32 };
				let dd_count =
					if let Some(x) = del_delegation_count.get(delegator) { *x } else { 0u32 };
				let cd_auto_compound_count =
					col_auto_compound_delegator_count.get(target).cloned().unwrap_or_default();
				if let Err(error) = <Pallet<T>>::delegate_with_auto_compound(
					T::RuntimeOrigin::from(Some(delegator.clone()).into()),
					target.clone(),
					balance,
					auto_compound,
					cd_count,
					cd_auto_compound_count,
					dd_count,
				) {
					log::warn!("Delegate failed in genesis with error {:?}", error);
				} else {
					if let Some(x) = col_delegator_count.get_mut(target) {
						*x = x.saturating_add(1u32);
					} else {
						col_delegator_count.insert(target.clone(), 1u32);
					};
					if let Some(x) = del_delegation_count.get_mut(delegator) {
						*x = x.saturating_add(1u32);
					} else {
						del_delegation_count.insert(delegator.clone(), 1u32);
					};
					if !auto_compound.is_zero() {
						col_auto_compound_delegator_count
							.entry(target.clone())
							.and_modify(|x| *x = x.saturating_add(1))
							.or_insert(1);
					}
				}
			}
			// Set operator commission to default config
			<OperatorCommission<T>>::put(self.operator_commission);

			// Choose top TotalSelected operator operators
			let (_, v_count, _, total_staked) = <Pallet<T>>::select_top_operators(1u32);
			// // Start Round 1 at Block 0
			// let round: RoundInfo<BlockNumberFor<T>> =
			// 	RoundInfo::new(1u32, Zero::zero(), self.blocks_per_round, 0);
			// <Round<T>>::put(round);
			<Pallet<T>>::deposit_event(Event::NewRound {
				starting_block: Zero::zero(),
				round: 1u32,
				selected_operators_number: v_count,
				total_balance: total_staked,
			});
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the commission for all operators
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::set_operator_commission())]
		pub fn set_operator_commission(
			origin: OriginFor<T>,
			new: Perbill,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			let old = <OperatorCommission<T>>::get();
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			<OperatorCommission<T>>::put(new);
			Self::deposit_event(Event::OperatorCommissionSet { old, new });
			Ok(().into())
		}

		/// Join the set of operator operators
		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::join_operators(*operator_count))]
		pub fn join_operators(
			origin: OriginFor<T>,
			bond: BalanceOf<T>,
			operator_count: u32,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			ensure!(bond >= T::MinOperatorStk::get(), Error::<T>::OperatorBondBelowMin);
			Self::join_operators_inner(acc, bond, operator_count)
		}

		/// Request to leave the set of operators. If successful, the account is immediately
		/// removed from the operator pool to prevent selection as an operator.
		#[pallet::call_index(8)]
		#[pallet::weight(<T as Config>::WeightInfo::schedule_leave_operators(*operator_count))]
		pub fn schedule_leave_operators(
			origin: OriginFor<T>,
			operator_count: u32,
		) -> DispatchResultWithPostInfo {
			let operator = ensure_signed(origin)?;
			let mut state = <OperatorInfo<T>>::get(&operator).ok_or(Error::<T>::OperatorDNE)?;
			let (now, when) = state.schedule_leave::<T>()?;
			let mut operators = <OperatorPool<T>>::get();
			ensure!(
				operator_count >= operators.0.len() as u32,
				Error::<T>::TooLowOperatorCountToLeaveOperators
			);
			if operators.remove(&Bond::from_owner(operator.clone())) {
				<OperatorPool<T>>::put(operators);
			}
			<OperatorInfo<T>>::insert(&operator, state);
			Self::deposit_event(Event::OperatorScheduledExit {
				exit_allowed_round: now,
				operator,
				scheduled_exit: when,
			});
			Ok(().into())
		}

		/// Execute leave operators request
		#[pallet::call_index(9)]
		#[pallet::weight(
			<T as Config>::WeightInfo::execute_leave_operators_worst_case(*operator_delegation_count)
		)]
		pub fn execute_leave_operators(
			origin: OriginFor<T>,
			operator: T::AccountId,
			operator_delegation_count: u32,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			let state = <OperatorInfo<T>>::get(&operator).ok_or(Error::<T>::OperatorDNE)?;
			ensure!(
				state.delegation_count <= operator_delegation_count,
				Error::<T>::TooLowOperatorDelegationCountToLeaveOperators
			);
			<Pallet<T>>::execute_leave_operators_inner(operator)
		}

		/// Cancel open request to leave operators
		/// - only callable by operator account
		/// - result upon successful call is the operator is active in the operator pool
		#[pallet::call_index(10)]
		#[pallet::weight(<T as Config>::WeightInfo::cancel_leave_operators(*operator_count))]
		pub fn cancel_leave_operators(
			origin: OriginFor<T>,
			operator_count: u32,
		) -> DispatchResultWithPostInfo {
			let operator = ensure_signed(origin)?;
			let mut state = <OperatorInfo<T>>::get(&operator).ok_or(Error::<T>::OperatorDNE)?;
			ensure!(state.is_leaving(), Error::<T>::OperatorNotLeaving);
			state.go_online();
			let mut operators = <OperatorPool<T>>::get();
			ensure!(
				operators.0.len() as u32 <= operator_count,
				Error::<T>::TooLowOperatorCountWeightHintCancelLeaveOperators
			);
			let maybe_inserted_operator = operators
				.try_insert(Bond { owner: operator.clone(), amount: state.total_counted })
				.map_err(|_| Error::<T>::OperatorLimitReached)?;
			ensure!(maybe_inserted_operator, Error::<T>::AlreadyActive);
			<OperatorPool<T>>::put(operators);
			<OperatorInfo<T>>::insert(&operator, state);
			Self::deposit_event(Event::CancelledOperatorExit { operator });
			Ok(().into())
		}

		/// Temporarily leave the set of operator operators without unbonding
		#[pallet::call_index(11)]
		#[pallet::weight(<T as Config>::WeightInfo::go_offline(MAX_OPERATORS))]
		pub fn go_offline(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let operator = ensure_signed(origin)?;
			<Pallet<T>>::go_offline_inner(operator)
		}

		/// Rejoin the set of operator operators if previously had called `go_offline`
		#[pallet::call_index(12)]
		#[pallet::weight(<T as Config>::WeightInfo::go_online(MAX_OPERATORS))]
		pub fn go_online(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let operator = ensure_signed(origin)?;
			<Pallet<T>>::go_online_inner(operator)
		}

		/// Increase operator operator self bond by `more`
		#[pallet::call_index(13)]
		#[pallet::weight(<T as Config>::WeightInfo::operator_bond_more(MAX_OPERATORS))]
		pub fn operator_bond_more(
			origin: OriginFor<T>,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let operator = ensure_signed(origin)?;
			<Pallet<T>>::operator_bond_more_inner(operator, more)
		}

		/// Request by operator operator to decrease self bond by `less`
		#[pallet::call_index(14)]
		#[pallet::weight(<T as Config>::WeightInfo::schedule_operator_bond_less())]
		pub fn schedule_operator_bond_less(
			origin: OriginFor<T>,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let operator = ensure_signed(origin)?;
			let mut state = <OperatorInfo<T>>::get(&operator).ok_or(Error::<T>::OperatorDNE)?;
			let when = state.schedule_bond_less::<T>(less)?;
			<OperatorInfo<T>>::insert(&operator, state);
			Self::deposit_event(Event::OperatorBondLessRequested {
				operator,
				amount_to_decrease: less,
				execute_round: when,
			});
			Ok(().into())
		}

		/// Execute pending request to adjust the operator operator self bond
		#[pallet::call_index(15)]
		#[pallet::weight(<T as Config>::WeightInfo::execute_operator_bond_less(MAX_OPERATORS))]
		pub fn execute_operator_bond_less(
			origin: OriginFor<T>,
			operator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?; // we may want to reward this if caller != operator
			<Pallet<T>>::execute_operator_bond_less_inner(operator)
		}

		/// Cancel pending request to adjust the operator operator self bond
		#[pallet::call_index(16)]
		#[pallet::weight(<T as Config>::WeightInfo::cancel_operator_bond_less())]
		pub fn cancel_operator_bond_less(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let operator = ensure_signed(origin)?;
			let mut state = <OperatorInfo<T>>::get(&operator).ok_or(Error::<T>::OperatorDNE)?;
			state.cancel_bond_less::<T>(operator.clone())?;
			<OperatorInfo<T>>::insert(&operator, state);
			Ok(().into())
		}

		/// DEPRECATED use delegateWithAutoCompound
		/// If caller is not a delegator and not a collator, then join the set of delegators
		/// If caller is a delegator, then makes delegation to change their delegation state
		#[pallet::call_index(17)]
		#[pallet::weight(
			<T as Config>::WeightInfo::delegate_with_auto_compound_worst()
		)]
		pub fn delegate(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			amount: BalanceOf<T>,
			candidate_delegation_count: u32,
			delegation_count: u32,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			<AutoCompoundDelegations<T>>::delegate_with_auto_compound(
				candidate,
				delegator,
				amount,
				Percent::zero(),
				candidate_delegation_count,
				0,
				delegation_count,
			)
		}

		/// If caller is not a delegator and not an operator, then join the set of delegators
		/// If caller is a delegator, then makes delegation to change their delegation state
		/// Sets the auto-compound config for the delegation
		#[pallet::call_index(18)]
		#[pallet::weight(
			<T as Config>::WeightInfo::delegate_with_auto_compound(
				*operator_delegation_count,
				*operator_auto_compounding_delegation_count,
				*delegation_count,
			)
		)]
		pub fn delegate_with_auto_compound(
			origin: OriginFor<T>,
			operator: T::AccountId,
			amount: BalanceOf<T>,
			// asset_id: AssetId,
			auto_compound: Percent,
			operator_delegation_count: u32,
			operator_auto_compounding_delegation_count: u32,
			delegation_count: u32,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			<AutoCompoundDelegations<T>>::delegate_with_auto_compound(
				operator,
				delegator,
				amount,
				// asset_id,
				auto_compound,
				operator_delegation_count,
				operator_auto_compounding_delegation_count,
				delegation_count,
			)
		}

		/// Request to revoke an existing delegation. If successful, the delegation is scheduled
		/// to be allowed to be revoked via the `execute_delegation_request` extrinsic.
		/// The delegation receives no rewards for the rounds while a revoke is pending.
		/// A revoke may not be performed if any other scheduled request is pending.
		#[pallet::call_index(22)]
		#[pallet::weight(<T as Config>::WeightInfo::schedule_revoke_delegation(
			T::MaxTopDelegationsPerOperator::get() + T::MaxBottomDelegationsPerOperator::get()
		))]
		pub fn schedule_revoke_delegation(
			origin: OriginFor<T>,
			operator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			Self::delegation_schedule_revoke(operator, delegator)
		}

		/// Bond more for delegators wrt a specific operator operator.
		#[pallet::call_index(23)]
		#[pallet::weight(<T as Config>::WeightInfo::delegator_bond_more(
			T::MaxTopDelegationsPerOperator::get() + T::MaxBottomDelegationsPerOperator::get()
		))]
		pub fn delegator_bond_more(
			origin: OriginFor<T>,
			operator: T::AccountId,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			let (in_top, weight) = Self::delegation_bond_more_without_event(
				delegator.clone(),
				operator.clone(),
				more.clone(),
			)?;
			Pallet::<T>::deposit_event(Event::DelegationIncreased {
				delegator,
				operator,
				amount: more,
				in_top,
			});

			Ok(Some(weight).into())
		}

		/// Request bond less for delegators wrt a specific operator operator. The delegation's
		/// rewards for rounds while the request is pending use the reduced bonded amount.
		/// A bond less may not be performed if any other scheduled request is pending.
		#[pallet::call_index(24)]
		#[pallet::weight(<T as Config>::WeightInfo::schedule_delegator_bond_less(
			T::MaxTopDelegationsPerOperator::get() + T::MaxBottomDelegationsPerOperator::get()
		))]
		pub fn schedule_delegator_bond_less(
			origin: OriginFor<T>,
			operator: T::AccountId,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			Self::delegation_schedule_bond_decrease(operator, delegator, less)
		}

		/// Execute pending request to change an existing delegation
		#[pallet::call_index(25)]
		#[pallet::weight(<T as Config>::WeightInfo::execute_delegator_revoke_delegation_worst())]
		pub fn execute_delegation_request(
			origin: OriginFor<T>,
			delegator: T::AccountId,
			operator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?; // we may want to reward caller if caller != delegator
			Self::delegation_execute_scheduled_request(operator, delegator)
		}

		/// Cancel request to change an existing delegation.
		#[pallet::call_index(26)]
		#[pallet::weight(<T as Config>::WeightInfo::cancel_delegation_request(350))]
		pub fn cancel_delegation_request(
			origin: OriginFor<T>,
			operator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			Self::delegation_cancel_request(operator, delegator)
		}

		/// Sets the auto-compounding reward percentage for a delegation.
		#[pallet::call_index(27)]
		#[pallet::weight(<T as Config>::WeightInfo::set_auto_compound(
			*operator_auto_compounding_delegation_count_hint,
			*delegation_count_hint,
		))]
		pub fn set_auto_compound(
			origin: OriginFor<T>,
			operator: T::AccountId,
			value: Percent,
			operator_auto_compounding_delegation_count_hint: u32,
			delegation_count_hint: u32,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			<AutoCompoundDelegations<T>>::set_auto_compound(
				operator,
				delegator,
				value,
				operator_auto_compounding_delegation_count_hint,
				delegation_count_hint,
			)
		}

		/// Hotfix to remove existing empty entries for operators that have left.
		#[pallet::call_index(28)]
		#[pallet::weight(
			T::DbWeight::get().reads_writes(2 * operators.len() as u64, operators.len() as u64)
		)]
		pub fn hotfix_remove_delegation_requests_exited_operator(
			origin: OriginFor<T>,
			operators: Vec<T::AccountId>,
		) -> DispatchResult {
			ensure_signed(origin)?;
			ensure!(operators.len() < 100, <Error<T>>::InsufficientBalance);
			for operator in &operators {
				ensure!(
					<OperatorInfo<T>>::get(&operator).is_none(),
					<Error<T>>::OperatorNotLeaving
				);
				ensure!(
					<DelegationScheduledRequests<T>>::get(&operator).is_empty(),
					<Error<T>>::OperatorNotLeaving
				);
			}

			for operator in operators {
				<DelegationScheduledRequests<T>>::remove(operator);
			}

			Ok(().into())
		}

		/// Notify an operator is inactive during MaxOfflineRounds
		#[pallet::call_index(29)]
		#[pallet::weight(<T as Config>::WeightInfo::notify_inactive_operator())]
		pub fn notify_inactive_operator(
			origin: OriginFor<T>,
			operator: T::AccountId,
		) -> DispatchResult {
			ensure!(<EnableMarkingOffline<T>>::get(), <Error<T>>::MarkingOfflineNotEnabled);
			ensure_signed(origin)?;

			let mut operators_len = 0usize;
			let max_candidates = <TotalSelected<T>>::get();

			if let Some(len) = <SelectedOperators<T>>::decode_len() {
				operators_len = len;
			};

			// Check operators length is not below or eq to 66% of max_candidates.
			// We use saturating logic here with (2/3)
			// as it is dangerous to use floating point numbers directly.
			ensure!(
				operators_len * 3 > (max_candidates * 2) as usize,
				<Error<T>>::TooLowOperatorCountToNotifyAsInactive
			);

			let round_info = <Round<T>>::get();
			let max_offline_rounds = T::MaxOfflineRounds::get();

			ensure!(round_info.current > max_offline_rounds, <Error<T>>::CurrentRoundTooLow);

			// Have rounds_to_check = [8,9]
			// in case we are in round 10 for instance
			// with MaxOfflineRounds = 2
			let first_round_to_check = round_info.current.saturating_sub(max_offline_rounds);
			let rounds_to_check = first_round_to_check..round_info.current;

			// If this counter is eq to max_offline_rounds,
			// the operator should be notified as inactive
			let mut inactive_counter: RoundIndex = 0u32;

			// Iter rounds to check
			//
			// - The operator has AtStake associated and their AwardedPts are zero
			//
			// If the previous condition is met in all rounds of rounds_to_check,
			// the operator is notified as inactive
			for r in rounds_to_check {
				let stake = <AtStake<T>>::get(r, &operator);
				let pts = <AwardedPts<T>>::get(r, &operator);

				if stake.is_some() && pts.is_zero() {
					inactive_counter = inactive_counter.saturating_add(1);
				}
			}

			if inactive_counter == max_offline_rounds {
				let _ = T::OnInactiveOperator::on_inactive_operator(
					operator.clone(),
					round_info.current.saturating_sub(1),
				);
			} else {
				return Err(<Error<T>>::CannotBeNotifiedAsInactive.into());
			}

			Ok(().into())
		}

		/// Enable/Disable marking offline feature
		#[pallet::call_index(30)]
		#[pallet::weight(
			Weight::from_parts(3_000_000u64, 4_000u64)
				.saturating_add(T::DbWeight::get().writes(1u64))
		)]
		pub fn enable_marking_offline(origin: OriginFor<T>, value: bool) -> DispatchResult {
			ensure_root(origin)?;
			<EnableMarkingOffline<T>>::set(value);
			Ok(())
		}

		/// Force join the set of operator operators.
		/// It will skip the minimum required bond check.
		#[pallet::call_index(31)]
		#[pallet::weight(<T as Config>::WeightInfo::join_operators(*operator_count))]
		pub fn force_join_operators(
			origin: OriginFor<T>,
			account: T::AccountId,
			bond: BalanceOf<T>,
			operator_count: u32,
		) -> DispatchResultWithPostInfo {
			T::MonetaryGovernanceOrigin::ensure_origin(origin.clone())?;
			Self::join_operators_inner(account, bond, operator_count)
		}
	}

	/// Represents a payout made via `pay_one_operator_reward`.
	pub(crate) enum RewardPayment {
		/// An operator was paid
		Paid,
		/// An operator was skipped for payment. This can happen if they haven't been awarded any
		/// points, that is, they did not produce any blocks.
		Skipped,
		/// All operator payments have been processed.
		Finished,
	}

	impl<T: Config> Pallet<T> {
		pub fn set_operator_bond_to_zero(acc: &T::AccountId) -> Weight {
			let actual_weight =
				<T as Config>::WeightInfo::set_operator_bond_to_zero(T::MaxOperators::get());
			if let Some(mut state) = <OperatorInfo<T>>::get(&acc) {
				state.bond_less::<T>(acc.clone(), state.bond);
				<OperatorInfo<T>>::insert(&acc, state);
			}
			actual_weight
		}

		pub fn is_delegator(acc: &T::AccountId) -> bool {
			<DelegatorState<T>>::get(acc).is_some()
		}

		pub fn is_operator(acc: &T::AccountId) -> bool {
			<OperatorInfo<T>>::get(acc).is_some()
		}

		pub fn is_selected_operator(acc: &T::AccountId) -> bool {
			<SelectedOperators<T>>::get().binary_search(acc).is_ok()
		}

		pub fn join_operators_inner(
			acc: T::AccountId,
			bond: BalanceOf<T>,
			operator_count: u32,
		) -> DispatchResultWithPostInfo {
			ensure!(!Self::is_operator(&acc), Error::<T>::OperatorExists);
			ensure!(!Self::is_delegator(&acc), Error::<T>::DelegatorExists);
			let mut operators = <OperatorPool<T>>::get();
			let old_count = operators.0.len() as u32;
			ensure!(
				operator_count >= old_count,
				Error::<T>::TooLowOperatorCountWeightHintJoinOperators
			);
			let maybe_inserted_operator = operators
				.try_insert(Bond { owner: acc.clone(), amount: bond })
				.map_err(|_| Error::<T>::OperatorLimitReached)?;
			ensure!(maybe_inserted_operator, Error::<T>::OperatorExists);

			ensure!(
				Self::get_operator_stakable_free_balance(&acc) >= bond,
				Error::<T>::InsufficientBalance,
			);
			T::Currency::set_lock(OPERATOR_LOCK_ID, &acc, bond, WithdrawReasons::all());
			let operator = OperatorMetadata::new(bond);
			<OperatorInfo<T>>::insert(&acc, operator);
			let empty_delegations: Delegations<T::AccountId, BalanceOf<T>> = Default::default();
			// insert empty top delegations
			<TopDelegations<T>>::insert(&acc, empty_delegations.clone());
			// insert empty bottom delegations
			<BottomDelegations<T>>::insert(&acc, empty_delegations);
			<OperatorPool<T>>::put(operators);
			let new_total = <Total<T>>::get().saturating_add(bond);
			<Total<T>>::put(new_total);
			Self::deposit_event(Event::JoinedOperatorOperators {
				account: acc,
				amount_locked: bond,
				new_total_amt_locked: new_total,
			});
			Ok(().into())
		}

		pub fn go_offline_inner(operator: T::AccountId) -> DispatchResultWithPostInfo {
			let mut state = <OperatorInfo<T>>::get(&operator).ok_or(Error::<T>::OperatorDNE)?;
			let mut operators = <OperatorPool<T>>::get();
			let actual_weight = <T as Config>::WeightInfo::go_offline(operators.0.len() as u32);

			ensure!(
				state.is_active(),
				DispatchErrorWithPostInfo {
					post_info: Some(actual_weight).into(),
					error: <Error<T>>::AlreadyOffline.into(),
				}
			);
			state.go_offline();

			if operators.remove(&Bond::from_owner(operator.clone())) {
				<OperatorPool<T>>::put(operators);
			}
			<OperatorInfo<T>>::insert(&operator, state);
			Self::deposit_event(Event::OperatorWentOffline { operator });
			Ok(Some(actual_weight).into())
		}

		pub fn go_online_inner(operator: T::AccountId) -> DispatchResultWithPostInfo {
			let mut state = <OperatorInfo<T>>::get(&operator).ok_or(Error::<T>::OperatorDNE)?;
			let mut operators = <OperatorPool<T>>::get();
			let actual_weight = <T as Config>::WeightInfo::go_online(operators.0.len() as u32);

			ensure!(
				!state.is_active(),
				DispatchErrorWithPostInfo {
					post_info: Some(actual_weight).into(),
					error: <Error<T>>::AlreadyActive.into(),
				}
			);
			ensure!(
				!state.is_leaving(),
				DispatchErrorWithPostInfo {
					post_info: Some(actual_weight).into(),
					error: <Error<T>>::CannotGoOnlineIfLeaving.into(),
				}
			);
			state.go_online();

			let maybe_inserted_operator = operators
				.try_insert(Bond { owner: operator.clone(), amount: state.total_counted })
				.map_err(|_| Error::<T>::OperatorLimitReached)?;
			ensure!(
				maybe_inserted_operator,
				DispatchErrorWithPostInfo {
					post_info: Some(actual_weight).into(),
					error: <Error<T>>::AlreadyActive.into(),
				},
			);

			<OperatorPool<T>>::put(operators);
			<OperatorInfo<T>>::insert(&operator, state);
			Self::deposit_event(Event::OperatorBackOnline { operator });
			Ok(Some(actual_weight).into())
		}

		pub fn operator_bond_more_inner(
			operator: T::AccountId,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let mut state = <OperatorInfo<T>>::get(&operator).ok_or(Error::<T>::OperatorDNE)?;
			let actual_weight =
				<T as Config>::WeightInfo::operator_bond_more(T::MaxOperators::get());

			state.bond_more::<T>(operator.clone(), more).map_err(|err| {
				DispatchErrorWithPostInfo { post_info: Some(actual_weight).into(), error: err }
			})?;
			let (is_active, total_counted) = (state.is_active(), state.total_counted);
			<OperatorInfo<T>>::insert(&operator, state);
			if is_active {
				Self::update_active(operator, total_counted);
			}
			Ok(Some(actual_weight).into())
		}

		pub fn execute_operator_bond_less_inner(
			operator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let mut state = <OperatorInfo<T>>::get(&operator).ok_or(Error::<T>::OperatorDNE)?;
			let actual_weight =
				<T as Config>::WeightInfo::execute_operator_bond_less(T::MaxOperators::get());

			state.execute_bond_less::<T>(operator.clone()).map_err(|err| {
				DispatchErrorWithPostInfo { post_info: Some(actual_weight).into(), error: err }
			})?;
			<OperatorInfo<T>>::insert(&operator, state);
			Ok(Some(actual_weight).into())
		}

		pub fn execute_leave_operators_inner(operator: T::AccountId) -> DispatchResultWithPostInfo {
			let state = <OperatorInfo<T>>::get(&operator).ok_or(Error::<T>::OperatorDNE)?;
			let actual_auto_compound_delegation_count =
				<AutoCompoundingDelegations<T>>::decode_len(&operator).unwrap_or_default() as u32;

			// TODO use these to return actual weight used via `execute_leave_operators`
			let actual_delegation_count = state.delegation_count;
			let actual_weight = <T as Config>::WeightInfo::execute_leave_operators_ideal(
				actual_delegation_count,
				actual_auto_compound_delegation_count,
			);

			state.can_leave::<T>().map_err(|err| DispatchErrorWithPostInfo {
				post_info: Some(actual_weight).into(),
				error: err,
			})?;
			let return_stake = |bond: Bond<T::AccountId, BalanceOf<T>>| {
				// remove delegation from delegator state
				let mut delegator = DelegatorState::<T>::get(&bond.owner).expect(
					"Operator state and delegator state are consistent. 
						Operator state has a record of this delegation. Therefore, 
						Delegator state also has a record. qed.",
				);

				if let Some(remaining) = delegator.rm_delegation::<T>(&operator) {
					Self::delegation_remove_request_with_state(
						&operator,
						&bond.owner,
						&mut delegator,
					);
					<AutoCompoundDelegations<T>>::remove_auto_compound(&operator, &bond.owner);

					if remaining.is_zero() {
						// we do not remove the scheduled delegation requests from other operators
						// since it is assumed that they were removed incrementally before only the
						// last delegation was left.
						<DelegatorState<T>>::remove(&bond.owner);
						T::Currency::remove_lock(DELEGATOR_LOCK_ID, &bond.owner);
					} else {
						<DelegatorState<T>>::insert(&bond.owner, delegator);
					}
				} else {
					// TODO: review. we assume here that this delegator has no remaining staked
					// balance, so we ensure the lock is cleared
					T::Currency::remove_lock(DELEGATOR_LOCK_ID, &bond.owner);
				}
			};
			// total backing stake is at least the operator self bond
			let mut total_backing = state.bond;
			// return all top delegations
			let top_delegations =
				<TopDelegations<T>>::take(&operator).expect("OperatorInfo existence checked");
			for bond in top_delegations.delegations {
				return_stake(bond);
			}
			total_backing = total_backing.saturating_add(top_delegations.total);
			// return all bottom delegations
			let bottom_delegations =
				<BottomDelegations<T>>::take(&operator).expect("OperatorInfo existence checked");
			for bond in bottom_delegations.delegations {
				return_stake(bond);
			}
			total_backing = total_backing.saturating_add(bottom_delegations.total);
			// return stake to operator
			T::Currency::remove_lock(OPERATOR_LOCK_ID, &operator);
			<OperatorInfo<T>>::remove(&operator);
			<DelegationScheduledRequests<T>>::remove(&operator);
			<AutoCompoundingDelegations<T>>::remove(&operator);
			<TopDelegations<T>>::remove(&operator);
			<BottomDelegations<T>>::remove(&operator);
			let new_total_staked = <Total<T>>::get().saturating_sub(total_backing);
			<Total<T>>::put(new_total_staked);
			Self::deposit_event(Event::OperatorLeft {
				ex_operator: operator,
				unlocked_amount: total_backing,
				new_total_amt_locked: new_total_staked,
			});
			Ok(Some(actual_weight).into())
		}

		/// Returns an account's free balance which is not locked in delegation staking
		pub fn get_delegator_stakable_free_balance(acc: &T::AccountId) -> BalanceOf<T> {
			let mut balance = T::Currency::free_balance(acc);
			if let Some(state) = <DelegatorState<T>>::get(acc) {
				balance = balance.saturating_sub(state.total());
			}
			balance
		}

		/// Returns an account's free balance which is not locked in operator staking
		pub fn get_operator_stakable_free_balance(acc: &T::AccountId) -> BalanceOf<T> {
			let mut balance = T::Currency::free_balance(acc);
			if let Some(info) = <OperatorInfo<T>>::get(acc) {
				balance = balance.saturating_sub(info.bond);
			}
			balance
		}

		/// Returns a delegations auto-compound value.
		pub fn delegation_auto_compound(
			operator: &T::AccountId,
			delegator: &T::AccountId,
		) -> Percent {
			<AutoCompoundDelegations<T>>::auto_compound(operator, delegator)
		}

		/// Caller must ensure operator is active before calling
		pub(crate) fn update_active(operator: T::AccountId, total: BalanceOf<T>) {
			let mut operators = <OperatorPool<T>>::get();
			operators.remove(&Bond::from_owner(operator.clone()));
			operators.try_insert(Bond { owner: operator, amount: total }).expect(
				"the operator is removed in previous step so the length cannot increase; qed",
			);
			<OperatorPool<T>>::put(operators);
		}

		/// Remove delegation from operator state
		/// Amount input should be retrieved from delegator and it informs the storage lookups
		pub(crate) fn delegator_leaves_operator(
			operator: T::AccountId,
			delegator: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let mut state = <OperatorInfo<T>>::get(&operator).ok_or(Error::<T>::OperatorDNE)?;
			state.rm_delegation_if_exists::<T>(&operator, delegator.clone(), amount)?;
			let new_total_locked = <Total<T>>::get().saturating_sub(amount);
			<Total<T>>::put(new_total_locked);
			let new_total = state.total_counted;
			<OperatorInfo<T>>::insert(&operator, state);
			Self::deposit_event(Event::DelegatorLeftOperator {
				delegator,
				operator,
				unstaked_amount: amount,
				total_operator_staked: new_total,
			});
			Ok(())
		}

		pub(crate) fn prepare_staking_payouts(
			round_info: RoundInfo<BlockNumberFor<T>>,
			round_duration: u64,
		) -> Weight {
			let RoundInfo { current: now, length: round_length, .. } = round_info;

			// This function is called right after the round index increment,
			// and the goal is to compute the payout informations for the round that just ended.
			// We don't need to saturate here because the genesis round is 1.
			let prepare_payout_for_round = now - 1;

			// Return early if there is no blocks for this round
			if <Points<T>>::get(prepare_payout_for_round).is_zero() {
				return Weight::zero();
			}

			<T as Config>::WeightInfo::prepare_staking_payouts()
		}

		/// Wrapper around pay_one_operator_reward which handles the following logic:
		/// * whether or not a payout needs to be made
		/// * cleaning up when payouts are done
		/// * returns the weight consumed by pay_one_operator_reward if applicable
		fn handle_delayed_payouts(now: RoundIndex) -> Weight {
			let delay = T::RewardPaymentDelay::get();

			// don't underflow uint
			if now < delay {
				return Weight::from_parts(0u64, 0);
			}

			let paid_for_round = now.saturating_sub(delay);

			if let Some(payout_info) = <DelayedPayouts<T>>::get(paid_for_round) {
				let result = Self::pay_one_operator_reward(paid_for_round, payout_info);

				// clean up storage items that we no longer need
				if matches!(result.0, RewardPayment::Finished) {
					<DelayedPayouts<T>>::remove(paid_for_round);
					<Points<T>>::remove(paid_for_round);
				}
				result.1 // weight consumed by pay_one_operator_reward
			} else {
				Weight::from_parts(0u64, 0)
			}
		}

		/// Payout a single operator from the given round.
		///
		/// Returns an optional tuple of (Operator's AccountId, total paid)
		/// or None if there were no more payouts to be made for the round.
		pub(crate) fn pay_one_operator_reward(
			paid_for_round: RoundIndex,
			payout_info: DelayedPayout<BalanceOf<T>>,
		) -> (RewardPayment, Weight) {
			// 'early_weight' tracks weight used for reads/writes done early in this fn before its
			// early-exit codepaths.
			let mut early_weight = Weight::zero();

			// TODO: it would probably be optimal to roll Points into the DelayedPayouts storage
			// item so that we do fewer reads each block
			let total_points = <Points<T>>::get(paid_for_round);
			early_weight = early_weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));

			if total_points.is_zero() {
				// TODO: this case is obnoxious... it's a value query, so it could mean one of two
				// different logic errors:
				// 1. we removed it before we should have
				// 2. we called pay_one_operator_reward when we were actually done with deferred
				//    payouts
				log::warn!("pay_one_operator_reward called with no <Points<T>> for the round!");
				return (RewardPayment::Finished, early_weight);
			}

			let operator_fee = payout_info.operator_commission;
			let operator_issuance = operator_fee * payout_info.round_issuance;
			if let Some((operator, state)) =
				<AtStake<T>>::iter_prefix(paid_for_round).drain().next()
			{
				// read and kill AtStake
				early_weight = early_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

				// Take the awarded points for the operator
				let pts = <AwardedPts<T>>::take(paid_for_round, &operator);
				// read and kill AwardedPts
				early_weight = early_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
				if pts == 0 {
					return (RewardPayment::Skipped, early_weight);
				}

				// 'extra_weight' tracks weight returned from fns that we delegate to which can't be
				// known ahead of time.
				let mut extra_weight = Weight::zero();
				let pct_due = Perbill::from_rational(pts, total_points);
				let total_paid = pct_due * payout_info.total_staking_reward;
				let mut amt_due = total_paid;

				let num_delegators = state.delegations.len();
				let mut num_paid_delegations = 0u32;
				let mut num_auto_compounding = 0u32;
				let num_scheduled_requests =
					<DelegationScheduledRequests<T>>::decode_len(&operator).unwrap_or_default();
				if state.delegations.is_empty() {
					// solo operator with no delegators
					extra_weight = extra_weight
						.saturating_add(T::PayoutOperatorReward::payout_operator_reward(
							paid_for_round,
							operator.clone(),
							amt_due,
						))
						.saturating_add(T::OnOperatorPayout::on_operator_payout(
							paid_for_round,
							operator.clone(),
							amt_due,
						));
				} else {
					// pay operator first; commission + due_portion
					let operator_pct = Perbill::from_rational(state.bond, state.total);
					let commission = pct_due * operator_issuance;
					amt_due = amt_due.saturating_sub(commission);
					let operator_reward = (operator_pct * amt_due).saturating_add(commission);
					extra_weight = extra_weight
						.saturating_add(T::PayoutOperatorReward::payout_operator_reward(
							paid_for_round,
							operator.clone(),
							operator_reward,
						))
						.saturating_add(T::OnOperatorPayout::on_operator_payout(
							paid_for_round,
							operator.clone(),
							operator_reward,
						));

					// pay delegators due portion
					for BondWithAutoCompound { owner, amount, auto_compound } in state.delegations {
						let percent = Perbill::from_rational(amount, state.total);
						let due = percent * amt_due;
						if !due.is_zero() {
							num_auto_compounding += if auto_compound.is_zero() { 0 } else { 1 };
							num_paid_delegations += 1u32;
							Self::mint_and_compound(
								due,
								auto_compound.clone(),
								operator.clone(),
								owner.clone(),
							);
						}
					}
				}

				extra_weight = extra_weight.saturating_add(
					<T as Config>::WeightInfo::pay_one_operator_reward_best(
						num_paid_delegations,
						num_auto_compounding,
						num_scheduled_requests as u32,
					),
				);

				(
					RewardPayment::Paid,
					<T as Config>::WeightInfo::pay_one_operator_reward(num_delegators as u32)
						.saturating_add(extra_weight),
				)
			} else {
				// Note that we don't clean up storage here; it is cleaned up in
				// handle_delayed_payouts()
				(RewardPayment::Finished, Weight::from_parts(0u64, 0))
			}
		}

		/// Compute the top `TotalSelected` operators in the OperatorPool and return
		/// a vec of their AccountIds (sorted by AccountId).
		///
		/// If the returned vec is empty, the previous operators should be used.
		pub fn compute_top_operator() -> Vec<T::AccountId> {
			let top_n = <TotalSelected<T>>::get() as usize;
			if top_n == 0 {
				return vec![];
			}

			let operators = <OperatorPool<T>>::get().0;

			// If the number of operators is greater than top_n, select the operators with higher
			// amount. Otherwise, return all the operators.
			if operators.len() > top_n {
				// Partially sort operators such that element at index `top_n - 1` is sorted, and
				// all the elements in the range 0..top_n are the top n elements.
				let sorted_operator = operators
					.try_mutate(|inner| {
						inner.select_nth_unstable_by(top_n - 1, |a, b| {
							// Order by amount, then owner. The owner is needed to ensure a stable order
							// when two accounts have the same amount.
							a.amount.cmp(&b.amount).then_with(|| a.owner.cmp(&b.owner)).reverse()
						});
					})
					.expect("sort cannot increase item count; qed");

				let mut operators =
					sorted_operator.into_iter().take(top_n).map(|x| x.owner).collect::<Vec<_>>();

				// Sort operators by AccountId
				operators.sort();

				operators
			} else {
				// Return all operators
				// The operators are already sorted by AccountId, so no need to sort again
				operators.into_iter().map(|x| x.owner).collect::<Vec<_>>()
			}
		}
		/// Best as in most cumulatively supported in terms of stake
		/// Returns [operator_count, delegation_count, total staked]
		pub(crate) fn select_top_operators(now: RoundIndex) -> (Weight, u32, u32, BalanceOf<T>) {
			let (mut operator_count, mut delegation_count, mut total) =
				(0u32, 0u32, BalanceOf::<T>::zero());
			// choose the top TotalSelected qualified operators, ordered by stake
			let operators = Self::compute_top_operator();
			if operators.is_empty() {
				// SELECTION FAILED TO SELECT >=1 operator => select operators from previous round
				let last_round = now.saturating_sub(1u32);
				let mut total_per_operator: BTreeMap<T::AccountId, BalanceOf<T>> = BTreeMap::new();
				// set this round AtStake to last round AtStake
				for (account, snapshot) in <AtStake<T>>::iter_prefix(last_round) {
					operator_count = operator_count.saturating_add(1u32);
					delegation_count =
						delegation_count.saturating_add(snapshot.delegations.len() as u32);
					total = total.saturating_add(snapshot.total);
					total_per_operator.insert(account.clone(), snapshot.total);
					<AtStake<T>>::insert(now, account, snapshot);
				}
				// `SelectedOperators` remains unchanged from last round
				// emit OperatorChosen event for tools that use this event
				for operator in <SelectedOperators<T>>::get() {
					let snapshot_total = total_per_operator
						.get(&operator)
						.expect("all selected operators have snapshots");
					Self::deposit_event(Event::OperatorChosen {
						round: now,
						operator_account: operator,
						total_exposed_amount: *snapshot_total,
					})
				}
				let weight = <T as Config>::WeightInfo::select_top_operators(0, 0);
				return (weight, operator_count, delegation_count, total);
			}

			// snapshot exposure for round for weighting reward distribution
			for account in operators.iter() {
				let state = <OperatorInfo<T>>::get(account)
					.expect("all members of OperatorQ must be operators");

				operator_count = operator_count.saturating_add(1u32);
				delegation_count = delegation_count.saturating_add(state.delegation_count);
				total = total.saturating_add(state.total_counted);
				let CountedDelegations { uncounted_stake, rewardable_delegations } =
					Self::get_rewardable_delegators(&account);
				let total_counted = state.total_counted.saturating_sub(uncounted_stake);

				let auto_compounding_delegations = <AutoCompoundingDelegations<T>>::get(&account)
					.into_iter()
					.map(|x| (x.delegator, x.value))
					.collect::<BTreeMap<_, _>>();
				let rewardable_delegations = rewardable_delegations
					.into_iter()
					.map(|d| BondWithAutoCompound {
						owner: d.owner.clone(),
						amount: d.amount,
						auto_compound: auto_compounding_delegations
							.get(&d.owner)
							.cloned()
							.unwrap_or_else(|| Percent::zero()),
					})
					.collect();

				let snapshot = OperatorSnapshot {
					bond: state.bond,
					delegations: rewardable_delegations,
					total: total_counted,
				};
				<AtStake<T>>::insert(now, account, snapshot);
				Self::deposit_event(Event::OperatorChosen {
					round: now,
					operator_account: account.clone(),
					total_exposed_amount: state.total_counted,
				});
			}
			// insert canonical operator set
			<SelectedOperators<T>>::put(
				BoundedVec::try_from(operators)
					.expect("subset of operators is always less than or equal to max operators"),
			);

			let avg_delegator_count = delegation_count.checked_div(operator_count).unwrap_or(0);
			let weight = <T as Config>::WeightInfo::select_top_operators(
				operator_count,
				avg_delegator_count,
			);
			(weight, operator_count, delegation_count, total)
		}

		/// Apply the delegator intent for revoke and decrease in order to build the
		/// effective list of delegators with their intended bond amount.
		///
		/// This will:
		/// - if [DelegationChange::Revoke] is outstanding, set the bond amount to 0.
		/// - if [DelegationChange::Decrease] is outstanding, subtract the bond by specified amount.
		/// - else, do nothing
		///
		/// The intended bond amounts will be used while calculating rewards.
		pub(crate) fn get_rewardable_delegators(operator: &T::AccountId) -> CountedDelegations<T> {
			let requests = <DelegationScheduledRequests<T>>::get(operator)
				.into_iter()
				.map(|x| (x.delegator, x.action))
				.collect::<BTreeMap<_, _>>();
			let mut uncounted_stake = BalanceOf::<T>::zero();
			let rewardable_delegations = <TopDelegations<T>>::get(operator)
				.expect("all members of OperatorQ must be operators")
				.delegations
				.into_iter()
				.map(|mut bond| {
					bond.amount = match requests.get(&bond.owner) {
						None => bond.amount,
						Some(DelegationAction::Revoke(_)) => {
							uncounted_stake = uncounted_stake.saturating_add(bond.amount);
							BalanceOf::<T>::zero()
						},
						Some(DelegationAction::Decrease(amount)) => {
							uncounted_stake = uncounted_stake.saturating_add(*amount);
							bond.amount.saturating_sub(*amount)
						},
					};

					bond
				})
				.collect();
			CountedDelegations { uncounted_stake, rewardable_delegations }
		}

		/// This function exists as a helper to delegator_bond_more & auto_compound functionality.
		/// Any changes to this function must align with both user-initiated bond increases and
		/// auto-compounding bond increases.
		/// Any feature-specific preconditions should be validated before this function is invoked.
		/// Any feature-specific events must be emitted after this function is invoked.
		pub fn delegation_bond_more_without_event(
			delegator: T::AccountId,
			operator: T::AccountId,
			more: BalanceOf<T>,
		) -> Result<
			(bool, Weight),
			DispatchErrorWithPostInfo<frame_support::dispatch::PostDispatchInfo>,
		> {
			let mut state = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			ensure!(
				!Self::delegation_request_revoke_exists(&operator, &delegator),
				Error::<T>::PendingDelegationRevoke
			);

			let actual_weight = <T as Config>::WeightInfo::delegator_bond_more(
				<DelegationScheduledRequests<T>>::decode_len(&operator).unwrap_or_default() as u32,
			);
			let in_top = state.increase_delegation::<T>(operator.clone(), more).map_err(|err| {
				DispatchErrorWithPostInfo { post_info: Some(actual_weight).into(), error: err }
			})?;

			Ok((in_top, actual_weight))
		}

		/// Mint a specified reward amount to the beneficiary account. Emits the [Rewarded] event.
		pub fn mint(amt: BalanceOf<T>, to: T::AccountId) {
			if let Ok(amount_transferred) = T::Currency::deposit_into_existing(&to, amt) {
				Self::deposit_event(Event::Rewarded {
					account: to.clone(),
					rewards: amount_transferred.peek(),
				});
			}
		}

		/// Mint a specified reward amount to the operator's account. Emits the [Rewarded] event.
		pub fn mint_operator_reward(
			_paid_for_round: RoundIndex,
			operator_id: T::AccountId,
			amt: BalanceOf<T>,
		) -> Weight {
			if let Ok(amount_transferred) = T::Currency::deposit_into_existing(&operator_id, amt) {
				Self::deposit_event(Event::Rewarded {
					account: operator_id.clone(),
					rewards: amount_transferred.peek(),
				});
			}
			<T as Config>::WeightInfo::mint_operator_reward()
		}

		/// Mint and compound delegation rewards. The function mints the amount towards the
		/// delegator and tries to compound a specified percent of it back towards the delegation.
		/// If a scheduled delegation revoke exists, then the amount is only minted, and nothing is
		/// compounded. Emits the [Compounded] event.
		pub fn mint_and_compound(
			amt: BalanceOf<T>,
			compound_percent: Percent,
			operator: T::AccountId,
			delegator: T::AccountId,
		) {
			if let Ok(amount_transferred) =
				T::Currency::deposit_into_existing(&delegator, amt.clone())
			{
				Self::deposit_event(Event::Rewarded {
					account: delegator.clone(),
					rewards: amount_transferred.peek(),
				});

				let compound_amount = compound_percent.mul_ceil(amount_transferred.peek());
				if compound_amount.is_zero() {
					return;
				}

				if let Err(err) = Self::delegation_bond_more_without_event(
					delegator.clone(),
					operator.clone(),
					compound_amount.clone(),
				) {
					log::debug!(
						"skipped compounding staking reward towards operator '{:?}' for delegator '{:?}': {:?}",
						operator,
						delegator,
						err
					);
					return;
				};

				Pallet::<T>::deposit_event(Event::Compounded {
					delegator,
					operator,
					amount: compound_amount.clone(),
				});
			};
		}
	}

	impl<T: Config> Get<Vec<T::AccountId>> for Pallet<T> {
		fn get() -> Vec<T::AccountId> {
			Self::selected_operator().into_inner()
		}
	}
}

impl<T: Config> sp_runtime::BoundToRuntimeAppPublic for Pallet<T> {
	type Public = T::RoleKeyId;
}

impl<T: Config> OneSessionHandler<T::AccountId> for Pallet<T> {
	type Key = T::RoleKeyId;

	fn on_genesis_session<'a, I: 'a>(validators: I)
	where
		I: Iterator<Item = (&'a T::AccountId, T::RoleKeyId)>,
	{
	}

	fn on_new_session<'a, I: 'a>(_changed: bool, validators: I, _queued_validators: I)
	where
		I: Iterator<Item = (&'a T::AccountId, T::RoleKeyId)>,
	{
	}

	fn on_disabled(_i: u32) {
		// ignore
	}

	// Distribute the inflation rewards
	fn on_before_session_ending() {
		// ignore
	}
}
