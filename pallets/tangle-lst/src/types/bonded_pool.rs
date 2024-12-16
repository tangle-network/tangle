use super::*;

/// Pool permissions and state
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, DebugNoBound, Clone)]
#[codec(mel_bound(T: Config))]
#[scale_info(skip_type_params(T))]
pub struct BondedPoolInner<T: Config> {
	/// The commission rate of the pool.
	pub commission: Commission<T>,
	/// See [`PoolRoles`].
	pub roles: PoolRoles<T::AccountId>,
	/// The current state of the pool.
	pub state: PoolState,
	/// pool metadata
	pub metadata: PoolMetadata<T>,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, DebugNoBound, Clone)]
#[codec(mel_bound(T: Config))]
#[scale_info(skip_type_params(T))]
pub struct PoolMetadata<T: Config> {
	/// pool name
	pub name: Option<BoundedVec<u8, T::MaxNameLength>>,
	/// pool icon
	pub icon: Option<BoundedVec<u8, T::MaxIconLength>>,
}

/// A wrapper for bonded pools, with utility functions.
///
/// The main purpose of this is to wrap a [`BondedPoolInner`], with the account
/// + id of the pool, for easier access.
#[derive(RuntimeDebugNoBound)]
#[cfg_attr(feature = "std", derive(Clone))]
pub struct BondedPool<T: Config> {
	/// The identifier of the pool.
	pub id: PoolId,
	/// The inner fields.
	pub inner: BondedPoolInner<T>,
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
	pub fn new(
		id: PoolId,
		roles: PoolRoles<T::AccountId>,
		name: Option<BoundedVec<u8, T::MaxNameLength>>,
		icon: Option<BoundedVec<u8, T::MaxIconLength>>,
	) -> Self {
		Self {
			id,
			inner: BondedPoolInner {
				commission: Commission::default(),
				roles,
				state: PoolState::Open,
				metadata: PoolMetadata { name, icon },
			},
		}
	}

	/// Get [`Self`] from storage. Returns `None` if no entry for `pool_account` exists.
	pub fn get(id: PoolId) -> Option<Self> {
		BondedPools::<T>::try_get(id).ok().map(|inner| Self { id, inner })
	}

	/// Get the bonded account id of this pool.
	pub fn bonded_account(&self) -> T::AccountId {
		Pallet::<T>::create_bonded_account(self.id)
	}

	/// Get the reward account id of this pool.
	pub fn reward_account(&self) -> T::AccountId {
		Pallet::<T>::create_reward_account(self.id)
	}

	/// Consume self and put into storage.
	pub fn put(self) {
		BondedPools::<T>::insert(self.id, self.inner);
	}

	pub fn points(&self) -> BalanceOf<T> {
		// the total points of the pool is the total supply of LST token of the pool
		T::Fungibles::total_issuance(self.id.into())
	}

	/// Consume self and remove from storage.
	pub fn remove(self) {
		BondedPools::<T>::remove(self.id);
	}

	/// Convert the given amount of balance to points given the current pool state.
	///
	/// This is often used for bonding and issuing new funds into the pool.
	pub fn balance_to_point(&self, new_funds: BalanceOf<T>) -> BalanceOf<T> {
		let bonded_balance =
			T::Staking::active_stake(&self.bonded_account()).unwrap_or(Zero::zero());
		Pallet::<T>::balance_to_point(bonded_balance, self.points(), new_funds)
	}

	/// Convert the given number of points to balance given the current pool state.
	///
	/// This is often used for unbonding.
	pub fn points_to_balance(&self, points: BalanceOf<T>) -> BalanceOf<T> {
		let bonded_balance =
			T::Staking::active_stake(&self.bonded_account()).unwrap_or(Zero::zero());
		Pallet::<T>::point_to_balance(bonded_balance, self.points(), points)
	}

	/// Issue points to [`Self`] for `new_funds`.
	pub fn issue(&mut self, new_funds: BalanceOf<T>) -> BalanceOf<T> {
		self.balance_to_point(new_funds)
	}

	/// Dissolve some points from the pool i.e. unbond the given amount of points from this pool.
	/// This is the opposite of issuing some funds into the pool.
	///
	/// Mutates self in place, but does not write anything to storage.
	///
	/// Returns the equivalent balance amount that actually needs to get unbonded.
	pub fn dissolve(&mut self, points: BalanceOf<T>) -> BalanceOf<T> {
		// NOTE: do not optimize by removing `balance`. it must be computed before mutating
		// `self.point`.
		self.points_to_balance(points)
	}

