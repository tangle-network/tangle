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
#![allow(clippy::all)]
use super::*;
use crate::{CurrentRound, Error};
use frame_support::{assert_noop, assert_ok};
use sp_keyring::AccountKeyring::{Alice, Bob, Charlie};
use tangle_primitives::services::Asset;

#[test]
fn delegate_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		create_and_mint_tokens(VDOT, who.clone(), amount);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id.clone(),
			amount,
			None,
			None,
		));

		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			amount,
			Default::default(),
		));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegations.len(), 1);
		let delegation = &metadata.delegations[0];
		assert_eq!(delegation.operator, operator.clone());
		assert_eq!(delegation.amount, amount);
		assert_eq!(delegation.asset_id, asset_id);

		// Check the operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.delegation_count, 1);
		assert_eq!(operator_metadata.delegations.len(), 1);
		let operator_delegation = &operator_metadata.delegations[0];
		assert_eq!(operator_delegation.delegator, who.clone());
		assert_eq!(operator_delegation.amount, amount);
		assert_eq!(operator_delegation.asset_id, asset_id);
	});
}

#[test]
fn schedule_delegator_unstake_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		create_and_mint_tokens(VDOT, who.clone(), amount);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Deposit and delegate first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id.clone(),
			amount,
			None,
			None
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			amount,
			Default::default(),
		));

		assert_ok!(MultiAssetDelegation::schedule_delegator_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			amount,
		));

		// Assert
		// Check the delegator metadata
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert!(!metadata.delegator_unstake_requests.is_empty());
		let request = &metadata.delegator_unstake_requests[0];
		assert_eq!(request.asset_id, asset_id);
		assert_eq!(request.amount, amount);

		// Check the operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.delegation_count, 1);
		assert_eq!(operator_metadata.delegations.len(), 1);
		// Move to next round
		CurrentRound::<Runtime>::put(10);
		// Execute the unstake
		assert_ok!(MultiAssetDelegation::execute_delegator_unstake(RuntimeOrigin::signed(
			who.clone()
		),));

		// Check the operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.delegation_count, 0);
		assert_eq!(operator_metadata.delegations.len(), 0);
	});
}

#[test]
fn execute_delegator_unstake_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		create_and_mint_tokens(VDOT, who.clone(), amount);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Deposit, delegate and schedule unstake first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id.clone(),
			amount,
			None,
			None
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			amount,
			Default::default(),
		));

		assert_ok!(MultiAssetDelegation::schedule_delegator_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			amount,
		));

		// Simulate round passing
		CurrentRound::<Runtime>::put(10);

		assert_ok!(MultiAssetDelegation::execute_delegator_unstake(RuntimeOrigin::signed(
			who.clone()
		),));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert!(metadata.delegator_unstake_requests.is_empty());
		assert!(metadata.deposits.get(&asset_id).is_some());
		let deposit = metadata.deposits.get(&asset_id).unwrap();
		assert_eq!(deposit.amount, amount);
		assert_eq!(deposit.delegated_amount, 0);
	});
}

#[test]
fn cancel_delegator_unstake_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		create_and_mint_tokens(VDOT, who.clone(), amount);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Deposit, delegate and schedule unstake first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id.clone(),
			amount,
			None,
			None
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			amount,
			Default::default(),
		));

		assert_ok!(MultiAssetDelegation::schedule_delegator_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			amount,
		));

		// Assert
		// Check the delegator metadata
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert!(!metadata.delegator_unstake_requests.is_empty());
		let request = &metadata.delegator_unstake_requests[0];
		assert_eq!(request.asset_id, asset_id);
		assert_eq!(request.amount, amount);

		// Check the operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.delegation_count, 1);
		assert_eq!(operator_metadata.delegations.len(), 1);

		assert_ok!(MultiAssetDelegation::cancel_delegator_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			amount
		));

		// Assert
		// Check the delegator metadata
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert!(metadata.delegator_unstake_requests.is_empty());

		// Check the operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.delegation_count, 1);
		assert_eq!(operator_metadata.delegations.len(), 1);
		let operator_delegation = &operator_metadata.delegations[0];
		assert_eq!(operator_delegation.delegator, who.clone());
		assert_eq!(operator_delegation.amount, amount); // Amount added back
		assert_eq!(operator_delegation.asset_id, asset_id);
	});
}

