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
use extra::CheckNominatedRestaked;
use frame_support::{
	assert_err, assert_noop, assert_ok,
	dispatch::DispatchInfo,
	pallet_prelude::{InvalidTransaction, TransactionValidityError},
	traits::Hooks,
};
use sp_keyring::AccountKeyring::{Alice, Bob, Charlie, Dave};
use sp_runtime::traits::SignedExtension;
use tangle_primitives::services::Asset;

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
		<Session as Hooks<BlockNumber>>::on_initialize(2);
		<Staking as Hooks<BlockNumber>>::on_initialize(2);
		<Session as Hooks<BlockNumber>>::on_finalize(2);
		<Staking as Hooks<BlockNumber>>::on_finalize(2);
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
		assert_eq!(delegation.asset, Asset::Custom(TNT));
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
		<Session as Hooks<BlockNumber>>::on_initialize(2);
		<Staking as Hooks<BlockNumber>>::on_initialize(2);
		<Session as Hooks<BlockNumber>>::on_finalize(2);
		<Staking as Hooks<BlockNumber>>::on_finalize(2);

		// Verify initial staking state
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

		// Verify delegation state
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegations.len(), 1);
		let delegation = &metadata.delegations[0];
		assert_eq!(delegation.operator, operator);
		assert_eq!(delegation.amount, delegate_amount);
		assert!(delegation.is_nomination);
		assert_eq!(delegation.asset, Asset::Custom(TNT));

		// Check operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.delegation_count, 1);
		let operator_delegation = &operator_metadata.delegations[0];
		assert_eq!(operator_delegation.delegator, who.clone());
		assert_eq!(operator_delegation.amount, delegate_amount);

		// Check locks before unbond attempt
		let locks = pallet_balances::Pallet::<Runtime>::locks(&who);
		assert_eq!(locks.len(), 2);
		assert_eq!(&locks[0].id, b"staking ");
		assert_eq!(locks[0].amount, amount);
		assert_eq!(&locks[1].id, b"delegate");
		assert_eq!(locks[1].amount, delegate_amount);
		let call = RuntimeCall::Staking(pallet_staking::Call::unbond { value: amount });

		// Try to unbond from the staking pallet - should fail
		assert_err!(
			CheckNominatedRestaked::<Runtime>::new().validate(
				&who,
				&call,
				&DispatchInfo::default(),
				0
			),
			TransactionValidityError::Invalid(InvalidTransaction::Custom(1))
		);

		// Verify state remains unchanged after failed unbond
		let ledger = Staking::ledger(sp_staking::StakingAccount::Stash(who.clone())).unwrap();
		assert_eq!(ledger.active, amount);
		assert_eq!(ledger.total, amount);
		assert_eq!(ledger.unlocking.len(), 0);

		// Verify delegation state remains unchanged
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegations.len(), 1);
		let delegation = &metadata.delegations[0];
		assert_eq!(delegation.operator, operator);
		assert_eq!(delegation.amount, delegate_amount);
		assert!(delegation.is_nomination);

		// Verify locks remain unchanged
		let locks = pallet_balances::Pallet::<Runtime>::locks(&who);
		assert_eq!(locks.len(), 2);
		assert_eq!(&locks[0].id, b"staking ");
		assert_eq!(locks[0].amount, amount);
		assert_eq!(&locks[1].id, b"delegate");
		assert_eq!(locks[1].amount, delegate_amount);
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
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(who.clone()),
			total_nomination,
			pallet_staking::RewardDestination::Staked
		));
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![operator.clone()]));

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
		assert_eq!(metadata.delegations.len(), 1);
		let delegation = &metadata.delegations[0];
		assert_eq!(delegation.operator, operator.clone());
		assert_eq!(delegation.amount, first_restake + second_restake);

		// Check operator metadata
		let operator_metadata = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(operator_metadata.delegation_count, 1);
		let operator_delegation = &operator_metadata.delegations[0];
		assert_eq!(operator_delegation.delegator, who.clone());
		assert_eq!(operator_delegation.amount, first_restake + second_restake);
	});
}

