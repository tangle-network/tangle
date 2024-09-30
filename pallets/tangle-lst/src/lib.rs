// This file is part of Substrate.

//! # Nomination Pools for Liquid Staking

//! A pallet that allows members to delegate their stake to nominating pools. A nomination pool acts
//! as a nominator and nominates validators on behalf of its members.

//! ## Key Terms

//! * pool id: A unique identifier for each pool. Set to u32.
//! * bonded pool: Tracks the distribution of actively staked funds. See [`BondedPool`] and
//!   [`BondedPoolInner`].
//! * reward pool: Tracks rewards earned by actively staked funds. See [`RewardPool`] and
//!   [`RewardPools`].
//! * unbonding sub pools: Collection of pools at different phases of the unbonding lifecycle. See
//!   [`SubPools`] and [`SubPoolsStorage`].
//! * members: Accounts that are members of pools. See [`PoolMember`] and [`PoolMembers`].
//! * roles: Administrative roles of each pool, capable of controlling nomination and the state of
//!   the pool.
//! * point: A unit of measure for a member's portion of a pool's funds. Points initially have a
//!   ratio of 1 (as set by `POINTS_TO_BALANCE_INIT_RATIO`) to balance, but this can change as
//!   slashing occurs.
//! * kick: The act of a pool administrator forcibly ejecting a member.
//! * bonded account: A key-less account id derived from the pool id that acts as the bonded
//!   account.
//! * reward account: A similar key-less account that is set as the `Payee` account for the bonded
//!   account for all staking rewards.
//! * change rate: The rate at which pool commission can be changed. A change rate consists of a
//!   `max_increase` and `min_delay`, dictating the maximum percentage increase that can be applied
//!   to the commission per number of blocks.
//! * throttle: An attempted commission increase is throttled if the attempted change falls outside
//!   the change rate bounds.

//! ## Usage

//! ### Join

//! An account can stake funds with a nomination pool by calling [`Call::join`].

//! ### Claim rewards

//! After joining a pool, a member can claim rewards by calling [`Call::claim_payout`].

//! A pool member can also set a `ClaimPermission` with [`Call::set_claim_permission`], to allow
//! other members to permissionlessly bond or withdraw their rewards by calling
//! [`Call::bond_extra_other`] or [`Call::claim_payout_other`] respectively.

//! ### Leave

//! To leave, a member must take two steps:

//! 1. Call [`Call::unbond`] to start the unbonding process for all or a portion of their funds.
//! 2. Once [`sp_staking::StakingInterface::bonding_duration`] eras have passed, call
//!    [`Call::withdraw_unbonded`] to withdraw any funds that are free.

//! ### Slashes

//! Slashes are distributed evenly across the bonded pool and the unbonding pools from slash era+1
//! through the slash apply era. Any member who either unbonded or was actively bonded in this range
//! of eras will be affected by the slash. A member is slashed pro-rata based on its stake relative
//! to the total slash amount.

//! ### Administration

//! A pool can be created with the [`Call::create`] or [`Call::create_with_pool_id`] calls. Once
//! created, the pool's nominator or root user must call [`Call::nominate`] to start nominating.

//! Similar to [`Call::nominate`], [`Call::chill`] will chill the pool in the staking system, and
//! [`Call::pool_withdraw_unbonded`] will withdraw any unbonding chunks of the pool bonded account.

//! To help facilitate pool administration, the pool has one of three states (see [`PoolState`]):
//! Open, Blocked, or Destroying.

//! A pool has 4 administrative roles (see [`PoolRoles`]): Depositor, Nominator, Bouncer, and Root.

//! ### Commission

//! A pool can optionally have a commission configuration, set by the `root` role with
//! [`Call::set_commission`] and claimed with [`Call::claim_commission`]. Commission is subject to
//! a global maximum and a change rate, which can be set with [`Call::set_commission_max`] and
//! [`Call::set_commission_change_rate`] respectively.

//! ### Dismantling

//! A pool is destroyed once all members have fully unbonded and withdrawn, and the depositor has
//! fully unbonded and withdrawn.

//! ## New Features

//! ### Liquid Staking Tokens

//! The pallet now supports the creation of liquid staking tokens for each pool. When a member joins
//! a pool, they receive liquid staking tokens representing their share of the pool. These tokens
//! can be transferred or used in other DeFi applications while the underlying stake remains bonded.

//! ### Pool Creation with Specific ID

//! Pools can now be created with a specific ID using the [`Call::create_with_pool_id`] function.
//! This allows for more flexible pool management and integration with external systems.

//! ### Adjustable Pool Deposit

//! The [`Call::adjust_pool_deposit`] function allows for topping up the deficit or withdrawing the
//! excess Existential Deposit (ED) from the pool's reward account. This ensures that the pool
//! always has the correct ED, even if the ED requirement changes over time.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use frame_support::traits::fungibles;
use frame_support::traits::fungibles::Create;
use frame_support::traits::fungibles::Inspect as FungiblesInspect;
use frame_support::traits::fungibles::Mutate as FungiblesMutate;
use frame_support::traits::tokens::Precision;
use frame_support::traits::tokens::Preservation;
use frame_support::traits::Currency;
use frame_support::traits::ExistenceRequirement;
use frame_support::traits::LockableCurrency;
use frame_support::traits::ReservableCurrency;
use frame_support::{
	defensive, defensive_assert, ensure,
	pallet_prelude::{MaxEncodedLen, *},
	storage::bounded_btree_map::BoundedBTreeMap,
	traits::{
		tokens::Fortitude, Defensive, DefensiveOption, DefensiveResult, DefensiveSaturating, Get,
	},
	DefaultNoBound, PalletError,
};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use sp_core::U256;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::traits::{
	AtLeast32BitUnsigned, Bounded, CheckedAdd, Convert, Saturating, StaticLookup, Zero,
};
use sp_runtime::FixedPointNumber;
use sp_runtime::Perbill;
use sp_staking::{EraIndex, StakingInterface};
use sp_std::{collections::btree_map::BTreeMap, fmt::Debug, ops::Div, vec::Vec};

