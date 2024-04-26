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
#![cfg(test)]
use super::*;
use crate::mock_evm::{address_build, EIP1559UnsignedTransaction};
use ethers::prelude::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use serde_json::Value;
use sp_core::{bounded_vec, U256};
use sp_runtime::traits::BlakeTwo256;
use sp_std::sync::Arc;
use std::fs;
use tangle_primitives::jobs::v2::*;

const ALICE: u8 = 1;
const BOB: u8 = 2;
const CHARLIE: u8 = 3;
const DAVE: u8 = 4;
const EVE: u8 = 5;

const TEN: u8 = 10;
const TWENTY: u8 = 20;
const HUNDRED: u8 = 100;

#[test]
fn create_service_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);

		let blueprint = ServiceBlueprint {
			metadata: ServiceMetadata {
				name: "CGGMP21 TSS".try_into().unwrap(),
				..Default::default()
			},
			jobs: bounded_vec![
				JobDefinition {
					metadata: JobMetadata {
						name: "keygen".try_into().unwrap(),
						..Default::default()
					},
					params: bounded_vec![FieldType::Uint8],
					result: bounded_vec![FieldType::Array(33, Box::new(FieldType::Uint8))],
					verifier: JobResultVerifier::None,
				},
				JobDefinition {
					metadata: JobMetadata {
						name: "sign".try_into().unwrap(),
						..Default::default()
					},
					params: bounded_vec![FieldType::Array(32, Box::new(FieldType::Uint8))],
					result: bounded_vec![FieldType::Array(64, Box::new(FieldType::Uint8))],
					verifier: JobResultVerifier::None,
				},
			],
			registration_hook: ServiceRegistrationHook::None,
			registration_params: bounded_vec![],
			request_hook: ServiceRequestHook::None,
			request_params: bounded_vec![],
			gadget: Default::default(),
		};

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let next_id = Services::next_blueprint_id();
		assert_eq!(next_id, 1);
		assert_events(vec![RuntimeEvent::Services(crate::Event::BlueprintCreated {
			owner: alice,
			blueprint_id: next_id - 1,
		})]);
	});
}
