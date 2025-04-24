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
fn register_on_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);
		let bob_ecdsa_key = test_ecdsa_key();

		assert_ok!(join_and_register(
			bob.clone(),
			0,
			bob_ecdsa_key,
			price_targets(MachineKind::Large),
			1000,
		));

		let events = System::events()
			.into_iter()
			.map(|e| e.event)
			.filter(|e| matches!(e, RuntimeEvent::Services(_)))
			.collect::<Vec<_>>();

		assert!(events.contains(&RuntimeEvent::Services(crate::Event::Registered {
			provider: bob.clone(),
			blueprint_id: 0,
			preferences: OperatorPreferences {
				key: bob_ecdsa_key,
				price_targets: price_targets(MachineKind::Large),
			},
			registration_args: Default::default(),
		})));

		// The blueprint should be added to my blueprints in my profile.
		let profile = OperatorsProfile::<Runtime>::get(bob.clone()).unwrap();
		assert!(profile.blueprints.contains(&0));

		// if we try to register again, it should fail.
		assert_err!(
			Services::register(
				RuntimeOrigin::signed(bob),
				0,
				OperatorPreferences { key: bob_ecdsa_key, price_targets: Default::default() },
				Default::default(),
				0,
			),
			crate::Error::<Runtime>::AlreadyRegistered
		);

		// if we try to register with a non active operator, should fail
		assert_err!(
			Services::register(
				RuntimeOrigin::signed(mock_pub_key(100)),
				0,
				OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
				Default::default(),
				0,
			),
			crate::Error::<Runtime>::OperatorNotActive
		);
	});
}

#[test]
fn pre_register_on_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);
		let pre_registration_call = Services::pre_register(RuntimeOrigin::signed(bob.clone()), 0);
		assert_ok!(pre_registration_call);

		let events = System::events()
			.into_iter()
			.map(|e| e.event)
			.filter(|e| matches!(e, RuntimeEvent::Services(_)))
			.collect::<Vec<_>>();

		assert!(events.contains(&RuntimeEvent::Services(crate::Event::PreRegistration {
			operator: bob.clone(),
			blueprint_id: 0,
		})));
	});
}

#[test]
fn update_price_targets() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);
		let bob_operator_ecdsa_key = test_ecdsa_key();

		// Join operators and register
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			bob_operator_ecdsa_key,
			price_targets(MachineKind::Small),
			1000,
		));

		assert_eq!(Operators::<Runtime>::get(0, &bob).unwrap(), OperatorPreferences {
			key: bob_operator_ecdsa_key,
			price_targets: price_targets(MachineKind::Small)
		});

		System::reset_events(); // Clear all previous events

		// update price targets
		assert_ok!(Services::update_price_targets(
			RuntimeOrigin::signed(bob.clone()),
			0,
			price_targets(MachineKind::Medium),
		));

		assert_eq!(
			Operators::<Runtime>::get(0, &bob).unwrap().price_targets,
			price_targets(MachineKind::Medium)
		);

		assert_events(vec![RuntimeEvent::Services(crate::Event::PriceTargetsUpdated {
			operator: bob,
			blueprint_id: 0,
			price_targets: price_targets(MachineKind::Medium),
		})]);

		// try to update price targets when not registered
		let charlie = mock_pub_key(CHARLIE);
		assert_err!(
			Services::update_price_targets(
				RuntimeOrigin::signed(charlie),
				0,
				price_targets(MachineKind::Medium)
			),
			crate::Error::<Runtime>::NotRegistered
		);
	});
}

#[test]
fn unregister_from_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), Default::default(), 1000,));
		assert_ok!(Services::unregister(RuntimeOrigin::signed(bob.clone()), 0));
		assert!(!Operators::<Runtime>::contains_key(0, &bob));

		// The blueprint should be removed from my blueprints in my profile.
		let profile = OperatorsProfile::<Runtime>::get(bob.clone()).unwrap();
		assert!(!profile.blueprints.contains(&0));

		let events = System::events()
			.into_iter()
			.map(|e| e.event)
			.filter(|e| matches!(e, RuntimeEvent::Services(_)))
			.collect::<Vec<_>>();

		assert!(events.contains(&RuntimeEvent::Services(crate::Event::Unregistered {
			operator: bob,
			blueprint_id: 0,
		})));

		// try to deregister when not registered
		let charlie = mock_pub_key(CHARLIE);
		assert_err!(
			Services::unregister(RuntimeOrigin::signed(charlie), 0),
			crate::Error::<Runtime>::NotRegistered
		);
	});
}