/// The log target of this pallet.
pub const LOG_TARGET: &str = "runtime::nomination-pools";
// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: $crate::LOG_TARGET,
			concat!("[{:?}] 🏊‍♂️ ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

#[cfg(any(test, feature = "fuzzing"))]
pub mod mock;
#[cfg(test)]
mod tests;

pub mod types;
pub mod weights;
pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::traits::StorageVersion;
	use frame_system::{ensure_signed, pallet_prelude::*};
	use sp_runtime::Perbill;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(8);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: weights::WeightInfo;

		/// The currency type used for nomination pool.
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;

		/// The overarching freeze reason.
		type RuntimeFreezeReason: From<FreezeReason>;

		/// The type that is used for reward counter.
		///
		/// The arithmetic of the reward counter might saturate based on the size of the
		/// `Currency::Balance`. If this happens, operations fails. Nonetheless, this type should be
		/// chosen such that this failure almost never happens, as if it happens, the pool basically
		/// needs to be dismantled (or all pools migrated to a larger `RewardCounter` type, which is
		/// a PITA to do).
		///
		/// See the inline code docs of `Member::pending_rewards` and `RewardPool::update_recorded`
		/// for example analysis. A [`sp_runtime::FixedU128`] should be fine for chains with balance
		/// types similar to that of Polkadot and Kusama, in the absence of severe slashing (or
		/// prevented via a reasonable `MaxPointsToBalance`), for many many years to come.
		type RewardCounter: FixedPointNumber + MaxEncodedLen + TypeInfo + Default + codec::FullCodec;

		/// The nomination pool's pallet id.
		#[pallet::constant]
		type PalletId: Get<frame_support::PalletId>;

		/// The maximum pool points-to-balance ratio that an `open` pool can have.
		///
		/// This is important in the event slashing takes place and the pool's points-to-balance
		/// ratio becomes disproportional.
		///
		/// Moreover, this relates to the `RewardCounter` type as well, as the arithmetic operations
		/// are a function of number of points, and by setting this value to e.g. 10, you ensure
		/// that the total number of points in the system are at most 10 times the total_issuance of
		/// the chain, in the absolute worse case.
		///
		/// For a value of 10, the threshold would be a pool points-to-balance ratio of 10:1.
		/// Such a scenario would also be the equivalent of the pool being 90% slashed.
		#[pallet::constant]
		type MaxPointsToBalance: Get<u8>;

		/// The maximum number of simultaneous unbonding chunks that can exist per member.
		#[pallet::constant]
		type MaxUnbonding: Get<u32>;

		/// The maximum length of a pool name.
		#[pallet::constant]
		type MaxNameLength: Get<u32> + Clone;

		/// Infallible method for converting `Currency::Balance` to `U256`.
		type BalanceToU256: Convert<BalanceOf<Self>, U256>;

		/// Infallible method for converting `U256` to `Currency::Balance`.
		type U256ToBalance: Convert<U256, BalanceOf<Self>>;

		/// The interface for nominating.
		type Staking: StakingInterface<Balance = BalanceOf<Self>, AccountId = Self::AccountId>;

		/// The amount of eras a `SubPools::with_era` pool can exist before it gets merged into the
		/// `SubPools::no_era` pool. In other words, this is the amount of eras a member will be
		/// able to withdraw from an unbonding pool which is guaranteed to have the correct ratio of
		/// points to balance; once the `with_era` pool is merged into the `no_era` pool, the ratio
		/// can become skewed due to some slashed ratio getting merged in at some point.
		type PostUnbondingPoolsWindow: Get<u32>;

		/// The maximum length, in bytes, that a pools metadata maybe.
		type MaxMetadataLen: Get<u32>;

		/// The fungibles trait used for managing fungible assets.
		type Fungibles: fungibles::Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = BalanceOf<Self>>
			+ fungibles::Mutate<Self::AccountId, AssetId = Self::AssetId>
			+ fungibles::Create<Self::AccountId>;

		/// The asset ID type.
		type AssetId: AtLeast32BitUnsigned
			+ Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ Clone
			+ Copy
			+ PartialOrd
			+ MaxEncodedLen;

		/// The pool ID type.
		type PoolId: AtLeast32BitUnsigned
			+ Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ Clone
			+ Copy
			+ PartialOrd
			+ MaxEncodedLen;

		/// The origin with privileged access
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	/// The sum of funds across all pools.
	///
	/// This might be lower but never higher than the sum of `total_balance` of all [`PoolMembers`]
	/// because calling `pool_withdraw_unbonded` might decrease the total stake of the pool's
	/// `bonded_account` without adjusting the pallet-internal `UnbondingPool`'s.
	#[pallet::storage]
	pub type TotalValueLocked<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// Minimum amount to bond to join a pool.
	#[pallet::storage]
	pub type MinJoinBond<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// Minimum bond required to create a pool.
	///
	/// This is the amount that the depositor must put as their initial stake in the pool, as an
	/// indication of "skin in the game".
	///
	/// This is the value that will always exist in the staking ledger of the pool bonded account
	/// while all other accounts leave.
	#[pallet::storage]
	pub type MinCreateBond<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// Maximum number of nomination pools that can exist. If `None`, then an unbounded number of
	/// pools can exist.
	#[pallet::storage]
	pub type MaxPools<T: Config> = StorageValue<_, u32, OptionQuery>;

	/// The maximum commission that can be charged by a pool. Used on commission payouts to bound
	/// pool commissions that are > `GlobalMaxCommission`, necessary if a future
	/// `GlobalMaxCommission` is lower than some current pool commissions.
	#[pallet::storage]
	pub type GlobalMaxCommission<T: Config> = StorageValue<_, Perbill, OptionQuery>;

	/// Storage for bonded pools.
	// To get or insert a pool see [`BondedPool::get`] and [`BondedPool::put`]
	#[pallet::storage]
	pub type BondedPools<T: Config> =
		CountedStorageMap<_, Twox64Concat, PoolId, BondedPoolInner<T>>;

	/// Reward pools. This is where there rewards for each pool accumulate. When a members payout is
	/// claimed, the balance comes out fo the reward pool. Keyed by the bonded pools account.
	#[pallet::storage]
	pub type RewardPools<T: Config> = CountedStorageMap<_, Twox64Concat, PoolId, RewardPool<T>>;

	/// Groups of unbonding pools. Each group of unbonding pools belongs to a
	/// bonded pool, hence the name sub-pools. Keyed by the bonded pools account.
	#[pallet::storage]
	pub type SubPoolsStorage<T: Config> = CountedStorageMap<_, Twox64Concat, PoolId, SubPools<T>>;

	/// Metadata for the pool.
	#[pallet::storage]
	pub type Metadata<T: Config> =
		CountedStorageMap<_, Twox64Concat, PoolId, BoundedVec<u8, T::MaxMetadataLen>, ValueQuery>;

	/// Ever increasing number of all pools created so far.
	#[pallet::storage]
	pub type LastPoolId<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Unbonding members.
	///
	/// TWOX-NOTE: SAFE since `AccountId` is a secure hash.
	#[pallet::storage]
	pub type UnbondingMembers<T: Config> =
		CountedStorageMap<_, Twox64Concat, T::AccountId, PoolMember<T>>;

	/// A reverse lookup from the pool's account id to its id.
	///
	/// This is only used for slashing. In all other instances, the pool id is used, and the
	/// accounts are deterministically derived from it.
	#[pallet::storage]
	pub type ReversePoolIdLookup<T: Config> =
		CountedStorageMap<_, Twox64Concat, T::AccountId, PoolId, OptionQuery>;

	/// Map from a pool member account to their opted claim permission.
	#[pallet::storage]
	pub type ClaimPermissions<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, ClaimPermission, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub min_join_bond: BalanceOf<T>,
		pub min_create_bond: BalanceOf<T>,
		pub max_pools: Option<u32>,
		pub max_members_per_pool: Option<u32>,
		pub max_members: Option<u32>,
		pub global_max_commission: Option<Perbill>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				min_join_bond: Zero::zero(),
				min_create_bond: Zero::zero(),
				max_pools: Some(16),
				max_members_per_pool: Some(32),
				max_members: Some(16 * 32),
				global_max_commission: None,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			MinJoinBond::<T>::put(self.min_join_bond);
			MinCreateBond::<T>::put(self.min_create_bond);

			if let Some(max_pools) = self.max_pools {
				MaxPools::<T>::put(max_pools);
			}
			if let Some(global_max_commission) = self.global_max_commission {
				GlobalMaxCommission::<T>::put(global_max_commission);
			}
		}
	}

	/// Events of this pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A pool has been created.
		Created { depositor: T::AccountId, pool_id: PoolId },
		/// A member has became bonded in a pool.
		Bonded { member: T::AccountId, pool_id: PoolId, bonded: BalanceOf<T>, joined: bool },
		/// A payout has been made to a member.
		PaidOut { member: T::AccountId, pool_id: PoolId, payout: BalanceOf<T> },
		/// A member has unbonded from their pool.
		///
		/// - `balance` is the corresponding balance of the number of points that has been
		///   requested to be unbonded (the argument of the `unbond` transaction) from the bonded
		///   pool.
		/// - `points` is the number of points that are issued as a result of `balance` being
		/// dissolved into the corresponding unbonding pool.
		/// - `era` is the era in which the balance will be unbonded.
		/// In the absence of slashing, these values will match. In the presence of slashing, the
		/// number of points that are issued in the unbonding pool will be less than the amount
		/// requested to be unbonded.
		Unbonded {
			member: T::AccountId,
			pool_id: PoolId,
			balance: BalanceOf<T>,
			points: BalanceOf<T>,
			era: EraIndex,
		},
		/// A member has withdrawn from their pool.
		///
		/// The given number of `points` have been dissolved in return of `balance`.
		///
		/// Similar to `Unbonded` event, in the absence of slashing, the ratio of point to balance
		/// will be 1.
		Withdrawn {
			member: T::AccountId,
			pool_id: PoolId,
			balance: BalanceOf<T>,
			points: BalanceOf<T>,
		},
		/// A pool has been destroyed.
		Destroyed { pool_id: PoolId },
		/// The state of a pool has changed
		StateChanged { pool_id: PoolId, new_state: PoolState },
		/// A member has been removed from a pool.
		///
		/// The removal can be voluntary (withdrawn all unbonded funds) or involuntary (kicked).
		MemberRemoved { pool_id: PoolId, member: T::AccountId },
		/// The roles of a pool have been updated to the given new roles. Note that the depositor
		/// can never change.
		RolesUpdated {
			root: Option<T::AccountId>,
			bouncer: Option<T::AccountId>,
			nominator: Option<T::AccountId>,
		},
		/// The active balance of pool `pool_id` has been slashed to `balance`.
		PoolSlashed { pool_id: PoolId, balance: BalanceOf<T> },
		/// The unbond pool at `era` of pool `pool_id` has been slashed to `balance`.
		UnbondingPoolSlashed { pool_id: PoolId, era: EraIndex, balance: BalanceOf<T> },
		/// A pool's commission setting has been changed.
		PoolCommissionUpdated { pool_id: PoolId, current: Option<(Perbill, T::AccountId)> },
		/// A pool's maximum commission setting has been changed.
		PoolMaxCommissionUpdated { pool_id: PoolId, max_commission: Perbill },
		/// A pool's commission `change_rate` has been changed.
		PoolCommissionChangeRateUpdated {
			pool_id: PoolId,
			change_rate: CommissionChangeRate<BlockNumberFor<T>>,
		},
		/// Pool commission claim permission has been updated.
		PoolCommissionClaimPermissionUpdated {
			pool_id: PoolId,
			permission: Option<CommissionClaimPermission<T::AccountId>>,
		},
		/// Pool commission has been claimed.
		PoolCommissionClaimed { pool_id: PoolId, commission: BalanceOf<T> },
		/// Topped up deficit in frozen ED of the reward pool.
		MinBalanceDeficitAdjusted { pool_id: PoolId, amount: BalanceOf<T> },
		/// Claimed excess frozen ED of af the reward pool.
		MinBalanceExcessAdjusted { pool_id: PoolId, amount: BalanceOf<T> },
	}

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq))]
	pub enum Error<T> {
		/// A (bonded) pool id does not exist.
		PoolNotFound,
		/// An account is not a member.
		PoolMemberNotFound,
		/// A reward pool does not exist. In all cases this is a system logic error.
		RewardPoolNotFound,
		/// A sub pool does not exist.
		SubPoolsNotFound,
		/// The member is fully unbonded (and thus cannot access the bonded and reward pool
		/// anymore to, for example, collect rewards).
		FullyUnbonding,
		/// The member cannot unbond further chunks due to reaching the limit.
		MaxUnbondingLimit,
		/// None of the funds can be withdrawn yet because the bonding duration has not passed.
		CannotWithdrawAny,
		/// The amount does not meet the minimum bond to either join or create a pool.
		///
		/// The depositor can never unbond to a value less than `Pallet::depositor_min_bond`. The
		/// caller does not have nominating permissions for the pool. Members can never unbond to a
		/// value below `MinJoinBond`.
		MinimumBondNotMet,
		/// The transaction could not be executed due to overflow risk for the pool.
		OverflowRisk,
		/// A pool must be in [`PoolState::Destroying`] in order for the depositor to unbond or for
		/// other members to be permissionlessly unbonded.
		NotDestroying,
		/// The caller does not have nominating permissions for the pool.
		NotNominator,
		/// Either a) the caller cannot make a valid kick or b) the pool is not destroying.
		NotKickerOrDestroying,
		/// The pool is not open to join
		NotOpen,
		/// The system is maxed out on pools.
		MaxPools,
		/// Too many members in the pool or system.
		MaxPoolMembers,
		/// The pools state cannot be changed.
		CanNotChangeState,
		/// The caller does not have adequate permissions.
		DoesNotHavePermission,
		/// Metadata exceeds [`Config::MaxMetadataLen`]
		MetadataExceedsMaxLen,
		/// Some error occurred that should never happen. This should be reported to the
		/// maintainers.
		Defensive(DefensiveError),
		/// Partial unbonding now allowed permissionlessly.
		PartialUnbondNotAllowedPermissionlessly,
		/// The pool's max commission cannot be set higher than the existing value.
		MaxCommissionRestricted,
		/// The supplied commission exceeds the max allowed commission.
		CommissionExceedsMaximum,
		/// The supplied commission exceeds global maximum commission.
		CommissionExceedsGlobalMaximum,
		/// Not enough blocks have surpassed since the last commission update.
		CommissionChangeThrottled,
		/// The submitted changes to commission change rate are not allowed.
		CommissionChangeRateNotAllowed,
		/// There is no pending commission to claim.
		NoPendingCommission,
		/// No commission current has been set.
		NoCommissionCurrentSet,
		/// Pool id currently in use.
		PoolIdInUse,
		/// Pool id provided is not correct/usable.
		InvalidPoolId,
		/// Bonding extra is restricted to the exact pending reward amount.
		BondExtraRestricted,
		/// No imbalance in the ED deposit for the pool.
		NothingToAdjust,
		/// Pool token creation failed.
		PoolTokenCreationFailed,
		/// No balance to unbond.
		NoBalanceToUnbond,
	}

	#[derive(Encode, Decode, PartialEq, TypeInfo, PalletError, RuntimeDebug)]
	pub enum DefensiveError {
		/// There isn't enough space in the unbond pool.
		NotEnoughSpaceInUnbondPool,
		/// A (bonded) pool id does not exist.
		PoolNotFound,
		/// A reward pool does not exist. In all cases this is a system logic error.
		RewardPoolNotFound,
		/// A sub pool does not exist.
		SubPoolsNotFound,
		/// The bonded account should only be killed by the staking system when the depositor is
		/// withdrawing
		BondedStashKilledPrematurely,
	}

	impl<T> From<DefensiveError> for Error<T> {
		fn from(e: DefensiveError) -> Error<T> {
			Error::<T>::Defensive(e)
		}
	}

	/// A reason for freezing funds.
	#[pallet::composite_enum]
	pub enum FreezeReason {
		/// Pool reward account is restricted from going below Existential Deposit.
		#[codec(index = 0)]
		PoolMinBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Stake funds with a pool. The amount to bond is transferred from the member to the
		/// pools account and immediately increases the pools bond.
		///
		/// # Note
		///
		/// * This call will *not* dust the member account, so the member must have at least
		///   `existential deposit + amount` in their account.
		/// * Only a pool with [`PoolState::Open`] can be joined
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::join())]
		pub fn join(
			origin: OriginFor<T>,
			#[pallet::compact] amount: BalanceOf<T>,
			pool_id: PoolId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(amount >= MinJoinBond::<T>::get(), Error::<T>::MinimumBondNotMet);

			let mut bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			bonded_pool.ok_to_join()?;

			let mut reward_pool = RewardPools::<T>::get(pool_id)
				.defensive_ok_or::<Error<T>>(DefensiveError::RewardPoolNotFound.into())?;

			// IMPORTANT: reward pool records must be updated with the old points.
			reward_pool.update_records(
				pool_id,
				bonded_pool.points(),
				bonded_pool.commission.current(),
			)?;

			bonded_pool.try_bond_funds(&who, amount, BondType::Later)?;

			Self::deposit_event(Event::<T>::Bonded {
				member: who,
				pool_id,
				bonded: amount,
				joined: true,
			});

			bonded_pool.put();
			RewardPools::<T>::insert(pool_id, reward_pool);

			Ok(())
		}

		/// Bond `extra` more funds from `origin` into the pool to which they already belong.
		///
		/// Additional funds can come from either the free balance of the account, of from the
		/// accumulated rewards, see [`BondExtra`].
		///
		/// Bonding extra funds implies an automatic payout of all pending rewards as well.
		/// See `bond_extra_other` to bond pending rewards of `other` members.
		// NOTE: this transaction is implemented with the sole purpose of readability and
		// correctness, not optimization. We read/write several storage items multiple times instead
		// of just once, in the spirit reusing code.
		#[pallet::call_index(1)]
		#[pallet::weight(
			T::WeightInfo::bond_extra_transfer()
			.max(T::WeightInfo::bond_extra_other())
		)]
		pub fn bond_extra(
			origin: OriginFor<T>,
			pool_id: PoolId,
			extra: BondExtra<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_bond_extra(who.clone(), who, pool_id, extra)
		}

		/// Unbond up to `unbonding_points` of the `member_account`'s funds from the pool. It
		/// implicitly collects the rewards one last time, since not doing so would mean some
		/// rewards would be forfeited.
		///
		/// Under certain conditions, this call can be dispatched permissionlessly (i.e. by any
		/// account).
		///
		/// # Conditions for a permissionless dispatch.
		///
		/// * The pool is blocked and the caller is either the root or bouncer. This is refereed to
		///   as a kick.
		/// * The pool is destroying and the member is not the depositor.
		/// * The pool is destroying, the member is the depositor and no other members are in the
		///   pool.
		///
		/// ## Conditions for permissioned dispatch (i.e. the caller is also the
		/// `member_account`):
		///
		/// * The caller is not the depositor.
		/// * The caller is the depositor, the pool is destroying and no other members are in the
		///   pool.
		///
		/// # Note
		///
		/// If there are too many unlocking chunks to unbond with the pool account,
		/// [`Call::pool_withdraw_unbonded`] can be called to try and minimize unlocking chunks.
		/// The [`StakingInterface::unbond`] will implicitly call [`Call::pool_withdraw_unbonded`]
		/// to try to free chunks if necessary (ie. if unbound was called and no unlocking chunks
		/// are available). However, it may not be possible to release the current unlocking chunks,
		/// in which case, the result of this call will likely be the `NoMoreChunks` error from the
		/// staking system.
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::unbond())]
		pub fn unbond(
			origin: OriginFor<T>,
			member_account: AccountIdLookupOf<T>,
			pool_id: PoolId,
			#[pallet::compact] unbonding_points: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let member_account = T::Lookup::lookup(member_account)?;

			let mut bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

			let total_points = T::Fungibles::balance(pool_id.into(), &member_account);

			ensure!(total_points >= unbonding_points, Error::<T>::NoBalanceToUnbond);

			bonded_pool.ok_to_unbond_with(&who, &member_account, total_points, unbonding_points)?;

			// let burn the pool tokens
			T::Fungibles::burn_from(
				pool_id.into(),
				&member_account,
				unbonding_points,
				Preservation::Preserve,
				Precision::Exact,
				Fortitude::Force,
			)?;

			let current_era = T::Staking::current_era();
			let unbond_era = T::Staking::bonding_duration().saturating_add(current_era);

			// Unbond in the actual underlying nominator.
			let unbonding_balance = bonded_pool.dissolve(unbonding_points);
			T::Staking::unbond(&bonded_pool.bonded_account(), unbonding_balance)?;

			// Note that we lazily create the unbonding pools here if they don't already exist
			let mut sub_pools = SubPoolsStorage::<T>::get(pool_id)
				.unwrap_or_default()
				.maybe_merge_pools(current_era);

			// Update the unbond pool associated with the current era with the unbonded funds. Note
			// that we lazily create the unbond pool if it does not yet exist.
			if !sub_pools.with_era.contains_key(&unbond_era) {
				sub_pools
					.with_era
					.try_insert(unbond_era, UnbondPool::default())
					// The above call to `maybe_merge_pools` should ensure there is
					// always enough space to insert.
					.defensive_map_err::<Error<T>, _>(|_| {
						DefensiveError::NotEnoughSpaceInUnbondPool.into()
					})?;
			}

			let points_unbonded = sub_pools
				.with_era
				.get_mut(&unbond_era)
				// The above check ensures the pool exists.
				.defensive_ok_or::<Error<T>>(DefensiveError::PoolNotFound.into())?
				.issue(unbonding_balance);

			// Try and unbond in the member map.
			UnbondingMembers::<T>::try_mutate(
				member_account.clone(),
				|member| -> DispatchResult {
					let member = member.get_or_insert_with(|| PoolMember {
						pool_id,
						unbonding_eras: Default::default(),
					});
					member
						.unbonding_eras
						.try_insert(unbond_era, points_unbonded)
						.map(|old| {
							if old.is_some() {
								defensive!("value checked to not exist in the map; qed");
							}
						})
						.map_err(|_| Error::<T>::MaxUnbondingLimit)?;
					Ok(())
				},
			)?;

			Self::deposit_event(Event::<T>::Unbonded {
				member: member_account.clone(),
				pool_id,
				points: points_unbonded,
				balance: unbonding_balance,
				era: unbond_era,
			});

			// Now that we know everything has worked write the items to storage.
			SubPoolsStorage::insert(pool_id, sub_pools);
			Ok(())
		}

		/// Call `withdraw_unbonded` for the pools account. This call can be made by any account.
		///
		/// This is useful if there are too many unlocking chunks to call `unbond`, and some
		/// can be cleared by withdrawing. In the case there are too many unlocking chunks, the user
		/// would probably see an error like `NoMoreChunks` emitted from the staking system when
		/// they attempt to unbond.
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::pool_withdraw_unbonded(*num_slashing_spans))]
		pub fn pool_withdraw_unbonded(
			origin: OriginFor<T>,
			pool_id: PoolId,
			num_slashing_spans: u32,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

			// For now we only allow a pool to withdraw unbonded if its not destroying. If the pool
			// is destroying then `withdraw_unbonded` can be used.
			ensure!(pool.state != PoolState::Destroying, Error::<T>::NotDestroying);
			pool.withdraw_from_staking(num_slashing_spans)?;

			Ok(())
		}

		/// Withdraw unbonded funds from `member_account`. If no bonded funds can be unbonded, an
		/// error is returned.
		///
		/// Under certain conditions, this call can be dispatched permissionlessly (i.e. by any
		/// account).
		///
		/// # Conditions for a permissionless dispatch
		///
		/// * The pool is in destroy mode and the target is not the depositor.
		/// * The target is the depositor and they are the only member in the sub pools.
		/// * The pool is blocked and the caller is either the root or bouncer.
		///
		/// # Conditions for permissioned dispatch
		///
		/// * The caller is the target and they are not the depositor.
		///
		/// # Note
		///
		/// If the target is the depositor, the pool will be destroyed.
		#[pallet::call_index(5)]
		#[pallet::weight(
			T::WeightInfo::withdraw_unbonded_kill(*num_slashing_spans)
		)]
		pub fn withdraw_unbonded(
			origin: OriginFor<T>,
			member_account: AccountIdLookupOf<T>,
			pool_id: PoolId,
			num_slashing_spans: u32,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			let member_account = T::Lookup::lookup(member_account)?;
			let mut member = UnbondingMembers::<T>::get(&member_account)
				.ok_or(Error::<T>::PoolMemberNotFound)?;
			let current_era = T::Staking::current_era();

			let bonded_pool = BondedPool::<T>::get(member.pool_id)
				.defensive_ok_or::<Error<T>>(DefensiveError::PoolNotFound.into())?;
			let mut sub_pools =
				SubPoolsStorage::<T>::get(member.pool_id).ok_or(Error::<T>::SubPoolsNotFound)?;

			bonded_pool.ok_to_withdraw_unbonded_with(&caller, &member_account)?;

			// NOTE: must do this after we have done the `ok_to_withdraw_unbonded_other_with` check.
			let withdrawn_points = member.withdraw_unlocked(current_era);
			ensure!(!withdrawn_points.is_empty(), Error::<T>::CannotWithdrawAny);

			// Before calculating the `balance_to_unbond`, we call withdraw unbonded to ensure the
			// `transferrable_balance` is correct.
			let stash_killed = bonded_pool.withdraw_from_staking(num_slashing_spans)?;

			// defensive-only: the depositor puts enough funds into the stash so that it will only
			// be destroyed when they are leaving.
			ensure!(
				!stash_killed || caller == bonded_pool.roles.depositor,
				Error::<T>::Defensive(DefensiveError::BondedStashKilledPrematurely)
			);

			let mut sum_unlocked_points: BalanceOf<T> = Zero::zero();
			let balance_to_unbond = withdrawn_points
				.iter()
				.fold(BalanceOf::<T>::zero(), |accumulator, (era, unlocked_points)| {
					sum_unlocked_points = sum_unlocked_points.saturating_add(*unlocked_points);
					if let Some(era_pool) = sub_pools.with_era.get_mut(era) {
						let balance_to_unbond = era_pool.dissolve(*unlocked_points);
						if era_pool.points.is_zero() {
							sub_pools.with_era.remove(era);
						}
						accumulator.saturating_add(balance_to_unbond)
					} else {
						// A pool does not belong to this era, so it must have been merged to the
						// era-less pool.
						accumulator.saturating_add(sub_pools.no_era.dissolve(*unlocked_points))
					}
				})
				// A call to this transaction may cause the pool's stash to get dusted. If this
				// happens before the last member has withdrawn, then all subsequent withdraws will
				// be 0. However the unbond pools do no get updated to reflect this. In the
				// aforementioned scenario, this check ensures we don't try to withdraw funds that
				// don't exist. This check is also defensive in cases where the unbond pool does not
				// update its balance (e.g. a bug in the slashing hook.) We gracefully proceed in
				// order to ensure members can leave the pool and it can be destroyed.
				.min(bonded_pool.transferable_balance());

			T::Currency::transfer(
				&bonded_pool.bonded_account(),
				&member_account,
				balance_to_unbond,
				ExistenceRequirement::AllowDeath,
			)
			.defensive()?;

			Self::deposit_event(Event::<T>::Withdrawn {
				member: member_account.clone(),
				pool_id: member.pool_id,
				points: sum_unlocked_points,
				balance: balance_to_unbond,
			});

			let post_info_weight = if member.unbonding_points().is_zero() {
				// member being reaped.
				UnbondingMembers::<T>::remove(&member_account);
				Self::deposit_event(Event::<T>::MemberRemoved {
					pool_id,
					member: member_account.clone(),
				});

				if member_account == bonded_pool.roles.depositor {
					Pallet::<T>::dissolve_pool(bonded_pool);
					None
				} else {
					SubPoolsStorage::<T>::insert(member.pool_id, sub_pools);
					Some(T::WeightInfo::withdraw_unbonded_update(num_slashing_spans))
				}
			} else {
				// we certainly don't need to delete any pools, because no one is being removed.
				SubPoolsStorage::<T>::insert(pool_id, sub_pools);
				UnbondingMembers::<T>::insert(&member_account, member);
				Some(T::WeightInfo::withdraw_unbonded_update(num_slashing_spans))
			};

			Ok(post_info_weight.into())
		}

		/// Create a new delegation pool.
		///
		/// # Arguments
		///
		/// * `amount` - The amount of funds to delegate to the pool. This also acts of a sort of
		///   deposit since the pools creator cannot fully unbond funds until the pool is being
		///   destroyed.
		/// * `index` - A disambiguation index for creating the account. Likely only useful when
		///   creating multiple pools in the same extrinsic.
		/// * `root` - The account to set as [`PoolRoles::root`].
		/// * `nominator` - The account to set as the [`PoolRoles::nominator`].
		/// * `bouncer` - The account to set as the [`PoolRoles::bouncer`].
		///
		/// # Note
		///
		/// In addition to `amount`, the caller will transfer the existential deposit; so the caller
		/// needs at have at least `amount + existential_deposit` transferable.
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::create())]
		pub fn create(
			origin: OriginFor<T>,
			#[pallet::compact] amount: BalanceOf<T>,
			root: AccountIdLookupOf<T>,
			nominator: AccountIdLookupOf<T>,
			bouncer: AccountIdLookupOf<T>,
			name: BoundedVec<u8, T::MaxNameLength>,
		) -> DispatchResult {
			let depositor = ensure_signed(origin)?;

			let pool_id = LastPoolId::<T>::try_mutate::<_, Error<T>, _>(|id| {
				*id = id.checked_add(1).ok_or(Error::<T>::OverflowRisk)?;
				Ok(*id)
			})?;

			Self::do_create(depositor, amount, root, nominator, bouncer, pool_id, name)
		}

		/// Create a new delegation pool with a previously used pool id
		///
		/// # Arguments
		///
		/// same as `create` with the inclusion of
		/// * `pool_id` - `A valid PoolId.
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::create())]
		pub fn create_with_pool_id(
			origin: OriginFor<T>,
			#[pallet::compact] amount: BalanceOf<T>,
			root: AccountIdLookupOf<T>,
			nominator: AccountIdLookupOf<T>,
			bouncer: AccountIdLookupOf<T>,
			pool_id: PoolId,
			name: BoundedVec<u8, T::MaxNameLength>,
		) -> DispatchResult {
			let depositor = ensure_signed(origin)?;

			ensure!(!BondedPools::<T>::contains_key(pool_id), Error::<T>::PoolIdInUse);
			ensure!(pool_id < LastPoolId::<T>::get(), Error::<T>::InvalidPoolId);

			Self::do_create(depositor, amount, root, nominator, bouncer, pool_id, name)
		}

		/// Nominate on behalf of the pool.
		///
		/// The dispatch origin of this call must be signed by the pool nominator or the pool
		/// root role.
		///
		/// This directly forward the call to the staking pallet, on behalf of the pool bonded
		/// account.
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::nominate(validators.len() as u32))]
		pub fn nominate(
			origin: OriginFor<T>,
			pool_id: PoolId,
			validators: Vec<T::AccountId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			ensure!(bonded_pool.can_nominate(&who), Error::<T>::NotNominator);
			T::Staking::nominate(&bonded_pool.bonded_account(), validators)
		}

		/// Set a new state for the pool.
		///
		/// If a pool is already in the `Destroying` state, then under no condition can its state
		/// change again.
		///
		/// The dispatch origin of this call must be either:
		///
		/// 1. signed by the bouncer, or the root role of the pool,
		/// 2. if the pool conditions to be open are NOT met (as described by `ok_to_be_open`), and
		///    then the state of the pool can be permissionlessly changed to `Destroying`.
		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::set_state())]
		pub fn set_state(
			origin: OriginFor<T>,
			pool_id: PoolId,
			state: PoolState,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			ensure!(bonded_pool.state != PoolState::Destroying, Error::<T>::CanNotChangeState);

			if bonded_pool.can_toggle_state(&who) {
				bonded_pool.set_state(state);
			} else if bonded_pool.ok_to_be_open().is_err() && state == PoolState::Destroying {
				// If the pool has bad properties, then anyone can set it as destroying
				bonded_pool.set_state(PoolState::Destroying);
			} else {
				Err(Error::<T>::CanNotChangeState)?;
			}

			bonded_pool.put();

			Ok(())
		}

		/// Set a new metadata for the pool.
		///
		/// The dispatch origin of this call must be signed by the bouncer, or the root role of the
		/// pool.
		#[pallet::call_index(10)]
		#[pallet::weight(T::WeightInfo::set_metadata(metadata.len() as u32))]
		pub fn set_metadata(
			origin: OriginFor<T>,
			pool_id: PoolId,
			metadata: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let metadata: BoundedVec<_, _> =
				metadata.try_into().map_err(|_| Error::<T>::MetadataExceedsMaxLen)?;
			ensure!(
				BondedPool::<T>::get(pool_id)
					.ok_or(Error::<T>::PoolNotFound)?
					.can_set_metadata(&who),
				Error::<T>::DoesNotHavePermission
			);

			Metadata::<T>::mutate(pool_id, |pool_meta| *pool_meta = metadata);

			Ok(())
		}

		/// Update configurations for the nomination pools. The origin for this call must be
		/// Root.
		///
		/// # Arguments
		///
		/// * `min_join_bond` - Set [`MinJoinBond`].
		/// * `min_create_bond` - Set [`MinCreateBond`].
		/// * `max_pools` - Set [`MaxPools`].
		/// * `max_members` - Set [`MaxPoolMembers`].
		/// * `max_members_per_pool` - Set [`MaxPoolMembersPerPool`].
		/// * `global_max_commission` - Set [`GlobalMaxCommission`].
		#[pallet::call_index(11)]
		#[pallet::weight(T::WeightInfo::set_configs())]
		pub fn set_configs(
			origin: OriginFor<T>,
			min_join_bond: ConfigOp<BalanceOf<T>>,
			min_create_bond: ConfigOp<BalanceOf<T>>,
			max_pools: ConfigOp<u32>,
			global_max_commission: ConfigOp<Perbill>,
		) -> DispatchResult {
			ensure_root(origin)?;

			macro_rules! config_op_exp {
				($storage:ty, $op:ident) => {
					match $op {
						ConfigOp::Noop => (),
						ConfigOp::Set(v) => <$storage>::put(v),
						ConfigOp::Remove => <$storage>::kill(),
					}
				};
			}

			config_op_exp!(MinJoinBond::<T>, min_join_bond);
			config_op_exp!(MinCreateBond::<T>, min_create_bond);
			config_op_exp!(MaxPools::<T>, max_pools);
			config_op_exp!(GlobalMaxCommission::<T>, global_max_commission);
			Ok(())
		}

		/// Update the roles of the pool.
		///
		/// The root is the only entity that can change any of the roles, including itself,
		/// excluding the depositor, who can never change.
		///
		/// It emits an event, notifying UIs of the role change. This event is quite relevant to
		/// most pool members and they should be informed of changes to pool roles.
		#[pallet::call_index(12)]
		#[pallet::weight(T::WeightInfo::update_roles())]
		pub fn update_roles(
			origin: OriginFor<T>,
			pool_id: PoolId,
			new_root: ConfigOp<T::AccountId>,
			new_nominator: ConfigOp<T::AccountId>,
			new_bouncer: ConfigOp<T::AccountId>,
		) -> DispatchResult {
			let mut bonded_pool = match ensure_root(origin.clone()) {
				Ok(()) => BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?,
				Err(frame_support::error::BadOrigin) => {
					let who = ensure_signed(origin)?;
					let bonded_pool =
						BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
					ensure!(bonded_pool.can_update_roles(&who), Error::<T>::DoesNotHavePermission);
					bonded_pool
				},
			};

			match new_root {
				ConfigOp::Noop => (),
				ConfigOp::Remove => bonded_pool.roles.root = None,
				ConfigOp::Set(v) => bonded_pool.roles.root = Some(v),
			};
			match new_nominator {
				ConfigOp::Noop => (),
				ConfigOp::Remove => bonded_pool.roles.nominator = None,
				ConfigOp::Set(v) => bonded_pool.roles.nominator = Some(v),
			};
			match new_bouncer {
				ConfigOp::Noop => (),
				ConfigOp::Remove => bonded_pool.roles.bouncer = None,
				ConfigOp::Set(v) => bonded_pool.roles.bouncer = Some(v),
			};

			Self::deposit_event(Event::<T>::RolesUpdated {
				root: bonded_pool.roles.root.clone(),
				nominator: bonded_pool.roles.nominator.clone(),
				bouncer: bonded_pool.roles.bouncer.clone(),
			});

			bonded_pool.put();
			Ok(())
		}

		/// Chill on behalf of the pool.
		///
		/// The dispatch origin of this call must be signed by the pool nominator or the pool
		/// root role, same as [`Pallet::nominate`].
		///
		/// This directly forward the call to the staking pallet, on behalf of the pool bonded
		/// account.
		#[pallet::call_index(13)]
		#[pallet::weight(T::WeightInfo::chill())]
		pub fn chill(origin: OriginFor<T>, pool_id: PoolId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			ensure!(bonded_pool.can_nominate(&who), Error::<T>::NotNominator);
			T::Staking::chill(&bonded_pool.bonded_account())
		}

		/// `origin` bonds funds from `extra` for some pool member `member` into their respective
		/// pools.
		///
		/// `origin` can bond extra funds from free balance or pending rewards when `origin ==
		/// other`.
		///
		/// In the case of `origin != other`, `origin` can only bond extra pending rewards of
		/// `other` members assuming set_claim_permission for the given member is
		/// `PermissionlessAll` or `PermissionlessCompound`.
		#[pallet::call_index(14)]
		#[pallet::weight(
			T::WeightInfo::bond_extra_transfer()
			.max(T::WeightInfo::bond_extra_other())
		)]
		pub fn bond_extra_other(
			origin: OriginFor<T>,
			member: AccountIdLookupOf<T>,
			pool_id: PoolId,
			extra: BondExtra<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_bond_extra(who, T::Lookup::lookup(member)?, pool_id, extra)
		}

		/// Set the commission of a pool.
		//
		/// Both a commission percentage and a commission payee must be provided in the `current`
		/// tuple. Where a `current` of `None` is provided, any current commission will be removed.
		///
		/// - If a `None` is supplied to `new_commission`, existing commission will be removed.
		#[pallet::call_index(17)]
		#[pallet::weight(T::WeightInfo::set_commission())]
		pub fn set_commission(
			origin: OriginFor<T>,
			pool_id: PoolId,
			new_commission: Option<(Perbill, T::AccountId)>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			ensure!(bonded_pool.can_manage_commission(&who), Error::<T>::DoesNotHavePermission);

			let mut reward_pool = RewardPools::<T>::get(pool_id)
				.defensive_ok_or::<Error<T>>(DefensiveError::RewardPoolNotFound.into())?;
			// IMPORTANT: make sure that everything up to this point is using the current commission
			// before it updates. Note that `try_update_current` could still fail at this point.
			reward_pool.update_records(
				pool_id,
				bonded_pool.points(),
				bonded_pool.commission.current(),
			)?;
			RewardPools::insert(pool_id, reward_pool);

			bonded_pool.commission.try_update_current(&new_commission)?;
			bonded_pool.put();
			Self::deposit_event(Event::<T>::PoolCommissionUpdated {
				pool_id,
				current: new_commission,
			});
			Ok(())
		}

		/// Set the maximum commission of a pool.
		///
		/// - Initial max can be set to any `Perbill`, and only smaller values thereafter.
		/// - Current commission will be lowered in the event it is higher than a new max
		///   commission.
		#[pallet::call_index(18)]
		#[pallet::weight(T::WeightInfo::set_commission_max())]
		pub fn set_commission_max(
			origin: OriginFor<T>,
			pool_id: PoolId,
			max_commission: Perbill,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			ensure!(bonded_pool.can_manage_commission(&who), Error::<T>::DoesNotHavePermission);

			bonded_pool.commission.try_update_max(pool_id, max_commission)?;
			bonded_pool.put();

			Self::deposit_event(Event::<T>::PoolMaxCommissionUpdated { pool_id, max_commission });
			Ok(())
		}

		/// Set the commission change rate for a pool.
		///
		/// Initial change rate is not bounded, whereas subsequent updates can only be more
		/// restrictive than the current.
		#[pallet::call_index(19)]
		#[pallet::weight(T::WeightInfo::set_commission_change_rate())]
		pub fn set_commission_change_rate(
			origin: OriginFor<T>,
			pool_id: PoolId,
			change_rate: CommissionChangeRate<BlockNumberFor<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			ensure!(bonded_pool.can_manage_commission(&who), Error::<T>::DoesNotHavePermission);

			bonded_pool.commission.try_update_change_rate(change_rate)?;
			bonded_pool.put();

			Self::deposit_event(Event::<T>::PoolCommissionChangeRateUpdated {
				pool_id,
				change_rate,
			});
			Ok(())
		}

		/// Claim pending commission.
		///
		/// The dispatch origin of this call must be signed by the `root` role of the pool. Pending
		/// commission is paid out and added to total claimed commission`. Total pending commission
		/// is reset to zero. the current.
		#[pallet::call_index(20)]
		#[pallet::weight(T::WeightInfo::claim_commission())]
		pub fn claim_commission(origin: OriginFor<T>, pool_id: PoolId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_claim_commission(who, pool_id)
		}

		/// Top up the deficit or withdraw the excess ED from the pool.
		///
		/// When a pool is created, the pool depositor transfers ED to the reward account of the
		/// pool. ED is subject to change and over time, the deposit in the reward account may be
		/// insufficient to cover the ED deficit of the pool or vice-versa where there is excess
		/// deposit to the pool. This call allows anyone to adjust the ED deposit of the
		/// pool by either topping up the deficit or claiming the excess.
		#[pallet::call_index(21)]
		#[pallet::weight(T::WeightInfo::adjust_pool_deposit())]
		pub fn adjust_pool_deposit(origin: OriginFor<T>, pool_id: PoolId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_adjust_pool_deposit(who, pool_id)
		}

		/// Set or remove a pool's commission claim permission.
		///
		/// Determines who can claim the pool's pending commission. Only the `Root` role of the pool
		/// is able to conifigure commission claim permissions.
		#[pallet::call_index(22)]
		#[pallet::weight(T::WeightInfo::set_commission_claim_permission())]
		pub fn set_commission_claim_permission(
			origin: OriginFor<T>,
			pool_id: PoolId,
			permission: Option<CommissionClaimPermission<T::AccountId>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			ensure!(bonded_pool.can_manage_commission(&who), Error::<T>::DoesNotHavePermission);

			bonded_pool.commission.claim_permission.clone_from(&permission);
			bonded_pool.put();

			Self::deposit_event(Event::<T>::PoolCommissionClaimPermissionUpdated {
				pool_id,
				permission,
			});

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		#[cfg(feature = "try-runtime")]
		fn try_state(_n: BlockNumberFor<T>) -> Result<(), TryRuntimeError> {
			Self::do_try_state(u8::MAX)
		}

		fn integrity_test() {
			assert!(
				T::MaxPointsToBalance::get() > 0,
				"Minimum points to balance ratio must be greater than 0"
			);
			assert!(
				T::Staking::bonding_duration() < TotalUnbondingPools::<T>::get(),
				"There must be more unbonding pools then the bonding duration /
				so a slash can be applied to relevant unboding pools. (We assume /
				the bonding duration > slash deffer duration.",
			);
		}
	}
}