#[test]
fn native_restake_exceeding_nomination_amount() {
	new_test_ext().execute_with(|| {
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
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(who.clone()),
			nomination_amount,
			pallet_staking::RewardDestination::Staked
		));
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![operator.clone()]));

		// Try to restake more than nominated
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
fn native_restake_with_no_active_nomination() {
	new_test_ext().execute_with(|| {
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let amount = 100;

		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Try to restake without nomination
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
fn native_restake_to_non_operator() {
	new_test_ext().execute_with(|| {
		let who: AccountId = Bob.into();
		let non_operator: AccountId = Charlie.into();
		let amount = 100;

		// Setup nomination
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(who.clone()),
			amount,
			pallet_staking::RewardDestination::Staked
		));
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![
			non_operator.clone()
		]));

		// Try to restake to non-operator
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
fn native_restake_and_unstake_flow() {
	new_test_ext().execute_with(|| {
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let amount = 100;
		let unstake_amount = 40;

		// Setup
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(who.clone()),
			amount,
			pallet_staking::RewardDestination::Staked
		));
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![operator.clone()]));

		// Restake
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			amount,
			Default::default(),
		));

		// Schedule unstake
		assert_ok!(MultiAssetDelegation::schedule_nomination_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			unstake_amount,
			Default::default(),
		));

		// Verify unstake request
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegator_unstake_requests.len(), 1);
		let request = &metadata.delegator_unstake_requests[0];
		assert_eq!(request.operator, operator.clone());
		assert_eq!(request.amount, unstake_amount);

		// Move to next round
		CurrentRound::<Runtime>::put(10);

		// Execute unstake
		assert_ok!(MultiAssetDelegation::execute_nomination_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
		));

		// Verify final state
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegations.len(), 1);
		let delegation = &metadata.delegations[0];
		assert_eq!(delegation.amount, amount - unstake_amount);
	});
}

#[test]
fn native_restake_zero_amount() {
	new_test_ext().execute_with(|| {
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let amount = 100;

		// Setup
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(who.clone()),
			amount,
			pallet_staking::RewardDestination::Staked
		));
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![operator.clone()]));

		// Try to restake zero amount
		assert_noop!(
			MultiAssetDelegation::delegate_nomination(
				RuntimeOrigin::signed(who.clone()),
				operator.clone(),
				0,
				Default::default(),
			),
			Error::<Runtime>::InvalidAmount
		);
	});
}

#[test]
fn native_restake_concurrent_operations() {
	new_test_ext().execute_with(|| {
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let amount = 100;

		// Setup
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(who.clone()),
			amount,
			pallet_staking::RewardDestination::Staked
		));
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![operator.clone()]));

		// Perform multiple operations in same block
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			50,
			Default::default(),
		));
		assert_ok!(MultiAssetDelegation::schedule_nomination_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			20,
			Default::default(),
		));
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			30,
			Default::default(),
		));

		// Verify final state
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		let delegation = &metadata.delegations[0];
		assert_eq!(delegation.amount, 80); // 50 + 30
		assert_eq!(metadata.delegator_unstake_requests.len(), 1);
	});
}

