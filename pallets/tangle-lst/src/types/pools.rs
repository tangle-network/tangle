use super::*;

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
pub enum BondType {
	/// Someone is bonding into the pool upon creation.
	Create,
	/// Someone is adding more funds later to this pool.
	Later,
}

/// How to increase the bond of a member.
#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq, TypeInfo)]
pub enum BondExtra<Balance> {
	/// Take from the free balance.
	FreeBalance(Balance),
	/// Take the entire amount from the accumulated rewards.
	Rewards,
}

/// The type of account being created.
#[derive(Encode, Decode)]
pub enum AccountType {
	Bonded,
	Reward,
}

/// The permission a pool member can set for other accounts to claim rewards on their behalf.
#[derive(Encode, Decode, MaxEncodedLen, Clone, Copy, Debug, PartialEq, Eq, TypeInfo)]
pub enum ClaimPermission {
	/// Only the pool member themself can claim their rewards.
	Permissioned,
	/// Anyone can compound rewards on a pool member's behalf.
	PermissionlessCompound,
	/// Anyone can withdraw rewards on a pool member's behalf.
	PermissionlessWithdraw,
	/// Anyone can withdraw and compound rewards on a pool member's behalf.
	PermissionlessAll,
}

impl ClaimPermission {
	pub fn can_bond_extra(&self) -> bool {
		matches!(self, ClaimPermission::PermissionlessAll | ClaimPermission::PermissionlessCompound)
	}

	pub fn can_claim_payout(&self) -> bool {
		matches!(self, ClaimPermission::PermissionlessAll | ClaimPermission::PermissionlessWithdraw)
	}
}

impl Default for ClaimPermission {
	fn default() -> Self {
		Self::Permissioned
	}
}

/// A member in a pool.
#[derive(
	Encode,
	Decode,
	MaxEncodedLen,
	TypeInfo,
	RuntimeDebugNoBound,
	CloneNoBound,
	frame_support::PartialEqNoBound,
)]
#[cfg_attr(feature = "std", derive(DefaultNoBound))]
#[codec(mel_bound(T: Config))]
#[scale_info(skip_type_params(T))]
pub struct PoolMember<T: Config> {
	/// The identifier of the pool to which `who` belongs.
	pub pool_id: PoolId,
	/// The quantity of points this member has in the bonded pool or in a sub pool if
	/// `Self::unbonding_era` is some.
	pub points: BalanceOf<T>,
	/// The reward counter at the time of this member's last payout claim.
	pub last_recorded_reward_counter: T::RewardCounter,
	/// The eras in which this member is unbonding, mapped from era index to the number of
	/// points scheduled to unbond in the given era.
	pub unbonding_eras: BoundedBTreeMap<EraIndex, BalanceOf<T>, T::MaxUnbonding>,
}

impl<T: Config> PoolMember<T> {
	/// The pending rewards of this member.
	pub fn pending_rewards(
		&self,
		current_reward_counter: T::RewardCounter,
	) -> Result<BalanceOf<T>, Error<T>> {
		// accuracy note: Reward counters are `FixedU128` with base of 10^18. This value is being
		// multiplied by a point. The worse case of a point is 10x the granularity of the balance
		// (10x is the common configuration of `MaxPointsToBalance`).
		//
		// Assuming roughly the current issuance of polkadot (12,047,781,394,999,601,455, which is
		// 1.2 * 10^9 * 10^10 = 1.2 * 10^19), the worse case point value is around 10^20.
		//
		// The final multiplication is:
		//
		// rc * 10^20 / 10^18 = rc * 100
		//
		// the implementation of `multiply_by_rational_with_rounding` shows that it will only fail
		// if the final division is not enough to fit in u128. In other words, if `rc * 100` is more
		// than u128::max. Given that RC is interpreted as reward per unit of point, and unit of
		// point is equal to balance (normally), and rewards are usually a proportion of the points
		// in the pool, the likelihood of rc reaching near u128::MAX is near impossible.

		(current_reward_counter.defensive_saturating_sub(self.last_recorded_reward_counter))
			.checked_mul_int(self.active_points())
			.ok_or(Error::<T>::OverflowRisk)
	}