impl<T: Config> Pallet<T> {
	/// The amount of bond that MUST REMAIN IN BONDED in ALL POOLS.
	///
	/// It is the responsibility of the depositor to put these funds into the pool initially. Upon
	/// unbond, they can never unbond to a value below this amount.
	///
	/// It is essentially `max { MinNominatorBond, MinCreateBond, MinJoinBond }`, where the former
	/// is coming from the staking pallet and the latter two are configured in this pallet.
	pub fn depositor_min_bond() -> BalanceOf<T> {
		T::Staking::minimum_nominator_bond()
			.max(MinCreateBond::<T>::get())
			.max(MinJoinBond::<T>::get())
			.max(T::Currency::minimum_balance())
	}
	/// Remove everything related to the given bonded pool.
	///
	/// Metadata and all of the sub-pools are also deleted. All accounts are dusted and the leftover
	/// of the reward account is returned to the depositor.
	pub fn dissolve_pool(bonded_pool: BondedPool<T>) {
		let reward_account = bonded_pool.reward_account();
		let bonded_account = bonded_pool.bonded_account();

		ReversePoolIdLookup::<T>::remove(&bonded_account);
		RewardPools::<T>::remove(bonded_pool.id);
		SubPoolsStorage::<T>::remove(bonded_pool.id);

		// remove the ED restriction from the pool reward account.
		let _ = Self::unfreeze_pool_deposit(&bonded_pool.reward_account()).defensive();

		// Kill accounts from storage by making their balance go below ED. We assume that the
		// accounts have no references that would prevent destruction once we get to this point. We
		// don't work with the system pallet directly, but
		// 1. we drain the reward account and kill it. This account should never have any extra
		// consumers anyway.
		// 2. the bonded account should become a 'killed stash' in the staking system, and all of
		//    its consumers removed.
		defensive_assert!(
			frame_system::Pallet::<T>::consumers(&reward_account) == 0,
			"reward account of dissolving pool should have no consumers"
		);
		defensive_assert!(
			frame_system::Pallet::<T>::consumers(&bonded_account) == 0,
			"bonded account of dissolving pool should have no consumers"
		);
		defensive_assert!(
			T::Staking::total_stake(&bonded_account).unwrap_or_default() == Zero::zero(),
			"dissolving pool should not have any stake in the staking pallet"
		);

		// This shouldn't fail, but if it does we don't really care. Remaining balance can consist
		// of unclaimed pending commission, erroneous transfers to the reward account, etc.
		let reward_pool_remaining = T::Currency::free_balance(&reward_account);

		let _ = T::Currency::transfer(
			&reward_account,
			&bonded_pool.roles.depositor,
			reward_pool_remaining,
			ExistenceRequirement::AllowDeath,
		);

		defensive_assert!(
			T::Currency::total_balance(&reward_account) == Zero::zero(),
			"could not transfer all amount to depositor while dissolving pool"
		);
		defensive_assert!(
			T::Currency::total_balance(&bonded_pool.bonded_account()) == Zero::zero(),
			"dissolving pool should not have any balance"
		);
		// NOTE: Defensively force set balance to zero.
		T::Currency::make_free_balance_be(&reward_account, Zero::zero());
		T::Currency::make_free_balance_be(&bonded_pool.bonded_account(), Zero::zero());

		Self::deposit_event(Event::<T>::Destroyed { pool_id: bonded_pool.id });
		// Remove bonded pool metadata.
		Metadata::<T>::remove(bonded_pool.id);

		bonded_pool.remove();
	}

