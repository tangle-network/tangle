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
use frame_support::{assert_err, assert_ok};
use sp_core::{ByteArray, offchain::KeyTypeId};

#[test]
fn job_calls() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Register multiple operators
		let bob = mock_pub_key(BOB);
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(join_and_register(
			charlie.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		let dave = mock_pub_key(DAVE);
		assert_ok!(join_and_register(
			dave.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		let eve = mock_pub_key(EVE);
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			Default::default(),
			vec![get_security_requirement(WETH, &[10, 20])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 3 },
		));

		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// All operators approve with security commitments
		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 0, vec![
			get_security_commitment(WETH, 10),
			get_security_commitment(TNT, 10)
		],));

		assert_ok!(Services::approve(RuntimeOrigin::signed(charlie.clone()), 0, vec![
			get_security_commitment(WETH, 10),
			get_security_commitment(TNT, 10)
		],));

		assert_ok!(Services::approve(RuntimeOrigin::signed(dave.clone()), 0, vec![
			get_security_commitment(WETH, 10),
			get_security_commitment(TNT, 10)
		],));

		let service = Instances::<Runtime>::get(0).unwrap();
		let operator_security_commitments = service.operator_security_commitments;

		let events = System::events()
			.into_iter()
			.map(|e| e.event)
			.filter(|e| matches!(e, RuntimeEvent::Services(_)))
			.collect::<Vec<_>>();

		assert!(events.contains(&RuntimeEvent::Services(crate::Event::ServiceInitiated {
			owner: eve.clone(),
			request_id: 0,
			service_id: 0,
			blueprint_id: 0,
			operator_security_commitments,
		})));

		// now we can call the jobs
		let job_call_id = 0;
		assert_ok!(Services::call(RuntimeOrigin::signed(eve.clone()), 0, 0, bounded_vec![
			Field::Uint8(2)
		],));

		assert!(JobCalls::<Runtime>::contains_key(0, job_call_id));
		let events = System::events()
			.into_iter()
			.map(|e| e.event)
			.filter(|e| matches!(e, RuntimeEvent::Services(_)))
			.collect::<Vec<_>>();

		assert!(events.contains(&RuntimeEvent::Services(crate::Event::JobCalled {
			caller: eve,
			service_id: 0,
			job: 0,
			call_id: job_call_id,
			args: vec![Field::Uint8(2)],
		})));
	});
}

#[test]
fn job_result() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Register multiple operators
		let bob = mock_pub_key(BOB);
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(join_and_register(
			charlie.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		let dave = mock_pub_key(DAVE);
		assert_ok!(join_and_register(
			dave.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		let eve = mock_pub_key(EVE);
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			Default::default(),
			vec![get_security_requirement(WETH, &[10, 20])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 3 },
		));

		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// All operators approve with security commitments
		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 0, vec![
			get_security_commitment(WETH, 10),
			get_security_commitment(TNT, 10)
		],));

		assert_ok!(Services::approve(RuntimeOrigin::signed(charlie.clone()), 0, vec![
			get_security_commitment(WETH, 10),
			get_security_commitment(TNT, 10)
		],));

		assert_ok!(Services::approve(RuntimeOrigin::signed(dave.clone()), 0, vec![
			get_security_commitment(WETH, 10),
			get_security_commitment(TNT, 10)
		],));

		let service = Instances::<Runtime>::get(0).unwrap();
		let operator_security_commitments = service.operator_security_commitments;

		let events = System::events()
			.into_iter()
			.map(|e| e.event)
			.filter(|e| matches!(e, RuntimeEvent::Services(_)))
			.collect::<Vec<_>>();

		assert!(events.contains(&RuntimeEvent::Services(crate::Event::ServiceInitiated {
			owner: eve.clone(),
			request_id: 0,
			service_id: 0,
			blueprint_id: 0,
			operator_security_commitments,
		})));

		// now we can call the jobs
		let keygen_job_call_id = 0;

		assert_ok!(Services::call(RuntimeOrigin::signed(eve.clone()), 0, 0, bounded_vec![
			Field::Uint8(2)
		]));

		assert!(JobCalls::<Runtime>::contains_key(0, keygen_job_call_id));

		// now we can set the job result
		let key_type = KeyTypeId(*b"mdkg");
		let dkg = sp_io::crypto::ecdsa_generate(key_type, None);
		assert_ok!(Services::submit_result(
			RuntimeOrigin::signed(bob.clone()),
			0,
			keygen_job_call_id,
			bounded_vec![Field::from(BoundedVec::try_from(dkg.to_raw_vec()).unwrap())],
		));

		// submit signing job

		let data_hash = sp_core::keccak_256(&[1; 32]);

		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			0,
			SIGN_JOB_ID,
			bounded_vec![
				Field::Uint64(keygen_job_call_id),
				Field::from(BoundedVec::try_from(data_hash.to_vec()).unwrap())
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
		// let signing_job_call_id = 1;
		// assert_ok!(Services::submit_result(
		//     RuntimeOrigin::signed(bob.clone()),
		//     0,
		//     signing_job_call_id,
		//     bounded_vec![Field::Bytes(signature_bytes.try_into().unwrap())],
		// ));
	});
}

