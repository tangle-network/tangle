//! # Nomination Pools for Staking Delegation
//!
//! A pallet that allows members to delegate their stake to nomination pools. A nomination pool acts
//! as nominator and nominates validators on the members' behalf.
//!
//! # Index
//!
//! * [Key terms](#key-terms)
//! * [Usage](#usage)
//! * [Implementor's Guide](#implementors-guide)
//! * [Design](#design)
//!
//! ## Key Terms
//!
//!  * pool id: A unique identifier of each pool. Set to u32.
//!  * bonded pool: Tracks the distribution of actively staked funds. See [`BondedPool`] and
//! [`BondedPoolInner`].
//! * unbonding sub pools: Collection of pools at different phases of the unbonding lifecycle. See
//!   [`SubPools`] and [`SubPoolsStorage`].
//! * members: Accounts that are members of pools. See [`PoolMember`] and [`UnbondingMembers`].
//! * roles: Administrative roles of each pool, capable of controlling nomination, and the state of
//!   the pool.
//! * point: A unit of measure for a members portion of a pool's funds. Points initially have a
//!   ratio of 1 (as set by `POINTS_TO_BALANCE_INIT_RATIO`) to balance, but as slashing happens,
//!   this can change. A point is equivalent to 1 LST.
//! * kick: The act of a pool administrator forcibly ejecting a member.
//! * bonded account: A key-less account id derived from the pool id that acts as the bonded
//!   account. This account registers itself as a nominator in the staking system, and follows
//!   exactly the same rules and conditions as a normal staker. Its bond increases or decreases as
//!   members join, it can `nominate` or `chill`, and might not even earn staking rewards if it is
//!   not nominating proper validators.
//! * reward account: A similar key-less account, that is set as the `Payee` account for the bonded
//!   account for all staking rewards.
//! * LST - the liquid token that represents staked TNT. It is equivalent to 1 point.
//!
//!
//! ## Usage
//!
//! ### Join
//!
//! An account can stake funds with a nomination pool by calling [`Call::bond`].
//!
//!
//! For design docs see the [reward pool](#reward-pool) section.
//!
//! ### Leave
//!
//! In order to leave, a member must take two steps.
//!
//! First, they must call [`Call::unbond`]. The unbond extrinsic will start the unbonding process by
//! unbonding all or a portion of the members funds.
//!
//! > A member can have up to [`Config::MaxUnbonding`] distinct active unbonding requests.
//!
//! Second, once [`sp_staking::StakingInterface::bonding_duration`] eras have passed, the member can
//! call [`Call::withdraw_unbonded`] to withdraw any funds that are free.
//!
//! For design docs see the [bonded pool](#bonded-pool) and [unbonding sub
//! pools](#unbonding-sub-pools) sections.
//!
//! ### Slashes
//!
//! Slashes are distributed evenly across the bonded pool and the unbonding pools from slash era+1
//! through the slash apply era. Thus, any member who either
//!
//! 1. unbonded, or
//! 2. was actively bonded
//
//! in the aforementioned range of eras will be affected by the slash. A member is slashed pro-rata
//! based on its stake relative to the total slash amount.
//!
//! Slashing does not change any single member's balance. Instead, the slash will only reduce the
//! balance associated with a particular pool. But, we never change the total *points* of a pool
//! because of slashing. Therefore, when a slash happens, the ratio of points to balance changes in
//! a pool. In other words, the value of one point, which is initially 1-to-1 against a unit of
//! balance, is now less than one balance because of the slash.
//!
//! ### Administration
//!
//! A pool can be created with the [`Call::create`] call. To create a pool, it is required to hold a
//! token from the [`Config::PoolCollectionId`] collection. Once created, the pool's nominator or
//! admin user must call [`Call::nominate`] to start nominating. [`Call::nominate`] can be called at
//! anytime to update validator selection.
//!
//! Similar to [`Call::nominate`], [`Call::chill`] will chill to pool in the staking system, and
//! [`Call::pool_withdraw_unbonded`] will withdraw any unbonding chunks of the pool bonded account.
//! The latter call is permissionless and can be called by anyone at any time.
//!
//! To help facilitate pool administration the pool has one of three states (see [`PoolState`]):
//!
//! * Open: Anyone can join the pool and no members can be permissionlessly removed.
//! * Blocked: No members can join and some admin roles can kick members. Kicking is not instant,
//!   and follows the same process of `unbond` and then `withdraw_unbonded`. In other words,
//!   administrators can permissionlessly unbond other members.
//! * Destroying: No members can join and all members can be permissionlessly removed with
//!   [`Call::unbond`] and [`Call::withdraw_unbonded`]. Once a pool is in destroying state, it
//!   cannot be reverted to another state.
//!
//! ## Commission
//!
//! A pool can optionally have a commission configuration, via the `admin` role, set with
//! [`Call::mutate`]. The holder of the pool's NFT token will receive the commission. Beyond the
//! commission itself, a pool can have a maximum commission and a change rate.
//!
//! Importantly, both max commission and change rate can not be removed once set, and can only be
//! set to more restrictive values (i.e. a lower max commission or a slower change rate) in
//! subsequent updates.
//!
//! If set, a pool's commission is bound to [`GlobalMaxCommission`] at the time it is applied to
//! pending rewards. [`GlobalMaxCommission`] is intended to be updated only via governance.
//!
//! When a pool is dissolved, any outstanding pending commission that has not been claimed will be
//! transferred to the depositor.
//!
//! Implementation note: Commission is analogous to a separate member account of the pool, with its
//! own reward counter in the form of `current_pending_commission`.
//!
//! Crucially, commission is applied to rewards based on the current commission in effect at the
//! time rewards are transferred into the reward pool. This is to prevent the malicious behaviour of
//! changing the commission rate to a very high value after rewards are accumulated, and thus claim
//! an unexpectedly high chunk of the reward.
//!
//! ### Dismantling
//!
//! As noted, a pool is destroyed once
//!
//! 1. First, all members need to fully unbond and withdraw. If the pool state is set to
//!    `Destroying`, this can happen permissionlessly.
//! 2. The deposit is unbonded and withdrawn by calling [`Call::unbond_deposit`] and
//!    [`Call::withdraw_deposit`].
//!
//! > Note that at this point, based on the requirements of the staking system, the pool's bonded
//! > account's stake might not be able to ge below a certain threshold as a nominator. At this
//! > point, the pool should `chill` itself to allow the last member to leave. See [`Call::chill`].
//!
//! ## Implementor's Guide
//!
//! Some notes and common mistakes that wallets/apps wishing to implement this pallet should be
//! aware of:
//!
//!
//! ### Pool Members
//!
//! * Joining a pool implies transferring funds to the pool account. So it might be (based on which
//!   wallet that you are using) that you no longer see the funds that are moved to the pool in your
//!   “free balance” section. Make sure the user is aware of this, and not surprised by seeing this.
//!   Also, the transfer that happens here is configured to to never accidentally destroy the sender
//!   account. So to join a Pool, your sender account must remain alive with 0.1 TNT left in it.
//!   This means, with 0.1 TNT as existential deposit, and 1 TNT as minimum to join a pool, you need
//!   at least 1.1 TNT to join a pool (plus transaction fees). Consequently, if you are suggesting
//!   members to join a pool with “Maximum possible value”, you must subtract 0.1 TNT to remain in
//!   the sender account to not accidentally kill it.
//! * Points and balance are not the same! Any pool member, at any point in time, can have points in
//!   either the bonded pool or any of the unbonding pools. The crucial fact is that in any of these
//!   pools, the ratio of point to balance is different and might not be 1. Each pool starts with a
//!   ratio of 1, but as time goes on, for reasons such as slashing or paying out rewards, the ratio
//!   gets broken. Over time, 100 points in a bonded pool can be worth 90 TNT. Make sure you are
//!   either representing points as points (not as TNT), or even better, always display both: “You
//!   have x points in pool y which is worth z TNT. See here and here for examples of how to
//!   calculate point to balance ratio of each pool (it is almost trivial ;))
//!
//! ### Pool Management
//!
//! * The pool will be seen from the perspective of the rest of the system as a single nominator.
//!   Ergo, This nominator must always respect the `staking.minNominatorBond` limit. Similar to a
//!   normal nominator, who has to first `chill` before fully unbonding, the pool must also do the
//!   same. The pool’s bonded account will be fully unbonded only when the last member wants to
//!   leave and dismantle the pool.
//!
//! ## Design
//!
//! _Notes_: this section uses pseudo code to explain general design and does not necessarily
//! reflect the exact implementation. Additionally, a working knowledge of `pallet-staking`'s api is
//! assumed.
//!
//! ### Goals
//!
//! * Maintain network security by upholding integrity of slashing events, sufficiently penalizing
//!   members that where in the pool while it was backing a validator that got slashed.
//! * Maximize scalability in terms of member count.
//!
//! In order to maintain scalability, all operations are independent of the number of members. To do
//! this, delegation specific information is stored local to the member while the pool data
//! structures have bounded datum.
//!
//! ### Bonded pool
//!
//! A bonded pool nominates with its total balance, excluding that which has been withdrawn for
//! unbonding. The total points of a bonded pool are always equal to the sum of points of the
//! delegation members. A bonded pool tracks its points and reads its bonded balance.
//!
//! When a member joins a pool, `amount_transferred` is transferred from the members account to the
//! bonded pools account. Then the pool calls `staking::bond_extra(amount_transferred)` and issues
//! new points which are tracked by the member and added to the bonded pool's points.
//!
//! When the pool already has some balance, we want the value of a point before the transfer to
//! equal the value of a point after the transfer. So, when a member joins a bonded pool with a
//! given `amount_transferred`, we maintain the ratio of bonded balance to points such that:
//!
//! ```text
//! balance_after_transfer / points_after_transfer == balance_before_transfer / points_before_transfer;
//! ```
//!
//! To achieve this, we issue points based on the following:
//!
//! ```text
//! points_issued = (points_before_transfer / balance_before_transfer) * amount_transferred;
//! ```
//!
//! For new bonded pools we can set the points issued per balance arbitrarily. In this
//! implementation we use a 1 points to 1 balance ratio for pool creation (see
//! [`POINTS_TO_BALANCE_INIT_RATIO`]).
//!
//! **Relevant extrinsics:**
//!
//! * [`Call::create`]
//! * [`Call::bond`]
//!
//! ### Reward and bonus accounts
//!
//! When a pool is first bonded it sets up two deterministic, inaccessible accounts: one is the
//! reward destination and the other holds the bonus rewards.
//!
//! See [this link](https://hackmd.io/PFGn6wI5TbCmBYoEA_f2Uw) for an in-depth explanation of the
//! reward pool mechanism.
//!
//! **Relevant extrinsics:**
//!
//!
//! ### Unbonding sub pools
//!
//! When a member unbonds, it's balance is unbonded in the bonded pool's account and tracked in
//! an unbonding pool associated with the active era. If no such pool exists, one is created. To
//! track which unbonding sub pool a member belongs too, a member tracks it's
//! `unbonding_era`.
//!
//! When a member initiates unbonding it's claim on the bonded pool
//! (`balance_to_unbond`) is computed as:
//!
//! ```text
//! balance_to_unbond = (bonded_pool.balance / bonded_pool.points) * member.points;
//! ```
//!
//! If this is the first transfer into an unbonding pool arbitrary amount of points can be issued
//! per balance. In this implementation unbonding pools are initialized with a 1 point to 1 balance
//! ratio (see [`POINTS_TO_BALANCE_INIT_RATIO`]). Otherwise, the unbonding pools hold the same
//! points to balance ratio properties as the bonded pool, so member points in the
//! unbonding pool are issued based on
//!
//! ```text
//! new_points_issued = (points_before_transfer / balance_before_transfer) * balance_to_unbond;
//! ```
//!
//! For scalability, a bound is maintained on the number of unbonding sub pools (see
//! [`TotalUnbondingPools`]). An unbonding pool is removed once its older than `current_era -
//! TotalUnbondingPools`. An unbonding pool is merged into the unbonded pool with
//!
//! ```text
//! unbounded_pool.balance = unbounded_pool.balance + unbonding_pool.balance;
//! unbounded_pool.points = unbounded_pool.points + unbonding_pool.points;
//! ```
//!
//! This scheme "averages" out the points value in the unbonded pool.
//!
//! Once a members `unbonding_era` is older than `current_era -
//! [sp_staking::StakingInterface::bonding_duration]`, it can can cash it's points out of the
//! corresponding unbonding pool. If it's `unbonding_era` is older than `current_era -
//! TotalUnbondingPools`, it can cash it's points from the unbonded pool.
//!
//! **Relevant extrinsics:**
//!
//! * [`Call::unbond`]
//! * [`Call::withdraw_unbonded`]
//!
//! ### Slashing
//!
//! This section assumes that the slash computation is executed by
//! `pallet_staking::StakingLedger::slash`, which passes the information to this pallet via
//! [`sp_staking::OnStakingUpdate::on_slash`].
//!
//! Unbonding pools need to be slashed to ensure all nominators whom where in the bonded pool
//! while it was backing a validator that equivocated are punished. Without these measures a
//! member could unbond right after a validator equivocated with no consequences.
//!
//! This strategy is unfair to members who joined after the slash, because they get slashed as
//! well, but spares members who unbond. The latter is much more important for security: if a
//! pool's validators are attacking the network, their members need to unbond fast! Avoiding
//! slashes gives them an incentive to do that if validators get repeatedly slashed.
//!
//! To be fair to joiners, this implementation also need joining pools, which are actively staking,
//! in addition to the unbonding pools. For maintenance simplicity these are not implemented.
//! Related: <https://github.com/paritytech/substrate/issues/10860>
//!
//!
//! ### Limitations
//!
//! * PoolMembers cannot quickly transfer to another pool if they do no like nominations, instead
//!   they must wait for the unbonding duration.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	defensive, ensure,
	pallet_prelude::{MaxEncodedLen, *},
	storage::bounded_btree_map::BoundedBTreeMap,
	traits::{
		BuildGenesisConfig, Currency, Defensive, DefensiveOption, DefensiveResult,
		ExistenceRequirement, Get,
	},
	DefaultNoBound,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use parity_scale_codec::{Codec, FullCodec};
