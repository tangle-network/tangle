//! Common types for this pallet

use super::*;
use crate::GlobalMaxCommission;
use frame_support::traits::Imbalance;
use smart_default::SmartDefault;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Bounded},
	SaturatedConversion,
};
use sp_runtime::traits::CheckedAdd;
use frame_support::traits::fungibles::Inspect as FungiblesInspect;
use frame_support::traits::fungible::Inspect as FungibleInspect;
use frame_support::traits::tokens::Preservation;
use frame_support::traits::tokens::Fortitude;

/// The balance type used by the currency system.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
pub type CurrencyOf<T> = <T as Config>::Currency;
pub type AssetIdOf<T> = <<T as Config>::Fungibles as FungiblesInspect<AccountIdOf<T>>>::AssetId;
pub type TokenBalanceOf<T> = <<T as Config>::Fungibles as FungiblesInspect<<T as frame_system::Config>::AccountId>>::Balance;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub(super) type WeightInfoOf<T> = <T as Config>::WeightInfo;

/// Type used in early bird weight calculations. It makes sense to use `Balance` because the
/// calculation is based on it.
pub(super) type EarlyBirdWeightOf<T> = BalanceOf<T>;

/// Possible operations on the configuration values of this pallet.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebugNoBound, PartialEq, Clone)]
pub enum ConfigOp<T: Codec + Debug> {
	/// Don't change.
	Noop,
	/// Set the given value.
	Set(T),
	/// Remove from storage.
	Remove,
}

/// The type of bonding that can happen to a pool.
pub(crate) enum BondType {
	/// Someone is bonding into the pool upon creation.
	Create,
	/// Someone is adding more funds later to this pool.
	Later,
}

/// Explicit [`BondValue`]
pub type BondValueOf<T> = BondValue<BalanceOf<T>>;

/// Amount when joining or bonding extra to a pool
#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq, TypeInfo)]
pub enum BondValue<Balance> {
	/// A specific amount of balance
	Amount(#[codec(compact)] Balance),
	/// Fill the remaining amount of the pool's capacity
	Fill,
}

impl<Balance> From<Balance> for BondValue<Balance> {
	fn from(value: Balance) -> Self {
		Self::Amount(value)
	}
}

/// The type of account for a nomination pool.
#[derive(Encode, Decode)]
#[allow(clippy::unnecessary_cast)]
pub enum AccountType {
	/// Account that does the bonding
	Bonded = 1,
	/// Account that holds the rewards
	Reward = 2,
	/// Account that holds the bonus
	Bonus = 3,
}

/// A reward pool.
///
/// A reward pool is not so much a pool anymore, since it does not contain any shares or points.
/// Rather, simply to fit nicely next to bonded pool and unbonding pools in terms of terminology. In
/// reality, a reward pool is just a container for a few pool-dependent data related to the rewards.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebugNoBound)]
#[cfg_attr(feature = "std", derive(Clone, PartialEq, DefaultNoBound))]
#[codec(mel_bound(T: Config))]
#[scale_info(skip_type_params(T))]
pub struct RewardPool<T: Config> {
	/// The last recorded value of the reward counter.
	///
	/// This is updated ONLY when the points in the bonded pool change, which means `join`,
	/// `bond_extra` and `unbond`, all of which is done through `update_recorded`.
	last_recorded_reward_counter: T::RewardCounter,
	/// The last recorded total payouts of the reward pool.
	///
	/// Payouts is essentially income of the pool.
	///
	/// Update criteria is same as that of `last_recorded_reward_counter`.
	last_recorded_total_payouts: BalanceOf<T>,
	/// Total amount that this pool has paid out so far to the members.
	total_rewards_claimed: BalanceOf<T>,
	/// The amount of commission pending to be claimed.
	total_commission_pending: BalanceOf<T>,
	/// The amount of commission that has been claimed.
	total_commission_claimed: BalanceOf<T>,
}


impl<T: Config> RewardPool<T> {
	/// Getter for [`RewardPool::last_recorded_reward_counter`].
	pub(crate) fn last_recorded_reward_counter(&self) -> T::RewardCounter {
		self.last_recorded_reward_counter
	}

	/// Register some rewards that are claimed from the pool by the members.
	fn register_claimed_reward(&mut self, reward: BalanceOf<T>) {
		self.total_rewards_claimed = self.total_rewards_claimed.saturating_add(reward);
	}

