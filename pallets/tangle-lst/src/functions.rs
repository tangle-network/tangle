//! Functions for this pallet
use super::*;
use frame_support::{
	dispatch::WithPostDispatchInfo,
	traits::{Imbalance, OnUnbalanced, WithdrawReasons},
};
use pallet_staking::{CurrentEra, Error as StakingError};
use scale_info::prelude::string::String;
#[cfg(any(feature = "try-runtime", feature = "fuzzing", test, debug_assertions))]
use sp_runtime::TryRuntimeError;
use sp_runtime::{
	traits::{CheckedDiv, One},
	ArithmeticError, Perquintill, SaturatedConversion,
};

/// Calculate the real weight of a pool.
/// The weight is calculated as:
/// `normalized_weight = (value - min) / (max - min)`
/// `real_weight = normalized_weight * scale_factor`
pub(crate) fn calculate_real_weight<T: Config>(
	value: EraIndex,
	min: EraIndex,
	max_min_difference: EraIndex,
	scale_factor: BalanceOf<T>,
) -> BalanceOf<T> {
	let normalized_weight = Perbill::from_rational(value.saturating_sub(min), max_min_difference);
	normalized_weight.mul_floor(scale_factor)
}

impl<T: Config> Pallet<T> {
	/// The amount of bond that MUST REMAIN IN BONDED in ALL POOLS.
	///
	/// It is the responsibility of the pool creator to put these funds into the pool initially.
	/// Upon unbond, they can never unbond to a value below this amount.
	///
	/// It is essentially `max { MinNominatorBond, MinCreateBond, MinJoinBond }`, where the former
	/// is coming from the staking pallet and the latter two are configured in this pallet.
	pub fn depositor_min_bond() -> BalanceOf<T> {
		T::Staking::minimum_nominator_bond()
			.max(MinCreateBond::<T>::get())
			.max(MinJoinBond::<T>::get())
			.max(CurrencyOf::<T>::minimum_balance())
	}
	/// Remove everything related to the given bonded pool.
	///
	/// Metadata and all of the sub-pools are also deleted. All accounts are dusted and the leftover
	/// of the reward account is returned to the last member of the pool.
	pub fn dissolve_pool(
		bonded_pool: BondedPool<T>,
		leftover_dest: &T::AccountId,
	) -> DispatchResult {
		let reward_account = bonded_pool.reward_account();
		let bonded_account = bonded_pool.bonded_account();
		let bonus_account = bonded_pool.bonus_account();

		ReversePoolIdLookup::<T>::remove(&bonded_account);
		UsedPoolTokenIds::<T>::remove(bonded_pool.token_id);
		SubPoolsStorage::<T>::remove(bonded_pool.id);

		// Kill accounts from storage by making their balance go below ED. We assume that the
		// accounts have no references that would prevent destruction once we get to this point. We
		// don't work with the system pallet directly, but
		// 1. we drain the reward account and kill it. This account should never have any extra
		// consumers anyway.
		// 2. the bonded account should become a 'killed stash' in the staking system, and all of
		//    its consumers removed.
		debug_assert_eq!(frame_system::Pallet::<T>::consumers(&reward_account), 0);
		debug_assert_eq!(frame_system::Pallet::<T>::consumers(&bonded_account), 0);
		debug_assert_eq!(frame_system::Pallet::<T>::consumers(&bonus_account), 0);
		debug_assert_eq!(
			T::Staking::total_stake(&bonded_account).unwrap_or_default(),
			Zero::zero()
		);

		// This shouldn't fail, but if it does we don't really care
		let remaining = CurrencyOf::<T>::free_balance(&reward_account);
		let _ = CurrencyOf::<T>::transfer(
			&reward_account,
			leftover_dest,
			remaining,
			ExistenceRequirement::AllowDeath,
		);

		let remaining = CurrencyOf::<T>::free_balance(&bonus_account);
		let _ = CurrencyOf::<T>::transfer(
			&bonus_account,
			&T::UnclaimedBalanceReceiver::get(),
			remaining,
			ExistenceRequirement::AllowDeath,
		);

		// NOTE: this is purely defensive.
		CurrencyOf::<T>::make_free_balance_be(&reward_account, Zero::zero());
		CurrencyOf::<T>::make_free_balance_be(&bonded_account, Zero::zero());
		CurrencyOf::<T>::make_free_balance_be(&bonus_account, Zero::zero());

		// remove the token storage if needed
		let collection_id = T::LstCollectionId::get();
		let token_id = bonded_pool.id.saturated_into();
		if !T::FungibleHandler::total_supply_of(collection_id, token_id).is_zero() {
			T::FungibleHandler::burn(T::LstCollectionOwner::get(), collection_id, true)?;
		}

		Self::deposit_event(Event::<T>::Destroyed { pool_id: bonded_pool.id });

		bonded_pool.remove();
		Ok(())
	}