	/// Create the main, bonded account of a pool with the given id.
	pub fn create_bonded_account(id: PoolId) -> T::AccountId {
		T::PalletId::get().into_sub_account_truncating((AccountType::Bonded, id))
	}

	/// Create the reward account of a pool with the given id.
	pub fn create_reward_account(id: PoolId) -> T::AccountId {
		// NOTE: in order to have a distinction in the test account id type (u128), we put
		// account_type first so it does not get truncated out.
		T::PalletId::get().into_sub_account_truncating((AccountType::Reward, id))
	}

	/// Calculate the equivalent point of `new_funds` in a pool with `current_balance` and
	/// `current_points`.
	fn balance_to_point(
		current_balance: BalanceOf<T>,
		current_points: BalanceOf<T>,
		new_funds: BalanceOf<T>,
	) -> BalanceOf<T> {
		let u256 = T::BalanceToU256::convert;
		let balance = T::U256ToBalance::convert;
		match (current_balance.is_zero(), current_points.is_zero()) {
			(_, true) => new_funds.saturating_mul(POINTS_TO_BALANCE_INIT_RATIO.into()),
			(true, false) => {
				// The pool was totally slashed.
				// This is the equivalent of `(current_points / 1) * new_funds`.
				new_funds.saturating_mul(current_points)
			},
			(false, false) => {
				// Equivalent to (current_points / current_balance) * new_funds
				balance(
					u256(current_points)
						.saturating_mul(u256(new_funds))
						// We check for zero above
						.div(u256(current_balance)),
				)
			},
		}
	}