	/// Update the recorded values of the reward pool.
	///
	/// This function MUST be called whenever the points in the bonded pool change, AND whenever the
	/// the pools commission is updated. The reason for the former is that a change in pool points
	/// will alter the share of the reward balance among pool members, and the reason for the latter
	/// is that a change in commission will alter the share of the reward balance among the pool.
	fn update_records(
		&mut self,
		id: PoolId,
		bonded_points: BalanceOf<T>,
		commission: Perbill,
	) -> Result<(), Error<T>> {
		let balance = Self::current_balance(id);

		let (current_reward_counter, new_pending_commission) =
			self.current_reward_counter(id, bonded_points, commission)?;

		// Store the reward counter at the time of this update. This is used in subsequent calls to
		// `current_reward_counter`, whereby newly pending rewards (in points) are added to this
		// value.
		self.last_recorded_reward_counter = current_reward_counter;

		// Add any new pending commission that has been calculated from `current_reward_counter` to
		// determine the total pending commission at the time of this update.
		self.total_commission_pending =
			self.total_commission_pending.saturating_add(new_pending_commission);

		// Total payouts are essentially the entire historical balance of the reward pool, equating
		// to the current balance + the total rewards that have left the pool + the total commission
		// that has left the pool.
		let last_recorded_total_payouts = balance
			.checked_add(&self.total_rewards_claimed.saturating_add(self.total_commission_claimed))
			.ok_or(Error::<T>::OverflowRisk)?;

		// Store the total payouts at the time of this update.
		//
		// An increase in ED could cause `last_recorded_total_payouts` to decrease but we should not
		// allow that to happen since an already paid out reward cannot decrease. The reward account
		// might go in deficit temporarily in this exceptional case but it will be corrected once
		// new rewards are added to the pool.
		self.last_recorded_total_payouts =
			self.last_recorded_total_payouts.max(last_recorded_total_payouts);

		Ok(())
	}

	/// Get the current reward counter, based on the given `bonded_points` being the state of the
	/// bonded pool at this time.
	fn current_reward_counter(
		&self,
		id: PoolId,
		bonded_points: BalanceOf<T>,
		commission: Perbill,
	) -> Result<(T::RewardCounter, BalanceOf<T>), Error<T>> {
		let balance = Self::current_balance(id);

		// Calculate the current payout balance. The first 3 values of this calculation added
		// together represent what the balance would be if no payouts were made. The
		// `last_recorded_total_payouts` is then subtracted from this value to cancel out previously
		// recorded payouts, leaving only the remaining payouts that have not been claimed.
		let current_payout_balance = balance
			.saturating_add(self.total_rewards_claimed)
			.saturating_add(self.total_commission_claimed)
			.saturating_sub(self.last_recorded_total_payouts);

		// Split the `current_payout_balance` into claimable rewards and claimable commission
		// according to the current commission rate.
		let new_pending_commission = commission * current_payout_balance;
		let new_pending_rewards = current_payout_balance.saturating_sub(new_pending_commission);

		// * accuracy notes regarding the multiplication in `checked_from_rational`:
		// `current_payout_balance` is a subset of the total_issuance at the very worse.
		// `bonded_points` are similarly, in a non-slashed pool, have the same granularity as
		// balance, and are thus below within the range of total_issuance. In the worse case
		// scenario, for `saturating_from_rational`, we have:
		//
		// dot_total_issuance * 10^18 / `minJoinBond`
		//
		// assuming `MinJoinBond == ED`
		//
		// dot_total_issuance * 10^18 / 10^10 = dot_total_issuance * 10^8
		//
		// which, with the current numbers, is a miniscule fraction of the u128 capacity.
		//
		// Thus, adding two values of type reward counter should be safe for ages in a chain like
		// Polkadot. The important note here is that `reward_pool.last_recorded_reward_counter` only
		// ever accumulates, but its semantics imply that it is less than total_issuance, when
		// represented as `FixedU128`, which means it is less than `total_issuance * 10^18`.
		//
		// * accuracy notes regarding `checked_from_rational` collapsing to zero, meaning that no
		//   reward can be claimed:
		//
		// largest `bonded_points`, such that the reward counter is non-zero, with `FixedU128` will
		// be when the payout is being computed. This essentially means `payout/bonded_points` needs
		// to be more than 1/1^18. Thus, assuming that `bonded_points` will always be less than `10
		// * dot_total_issuance`, if the reward_counter is the smallest possible value, the value of
		//   the
		// reward being calculated is:
		//
		// x / 10^20 = 1/ 10^18
		//
		// x = 100
		//
		// which is basically 10^-8 DOTs. See `smallest_claimable_reward` for an example of this.
		let current_reward_counter =
			T::RewardCounter::checked_from_rational(new_pending_rewards, bonded_points)
				.and_then(|ref r| self.last_recorded_reward_counter.checked_add(r))
				.ok_or(Error::<T>::OverflowRisk)?;

		Ok((current_reward_counter, new_pending_commission))
	}

	/// Current free balance of the reward pool.
	///
	/// This is sum of all the rewards that are claimable by pool members.
	fn current_balance(id: PoolId) -> BalanceOf<T> {
		T::Currency::reducible_balance(
			&Pallet::<T>::create_reward_account(id),
			Preservation::Expendable,
			Fortitude::Polite,
		)
	}
}

/// A member in a pool.
#[derive(
	Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebugNoBound, CloneNoBound, DefaultNoBound,
)]
#[cfg_attr(feature = "std", derive(frame_support::PartialEqNoBound))]
#[codec(mel_bound(T: Config))]
#[scale_info(skip_type_params(T))]
pub struct PoolMember<T: Config> {
	/// The eras in which this member is unbonding, mapped from era index to the number of
	/// points scheduled to unbond in the given era.
	pub unbonding_eras: BoundedBTreeMap<EraIndex, BalanceOf<T>, T::MaxUnbonding>,
}

