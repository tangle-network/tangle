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

#[test]
fn test_security_requirements_validation() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		let eve = mock_pub_key(EVE);
		// Register operator
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), Default::default(), 1000));
		// Test Case 1: Invalid min exposure (0%)
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(eve.clone()),
				None,
				0,
				vec![alice.clone()],
				vec![bob.clone()],
				Default::default(),
				vec![get_security_requirement(WETH, &[0, 20])],
				100,
				Asset::Custom(USDC),
				0,
				MembershipModel::Fixed { min_operators: 1 },
			),
			Error::<Runtime>::InvalidSecurityRequirements
		);
		// Test Case 2: Invalid max exposure (0%)
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(eve.clone()),
				None,
				0,
				vec![alice.clone()],
				vec![bob.clone()],
				Default::default(),
				vec![get_security_requirement(WETH, &[10, 0])],
				100,
				Asset::Custom(USDC),
				0,
				MembershipModel::Fixed { min_operators: 1 },
			),
			Error::<Runtime>::InvalidSecurityRequirements
		);
		// Test Case 3: Min exposure > Max exposure
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(eve.clone()),
				None,
				0,
				vec![alice.clone()],
				vec![bob.clone()],
				Default::default(),
				vec![get_security_requirement(WETH, &[30, 20])],
				100,
				Asset::Custom(USDC),
				0,
				MembershipModel::Fixed { min_operators: 1 },
			),
			Error::<Runtime>::InvalidSecurityRequirements
		);
		// Test Case 4: Max exposure > 100%
		// NOTE: this one passes because the max exposure is capped at 100% anyway
		// This enforcement is done in the [`Percent`] type
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(WETH, &[10, 101])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));
		// Test Case 5: Valid security requirements
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
	});
}

#[test]
fn test_security_commitment_validation() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		let eve = mock_pub_key(EVE);
		// Register operator
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), Default::default(), 1000));
		// Create service request
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(WETH, &[10, 20]),],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));
		// Test Case 1: Commitment below minimum exposure
		assert_err!(
			Services::approve(
				RuntimeOrigin::signed(bob.clone()),
				0,
				vec![get_security_commitment(WETH, 5)],
			),
			Error::<Runtime>::InvalidSecurityCommitments
		);
		// Test Case 2: Commitment above maximum exposure
		assert_err!(
			Services::approve(
				RuntimeOrigin::signed(bob.clone()),
				0,
				vec![get_security_commitment(WETH, 25)],
			),
			Error::<Runtime>::InvalidSecurityCommitments
		);
		// Test Case 3: Missing required asset commitment (native asset)
		assert_err!(
			Services::approve(
				RuntimeOrigin::signed(bob.clone()),
				0,
				vec![get_security_commitment(WETH, 15)],
			),
			Error::<Runtime>::InvalidSecurityCommitments
		);
		// Test Case 4: Wrong asset provided
		assert_err!(
			Services::approve(
				RuntimeOrigin::signed(bob.clone()),
				0,
				vec![get_security_commitment(USDC, 15)],
			),
			Error::<Runtime>::InvalidSecurityCommitments
		);
		// Test Case 4: Valid commitment
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			vec![get_security_commitment(WETH, 15), get_security_commitment(TNT, 15)],
		));
	});
}

#[test]
fn test_exposure_calculations() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let dave = mock_pub_key(DAVE);
		let eve = mock_pub_key(EVE);

		// Register operators
		for operator in [bob.clone(), charlie.clone(), dave.clone()] {
			assert_ok!(join_and_register(operator, 0, test_ecdsa_key(), Default::default(), 1000));
		}

		// Create service with multiple assets and exposure requirements
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			Default::default(),
			vec![
				get_security_requirement(WETH, &[10, 30]),
				get_security_requirement(USDC, &[15, 25]),
			],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 3 },
		));

		// Test different exposure combinations
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			vec![
				get_security_commitment(WETH, 20),
				get_security_commitment(USDC, 20),
				get_security_commitment(TNT, 20)
			],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			vec![
				get_security_commitment(WETH, 25),
				get_security_commitment(USDC, 15),
				get_security_commitment(TNT, 10),
			],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(dave.clone()),
			0,
			vec![
				get_security_commitment(WETH, 15),
				get_security_commitment(USDC, 20),
				get_security_commitment(TNT, 10),
			],
		));

		let service = Instances::<Runtime>::get(0).unwrap();
		let operator_security_commitments = service.operator_security_commitments;

		// Verify service is initiated with correct exposures
		assert!(Instances::<Runtime>::contains_key(0));
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
	});
}

#[test]
fn test_exposure_limits() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let dave = mock_pub_key(DAVE);
		let eve = mock_pub_key(EVE);

		// Register operators
		for operator in [bob.clone(), charlie.clone(), dave.clone()] {
			assert_ok!(join_and_register(operator, 0, test_ecdsa_key(), Default::default(), 1000));
		}

		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			Default::default(),
			vec![get_security_requirement(WETH, &[40, 60])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 3 },
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			vec![get_security_commitment(WETH, 50), get_security_commitment(TNT, 50)],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			vec![get_security_commitment(WETH, 50), get_security_commitment(TNT, 50)],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(dave.clone()),
			0,
			vec![get_security_commitment(WETH, 50), get_security_commitment(TNT, 50)],
		));

		// Create second service that shares the same security (overlapping exposures)
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone()],
			Default::default(),
			vec![get_security_requirement(WETH, &[40, 60])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 2 },
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			1,
			vec![get_security_commitment(WETH, 50), get_security_commitment(TNT, 50)],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			1,
			vec![get_security_commitment(WETH, 50), get_security_commitment(TNT, 50)],
		));

		// Create third service with different asset (USDC)
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone()],
			Default::default(),
			vec![get_security_requirement(USDC, &[40, 60])],
			100,
			Asset::Custom(WETH),
			0,
			MembershipModel::Fixed { min_operators: 2 },
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			2,
			vec![get_security_commitment(USDC, 50), get_security_commitment(TNT, 50)],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			2,
			vec![get_security_commitment(USDC, 50), get_security_commitment(TNT, 50)],
		));

		// Verify all services are active
		let service0 = Instances::<Runtime>::get(0).unwrap();
		let service1 = Instances::<Runtime>::get(1).unwrap();
		let service2 = Instances::<Runtime>::get(2).unwrap();

		// Verify events for service initiation and approvals
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
			operator_security_commitments: service0.operator_security_commitments.clone(),
		})));

		assert!(events.contains(&RuntimeEvent::Services(crate::Event::ServiceInitiated {
			owner: eve.clone(),
			request_id: 1,
			service_id: 1,
			blueprint_id: 0,
			operator_security_commitments: service1.operator_security_commitments.clone(),
		})));

		assert!(events.contains(&RuntimeEvent::Services(crate::Event::ServiceInitiated {
			owner: eve.clone(),
			request_id: 2,
			service_id: 2,
			blueprint_id: 0,
			operator_security_commitments: service2.operator_security_commitments.clone(),
		})));
	});
}
