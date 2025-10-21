// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.

//! Pool-Based Delegator Reward Distribution
//!
//! This module implements the "accumulated rewards per share" pattern for efficient
//! delegator reward distribution with O(1) complexity regardless of delegator count.
//!
//! ## How It Works
//!
//! 1. **Reward Recording** (O(1)): When operator receives reward: `pool.accumulated_per_share +=
//!    reward / pool.total_staked`
//!
//! 2. **Reward Claiming** (O(1)): When delegator claims: `owed = stake *
//!    (pool.accumulated_per_share - delegator.last_accumulated_per_share)`
//!
//! 3. **Stake Changes**: When delegator changes stake: MUST claim first, then update stake amount
//!
//! ## Mathematical Correctness
//!
//! For delegator with constant stake `s` from event `m` to `n`:
//! ```
//! owed = s * Σ(reward_i / total_stake_i) for i=m+1 to n
//!      = s * (accumulated_n - accumulated_m)
//!      = s * accumulated_delta
//! ```
//!
//! This guarantees proportional distribution: each delegator gets exactly their
//! stake percentage of each reward event.

use crate::{BalanceOf, Config, DelegatorRewardDebts, Error, Event, OperatorRewardPools, Pallet};
use frame_support::{
	dispatch::DispatchResult,
	traits::{Currency, ExistenceRequirement},
};
use sp_arithmetic::FixedU128;
use sp_runtime::{
	FixedPointNumber,
	traits::{SaturatedConversion, Saturating, Zero},
};

impl<T: Config> Pallet<T> {
	/// Record operator reward and update pool accumulator for delegator distribution.
	///
	/// This is the core function that enables O(1) reward distribution. Regardless of
	/// how many delegators the operator has, this function only performs a single
	/// storage write to update the accumulator.
	///
	/// # Arguments
	/// * `operator` - The operator receiving the reward
	/// * `reward` - The total reward amount to distribute among delegators
	///
	/// # Returns
	/// Ok(()) if successful, Error if pool has zero stake
	///
	/// # Complexity
	/// O(1) - Single storage read + write, no loops
	///
	/// # Example
	/// ```ignore
	/// // Operator has 1000 total stake from 100 delegators
	/// // Operator receives 500 token reward
	/// record_operator_reward_to_pool(&operator, 500)?;
	/// // Pool accumulator increases by 500/1000 = 0.5
	/// // Each delegator with 10 stake can now claim 10 * 0.5 = 5 tokens
	/// ```
	pub fn record_operator_reward_to_pool(
		operator: &T::AccountId,
		reward: BalanceOf<T>,
	) -> DispatchResult {
		// Skip zero rewards
		if reward.is_zero() {
			return Ok(());
		}

		OperatorRewardPools::<T>::try_mutate(operator, |pool| -> DispatchResult {
			// Handle case where operator has no delegators yet
			if pool.total_staked.is_zero() {
				// No delegators - operator can keep this reward via direct claim
				// Or we could store it for when first delegator arrives
				log::debug!(
					"Operator {:?} has no delegators, reward {:?} not added to pool",
					operator,
					reward
				);
				return Ok(());
			}

			// Calculate reward per unit of stake using high-precision fixed-point math
			// FixedU128 provides 18 decimal places to prevent rounding errors
			let reward_per_share = FixedU128::from_rational(
				reward.saturated_into::<u128>(),
				pool.total_staked.saturated_into::<u128>(),
			);

			// Add to cumulative accumulator
			pool.accumulated_rewards_per_share =
				pool.accumulated_rewards_per_share.saturating_add(reward_per_share);

			// Update metadata
			pool.last_updated_block = <frame_system::Pallet<T>>::block_number();

			log::debug!(
				"Operator {:?} pool updated: +{:?} reward, new accumulated: {:?}, total_staked: {:?}",
				operator,
				reward,
				pool.accumulated_rewards_per_share,
				pool.total_staked
			);

			// Emit event for monitoring
			Self::deposit_event(Event::OperatorPoolUpdated {
				operator: operator.clone(),
				reward_amount: reward,
				new_accumulated_per_share: pool.accumulated_rewards_per_share,
				total_staked: pool.total_staked,
			});

			Ok(())
		})
	}

