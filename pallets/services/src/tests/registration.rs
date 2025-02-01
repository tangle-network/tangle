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
fn register_on_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);
		let bob_ecdsa_key = test_ecdsa_key();

		let registration_call = Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: bob_ecdsa_key,
				price_targets: price_targets(MachineKind::Large),
			},
			Default::default(),
			0,
		);
		assert_ok!(registration_call);

		assert_events(vec![RuntimeEvent::Services(crate::Event::Registered {
			provider: bob.clone(),
			blueprint_id: 0,
			preferences: OperatorPreferences {
				key: bob_ecdsa_key,
				price_targets: price_targets(MachineKind::Large),
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
				OperatorPreferences { key: bob_ecdsa_key, price_targets: Default::default() },
				Default::default(),
				0,
			),
			crate::Error::<Runtime>::AlreadyRegistered
		);

		// if we try to register with a non active operator, should fail
		assert_err!(
			Services::register(
				RuntimeOrigin::signed(mock_pub_key(10)),
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

		assert_events(vec![RuntimeEvent::Services(crate::Event::PreRegistration {
			operator: bob.clone(),
			blueprint_id: 0,
		})]);
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
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: bob_operator_ecdsa_key,
				price_targets: price_targets(MachineKind::Small)
			},
			Default::default(),
			0,
		));

		assert_eq!(
			Operators::<Runtime>::get(0, &bob).unwrap(),
			OperatorPreferences {
				key: bob_operator_ecdsa_key,
				price_targets: price_targets(MachineKind::Small)
			}
		);

		assert_events(vec![RuntimeEvent::Services(crate::Event::Registered {
			provider: bob.clone(),
			blueprint_id: 0,
			preferences: OperatorPreferences {
				key: bob_operator_ecdsa_key,
				price_targets: price_targets(MachineKind::Small),
			},
			registration_args: Default::default(),
		})]);

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
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));
		assert_ok!(Services::unregister(RuntimeOrigin::signed(bob.clone()), 0));
		assert!(!Operators::<Runtime>::contains_key(0, &bob));

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
