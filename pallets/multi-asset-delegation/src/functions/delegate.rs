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

use crate::{types::*, Config, Delegators, Error, Event, Operators, Pallet};
use frame_support::{
	ensure,
	pallet_prelude::DispatchResult,
	traits::{Get, LockIdentifier, LockableCurrency, WithdrawReasons},
};
use sp_runtime::{
	traits::{CheckedAdd, Saturating, Zero},
	DispatchError,
};
use sp_staking::StakingInterface;
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
use tangle_primitives::{services::Asset, traits::MultiAssetDelegationInfo, RoundIndex};

pub const DELEGATION_LOCK_ID: LockIdentifier = *b"delegate";

impl<T: Config> Pallet<T> {
	/// Processes the delegation of an amount of an asset to an operator.
	///
	/// This function handles both creating new delegations and increasing existing ones.
	/// It updates three main pieces of state:
	/// 1. The delegator's deposit record (marking funds as delegated)
	/// 2. The delegator's delegation list
	/// 3. The operator's delegation records
	///
	/// # Performance Considerations
	/// - Single storage read for operator verification
	/// - Single storage write for delegation update
	/// - Bounded by MaxDelegations for new delegations
	///
	/// # Arguments
	/// * `who` - The account ID of the delegator
	/// * `operator` - The account ID of the operator to delegate to
	/// * `asset_id` - The asset being delegated
	/// * `amount` - The amount to delegate
	/// * `blueprint_selection` - Strategy for selecting which blueprints to work with:
	///   - Fixed: Work with specific blueprints
	///   - All: Work with all available blueprints
	///
	/// # Errors
	/// * `NotDelegator` - Account is not a delegator
	/// * `NotAnOperator` - Target account is not an operator
	/// * `InsufficientBalance` - Not enough deposited balance
	/// * `MaxDelegationsExceeded` - Would exceed maximum allowed delegations
	/// * `OverflowRisk` - Arithmetic overflow during calculations
	/// * `OperatorNotActive` - Operator is not in active status
	///
	/// # Example
	/// ```ignore
	/// // Delegate 100 tokens to operator with Fixed blueprint selection
	/// let blueprint_ids = vec![1, 2, 3];
	/// process_delegate(
	///     delegator,
	///     operator,
	///     Asset::Custom(token_id),
	///     100,
	///     DelegatorBlueprintSelection::Fixed(blueprint_ids)
	/// )?;
	/// ```
	pub fn process_delegate(
		who: T::AccountId,
		operator: T::AccountId,
		asset_id: Asset<T::AssetId>,
		amount: BalanceOf<T>,
		blueprint_selection: DelegatorBlueprintSelection<T::MaxDelegatorBlueprints>,
	) -> DispatchResult {
		// Verify operator exists and is active
		ensure!(Self::is_operator(&operator), Error::<T>::NotAnOperator);
		ensure!(Self::is_operator_active(&operator), Error::<T>::NotActiveOperator);
		ensure!(!amount.is_zero(), Error::<T>::InvalidAmount);

		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Ensure enough deposited balance and update it
			let user_deposit =
				metadata.deposits.get_mut(&asset_id).ok_or(Error::<T>::InsufficientBalance)?;
			user_deposit
				.increase_delegated_amount(amount)
				.map_err(|_| Error::<T>::InsufficientBalance)?;

			// Find existing delegation or create new one
			let delegation_exists = metadata
				.delegations
				.iter()
				.position(|d| d.operator == operator && d.asset_id == asset_id && !d.is_nomination);

			match delegation_exists {
				Some(idx) => {
					// Update existing delegation
					let delegation = &mut metadata.delegations[idx];
					delegation.amount =
						delegation.amount.checked_add(&amount).ok_or(Error::<T>::OverflowRisk)?;
				},
				None => {
					// Create new delegation
					metadata
						.delegations
						.try_push(BondInfoDelegator {
							operator: operator.clone(),
							amount,
							asset_id,
							blueprint_selection,
							is_nomination: false,
						})
						.map_err(|_| Error::<T>::MaxDelegationsExceeded)?;

					metadata.status = DelegatorStatus::Active;
				},
			}

			// Update operator metadata
			Self::update_operator_metadata(&operator, &who, asset_id, amount, true)?;

			// Emit event
			Self::deposit_event(Event::Delegated { who: who.clone(), operator, amount, asset_id });

			Ok(())
		})
	}

	/// Schedules a stake reduction for a delegator.
	///
	/// Creates an unstake request that can be executed after the delegation bond less delay period.
	/// The actual unstaking occurs when `execute_delegator_unstake` is called after the delay.
	/// Multiple unstake requests for the same delegation are allowed, but each must be within
	/// the available delegated amount.
	///
	/// # Performance Considerations
	/// - Single storage read for delegation verification
	/// - Single storage write for request creation
	/// - Bounded by MaxUnstakeRequests
	///
	/// # Arguments
	/// * `who` - The account ID of the delegator
	/// * `operator` - The account ID of the operator
	/// * `asset_id` - The asset to unstake
	/// * `amount` - The amount to unstake
	///
	/// # Errors
	/// * `NotDelegator` - Account is not a delegator
	/// * `NoActiveDelegation` - No active delegation found for operator and asset
	/// * `InsufficientBalance` - Trying to unstake more than delegated
	/// * `MaxUnstakeRequestsExceeded` - Too many pending unstake requests
	/// * `InvalidAmount` - Attempting to unstake zero tokens
	///
	/// # Example
	/// ```ignore
	/// // Schedule unstaking of 50 tokens from operator
	/// process_schedule_delegator_unstake(
	///     delegator,
	///     operator,
	///     Asset::Custom(token_id),
	///     50
	/// )?;
	/// ```
	pub fn process_schedule_delegator_unstake(
		who: T::AccountId,
		operator: T::AccountId,
		asset_id: Asset<T::AssetId>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		ensure!(!amount.is_zero(), Error::<T>::InvalidAmount);

		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Find and validate delegation in a single pass
			let delegation = metadata
				.delegations
				.iter()
				.find(|d| d.operator == operator && d.asset_id == asset_id && !d.is_nomination)
				.ok_or(Error::<T>::NoActiveDelegation)?;

			// Verify sufficient delegation amount considering existing unstake requests
			let pending_unstake_amount: BalanceOf<T> = metadata
				.delegator_unstake_requests
				.iter()
				.filter(|r| r.operator == operator && r.asset_id == asset_id)
				.fold(Zero::zero(), |acc, r| acc.saturating_add(r.amount));

			let available_amount = delegation.amount.saturating_sub(pending_unstake_amount);
			ensure!(available_amount >= amount, Error::<T>::InsufficientBalance);

			// Create the unstake request
			Self::create_unstake_request(
				metadata,
				operator.clone(),
				asset_id,
				amount,
				delegation.blueprint_selection.clone(),
				false, // is_nomination = false for regular delegations
			)?;

			Ok(())
		})
	}

	/// Cancels a scheduled stake reduction for a delegator.
	///
	/// This function removes a pending unstake request without modifying any actual delegations.
	/// It performs a simple lookup and removal of the matching request.
	///
	/// # Performance Considerations
	/// - Single storage read for request verification
	/// - Single storage write for request removal
	/// - O(n) search through unstake requests
	///
	/// # Arguments
	/// * `who` - The account ID of the delegator
	/// * `operator` - The operator whose unstake request to cancel
	/// * `asset_id` - The asset of the unstake request
	/// * `amount` - The exact amount of the unstake request to cancel
	///
	/// # Errors
	/// * `NotDelegator` - Account is not a delegator
	/// * `NoBondLessRequest` - No matching unstake request found
	/// * `InvalidAmount` - Amount specified is zero
	///
	/// # Example
	/// ```ignore
	/// // Cancel an unstake request for 50 tokens
	/// process_cancel_delegator_unstake(
	///     delegator,
	///     operator,
	///     Asset::Custom(token_id),
	///     50
	/// )?;
	/// ```
	pub fn process_cancel_delegator_unstake(
		who: T::AccountId,
		operator: T::AccountId,
		asset_id: Asset<T::AssetId>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		ensure!(!amount.is_zero(), Error::<T>::InvalidAmount);

		Delegators::<T>::try_mutate(&who, |maybe_metadata| {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Find and remove the matching unstake request
			let request_index = metadata
				.delegator_unstake_requests
				.iter()
				.position(|r| {
					r.asset_id == asset_id
						&& r.amount == amount
						&& r.operator == operator
						&& !r.is_nomination
				})
				.ok_or(Error::<T>::NoBondLessRequest)?;

			// Remove the request and emit event
			metadata.delegator_unstake_requests.remove(request_index);

			Ok(())
		})
	}

	/// Executes all ready unstake requests for a delegator.
	///
	/// This function processes multiple unstake requests in a batched manner for efficiency:
	/// 1. Aggregates all ready requests by asset and operator to minimize storage operations
	/// 2. Updates deposits, delegations, and operator metadata in batches
	/// 3. Removes zero-amount delegations and processed requests
	///
	/// # Performance Considerations
	/// - Uses batch processing to minimize storage reads/writes
	/// - Aggregates updates by asset and operator
	/// - Removes items in reverse order to avoid unnecessary shifting
	///
	/// # Arguments
	/// * `who` - The account ID of the delegator
	///
	/// # Errors
	/// * `NotDelegator` - Account is not a delegator
	/// * `NoBondLessRequest` - No unstake requests exist
	/// * `BondLessNotReady` - No requests are ready for execution
	/// * `NoActiveDelegation` - Referenced delegation not found
	/// * `InsufficientBalance` - Insufficient balance for unstaking
	pub fn process_execute_delegator_unstake(
		who: T::AccountId,
	) -> Result<Vec<(T::AccountId, Asset<T::AssetId>, BalanceOf<T>)>, DispatchError> {
		Delegators::<T>::try_mutate(&who, |maybe_metadata| -> Result<Vec<(T::AccountId, Asset<T::AssetId>, BalanceOf<T>)>, DispatchError> {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;
			ensure!(!metadata.delegator_unstake_requests.is_empty(), Error::<T>::NoBondLessRequest);

			let current_round = Self::current_round();
			let delay = T::DelegationBondLessDelay::get();

			// Aggregate all updates from ready requests
			let (deposit_updates, delegation_updates, operator_updates, indices_to_remove) =
				Self::aggregate_unstake_requests(metadata, current_round, delay)?;

			// Create a map to aggregate amounts by operator and asset
			let mut event_aggregates = BTreeMap::<(T::AccountId, Asset<T::AssetId>), BalanceOf<T>>::new();

			// Sum up amounts by operator and asset
			for &idx in &indices_to_remove {
				if let Some(request) = metadata.delegator_unstake_requests.get(idx) {
					let key = (request.operator.clone(), request.asset_id);
					let entry = event_aggregates.entry(key).or_insert(Zero::zero());
					*entry = entry.saturating_add(request.amount);
				}
			}

			// Apply updates in batches
			// 1. Update deposits
			for (asset_id, amount) in deposit_updates {
				metadata
					.deposits
					.get_mut(&asset_id)
					.ok_or(Error::<T>::InsufficientBalance)?
					.decrease_delegated_amount(amount)
					.map_err(|_| Error::<T>::InsufficientBalance)?;
			}

			// 2. Update delegations
			let mut delegations_to_remove = Vec::new();
			for ((_, _), (idx, amount)) in delegation_updates {
				let delegation =
					metadata.delegations.get_mut(idx).ok_or(Error::<T>::NoActiveDelegation)?;
				ensure!(delegation.amount >= amount, Error::<T>::InsufficientBalance);

				delegation.amount = delegation.amount.saturating_sub(amount);
				if delegation.amount.is_zero() {
					delegations_to_remove.push(idx);
				}
			}

			// 3. Remove zero-amount delegations
			delegations_to_remove.sort_unstable_by(|a, b| b.cmp(a));
			for idx in delegations_to_remove {
				metadata.delegations.remove(idx);
			}

			// 4. Update operator metadata
			for ((operator, asset_id), amount) in operator_updates {
				Self::update_operator_metadata(&operator, &who, asset_id, amount, false)?;
			}

			// 5. Remove processed requests
			let mut indices = indices_to_remove;
			indices.sort_unstable_by(|a, b| b.cmp(a));
			for idx in indices {
				metadata.delegator_unstake_requests.remove(idx);
			}

			// Convert the aggregates map into a vector for return
			Ok(event_aggregates
				.into_iter()
				.map(|((operator, asset_id), amount)| (operator, asset_id, amount))
				.collect())
		})
	}

	/// Processes the delegation of nominated tokens to an operator.
	///
	/// This function allows delegators to utilize their nominated (staked) tokens in the delegation system.
	/// It differs from regular delegation in that:
	/// 1. It uses nominated tokens instead of deposited assets
	/// 2. It maintains a lock on the nominated tokens
	/// 3. It tracks total nomination delegations to prevent over-delegation
	///
	/// # Performance Considerations
	/// - External call to staking system for verification
	/// - Single storage read for delegation lookup
	/// - Single storage write for delegation update
	/// - Additional storage write for token locking
	///
	/// # Arguments
	/// * `who` - The account ID of the delegator
	/// * `operator` - The operator to delegate to
	/// * `amount` - The amount of nominated tokens to delegate
	/// * `blueprint_selection` - Strategy for selecting which blueprints to work with
	///
	/// # Errors
	/// * `NotDelegator` - Account is not a delegator
	/// * `NotNominator` - Account has no nominated tokens
	/// * `InsufficientBalance` - Not enough nominated tokens available
	/// * `MaxDelegationsExceeded` - Would exceed maximum allowed delegations
	/// * `OverflowRisk` - Arithmetic overflow during calculations
	/// * `InvalidAmount` - Amount specified is zero
	///
	/// # Example
	/// ```ignore
	/// // Delegate 1000 nominated tokens to operator
	/// process_delegate_nominations(
	///     delegator,
	///     operator,
	///     1000,
	///     DelegatorBlueprintSelection::All
	/// )?;
	/// ```
	pub(crate) fn process_delegate_nominations(
		who: T::AccountId,
		operator: T::AccountId,
		amount: BalanceOf<T>,
		blueprint_selection: DelegatorBlueprintSelection<T::MaxDelegatorBlueprints>,
	) -> DispatchResult {
		ensure!(!amount.is_zero(), Error::<T>::InvalidAmount);
		ensure!(Self::is_operator(&operator), Error::<T>::NotAnOperator);
		ensure!(Self::is_operator_active(&operator), Error::<T>::NotActiveOperator);

		// Verify nomination amount in the staking system
		let nominated_amount = Self::verify_nomination_amount(&who, amount)?;

		Delegators::<T>::try_mutate(&who, |maybe_metadata| -> DispatchResult {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Calculate new total after this delegation
			let current_total = metadata.total_nomination_delegations();
			let new_total = current_total.checked_add(&amount).ok_or(Error::<T>::OverflowRisk)?;

			// Ensure total delegations don't exceed nominated amount
			ensure!(new_total <= nominated_amount, Error::<T>::InsufficientBalance);

			// Find existing nomination delegation or create new one
			let delegation_exists = metadata
				.delegations
				.iter()
				.position(|d| d.operator == operator && d.is_nomination);

			match delegation_exists {
				Some(idx) => {
					// Update existing delegation
					let delegation = &mut metadata.delegations[idx];
					let new_amount =
						delegation.amount.checked_add(&amount).ok_or(Error::<T>::OverflowRisk)?;

					delegation.amount = new_amount;
					T::Currency::set_lock(
						DELEGATION_LOCK_ID,
						&who,
						new_amount,
						WithdrawReasons::TRANSFER,
					);
				},
				None => {
					// Create new delegation
					metadata
						.delegations
						.try_push(BondInfoDelegator {
							operator: operator.clone(),
							amount,
							asset_id: Asset::Custom(Zero::zero()),
							blueprint_selection,
							is_nomination: true,
						})
						.map_err(|_| Error::<T>::MaxDelegationsExceeded)?;

					T::Currency::set_lock(
						DELEGATION_LOCK_ID,
						&who,
						amount,
						WithdrawReasons::TRANSFER,
					);
				},
			}

			// Update operator metadata
			Self::update_operator_metadata(
				&operator,
				&who,
				Asset::Custom(Zero::zero()),
				amount,
				true, // is_increase = true for delegation
			)?;

			// Emit event
			Self::deposit_event(Event::NominationDelegated {
				who: who.clone(),
				operator: operator.clone(),
				amount,
			});

			Ok(())
		})?;

		Ok(())
	}

	/// Schedules an unstake request for nomination delegations.
	///
	/// Similar to regular unstaking but specifically for nominated tokens. This function:
	/// 1. Verifies the nomination delegation exists
	/// 2. Checks if there's enough balance to unstake
	/// 3. Creates an unstake request that can be executed after the delay period
	///
	/// # Performance Considerations
	/// - Single storage read for delegation verification
	/// - Single storage write for request creation
	/// - O(n) search through delegations
	///
	/// # Arguments
	/// * `who` - The account ID of the delegator
	/// * `operator` - The operator to unstake from
	/// * `amount` - The amount of nominated tokens to unstake
	/// * `blueprint_selection` - The blueprint selection to use after unstaking
	///
	/// # Errors
	/// * `NotDelegator` - Account is not a delegator
	/// * `NoActiveDelegation` - No active nomination delegation found
	/// * `InsufficientBalance` - Trying to unstake more than delegated
	/// * `MaxUnstakeRequestsExceeded` - Too many pending unstake requests
	/// * `InvalidAmount` - Amount specified is zero
	/// * `AssetNotWhitelisted` - Invalid asset type for nominations
	///
	/// # Example
	/// ```ignore
	/// // Schedule unstaking of 500 nominated tokens
	/// process_schedule_delegator_nomination_unstake(
	///     &delegator,
	///     operator,
	///     500,
	///     DelegatorBlueprintSelection::All
	/// )?;
	/// ```
	pub fn process_schedule_delegator_nomination_unstake(
		who: &T::AccountId,
		operator: T::AccountId,
		amount: BalanceOf<T>,
		blueprint_selection: DelegatorBlueprintSelection<T::MaxDelegatorBlueprints>,
	) -> Result<RoundIndex, DispatchError> {
		ensure!(!amount.is_zero(), Error::<T>::InvalidAmount);
		ensure!(Self::is_operator(&operator), Error::<T>::NotAnOperator);

		Delegators::<T>::try_mutate(who, |maybe_metadata| -> Result<RoundIndex, DispatchError> {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Find the nomination delegation and verify amount
			let (_, current_amount) =
				Self::find_nomination_delegation(&metadata.delegations, &operator)?
					.ok_or(Error::<T>::NoActiveDelegation)?;

			// Calculate total pending unstakes
			let pending_unstake_amount: BalanceOf<T> = metadata
				.delegator_unstake_requests
				.iter()
				.filter(|r| r.operator == operator && r.is_nomination)
				.fold(Zero::zero(), |acc, r| acc.saturating_add(r.amount));

			let available_amount = current_amount.saturating_sub(pending_unstake_amount);
			ensure!(available_amount >= amount, Error::<T>::InsufficientBalance);

			// Create the unstake request
			Self::create_unstake_request(
				metadata,
				operator.clone(),
				Asset::Custom(Zero::zero()),
				amount,
				blueprint_selection,
				true, // is_nomination = true for nomination delegations
			)?;

			let when = Self::current_round() + T::DelegationBondLessDelay::get();
			Ok(when)
		})
	}

	/// Cancels a scheduled unstake request for nomination delegations.
	///
	/// Similar to regular unstake cancellation but specifically for nominated tokens.
	/// This function removes a pending unstake request without modifying any actual delegations.
	///
	/// # Performance Considerations
	/// - Single storage read for request verification
	/// - Single storage write for request removal
	/// - O(n) search through unstake requests
	///
	/// # Arguments
	/// * `who` - The account ID of the delegator
	/// * `operator` - The operator whose unstake request to cancel
	///
	/// # Errors
	/// * `NotDelegator` - Account is not a delegator
	/// * `NoBondLessRequest` - No matching unstake request found
	///
	/// # Example
	/// ```ignore
	/// // Cancel nomination unstake request from operator
	/// process_cancel_delegator_nomination_unstake(
	///     &delegator,
	///     operator
	/// )?;
	/// ```
	pub(crate) fn process_cancel_delegator_nomination_unstake(
		who: &T::AccountId,
		operator: T::AccountId,
	) -> Result<
		BondLessRequest<T::AccountId, T::AssetId, BalanceOf<T>, T::MaxDelegatorBlueprints>,
		DispatchError,
	> {
		Delegators::<T>::try_mutate(
			who,
			|maybe_metadata| -> Result<
				BondLessRequest<T::AccountId, T::AssetId, BalanceOf<T>, T::MaxDelegatorBlueprints>,
				DispatchError,
			> {
				let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

				// Find and remove the unstake request
				let request_index = metadata
					.delegator_unstake_requests
					.iter()
					.position(|r| r.operator == operator && r.is_nomination)
					.ok_or(Error::<T>::NoBondLessRequest)?;

				// Remove the request
				let request = metadata.delegator_unstake_requests.remove(request_index);

				Ok(request.clone())
			},
		)
	}

	/// Execute an unstake request for nomination delegations
	pub fn process_execute_delegator_nomination_unstake(
		who: &T::AccountId,
		operator: T::AccountId,
	) -> Result<BalanceOf<T>, DispatchError> {
		Delegators::<T>::try_mutate(who, |maybe_metadata| -> Result<BalanceOf<T>, DispatchError> {
			let metadata = maybe_metadata.as_mut().ok_or(Error::<T>::NotDelegator)?;

			// Find and validate the unstake request
			let (request_index, request) = metadata
				.delegator_unstake_requests
				.iter()
				.enumerate()
				.find(|(_, r)| r.operator == operator && r.is_nomination)
				.ok_or(Error::<T>::NoBondLessRequest)?;

			ensure!(request.requested_round <= Self::current_round(), Error::<T>::BondLessNotReady);

			// Store the amount before removing the request
			let unstake_amount = request.amount;

			// Find the nomination delegation
			let (delegation_index, current_amount) =
				Self::find_nomination_delegation(&metadata.delegations, &operator)?
					.ok_or(Error::<T>::NoActiveDelegation)?;

			// Verify the unstake amount is still valid
			ensure!(current_amount >= unstake_amount, Error::<T>::InsufficientBalance);

			// Update the delegation
			let delegation = &mut metadata.delegations[delegation_index];
			delegation.amount = delegation.amount.saturating_sub(unstake_amount);

			// Update operator metadata during execution
			Self::update_operator_metadata(
				&operator,
				who,
				Asset::Custom(Zero::zero()),
				unstake_amount,
				false, // is_increase = false for unstaking
			)?;

			// Remove the unstake request
			metadata.delegator_unstake_requests.remove(request_index);

			// Set the lock to the new amount or remove it if zero
			if delegation.amount.is_zero() {
				T::Currency::remove_lock(DELEGATION_LOCK_ID, who);
				metadata.delegations.remove(delegation_index);
			} else {
				T::Currency::set_lock(
					DELEGATION_LOCK_ID,
					who,
					delegation.amount,
					WithdrawReasons::TRANSFER,
				);
			}

			Ok(unstake_amount)
		})
	}

	/// Helper function to update operator metadata for a delegation change
	fn update_operator_metadata(
		operator: &T::AccountId,
		who: &T::AccountId,
		asset_id: Asset<T::AssetId>,
		amount: BalanceOf<T>,
		is_increase: bool,
	) -> DispatchResult {
		Operators::<T>::try_mutate(operator, |maybe_operator_metadata| -> DispatchResult {
			let operator_metadata =
				maybe_operator_metadata.as_mut().ok_or(Error::<T>::NotAnOperator)?;

			let mut delegations = operator_metadata.delegations.clone();

			if is_increase {
				// Adding or increasing delegation
				ensure!(
					operator_metadata.delegation_count < T::MaxDelegations::get(),
					Error::<T>::MaxDelegationsExceeded
				);

				if let Some(existing_delegation) =
					delegations.iter_mut().find(|d| d.delegator == *who && d.asset_id == asset_id)
				{
					existing_delegation.amount = existing_delegation
						.amount
						.checked_add(&amount)
						.ok_or(Error::<T>::OverflowRisk)?;
				} else {
					delegations
						.try_push(DelegatorBond { delegator: who.clone(), amount, asset_id })
						.map_err(|_| Error::<T>::MaxDelegationsExceeded)?;
					operator_metadata.delegation_count =
						operator_metadata.delegation_count.saturating_add(1);
				}
			} else {
				// Decreasing or removing delegation
				if let Some(index) =
					delegations.iter().position(|d| d.delegator == *who && d.asset_id == asset_id)
				{
					let delegation = &mut delegations[index];
					ensure!(delegation.amount >= amount, Error::<T>::InsufficientBalance);

					delegation.amount = delegation.amount.saturating_sub(amount);
					if delegation.amount.is_zero() {
						delegations.remove(index);
						operator_metadata.delegation_count = operator_metadata
							.delegation_count
							.checked_sub(1)
							.ok_or(Error::<T>::InsufficientBalance)?;
					}
				}
			}

			operator_metadata.delegations = delegations;
			Ok(())
		})
	}

	/// Helper function to verify and get nomination amount
	fn verify_nomination_amount(
		who: &T::AccountId,
		required_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, Error<T>> {
		let stake = T::StakingInterface::stake(who).map_err(|_| Error::<T>::NotNominator)?;
		ensure!(stake.active >= required_amount, Error::<T>::InsufficientBalance);
		Ok(stake.active)
	}

	/// Helper function to find and validate a nomination delegation
	fn find_nomination_delegation(
		delegations: &[BondInfoDelegator<
			T::AccountId,
			BalanceOf<T>,
			T::AssetId,
			T::MaxDelegatorBlueprints,
		>],
		operator: &T::AccountId,
	) -> Result<Option<(usize, BalanceOf<T>)>, Error<T>> {
		if let Some((index, delegation)) = delegations
			.iter()
			.enumerate()
			.find(|(_, d)| d.operator == *operator && d.is_nomination)
		{
			ensure!(
				delegation.asset_id == Asset::Custom(Zero::zero()),
				Error::<T>::AssetNotWhitelisted
			);
			Ok(Some((index, delegation.amount.clone())))
		} else {
			Ok(None)
		}
	}

	/// Helper function to create an unstake request
	fn create_unstake_request(
		metadata: &mut DelegatorMetadataOf<T>,
		operator: T::AccountId,
		asset_id: Asset<T::AssetId>,
		amount: BalanceOf<T>,
		blueprint_selection: DelegatorBlueprintSelection<T::MaxDelegatorBlueprints>,
		is_nomination: bool,
	) -> DispatchResult {
		let unstake_request = BondLessRequest {
			operator,
			asset_id,
			amount,
			requested_round: Self::current_round(),
			blueprint_selection,
			is_nomination,
		};

		metadata
			.delegator_unstake_requests
			.try_push(unstake_request)
			.map_err(|_| Error::<T>::MaxUnstakeRequestsExceeded)?;

		Ok(())
	}

	/// Helper function to process a batch of unstake requests
	/// Returns aggregated updates for deposits, delegations, and operators
	fn aggregate_unstake_requests(
		metadata: &DelegatorMetadataOf<T>,
		current_round: RoundIndex,
		delay: RoundIndex,
	) -> Result<
		(
			BTreeMap<Asset<T::AssetId>, BalanceOf<T>>, // deposit_updates
			BTreeMap<(T::AccountId, Asset<T::AssetId>), (usize, BalanceOf<T>)>, // delegation_updates
			BTreeMap<(T::AccountId, Asset<T::AssetId>), BalanceOf<T>>, // operator_updates
			Vec<usize>,                                // indices_to_remove
		),
		Error<T>,
	> {
		let mut indices_to_remove = Vec::new();
		let mut delegation_updates = BTreeMap::new();
		let mut deposit_updates = BTreeMap::new();
		let mut operator_updates = BTreeMap::new();

		for (idx, request) in metadata.delegator_unstake_requests.iter().enumerate() {
			if current_round < delay + request.requested_round {
				continue;
			}

			*deposit_updates.entry(request.asset_id).or_default() += request.amount;

			let delegation_key = (request.operator.clone(), request.asset_id);
			if let Some(delegation_idx) = metadata.delegations.iter().position(|d| {
				d.operator == request.operator && d.asset_id == request.asset_id && !d.is_nomination
			}) {
				let (_, total_unstake) = delegation_updates
					.entry(delegation_key.clone())
					.or_insert((delegation_idx, BalanceOf::<T>::zero()));
				*total_unstake += request.amount;
			} else {
				return Err(Error::<T>::NoActiveDelegation);
			}

			*operator_updates.entry(delegation_key).or_default() += request.amount;
			indices_to_remove.push(idx);
		}

		ensure!(!indices_to_remove.is_empty(), Error::<T>::BondLessNotReady);
		Ok((deposit_updates, delegation_updates, operator_updates, indices_to_remove))
	}
}
