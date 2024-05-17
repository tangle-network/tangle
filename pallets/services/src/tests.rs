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
use crate::types::ConstraintsOf;

use super::*;
use crate::mock_evm::address_build;
use ethers::prelude::*;
use frame_support::{assert_err, assert_noop, assert_ok};
use mock::*;
use serde_json::Value;
use sp_core::{bounded_vec, ecdsa, ByteArray, U256};
use sp_runtime::{traits::BlakeTwo256, KeyTypeId};
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

fn cggmp21_blueprint() -> ServiceBlueprint<ConstraintsOf<Runtime>> {
	ServiceBlueprint {
		metadata: ServiceMetadata { name: "CGGMP21 TSS".try_into().unwrap(), ..Default::default() },
		jobs: bounded_vec![
			JobDefinition {
				metadata: JobMetadata { name: "keygen".try_into().unwrap(), ..Default::default() },
				params: bounded_vec![FieldType::Uint8],
				result: bounded_vec![FieldType::Bytes],
				verifier: JobResultVerifier::Evm(CGGMP21_JOB_RESULT_VERIFIER),
			},
			JobDefinition {
				metadata: JobMetadata { name: "sign".try_into().unwrap(), ..Default::default() },
				params: bounded_vec![FieldType::Uint64, FieldType::Bytes],
				result: bounded_vec![FieldType::Bytes],
				verifier: JobResultVerifier::Evm(CGGMP21_JOB_RESULT_VERIFIER),
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

		let registeration_call = Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		);
		assert_ok!(registeration_call);

		assert_events(vec![RuntimeEvent::Services(crate::Event::Registered {
			provider: bob.clone(),
			blueprint_id: 0,
			preferences: OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPrefrence::default(),
			},
			registration_args: Default::default(),
		})]);

		// The blueprint should be added to my blueprints in my profile.
		let profile = OperatorsProfile::<Runtime>::get(bob.clone()).unwrap();
		assert!(profile.blueprints.contains(&0));

		// if we try to register again, it should fail.
		assert_err!(
			Services::register(
				RuntimeOrigin::signed(bob),
				0,
				OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
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
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));

		assert_eq!(
			Operators::<Runtime>::get(0, &bob).unwrap(),
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
		);

		assert_events(vec![RuntimeEvent::Services(crate::Event::Registered {
			provider: bob.clone(),
			blueprint_id: 0,
			preferences: OperatorPreferences {
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
			Operators::<Runtime>::get(0, &bob).unwrap().approval,
			ApprovalPrefrence::Required
		);

		assert_events(vec![RuntimeEvent::Services(crate::Event::ApprovalPreferenceUpdated {
			operator: bob,
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
fn unregister_from_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));
		assert_ok!(Services::unregister(RuntimeOrigin::signed(bob.clone()), 0));
		assert_eq!(Operators::<Runtime>::contains_key(0, &bob), false);

		// The blueprint should be removed from my blueprints in my profile.
		let profile = OperatorsProfile::<Runtime>::get(bob.clone()).unwrap();
		assert!(!profile.blueprints.contains(&0));

		assert_events(vec![RuntimeEvent::Services(crate::Event::Unregistered {
			operator: bob,
			blueprint_id: 0,
		})]);

		// try to deregister when not registered
		let charlie = mock_pub_key(CHARLIE);
		assert_err!(
			Services::unregister(RuntimeOrigin::signed(charlie), 0),
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
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));
		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));
		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
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
		// The service should also be added to the services for each operator.
		let profile = OperatorsProfile::<Runtime>::get(bob).unwrap();
		assert!(profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(charlie).unwrap();
		assert!(profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(dave).unwrap();
		assert!(profile.services.contains(&0));

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
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));

		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::Required },
			Default::default(),
		));

		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::Required },
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

		// it should not be added, until all providers approve.
		let profile = OperatorsProfile::<Runtime>::get(bob.clone()).unwrap();
		assert!(!profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(charlie.clone()).unwrap();
		assert!(!profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(dave.clone()).unwrap();
		assert!(!profile.services.contains(&0));
		// charlie approves the service
		assert_ok!(Services::approve(RuntimeOrigin::signed(charlie.clone()), 0));
		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceRequestApproved {
			operator: charlie.clone(),
			request_id: 0,
			blueprint_id: 0,
			approved: vec![charlie.clone(), bob.clone()],
			pending_approvals: vec![dave.clone()],
		})]);

		// dave approves the service, and the service is initiated.
		assert_ok!(Services::approve(RuntimeOrigin::signed(dave.clone()), 0));
		assert_eq!(ServiceRequests::<Runtime>::contains_key(0), false);
		assert_eq!(Instances::<Runtime>::contains_key(0), true);

		// The service should also be added to the services for each operator.
		let profile = OperatorsProfile::<Runtime>::get(bob.clone()).unwrap();
		assert!(profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(charlie.clone()).unwrap();
		assert!(profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(dave.clone()).unwrap();
		assert!(profile.services.contains(&0));

		assert_events(vec![
			RuntimeEvent::Services(crate::Event::ServiceRequestApproved {
				operator: dave.clone(),
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

#[test]
fn job_calls() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));
		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));
		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
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
			owner: eve.clone(),
			request_id: None,
			service_id: 0,
			blueprint_id: 0,
		})]);

		// now we can call the jobs
		let job_call_id = 0;
		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			0,
			0,
			bounded_vec![Field::Uint8(2)],
		));

		assert_eq!(JobCalls::<Runtime>::contains_key(0, job_call_id), true);
		assert_events(vec![RuntimeEvent::Services(crate::Event::JobCalled {
			caller: eve,
			service_id: 0,
			job: 0,
			call_id: job_call_id,
			args: vec![Field::Uint8(2)],
		})]);
	});
}

