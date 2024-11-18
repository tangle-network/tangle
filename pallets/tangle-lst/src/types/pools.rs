use super::*;

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
#[codec(mel_bound(T: Config))]
#[scale_info(skip_type_params(T))]
pub struct PoolMember<T: Config> {
	/// The eras in which this member is unbonding, mapped from era index to the number of
	/// points scheduled to unbond in the given era.
	pub unbonding_eras: BoundedBTreeMap<EraIndex, (PoolId, BalanceOf<T>), T::MaxUnbonding>,
}

impl<T: Config> PoolMember<T> {
	/// Inactive points of the member, waiting to be withdrawn.
	pub fn unbonding_points(&self) -> BalanceOf<T> {
		self.unbonding_eras
			.as_ref()
			.iter()
			.fold(BalanceOf::<T>::zero(), |acc, (_, v)| acc.saturating_add(v.1))
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
					.try_insert(*e, (p.1))
					.expect("source map is bounded, this is a subset, will be bounded; qed");
				false
			}
		});
		removed_points
	}

	pub fn get_by_pool_id(&self, current_era: EraIndex, pool_id: PoolId) -> Option<PoolId> {
		self.unbonding_eras
			.get(&current_era)
			.and_then(|(p, b)| if *p == pool_id { Some(*p) } else { None })
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