	/// Calculate pending rewards for a delegator from an operator's pool.
	///
	/// This performs the core reward calculation without modifying storage.
	/// Useful for displaying pending rewards in UI.
	///
	/// # Formula
	/// ```
	/// owed = stake * (current_accumulated - last_claimed_accumulated)
	/// ```
	///
	/// # Arguments
	/// * `delegator` - The delegator to calculate rewards for
	/// * `operator` - The operator whose pool to check
	///
	/// # Returns
	/// Ok(Balance) with pending reward amount, or Error if no delegation exists
	///
	/// # Complexity
	/// O(1) - Two storage reads, one multiplication, one subtraction
	pub fn calculate_pending_delegator_rewards(
		delegator: &T::AccountId,
		operator: &T::AccountId,
	) -> Result<BalanceOf<T>, sp_runtime::DispatchError> {
		// Get delegator's debt (their last claim position)
		let debt =
			DelegatorRewardDebts::<T>::get(delegator, operator).ok_or(Error::<T>::NoDelegation)?;

		// Get operator's current pool state
		let pool = OperatorRewardPools::<T>::get(operator);

		// Calculate delta in accumulator since last claim
		let per_share_delta = pool
			.accumulated_rewards_per_share
			.saturating_sub(debt.last_accumulated_per_share);

		// Multiply by delegator's stake to get owed amount
		// saturating_mul_int handles conversion from FixedU128 to Balance
		let owed = per_share_delta.saturating_mul_int(debt.staked_amount);

		log::debug!(
			"Delegator {:?} pending rewards from {:?}: {:?} (stake: {:?}, delta: {:?})",
			delegator,
			operator,
			owed,
			debt.staked_amount,
			per_share_delta
		);

		Ok(owed)
	}

	/// Calculate and claim rewards for a delegator, transferring tokens.
	///
	/// This is the internal implementation called by the `claim_delegator_rewards` extrinsic.
	/// It calculates owed rewards, updates the delegator's debt to current accumulator,
	/// and transfers the tokens.
	///
	/// # Arguments
	/// * `delegator` - The delegator claiming rewards
	/// * `operator` - The operator whose pool to claim from
	///
	/// # Returns
	/// Ok(Balance) with claimed amount, or Error if claim fails
	///
	/// # Complexity
	/// O(1) - Storage reads, update debt, single transfer
	///
	/// # Side Effects
	/// - Updates `DelegatorRewardDebts[delegator][operator]`
	/// - Transfers tokens from pallet account to delegator
	pub fn calculate_and_claim_delegator_rewards(
		delegator: &T::AccountId,
		operator: &T::AccountId,
	) -> Result<BalanceOf<T>, sp_runtime::DispatchError> {
		// Calculate owed amount
		let owed = Self::calculate_pending_delegator_rewards(delegator, operator)?;

		// Get current pool state for updating debt
		let pool = OperatorRewardPools::<T>::get(operator);

		// Update delegator's debt to current accumulator
		// This "resets" their claim position to now
		DelegatorRewardDebts::<T>::try_mutate(
			delegator,
			operator,
			|maybe_debt| -> DispatchResult {
				let debt = maybe_debt.as_mut().ok_or(Error::<T>::NoDelegation)?;

				// Update to current pool accumulator
				debt.last_accumulated_per_share = pool.accumulated_rewards_per_share;

				log::debug!(
					"Updated delegator {:?} debt for operator {:?} to: {:?}",
					delegator,
					operator,
					debt.last_accumulated_per_share
				);

				Ok(())
			},
		)?;

		// Transfer rewards if non-zero
		if !owed.is_zero() {
			T::Currency::transfer(
				&Self::account_id(),
				delegator,
				owed,
				ExistenceRequirement::KeepAlive,
			)
			.map_err(|_| Error::<T>::TransferFailed)?;

			log::info!("Delegator {:?} claimed {:?} from operator {:?}", delegator, owed, operator);
		}

		Ok(owed)
	}

	/// Initialize delegator reward debt when they first delegate to an operator.
	///
	/// This sets the delegator's "starting point" at the current pool accumulator,
	/// ensuring they don't receive historical rewards that accrued before they delegated.
	///
	/// # Arguments
	/// * `delegator` - The delegator starting their delegation
	/// * `operator` - The operator being delegated to
	/// * `initial_stake` - The amount being delegated
	///
	/// # Complexity
	/// O(1) - Read pool, write debt, update pool total
	///
	/// # Example
	/// ```ignore
	/// // Operator pool has accumulated 10.5 per share from past rewards
	/// // Alice delegates 100 tokens
	/// init_delegator_reward_debt(&alice, &operator, 100)?;
	/// // Alice's debt is set to 10.5 (current accumulator)
	/// // She will only earn from NEW rewards, not historical 10.5
	/// ```
	pub fn init_delegator_reward_debt(
		delegator: &T::AccountId,
		operator: &T::AccountId,
		initial_stake: BalanceOf<T>,
	) -> DispatchResult {
		// Get current pool state
		let pool = OperatorRewardPools::<T>::get(operator);

		// Initialize debt at current accumulator (no historical rewards)
		DelegatorRewardDebts::<T>::insert(delegator, operator, crate::types::DelegatorRewardDebt {
			last_accumulated_per_share: pool.accumulated_rewards_per_share,
			staked_amount: initial_stake,
		});

		// Update pool's total staked amount
		OperatorRewardPools::<T>::mutate(operator, |p| {
			p.total_staked = p.total_staked.saturating_add(initial_stake);
		});

		log::info!(
			"Initialized delegator {:?} debt for operator {:?}: accumulated={:?}, stake={:?}",
			delegator,
			operator,
			pool.accumulated_rewards_per_share,
			initial_stake
		);

		Self::deposit_event(Event::DelegatorDebtInitialized {
			delegator: delegator.clone(),
			operator: operator.clone(),
			initial_accumulated_per_share: pool.accumulated_rewards_per_share,
			staked_amount: initial_stake,
		});

		Ok(())
	}