impl<T: Config> PoolMember<T> {
	/// Total points of this member, both active and unbonding.
	pub(crate) fn total_points(
		&self,
		pool_id: PoolId,
		member_account: T::AccountId,
	) -> BalanceOf<T> {
		Pallet::<T>::member_points(pool_id, member_account).saturating_add(self.unbonding_points())
	}

	/// Inactive points of the member, waiting to be withdrawn.
	pub(crate) fn unbonding_points(&self) -> BalanceOf<T> {
		self.unbonding_eras
			.as_ref()
			.iter()
			.fold(BalanceOf::<T>::zero(), |acc, (_, v)| acc.saturating_add(*v))
	}

	/// Mint `points_issued` into the corresponding `era`'s unlock schedule.
	///
	/// In the absence of slashing, these two points are always the same. In the presence of
	/// slashing, the value of points in different pools varies.
	///
	/// Returns `Ok(())` and updates `unbonding_eras` and `points` if success, `Err(_)` otherwise.
	pub(crate) fn try_unbond(
		&mut self,
		points_issued: BalanceOf<T>,
		unbonding_era: EraIndex,
	) -> Result<(), Error<T>> {
		match self.unbonding_eras.get_mut(&unbonding_era) {
			Some(already_unbonding_points) => {
				*already_unbonding_points = already_unbonding_points.saturating_add(points_issued)
			},
			None => self
				.unbonding_eras
				.try_insert(unbonding_era, points_issued)
				.map(|old| {
					if old.is_some() {
						defensive!("value checked to not exist in the map; qed");
					}
				})
				.map_err(|_| Error::<T>::MaxUnbondingLimit)?,
		}
		Ok(())
	}

	/// Withdraw any funds in [`Self::unbonding_eras`] who's deadline in reached and is fully
	/// unlocked.
	///
	/// Returns a a subset of [`Self::unbonding_eras`] that got withdrawn.
	///
	/// Infallible, noop if no unbonding eras exist.
	pub(crate) fn withdraw_unlocked(
		&mut self,
		current_era: EraIndex,
	) -> BoundedBTreeMap<EraIndex, BalanceOf<T>, T::MaxUnbonding> {
		// NOTE: if only drain-filter was stable..
		let mut removed_points =
			BoundedBTreeMap::<EraIndex, BalanceOf<T>, T::MaxUnbonding>::default();
		self.unbonding_eras.retain(|e, p| {
			if *e > current_era {
				true
			} else {
				removed_points
					.try_insert(*e, *p)
					.expect("source map is bounded, this is a subset, will be bounded; qed");
				false
			}
		});
		removed_points
	}
}

/// A pool's possible states.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, RuntimeDebugNoBound, Clone, Copy)]
pub enum PoolState {
	/// The pool is open to be joined, and is working normally.
	Open,
	/// The pool is blocked. No one else can join.
	Blocked,
	/// The pool is in the process of being destroyed.
	///
	/// All members can now be permissionlessly unbonded, and the pool can never go back to any
	/// other state other than being dissolved.
	Destroying,
}

/// Pool administration roles.
///
/// Any pool has a depositor, which can never change. But, all the other roles are optional, and
/// cannot exist. Note that if `root` is set to `None`, it basically means that the roles of this
/// pool can never change again (except via governance).
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, PartialEq, Clone)]
pub struct PoolRoles<AccountId> {
	/// Creates the pool and is the initial member. They can only leave the pool once all other
	/// members have left. Once they fully leave, the pool is destroyed.
	pub depositor: AccountId,
	/// Can change the nominator, bouncer, or itself and can perform any of the actions the
	/// nominator or bouncer can.
	pub root: AccountId,
	/// Can select which validators the pool nominates.
	pub nominator: Option<AccountId>,
	/// Can change the pools state and kick members if the pool is blocked.
	pub bouncer: Option<AccountId>,
}

/// Pool permissions and state
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, DebugNoBound, PartialEqNoBound, CloneNoBound)]
#[codec(mel_bound(T: Config))]
#[scale_info(skip_type_params(T))]
pub struct BondedPoolInner<T: Config> {
	/// The commission rate of the pool.
	pub commission: Commission<T>,
	/// Total points of all the members in the pool who are actively bonded.
	pub points: BalanceOf<T>,
	/// See [`PoolRoles`].
	pub roles: PoolRoles<T::AccountId>,
	/// The current state of the pool.
	pub state: PoolState,
}

/// A wrapper for bonded pools, with utility functions.
///
/// The main purpose of this is to wrap a [`BondedPoolInner`], with the account + id of the pool,
/// for easier access.
#[derive(RuntimeDebugNoBound)]
#[cfg_attr(feature = "std", derive(CloneNoBound, PartialEqNoBound))]
pub struct BondedPool<T: Config> {
	/// The identifier of the pool.
	pub(crate) id: PoolId,
	/// The inner fields.
	pub(crate) inner: BondedPoolInner<T>,
}