#[test]
fn cancel_delegator_unstake_should_update_already_existing() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		create_and_mint_tokens(VDOT, who.clone(), amount);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Deposit, delegate and schedule unstake first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id.clone(),
			amount,
			None,
			None
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			amount,
			Default::default(),
		));

		assert_ok!(MultiAssetDelegation::schedule_delegator_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			10,
		));

		// Assert
		// Check the delegator metadata
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert!(!metadata.delegator_unstake_requests.is_empty());
		let request = &metadata.delegator_unstake_requests[0];
		assert_eq!(request.asset_id, asset_id);
		assert_eq!(request.amount, 10);

		// Check the operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.delegation_count, 1);
		assert_eq!(operator_metadata.delegations.len(), 1);
		let operator_delegation = &operator_metadata.delegations[0];
		assert_eq!(operator_delegation.delegator, who.clone());
		assert_eq!(operator_delegation.amount, amount);
		assert_eq!(operator_delegation.asset_id, asset_id);

		assert_ok!(MultiAssetDelegation::cancel_delegator_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			10
		));

		// Assert
		// Check the delegator metadata
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert!(metadata.delegator_unstake_requests.is_empty());

		// Check the operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.delegation_count, 1);
		assert_eq!(operator_metadata.delegations.len(), 1);
		let operator_delegation = &operator_metadata.delegations[0];
		assert_eq!(operator_delegation.delegator, who.clone());
		assert_eq!(operator_delegation.amount, amount); // Amount added back
		assert_eq!(operator_delegation.asset_id, asset_id);
	});
}

#[test]
fn delegate_should_fail_if_not_enough_balance() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 10_000;

		create_and_mint_tokens(VDOT, who.clone(), amount);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id.clone(),
			amount - 20,
			None,
			None
		));

		assert_noop!(
			MultiAssetDelegation::delegate(
				RuntimeOrigin::signed(who.clone()),
				operator.clone(),
				asset_id.clone(),
				amount,
				Default::default(),
			),
			Error::<Runtime>::InsufficientBalance
		);
	});
}

#[test]
fn schedule_delegator_unstake_should_fail_if_no_delegation() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		create_and_mint_tokens(VDOT, who.clone(), amount);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id.clone(),
			amount,
			None,
			None
		));

		assert_noop!(
			MultiAssetDelegation::schedule_delegator_unstake(
				RuntimeOrigin::signed(who.clone()),
				operator.clone(),
				asset_id.clone(),
				amount,
			),
			Error::<Runtime>::NoActiveDelegation
		);
	});
}

#[test]
fn execute_delegator_unstake_should_fail_if_not_ready() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		create_and_mint_tokens(VDOT, who.clone(), amount);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Deposit, delegate and schedule unstake first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id.clone(),
			amount,
			None,
			None
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			amount,
			Default::default(),
		));

		assert_noop!(
			MultiAssetDelegation::cancel_delegator_unstake(
				RuntimeOrigin::signed(who.clone()),
				operator.clone(),
				asset_id.clone(),
				amount
			),
			Error::<Runtime>::NoBondLessRequest
		);

		assert_ok!(MultiAssetDelegation::schedule_delegator_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			amount,
		));

		assert_noop!(
			MultiAssetDelegation::execute_delegator_unstake(RuntimeOrigin::signed(who.clone()),),
			Error::<Runtime>::BondLessNotReady
		);
	});
}