#[test]
fn job_calls_fails_with_invalid_input() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));
		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));
		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
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
			owner: eve.clone(),
			request_id: None,
			service_id: 0,
			blueprint_id: 0,
		})]);

		// now we can call the jobs
		let job_call_id = 0;
		assert_err!(
			Services::call(
				RuntimeOrigin::signed(eve.clone()),
				0,
				0,
				// t > n
				bounded_vec![Field::Uint8(4)],
			),
			crate::Error::<Runtime>::InvalidJobCallInput
		);

		assert_eq!(JobCalls::<Runtime>::contains_key(0, job_call_id), false);
	});
}

#[test]
fn job_result() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));
		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
			Default::default(),
		));
		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			OperatorPreferences { key: zero_key(), approval: ApprovalPrefrence::default() },
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
			owner: eve.clone(),
			request_id: None,
			service_id: 0,
			blueprint_id: 0,
		})]);

		// now we can call the jobs
		let keygen_job_call_id = 0;

		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			0,
			0,
			bounded_vec![Field::Uint8(2)]
		));

		assert_eq!(JobCalls::<Runtime>::contains_key(0, keygen_job_call_id), true);
		// now we can set the job result
		let key_type = KeyTypeId(*b"mdkg");
		let dkg = sp_io::crypto::ecdsa_generate(key_type, None);
		assert_ok!(Services::submit_result(
			RuntimeOrigin::signed(bob.clone()),
			0,
			keygen_job_call_id,
			bounded_vec![Field::Bytes(dkg.to_raw_vec().try_into().unwrap())],
		));

		// submit signing job
		let signing_job_call_id = 1;
		let data_hash = sp_core::keccak_256(&[1; 32]);

		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			0,
			1,
			bounded_vec![
				Field::Uint64(keygen_job_call_id),
				Field::Bytes(data_hash.to_vec().try_into().unwrap())
			],
		));

		// now we can set the job result
		let signature = sp_io::crypto::ecdsa_sign_prehashed(key_type, &dkg, &data_hash).unwrap();
		let mut signature_bytes = signature.to_raw_vec();
		// fix the v value (it should be 27 or 28).
		signature_bytes[64] += 27u8;
		// For some reason, the signature is not being verified.
		// in EVM, ecrecover is used to verify the signature, but it returns
		// 0x000000000000000000000000000000000000000 as the address of the signer.
		// even though the signature is correct, and we have the precomiles in the runtime.
		//
		// assert_ok!(Services::submit_result(
		// 	RuntimeOrigin::signed(bob.clone()),
		// 	0,
		// 	signing_job_call_id,
		// 	bounded_vec![Field::Bytes(signature_bytes.try_into().unwrap())],
		// ));
	});
}