impl<T: Config> sp_std::ops::Deref for BondedPool<T> {
	type Target = BondedPoolInner<T>;
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<T: Config> sp_std::ops::DerefMut for BondedPool<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

impl<T: Config> BondedPool<T> {

		/// Create a new bonded pool with the given roles and identifier.
		pub fn new(id: PoolId, roles: PoolRoles<T::AccountId>) -> Self {
			Self {
				id,
				inner: BondedPoolInner {
					commission: Commission::default(),
					points: Zero::zero(),
					roles,
					state: PoolState::Open,
				},
			}
		}

	/// Get [`Self`] from storage. Returns `None` if no entry for `pool_account` exists.
	pub fn get(id: PoolId) -> Option<Self> {
		BondedPools::<T>::try_get(id).ok().map(|inner| Self { id, inner })
	}

	/// Get one of the pool's accounts
	pub(crate) fn bonded_account(&self) -> T::AccountId {
		Pallet::<T>::compute_pool_account_id(self.id, AccountType::Bonded)
	}

	/// Get the reward account id of this pool.
	pub(crate) fn reward_account(&self) -> T::AccountId {
		Pallet::<T>::compute_pool_account_id(self.id, AccountType::Reward)
	}

	/// Get the bonus account id of this pool.
	pub(crate) fn bonus_account(&self) -> T::AccountId {
		Pallet::<T>::compute_pool_account_id(self.id, AccountType::Bonus)
	}

	/// Consume self and put into storage.
	pub(crate) fn put(self) {
		BondedPools::<T>::insert(self.id, self.inner);
	}

	/// Consume self and remove from storage.
	pub(crate) fn remove(self) {
		BondedPools::<T>::remove(self.id);
	}

	pub fn points(&self) -> BalanceOf<T> {
		T::Fungibles::total_issuance(self.id.saturated_into())
	}

	/// Convert the given amount of balance to points given the current pool state.
	///
	/// This is often used for bonding and issuing new funds into the pool.
	pub(crate) fn balance_to_point(&self, new_funds: BalanceOf<T>) -> BalanceOf<T> {
		let bonded_balance =
			T::Staking::active_stake(&self.bonded_account()).unwrap_or(Zero::zero());
		Pallet::<T>::balance_to_point(bonded_balance, self.points(), new_funds)
	}

	/// Convert the given number of points to balance given the current pool state.
	///
	/// This is often used for unbonding.
	pub(crate) fn points_to_balance(&self, points: BalanceOf<T>) -> BalanceOf<T> {
		let bonded_balance =
			T::Staking::active_stake(&self.bonded_account()).unwrap_or(Zero::zero());
		Pallet::<T>::point_to_balance(bonded_balance, self.points(), points)
	}