	/// Generates the account id for a pool depending on the `account_type`
	pub fn compute_pool_account_id(id: PoolId, account_type: AccountType) -> T::AccountId {
		T::PalletId::get().into_sub_account_truncating((account_type, id))
	}

	/// The account that holds the deposited `lst`
	pub fn deposit_account_id(id: PoolId) -> T::AccountId {
		Self::compute_pool_account_id(id, AccountType::Bonded)
	}

	/// Calculate the equivalent point of `new_funds` in a pool with `current_balance` and
	/// `current_points`.
	pub(crate) fn balance_to_point(
		current_balance: BalanceOf<T>,
		current_points: BalanceOf<T>,
		new_funds: BalanceOf<T>,
	) -> BalanceOf<T> {
		let to_u256 = T::BalanceToU256::convert;
		let to_balance = T::U256ToBalance::convert;
		match (current_balance.is_zero(), current_points.is_zero()) {
			(_, true) => new_funds.saturating_mul(POINTS_TO_BALANCE_INIT_RATIO.into()),
			(true, false) => {
				// The pool was totally slashed.
				// This is the equivalent of `(current_points / 1) * new_funds`.
				new_funds.saturating_mul(current_points)
			},
			(false, false) => {
				// Equivalent to (current_points / current_balance) * new_funds
				to_balance(
					to_u256(current_points)
						.saturating_mul(to_u256(new_funds))
						// We check for zero above
						.div(to_u256(current_balance)),
				)
			},
		}
	}

	/// Calculate the equivalent balance of `points` in a pool with `current_balance` and
	/// `current_points`.
	pub(crate) fn point_to_balance(
		current_balance: BalanceOf<T>,
		current_points: BalanceOf<T>,
		points: BalanceOf<T>,
	) -> BalanceOf<T> {
		let to_u256 = T::BalanceToU256::convert;
		let to_balance = T::U256ToBalance::convert;
		if current_balance.is_zero() || current_points.is_zero() || points.is_zero() {
			// There is nothing to unbond
			return Zero::zero();
		}

		// Equivalent of (current_balance / current_points) * points
		to_balance(
			to_u256(current_balance)
				.saturating_mul(to_u256(points))
				// We check for zero above
				.div(to_u256(current_points)),
		)
	}

	/// See [`Pallet::create`].
	#[allow(clippy::too_many_arguments)]
	pub(crate) fn do_create(
		who: T::AccountId,
		pool_id: PoolId,
		token_id: TokenIdOf<T>,
		deposit: BalanceOf<T>,
		capacity: BalanceOf<T>,
		duration: EraIndex,
		name: PoolNameOf<T>,
	) -> DispatchResult {
		ensure!(duration >= T::MinDuration::get(), Error::<T>::DurationOutOfBounds);
		ensure!(duration <= T::MaxDuration::get(), Error::<T>::DurationOutOfBounds);
		ensure!(deposit >= Pallet::<T>::depositor_min_bond(), Error::<T>::MinimumBondNotMet);
		ensure!(
			T::FungibleHandler::balance_of(T::PoolCollectionId::get(), token_id, who.clone())
				.is_one(),
			Error::<T>::TokenRequired
		);

		// make sure the pool for token_id doesn't already exist
		ensure!(!UsedPoolTokenIds::<T>::contains_key(token_id), Error::<T>::PoolTokenAlreadyInUse);

		let max_pool_capacity = Self::get_max_pool_capacity(token_id)?;

		// ensure capacity doesnt exceed the limit
		ensure!(
			max_pool_capacity <= T::GlobalMaxCapacity::get(),
			Error::<T>::AttributeCapacityExceedsGlobalCapacity
		);
		ensure!(capacity <= max_pool_capacity, Error::<T>::CapacityExceeded);

		let current_era = CurrentEra::<T>::get().unwrap_or_default();

		let mut bonded_pool = BondedPool::<T> {
			id: pool_id,
			inner: BondedPoolInner { state: PoolState::Open, token_id, capacity },
		};

		let points = bonded_pool.try_bond_funds(&who, deposit, BondType::Create)?;

		// transfer minimum balance to reward account
		CurrencyOf::<T>::transfer(
			&who,
			&bonded_pool.reward_account(),
			CurrencyOf::<T>::minimum_balance(),
			ExistenceRequirement::AllowDeath,
		)?;

		// transfer minimum balance to bonus account
		CurrencyOf::<T>::transfer(
			&who,
			&bonded_pool.bonus_account(),
			CurrencyOf::<T>::minimum_balance(),
			ExistenceRequirement::AllowDeath,
		)?;

		ReversePoolIdLookup::<T>::insert(bonded_pool.bonded_account(), pool_id);
		UsedPoolTokenIds::<T>::insert(token_id, pool_id);

		Self::deposit_event(Event::<T>::Created { creator: who.clone(), pool_id, capacity });

		Self::deposit_event(Event::<T>::Bonded {
			member: Self::deposit_account_id(pool_id),
			pool_id,
			bonded: deposit,
		});

		bonded_pool.put();

		Self::mint_lst(who.clone(), pool_id, points, true)?;

		Ok(())
	}

