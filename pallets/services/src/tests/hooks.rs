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
use frame_support::assert_ok;
use tangle_primitives::services::BoundedString;

#[test]
fn hooks() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let blueprint = ServiceBlueprint {
			metadata: ServiceMetadata {
				name: "Hooks Tests".try_into().unwrap(),
				..Default::default()
			},
			manager: BlueprintServiceManager::Evm(HOOKS_TEST),
			master_manager_revision: MasterBlueprintServiceManagerRevision::Latest,
			jobs: bounded_vec![JobDefinition {
				metadata: JobMetadata { name: "foo".try_into().unwrap(), ..Default::default() },
				params: bounded_vec![],
				result: bounded_vec![],
			},],
			registration_params: bounded_vec![],
			request_params: bounded_vec![],
			sources: Default::default(),
			supported_membership_models: bounded_vec![
				MembershipModelType::Fixed,
				MembershipModelType::Dynamic,
			],
		};

		// OnBlueprintCreated hook should be called
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnBlueprintCreated()")]);

		// OnRegister hook should be called
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnRegister()")]);

		// OnUnregister hook should be called
		assert_ok!(Services::unregister(RuntimeOrigin::signed(bob.clone()), 0));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnUnregister()")]);

		// Register again to continue testing
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: test_ecdsa_key(),
				rpc_address: BoundedString::try_from("https://example.com/rpc".to_string())
					.unwrap()
			},
			Default::default(),
			0,
		));

		// OnRequest hook should be called
		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![
				get_security_requirement(USDC, &[10, 20]),
				get_security_requirement(WETH, &[10, 20])
			],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnRequest()")]);

		// OnReject hook should be called
		assert_ok!(Services::reject(RuntimeOrigin::signed(bob.clone()), 0));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnReject()")]);

		// Create another request to test remaining hooks
		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![
				get_security_requirement(USDC, &[10, 20]),
				get_security_requirement(WETH, &[10, 20])
			],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// OnApprove hook should be called
		// OnServiceInitialized is also called
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			1,
			vec![
				get_security_commitment(USDC, 10),
				get_security_commitment(WETH, 10),
				get_security_commitment(TNT, 10)
			],
		));
		assert_evm_logs(&[
			evm_log!(HOOKS_TEST, b"OnApprove()"),
			evm_log!(HOOKS_TEST, b"OnServiceInitialized()"),
		]);

		// OnJobCall hook should be called
		assert_ok!(Services::call(RuntimeOrigin::signed(charlie.clone()), 0, 0, bounded_vec![],));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnJobCall()")]);

		// OnJobResult hook should be called
		assert_ok!(Services::submit_result(
			RuntimeOrigin::signed(bob.clone()),
			0,
			0,
			bounded_vec![],
		));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnJobResult()")]);

		// OnServiceTermination hook should be called
		assert_ok!(Services::terminate(RuntimeOrigin::signed(charlie.clone()), 0));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnServiceTermination()")]);
	});
}
