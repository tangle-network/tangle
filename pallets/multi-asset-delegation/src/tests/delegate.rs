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
#![allow(clippy::all)]
use super::*;
use crate::{types::*, CurrentRound, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::Percent;
use std::collections::BTreeMap;

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
		assert_ok!(MultiAssetDelegation::deposit(RuntimeOrigin::signed(who), asset_id, amount,));

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
fn schedule_delegator_unstake_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let operator = 2;
		let asset_id = VDOT;
		let amount = 100;

		create_and_mint_tokens(VDOT, who, amount);

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator), 10_000));

		// Deposit and delegate first
		assert_ok!(MultiAssetDelegation::deposit(RuntimeOrigin::signed(who), asset_id, amount,));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		assert_ok!(MultiAssetDelegation::schedule_delegator_unstake(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		// Assert
		// Check the delegator metadata
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(!metadata.delegator_unstake_requests.is_empty());
		let request = &metadata.delegator_unstake_requests[0];
		assert_eq!(request.asset_id, asset_id);
		assert_eq!(request.amount, amount);

		// Check the operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator).unwrap();
		assert_eq!(operator_metadata.delegation_count, 0);
		assert_eq!(operator_metadata.delegations.len(), 0);
	});
}

#[test]
fn execute_delegator_unstake_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let operator = 2;
		let asset_id = VDOT;
		let amount = 100;

		create_and_mint_tokens(VDOT, who, amount);

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator), 10_000));

		// Deposit, delegate and schedule unstake first
		assert_ok!(MultiAssetDelegation::deposit(RuntimeOrigin::signed(who), asset_id, amount,));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));
		assert_ok!(MultiAssetDelegation::schedule_delegator_unstake(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		// Simulate round passing
		CurrentRound::<Test>::put(10);

		assert_ok!(MultiAssetDelegation::execute_delegator_unstake(RuntimeOrigin::signed(who),));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(metadata.delegator_unstake_requests.is_empty());
		assert!(metadata.deposits.get(&asset_id).is_some());
		assert_eq!(metadata.deposits.get(&asset_id).unwrap(), &amount);
	});
}