	/// See [`Pallet::unbond`].
	pub(crate) fn do_unbond(
		who: T::AccountId,
		pool_id: PoolId,
		member_account: T::AccountId,
		unbonding_points: BalanceOf<T>,
	) -> DispatchResult {
		let mut member = UnbondingMembers::<T>::get(pool_id, &member_account).unwrap_or_default();
		let mut bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

		bonded_pool.ok_to_unbond_with(&who, pool_id, &member_account, unbonding_points)?;

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
		member.try_unbond(points_unbonded, unbond_era)?;

		// Burn `lst`
		Self::burn_lst(member_account.clone(), pool_id, unbonding_points)?;

		Self::deposit_event(Event::<T>::Unbonded {
			member: member_account.clone(),
			pool_id,
			points: points_unbonded,
			balance: unbonding_balance,
			era: unbond_era,
		});

		// Now that we know everything has worked write the items to storage.
		SubPoolsStorage::insert(pool_id, sub_pools);
		UnbondingMembers::<T>::insert(pool_id, member_account, member);
		bonded_pool.put();

		Ok(())
	}

	/// See [`Pallet::withdraw_unbonded`].
	pub(crate) fn do_withdraw_unbonded(
		caller: T::AccountId,
		pool_id: PoolId,
		member_account: T::AccountId,
		num_slashing_spans: u32,
	) -> DispatchResultWithPostInfo {
		let is_withdrawing_deposit = member_account == Self::deposit_account_id(pool_id);

		let mut member = UnbondingMembers::<T>::get(pool_id, &member_account)
			.ok_or(Error::<T>::PoolMemberNotFound)?;
		let current_era = T::Staking::current_era();

		let bonded_pool = BondedPool::<T>::get(pool_id)
			.defensive_ok_or::<Error<T>>(DefensiveError::PoolNotFound.into())?;

		let mut sub_pools =
			SubPoolsStorage::<T>::get(pool_id).ok_or(Error::<T>::SubPoolsNotFound)?;

		bonded_pool.ok_to_withdraw_unbonded_with(&caller, &member_account)?;

		// NOTE: must do this after we have done the `ok_to_withdraw_unbonded_other_with` check.
		let withdrawn_points = member.withdraw_unlocked(current_era);
		ensure!(!withdrawn_points.is_empty(), Error::<T>::CannotWithdrawAny);

		// Before calculating the `balance_to_unbond`, we call withdraw unbonded to ensure the
		// `transferrable_balance` is correct.
		let stash_killed =
			T::Staking::withdraw_unbonded(bonded_pool.bonded_account(), num_slashing_spans)?;

		// defensive-only: the depositor puts enough funds into the stash so that it will only
		// be destroyed when they are leaving.
		ensure!(
			!stash_killed || is_withdrawing_deposit,
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
			.min(bonded_pool.transferrable_balance());

		let recipient = if is_withdrawing_deposit {
			// if this is deposit withdrawal, we send the funds to the owner of the pool's token.
			// if the owner is missing, i.e token is burned, funds are sent to the treasury.
			bonded_pool
				.token_owner()
				.map_or(T::UnclaimedBalanceReceiver::get(), |owner| owner)
		} else {
			// If the member is withdrawing their unbonded funds, then we send the funds to the
			// member's account.
			member_account.clone()
		};

		CurrencyOf::<T>::transfer(
			&bonded_pool.bonded_account(),
			&recipient,
			balance_to_unbond,
			ExistenceRequirement::AllowDeath,
		)
		.defensive()?;

		Self::deposit_event(Event::<T>::Withdrawn {
			member: recipient.clone(),
			pool_id,
			points: sum_unlocked_points,
			balance: balance_to_unbond,
		});

		let post_info_weight = if member.total_points(pool_id, member_account.clone()).is_zero() {
			// member being reaped.
			UnbondingMembers::<T>::remove(pool_id, &member_account);

			if is_withdrawing_deposit {
				// also remove the owner of the pool's token.
				if let Some(owner) = bonded_pool.token_owner() {
					UnbondingMembers::<T>::remove(pool_id, &owner);
				}
				Pallet::<T>::dissolve_pool(bonded_pool, &recipient)?;
				None
			} else {
				SubPoolsStorage::<T>::insert(pool_id, sub_pools);
				Some(WeightInfoOf::<T>::withdraw_unbonded_update(num_slashing_spans))
			}
		} else {
			// we certainly don't need to delete any pools, because no one is being removed.
			SubPoolsStorage::<T>::insert(pool_id, sub_pools);
			UnbondingMembers::<T>::insert(pool_id, &member_account, member);
			Some(WeightInfoOf::<T>::withdraw_unbonded_update(num_slashing_spans))
		};

		Ok(post_info_weight.into())
	}

	/// See ['payout_rewards'](Self::payout_rewards)
	pub(crate) fn do_payout_rewards(
		_origin: OriginFor<T>,
		validator_stash: T::AccountId,
		era: EraIndex,
	) -> DispatchResultWithPostInfo {
		// get timing data
		let current_era = CurrentEra::<T>::get().ok_or_else(|| {
			StakingError::<T>::InvalidEraToReward.with_weight(WeightInfoOf::<T>::payout_rewards(0))
		})?;
		let history_depth = T::HistoryDepth::get();

		// checks for exiting early (same checks as pallet_staking::payout_stakers)
		ensure!(
			era <= current_era && era >= current_era.saturating_sub(history_depth),
			StakingError::<T>::InvalidEraToReward.with_weight(WeightInfoOf::<T>::payout_rewards(0))
		);
		ensure!(
			pallet_staking::ErasValidatorReward::<T>::contains_key(era),
			StakingError::<T>::InvalidEraToReward.with_weight(WeightInfoOf::<T>::payout_rewards(0))
		);
		let controller = pallet_staking::Bonded::<T>::get(&validator_stash).ok_or_else(|| {
			StakingError::<T>::NotStash.with_weight(WeightInfoOf::<T>::payout_rewards(0))
		})?;
		ensure!(
			pallet_staking::Ledger::<T>::contains_key(&controller),
			StakingError::<T>::NotController.with_weight(WeightInfoOf::<T>::payout_rewards(0))
		);

		let mut pool_infos = Vec::default();

		// minimum and maximum duration of all pools, used for min-max normalization
		let mut min_duration = EraIndex::MAX;
		let mut max_duration = EraIndex::MIN;

		// NOTE: if we ever have more than 512 nominators for a validator, the page will need to be
		// calculated properly
		let page = 0;

		let exposure =
			pallet_staking::EraInfo::<T>::get_paged_exposure(era, &validator_stash, page)
				.ok_or_else(|| {
					pallet_staking::Error::<T>::InvalidEraToReward
						.with_weight(WeightInfoOf::<T>::payout_rewards(0))
				})?;

		// iterate nominators to do once per era calls and store relevant data in `pool_infos`
		for nominator in exposure.others() {
			if let Some(pool_id) = ReversePoolIdLookup::<T>::get(&nominator.who) {
				// bonus payments are delayed by one era because we need to have all of the
				// reward payments first, and they may occur over multiple transactions

				// use a cfg here because it is difficult to change era for benchmarks
				let check_bonus = {
					cfg_if::cfg_if! {
						if #[cfg(all(feature = "runtime-benchmarks", not(test)))] {
							// always pay bonus for benchmarks when not testing
							true
						} else {
							// normal behavior is to skip zero (because 0 - 1 isn't possible)
							!era.is_zero()
						}
					}
				};

				let mut pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
				let duration = pool.inner.bonus_cycle.duration();

				// update duration bounds if needed
				pool.update_duration_bounds(&mut min_duration, &mut max_duration);

				if check_bonus {
					// end the previous era if bonuses have not been paid already
					pool.maybe_end_era(current_era, era.saturating_sub(One::one()), history_depth)?;
					pool.put();
				}

				// store initial reward balances
				let reward_account = Self::compute_pool_account_id(pool_id, AccountType::Reward);

				pool_infos.push(PoolInfo {
					initial_reward_balance: CurrencyOf::<T>::free_balance(&reward_account),
					duration,
					reward: Zero::zero(),
					real_weight: Zero::zero(),
				});
			}
		}

		// pay stakers from staking pallet
		cfg_if::cfg_if! {
			if #[cfg(test)] {
				// when testing, simulate the payout using mock rewards
				let converted_validator_stash = Decode::decode(&mut &validator_stash.encode()[..]).unwrap();
				crate::tests::payout_rewards::simulate(converted_validator_stash, era);
			} else {
				// when not testing, pay out the stakers
				pallet_staking::Pallet::<T>::payout_stakers(_origin, validator_stash.clone(), era)?;
			}
		}