	/// Calculate the equivalent balance of `points` in a pool with `current_balance` and
	/// `current_points`.
	fn point_to_balance(
		current_balance: BalanceOf<T>,
		current_points: BalanceOf<T>,
		points: BalanceOf<T>,
	) -> BalanceOf<T> {
		let u256 = T::BalanceToU256::convert;
		let balance = T::U256ToBalance::convert;
		if current_balance.is_zero() || current_points.is_zero() || points.is_zero() {
			// There is nothing to unbond
			return Zero::zero();
		}

		// Equivalent of (current_balance / current_points) * points
		balance(
			u256(current_balance)
				.saturating_mul(u256(points))
				// We check for zero above
				.div(u256(current_points)),
		)
	}

	fn do_create(
		who: T::AccountId,
		amount: BalanceOf<T>,
		root: AccountIdLookupOf<T>,
		nominator: AccountIdLookupOf<T>,
		bouncer: AccountIdLookupOf<T>,
		pool_id: PoolId,
		name: BoundedVec<u8, T::MaxNameLength>,
	) -> DispatchResult {
		let root = T::Lookup::lookup(root)?;
		let nominator = T::Lookup::lookup(nominator)?;
		let bouncer = T::Lookup::lookup(bouncer)?;

		// ensure that pool token can be created
		// if this fails, it means that the pool token already exists or the token counter needs to be incremented correctly
		ensure!(
			T::Fungibles::total_issuance(pool_id.into()) == 0_u32.into(),
			Error::<T>::PoolTokenCreationFailed
		);

		let admin_account = T::PalletId::get().into_account_truncating();
		T::Fungibles::create(pool_id.into(), admin_account, false, 1_u32.into())?;

		ensure!(amount >= Pallet::<T>::depositor_min_bond(), Error::<T>::MinimumBondNotMet);
		ensure!(
			MaxPools::<T>::get().map_or(true, |max_pools| BondedPools::<T>::count() < max_pools),
			Error::<T>::MaxPools
		);
		let mut bonded_pool = BondedPool::<T>::new(
			pool_id,
			PoolRoles {
				root: Some(root),
				nominator: Some(nominator),
				bouncer: Some(bouncer),
				depositor: who.clone(),
			},
			name,
		);

		bonded_pool.try_bond_funds(&who, amount, BondType::Create)?;

		// Transfer the minimum balance for the reward account.
		T::Currency::transfer(
			&who,
			&bonded_pool.reward_account(),
			T::Currency::minimum_balance(),
			ExistenceRequirement::KeepAlive,
		)?;

		RewardPools::<T>::insert(
			pool_id,
			RewardPool::<T> {
				last_recorded_reward_counter: Zero::zero(),
				last_recorded_total_payouts: Zero::zero(),
				total_rewards_claimed: Zero::zero(),
				total_commission_pending: Zero::zero(),
				total_commission_claimed: Zero::zero(),
			},
		);
		ReversePoolIdLookup::<T>::insert(bonded_pool.bonded_account(), pool_id);

		Self::deposit_event(Event::<T>::Created { depositor: who.clone(), pool_id });

		Self::deposit_event(Event::<T>::Bonded {
			member: who,
			pool_id,
			bonded: amount,
			joined: true,
		});

		bonded_pool.put();

		Ok(())
	}

