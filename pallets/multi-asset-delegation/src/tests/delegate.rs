// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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
use crate::CurrentRound;
use crate::Error;
use frame_support::{assert_noop, assert_ok};

#[test]
fn delegate_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let operator = 2;
		let asset_id = VDOT;
		let amount = 100;

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator), 10_000));

		create_and_mint_tokens(VDOT, who, amount);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));

		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(metadata.deposits.get(&asset_id).is_none());
		assert_eq!(metadata.delegations.len(), 1);
		let delegation = &metadata.delegations[0];
		assert_eq!(delegation.operator, operator);
		assert_eq!(delegation.amount, amount);
		assert_eq!(delegation.asset_id, asset_id);

		// Check the operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator).unwrap();
		assert_eq!(operator_metadata.delegation_count, 1);
		assert_eq!(operator_metadata.delegations.len(), 1);
		let operator_delegation = &operator_metadata.delegations[0];
		assert_eq!(operator_delegation.delegator, who);
		assert_eq!(operator_delegation.amount, amount);
		assert_eq!(operator_delegation.asset_id, asset_id);
	});
}

#[test]
fn schedule_delegator_bond_less_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let operator = 2;
		let asset_id = VDOT;
		let amount = 100;

		create_and_mint_tokens(VDOT, who, amount);

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator), 10_000));

		// Deposit and delegate first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		assert_ok!(MultiAssetDelegation::schedule_delegator_bond_less(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		// Assert
		// Check the delegator metadata
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(!metadata.delegator_bond_less_requests.is_empty());
		let request = &metadata.delegator_bond_less_requests[0];
		assert_eq!(request.asset_id, asset_id);
		assert_eq!(request.amount, amount);

		// Check the operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator).unwrap();
		assert_eq!(operator_metadata.delegation_count, 0);
		assert_eq!(operator_metadata.delegations.len(), 0);
	});
}

#[test]
fn execute_delegator_bond_less_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let operator = 2;
		let asset_id = VDOT;
		let amount = 100;

		create_and_mint_tokens(VDOT, who, amount);

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator), 10_000));

		// Deposit, delegate and schedule bond less first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));
		assert_ok!(MultiAssetDelegation::schedule_delegator_bond_less(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		// Simulate round passing
		CurrentRound::<Test>::put(10);

		assert_ok!(MultiAssetDelegation::execute_delegator_bond_less(RuntimeOrigin::signed(who),));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(metadata.delegator_bond_less_requests.is_empty());
		assert!(metadata.deposits.get(&asset_id).is_some());
		assert_eq!(metadata.deposits.get(&asset_id).unwrap(), &amount);
	});
}

#[test]
fn cancel_delegator_bond_less_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let operator = 2;
		let asset_id = VDOT;
		let amount = 100;

		create_and_mint_tokens(VDOT, who, amount);

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator), 10_000));

		// Deposit, delegate and schedule bond less first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));
		assert_ok!(MultiAssetDelegation::schedule_delegator_bond_less(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		assert_ok!(MultiAssetDelegation::cancel_delegator_bond_less(
			RuntimeOrigin::signed(who),
			asset_id,
			amount
		));

		// Assert
		// Check the delegator metadata
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(metadata.delegator_bond_less_requests.is_empty());

		// Check the operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator).unwrap();
		assert_eq!(operator_metadata.delegation_count, 1);
		assert_eq!(operator_metadata.delegations.len(), 1);
		let operator_delegation = &operator_metadata.delegations[0];
		assert_eq!(operator_delegation.delegator, who);
		assert_eq!(operator_delegation.amount, amount); // Amount added back
		assert_eq!(operator_delegation.asset_id, asset_id);
	});
}

#[test]
fn delegate_should_fail_if_not_enough_balance() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let operator = 2;
		let asset_id = VDOT;
		let amount = 10_000;

		create_and_mint_tokens(VDOT, who, amount);

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator), 10_000));

		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount - 20,
		));

		assert_noop!(
			MultiAssetDelegation::delegate(RuntimeOrigin::signed(who), operator, asset_id, amount,),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn schedule_delegator_bond_less_should_fail_if_no_delegation() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let operator = 2;
		let asset_id = VDOT;
		let amount = 100;

		create_and_mint_tokens(VDOT, who, amount);

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator), 10_000));

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));

		assert_noop!(
			MultiAssetDelegation::schedule_delegator_bond_less(
				RuntimeOrigin::signed(who),
				operator,
				asset_id,
				amount,
			),
			Error::<Test>::NoActiveDelegation
		);
	});
}

#[test]
fn execute_delegator_bond_less_should_fail_if_not_ready() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let operator = 2;
		let asset_id = VDOT;
		let amount = 100;

		create_and_mint_tokens(VDOT, who, amount);

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator), 10_000));

		// Deposit, delegate and schedule bond less first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		assert_noop!(
			MultiAssetDelegation::cancel_delegator_bond_less(
				RuntimeOrigin::signed(who),
				asset_id,
				amount
			),
			Error::<Test>::NoBondLessRequest
		);

		assert_ok!(MultiAssetDelegation::schedule_delegator_bond_less(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		assert_noop!(
			MultiAssetDelegation::execute_delegator_bond_less(RuntimeOrigin::signed(who),),
			Error::<Test>::BondLessNotReady
		);
	});
}

#[test]
fn delegate_should_not_create_multiple_on_repeat_delegation() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let operator = 2;
		let asset_id = VDOT;
		let amount = 100;
		let additional_amount = 50;

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator), 10_000));

		create_and_mint_tokens(VDOT, who, amount + additional_amount);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount + additional_amount,
		));

		// Delegate first time
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		// Assert first delegation
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(metadata.deposits.get(&asset_id).is_some());
		assert_eq!(metadata.delegations.len(), 1);
		let delegation = &metadata.delegations[0];
		assert_eq!(delegation.operator, operator);
		assert_eq!(delegation.amount, amount);
		assert_eq!(delegation.asset_id, asset_id);

		// Check the operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator).unwrap();
		assert_eq!(operator_metadata.delegation_count, 1);
		assert_eq!(operator_metadata.delegations.len(), 1);
		let operator_delegation = &operator_metadata.delegations[0];
		assert_eq!(operator_delegation.delegator, who);
		assert_eq!(operator_delegation.amount, amount);
		assert_eq!(operator_delegation.asset_id, asset_id);

		// Delegate additional amount
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			additional_amount,
		));

		// Assert updated delegation
		let updated_metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(updated_metadata.deposits.get(&asset_id).is_none());
		assert_eq!(updated_metadata.delegations.len(), 1);
		let updated_delegation = &updated_metadata.delegations[0];
		assert_eq!(updated_delegation.operator, operator);
		assert_eq!(updated_delegation.amount, amount + additional_amount);
		assert_eq!(updated_delegation.asset_id, asset_id);

		// Check the updated operator metadata
		let updated_operator_metadata = MultiAssetDelegation::operator_info(operator).unwrap();
		assert_eq!(updated_operator_metadata.delegation_count, 1);
		assert_eq!(updated_operator_metadata.delegations.len(), 1);
		let updated_operator_delegation = &updated_operator_metadata.delegations[0];
		assert_eq!(updated_operator_delegation.delegator, who);
		assert_eq!(updated_operator_delegation.amount, amount + additional_amount);
		assert_eq!(updated_operator_delegation.asset_id, asset_id);
	});
}