		// increment payout count
		EraPayoutInfo::<T>::mutate(|info| {
			if info.era == current_era {
				info.payout_count.saturating_inc();
			} else {
				info.era = current_era;
				info.payout_count = 1;
				info.payouts_processed = false;
			}
		});

		// the bonus weights for each pool
		// the bonus for all pools with this validator
		let mut total_bonus = <NegativeImbalanceOf<T> as Imbalance<BalanceOf<T>>>::zero();

		// values that will be used to calculate the bonus weights
		let mut total_rewards = BalanceOf::<T>::zero();
		let mut total_real_weight = BalanceOf::<T>::zero();

		let duration_bounds_difference = max_duration.saturating_sub(min_duration);

		let pool_infos_count = pool_infos.len();

		// iterate pools and transfer part of reward to total_bonus
		let mut pool_count = 0;
		let bonus_percentage = T::BonusPercentage::get();
		for nominator in exposure.others() {
			if let Some(pool_id) = ReversePoolIdLookup::<T>::get(&nominator.who) {
				let pool_info = &mut pool_infos[pool_count];

				// get the reward balance from the reward account
				let reward_account = Self::compute_pool_account_id(pool_id, AccountType::Reward);
				let mut reward_amount = CurrencyOf::<T>::free_balance(&reward_account)
					.saturating_sub(pool_info.initial_reward_balance);

				if !reward_amount.is_zero() {
					// distribute a portion to the bonus
					let to_bonus = bonus_percentage.mul_floor(reward_amount);
					reward_amount.saturating_reduce(to_bonus);

					// imbalance is added to total_bonus
					let imbalance = CurrencyOf::<T>::withdraw(
						&reward_account,
						to_bonus,
						WithdrawReasons::TRANSFER,
						ExistenceRequirement::KeepAlive,
					)?;
					total_bonus.subsume(imbalance);
				}

				// calculate real weight, keeping in mind that duration bounds may be 0
				let real_weight = if duration_bounds_difference.is_zero() {
					// if duration bounds are 0, then all pools have the same weight that adds up to
					// 1
					Perbill::from_rational(1, pool_infos_count as u32).mul_floor(reward_amount)
				} else {
					calculate_real_weight::<T>(
						pool_info.duration,
						min_duration,
						duration_bounds_difference,
						reward_amount,
					)
				};

				pool_info.reward = reward_amount;
				pool_info.real_weight = real_weight;

				// update total values
				total_rewards.saturating_accrue(reward_amount);
				total_real_weight.saturating_accrue(real_weight);

				pool_count.saturating_inc();
			}
		}

