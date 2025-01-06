use super::*;

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
	pub last_recorded_reward_counter: T::RewardCounter,
	/// The last recorded total payouts of the reward pool.
	///
	/// Payouts is essentially income of the pool.
	///
	/// Update criteria is same as that of `last_recorded_reward_counter`.
	pub last_recorded_total_payouts: BalanceOf<T>,
	/// Total amount that this pool has paid out so far to the members.
	pub total_rewards_claimed: BalanceOf<T>,
	/// The amount of commission pending to be claimed.
	pub total_commission_pending: BalanceOf<T>,
	/// The amount of commission that has been claimed.
	pub total_commission_claimed: BalanceOf<T>,
}

impl<T: Config> RewardPool<T> {
	/// Getter for [`RewardPool::last_recorded_reward_counter`].
	pub fn last_recorded_reward_counter(&self) -> T::RewardCounter {
		self.last_recorded_reward_counter
	}

	/// Register some rewards that are claimed from the pool by the members.
	pub fn register_claimed_reward(&mut self, reward: BalanceOf<T>) {
		self.total_rewards_claimed = self.total_rewards_claimed.saturating_add(reward);
	}

	/// Update the recorded values of the reward pool.
	///
	/// This function MUST be called whenever the points in the bonded pool change, AND whenever the
	/// the pools commission is updated. The reason for the former is that a change in pool points
	/// will alter the share of the reward balance among pool members, and the reason for the latter
	/// is that a change in commission will alter the share of the reward balance among the pool.
	pub fn update_records(
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
	pub fn current_reward_counter(
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
		let new_pending_commission = commission.mul_floor(current_payout_balance);
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
	pub fn current_balance(id: PoolId) -> BalanceOf<T> {
		T::Currency::free_balance(&Pallet::<T>::create_reward_account(id))
	}
}

/// An unbonding pool. This is always mapped with an era.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, DefaultNoBound, RuntimeDebugNoBound)]
#[cfg_attr(feature = "std", derive(Clone, PartialEq, Eq))]
#[codec(mel_bound(T: Config))]
#[scale_info(skip_type_params(T))]
pub struct UnbondPool<T: Config> {
	/// The points in this pool.
	pub points: BalanceOf<T>,
	/// The funds in the pool.
	pub balance: BalanceOf<T>,
}

impl<T: Config> UnbondPool<T> {
	pub fn balance_to_point(&self, new_funds: BalanceOf<T>) -> BalanceOf<T> {
		Pallet::<T>::balance_to_point(self.balance, self.points, new_funds)
	}

	pub fn point_to_balance(&self, points: BalanceOf<T>) -> BalanceOf<T> {
		Pallet::<T>::point_to_balance(self.balance, self.points, points)
	}

	/// Issue the equivalent points of `new_funds` into self.
	///
	/// Returns the actual amounts of points issued.
	pub fn issue(&mut self, new_funds: BalanceOf<T>) -> BalanceOf<T> {
		let new_points = self.balance_to_point(new_funds);
		self.points = self.points.saturating_add(new_points);
		self.balance = self.balance.saturating_add(new_funds);
		new_points
	}

	/// Dissolve some points from the unbonding pool, reducing the balance of the pool
	/// proportionally. This is the opposite of `issue`.
	///
	/// Returns the actual amount of `Balance` that was removed from the pool.
	pub fn dissolve(&mut self, points: BalanceOf<T>) -> BalanceOf<T> {
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
	/// of `Self::with_era` will lazily be merged into this pool if they are
	/// older then `current_era - TotalUnbondingPools`.
	pub no_era: UnbondPool<T>,
	/// Map of era in which a pool becomes unbonded in => unbond pools.
	pub with_era: BoundedBTreeMap<EraIndex, UnbondPool<T>, TotalUnbondingPools<T>>,
}

impl<T: Config> SubPools<T> {
	/// Merge the oldest `with_era` unbond pools into the `no_era` unbond pool.
	///
	/// This is often used whilst getting the sub-pool from storage, thus it consumes and returns
	/// `Self` for ergonomic purposes.
	pub fn maybe_merge_pools(mut self, current_era: EraIndex) -> Self {
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

	/// The sum of all unbonding balance, regardless of whether they are actually unlocked or not.
	#[cfg(any(feature = "try-runtime", feature = "fuzzing", test, debug_assertions))]
	pub fn sum_unbonding_balance(&self) -> BalanceOf<T> {
		self.no_era.balance.saturating_add(
			self.with_era
				.values()
				.fold(BalanceOf::<T>::zero(), |acc, pool| acc.saturating_add(pool.balance)),
		)
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