#[test]
fn test_registration_max_blueprints() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let bob_ecdsa_key = test_ecdsa_key();

		// Join as operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(bob.clone()), 1000,));

		// Create maximum number of blueprints
		for i in 0..MaxBlueprintsPerOperator::get() {
			let blueprint = cggmp21_blueprint();
			assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

			// Register for each blueprint
			assert_ok!(Services::register(
				RuntimeOrigin::signed(bob.clone()),
				i.into(),
				OperatorPreferences {
					key: bob_ecdsa_key,
					price_targets: price_targets(MachineKind::Large),
				},
				Default::default(),
				0,
			));
		}

		// Create one more blueprint
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Try to register for one more blueprint - should fail
		assert_err!(
			Services::register(
				RuntimeOrigin::signed(bob.clone()),
				MaxBlueprintsPerOperator::get().into(),
				OperatorPreferences {
					key: bob_ecdsa_key,
					price_targets: price_targets(MachineKind::Large),
				},
				Default::default(),
				0,
			),
			Error::<Runtime>::MaxBlueprintsPerOperatorExceeded
		);
	});
}

#[test]
fn test_registration_invalid_preferences() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);

		// Test with invalid ECDSA key (zero key)
		let invalid_key = [0u8; 65];
		assert_err!(
			Services::register(
				RuntimeOrigin::signed(bob.clone()),
				0,
				OperatorPreferences {
					key: invalid_key,
					price_targets: price_targets(MachineKind::Large),
				},
				Default::default(),
				0,
			),
			Error::<Runtime>::InvalidKey
		);

		// TODO: Decide how we want to validate price targets
		// // Test with invalid price targets (all zeros)
		// assert_err!(
		// 	Services::register(
		// 		RuntimeOrigin::signed(bob.clone()),
		// 		0,
		// 		OperatorPreferences {
		// 			key: test_ecdsa_key(),
		// 			price_targets: PriceTargets {
		// 				cpu: 0,
		// 				mem: 0,
		// 				storage_hdd: 0,
		// 				storage_ssd: 0,
		// 				storage_nvme: 0,
		// 			},
		// 		},
		// 		Default::default(),
		// 		0,
		// 	),
		// 	Error::<Runtime>::InvalidPriceTargets
		// );
	});
}

#[test]
fn test_registration_duplicate_keys() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let ecdsa_key = test_ecdsa_key();

		// First registration should succeed
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			ecdsa_key,
			price_targets(MachineKind::Large),
			1000,
		));

		// Join operators first for Charlie
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(charlie.clone()),
			1000
		));

		// Second registration with same key should fail with DuplicateKey error
		assert_err!(
			Services::register(
				RuntimeOrigin::signed(charlie.clone()),
				0,
				OperatorPreferences {
					key: ecdsa_key,
					price_targets: price_targets(MachineKind::Large),
				},
				Default::default(),
				0,
			),
			Error::<Runtime>::DuplicateKey
		);
	});
}

#[test]
fn test_registration_during_active_services() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let eve = mock_pub_key(EVE);

		// Join operators with stake for Bob
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(bob.clone()), 1000));

		// Register Bob as an operator
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: test_ecdsa_key(),
				price_targets: price_targets(MachineKind::Large),
			},
			Default::default(),
			0,
		));

		// Create a service request
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

		// Verify service request exists but service instance doesn't yet
		assert!(ServiceRequests::<Runtime>::contains_key(0));
		assert!(!Instances::<Runtime>::contains_key(0));
		assert!(!UserServices::<Runtime>::get(eve.clone()).contains(&0));

		// Approve the service request
		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 0, vec![
			get_security_commitment(WETH, 10),
			get_security_commitment(TNT, 10)
		],));

		// Verify service is active and in instances storage
		// Check service instance exists
		assert!(Instances::<Runtime>::contains_key(0));
		// Verify service details
		let service = Instances::<Runtime>::get(0).unwrap();
		assert_eq!(service.owner, eve);
		let service_operators = service
			.operator_security_commitments
			.iter()
			.map(|r| r.0.clone())
			.collect::<Vec<_>>();
		assert_eq!(service_operators.len(), 1);
		assert!(service_operators.contains(&bob));
		// Verify user services mapping
		assert!(UserServices::<Runtime>::get(eve).contains(&0));

		// Try to unregister while service is active - should fail
		assert_err!(
			Services::unregister(RuntimeOrigin::signed(bob.clone()), 0),
			Error::<Runtime>::NotAllowedToUnregister
		);

		// Try to register another operator for the same blueprint
		assert_ok!(join_and_register(
			charlie.clone(),
			0,
			test_ecdsa_key(),
			price_targets(MachineKind::Large),
			1000,
		));

		// Verify Charlie was registered successfully despite active service
		let profile = OperatorsProfile::<Runtime>::get(charlie.clone()).unwrap();
		assert!(profile.blueprints.contains(&0));
	});
}