		let total_base_bonus_amount =
			T::BaseBonusRewardPercentage::get().mul_floor(total_bonus.peek());

		let (mut total_base_bonus, mut total_weighted_bonus) =
			total_bonus.split(total_base_bonus_amount);

		// need to have this original value since `total_weighted_bonus` will be consumed by each
		// iteration
		let total_weighted_bonus_amount = total_weighted_bonus.peek();

		// calculate base and weighted bonus factors
		let base_bonus_factor = Perbill::from_rational(total_base_bonus_amount, total_rewards);
		let weighted_bonus_factor =
			Perbill::from_rational(total_weighted_bonus_amount, total_real_weight);

		// counter for `pool_bonus_weights`
		let mut pool_index = 0;
		for nominator in exposure.others() {
			if let Some(pool_id) = ReversePoolIdLookup::<T>::get(&nominator.who) {
				let pool_info = &pool_infos[pool_index];

				pool_index.saturating_inc();

				// calculate bonus for this pool
				let (bonus, base_remainder, weighted_remainder) = pool_info.calculate_bonus(
					total_base_bonus,
					total_weighted_bonus,
					base_bonus_factor,
					weighted_bonus_factor,
				);

				let bonus_amount = bonus.peek();
				total_base_bonus = base_remainder;
				total_weighted_bonus = weighted_remainder;

				CurrencyOf::<T>::resolve_into_existing(
					&Self::compute_pool_account_id(pool_id, AccountType::Bonus),
					bonus,
				)
				.ok();

				pool_count.saturating_inc();

				Self::deposit_event(Event::<T>::RewardPaid {
					pool_id,
					era,
					validator_stash: validator_stash.clone(),
					reward: pool_info.reward,
					bonus: bonus_amount,
				});
			}
		}