#[test]
fn delegate_should_not_create_multiple_on_repeat_delegation() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;
		let additional_amount = 50;

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		create_and_mint_tokens(VDOT, who.clone(), amount + additional_amount);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id.clone(),
			amount + additional_amount,
			None,
			None
		));

		// Delegate first time
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			amount,
			Default::default(),
		));

		// Assert first delegation
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert!(metadata.deposits.get(&asset_id).is_some());
		assert_eq!(metadata.delegations.len(), 1);
		let delegation = &metadata.delegations[0];
		assert_eq!(delegation.operator, operator.clone());
		assert_eq!(delegation.amount, amount);
		assert_eq!(delegation.asset_id, asset_id);

		// Check the operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.delegation_count, 1);
		assert_eq!(operator_metadata.delegations.len(), 1);
		let operator_delegation = &operator_metadata.delegations[0];
		assert_eq!(operator_delegation.delegator, who.clone());
		assert_eq!(operator_delegation.amount, amount);
		assert_eq!(operator_delegation.asset_id, asset_id);

		// Delegate additional amount
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id.clone(),
			additional_amount,
			Default::default(),
		));

		// Check the updated operator metadata
		let updated_operator_metadata =
			MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(updated_operator_metadata.delegation_count, 1);
		assert_eq!(updated_operator_metadata.delegations.len(), 1);
		let updated_operator_delegation = &updated_operator_metadata.delegations[0];
		assert_eq!(updated_operator_delegation.delegator, who.clone());
		assert_eq!(updated_operator_delegation.amount, amount + additional_amount);
		assert_eq!(updated_operator_delegation.asset_id, asset_id);
	});
}

#[test]
fn delegate_exceeds_max_delegations() {
	new_test_ext().execute_with(|| {
		let who: AccountId = Bob.into();
		let amount = 100;

		// Setup max number of operators
		let mut operators = vec![];
		for i in 0..MaxDelegations::get() {
			let operator_account: AccountId = AccountId::new([i as u8; 32]);
			// Give operator enough balance to join
			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				operator_account.clone(),
				100_000
			));
			operators.push(operator_account.clone());
			assert_ok!(MultiAssetDelegation::join_operators(
				RuntimeOrigin::signed(operator_account),
				10_000
			));
		}

		// Create max number of delegations with same asset
		let asset_id = Asset::Custom(999);
		create_and_mint_tokens(999, who.clone(), amount * MaxDelegations::get() as u128);
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id.clone(),
			amount * MaxDelegations::get() as u128,
			None,
			None,
		));

		println!("Max delegations: {}", MaxDelegations::get());
		for i in 0..MaxDelegations::get() {
			assert_ok!(MultiAssetDelegation::delegate(
				RuntimeOrigin::signed(who.clone()),
				operators[i as usize].clone(),
				asset_id.clone(),
				1u128,
				Default::default(),
			));
		}

		let operator: AccountId = Charlie.into();
		// Give operator enough balance to join
		assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), operator.clone(), 100_000));
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));
		assert_noop!(
			MultiAssetDelegation::delegate(
				RuntimeOrigin::signed(who.clone()),
				operator.clone(),
				asset_id,
				amount,
				Default::default(),
			),
			Error::<Runtime>::MaxDelegationsExceeded
		);

		// Verify state
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegations.len() as u32, MaxDelegations::get());
	});
}

#[test]
fn delegate_insufficient_deposit() {
	new_test_ext().execute_with(|| {
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let deposit_amount = 100;
		let delegate_amount = deposit_amount + 1;
		let asset_id = Asset::Custom(USDC);

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		create_and_mint_tokens(USDC, who.clone(), deposit_amount);

		// Make deposit
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id.clone(),
			deposit_amount,
			None,
			None,
		));

		// Try to delegate more than deposited
		assert_noop!(
			MultiAssetDelegation::delegate(
				RuntimeOrigin::signed(who.clone()),
				operator.clone(),
				asset_id.clone(),
				delegate_amount,
				Default::default(),
			),
			Error::<Runtime>::InsufficientBalance
		);

		// Verify state remains unchanged
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegations.len(), 0);
		assert_eq!(metadata.deposits.len(), 1);
		assert_eq!(metadata.deposits.get(&asset_id).unwrap().amount, deposit_amount);
	});
}