use scale_info::TypeInfo;
use sp_core::U256;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;
use sp_runtime::{
	traits::{AccountIdConversion, Convert, Saturating, StaticLookup, Zero},
	FixedPointNumber, Perbill,
};
use sp_staking::{EraIndex, StakingInterface};
use sp_std::{fmt::Debug, ops::Div, vec::Vec};
pub use types::*;

mod functions;
mod impls;
#[cfg(any(test, feature = "fuzzing"))]
#[rustfmt::skip]
pub mod mock;
#[cfg(test)]
mod tests;
mod types;

pub mod weights;

pub use pallet::*;
use sp_runtime::SaturatedConversion;
pub use weights::WeightInfo;
/// Type used for unique identifier of each pool.
pub type PoolId = u32;

/// Explicit lookup source
type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

/// The initial points to balance ratio for new pools
pub const POINTS_TO_BALANCE_INIT_RATIO: u32 = 1;

pub const LOG_TARGET: &str = "nomination_pools";

#[allow(clippy::missing_docs_in_private_items)]
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::traits::StorageVersion;
	use sp_runtime::Perquintill;

	/// The current storage version.
	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(9);

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
		type Currency: Mutate<Self::AccountId>
			+ MutateFreeze<Self::AccountId, Id = Self::RuntimeFreezeReason>;

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

		/// The LST collection id.
		type LstCollectionId: Get<AssetId>;
	}

	/// Minimum amount to bond to join a pool.
	#[pallet::storage]
	pub type MinJoinBond<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// Minimum bond required to create a pool.
	///
	/// This is the amount that the pool creator must put as their initial stake in the pool, as an
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

	/// Maximum number of members that can exist in the system. If `None`, then the count
	/// members are not bound on a system wide basis.
	#[pallet::storage]
	pub type MaxPoolMembers<T: Config> = StorageValue<_, u32, OptionQuery>;

	/// Maximum number of members that may belong to pool. If `None`, then the count of
	/// members is not bound on a per pool basis.
	#[pallet::storage]
	pub type MaxPoolMembersPerPool<T: Config> = StorageValue<_, u32, OptionQuery>;

	/// Pool Members who are Unbonding.
	///
	/// TWOX-NOTE: SAFE since `AccountId` is a secure hash.
	#[pallet::storage]
	pub type UnbondingMembers<T: Config> =
		StorageDoubleMap<_, Twox64Concat, PoolId, Twox64Concat, T::AccountId, PoolMember<T>>;

	/// Storage for bonded pools.
	// To get or insert a pool see [`BondedPool::get`] and [`BondedPool::put`]
	#[pallet::storage]
	pub type BondedPools<T: Config> =
		CountedStorageMap<_, Twox64Concat, PoolId, BondedPoolInner<T>>;

	/// Groups of unbonding pools. Each group of unbonding pools belongs to a bonded pool,
	/// hence the name sub-pools. Keyed by the bonded pools account.
	#[pallet::storage]
	pub type SubPoolsStorage<T: Config> = CountedStorageMap<_, Twox64Concat, PoolId, SubPools<T>>;

	/// The next pool id that will be used in [`create`](Pallet::create). Increments by one with
	/// each pool created.
	#[pallet::storage]
	pub type NextPoolId<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// A reverse lookup from the pool's account id to its id.
	///
	/// This is only used for slashing. In all other instances, the pool id is used, and the
	/// accounts are deterministically derived from it.
	#[pallet::storage]
	pub type ReversePoolIdLookup<T: Config> =
		CountedStorageMap<_, Twox64Concat, T::AccountId, PoolId, OptionQuery>;

	/// A reverse lookup from the token_id to pool_id.
	///
	/// This is used for making sure the same token is not used to create multiple pools
	#[pallet::storage]
	pub type UsedPoolTokenIds<T: Config> =
		StorageMap<_, Twox64Concat, TokenIdOf<T>, PoolId, OptionQuery>;

	/// The maximum commission that can be charged by a pool. Used on commission payouts to bound
	/// pool commissions that are > `GlobalMaxCommission`, necessary if a future
	#[pallet::storage]
	pub type GlobalMaxCommission<T: Config> = StorageValue<_, Perbill, OptionQuery>;

	/// The general staking parameters
	#[pallet::storage]
	pub type StakingInformation<T: Config> = StorageValue<_, StakingInfo, OptionQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub min_join_bond: BalanceOf<T>,
		pub min_create_bond: BalanceOf<T>,
		pub max_pools: Option<u32>,
		pub max_members_per_pool: Option<u32>,
		pub max_members: Option<u32>,
		pub min_validator_commission: Option<Perbill>,
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
				min_validator_commission: Some(Perbill::zero()),
				global_max_commission: Some(Perbill::from_percent(10)),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			MinJoinBond::<T>::put(self.min_join_bond);
			MinCreateBond::<T>::put(self.min_create_bond);
			if let Some(min_validator_commission) = self.min_validator_commission {
				pallet_staking::MinCommission::<T>::put(min_validator_commission);
			}
			if let Some(global_max_commission) = self.global_max_commission {
				GlobalMaxCommission::<T>::put(global_max_commission);
			}
		}
	}

	/// Events of this pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
		/// A pool has been created.
		Created { creator: T::AccountId, pool_id: PoolId, capacity: BalanceOf<T> },
		/// A member has became bonded in a pool.
		Bonded { member: T::AccountId, pool_id: PoolId, bonded: BalanceOf<T> },
		/// A member has unbonded from their pool.
		Unbonded {
			/// The member that unbonded
			member: T::AccountId,
			/// The id of the pool unbonded from
			pool_id: PoolId,
			/// the corresponding balance of the number of points that has been requested to be
			/// unbonded (the argument of the `unbond` transaction) from the bonded pool.
			balance: BalanceOf<T>,
			/// the number of points that are issued as a result of `balance` being dissolved into
			/// the corresponding unbonding pool.
			points: BalanceOf<T>,
			/// the era in which the balance will be unbonded. In the absence of slashing,
			/// these values will match. In the presence of slashing, the number of points that are
			/// issued in the unbonding pool will be less than the amount requested to be unbonded.
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
		/// The active balance of pool `pool_id` has been slashed to `balance`.
		PoolSlashed { pool_id: PoolId, balance: BalanceOf<T> },
		/// The unbond pool at `era` of pool `pool_id` has been slashed to `balance`.
		UnbondingPoolSlashed { pool_id: PoolId, era: EraIndex, balance: BalanceOf<T> },
		/// A pool's commission rate has been changed.
		CommissionUpdated { pool_id: PoolId, current: Option<Perbill> },
		/// This event happens once per era on the previous era that rewards are paid out for. It
		/// pays commission, distributes bonus, and reinvests rewards.
		EraRewardsProcessed {
			/// The id of the pool
			pool_id: PoolId,
			/// The era that was processed.
			era: EraIndex,
			/// The commission that was paid
			commission: Option<CommissionPaymentOf<T>>,
			/// The amount of bonus that was unlocked
			bonus: BalanceOf<T>,
			/// The amount that was bonded
			reinvested: BalanceOf<T>,
			/// The current bonus cycle ended
			bonus_cycle_ended: bool,
		},
		/// Rewards were paid to a pool
		RewardPaid {
			/// The id of the pool
			pool_id: PoolId,
			/// The era that was processed.
			era: EraIndex,
			/// The validator that the payment was received from
			validator_stash: T::AccountId,
			/// The amount added to the pool's reward account
			reward: BalanceOf<T>,
			/// The amount that was added to the pool's bonus account
			bonus: BalanceOf<T>,
		},
		/// Pool has been mutated.
		PoolMutated { pool_id: PoolId, mutation: PoolMutationOf<T> },
		/// A nomination took place
		Nominated {
			/// The id of the pool
			pool_id: PoolId,
			/// The validators that were nominated
			validators: Vec<T::AccountId>,
		},
	}

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq))]
	pub enum Error<T> {
		/// A (bonded) pool id does not exist.
		PoolNotFound,
		/// Pool already exists for the given token_id
		PoolTokenAlreadyInUse,
		/// An account is not a member.
		PoolMemberNotFound,
		/// A reward pool does not exist. In all cases this is a system logic error.
		RewardPoolNotFound,
		/// A sub pool does not exist.
		SubPoolsNotFound,
		/// An account is already delegating in another pool. An account may only belong to one
		/// pool at a time.
		AccountBelongsToOtherPool,
		/// The member is fully unbonded (and thus cannot access the bonded and reward pool
		/// anymore to, for example, collect rewards).
		FullyUnbonding,
		/// The member cannot unbond further chunks due to reaching the limit.
		MaxUnbondingLimit,
		/// None of the funds can be withdrawn yet because the bonding duration has not passed.
		CannotWithdrawAny,
		/// The amount does not meet the minimum bond to either join or create a pool.
		///
		/// If the chain is not being destroyed no member can unbond to a value less than
		/// `Pallet::depositor_min_bond`. The caller does not have nominating
		/// permissions for the pool. Members can never unbond to a value below `MinJoinBond`.
		MinimumBondNotMet,
		/// The transaction could not be executed due to overflow risk for the pool.
		OverflowRisk,
		/// A pool must be in [`PoolState::Destroying`] in order for
		/// other members to be permissionlessly unbonded.
		NotDestroying,
		/// The caller does not have nominating permissions for the pool.
		NotNominator,
		/// Either a) the caller cannot make a valid kick or b) the pool is not destroying.
		NotKickerOrDestroying,
		/// The pool is not open to join
		NotOpen,
		/// The pools state cannot be changed.
		CanNotChangeState,
		/// The caller does not have adequate permissions.
		DoesNotHavePermission,
		/// Some error occurred that should never happen. This should be reported to the
		/// maintainers.
		Defensive(DefensiveError),
		/// Partial unbonding now allowed permissionlessly.
		PartialUnbondNotAllowedPermissionlessly,
		/// Pool id currently in use.
		PoolIdInUse,
		/// Pool id provided is not correct/usable.
		InvalidPoolId,
		/// Mint parameters are invalid.
		MintParamsCreationFailed,
		/// Burn parameters are invalid.
		BurnParamsCreationFailed,
		/// Transfer parameters are invalid.
		TransferParamsCreationFailed,
		/// The capacity of the pool is exceeded by the amount
		CapacityExceeded,
		/// The capacity can only be mutated for the first 14 eras of a cycle
		CapacityMutationRestricted,
		/// The duration is out of bounds
		DurationOutOfBounds,
		/// The required token is not owned by the caller
		TokenRequired,
		/// Deposit should be the last supply of pool's LST token to be unbonded
		DepositNotReadyForUnbonding,
		/// The pool's max commission cannot be set higher than the existing value.
		MaxCommissionRestricted,
		/// The supplied commission exceeds the max allowed commission.
		CommissionExceedsMaximum,
		/// Not enough blocks have surpassed since the last commission update.
		CommissionChangeThrottled,
		/// The submitted changes to commission change rate are not allowed.
		CommissionChangeRateNotAllowed,
		/// There is no pending commission to claim.
		NoPendingCommission,
		/// No commission current has been set.
		NoCommissionCurrentSet,
		/// The mutation does not change anything
		NoopMutation,
		/// The pool is not empty
		PoolMembersRemaining,
		/// A bounded value was exceeded
		BoundExceeded,
		/// Wrong account count
		WrongAccountCount,
	}

	#[derive(Encode, Decode, PartialEq, TypeInfo, frame_support::PalletError, RuntimeDebug)]
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
		/// Somehow division underflowed or overflowed
		DivisionError,
	}

	impl<T> From<DefensiveError> for Error<T> {
		fn from(e: DefensiveError) -> Error<T> {
			Error::<T>::Defensive(e)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Stake funds with a pool. The amount to bond is transferred from the member to the
		/// pools account and immediately increases the pools bond. The LST token will be minted
		/// and transferred to `origin`.
		///
		/// # Parameters
		/// - `origin`: the caller
		/// - `pool_id`: the pool id to bond
		/// - `amount`: the amount of tokens deposited into the pool
		///
		/// # Note
		///
		/// * An account can only be a member of a single pool.
		/// * An account cannot join the same pool multiple times.
		/// * This call will *not* dust the member account, so the member must have at least
		///   `existential deposit + amount` in their account.
		/// * Only a pool with [`PoolState::Open`] can be joined
		#[pallet::call_index(0)]
		#[pallet::weight(WeightInfoOf::<T>::bond())]
		pub fn bond(
			origin: OriginFor<T>,
			pool_id: PoolId,
			amount: BondValueOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

			bonded_pool.ok_to_join()?;

			let amount = match amount {
				BondValue::Amount(amount) => amount,
				BondValue::Fill => bonded_pool.points_to_balance(bonded_pool.available_points()),
			};
			ensure!(amount >= MinJoinBond::<T>::get(), Error::<T>::MinimumBondNotMet);

			let points_issued = bonded_pool.try_bond_funds(&who, amount, BondType::Later)?;

			// Mint tokens
			Self::mint_lst(who.clone(), bonded_pool.id, points_issued, false)?;

			bonded_pool.ok_to_be_open()?;

			Self::deposit_event(Event::<T>::Bonded { member: who, pool_id, bonded: amount });

			bonded_pool.put();

			Ok(())
		}

		/// Unbond up to `unbonding_points` of the `member_account`'s funds from the pool by burning
		/// LST.
		///
		/// Under certain conditions, this call can be dispatched permissionlessly (i.e. by any
		/// account).
		///
		/// # Conditions for a permissionless dispatch.
		///
		/// * The pool is blocked and the caller is holding the pool's token. This is refereed to as
		///   a kick.
		/// * The pool is destroying.
		/// * The pool is destroying and no other members are in the pool.
		///
		/// ## Conditions for permissioned dispatch (i.e. the caller is also the
		/// `member_account`):
		///
		/// * The caller is not the last member.
		/// * The caller is the last member and the pool is destroying.
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
		#[pallet::weight(WeightInfoOf::<T>::unbond())]
		pub fn unbond(
			origin: OriginFor<T>,
			pool_id: PoolId,
			member_account: AccountIdLookupOf<T>,
			#[pallet::compact] unbonding_points: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let member_account = T::Lookup::lookup(member_account)?;

			Self::do_unbond(who, pool_id, member_account, unbonding_points)
		}

		/// Call `withdraw_unbonded` for the pools account. This call can be made by any account.
		///
		/// This is useful if their are too many unlocking chunks to call `unbond`, and some
		/// can be cleared by withdrawing. In the case there are too many unlocking chunks, the user
		/// would probably see an error like `NoMoreChunks` emitted from the staking system when
		/// they attempt to unbond.
		#[pallet::call_index(4)]
		#[pallet::weight(WeightInfoOf::<T>::pool_withdraw_unbonded(*num_slashing_spans))]
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
			T::Staking::withdraw_unbonded(pool.bonded_account(), num_slashing_spans)?;
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
		/// * The pool is in destroy mode.
		/// * The target is the only member in the sub pools.
		/// * The pool is blocked and the caller is either the admin or state-toggler.
		///
		/// # Conditions for permissioned dispatch
		///
		/// * The caller is the target and they are not the last member.
		///
		/// # Note
		///
		/// If the target is the last member, the pool will be destroyed.
		#[pallet::call_index(5)]
		#[pallet::weight(
			WeightInfoOf::<T>::withdraw_unbonded_update(*num_slashing_spans)
		)]
		pub fn withdraw_unbonded(
			origin: OriginFor<T>,
			pool_id: PoolId,
			member_account: AccountIdLookupOf<T>,
			num_slashing_spans: u32,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			let member_account = T::Lookup::lookup(member_account)?;

			Self::do_withdraw_unbonded(caller, pool_id, member_account, num_slashing_spans)
		}

		/// Create a new nomination pool.
		///
		/// # Arguments
		///
		/// * `token_id` - Token that that will control the pool. This token must be from the
		///   [`Config::PoolCollectionId`] collection and it must be held by the caller.
		/// * `deposit` - The amount of funds to delegate to the pool. This also acts as a deposit
		///   because the pool's creator cannot fully unbond funds until the pool is destroyed.
		/// * `capacity` - The maximum total balance allowed in the pool. This is measured in LST.
		///   It must be below the pool's capacity. See `Capacity` section in crate level docs.
		/// * `duration` - The duration in blocks of the pool's bonus cycle
		///
		/// # Note
		///
		/// In addition to `deposit`, the caller will transfer the existential deposit for the
		/// pool's accounts; so the caller needs at have at least `deposit + existential_deposit *
		/// 2` transferable.
		#[pallet::call_index(6)]
		#[pallet::weight(WeightInfoOf::<T>::create())]
		pub fn create(
			origin: OriginFor<T>,
			token_id: TokenIdOf<T>,
			#[pallet::compact] deposit: BalanceOf<T>,
			#[pallet::compact] capacity: BalanceOf<T>,
			#[pallet::compact] duration: EraIndex,
			name: PoolNameOf<T>,
		) -> DispatchResult {
			let creator = ensure_signed(origin)?;

			let pool_id = NextPoolId::<T>::try_mutate::<_, Error<T>, _>(|next_id| {
				let current_id = *next_id;
				*next_id = next_id.checked_add(1).ok_or(Error::<T>::OverflowRisk)?;
				Ok(current_id)
			})?;

			Self::do_create(creator, pool_id, token_id, deposit, capacity, duration, name)
		}

		/// Nominate on behalf of the pool.
		///
		/// The dispatch origin of this call must be signed by the holder of the pool token.
		///
		/// This directly forward the call to the staking pallet, on behalf of the pool bonded
		/// account.
		#[pallet::call_index(8)]
		#[pallet::weight(WeightInfoOf::<T>::nominate(validators.len() as u32))]
		pub fn nominate(
			origin: OriginFor<T>,
			pool_id: PoolId,
			validators: Vec<T::AccountId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			ensure!(bonded_pool.can_nominate(&who), Error::<T>::NotNominator);
			T::Staking::nominate(&bonded_pool.bonded_account(), validators.clone())?;
			Self::deposit_event(Event::<T>::Nominated { pool_id, validators });
			Ok(())
		}

		/// Update configurations for the nomination pools. Callable only by
		/// [`Config::ForceOrigin`].
		///
		/// # Arguments
		///
		/// * `min_join_bond` - Set [`MinJoinBond`].
		/// * `min_create_bond` - Set [`MinCreateBond`].
		/// * `global_max_commission` - Set [`GlobalMaxCommission`].
		#[pallet::call_index(11)]
		#[pallet::weight(WeightInfoOf::<T>::set_configs())]
		pub fn set_configs(
			origin: OriginFor<T>,
			min_join_bond: ConfigOp<BalanceOf<T>>,
			min_create_bond: ConfigOp<BalanceOf<T>>,
			global_max_commission: ConfigOp<Perbill>,
			required_payout_count: ConfigOp<Perbill>,
		) -> DispatchResult {
			<T as Config>::ForceOrigin::ensure_origin(origin)?;

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
			config_op_exp!(GlobalMaxCommission::<T>, global_max_commission);

			// required_payout_count is not stored by itself, so it can't use the macro
			match required_payout_count {
				ConfigOp::Noop => (),
				ConfigOp::Set(value) => {
					EraPayoutInfo::<T>::mutate(|storage| storage.required_payments_percent = value)
				},
				// for removal, set it to zero percent
				ConfigOp::Remove => EraPayoutInfo::<T>::mutate(|storage| {
					storage.required_payments_percent = Perbill::from_percent(0)
				}),
			}
			Ok(())
		}

		/// Chill on behalf of the pool.
		///
		/// The dispatch origin of this call must be signed by the pool token holder, same as
		/// [`Pallet::nominate`].
		///
		/// This directly forward the call to the staking pallet, on behalf of the pool bonded
		/// account.
		#[pallet::call_index(13)]
		#[pallet::weight(WeightInfoOf::<T>::chill())]
		pub fn chill(origin: OriginFor<T>, pool_id: PoolId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			ensure!(bonded_pool.can_nominate(&who), Error::<T>::NotNominator);
			T::Staking::chill(&bonded_pool.bonded_account())
		}

		/// Destroy the pool.
		///
		/// The dispatch origin of this call must be signed by the account holding the pool token
		/// of the given pool_id.
		#[pallet::call_index(14)]
		#[pallet::weight(WeightInfoOf::<T>::destroy())]
		pub fn destroy(origin: OriginFor<T>, pool_id: PoolId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

			// If pool's token has an owner, then the origin must be the owner.
			// Otherwise, this call is permissionless.
			if let Some(owner) = bonded_pool.token_owner() {
				ensure!(owner == who, Error::<T>::DoesNotHavePermission);
			}

			bonded_pool.set_state(PoolState::Destroying);
			bonded_pool.put();
			Ok(())
		}

		/// Pays rewards to `validator_stash` and also distributes rewards to the reward accounts of
		/// the pools nominating it. The appropriate bonus is also calculated and stored in the
		/// bonus account.
		///
		/// This should be called once per era per validator. It is a permissionless call. It also
		/// processes rewards for the previous era if [`Self::process_payouts`] was not called.
		///
		/// ## Bonus Calculation
		///
		/// 1. Minimum duration and max duration are found for all pools nominating
		///    `validator_stash`
		/// 2. [`Config::BonusPercentage`] is set aside from rewards for bonus
		/// 3. Normalized weight is calculated and then scaled according to the total bonus. See
		///    `functions::calculate_real_weight`.
		/// 4. The scaled weight is offset according to [`Config::BaseBonusRewardPercentage`] so
		///    that all pools at least get the minimum weight
		/// 5. Final calculation is done in [`traits::Bonus::calculate_bonus`] and then transferred
		///    to the bonus account
		#[pallet::call_index(18)]
		#[pallet::weight(
			WeightInfoOf::<T>::payout_rewards(T::MaxExposurePageSize::get())
		)]
		pub fn payout_rewards(
			origin: OriginFor<T>,
			validator_stash: T::AccountId,
			era: EraIndex,
		) -> DispatchResultWithPostInfo {
			Self::do_payout_rewards(origin, validator_stash, era)
		}

		/// Processes the rewards for all pools that were distributed in [`Self::payout_rewards`].
		/// It will only succeed if it is called on the same era that payouts were made. It uses the
		/// [`EraPayoutInfo`] storage to verify this. This extrinsic is permissionless.
		///
		/// The following is done for each pool:
		/// 1. If the pool has reached the end of its cycle, it cycles the pool.
		/// 2. Sends bonus for the current era from the bonus account to the rewards account.
		/// 3. Sends reward commission to the depositor.
		/// 4. It bonds the pool's reward balance.
		///
		/// It is not required to call this extrinsic. If it is not called, the rewards will be
		/// processed when `payout_rewards` is called in the next era.
		#[pallet::call_index(25)]
		#[pallet::weight(WeightInfoOf::<T>::process_payouts(*pool_count))]
		pub fn process_payouts(origin: OriginFor<T>, pool_count: u32) -> DispatchResult {
			ensure_signed(origin)?;
			Self::do_process_payouts(pool_count)
		}

		/// Mutate the nomination pool data.
		///
		/// The dispatch origin of this call must be signed by the account holding the pool token
		/// of the given pool_id.
		#[pallet::call_index(19)]
		#[pallet::weight(WeightInfoOf::<T>::mutate())]
		pub fn mutate(
			origin: OriginFor<T>,
			pool_id: PoolId,
			mutation: PoolMutationOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!mutation.is_noop(), Error::<T>::NoopMutation);

			let mut bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			let is_manager = bonded_pool.has_pool_token(&who);
			ensure!(is_manager, Error::<T>::DoesNotHavePermission);

			if let Some(duration) = mutation.duration {
				ensure!(duration >= T::MinDuration::get(), Error::<T>::DurationOutOfBounds);
				ensure!(duration <= T::MaxDuration::get(), Error::<T>::DurationOutOfBounds);
				bonded_pool.bonus_cycle.pending_duration = Some(duration);
			}

			if let ShouldMutate::SomeMutation(new_commission) = mutation.new_commission {
				ensure!(bonded_pool.can_manage_commission(&who), Error::<T>::DoesNotHavePermission);
				bonded_pool.commission.try_update_current(&new_commission)?;
			}

			if let Some(max_commission) = mutation.max_commission {
				ensure!(bonded_pool.can_manage_commission(&who), Error::<T>::DoesNotHavePermission);
				bonded_pool.commission.try_update_max(pool_id, max_commission)?;
			}

			if let Some(change_rate) = mutation.change_rate {
				ensure!(bonded_pool.can_manage_commission(&who), Error::<T>::DoesNotHavePermission);
				bonded_pool.commission.try_update_change_rate(change_rate)?;
			}

			if let Some(capacity) = mutation.capacity {
				let max_pool_capacity = Self::get_max_pool_capacity(bonded_pool.token_id)?;

				// ensure capacity doesnt exceed the limit
				ensure!(
					max_pool_capacity <= T::GlobalMaxCapacity::get(),
					Error::<T>::AttributeCapacityExceedsGlobalCapacity
				);
				ensure!(capacity <= max_pool_capacity, Error::<T>::CapacityExceeded);

				// capacity can only be mutated for the first 14 eras of a cycle
				let era = <<T as Config>::Staking as StakingInterface>::current_era();
				ensure!(
					era <= bonded_pool
						.bonus_cycle
						.start
						.saturating_add(T::CapacityMutationPeriod::get()),
					Error::<T>::CapacityMutationRestricted
				);

				// capacity can not be set lower than the current points in the pool
				ensure!(capacity >= bonded_pool.points(), Error::<T>::CapacityExceeded);

				// capacity can not be set below `num of pool validators x min validator stake`
				if let Some(num_validators) =
					pallet_staking::Nominators::<T>::get(&bonded_pool.bonded_account())
						.map(|n| n.targets.len() as u32)
				{
					let min_validator_capacity =
						<<T as Config>::Staking as StakingInterface>::minimum_validator_bond()
							.saturating_mul(num_validators.into());

					let min_validator_capacity =
						bonded_pool.balance_to_point(min_validator_capacity);

					if min_validator_capacity >= bonded_pool.points() {
						ensure!(capacity >= min_validator_capacity, Error::<T>::CapacityExceeded);
					}
				}

				bonded_pool.capacity = capacity;
			}

			if let Some(name) = mutation.name.as_ref() {
				bonded_pool.name = name.clone();
			}

			bonded_pool.put();

			Self::deposit_event(Event::<T>::PoolMutated { pool_id, mutation });
			Ok(())
		}

		/// Unbonds the deposit
		///
		/// This call is permissionless but certain conditions must be met before the deposit can
		/// be unbonded:
		///
		/// - Pool must be in [`PoolState::Destroying`] mode
		/// - Deposit points must be the only points in the pool
		/// - [`UnbondingMembers`] must be empty
		///
		/// This will unbond the deposit from the pool.
		#[pallet::call_index(20)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::unbond_deposit())]
		pub fn unbond_deposit(origin: OriginFor<T>, pool_id: PoolId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			let deposit_points = bonded_pool.deposit();

			// this extrinsic should only be called to unbond the deposit
			bonded_pool.can_unbond_deposit()?;

			Self::do_unbond(who, pool_id, Self::deposit_account_id(pool_id), deposit_points)
		}

		/// Withdraws the deposit
		///
		/// This call is permissionless and should be called after the deposit has been unbonded.
		#[pallet::call_index(21)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::withdraw_deposit())]
		pub fn withdraw_deposit(
			origin: OriginFor<T>,
			pool_id: PoolId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::do_withdraw_unbonded(who, pool_id, Self::deposit_account_id(pool_id), u32::MAX)
		}

		/// Transfers `amount` from the pool's free balance to `destination`. Only callable by
		/// [`Config::ForceOrigin`].
		#[pallet::call_index(26)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::withdraw_free_balance())]
		pub fn withdraw_free_balance(
			origin: OriginFor<T>,
			pool_id: PoolId,
			destination: AccountIdLookupOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			<T as Config>::ForceOrigin::ensure_origin(origin)?;

			CurrencyOf::<T>::transfer(
				&Self::compute_pool_account_id(pool_id, AccountType::Bonded),
				&T::Lookup::lookup(destination)?,
				amount,
				ExistenceRequirement::KeepAlive,
			)
		}

		/// Set the annual inflation rate and collator payout cut
		///
		/// Callable only by [`Config::ForceOrigin`]
		#[pallet::call_index(22)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::withdraw_deposit())]
		pub fn set_staking_info(
			origin: OriginFor<T>,
			info: StakingInfo,
		) -> DispatchResultWithPostInfo {
			<T as Config>::ForceOrigin::ensure_origin(origin)?;

			StakingInformation::<T>::set(Some(info));

			Ok(().into())
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
				the bonding duration > slash defer duration.",
			);
		}
	}
}