#[test]
fn native_restake_early_unstake_execution_fails() {
	new_test_ext().execute_with(|| {
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let amount = 100;
		let unstake_amount = 40;

		// Setup
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(who.clone()),
			amount,
			pallet_staking::RewardDestination::Staked
		));
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![operator.clone()]));

		// Restake
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			amount,
			Default::default(),
		));

		// Verify delegation state after restaking
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegations.len(), 1);
		let delegation = &metadata.delegations[0];
		assert_eq!(delegation.operator, operator);
		assert_eq!(delegation.amount, amount);
		assert!(delegation.is_nomination);
		assert_eq!(metadata.delegator_unstake_requests.len(), 0);

		// Schedule unstake
		assert_ok!(MultiAssetDelegation::schedule_nomination_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			unstake_amount,
			Default::default(),
		));

		// Verify unstake request state
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegator_unstake_requests.len(), 1);
		let request = &metadata.delegator_unstake_requests[0];
		assert_eq!(request.operator, operator);
		assert_eq!(request.amount, unstake_amount);
		assert!(request.is_nomination);

		// Try to execute unstake immediately - should fail
		assert_noop!(
			MultiAssetDelegation::execute_nomination_unstake(
				RuntimeOrigin::signed(who.clone()),
				operator.clone(),
			),
			Error::<Runtime>::BondLessNotReady
		);

		// Verify state remains unchanged after failed execution
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegations.len(), 1);
		let delegation = &metadata.delegations[0];
		assert_eq!(delegation.operator, operator);
		assert_eq!(delegation.amount, amount);
		assert!(delegation.is_nomination);
		assert_eq!(metadata.delegator_unstake_requests.len(), 1);
		let request = &metadata.delegator_unstake_requests[0];
		assert_eq!(request.operator, operator);
		assert_eq!(request.amount, unstake_amount);
		assert!(request.is_nomination);
	});
}

#[test]
fn native_restake_cancel_unstake() {
	new_test_ext().execute_with(|| {
		let who: AccountId = Bob.into();
		let operator: AccountId = Alice.into();
		let amount = 100;
		let unstake_amount = 40;

		// Setup
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(who.clone()),
			amount,
			pallet_staking::RewardDestination::Staked
		));
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![operator.clone()]));

		// Restake
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			amount,
			Default::default(),
		));

		// Schedule unstake
		assert_ok!(MultiAssetDelegation::schedule_nomination_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			unstake_amount,
			Default::default(),
		));

		// Verify unstake request exists
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegator_unstake_requests.len(), 1);
		let request = &metadata.delegator_unstake_requests[0];
		assert_eq!(request.operator, operator.clone());
		assert_eq!(request.amount, unstake_amount);

		// Cancel unstake request
		assert_ok!(MultiAssetDelegation::cancel_nomination_unstake(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
		));

		// Verify unstake request is removed
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegator_unstake_requests.len(), 0);

		// Verify delegation amount remains unchanged
		let delegation = &metadata.delegations[0];
		assert_eq!(delegation.amount, amount);

		// Try to execute cancelled unstake - should fail
		assert_noop!(
			MultiAssetDelegation::execute_nomination_unstake(
				RuntimeOrigin::signed(who.clone()),
				operator.clone(),
			),
			Error::<Runtime>::NoBondLessRequest
		);
	});
}

#[test]
fn proxy_unbond_should_fail_if_delegated_nomination() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Dave.into();
		let proxy: AccountId = Charlie.into();
		let validator = Staking::invulnerables()[0].clone();
		let operator: AccountId = Alice.into();
		let amount = 100_000;
		let delegate_amount = amount / 2;

		// Setup proxy with Staking type
		assert_ok!(Proxy::add_proxy(
			RuntimeOrigin::signed(who.clone()),
			proxy.clone(),
			ProxyType::Staking,
			0
		));

		// Bond Some TNT
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(who.clone()),
			amount,
			pallet_staking::RewardDestination::Staked
		));
		// Nominate the validator
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![validator.clone()]));

		System::set_block_number(2);
		<Session as Hooks<BlockNumber>>::on_initialize(2);
		<Staking as Hooks<BlockNumber>>::on_initialize(2);
		<Session as Hooks<BlockNumber>>::on_finalize(2);
		<Staking as Hooks<BlockNumber>>::on_finalize(2);

		// Set up operator and delegation
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		// Restake through direct call
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			delegate_amount,
			Default::default(),
		));

		// Try to unbond through proxy - should fail
		let call = RuntimeCall::Staking(pallet_staking::Call::unbond { value: amount });
		let proxy_call = RuntimeCall::Proxy(pallet_proxy::Call::proxy {
			real: who.clone(),
			force_proxy_type: None,
			call: Box::new(call.clone()),
		});

		assert_err!(
			CheckNominatedRestaked::<Runtime>::new().validate(
				&proxy,
				&proxy_call,
				&DispatchInfo::default(),
				0
			),
			TransactionValidityError::Invalid(InvalidTransaction::Custom(1))
		);

		// Verify state remains unchanged
		let ledger = Staking::ledger(sp_staking::StakingAccount::Stash(who.clone())).unwrap();
		assert_eq!(ledger.active, amount);
		assert_eq!(ledger.total, amount);
		assert_eq!(ledger.unlocking.len(), 0);
	});
}