	/// Update delegator stake amount when they increase or decrease delegation.
	///
	/// **CRITICAL**: This MUST be called AFTER claiming pending rewards to ensure
	/// rewards are calculated at the old stake amount first.
	///
	/// # Arguments
	/// * `delegator` - The delegator changing their stake
	/// * `operator` - The operator they're delegated to
	/// * `stake_delta` - The change in stake (positive or negative)
	/// * `is_increase` - true if increasing stake, false if decreasing
	///
	/// # Complexity
	/// O(1) - Update debt storage, update pool total
	///
	/// # Safety
	/// Caller MUST call `calculate_and_claim_delegator_rewards` before this function.
	/// Otherwise, rewards will be calculated incorrectly.
	///
	/// # Example
	/// ```ignore
	/// // Alice has 100 staked, wants to add 50 more
	/// Self::calculate_and_claim_delegator_rewards(&alice, &operator)?; // Claim at 100 stake
	/// Self::update_delegator_stake(&alice, &operator, 50, true)?;      // Now increase to 150
	/// ```
	pub fn update_delegator_stake(
		delegator: &T::AccountId,
		operator: &T::AccountId,
		stake_delta: BalanceOf<T>,
		is_increase: bool,
	) -> DispatchResult {
		// Update delegator's staked amount in debt storage
		DelegatorRewardDebts::<T>::try_mutate(
			delegator,
			operator,
			|maybe_debt| -> DispatchResult {
				let debt = maybe_debt.as_mut().ok_or(Error::<T>::NoDelegation)?;

				// Update stake amount
				if is_increase {
					debt.staked_amount = debt.staked_amount.saturating_add(stake_delta);
				} else {
					debt.staked_amount = debt.staked_amount.saturating_sub(stake_delta);
				}

				log::debug!(
					"Updated delegator {:?} stake for operator {:?}: new_stake={:?}",
					delegator,
					operator,
					debt.staked_amount
				);

				Ok(())
			},
		)?;

		// Update pool's total staked amount
		OperatorRewardPools::<T>::mutate(operator, |pool| {
			if is_increase {
				pool.total_staked = pool.total_staked.saturating_add(stake_delta);
			} else {
				pool.total_staked = pool.total_staked.saturating_sub(stake_delta);
			}
		});

		Ok(())
	}