	/// Issue points to [`Self`] for `new_funds`.
	pub(crate) fn issue(&mut self, new_funds: BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError> {
		let points_to_issue = self.balance_to_point(new_funds);
		Ok(points_to_issue)
	}

	/// Returns the equivalent balance amount that actually needs to get unbonded.
	pub(crate) fn dissolve(&mut self, points: BalanceOf<T>) -> BalanceOf<T> {
		self.points_to_balance(points)
	}

	pub(crate) fn transferrable_balance(&self) -> BalanceOf<T> {
		let account = self.bonded_account();
		CurrencyOf::<T>::free_balance(&account)
			.saturating_sub(T::Staking::active_stake(&account).unwrap_or_default())
	}

	pub(crate) fn deposit(&self) -> BalanceOf<T> {
		Pallet::<T>::member_points(self.id, Pallet::<T>::deposit_account_id(self.id))
	}

	/// Returns true if `who` holds the pool token for this pool
	pub(crate) fn has_pool_token(&self, who: &T::AccountId) -> bool {
		let pool_token_balance = T::Fungibles::balance(
			self.id.saturated_into(),
			&who.clone(),
		);
		pool_token_balance > Zero::zero()
	}

	pub(crate) fn can_nominate(&self, who: &T::AccountId) -> bool {
		self.has_pool_token(who)
	}

	pub(crate) fn can_kick(&self, who: &T::AccountId) -> bool {
		self.has_pool_token(who)
	}

	pub(crate) fn can_manage_commission(&self, who: &T::AccountId) -> bool {
		self.has_pool_token(who)
	}

	pub(crate) fn can_unbond_deposit(&self) -> Result<(), DispatchError> {
		use frame_support::StorageDoubleMap;

		// we need to ensure that the pool is in the destroying state and that the deposit points
		// are the only points left in the pool.
		ensure!(
			self.is_destroying() && self.points() == self.deposit(),
			Error::<T>::DepositNotReadyForUnbonding
		);

		// this check should only be called once all the members of the pool has left
		// this check helps ensure that there are no currently unbonding funds, if the pool
		// is destroyed then the unbonding funds are lost forever.
		ensure!(!UnbondingMembers::<T>::contains_prefix(self.id), Error::<T>::PoolMembersRemaining);

		Ok(())
	}

	pub(crate) fn is_destroying(&self) -> bool {
		matches!(self.state, PoolState::Destroying)
	}

	/// Whether or not the pool is ok to be in `PoolSate::Open`. If this returns an `Err`, then the
	/// pool is unrecoverable and should be in the destroying state.
	pub(crate) fn ok_to_be_open(&self) -> Result<(), DispatchError> {
		ensure!(!self.is_destroying(), Error::<T>::CanNotChangeState);

		let bonded_balance =
			T::Staking::active_stake(&self.bonded_account()).unwrap_or(Zero::zero());
		ensure!(!bonded_balance.is_zero(), Error::<T>::OverflowRisk);

		let points_to_balance_ratio_floor = self
			.points()
			// We checked for zero above
			.div(bonded_balance);

		let max_points_to_balance = T::MaxPointsToBalance::get();

		// Pool points can inflate relative to balance, but only if the pool is slashed.
		// If we cap the ratio of points:balance so one cannot join a pool that has been slashed
		// by `max_points_to_balance`%, if not zero.
		ensure!(
			points_to_balance_ratio_floor < max_points_to_balance.into(),
			Error::<T>::OverflowRisk
		);

		// then we can be decently confident the bonding pool points will not overflow
		// `BalanceOf<T>`. Note that these are just heuristics.

		Ok(())
	}

	/// Check that the pool can accept a member with `new_funds`.
	pub(crate) fn ok_to_join(&self) -> Result<(), DispatchError> {
		ensure!(self.state == PoolState::Open, Error::<T>::NotOpen);
		self.ok_to_be_open()?;
		Ok(())
	}

	pub(crate) fn ok_to_unbond_with(
		&self,
		caller: &T::AccountId,
		pool_id: PoolId,
		target_account: &T::AccountId,
		unbonding_points: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		let is_permissioned = caller == target_account;
		let active_points = Pallet::<T>::member_points(pool_id, target_account.clone());

		// let is_deposit_unbond = T::LstCollectionOwner::get() == *target_account;

		// if is_deposit_unbond {
		// 	self.can_unbond_deposit()?;
		// }
		let is_deposit_unbond = false;

		// deposit unbonding is always full unbonding
		let is_full_unbond = is_deposit_unbond || active_points == unbonding_points;

		let balance_after_unbond = {
			let new_depositor_points = active_points.saturating_sub(unbonding_points);
			BondedPool::<T>::get(pool_id)
				.defensive()
				.map(|x| x.points_to_balance(new_depositor_points))
				.unwrap_or_default()
		};

		// any partial unbonding is only ever allowed if this unbond is permissioned.
		ensure!(
			is_permissioned || is_full_unbond,
			Error::<T>::PartialUnbondNotAllowedPermissionlessly
		);

		// any unbond must comply with the balance condition:
		ensure!(
			is_full_unbond || balance_after_unbond >= MinJoinBond::<T>::get(),
			Error::<T>::MinimumBondNotMet
		);

		if !is_permissioned && !is_deposit_unbond {
			// If the pool is blocked, then an admin with kicking permissions can remove a
			// member. If the pool is being destroyed, anyone can remove a member
			debug_assert!(is_full_unbond);
			ensure!(
				self.can_kick(caller) || self.is_destroying(),
				Error::<T>::NotKickerOrDestroying
			)
		}

		Ok(())
	}

	/// # Returns
	///
	/// * Ok(()) if [`Call::withdraw_unbonded`] can be called, `Err(DispatchError)` otherwise.
	pub(crate) fn ok_to_withdraw_unbonded_with(
		&self,
		caller: &T::AccountId,
		target_account: &T::AccountId,
	) -> Result<(), DispatchError> {
		// This isn't a depositor
		let is_permissioned = caller == target_account;
		ensure!(
			is_permissioned || self.can_kick(caller) || self.is_destroying(),
			Error::<T>::NotKickerOrDestroying
		);
		Ok(())
	}

	/// Bond exactly `amount` from `who`'s funds into this pool.
	///
	/// If the bond type is `Create`, `Staking::bond` is called, and `who`
	/// is allowed to be killed. Otherwise, `Staking::bond_extra` is called and `who`
	/// cannot be killed.
	///
	/// Returns `Ok(points_issues)`, `Err` otherwise.
	pub(crate) fn try_bond_funds(
		&mut self,
		who: &T::AccountId,
		amount: BalanceOf<T>,
		ty: BondType,
	) -> Result<BalanceOf<T>, DispatchError> {
		// Cache the value
		let bonded_account = self.bonded_account();
		CurrencyOf::<T>::transfer(
			who,
			&bonded_account,
			amount,
			match ty {
				BondType::Create => ExistenceRequirement::AllowDeath,
				BondType::Later => ExistenceRequirement::KeepAlive,
			},
		)?;
		// We must calculate the points issued *before* we bond who's funds, else points:balance
		// ratio will be wrong.
		let points_issued = self.issue(amount)?;

		match ty {
			BondType::Create => T::Staking::bond(&bonded_account, amount, &self.reward_account())?,
			// The pool should always be created in such a way its in a state to bond extra, but if
			// the active balance is slashed below the minimum bonded or the account cannot be
			// found, we exit early.
			BondType::Later => T::Staking::bond_extra(&bonded_account, amount)?,
		}

		Ok(points_issued)
	}

	// Set the state of `self`, and deposit an event if the state changed. State should never be set
	// directly in in order to ensure a state change event is always correctly deposited.
	pub(crate) fn set_state(&mut self, state: PoolState) {
		if self.state != state {
			self.state = state;
			Pallet::<T>::deposit_event(Event::<T>::StateChanged {
				pool_id: self.id,
				new_state: state,
			});
		};
	}
}

/// An unbonding pool. This is always mapped with an era.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, DefaultNoBound, RuntimeDebugNoBound)]
#[cfg_attr(feature = "std", derive(Clone, PartialEq, Eq))]
#[codec(mel_bound(T: Config))]
#[scale_info(skip_type_params(T))]
pub struct UnbondPool<T: Config> {
	/// The points in this pool.
	pub(crate) points: BalanceOf<T>,
	/// The funds in the pool.
	pub(crate) balance: BalanceOf<T>,
}

impl<T: Config> UnbondPool<T> {
	pub(crate) fn balance_to_point(&self, new_funds: BalanceOf<T>) -> BalanceOf<T> {
		Pallet::<T>::balance_to_point(self.balance, self.points, new_funds)
	}

