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
use sp_runtime::Percent;

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
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

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
			Error::<Runtime>::InvalidSecurityRequirement
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
			Error::<Runtime>::InvalidSecurityRequirement
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
			Error::<Runtime>::InvalidSecurityRequirement
		);

		// Test Case 4: Max exposure > 100%
		assert_err!(
			Services::request(
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
			),
			Error::<Runtime>::InvalidSecurityRequirement
		);

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
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		// Create service request
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

		// Test Case 1: Commitment below minimum exposure
		assert_err!(
			Services::approve(
				RuntimeOrigin::signed(bob.clone()),
				0,
				Percent::from_percent(10),
				vec![get_security_commitment(WETH, 5)],
			),
			Error::<Runtime>::InvalidSecurityCommitment
		);

		// Test Case 2: Commitment above maximum exposure
		assert_err!(
			Services::approve(
				RuntimeOrigin::signed(bob.clone()),
				0,
				Percent::from_percent(10),
				vec![get_security_commitment(WETH, 25)],
			),
			Error::<Runtime>::InvalidSecurityCommitment
		);

		// Test Case 3: Missing required asset commitment
		assert_err!(
			Services::approve(
				RuntimeOrigin::signed(bob.clone()),
				0,
				Percent::from_percent(10),
				vec![get_security_commitment(USDC, 15)],
			),
			Error::<Runtime>::MissingSecurityCommitment
		);

		// Test Case 4: Valid commitment
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 15)],
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
			assert_ok!(Services::register(
				RuntimeOrigin::signed(operator.clone()),
				0,
				OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
				Default::default(),
				0,
			));
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
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 20), get_security_commitment(USDC, 20),],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 25), get_security_commitment(USDC, 15),],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(dave.clone()),
			0,
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 15), get_security_commitment(USDC, 20),],
		));

		// Verify service is initiated with correct exposures
		assert!(Instances::<Runtime>::contains_key(0));
		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceInitiated {
			owner: eve.clone(),
			request_id: 0,
			service_id: 0,
			blueprint_id: 0,
			assets: vec![Asset::Custom(WETH), Asset::Custom(USDC)],
		})]);
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
			assert_ok!(Services::register(
				RuntimeOrigin::signed(operator.clone()),
				0,
				OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
				Default::default(),
				0,
			));
		}

		// Create first service with high exposure for WETH
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

		// All operators commit high exposure for WETH in first service
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 50)],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 50)],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(dave.clone()),
			0,
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 50)],
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

		// Operators can commit the same assets again since security can be shared
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			1,
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 50)],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			1,
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 50)],
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

		// Operators can commit to different assets independently
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			2,
			Percent::from_percent(10),
			vec![get_security_commitment(USDC, 50)],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			2,
			Percent::from_percent(10),
			vec![get_security_commitment(USDC, 50)],
		));

		// Verify all services are active
		assert!(Instances::<Runtime>::contains_key(0));
		assert!(Instances::<Runtime>::contains_key(1));
		assert!(Instances::<Runtime>::contains_key(2));

		// Verify events for service initiation
		assert_events(vec![
			RuntimeEvent::Services(crate::Event::ServiceInitiated {
				owner: eve.clone(),
				request_id: 0,
				service_id: 0,
				blueprint_id: 0,
				assets: vec![Asset::Custom(WETH)],
			}),
			RuntimeEvent::Services(crate::Event::ServiceInitiated {
				owner: eve.clone(),
				request_id: 1,
				service_id: 1,
				blueprint_id: 0,
				assets: vec![Asset::Custom(WETH)],
			}),
			RuntimeEvent::Services(crate::Event::ServiceInitiated {
				owner: eve.clone(),
				request_id: 2,
				service_id: 2,
				blueprint_id: 0,
				assets: vec![Asset::Custom(USDC)],
			}),
		]);
	});
}