	fn do_bond_extra(
		signer: T::AccountId,
		member_account: T::AccountId,
		pool_id: PoolId,
		extra: BondExtra<BalanceOf<T>>,
	) -> DispatchResult {
		if signer != member_account {
			ensure!(
				ClaimPermissions::<T>::get(&member_account).can_bond_extra(),
				Error::<T>::DoesNotHavePermission
			);
		}

		let mut bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
		bonded_pool.ok_to_join()?;

		let (_points_issued, bonded) = match extra {
			BondExtra::FreeBalance(amount) => {
				(bonded_pool.try_bond_funds(&member_account, amount, BondType::Later)?, amount)
			},
		};

		bonded_pool.ok_to_be_open()?;

		Self::deposit_event(Event::<T>::Bonded {
			member: member_account.clone(),
			pool_id,
			bonded,
			joined: false,
		});

		Ok(())
	}

	fn do_claim_commission(who: T::AccountId, pool_id: PoolId) -> DispatchResult {
		let bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
		ensure!(bonded_pool.can_claim_commission(&who), Error::<T>::DoesNotHavePermission);

		let mut reward_pool = RewardPools::<T>::get(pool_id)
			.defensive_ok_or::<Error<T>>(DefensiveError::RewardPoolNotFound.into())?;

		// IMPORTANT: ensure newly pending commission not yet processed is added to
		// `total_commission_pending`.
		reward_pool.update_records(
			pool_id,
			bonded_pool.points(),
			bonded_pool.commission.current(),
		)?;

		let commission = reward_pool.total_commission_pending;
		ensure!(!commission.is_zero(), Error::<T>::NoPendingCommission);

		let payee = bonded_pool
			.commission
			.current
			.as_ref()
			.map(|(_, p)| p.clone())
			.ok_or(Error::<T>::NoCommissionCurrentSet)?;

		// Payout claimed commission.
		T::Currency::transfer(
			&bonded_pool.reward_account(),
			&payee,
			commission,
			ExistenceRequirement::KeepAlive,
		)?;

		// Add pending commission to total claimed counter.
		reward_pool.total_commission_claimed =
			reward_pool.total_commission_claimed.saturating_add(commission);
		// Reset total pending commission counter to zero.
		reward_pool.total_commission_pending = Zero::zero();
		RewardPools::<T>::insert(pool_id, reward_pool);

		Self::deposit_event(Event::<T>::PoolCommissionClaimed { pool_id, commission });
		Ok(())
	}

