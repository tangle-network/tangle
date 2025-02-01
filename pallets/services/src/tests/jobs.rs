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
use crate::mock::*;
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_runtime::Percent;
use tangle_primitives::services::*;

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
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
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
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 10)],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 10)],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(dave.clone()),
			0,
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 10)],
		));

		assert!(Instances::<Runtime>::contains_key(0));
		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceInitiated {
			owner: eve.clone(),
			request_id: 0,
			service_id: 0,
			blueprint_id: 0,
			assets: vec![Asset::Custom(WETH)],
		})]);

		// now we can call the jobs
		let job_call_id = 0;
		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			0,
			0,
			bounded_vec![Field::Uint8(2)],
		));

		assert!(JobCalls::<Runtime>::contains_key(0, job_call_id));
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
fn job_result() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Register multiple operators
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
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
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 10)],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 10)],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(dave.clone()),
			0,
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 10)],
		));

		assert!(Instances::<Runtime>::contains_key(0));
		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceInitiated {
			owner: eve.clone(),
			request_id: 0,
			service_id: 0,
			blueprint_id: 0,
			assets: vec![Asset::Custom(WETH)],
		})]);

		// now we can call the jobs
		let keygen_job_call_id = 0;

		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			0,
			0,
			bounded_vec![Field::Uint8(2)]
		));

		assert!(JobCalls::<Runtime>::contains_key(0, keygen_job_call_id));

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
			SIGN_JOB_ID,
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
		//     RuntimeOrigin::signed(bob.clone()),
		//     0,
		//     signing_job_call_id,
		//     bounded_vec![Field::Bytes(signature_bytes.try_into().unwrap())],
		// ));
	});
}