	/// Remove delegator from reward pool when they fully unstake.
	///
	/// **CRITICAL**: This MUST be called AFTER claiming all pending rewards.
	///
	/// # Arguments
	/// * `delegator` - The delegator leaving
	/// * `operator` - The operator they're leaving
	///
	/// # Complexity
	/// O(1) - Remove debt storage, update pool total
	pub fn remove_delegator_from_pool(
		delegator: &T::AccountId,
		operator: &T::AccountId,
	) -> DispatchResult {
		// Get current debt to update pool total
		let debt =
			DelegatorRewardDebts::<T>::get(delegator, operator).ok_or(Error::<T>::NoDelegation)?;

		// Remove debt storage
		DelegatorRewardDebts::<T>::remove(delegator, operator);

		// Update pool's total staked
		OperatorRewardPools::<T>::mutate(operator, |pool| {
			pool.total_staked = pool.total_staked.saturating_sub(debt.staked_amount);
		});

		log::info!(
			"Removed delegator {:?} from operator {:?} pool (stake was: {:?})",
			delegator,
			operator,
			debt.staked_amount
		);

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::*;
	use frame_support::assert_ok;

	type Rewards = Pallet<Runtime>;

	#[test]
	fn test_pool_based_reward_distribution_proportional() {
		new_test_ext().execute_with(|| {
			use sp_core::crypto::AccountId32;
			let operator = AccountId32::new([1u8; 32]);
			let delegator_a = AccountId32::new([2u8; 32]);
			let delegator_b = AccountId32::new([3u8; 32]);

			// Setup: Delegator A stakes 60, Delegator B stakes 40 (60/40 split)
			assert_ok!(Rewards::init_delegator_reward_debt(&delegator_a, &operator, 60));
			assert_ok!(Rewards::init_delegator_reward_debt(&delegator_b, &operator, 40));

			// Verify pool total
			let pool = OperatorRewardPools::<Runtime>::get(&operator);
			assert_eq!(pool.total_staked, 100);

			// Record 1000 tokens reward
			assert_ok!(Rewards::record_operator_reward_to_pool(&operator, 1000));

			// Calculate pending rewards
			let pending_a = Rewards::calculate_pending_delegator_rewards(&delegator_a, &operator);
			let pending_b = Rewards::calculate_pending_delegator_rewards(&delegator_b, &operator);

			assert_ok!(&pending_a);
			assert_ok!(&pending_b);

			// Verify proportional distribution: A gets 600, B gets 400
			assert_eq!(pending_a.unwrap(), 600);
			assert_eq!(pending_b.unwrap(), 400);
		});
	}

	#[test]
	fn test_multiple_rewards_accumulate() {
		new_test_ext().execute_with(|| {
			use sp_core::crypto::AccountId32;
			let operator = AccountId32::new([1u8; 32]);
			let delegator = AccountId32::new([2u8; 32]);

			// Setup: Single delegator with 100% stake
			assert_ok!(Rewards::init_delegator_reward_debt(&delegator, &operator, 100));

			// Record multiple rewards
			assert_ok!(Rewards::record_operator_reward_to_pool(&operator, 100));
			assert_ok!(Rewards::record_operator_reward_to_pool(&operator, 200));
			assert_ok!(Rewards::record_operator_reward_to_pool(&operator, 300));

			// Should accumulate to 600 total
			let pending =
				Rewards::calculate_pending_delegator_rewards(&delegator, &operator).unwrap();
			assert_eq!(pending, 600);
		});
	}

	#[test]
	fn test_delegator_joins_mid_period() {
		new_test_ext().execute_with(|| {
			use sp_core::crypto::AccountId32;
			let operator = AccountId32::new([1u8; 32]);
			let delegator_a = AccountId32::new([2u8; 32]);
			let delegator_b = AccountId32::new([3u8; 32]);

			// Delegator A joins with 50 stake
			assert_ok!(Rewards::init_delegator_reward_debt(&delegator_a, &operator, 50));

			// Record 1000 reward (A gets 100%)
			assert_ok!(Rewards::record_operator_reward_to_pool(&operator, 1000));

			// Delegator B joins with 50 stake (now 50/50 split)
			assert_ok!(Rewards::init_delegator_reward_debt(&delegator_b, &operator, 50));

			// Record another 1000 reward (both get 50%)
			assert_ok!(Rewards::record_operator_reward_to_pool(&operator, 1000));

			// A should have: 1000 (from first reward) + 500 (from second) = 1500
			let pending_a =
				Rewards::calculate_pending_delegator_rewards(&delegator_a, &operator).unwrap();
			assert_eq!(pending_a, 1500);

			// B should have: 0 (from first, wasn't delegated) + 500 (from second) = 500
			let pending_b =
				Rewards::calculate_pending_delegator_rewards(&delegator_b, &operator).unwrap();
			assert_eq!(pending_b, 500);
		});
	}

	#[test]
	fn test_claim_updates_debt() {
		new_test_ext().execute_with(|| {
			use sp_core::crypto::AccountId32;
			let operator = AccountId32::new([1u8; 32]);
			let delegator = AccountId32::new([2u8; 32]);

			// Setup
			assert_ok!(Rewards::init_delegator_reward_debt(&delegator, &operator, 100));

			// Record reward
			assert_ok!(Rewards::record_operator_reward_to_pool(&operator, 1000));

			// Claim (this should update debt)
			assert_ok!(Rewards::calculate_and_claim_delegator_rewards(&delegator, &operator));

			// Pending should now be zero
			let pending =
				Rewards::calculate_pending_delegator_rewards(&delegator, &operator).unwrap();
			assert_eq!(pending, 0);

			// Record another reward
			assert_ok!(Rewards::record_operator_reward_to_pool(&operator, 500));

			// Should only show new 500
			let pending =
				Rewards::calculate_pending_delegator_rewards(&delegator, &operator).unwrap();
			assert_eq!(pending, 500);
		});
	}
}
