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
use frame_support::{
	assert_noop, assert_ok,
	traits::{OnFinalize, OnInitialize},
};
use sp_keyring::AccountKeyring::{Alice, Bob, Dave};
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
fn native_restaking_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Dave.into();
		let validator = Staking::invulnerables()[0].clone();
		let operator: AccountId = Alice.into();
		let amount = 100_000;
		let delegate_amount = amount / 2;
		// Bond Some TNT
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(who.clone()),
			amount,
			pallet_staking::RewardDestination::Staked
		));
		// Nominate the validator
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![validator.clone()]));

		System::set_block_number(2);
		Session::on_initialize(2);
		Staking::on_initialize(2);
		Session::on_finalize(2);
		Staking::on_finalize(2);
		// Assert
		let ledger = Staking::ledger(sp_staking::StakingAccount::Stash(who.clone())).unwrap();
		assert_eq!(ledger.active, amount);
		assert_eq!(ledger.total, amount);
		assert_eq!(ledger.unlocking.len(), 0);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Restake
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			delegate_amount,
			Default::default(),
		));
		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegations.len(), 1);
		let delegation = &metadata.delegations[0];
		assert_eq!(delegation.operator, operator.clone());
		assert_eq!(delegation.amount, delegate_amount);
		assert_eq!(delegation.asset_id, Asset::Custom(TNT));
		// Check the locks
		let locks = pallet_balances::Pallet::<Runtime>::locks(&who);
		// 1 lock for the staking
		// 1 lock for the delegation
		assert_eq!(locks.len(), 2);
		assert_eq!(&locks[0].id, b"staking ");
		assert_eq!(locks[0].amount, amount);
		assert_eq!(&locks[1].id, b"delegate");
		assert_eq!(locks[1].amount, delegate_amount);
	});
}

#[test]
fn unbond_should_fail_if_delegated_nomination() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Dave.into();
		let validator = Staking::invulnerables()[0].clone();
		let operator: AccountId = Alice.into();
		let amount = 100_000;
		let delegate_amount = amount / 2;
		// Bond Some TNT
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(who.clone()),
			amount,
			pallet_staking::RewardDestination::Staked
		));
		// Nominate the validator
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![validator.clone()]));

		System::set_block_number(2);
		Session::on_initialize(2);
		Staking::on_initialize(2);
		Session::on_finalize(2);
		Staking::on_finalize(2);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Restake
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			delegate_amount,
			Default::default(),
		));

		// Try to unbond from the staking pallet
		assert_ok!(Staking::unbond(RuntimeOrigin::signed(who.clone()), amount));

		// Assert
		let ledger = Staking::ledger(sp_staking::StakingAccount::Stash(who.clone())).unwrap();
		assert_eq!(ledger.active, 0);
		assert_eq!(ledger.total, amount);
		assert_eq!(ledger.unlocking.len(), 1);

		todo!("This test should fail but it is passing");
	});
}
