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

use super::*;
use crate::{CurrentRound, Error};
use frame_support::{assert_noop, assert_ok};
use sp_keyring::AccountKeyring::{Alice, Bob, Charlie};
use tangle_primitives::services::Asset;

#[test]
fn successful_native_restaking() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let nomination_amount = 100;

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Setup nomination for the delegator
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![operator.clone()]));
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(who.clone()),
			nomination_amount,
			pallet_staking::RewardDestination::Staked
		));

		// Act - Delegate native stake
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			nomination_amount,
			Default::default(),
		));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.nomination_delegations.len(), 1);
		let delegation = &metadata.nomination_delegations[0];
		assert_eq!(delegation.operator, operator.clone());
		assert_eq!(delegation.amount, nomination_amount);

		// Check operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.nomination_count, 1);
		assert_eq!(operator_metadata.nomination_delegations.len(), 1);
		let operator_delegation = &operator_metadata.nomination_delegations[0];
		assert_eq!(operator_delegation.delegator, who.clone());
		assert_eq!(operator_delegation.amount, nomination_amount);
	});
}

#[test]
fn successful_multiple_native_restaking() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let total_nomination = 100;
		let first_restake = 40;
		let second_restake = 30;

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Setup nomination
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![operator.clone()]));
		assert_ok!(Staking::bond(RuntimeOrigin::signed(who.clone()), total_nomination));

		// First restake
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			first_restake,
			Default::default(),
		));

		// Second restake
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			second_restake,
			Default::default(),
		));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.nomination_delegations.len(), 1);
		let delegation = &metadata.nomination_delegations[0];
		assert_eq!(delegation.operator, operator.clone());
		assert_eq!(delegation.amount, first_restake + second_restake);

		// Check operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.nomination_count, 1);
		assert_eq!(operator_metadata.nomination_delegations.len(), 1);
		let operator_delegation = &operator_metadata.nomination_delegations[0];
		assert_eq!(operator_delegation.delegator, who.clone());
		assert_eq!(operator_delegation.amount, first_restake + second_restake);
	});
}

#[test]
fn successful_native_and_non_native_restaking() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let nomination_amount = 100;
		let non_native_amount = 50;
		let asset_id = Asset::Custom(VDOT);

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Setup nomination
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![operator.clone()]));
		assert_ok!(Staking::bond(RuntimeOrigin::signed(who.clone()), nomination_amount));

		// Setup non-native asset
		create_and_mint_tokens(VDOT, who.clone(), non_native_amount);

		// Deposit non-native asset
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id.clone(),
			non_native_amount,
			None,
			None,
		));

		// Delegate native stake
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			nomination_amount,
			Default::default(),
		));

		// Delegate non-native asset
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			non_native_amount,
			Default::default(),
		));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();

		// Check nomination delegation
		assert_eq!(metadata.nomination_delegations.len(), 1);
		let nomination_delegation = &metadata.nomination_delegations[0];
		assert_eq!(nomination_delegation.operator, operator.clone());
		assert_eq!(nomination_delegation.amount, nomination_amount);

		// Check non-native delegation
		assert_eq!(metadata.delegations.len(), 1);
		let asset_delegation = &metadata.delegations[0];
		assert_eq!(asset_delegation.operator, operator.clone());
		assert_eq!(asset_delegation.amount, non_native_amount);
		assert_eq!(asset_delegation.asset_id, asset_id);

		// Check operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.nomination_count, 1);
		assert_eq!(operator_metadata.delegation_count, 1);
	});
}

#[test]
fn successful_restaking_and_partial_unbond() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let total_nomination = 100;
		let restake_amount = 60;
		let unbond_amount = 40;
		let excessive_unbond = 70;

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Setup nomination
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![operator.clone()]));
		assert_ok!(Staking::bond(RuntimeOrigin::signed(who.clone()), total_nomination));

		// Restake
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			restake_amount,
			Default::default(),
		));

		// Successful partial unbond
		assert_ok!(Staking::unbond(RuntimeOrigin::signed(who.clone()), unbond_amount));

		// Attempt excessive unbond - should fail
		assert_noop!(
			Staking::unbond(RuntimeOrigin::signed(who.clone()), excessive_unbond),
			Error::<Runtime>::InsufficientStakeRemaining
		);

		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.nomination_delegations.len(), 1);
		let delegation = &metadata.nomination_delegations[0];
		assert_eq!(delegation.amount, restake_amount);
	});
}