		let remainder = total_base_bonus.merge(total_weighted_bonus);

		// if any unhandled reward remains, handle it with pallet_staking's reward remainder
		T::RewardRemainder::on_unbalanced(remainder);

		Ok(().into())
	}

	/// See ['process_payouts'](Self::process_payouts)
	pub(crate) fn do_process_payouts(pool_count: u32) -> DispatchResult {
		// check pool count
		ensure!(pool_count >= BondedPools::<T>::count(), Error::<T>::WrongPoolCount);

		let era = CurrentEra::<T>::get().ok_or(StakingError::<T>::InvalidEraToReward)?;
		let history_depth = T::HistoryDepth::get();

		// check and update payout info
		EraPayoutInfo::<T>::try_mutate(|payout_info| -> DispatchResult {
			// use the minimum of actual validator count and ideal validator count
			let validator_count = pallet_staking::Validators::<T>::count()
				.min(pallet_staking::ValidatorCount::<T>::get());
			let required_count = payout_info.required_payments_percent.mul_ceil(validator_count);

			ensure!(
				payout_info.era == era && payout_info.payout_count >= required_count,
				Error::<T>::MissingPayouts
			);
			// error if payouts were already processed
			ensure!(!payout_info.payouts_processed, Error::<T>::PayoutsAlreadyProcessed);
			payout_info.payouts_processed = true;
			Ok(())
		})?;

		// we need to mutate each pool, so first get all of the pool ids
		let pool_ids = BondedPools::<T>::iter_keys().collect::<Vec<_>>();

		// process the payouts for each pool
		for pool_id in pool_ids {
			let mut pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			pool.maybe_end_era(era, era, history_depth)?;
			pool.put();
		}

		Ok(())
	}

	/// Ensure the correctness of the state of this pallet.
	///
	/// This should be valid before or after each state transition of this pallet.
	///
	/// ## Invariants:
	///
	/// First, let's consider pools:
	///
	/// * `BondedPools` and `RewardPools` must all have the EXACT SAME key-set.
	/// * `SubPoolsStorage` must be a subset of the above superset.
	/// * `Metadata` keys must be a subset of the above superset.
	/// * the count of the above set must be less than `MaxPools`.
	///
	/// Then, considering members as well:
	///
	/// * each `BondedPool.member_counter` must be:
	///   - correct (compared to actual count of member who have `.pool_id` this pool)
	///   - less than `MaxPoolMembersPerPool`.
	/// * each `member.pool_id` must correspond to an existing `BondedPool.id` (which implies the
	///   existence of the reward pool as well).
	/// * count of all members must be less than `MaxPoolMembers`.
	///
	/// Then, considering unbonding members:
	///
	/// for each pool:
	///   * sum of the balance that's tracked in all unbonding pools must be the same as the
	///     unbonded balance of the main account, as reported by the staking interface.
	///   * sum of the balance that's tracked in all unbonding pools, plus the bonded balance of the
	///     main account should be less than or qual to the total balance of the main account.
	///
	/// ## Sanity check level
	///
	/// To cater for tests that want to escape parts of these checks, this function is split into
	/// multiple `level`s, where the higher the level, the more checks we performs. So,
	/// `try_state(255)` is the strongest sanity check, and `0` performs no checks.
	#[cfg(any(feature = "try-runtime", feature = "fuzzing", test, debug_assertions))]
	pub fn do_try_state(level: u8) -> Result<(), TryRuntimeError> {
		if level.is_zero() {
			return Ok(());
		}
		// note: while a bit wacky, since they have the same key, even collecting to vec should
		// result in the same set of keys, in the same order.
		let bonded_pools = BondedPools::<T>::iter_keys().collect::<Vec<_>>();

		assert!(SubPoolsStorage::<T>::iter_keys().all(|k| bonded_pools.contains(&k)));

		if level <= 1 {
			return Ok(());
		}

		Ok(())
	}

	/// Fully unbond the shares of `member`, when executed from `origin`.
	///
	/// This is useful for backwards compatibility with the majority of tests that only deal with
	/// full unbonding, not partial unbonding.
	#[cfg(any(feature = "runtime-benchmarks", test))]
	pub fn fully_unbond(
		origin: OriginFor<T>,
		pool_id: PoolId,
		member: T::AccountId,
	) -> DispatchResult {
		let points = Self::member_points(pool_id, member.clone());
		let member_lookup = T::Lookup::unlookup(member);
		Self::unbond(origin, pool_id, member_lookup, points)
	}

	/// Set a new state for the pool.
	///
	/// If a pool is already in the `Destroying` state, then under no condition can its state
	/// change again.
	pub fn set_state(pool_id: PoolId, state: PoolState) -> DispatchResult {
		let mut bonded_pool = BondedPool::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
		ensure!(bonded_pool.state != PoolState::Destroying, Error::<T>::CanNotChangeState);

		if bonded_pool.ok_to_be_open().is_err() && state == PoolState::Destroying {
			// If the pool has bad properties, then anyone can set it as destroying
			bonded_pool.set_state(PoolState::Destroying);
		} else {
			bonded_pool.set_state(state);
		}

		bonded_pool.put();

		Ok(())
	}
}

