// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
//
// This file is part of pallet-evm-precompile-proxy package, originally developed by Purestake
// Inc. Pallet-evm-precompile-proxy package used in Tangle Network in terms of GPLv3.

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

use crate::mock::{
	mock_pub_key, roll_to, ExtBuilder, PCall, PrecompilesValue, Runtime, TestAccount,
};

use pallet_vesting::Vesting;
use precompile_utils::testing::*;
use sp_core::{H160, H256};

pub fn convert_h160_to_h256(address: H160) -> H256 {
	let mut bytes = [0u8; 32];
	bytes[12..32].copy_from_slice(&address.0);
	H256(bytes)
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

// Test unlocking any vested funds of the sender account.
#[test]
fn test_claim_vesting_schedule() {
	ExtBuilder::default().build().execute_with(|| {
		let account = sp_core::sr25519::Public::from(TestAccount::Alex);
		let schedules = Vesting::<Runtime>::get(account).unwrap();
		assert!(!schedules.is_empty());
		// VestingInfo { locked: 500000, per_block: 5000, starting_block: 10 }
		// Account is initially funded with 1_000_000 tokens and 500000 tokens are vested
		// Therefore total usable balance should be 500000 tokens
		let balance = pallet_balances::Pallet::<Runtime>::usable_balance(account);
		assert_eq!(balance, 500000u64);

		// Now lets roll block to 110.
		// Account should be fully vested after 110 blocks.
		roll_to(110);
		PrecompilesValue::get()
			.prepare_test(TestAccount::Alex, H160::from_low_u64_be(1), PCall::vest {})
			.execute_returns(());

		// Total usable balance after vesting should be 1_000_000 tokens.
		let balance = pallet_balances::Pallet::<Runtime>::usable_balance(account);
		assert_eq!(balance, 1_000_000u64);
	});
}

#[test]
fn non_vested_cannot_vest() {
	ExtBuilder::default().build().execute_with(|| {
		let non_vested_account = TestAccount::Dave;
		assert_eq!(pallet_vesting::Pallet::<Runtime>::vesting(
			sp_core::sr25519::Public::from(non_vested_account.clone())), None);

		let error_msg = "Dispatched call failed with error: Module(ModuleError { index: 4, error: [0, 0, 0, 0], message: Some(\"NotVesting\") })";
		// non_vested_account should not be able to vest.
		PrecompilesValue::get()
			.prepare_test(
				non_vested_account,
				H160::from_low_u64_be(1),
				PCall::vest {},
			)
			.execute_reverts(|output| output == error_msg.as_bytes());

	});
}

// Test unlocking any vested funds of a target account(substrate account).
#[test]
fn test_vest_other_substrate() {
	ExtBuilder::default().build().execute_with(|| {
		let target_account = sp_core::sr25519::Public::from(TestAccount::Bobo);
		let schedules = Vesting::<Runtime>::get(target_account).unwrap();
		assert!(!schedules.is_empty());

		// VestingInfo { locked: 500000, per_block: 5000, starting_block: 10 }
		// Account is initially funded with 1_000_000 tokens and 500000 tokens are vested
		// Therefore total usable balance should be 500000 tokens
		let balance = pallet_balances::Pallet::<Runtime>::usable_balance(target_account);
		assert_eq!(balance, 500000u64);

		// Now lets roll block to 110.
		// Account should be fully vested after 110 blocks.
		// We will use Alex to vest Bobo's account

		roll_to(110);
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::vest_other { target: target_account.into() },
			)
			.execute_returns(());

		// Total usable balance after vesting should be 1_000_000 tokens.
		let balance = pallet_balances::Pallet::<Runtime>::usable_balance(target_account);
		assert_eq!(balance, 1_000_000u64);
	});
}