	/// The pools balance that is transferable provided it is expendable by staking pallet.
	pub fn transferable_balance(&self) -> BalanceOf<T> {
		let account = self.bonded_account();
		// Note on why we can't use `Currency::reducible_balance`: Since pooled account has a
		// provider (staking pallet), the account can not be set expendable by
		// `pallet-nomination-pool`. This means reducible balance always returns balance preserving
		// ED in the account. What we want though is transferable balance given the account can be
		// dusted.
		T::Currency::free_balance(&account)
			.saturating_sub(T::Staking::active_stake(&account).unwrap_or_default())
	}

	pub fn is_root(&self, who: &T::AccountId) -> bool {
		self.roles.root.as_ref().map_or(false, |root| root == who)
	}

	pub fn is_bouncer(&self, who: &T::AccountId) -> bool {
		self.roles.bouncer.as_ref().map_or(false, |bouncer| bouncer == who)
	}

	pub fn can_update_roles(&self, who: &T::AccountId) -> bool {
		self.is_root(who)
	}

	pub fn can_nominate(&self, who: &T::AccountId) -> bool {
		self.is_root(who) ||
			self.roles.nominator.as_ref().map_or(false, |nominator| nominator == who)
	}

	pub fn can_kick(&self, who: &T::AccountId) -> bool {
		self.state == PoolState::Blocked && (self.is_root(who) || self.is_bouncer(who))
	}

	pub fn can_toggle_state(&self, who: &T::AccountId) -> bool {
		(self.is_root(who) || self.is_bouncer(who)) && !self.is_destroying()
	}

	pub fn can_set_metadata(&self, who: &T::AccountId) -> bool {
		self.is_root(who) || self.is_bouncer(who)
	}

	pub fn can_manage_commission(&self, who: &T::AccountId) -> bool {
		self.is_root(who)
	}

	pub fn can_claim_commission(&self, who: &T::AccountId) -> bool {
		if let Some(permission) = self.commission.claim_permission.as_ref() {
			match permission {
				CommissionClaimPermission::Permissionless => true,
				CommissionClaimPermission::Account(account) => account == who || self.is_root(who),
			}
		} else {
			self.is_root(who)
		}
	}

	pub fn is_destroying(&self) -> bool {
		matches!(self.state, PoolState::Destroying)
	}

	pub fn is_destroying_and_only_depositor(&self, alleged_depositor_points: BalanceOf<T>) -> bool {
		// initial `MinCreateBond` (or more) is what guarantees that the ledger of the pool does not
		// get killed in the staking system, and that it does not fall below `MinimumNominatorBond`,
		// which could prevent other non-depositor members from fully leaving. Thus, all members
		// must withdraw, then depositor can unbond, and finally withdraw after waiting another
		// cycle.
		self.is_destroying() && self.points() == alleged_depositor_points
	}