	pub(crate) fn point_to_balance(&self, points: BalanceOf<T>) -> BalanceOf<T> {
		Pallet::<T>::point_to_balance(self.balance, self.points, points)
	}

	/// Issue the equivalent points of `new_funds` into self.
	///
	/// Returns the actual amounts of points issued.
	pub(crate) fn issue(&mut self, new_funds: BalanceOf<T>) -> BalanceOf<T> {
		let new_points = self.balance_to_point(new_funds);
		self.points = self.points.saturating_add(new_points);
		self.balance = self.balance.saturating_add(new_funds);
		new_points
	}

	/// Dissolve some points from the unbonding pool, reducing the balance of the pool
	/// proportionally.
	///
	/// This is the opposite of `issue`.
	///
	/// Returns the actual amount of `Balance` that was removed from the pool.
	pub(crate) fn dissolve(&mut self, points: BalanceOf<T>) -> BalanceOf<T> {
		let balance_to_unbond = self.point_to_balance(points);
		self.points = self.points.saturating_sub(points);
		self.balance = self.balance.saturating_sub(balance_to_unbond);

		balance_to_unbond
	}
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, DefaultNoBound, RuntimeDebugNoBound)]
#[cfg_attr(feature = "std", derive(Clone, PartialEq))]
#[codec(mel_bound(T: Config))]
#[scale_info(skip_type_params(T))]
pub struct SubPools<T: Config> {
	/// A general, era agnostic pool of funds that have fully unbonded. The pools
	/// of `Self::with_era` will lazily be merged into into this pool if they are
	/// older then `current_era - TotalUnbondingPools`.
	pub(crate) no_era: UnbondPool<T>,
	/// Map of era in which a pool becomes unbonded in => unbond pools.
	pub(crate) with_era: BoundedBTreeMap<EraIndex, UnbondPool<T>, TotalUnbondingPools<T>>,
}

impl<T: Config> SubPools<T> {
	/// Merge the oldest `with_era` unbond pools into the `no_era` unbond pool.
	///
	/// This is often used whilst getting the sub-pool from storage, thus it consumes and returns
	/// `Self` for ergonomic purposes.
	pub(crate) fn maybe_merge_pools(mut self, current_era: EraIndex) -> Self {
		// Ex: if `TotalUnbondingPools` is 5 and current era is 10, we only want to retain pools
		// 6..=10. Note that in the first few eras where `checked_sub` is `None`, we don't remove
		// anything.
		if let Some(newest_era_to_remove) =
			current_era.checked_sub(T::PostUnbondingPoolsWindow::get())
		{
			self.with_era.retain(|k, v| {
				if *k > newest_era_to_remove {
					// keep
					true
				} else {
					// merge into the no-era pool
					self.no_era.points = self.no_era.points.saturating_add(v.points);
					self.no_era.balance = self.no_era.balance.saturating_add(v.balance);
					false
				}
			});
		}

		self
	}
}

/// The maximum amount of eras an unbonding pool can exist prior to being merged with the
/// `no_era` pool. This is guaranteed to at least be equal to the staking `UnbondingDuration`. For
/// improved UX [`Config::PostUnbondingPoolsWindow`] should be configured to a non-zero value.
pub struct TotalUnbondingPools<T: Config>(PhantomData<T>);
impl<T: Config> Get<u32> for TotalUnbondingPools<T> {
	fn get() -> u32 {
		// NOTE: this may be dangerous in the scenario bonding_duration gets decreased because
		// we would no longer be able to decode `BoundedBTreeMap::<EraIndex, UnbondPool<T>,
		// TotalUnbondingPools<T>>`, which uses `TotalUnbondingPools` as the bound
		T::Staking::bonding_duration() + T::PostUnbondingPoolsWindow::get()
	}
}

/// Pool commission.
///
/// The pool `admin` can set commission configuration after pool creation. By default, all
/// commission values are `None`. Pool `admin` can also set `max` and `change_rate` configurations
/// before setting an initial `current` commission.
///
/// `current` is a tuple of the commission percentage and payee of commission. `throttle_from`
/// keeps track of which block `current` was last updated. A `max` commission value can only be
/// decreased after the initial value is set, to prevent commission from repeatedly increasing.
///
/// An optional commission `change_rate` allows the pool to set strict limits to how much commission
/// can change in each update, and how often updates can take place.
#[derive(
	Encode, Decode, DefaultNoBound, MaxEncodedLen, TypeInfo, DebugNoBound, PartialEq, Copy, Clone,
)]
#[codec(mel_bound(T: Config))]
#[scale_info(skip_type_params(T))]
pub struct Commission<T: Config> {
	/// Optional commission rate of the pool
	pub current: Option<Perbill>,
	/// Optional maximum commission that can be set by the pool `admin`. Once set, this value can
	/// only be updated to a decreased value.
	pub max: Option<Perbill>,
	/// Optional configuration around how often commission can be updated, and when the last
	/// commission update took place.
	pub change_rate: Option<CommissionChangeRate<BlockNumberFor<T>>>,
	/// The block from where throttling should be checked from. This value will be updated on all
	/// commission updates and when setting an initial `change_rate`.
	pub throttle_from: Option<BlockNumberFor<T>>,
}

