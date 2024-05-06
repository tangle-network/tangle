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
use frame_support::{assert_err, assert_noop, assert_ok};
use mock::*;
use serde_json::Value;
use sp_core::{bounded_vec, ecdsa, U256};
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

fn zero_key() -> ecdsa::Public {
	ecdsa::Public([0; 33])
}

fn cggmp21_blueprint() -> ServiceBlueprint {
	ServiceBlueprint {
		metadata: ServiceMetadata { name: "CGGMP21 TSS".try_into().unwrap(), ..Default::default() },
		jobs: bounded_vec![
			JobDefinition {
				metadata: JobMetadata { name: "keygen".try_into().unwrap(), ..Default::default() },
				params: bounded_vec![FieldType::Uint8],
				result: bounded_vec![FieldType::Bytes],
				verifier: JobResultVerifier::None,
			},
			JobDefinition {
				metadata: JobMetadata { name: "sign".try_into().unwrap(), ..Default::default() },
				params: bounded_vec![FieldType::Array(32, Box::new(FieldType::Uint8))],
				result: bounded_vec![FieldType::Array(64, Box::new(FieldType::Uint8))],
				verifier: JobResultVerifier::None,
			},
		],
		registration_hook: ServiceRegistrationHook::Evm(CGGMP21_REGISTRATION_HOOK),
		registration_params: bounded_vec![],
		request_hook: ServiceRequestHook::Evm(CGGMP21_REQUEST_HOOK),
		request_params: bounded_vec![],
		gadget: Default::default(),
	}
}

#[test]
fn create_service_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);

		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let next_id = Services::next_blueprint_id();
		assert_eq!(next_id, 1);
		assert_events(vec![RuntimeEvent::Services(crate::Event::BlueprintCreated {
			owner: alice,
			blueprint_id: next_id - 1,
		})]);
	});
}

#[test]
fn register_on_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);

		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);

		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			ServiceProviderPrefrences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));

		assert_events(vec![RuntimeEvent::Services(crate::Event::Registered {
			provider: bob.clone(),
			blueprint_id: 0,
			preferences: ServiceProviderPrefrences {
				key: zero_key(),
				approval: ApprovalPrefrence::default(),
			},
			registration_args: Default::default(),
		})]);

		// if we try to register again, it should fail.
		assert_err!(
			Services::register(
				RuntimeOrigin::signed(bob),
				0,
				ServiceProviderPrefrences {
					key: zero_key(),
					approval: ApprovalPrefrence::default()
				},
				Default::default(),
			),
			crate::Error::<Runtime>::AlreadyRegistered
		);
	});
}

#[test]
fn update_approval_preference() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);

		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);

		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			ServiceProviderPrefrences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));

		assert_eq!(
			ServiceProviders::<Runtime>::get(0, &bob).unwrap(),
			ServiceProviderPrefrences { key: zero_key(), approval: ApprovalPrefrence::default() },
		);

		assert_events(vec![RuntimeEvent::Services(crate::Event::Registered {
			provider: bob.clone(),
			blueprint_id: 0,
			preferences: ServiceProviderPrefrences {
				key: zero_key(),
				approval: ApprovalPrefrence::default(),
			},
			registration_args: Default::default(),
		})]);

		// update approval preference
		assert_ok!(Services::update_approval_preference(
			RuntimeOrigin::signed(bob.clone()),
			0,
			ApprovalPrefrence::Required,
		));

		assert_eq!(
			ServiceProviders::<Runtime>::get(0, &bob).unwrap().approval,
			ApprovalPrefrence::Required
		);

		assert_events(vec![RuntimeEvent::Services(crate::Event::ApprovalPreferenceUpdated {
			provider: bob,
			blueprint_id: 0,
			approval_preference: ApprovalPrefrence::Required,
		})]);

		// try to update approval preference when not registered
		let charlie = mock_pub_key(CHARLIE);
		assert_err!(
			Services::update_approval_preference(
				RuntimeOrigin::signed(charlie),
				0,
				ApprovalPrefrence::Required
			),
			crate::Error::<Runtime>::NotRegistered
		);
	});
}

#[test]
fn deregister_from_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			ServiceProviderPrefrences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));
		assert_ok!(Services::deregister(RuntimeOrigin::signed(bob.clone()), 0));
		assert_eq!(ServiceProviders::<Runtime>::contains_key(0, &bob), false);

		assert_events(vec![RuntimeEvent::Services(crate::Event::Deregistered {
			provider: bob,
			blueprint_id: 0,
		})]);

		// try to deregister when not registered
		let charlie = mock_pub_key(CHARLIE);
		assert_err!(
			Services::deregister(RuntimeOrigin::signed(charlie), 0),
			crate::Error::<Runtime>::NotRegistered
		);
	});
}

#[test]
fn request_service() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			ServiceProviderPrefrences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));
		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			ServiceProviderPrefrences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));
		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			ServiceProviderPrefrences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));

		let eve = mock_pub_key(EVE);
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			100,
			Default::default(),
		));
		// this service gets immediately accepted by all providers.
		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);
		assert_eq!(Instances::<Runtime>::contains_key(0), true);
		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceInitiated {
			owner: eve,
			request_id: None,
			service_id: 0,
			blueprint_id: 0,
		})]);
	});
}

#[test]
fn request_service_with_approval_process() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			ServiceProviderPrefrences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));

		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			ServiceProviderPrefrences { key: zero_key(), approval: ApprovalPrefrence::Required },
			Default::default(),
		));

		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			ServiceProviderPrefrences { key: zero_key(), approval: ApprovalPrefrence::Required },
			Default::default(),
		));

		let eve = mock_pub_key(EVE);
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			100,
			Default::default(),
		));

		// the service should be pending approval from charlie and dave.
		assert_eq!(ServiceRequests::<Runtime>::contains_key(0), true);
		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceRequested {
			owner: eve.clone(),
			request_id: 0,
			blueprint_id: 0,
			approved: vec![bob.clone()],
			pending_approvals: vec![charlie.clone(), dave.clone()],
		})]);

		// charlie approves the service
		assert_ok!(Services::approve(RuntimeOrigin::signed(charlie.clone()), 0));
		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceRequestApproved {
			provider: charlie.clone(),
			request_id: 0,
			blueprint_id: 0,
			approved: vec![charlie.clone(), bob.clone()],
			pending_approvals: vec![dave.clone()],
		})]);

		// dave approves the service, and the service is initiated.
		assert_ok!(Services::approve(RuntimeOrigin::signed(dave.clone()), 0));
		assert_eq!(ServiceRequests::<Runtime>::contains_key(0), false);
		assert_eq!(Instances::<Runtime>::contains_key(0), true);
		assert_events(vec![
			RuntimeEvent::Services(crate::Event::ServiceRequestApproved {
				provider: dave.clone(),
				request_id: 0,
				blueprint_id: 0,
				approved: vec![charlie.clone(), dave.clone(), bob.clone()],
				pending_approvals: vec![],
			}),
			RuntimeEvent::Services(crate::Event::ServiceInitiated {
				owner: eve,
				request_id: Some(0),
				service_id: 0,
				blueprint_id: 0,
			}),
		]);
	});
}
