// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
//
// This file is part of pallet-evm-precompile-multi-asset-delegation package, originally developed by Purestake
// Inc. Pallet-evm-precompile-multi-asset-delegation package used in Tangle Network in terms of GPLv3.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::mock::*;
use crate::U256;
use frame_support::assert_ok;
use frame_support::traits::Currency;
use pallet_multi_asset_delegation::{Delegators, Operators};
use precompile_utils::testing::*;
use sp_core::{H160, H256};

// helper function
pub fn create_and_mint_tokens(
	asset_id: u32,
	recipient: <Runtime as frame_system::Config>::AccountId,
	amount: Balance,
) {
	assert_ok!(Assets::force_create(RuntimeOrigin::root(), asset_id, recipient, false, 1));
	assert_ok!(Assets::mint(RuntimeOrigin::signed(recipient), asset_id, recipient, amount));
}

#[test]
fn test_selector_less_than_four_bytes_reverts() {
	ExtBuilder::default().build().execute_with(|| {
		PrecompilesValue::get()
			.prepare_test(Alice, Precompile1, vec![1u8, 2, 3])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn test_unimplemented_selector_reverts() {
	ExtBuilder::default().build().execute_with(|| {
		PrecompilesValue::get()
			.prepare_test(Alice, Precompile1, vec![1u8, 2, 3, 4])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

// Test joining as an operator
#[test]
fn test_join_operators() {
	ExtBuilder::default().build().execute_with(|| {
		let account = sp_core::sr25519::Public::from(TestAccount::Alex);
		assert!(Operators::<Runtime>::get(account).is_none());

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::join_operators { bond_amount: U256::from(10_000) },
			)
			.execute_returns(());

		assert!(Operators::<Runtime>::get(account).is_some());
	});
}

// Test delegating assets
#[test]
fn test_delegate_assets() {
	ExtBuilder::default().build().execute_with(|| {
		let operator_account = sp_core::sr25519::Public::from(TestAccount::Bobo);
		let delegator_account = sp_core::sr25519::Public::from(TestAccount::Alex);

		Balances::make_free_balance_be(&operator_account, 20_000);
		Balances::make_free_balance_be(&delegator_account, 500);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account),
			10_000
		));

		create_and_mint_tokens(1, delegator_account, 500);

		// Deposit first
		assert_ok!(
			MultiAssetDelegation::deposit(RuntimeOrigin::signed(delegator_account), 1, 200,)
		);

		// Delegate assets
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: operator_account.into(),
					asset_id: U256::from(1),
					amount: U256::from(100),
				},
			)
			.execute_returns(());

		assert!(Delegators::<Runtime>::get(delegator_account).is_some());
	});
}

// Test scheduling a withdraw
#[test]
fn test_schedule_withdraw() {
	ExtBuilder::default().build().execute_with(|| {
		let operator_account = sp_core::sr25519::Public::from(TestAccount::Bobo);
		let delegator_account = sp_core::sr25519::Public::from(TestAccount::Alex);

		Balances::make_free_balance_be(&operator_account, 20_000);
		Balances::make_free_balance_be(&delegator_account, 500);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account),
			10_000
		));

		create_and_mint_tokens(1, delegator_account, 500);

		// Deposit first
		assert_ok!(
			MultiAssetDelegation::deposit(RuntimeOrigin::signed(delegator_account), 1, 200,)
		);

		// Delegate assets
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: operator_account.into(),
					asset_id: U256::from(1),
					amount: U256::from(100),
				},
			)
			.execute_returns(());

		assert!(Delegators::<Runtime>::get(delegator_account).is_some());

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::schedule_withdraw { asset_id: U256::from(1), amount: U256::from(100) },
			)
			.execute_returns(());

		// Assert
		let metadata = MultiAssetDelegation::delegators(delegator_account).unwrap();
		assert_eq!(metadata.deposits.get(&1), None);
		assert!(!metadata.withdraw_requests.is_empty());
	});
}

// Test executing a withdraw
#[test]
fn test_execute_withdraw() {
	ExtBuilder::default().build().execute_with(|| {
		let delegator_account = sp_core::sr25519::Public::from(TestAccount::Alex);
		let operator_account = sp_core::sr25519::Public::from(TestAccount::Bobo);

		Balances::make_free_balance_be(&operator_account, 20_000);
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator_account),
			10_000
		));

		create_and_mint_tokens(1, delegator_account, 500);

		// Deposit first
		assert_ok!(
			MultiAssetDelegation::deposit(RuntimeOrigin::signed(delegator_account), 1, 200,)
		);

		// Delegate assets
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::delegate {
					operator: operator_account.into(),
					asset_id: U256::from(1),
					amount: U256::from(100),
				},
			)
			.execute_returns(());

		assert!(Delegators::<Runtime>::get(delegator_account).is_some());

		// Schedule a withdraw
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::schedule_withdraw { asset_id: U256::from(1), amount: U256::from(100) },
			)
			.execute_returns(());

		// Assert
		let metadata = MultiAssetDelegation::delegators(delegator_account).unwrap();
		assert_eq!(metadata.deposits.get(&1), None);
		assert!(!metadata.withdraw_requests.is_empty());

		// Roll to the block where the withdraw can be executed
		roll_to(3);

		// Execute the withdraw
		PrecompilesValue::get()
			.prepare_test(TestAccount::Alex, H160::from_low_u64_be(1), PCall::execute_withdraw {})
			.execute_returns(());

		// Assert
		let metadata = MultiAssetDelegation::delegators(delegator_account).unwrap();
		assert_eq!(metadata.deposits.get(&1), None);
	});
}