impl<T: Config> Commission<T> {
	/// Returns true if the current commission updating to `to` would exhaust the change rate
	/// limits.
	///
	/// A commission update will be throttled (disallowed) if:
	/// 1. not enough blocks have passed since the `throttle_from` block, if exists, or
	/// 2. the new commission is greater than the maximum allowed increase.
	pub fn throttling(&self, to: &Perbill) -> bool {
		if let Some(t) = self.change_rate.as_ref() {
			let commission_as_percent = self.current.as_ref().copied().unwrap_or(Perbill::zero());

			// throttle if `to` is decreased by more than `max_delta`
			if *to < commission_as_percent {
				return commission_as_percent.saturating_sub(*to) > t.max_delta;
			}

			// throttle if `to` is increased by more than `max_delta`
			if *to > commission_as_percent {
				return (*to).saturating_sub(commission_as_percent) > t.max_delta;
			}

			// Test for `min_delay` throttling.
			//
			// Note: matching `None` is defensive only. `throttle_from` should always exist where
			// `change_rate` has already been set, so this scenario should never happen.
			return self.throttle_from.map_or_else(
				|| {
					defensive!("throttle_from should exist if change_rate is set");
					true
				},
				|f| {
					// if `min_delay` is zero (no delay), not throttling.
					if t.min_delay == Zero::zero() {
						false
					} else {
						// throttling if blocks passed is less than `min_delay`.
						let blocks_surpassed =
							<frame_system::Pallet<T>>::block_number().saturating_sub(f);
						blocks_surpassed < t.min_delay
					}
				},
			);
		}
		false
	}

	/// Gets the pool's current commission rate, or returns Perbill::zero if none is set.
	/// Bounded to global max if current is greater than `GlobalMaxCommission`.
	pub fn current(&self) -> Perbill {
		self.current
			.as_ref()
			.map_or(Perbill::zero(), |c| *c)
			.min(GlobalMaxCommission::<T>::get().unwrap_or(Bounded::max_value()))
	}

	/// Set the pool's commission.
	///
	/// Update commission based on `current`. If a `None` is supplied, allow the commission to be
	/// removed without any change rate restrictions. Updates `throttle_from` to the current block.
	/// If the supplied commission is zero, `None` will be inserted and `payee` will be ignored.
	pub fn try_update_current(&mut self, current: &Option<Perbill>) -> DispatchResult {
		self.current = match current {
			None => None,
			Some(commission) => {
				ensure!(!self.throttling(commission), Error::<T>::CommissionChangeThrottled);
				ensure!(
					self.max.map_or(true, |m| commission <= &m),
					Error::<T>::CommissionExceedsMaximum
				);
				ensure!(
					*commission <= GlobalMaxCommission::<T>::get().unwrap_or(Bounded::max_value()),
					Error::<T>::CommissionExceedsMaximum
				);

				if commission.is_zero() {
					None
				} else {
					Some(*commission)
				}
			},
		};
		self.register_update();
		Ok(())
	}