#[test]
fn cancel_delegator_unstake_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let operator = 2;
		let asset_id = VDOT;
		let amount = 100;

		create_and_mint_tokens(VDOT, who, amount);

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator), 10_000));

		// Deposit, delegate and schedule unstake first
		assert_ok!(MultiAssetDelegation::deposit(RuntimeOrigin::signed(who), asset_id, amount,));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		// ensure the storage is correct
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(metadata.delegator_unstake_requests.is_empty());
		assert!(metadata.delegations.len() == 1);
		let delegation = metadata.delegations.first().unwrap();
		assert_eq!(delegation.operator, operator);
		assert_eq!(delegation.amount, amount);
		assert_eq!(delegation.asset_id, asset_id);

		assert_ok!(MultiAssetDelegation::schedule_delegator_unstake(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(metadata.delegations.is_empty());

		// ensure the storage is correct
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(!metadata.delegator_unstake_requests.is_empty());

		assert_ok!(MultiAssetDelegation::cancel_delegator_unstake(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount
		));

		// Assert
		// Check the delegator metadata
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(metadata.delegator_unstake_requests.is_empty());

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
fn cancel_delegator_unstake_should_update_already_existing() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let operator = 2;
		let asset_id = VDOT;
		let amount = 100;

		create_and_mint_tokens(VDOT, who, amount);

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator), 10_000));

		// Deposit, delegate and schedule unstake first
		assert_ok!(MultiAssetDelegation::deposit(RuntimeOrigin::signed(who), asset_id, amount,));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		// ensure the storage is correct
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(metadata.delegator_unstake_requests.is_empty());
		assert!(metadata.delegations.len() == 1);
		let delegation = metadata.delegations.first().unwrap();
		assert_eq!(delegation.operator, operator);
		assert_eq!(delegation.amount, amount);
		assert_eq!(delegation.asset_id, asset_id);

		assert_ok!(MultiAssetDelegation::schedule_delegator_unstake(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			10,
		));
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(metadata.delegations.len() == 1);
		let delegation = metadata.delegations.first().unwrap();
		assert_eq!(delegation.operator, operator);
		assert_eq!(delegation.amount, amount - 10);
		assert_eq!(delegation.asset_id, asset_id);

		// ensure the storage is correct
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(!metadata.delegator_unstake_requests.is_empty());

		assert_ok!(MultiAssetDelegation::cancel_delegator_unstake(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			10
		));

		// Assert
		// Check the delegator metadata
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(metadata.delegator_unstake_requests.is_empty());

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
			asset_id,
			amount - 20,
		));

		assert_noop!(
			MultiAssetDelegation::delegate(RuntimeOrigin::signed(who), operator, asset_id, amount,),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn schedule_delegator_unstake_should_fail_if_no_delegation() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let operator = 2;
		let asset_id = VDOT;
		let amount = 100;

		create_and_mint_tokens(VDOT, who, amount);

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator), 10_000));

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(RuntimeOrigin::signed(who), asset_id, amount,));

		assert_noop!(
			MultiAssetDelegation::schedule_delegator_unstake(
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
fn execute_delegator_unstake_should_fail_if_not_ready() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let operator = 2;
		let asset_id = VDOT;
		let amount = 100;

		create_and_mint_tokens(VDOT, who, amount);

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator), 10_000));

		// Deposit, delegate and schedule unstake first
		assert_ok!(MultiAssetDelegation::deposit(RuntimeOrigin::signed(who), asset_id, amount,));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		assert_noop!(
			MultiAssetDelegation::cancel_delegator_unstake(
				RuntimeOrigin::signed(who),
				operator,
				asset_id,
				amount
			),
			Error::<Test>::NoBondLessRequest
		);

		assert_ok!(MultiAssetDelegation::schedule_delegator_unstake(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		assert_noop!(
			MultiAssetDelegation::execute_delegator_unstake(RuntimeOrigin::signed(who),),
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
			asset_id,
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

#[test]
fn distribute_rewards_should_work() {
	new_test_ext().execute_with(|| {
		let round = 1;
		let operator = 1;
		let delegator = 2;
		let asset_id = 1;
		let amount = 100;
		let cap = 50;
		let apy = Percent::from_percent(10); // 10%

		let initial_balance = Balances::free_balance(delegator);

		// Set up reward configuration
		let reward_config = RewardConfig {
			configs: {
				let mut map = BTreeMap::new();
				map.insert(asset_id, RewardConfigForAssetVault { apy, cap });
				map
			},
			whitelisted_blueprint_ids: vec![],
		};
		RewardConfigStorage::<Test>::put(reward_config);

		// Set up asset vault lookup
		AssetLookupRewardVaults::<Test>::insert(asset_id, asset_id);

		// Add delegation information
		AtStake::<Test>::insert(
			round,
			operator,
			OperatorSnapshot {
				delegations: vec![DelegatorBond { delegator, amount, asset_id }],
				stake: amount,
			},
		);

		// Distribute rewards
		assert_ok!(MultiAssetDelegation::distribute_rewards(round));

		// Check if rewards were distributed correctly
		let balance = Balances::free_balance(delegator);

		// Calculate the percentage of the cap that the user is staking
		let staking_percentage = amount.saturating_mul(100) / cap;
		// Calculate the expected reward based on the staking percentage
		let expected_reward = apy.mul_floor(amount);
		let calculated_reward = expected_reward.saturating_mul(staking_percentage) / 100;

		assert_eq!(balance - initial_balance, calculated_reward);
	});
}

#[test]
fn distribute_rewards_with_multiple_delegators_and_operators_should_work() {
	new_test_ext().execute_with(|| {
		let round = 1;

		let operator1 = 1;
		let operator2 = 2;
		let delegator1 = 3;
		let delegator2 = 4;

		let asset_id1 = 1;
		let asset_id2 = 2;

		let amount1 = 100;
		let amount2 = 200;

		let cap1 = 50;
		let cap2 = 150;

		let apy1 = Percent::from_percent(10); // 10%
		let apy2 = Percent::from_percent(20); // 20%

		let initial_balance1 = Balances::free_balance(delegator1);
		let initial_balance2 = Balances::free_balance(delegator2);

		// Set up reward configuration
		let reward_config = RewardConfig {
			configs: {
				let mut map = BTreeMap::new();
				map.insert(asset_id1, RewardConfigForAssetVault { apy: apy1, cap: cap1 });
				map.insert(asset_id2, RewardConfigForAssetVault { apy: apy2, cap: cap2 });
				map
			},
			whitelisted_blueprint_ids: vec![],
		};
		RewardConfigStorage::<Test>::put(reward_config);

		// Set up asset vault lookup
		AssetLookupRewardVaults::<Test>::insert(asset_id1, asset_id1);
		AssetLookupRewardVaults::<Test>::insert(asset_id2, asset_id2);

		// Add delegation information
		AtStake::<Test>::insert(
			round,
			operator1,
			OperatorSnapshot {
				delegations: vec![DelegatorBond {
					delegator: delegator1,
					amount: amount1,
					asset_id: asset_id1,
				}],
				stake: amount1,
			},
		);

		AtStake::<Test>::insert(
			round,
			operator2,
			OperatorSnapshot {
				delegations: vec![DelegatorBond {
					delegator: delegator2,
					amount: amount2,
					asset_id: asset_id2,
				}],
				stake: amount2,
			},
		);

		// Distribute rewards
		assert_ok!(MultiAssetDelegation::distribute_rewards(round));

		// Check if rewards were distributed correctly
		let balance1 = Balances::free_balance(delegator1);
		let balance2 = Balances::free_balance(delegator2);

		// Calculate the percentage of the cap that each user is staking
		let staking_percentage1 = amount1.saturating_mul(100) / cap1;
		let staking_percentage2 = amount2.saturating_mul(100) / cap2;

		// Calculate the expected rewards based on the staking percentages
		let expected_reward1 = apy1.mul_floor(amount1);
		let calculated_reward1 = expected_reward1.saturating_mul(staking_percentage1) / 100;

		let expected_reward2 = apy2.mul_floor(amount2);
		let calculated_reward2 = expected_reward2.saturating_mul(staking_percentage2) / 100;

		assert_eq!(balance1 - initial_balance1, calculated_reward1);
		assert_eq!(balance2 - initial_balance2, calculated_reward2);
	});
}