#[test]
fn test_concurrent_job_execution() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Register operators
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let dave = mock_pub_key(DAVE);
		let eve = mock_pub_key(EVE);

		for operator in [bob.clone(), charlie.clone(), dave.clone()] {
			assert_ok!(join_and_register(
				operator.clone(),
				0,
				test_ecdsa_key(),
				1000,
				Some("https://example.com/rpc")
			));
		}

		// Create and approve service
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			Default::default(),
			vec![get_security_requirement(WETH, &[10, 20])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 3 },
		));

		for operator in [bob.clone(), charlie.clone(), dave.clone()] {
			assert_ok!(Services::approve(RuntimeOrigin::signed(operator), 0, vec![
				get_security_commitment(WETH, 10),
				get_security_commitment(TNT, 10)
			],));
		}

		// Submit multiple concurrent job calls
		assert_ok!(Services::call(RuntimeOrigin::signed(eve.clone()), 0, 0, bounded_vec![
			Field::Uint8(1)
		],));

		assert_ok!(Services::call(RuntimeOrigin::signed(eve.clone()), 0, 0, bounded_vec![
			Field::Uint8(2)
		],));

		// Verify both jobs are tracked
		assert!(JobCalls::<Runtime>::contains_key(0, 0));
		assert!(JobCalls::<Runtime>::contains_key(0, 1));

		// Submit results for both jobs
		let key_type = KeyTypeId(*b"mdkg");
		let dkg = sp_io::crypto::ecdsa_generate(key_type, None);

		assert_ok!(Services::submit_result(
			RuntimeOrigin::signed(bob.clone()),
			0,
			0,
			bounded_vec![Field::from(BoundedVec::try_from(dkg.to_raw_vec()).unwrap())],
		));

		assert_ok!(Services::submit_result(
			RuntimeOrigin::signed(bob.clone()),
			0,
			1,
			bounded_vec![Field::from(BoundedVec::try_from(dkg.to_raw_vec()).unwrap())],
		));
	});
}

#[test]
fn test_result_submission_non_operators() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Register operators
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let dave = mock_pub_key(DAVE);
		let eve = mock_pub_key(EVE);

		for operator in [bob.clone(), charlie.clone()] {
			assert_ok!(join_and_register(
				operator.clone(),
				0,
				test_ecdsa_key(),
				1000,
				Some("https://example.com/rpc")
			));
		}

		// Create and approve service
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone()],
			Default::default(),
			vec![get_security_requirement(WETH, &[10, 20])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 2 },
		));

		for operator in [bob.clone(), charlie.clone()] {
			assert_ok!(Services::approve(RuntimeOrigin::signed(operator), 0, vec![
				get_security_commitment(WETH, 10),
				get_security_commitment(TNT, 10)
			],));
		}

		// Submit job call
		assert_ok!(Services::call(RuntimeOrigin::signed(eve.clone()), 0, 0, bounded_vec![
			Field::Uint8(1)
		],));

		// Non-operator tries to submit result
		let key_type = KeyTypeId(*b"mdkg");
		let dkg = sp_io::crypto::ecdsa_generate(key_type, None);

		assert_err!(
			Services::submit_result(RuntimeOrigin::signed(dave.clone()), 0, 0, bounded_vec![
				Field::from(BoundedVec::try_from(dkg.to_raw_vec()).unwrap())
			],),
			Error::<Runtime>::NotRegistered
		);
	});
}

#[test]
fn test_invalid_result_formats() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Register operators
		let bob = mock_pub_key(BOB);
		let eve = mock_pub_key(EVE);

		assert_ok!(join_and_register(
			bob.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		// Create and approve service
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(WETH, &[10, 20])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 0, vec![
			get_security_commitment(WETH, 10),
			get_security_commitment(TNT, 10)
		],));

		// Submit job call
		assert_ok!(Services::call(RuntimeOrigin::signed(eve.clone()), 0, 0, bounded_vec![
			Field::Uint8(1)
		],));

		// Try to submit result with wrong field type
		assert_err!(
			Services::submit_result(RuntimeOrigin::signed(bob.clone()), 0, 0, bounded_vec![
				Field::String("invalid".try_into().unwrap())
			],),
                        Error::<Runtime>::TypeCheck(TypeCheckError::ResultTypeMismatch {
                                index: 0,
                                expected: FieldType::List(Box::new(FieldType::String)),
                                actual: FieldType::String,
                        }),
		);
	});
}

#[test]
fn test_result_submission_after_termination() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Register operators
		let bob = mock_pub_key(BOB);
		let eve = mock_pub_key(EVE);

		assert_ok!(join_and_register(
			bob.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		// Create and approve service
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(WETH, &[10, 20])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 0, vec![
			get_security_commitment(WETH, 10),
			get_security_commitment(TNT, 10)
		],));

		// Submit job call
		assert_ok!(Services::call(RuntimeOrigin::signed(eve.clone()), 0, 0, bounded_vec![
			Field::Uint8(1)
		],));

		// Terminate service
		assert_ok!(Services::terminate(RuntimeOrigin::signed(eve.clone()), 0));

		// Try to submit result after termination
		let key_type = KeyTypeId(*b"mdkg");
		let dkg = sp_io::crypto::ecdsa_generate(key_type, None);

		assert_err!(
			Services::submit_result(RuntimeOrigin::signed(bob.clone()), 0, 0, bounded_vec![
				Field::from(BoundedVec::try_from(dkg.to_raw_vec()).unwrap())
			],),
			Error::<Runtime>::ServiceNotFound
		);
	});
}