#[test]
fn batch_unbond_should_fail_if_delegated_nomination() {
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
		<Session as Hooks<BlockNumber>>::on_initialize(2);
		<Staking as Hooks<BlockNumber>>::on_initialize(2);
		<Session as Hooks<BlockNumber>>::on_finalize(2);
		<Staking as Hooks<BlockNumber>>::on_finalize(2);

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

		// Try to unbond through batch call - should fail
		let unbond_call = RuntimeCall::Staking(pallet_staking::Call::unbond { value: amount });
		let batch_call =
			RuntimeCall::Utility(pallet_utility::Call::batch { calls: vec![unbond_call] });

		assert_err!(
			CheckNominatedRestaked::<Runtime>::new().validate(
				&who,
				&batch_call,
				&DispatchInfo::default(),
				0
			),
			TransactionValidityError::Invalid(InvalidTransaction::Custom(1))
		);

		// Verify state remains unchanged
		let ledger = Staking::ledger(sp_staking::StakingAccount::Stash(who.clone())).unwrap();
		assert_eq!(ledger.active, amount);
		assert_eq!(ledger.total, amount);
		assert_eq!(ledger.unlocking.len(), 0);
	});
}

#[test]
fn proxy_batch_unbond_should_fail_if_delegated_nomination() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Dave.into();
		let proxy: AccountId = Charlie.into();
		let validator = Staking::invulnerables()[0].clone();
		let operator: AccountId = Alice.into();
		let amount = 100_000;
		let delegate_amount = amount / 2;

		// Setup proxy with Staking type
		assert_ok!(Proxy::add_proxy(
			RuntimeOrigin::signed(who.clone()),
			proxy.clone(),
			ProxyType::Staking,
			0
		));

		// Bond Some TNT
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(who.clone()),
			amount,
			pallet_staking::RewardDestination::Staked
		));
		// Nominate the validator
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(who.clone()), vec![validator.clone()]));

		System::set_block_number(2);
		<Session as Hooks<BlockNumber>>::on_initialize(2);
		<Staking as Hooks<BlockNumber>>::on_initialize(2);
		<Session as Hooks<BlockNumber>>::on_finalize(2);
		<Staking as Hooks<BlockNumber>>::on_finalize(2);

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

		// Try to unbond through proxy batch call - should fail
		let unbond_call = RuntimeCall::Staking(pallet_staking::Call::unbond { value: amount });
		let batch_call =
			RuntimeCall::Utility(pallet_utility::Call::batch { calls: vec![unbond_call] });
		let proxy_batch_call = RuntimeCall::Proxy(pallet_proxy::Call::proxy {
			real: who.clone(),
			force_proxy_type: None,
			call: Box::new(batch_call.clone()),
		});

		assert_err!(
			CheckNominatedRestaked::<Runtime>::new().validate(
				&proxy,
				&proxy_batch_call,
				&DispatchInfo::default(),
				0
			),
			TransactionValidityError::Invalid(InvalidTransaction::Custom(1))
		);

		// Verify state remains unchanged
		let ledger = Staking::ledger(sp_staking::StakingAccount::Stash(who.clone())).unwrap();
		assert_eq!(ledger.active, amount);
		assert_eq!(ledger.total, amount);
		assert_eq!(ledger.unlocking.len(), 0);

		// Verify delegation state remains unchanged
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert_eq!(metadata.delegations.len(), 1);
		let delegation = &metadata.delegations[0];
		assert_eq!(delegation.operator, operator);
		assert_eq!(delegation.amount, delegate_amount);
		assert!(delegation.is_nomination);
	});
}