	fn do_adjust_pool_deposit(who: T::AccountId, pool: PoolId) -> DispatchResult {
		let bonded_pool = BondedPool::<T>::get(pool).ok_or(Error::<T>::PoolNotFound)?;
		let reward_acc = &bonded_pool.reward_account();
		let pre_frozen_balance = T::Currency::reserved_balance(reward_acc);
		let min_balance = T::Currency::minimum_balance();

		if pre_frozen_balance == min_balance {
			return Err(Error::<T>::NothingToAdjust.into());
		}

		// Update frozen amount with current ED.
		Self::freeze_pool_deposit(reward_acc)?;

		if pre_frozen_balance > min_balance {
			// Transfer excess back to depositor.
			let excess = pre_frozen_balance.saturating_sub(min_balance);
			T::Currency::transfer(reward_acc, &who, excess, ExistenceRequirement::KeepAlive)?;
			Self::deposit_event(Event::<T>::MinBalanceExcessAdjusted {
				pool_id: pool,
				amount: excess,
			});
		} else {
			// Transfer ED deficit from depositor to the pool
			let deficit = min_balance.saturating_sub(pre_frozen_balance);
			T::Currency::transfer(&who, reward_acc, deficit, ExistenceRequirement::KeepAlive)?;
			Self::deposit_event(Event::<T>::MinBalanceDeficitAdjusted {
				pool_id: pool,
				amount: deficit,
			});
		}

		Ok(())
	}