impl<T: Config> Pallet<T> {
	/// Create pool lst token or mint additional tokens
	///
	/// This function is called when a new amount is bonded to the pool.
	/// It creates a new `lst` token for the pool if the pool is just being created and
	/// it simply mints new `amount` `lst` tokens to the member who bonded, if the pool already
	/// exists.
	///
	/// `token_id` of the new token is the same as the `pool_id`.
	///
	/// # Parameters
	///
	/// - `who` - The member who bonded the amount
	/// - `pool_id` - The pool id
	/// - `amount` - The amount that was bonded
	/// - `is_new_pool` - Whether the new pool is being created. New token is created if true.
	///
	/// # Errors
	///
	/// * [Error::MintParamsCreationFailed] - If the mint params creation failed
	pub(crate) fn mint_lst(
		who: T::AccountId,
		pool_id: PoolId,
		points: BalanceOf<T>,
		is_new_pool: bool,
	) -> DispatchResult {
		let (mint_params, recipient) = {
			if is_new_pool {
				let mint_params =
					<T::FungibleHandler as FungibleHandlerBalances>::MintParams::try_from()
						.map_err(|_| Error::<T>::MintParamsCreationFailed)?;

				(mint_params, Self::deposit_account_id(pool_id))
			} else {
				let mint_params =
					<T::FungibleHandler as FungibleHandlerBalances>::MintParams::try_from(
						DefaultMintParams::Mint {
							token_id: pool_id.saturated_into(),
							amount: points,
							depositor: Some(who.clone()),
						},
					)
					.map_err(|_| Error::<T>::MintParamsCreationFailed)?;
				(mint_params, who)
			}
		};

		<T::FungibleHandler as FungibleHandlerBalances>::mint(
			T::LstCollectionOwner::get(),
			recipient,
			T::LstCollectionId::get(),
			mint_params,
		)?;

		Ok(())
	}

	/// Burn pool lst tokens when pool member exits/unbonds from the pool
	///
	/// Burns `amount` of `lst` tokens from `who`'s account
	///
	/// # Parameters
	///
	/// - `who` - The member who unbonded the amount
	/// - `pool_id` - The pool id
	/// - `amount` - The amount that was unbonded
	///
	/// # Errors
	///
	/// * [Error::BurnParamsCreationFailed] - If the burn params creation failed
	pub(crate) fn burn_lst(
		who: T::AccountId,
		pool_id: PoolId,
		points: BalanceOf<T>,
	) -> DispatchResult {
		// if this is the last remaining tokens of the pool, then destroy the token
		let remaining_pool_tokens =
			<T::FungibleHandler as FungibleHandlerBalances>::total_supply_of(
				T::LstCollectionId::get(),
				pool_id.saturated_into(),
			);
		let destroy_token_storage = remaining_pool_tokens == points;

		<T::FungibleHandler as FungibleHandlerBalances>::burn(
			who,
			T::LstCollectionId::get(),
			true,
		)?;

		Ok(())
	}

	/// Get the amount of points (lst) for `member`
	pub fn member_points(pool_id: PoolId, member: T::AccountId) -> BalanceOf<T> {
		T::FungibleHandler::total_balance_of(
			T::LstCollectionId::get(),
			pool_id.saturated_into(),
			member,
		)
	}
}

