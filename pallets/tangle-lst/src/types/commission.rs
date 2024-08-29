use super::*;

// A pool's possible commission claiming permissions.
#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum CommissionClaimPermission<AccountId> {
	Permissionless,
	Account(AccountId),
}

/// Pool commission.
///
/// The pool `root` can set commission configuration after pool creation. By default, all commission
/// values are `None`. Pool `root` can also set `max` and `change_rate` configurations before
/// setting an initial `current` commission.
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
	/// Optional commission rate of the pool along with the account commission is paid to.
	pub current: Option<(Perbill, T::AccountId)>,
	/// Optional maximum commission that can be set by the pool `root`. Once set, this value can
	/// only be updated to a decreased value.
	pub max: Option<Perbill>,
	/// Optional configuration around how often commission can be updated, and when the last
	/// commission update took place.
	pub change_rate: Option<CommissionChangeRate<BlockNumberFor<T>>>,
	/// The block from where throttling should be checked from. This value will be updated on all
	/// commission updates and when setting an initial `change_rate`.
	pub throttle_from: Option<BlockNumberFor<T>>,
	// Whether commission can be claimed permissionlessly, or whether an account can claim
	// commission. `Root` role can always claim.
	pub claim_permission: Option<CommissionClaimPermission<T::AccountId>>,
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
			let commission_as_percent =
				self.current.as_ref().map(|(x, _)| *x).unwrap_or(Perbill::zero());

			// do not throttle if `to` is the same or a decrease in commission.
			if *to <= commission_as_percent {
				return false;
			}
			// Test for `max_increase` throttling.
			//
			// Throttled if the attempted increase in commission is greater than `max_increase`.
			if (*to).saturating_sub(commission_as_percent) > t.max_increase {
				return true;
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

	/// Gets the pool's current commission, or returns Perbill::zero if none is set.
	/// Bounded to global max if current is greater than `GlobalMaxCommission`.
	pub fn current(&self) -> Perbill {
		self.current
			.as_ref()
			.map_or(Perbill::zero(), |(c, _)| *c)
			.min(GlobalMaxCommission::<T>::get().unwrap_or(Bounded::max_value()))
	}

	/// Set the pool's commission.
	///
	/// Update commission based on `current`. If a `None` is supplied, allow the commission to be
	/// removed without any change rate restrictions. Updates `throttle_from` to the current block.
	/// If the supplied commission is zero, `None` will be inserted and `payee` will be ignored.
	pub fn try_update_current(
		&mut self,
		current: &Option<(Perbill, T::AccountId)>,
	) -> DispatchResult {
		self.current = match current {
			None => None,
			Some((commission, payee)) => {
				ensure!(!self.throttling(commission), Error::<T>::CommissionChangeThrottled);
				ensure!(
					commission <= &GlobalMaxCommission::<T>::get().unwrap_or(Bounded::max_value()),
					Error::<T>::CommissionExceedsGlobalMaximum
				);
				ensure!(
					self.max.map_or(true, |m| commission <= &m),
					Error::<T>::CommissionExceedsMaximum
				);
				if commission.is_zero() {
					None
				} else {
					Some((*commission, payee.clone()))
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
			Error::<T>::CommissionExceedsGlobalMaximum
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
			.map(|(c, _)| {
				let u = *c > new_max;
				*c = (*c).min(new_max);
				u
			})
			.unwrap_or(false);

		if updated_current {
			if let Some((_, payee)) = self.current.as_ref() {
				Pallet::<T>::deposit_event(Event::<T>::PoolCommissionUpdated {
					pool_id,
					current: Some((new_max, payee.clone())),
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
	/// `max_increase` values.
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
	pub fn register_update(&mut self) {
		self.throttle_from = Some(<frame_system::Pallet<T>>::block_number());
	}

	/// Checks whether a change rate is less restrictive than the current change rate, if any.
	///
	/// No change rate will always be less restrictive than some change rate, so where no
	/// `change_rate` is currently set, `false` is returned.
	pub fn less_restrictive(&self, new: &CommissionChangeRate<BlockNumberFor<T>>) -> bool {
		self.change_rate
			.as_ref()
			.map(|c| new.max_increase > c.max_increase || new.min_delay < c.min_delay)
			.unwrap_or(false)
	}
}

/// Pool commission change rate preferences.
///
/// The pool root is able to set a commission change rate for their pool. A commission change rate
/// consists of 2 values; (1) the maximum allowed commission change, and (2) the minimum amount of
/// blocks that must elapse before commission updates are allowed again.
///
/// Commission change rates are not applied to decreases in commission.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, PartialEq, Copy, Clone)]
pub struct CommissionChangeRate<BlockNumber> {
	/// The maximum amount the commission can be updated by per `min_delay` period.
	pub max_increase: Perbill,
	/// How often an update can take place.
	pub min_delay: BlockNumber,
}