	/// Apply freeze on reward account to restrict it from going below ED.
	pub(crate) fn freeze_pool_deposit(reward_acc: &T::AccountId) -> DispatchResult {
		T::Currency::reserve(reward_acc, T::Currency::minimum_balance())
	}

	/// Removes the ED freeze on the reward account of `pool_id`.
	pub fn unfreeze_pool_deposit(reward_acc: &T::AccountId) -> DispatchResult {
		let _ = T::Currency::unreserve(reward_acc, T::Currency::minimum_balance());
		Ok(())
	}

	/// Fully unbond the shares of `member`, when executed from `origin`.
	///
	/// This is useful for backwards compatibility with the majority of tests that only deal with
	/// full unbonding, not partial unbonding.
	#[cfg(any(feature = "runtime-benchmarks", test))]
	pub fn fully_unbond(
		origin: frame_system::pallet_prelude::OriginFor<T>,
		member: T::AccountId,
		pool_id: PoolId,
	) -> DispatchResult {
		let points = T::Fungibles::balance(pool_id.into(), &member);
		let member_lookup = T::Lookup::unlookup(member);
		Self::unbond(origin, member_lookup, pool_id, points)
	}
}

impl<T: Config> sp_staking::OnStakingUpdate<T::AccountId, BalanceOf<T>> for Pallet<T> {
	/// Reduces the balances of the [`SubPools`], that belong to the pool involved in the
	/// slash, to the amount that is defined in the `slashed_unlocking` field of
	/// [`sp_staking::OnStakingUpdate::on_slash`]
	///
	/// Emits the `PoolsSlashed` event.
	fn on_slash(
		pool_account: &T::AccountId,
		// Bonded balance is always read directly from staking, therefore we don't need to update
		// anything here.
		slashed_bonded: BalanceOf<T>,
		slashed_unlocking: &BTreeMap<EraIndex, BalanceOf<T>>,
		total_slashed: BalanceOf<T>,
	) {
		let Some(pool_id) = ReversePoolIdLookup::<T>::get(pool_account) else { return };
		// As the slashed account belongs to a `BondedPool` the `TotalValueLocked` decreases and
		// an event is emitted.
		TotalValueLocked::<T>::mutate(|tvl| {
			tvl.defensive_saturating_reduce(total_slashed);
		});

		if let Some(mut sub_pools) = SubPoolsStorage::<T>::get(pool_id) {
			// set the reduced balance for each of the `SubPools`
			slashed_unlocking.iter().for_each(|(era, slashed_balance)| {
				if let Some(pool) = sub_pools.with_era.get_mut(era).defensive() {
					pool.balance = *slashed_balance;
					Self::deposit_event(Event::<T>::UnbondingPoolSlashed {
						era: *era,
						pool_id,
						balance: *slashed_balance,
					});
				}
			});
			SubPoolsStorage::<T>::insert(pool_id, sub_pools);
		} else if !slashed_unlocking.is_empty() {
			defensive!("Expected SubPools were not found");
		}
		Self::deposit_event(Event::<T>::PoolSlashed { pool_id, balance: slashed_bonded });
	}
}