	/// Whether or not the pool is ok to be in `PoolSate::Open`. If this returns an `Err`, then the
	/// pool is unrecoverable and should be in the destroying state.
	pub fn ok_to_be_open(&self) -> Result<(), DispatchError> {
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
	pub fn ok_to_join(&self) -> Result<(), DispatchError> {
		ensure!(self.state == PoolState::Open, Error::<T>::NotOpen);
		self.ok_to_be_open()?;
		Ok(())
	}

	pub fn ok_to_unbond_with(
		&self,
		caller: &T::AccountId,
		target_account: &T::AccountId,
		active_points: BalanceOf<T>,
		unbonding_points: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		let is_permissioned = caller == target_account;
		let is_depositor = *target_account == self.roles.depositor;
		let is_full_unbond = unbonding_points == active_points;

		let balance_after_unbond = active_points.saturating_sub(unbonding_points);

		// any partial unbonding is only ever allowed if this unbond is permissioned.
		ensure!(
			is_permissioned || is_full_unbond,
			Error::<T>::PartialUnbondNotAllowedPermissionlessly
		);

		// any unbond must comply with the balance condition:
		ensure!(
			is_full_unbond ||
				balance_after_unbond >=
					if is_depositor {
						Pallet::<T>::depositor_min_bond()
					} else {
						MinJoinBond::<T>::get()
					},
			Error::<T>::MinimumBondNotMet
		);

		// additional checks:
		match (is_permissioned, is_depositor) {
			(true, false) => (),
			(true, true) => {
				// permission depositor unbond: if destroying and pool is empty, always allowed,
				// with no additional limits.
				if self.is_destroying_and_only_depositor(balance_after_unbond) {
					// everything good, let them unbond anything.
				} else {
					// depositor cannot fully unbond yet.
					ensure!(!is_full_unbond, Error::<T>::MinimumBondNotMet);
				}
			},
			(false, false) => {
				// If the pool is blocked, then an admin with kicking permissions can remove a
				// member. If the pool is being destroyed, anyone can remove a member
				debug_assert!(is_full_unbond);
				ensure!(
					self.can_kick(caller) || self.is_destroying(),
					Error::<T>::NotKickerOrDestroying
				)
			},
			(false, true) => {
				// the depositor can simply not be unbonded permissionlessly, period.
				return Err(Error::<T>::DoesNotHavePermission.into())
			},
		};

		Ok(())
	}

	/// # Returns
	///
	/// * Ok(()) if [`Call::withdraw_unbonded`] can be called, `Err(DispatchError)` otherwise.
	pub fn ok_to_withdraw_unbonded_with(
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

	/// Bond exactly `amount` from `who`'s funds into this pool. Increases the [`TotalValueLocked`]
	/// by `amount`.
	///
	/// If the bond is [`BondType::Create`], [`Staking::bond`] is called, and `who` is allowed to be
	/// killed. Otherwise, [`Staking::bond_extra`] is called and `who` cannot be killed.
	///
	/// Returns `Ok(points_issues)`, `Err` otherwise.
	pub fn try_bond_funds(
		&mut self,
		who: &T::AccountId,
		amount: BalanceOf<T>,
		ty: BondType,
	) -> Result<BalanceOf<T>, DispatchError> {
		// Cache the value
		let bonded_account = self.bonded_account();
		T::Currency::transfer(
			who,
			&bonded_account,
			amount,
			match ty {
				BondType::Create => ExistenceRequirement::KeepAlive,
				BondType::Later => ExistenceRequirement::AllowDeath,
			},
		)?;
		// We must calculate the points issued *before* we bond who's funds, else points:balance
		// ratio will be wrong.
		let points_issued = self.issue(amount);

		match ty {
			BondType::Create => T::Staking::bond(&bonded_account, amount, &self.reward_account())?,
			// The pool should always be created in such a way its in a state to bond extra, but if
			// the active balance is slashed below the minimum bonded or the account cannot be
			// found, we exit early.
			BondType::Later => T::Staking::bond_extra(&bonded_account, amount)?,
		}
		TotalValueLocked::<T>::mutate(|tvl| {
			tvl.saturating_accrue(amount);
		});

		// finally mint the pool token
		T::Fungibles::mint_into(self.id.into(), who, points_issued)?;

		Ok(points_issued)
	}

	// Set the state of `self`, and deposit an event if the state changed. State should never be set
	// directly in in order to ensure a state change event is always correctly deposited.
	pub fn set_state(&mut self, state: PoolState) {
		if self.state != state {
			self.state = state;
			Pallet::<T>::deposit_event(Event::<T>::StateChanged {
				pool_id: self.id,
				new_state: state,
			});
		};
	}

	/// Withdraw all the funds that are already unlocked from staking for the
	/// [`BondedPool::bonded_account`].
	///
	/// Also reduces the [`TotalValueLocked`] by the difference of the
	/// [`T::Staking::total_stake`] of the [`BondedPool::bonded_account`] that might occur by
	/// [`T::Staking::withdraw_unbonded`].
	///
	/// Returns the result of [`T::Staking::withdraw_unbonded`]
	pub fn withdraw_from_staking(&self, num_slashing_spans: u32) -> Result<bool, DispatchError> {
		let bonded_account = self.bonded_account();

		let prev_total = T::Staking::total_stake(&bonded_account.clone()).unwrap_or_default();
		let outcome = T::Staking::withdraw_unbonded(bonded_account.clone(), num_slashing_spans);
		let diff = prev_total
			.defensive_saturating_sub(T::Staking::total_stake(&bonded_account).unwrap_or_default());
		TotalValueLocked::<T>::mutate(|tvl| {
			tvl.saturating_reduce(diff);
		});
		outcome
	}
}