// Test unlocking any vested funds of a target account(evm account).
#[test]
fn test_vest_other_evm() {
	ExtBuilder::default().build().execute_with(|| {
		let target_evm_address = H160::repeat_byte(0x03);
		let target_account = convert_h160_to_h256(target_evm_address);
		// AccountId mapped for target evm address
		let mapped_target_address = sp_core::sr25519::Public::from(TestAccount::Charlie);

		let schedules = Vesting::<Runtime>::get(mapped_target_address).unwrap();
		assert!(!schedules.is_empty());

		// VestingInfo { locked: 500000, per_block: 5000, starting_block: 10 }
		// Account is initially funded with 1_000_000 tokens and 500000 tokens are vested
		// Therefore total usable balance should be 500000 tokens
		let balance = pallet_balances::Pallet::<Runtime>::usable_balance(mapped_target_address);
		assert_eq!(balance, 500000u64);

		// Now lets roll block to 110.
		// Account should be fully vested after 110 blocks.
		// We will use Alex to vest target account

		roll_to(110);
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Bobo,
				H160::from_low_u64_be(1),
				PCall::vest_other { target: target_account.into() },
			)
			.execute_returns(());

		// Total usable balance after vesting should be 1_000_000 tokens.
		let balance = pallet_balances::Pallet::<Runtime>::usable_balance(mapped_target_address);
		assert_eq!(balance, 1_000_000u64);
	});
}

#[test]
fn non_vested_cannot_vest_other() {
	ExtBuilder::default().build().execute_with(|| {
		let non_vested_account = TestAccount::Dave;
		assert_eq!(pallet_vesting::Pallet::<Runtime>::vesting(
			sp_core::sr25519::Public::from(non_vested_account.clone())), None);

		let target = mock_pub_key(6);
		let error_msg = "Dispatched call failed with error: Module(ModuleError { index: 4, error: [0, 0, 0, 0], message: Some(\"NotVesting\") })";
		// non_vested_account should not be able to vest other.
		PrecompilesValue::get()
			.prepare_test(
				non_vested_account,
				H160::from_low_u64_be(1),
				PCall::vest_other { target: target.into() },
			)
			.execute_reverts(|output| output == error_msg.as_bytes());

	});
}

// Test vested transfer.
#[test]
fn test_vested_transfer() {
	ExtBuilder::default().build().execute_with(|| {
		let schedules =
			Vesting::<Runtime>::get(sp_core::sr25519::Public::from(TestAccount::Bobo)).unwrap();
		assert!(!schedules.is_empty());

		let target_evm_address = H160::repeat_byte(0x05);
		let target = convert_h160_to_h256(target_evm_address);
		// AccountId mapped for target evm address
		let mapped_target_address = sp_core::sr25519::Public::from(TestAccount::Eve);

		// Target account should be non vested account
		assert_eq!(pallet_vesting::Pallet::<Runtime>::vesting(mapped_target_address), None);

		// Lets transfer Bobo's vesting schedule to target account.
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Bobo,
				H160::from_low_u64_be(1),
				PCall::vested_transfer { target: target.into(), index: 0 },
			)
			.execute_returns(());

		// Now lets check if vesting schedule has been transferred to target account
		let schedules = Vesting::<Runtime>::get(mapped_target_address).unwrap();
		assert!(!schedules.is_empty());
	});
}

#[test]
fn non_vested_cannot_vest_transfer() {
	ExtBuilder::default().build().execute_with(|| {
		let non_vested_account = TestAccount::Dave;
		assert_eq!(
			pallet_vesting::Pallet::<Runtime>::vesting(sp_core::sr25519::Public::from(
				non_vested_account.clone()
			)),
			None
		);

		let target = mock_pub_key(6);
		let error_msg = "No vesting schedule found for the sender";
		// non_vested_account should not be able to transfer vest schedule.
		PrecompilesValue::get()
			.prepare_test(
				non_vested_account,
				H160::from_low_u64_be(1),
				PCall::vested_transfer { target: target.into(), index: 0 },
			)
			.execute_reverts(|output| output == error_msg.as_bytes());
	});
}
