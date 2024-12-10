// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// This file is part of pallet-evm-precompile-preimage package, originally developed by Purestake
// Inc. Pallet-evm-precompile-preimage package used in Tangle Network in terms of GPLv3.

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
use crate::{mock::*, *};
use precompile_utils::testing::*;

use frame_support::assert_ok;
use pallet_evm::{Call as EvmCall, Event as EvmEvent};
use sp_runtime::traits::Dispatchable;

use sp_core::{Hasher, U256};

fn evm_call(input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: Alice.into(),
		target: Precompile1.into(),
		input,
		value: U256::zero(),
		gas_limit: u64::MAX,
		max_fee_per_gas: 0.into(),
		max_priority_fee_per_gas: Some(U256::zero()),
		nonce: None,
		access_list: Vec::new(),
	}
}

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	check_precompile_implements_solidity_interfaces(&["Preimage.sol"], PCall::supports_selector)
}

#[test]
fn note_unnote_preimage_logs_work() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			let bytes = vec![1, 2, 3];
			let expected_hash = sp_runtime::traits::BlakeTwo256::hash(&bytes);

			// Note preimage
			let input = PCall::note_preimage { encoded_proposal: bytes.into() }.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Assert note preimage event is emited and matching frame event preimage hash.
			assert!([
				EvmEvent::Log {
					log: log1(
						Precompile1,
						SELECTOR_LOG_PREIMAGE_NOTED,
						solidity::encode_event_data(expected_hash)
					),
				}
				.into(),
				RuntimeEvent::Preimage(pallet_preimage::pallet::Event::Noted {
					hash: expected_hash
				})
			]
			.iter()
			.all(|log| events().contains(log)));

			// Unnote preimage
			let input = PCall::unnote_preimage { hash: expected_hash }.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Assert unnote preimage is emited
			assert!(events().contains(
				&EvmEvent::Log {
					log: log1(
						Precompile1,
						SELECTOR_LOG_PREIMAGE_UNNOTED,
						solidity::encode_event_data(expected_hash)
					),
				}
				.into()
			));
		})
}

#[test]
fn note_preimage_returns_preimage_hash() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 40)])
		.build()
		.execute_with(|| {
			let preimage = [1u8; 32];
			let preimage_hash = <mock::Runtime as frame_system::Config>::Hashing::hash(&preimage);

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::note_preimage { encoded_proposal: BoundedBytes::from(preimage) },
				)
				.execute_returns(preimage_hash);
		})
}