#[test]
fn delegate_to_inactive_operator() {
	new_test_ext().execute_with(|| {
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let amount = 100;

		// Setup operator but make them inactive
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));
		assert_ok!(MultiAssetDelegation::go_offline(RuntimeOrigin::signed(operator.clone())));

		// Make deposit
		create_and_mint_tokens(USDC, who.clone(), amount);
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			Asset::Custom(USDC),
			amount,
			None,
			None,
		));

		// Try to delegate to inactive operator
		assert_noop!(
			MultiAssetDelegation::delegate(
				RuntimeOrigin::signed(who.clone()),
				operator.clone(),
				Asset::Custom(USDC),
				amount,
				Default::default(),
			),
			Error::<Runtime>::NotActiveOperator
		);

		// Verify state remains unchanged
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegations.len(), 0);
	});
}

#[test]
fn delegate_repeated_same_asset() {
	new_test_ext().execute_with(|| {
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let initial_amount = 100;
		let additional_amount = 50;

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Make deposit
		create_and_mint_tokens(USDC, who.clone(), initial_amount + additional_amount);
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			Asset::Custom(USDC),
			initial_amount + additional_amount,
			None,
			None,
		));

		// First delegation
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			Asset::Custom(USDC),
			initial_amount,
			Default::default(),
		));

		// Second delegation with same asset
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			Asset::Custom(USDC),
			additional_amount,
			Default::default(),
		));

		// Verify state
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegations.len(), 1);
		let delegation = &metadata.delegations[0];
		assert_eq!(delegation.amount, initial_amount + additional_amount);
		assert_eq!(delegation.operator, operator);
		assert_eq!(delegation.asset_id, Asset::Custom(USDC));

		// Verify operator state
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.delegation_count, 1);
		let operator_delegation = &operator_metadata.delegations[0];
		assert_eq!(operator_delegation.amount, initial_amount + additional_amount);
		assert_eq!(operator_delegation.delegator, who);
		assert_eq!(operator_delegation.asset_id, Asset::Custom(USDC));
	});
}

#[test]
fn delegate_multiple_assets_same_operator() {
	new_test_ext().execute_with(|| {
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let amount = 100;

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Make deposits for different assets
		for asset_id in [USDC, WETH].iter() {
			create_and_mint_tokens(*asset_id, who.clone(), amount);
			assert_ok!(MultiAssetDelegation::deposit(
				RuntimeOrigin::signed(who.clone()),
				Asset::Custom(*asset_id),
				amount,
				None,
				None,
			));
			assert_ok!(MultiAssetDelegation::delegate(
				RuntimeOrigin::signed(who.clone()),
				operator.clone(),
				Asset::Custom(*asset_id),
				amount,
				Default::default(),
			));
		}

		// Verify state
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegations.len(), 2);

		// Verify each delegation
		for (i, asset_id) in [USDC, WETH].iter().enumerate() {
			let delegation = &metadata.delegations[i];
			assert_eq!(delegation.amount, amount);
			assert_eq!(delegation.operator, operator);
			assert_eq!(delegation.asset_id, Asset::Custom(*asset_id));
		}

		// Verify operator state
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.delegation_count, 2);
		for (i, asset_id) in [USDC, WETH].iter().enumerate() {
			let operator_delegation = &operator_metadata.delegations[i];
			assert_eq!(operator_delegation.amount, amount);
			assert_eq!(operator_delegation.delegator, who);
			assert_eq!(operator_delegation.asset_id, Asset::Custom(*asset_id));
		}
	});
}

#[test]
fn delegate_zero_amount() {
	new_test_ext().execute_with(|| {
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Try to delegate zero amount
		assert_noop!(
			MultiAssetDelegation::delegate(
				RuntimeOrigin::signed(who.clone()),
				operator.clone(),
				Asset::Custom(USDC),
				0,
				Default::default(),
			),
			Error::<Runtime>::InvalidAmount
		);
	});
}

#[test]
fn delegate_with_no_deposit() {
	new_test_ext().execute_with(|| {
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let amount = 100;

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Try to delegate without deposit
		assert_noop!(
			MultiAssetDelegation::delegate(
				RuntimeOrigin::signed(who.clone()),
				operator.clone(),
				Asset::Custom(USDC),
				amount,
				Default::default(),
			),
			Error::<Runtime>::NotDelegator
		);

		// Verify state remains unchanged
		let metadata = MultiAssetDelegation::delegators(who.clone());
		assert_eq!(metadata.is_none(), true);
	});
}