// pool functions related to reward payouts
impl<T: Config> BondedPool<T> {
	/// Pay a commission to the pool token holder. Called by [`BondedPool::end_era`]
	pub(crate) fn claim_commission(&self) -> Result<Option<CommissionPaymentOf<T>>, DispatchError> {
		if let Some(commission_rate) = self.commission.current.as_ref() {
			let reward_account = self.reward_account();
			let reward_balance = CurrencyOf::<T>::free_balance(&reward_account);
			let mut commission = commission_rate.mul_floor(reward_balance);
			if reward_balance.saturating_sub(commission) < CurrencyOf::<T>::minimum_balance() {
				commission = reward_balance.saturating_sub(CurrencyOf::<T>::minimum_balance());
			}
			if commission.is_zero() {
				return Ok(None);
			}

			let beneficiary =
				T::FungibleHandler::owner_of(&T::PoolCollectionId::get(), &self.token_id)
					.unwrap_or(T::UnclaimedBalanceReceiver::get());

			// payout the commission
			CurrencyOf::<T>::transfer(
				&reward_account,
				&beneficiary,
				commission,
				ExistenceRequirement::KeepAlive,
			)?;

			Ok(Some(CommissionPayment { beneficiary, amount: commission }))
		} else {
			Ok(None)
		}
	}

	/// Updates `bonuses_paid` and calls [`Self::end_era`] if it should be called
	pub(crate) fn maybe_end_era(
		&mut self,
		current_era: EraIndex,
		era: EraIndex,
		history_depth: u32,
	) -> DispatchResult {
		// trim history early only if we are out of space (it's expensive)
		if self.bonuses_paid.len() >= history_depth as usize - 1 {
			self.bonuses_paid.retain(|&x| x > current_era.saturating_sub(history_depth));
		}

		// check if this is first time pool is being called this era
		if let Err(position) = self.bonuses_paid.binary_search(&era) {
			// insert era into history
			self.bonuses_paid
				.try_insert(position, era)
				.map_err(|_| Error::<T>::BoundExceeded)?;

			// trim history
			self.bonuses_paid.retain(|&x| x >= current_era.saturating_sub(history_depth));

			// actually end the era for the pool
			self.end_era(era)?;
		}
		Ok(())
	}

	/// This should be called once per pool per era. It performs four tasks.
	/// 1. If the pool has reached the end of its cycle, it cycles the pool.
	/// 2. Sends bonus for `era` from the bonus account to the rewards account.
	/// 3. Sends reward commission to the depositor.
	/// 4. It bonds the pool's reward balance.
	pub(crate) fn end_era(&mut self, era: EraIndex) -> DispatchResult {
		// calculate bonus
		let (mut bonus_amount, cycle_ended) = self.maybe_cycle_and_calculate_bonus(era)?;

		// get accounts
		let reward_account_id = self.reward_account();
		let bonus_account_id = self.bonus_account();

		// transfer the bonus balance to rewards account
		if !bonus_amount.is_zero() {
			let bonus_balance = CurrencyOf::<T>::free_balance(&bonus_account_id);
			// ensure that the bonus account will still have the minimum balance
			if bonus_balance.saturating_sub(bonus_amount) < CurrencyOf::<T>::minimum_balance() {
				bonus_amount = bonus_balance.saturating_sub(CurrencyOf::<T>::minimum_balance());
			}
			if !bonus_amount.is_zero() {
				CurrencyOf::<T>::transfer(
					&bonus_account_id,
					&reward_account_id,
					bonus_amount,
					ExistenceRequirement::KeepAlive,
				)?;
			}
		}

		// send commission to pool token holder
		let commission_payment = self.claim_commission()?;

		// transfer rewards to bonded account
		let reward_balance = CurrencyOf::<T>::free_balance(&reward_account_id);
		let minimum_balance = CurrencyOf::<T>::minimum_balance();
		let bond_amount = if reward_balance > minimum_balance {
			let bonded_account_id = self.bonded_account();
			let reward_amount = reward_balance.saturating_sub(minimum_balance);

			if !reward_amount.is_zero() {
				CurrencyOf::<T>::transfer(
					&reward_account_id,
					&bonded_account_id,
					reward_amount,
					ExistenceRequirement::KeepAlive,
				)?;

				// bond the reward
				T::Staking::bond_extra(&bonded_account_id, reward_amount)?;
			}
			reward_amount
		} else {
			Zero::zero()
		};

		if commission_payment.as_ref().map(|x| !x.amount.is_zero()).unwrap_or_default()
			|| !bonus_amount.is_zero()
			|| !bond_amount.is_zero()
			|| cycle_ended
		{
			Pallet::<T>::deposit_event(Event::<T>::EraRewardsProcessed {
				pool_id: self.id,
				era,
				commission: commission_payment,
				bonus: bonus_amount,
				reinvested: bond_amount,
				bonus_cycle_ended: cycle_ended,
			});
		}
		Ok(())
	}
}