#[test]
fn fails_native_restake_no_nominations() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let amount = 100;

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Attempt to delegate without nomination
		assert_noop!(
			MultiAssetDelegation::delegate_nomination(
				RuntimeOrigin::signed(who.clone()),
				operator.clone(),
				amount,
				Default::default(),
			),
			Error::<Runtime>::NotNominator
		);
	});
}

#[test]
fn fails_native_restake_not_operator() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let non_operator: AccountId = Charlie.into();
		let amount = 100;

		// Setup nomination
		assert_ok!(Staking::nominate(
			RuntimeOrigin::signed(who.clone()),
			vec![non_operator.clone()]
		));
		assert_ok!(Staking::bond(RuntimeOrigin::signed(who.clone()), amount));

		// Attempt to delegate to non-operator
		assert_noop!(
			MultiAssetDelegation::delegate_nomination(
				RuntimeOrigin::signed(who.clone()),
				non_operator.clone(),
				amount,
				Default::default(),
			),
			Error::<Runtime>::NotAnOperator
		);
	});
}

#[test]
fn fails_native_restake_amount_exceeds_nomination() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let nomination_amount = 100;
		let excessive_amount = 150;

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Setup nomination
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![operator.clone()]));
		assert_ok!(Staking::bond(RuntimeOrigin::signed(who.clone()), nomination_amount));

		// Attempt to delegate more than nominated
		assert_noop!(
			MultiAssetDelegation::delegate_nomination(
				RuntimeOrigin::signed(who.clone()),
				operator.clone(),
				excessive_amount,
				Default::default(),
			),
			Error::<Runtime>::InsufficientBalance
		);
	});
}

#[test]
fn successful_native_restake_and_unstake() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let nomination_amount = 100;
		let unstake_amount = 40;

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Setup nomination
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![operator.clone()]));
		assert_ok!(Staking::bond(RuntimeOrigin::signed(who.clone()), nomination_amount));

		// Delegate native stake
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			nomination_amount,
			Default::default(),
		));

		// Schedule unstake
		assert_ok!(MultiAssetDelegation::schedule_nomination_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			unstake_amount,
			Default::default(),
		));

		// Simulate round passing
		CurrentRound::<Runtime>::put(10);

		// Execute unstake
		assert_ok!(MultiAssetDelegation::execute_nomination_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
		));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.nomination_delegations.len(), 1);
		let delegation = &metadata.nomination_delegations[0];
		assert_eq!(delegation.amount, nomination_amount - unstake_amount);
	});
}

#[test]
fn fails_native_restake_excessive_unstake() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let nomination_amount = 100;
		let excessive_unstake = 150;

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Setup nomination
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![operator.clone()]));
		assert_ok!(Staking::bond(RuntimeOrigin::signed(who.clone()), nomination_amount));

		// Delegate native stake
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			nomination_amount,
			Default::default(),
		));

		// Attempt to schedule excessive unstake
		assert_noop!(
			MultiAssetDelegation::schedule_nomination_unstake(
				RuntimeOrigin::signed(who.clone()),
				operator.clone(),
				excessive_unstake,
				Default::default(),
			),
			Error::<Runtime>::UnstakeAmountTooLarge
		);
	});
}

#[test]
fn successful_native_restake_unstake_and_cancel() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let nomination_amount = 100;
		let unstake_amount = 40;

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Setup nomination
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![operator.clone()]));
		assert_ok!(Staking::bond(RuntimeOrigin::signed(who.clone()), nomination_amount));

		// Delegate native stake
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			nomination_amount,
			Default::default(),
		));

		// Schedule unstake
		assert_ok!(MultiAssetDelegation::schedule_nomination_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			unstake_amount,
			Default::default(),
		));

		// Cancel unstake
		assert_ok!(MultiAssetDelegation::cancel_nomination_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
		));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.nomination_delegations.len(), 1);
		let delegation = &metadata.nomination_delegations[0];
		assert_eq!(delegation.amount, nomination_amount);
		assert!(metadata.nomination_unstake_requests.is_empty());

		// Check operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.nomination_count, 1);
		let operator_delegation = &operator_metadata.nomination_delegations[0];
		assert_eq!(operator_delegation.amount, nomination_amount);
	});
}
