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
	AccountId, ExtBuilder, PCall, PrecompilesValue, Runtime, RuntimeCall, RuntimeEvent,
	RuntimeOrigin, TestAccount, System, Balance, roll_to,
};
use frame_support::{assert_ok, traits::OnFinalize};
use pallet_evm::{Call as EvmCall, AddressMapping};
use pallet_vesting::{
	Call as VestingCall, Event as VestingEvent, Pallet as VestingPallet, Vesting, VestingInfo, MaxVestingSchedulesGet,
};
use precompile_utils::{
	assert_event_emitted, assert_event_not_emitted, precompile_set::AddressU64, prelude::*,
	testing::*,
};
use sp_core::{Get, H160, H256, U256};
use sp_runtime::traits::Dispatchable;
use std::{cell::Cell, rc::Rc, str::from_utf8};

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

#[test]
fn test_claim_vesting_schedule() {
	ExtBuilder::default().build().execute_with(|| {
		let schedules = Vesting::<Runtime>::get(sp_core::sr25519::Public::from(TestAccount::Alex)).unwrap();
		assert!(!schedules.is_empty());
		roll_to(1000);
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(6),
				PCall::vest {},
			)
			.execute_returns(());
	});
}