	/// Set the pool's maximum commission.
	///
	/// The pool's maximum commission can initially be set to any value, and only smaller values
	/// thereafter. If larger values are attempted, this function will return a dispatch error.
	///
	/// If `current.0` is larger than the updated max commission value, `current.0` will also be
	/// updated to the new maximum. This will also register a `throttle_from` update.
	/// A `PoolCommissionUpdated` event is triggered if `current.0` is updated.
	pub fn try_update_max(&mut self, pool_id: PoolId, new_max: Perbill) -> DispatchResult {
		ensure!(
			new_max <= GlobalMaxCommission::<T>::get().unwrap_or(Bounded::max_value()),
			Error::<T>::CommissionExceedsMaximum
		);

		if let Some(old) = self.max.as_mut() {
			if new_max > *old {
				return Err(Error::<T>::MaxCommissionRestricted.into());
			}
			*old = new_max;
		} else {
			self.max = Some(new_max)
		};
		let updated_current = self
			.current
			.as_mut()
			.map(|c| {
				let u = *c > new_max;
				*c = (*c).min(new_max);
				u
			})
			.unwrap_or(false);

		if updated_current {
			if self.current.as_ref().is_some() {
				Pallet::<T>::deposit_event(Event::<T>::CommissionUpdated {
					pool_id,
					current: Some(new_max),
				});
			}
			self.register_update();
		}
		Ok(())
	}

	/// Set the pool's commission `change_rate`.
	///
	/// Once a change rate configuration has been set, only more restrictive values can be set
	/// thereafter. These restrictions translate to increased `min_delay` values and decreased
	/// `max_delta` values.
	///
	/// Update `throttle_from` to the current block upon setting change rate for the first time, so
	/// throttling can be checked from this block.
	pub fn try_update_change_rate(
		&mut self,
		change_rate: CommissionChangeRate<BlockNumberFor<T>>,
	) -> DispatchResult {
		ensure!(!&self.less_restrictive(&change_rate), Error::<T>::CommissionChangeRateNotAllowed);

		if self.change_rate.is_none() {
			self.register_update();
		}
		self.change_rate = Some(change_rate);
		Ok(())
	}

	/// Updates a commission's `throttle_from` field to the current block.
	fn register_update(&mut self) {
		self.throttle_from = Some(<frame_system::Pallet<T>>::block_number());
	}

	/// Checks whether a change rate is less restrictive than the current change rate, if any.
	///
	/// No change rate will always be less restrictive than some change rate, so where no
	/// `change_rate` is currently set, `false` is returned.
	fn less_restrictive(&self, new: &CommissionChangeRate<BlockNumberFor<T>>) -> bool {
		self.change_rate
			.as_ref()
			.map(|c| new.max_delta > c.max_delta || new.min_delay < c.min_delay)
			.unwrap_or(false)
	}
}

/// Pool commission change rate preferences.
///
/// The pool admin is able to set a commission change rate for their pool. A commission change rate
/// consists of 2 values; (1) the maximum allowed commission change, and (2) the minimum amount of
/// blocks that must elapse before commission updates are allowed again.
///
/// Commission change rates are not applied to decreases in commission.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, PartialEq, Copy, Clone)]
pub struct CommissionChangeRate<BlockNumber> {
	/// The maximum amount the commission can be updated by per `min_delay` period.
	pub max_delta: Perbill,
	/// How often an update can take place.
	pub min_delay: BlockNumber,
}

/// Bonus type
#[derive(Debug)]
pub enum BonusType {
	/// Base bonus, rewarded regardless of the pool duration
	Base,
	/// Weighted bonus, rewarded based on the pool duration and main reward\
	Weighted,
}

/// Information about a pool needed for bonus calculation
#[derive(Default, Debug, Clone)]
pub(crate) struct PoolInfo<Balance: AtLeast32BitUnsigned> {
	/// Duration of the pool
	pub duration: EraIndex,
	/// The amount initially in the reward account
	pub initial_reward_balance: Balance,
	/// The reward received for `era`
	pub reward: Balance,
	/// The weight scaled to the reward
	pub real_weight: Balance,
}

/// Payment info for a commission
#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct CommissionPayment<AccountId, Balance> {
	/// The account that received the commission
	pub beneficiary: AccountId,
	/// The amount paid
	pub amount: Balance,
}

/// Explicit [`CommissionPayment`]
pub type CommissionPaymentOf<T> =
	CommissionPayment<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, PartialEq, Copy, Clone)]
/// Staking information representing annual inflation rate, collator and treasury payout cut
pub struct StakingInfo {
	pub annual_inflation_rate: Perbill,
	pub collator_payout_cut: Perbill,
	pub treasury_payout_cut: Perbill,
}

/// Tracks how many payouts have occurred in an era. Used in [`EraPayoutInfo`] storage.
#[derive(Encode, Decode, RuntimeDebug, PartialEq, Clone, TypeInfo, MaxEncodedLen)]
pub struct EraPayout {
	/// The era it is currently tracking
	#[codec(compact)]
	pub era: EraIndex,
	/// The number of payouts that have been made in `era`
	#[codec(compact)]
	pub payout_count: u32,
	/// If payouts were processed for `era`
	pub payouts_processed: bool,
	/// The percentage of the number of validators that need to make a payment in the era before
	/// the payments can be processed. For example, if there are 10 validators, and it's 80%, then
	/// 8 of the validators are required to make payments before `process_payouts` can be called.
	pub required_payments_percent: Perbill,
}

impl Default for EraPayout {
	fn default() -> Self {
		Self {
			era: 0,
			payout_count: 0,
			payouts_processed: false,
			required_payments_percent: Perbill::from_percent(100),
		}
	}
}
