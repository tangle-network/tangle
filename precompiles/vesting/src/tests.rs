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

use crate::mock::{roll_to, ExtBuilder, PCall, PrecompilesValue, Runtime, TestAccount};

use pallet_vesting::Vesting;
use precompile_utils::testing::*;
use sp_core::H160;

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
		let schedules =
			Vesting::<Runtime>::get(sp_core::sr25519::Public::from(TestAccount::Alex)).unwrap();
		assert!(!schedules.is_empty());
		roll_to(1000);
		PrecompilesValue::get()
			.prepare_test(TestAccount::Alex, H160::from_low_u64_be(1), PCall::vest {})
			.execute_returns(());
	});
}

// Test unlocking any vested funds of a `target` account.
#[test]
fn test_vest_other() {
	ExtBuilder::default().build().execute_with(|| {
		let schedules =
			Vesting::<Runtime>::get(sp_core::sr25519::Public::from(TestAccount::Alex)).unwrap();
		assert!(!schedules.is_empty());
		roll_to(1000);
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Bobo,
				H160::from_low_u64_be(1),
				PCall::vest_other {
					target: sp_core::sr25519::Public::from(TestAccount::Alex).into(),
				},
			)
			.execute_returns(());
	});
}

// Test vested transfer.
#[test]
fn test_vest_transfer() {
	ExtBuilder::default().build().execute_with(|| {
		let schedules =
			Vesting::<Runtime>::get(sp_core::sr25519::Public::from(TestAccount::Alex)).unwrap();
		assert!(!schedules.is_empty());
		let target = TestAccount::Bobo;
		roll_to(1000);
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::vested_transfer {
					target: sp_core::sr25519::Public::from(target).into(),
					index: 0,
				},
			)
			.execute_returns(());
	});
}