	/// Active balance of the member.
	///
	/// This is derived from the ratio of points in the pool to which the member belongs to.
	/// Might return different values based on the pool state for the same member and points.
	pub fn active_balance(&self) -> BalanceOf<T> {
		if let Some(pool) = BondedPool::<T>::get(self.pool_id).defensive() {
			pool.points_to_balance(self.points)
		} else {
			Zero::zero()
		}
	}

	/// Total balance of the member, both active and unbonding.
	/// Doesn't mutate state.
	#[cfg(any(feature = "try-runtime", feature = "fuzzing", test, debug_assertions))]
	pub fn total_balance(&self) -> BalanceOf<T> {
		let pool = BondedPool::<T>::get(self.pool_id).unwrap();
		let active_balance = pool.points_to_balance(self.active_points());

		let sub_pools = match SubPoolsStorage::<T>::get(self.pool_id) {
			Some(sub_pools) => sub_pools,
			None => return active_balance,
		};

		let unbonding_balance = self.unbonding_eras.iter().fold(
			BalanceOf::<T>::zero(),
			|accumulator, (era, unlocked_points)| {
				// if the `SubPools::with_era` has already been merged into the
				// `SubPools::no_era` use this pool instead.
				let era_pool = sub_pools.with_era.get(era).unwrap_or(&sub_pools.no_era);
				accumulator + (era_pool.point_to_balance(*unlocked_points))
			},
		);

		active_balance + unbonding_balance
	}

	/// Total points of this member, both active and unbonding.
	pub fn total_points(&self) -> BalanceOf<T> {
		self.active_points().saturating_add(self.unbonding_points())
	}

	/// Active points of the member.
	pub fn active_points(&self) -> BalanceOf<T> {
		self.points
	}

	/// Inactive points of the member, waiting to be withdrawn.
	pub fn unbonding_points(&self) -> BalanceOf<T> {
		self.unbonding_eras
			.as_ref()
			.iter()
			.fold(BalanceOf::<T>::zero(), |acc, (_, v)| acc.saturating_add(*v))
	}

	/// Try and unbond `points_dissolved` from self, and in return mint `points_issued` into the
	/// corresponding `era`'s unlock schedule.
	///
	/// In the absence of slashing, these two points are always the same. In the presence of
	/// slashing, the value of points in different pools varies.
	///
	/// Returns `Ok(())` and updates `unbonding_eras` and `points` if success, `Err(_)` otherwise.
	pub fn try_unbond(
		&mut self,
		points_dissolved: BalanceOf<T>,
		points_issued: BalanceOf<T>,
		unbonding_era: EraIndex,
	) -> Result<(), Error<T>> {
		if let Some(new_points) = self.points.checked_sub(&points_dissolved) {
			match self.unbonding_eras.get_mut(&unbonding_era) {
				Some(already_unbonding_points) => {
					*already_unbonding_points =
						already_unbonding_points.saturating_add(points_issued)
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
			self.points = new_points;
			Ok(())
		} else {
			Err(Error::<T>::MinimumBondNotMet)
		}
	}

	/// Withdraw any funds in [`Self::unbonding_eras`] who's deadline in reached and is fully
	/// unlocked.
	///
	/// Returns a a subset of [`Self::unbonding_eras`] that got withdrawn.
	///
	/// Infallible, noop if no unbonding eras exist.
	pub fn withdraw_unlocked(
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
	pub root: Option<AccountId>,
	/// Can select which validators the pool nominates.
	pub nominator: Option<AccountId>,
	/// Can change the pools state and kick members if the pool is blocked.
	pub bouncer: Option<AccountId>,
}

// A pool's possible commission claiming permissions.
#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum CommissionClaimPermission<AccountId> {
	Permissionless,
	Account(AccountId),
}
