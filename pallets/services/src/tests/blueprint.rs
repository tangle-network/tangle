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
fn update_mbsm() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		assert_eq!(Pallet::<Runtime>::mbsm_latest_revision(), 0);
		assert_eq!(Pallet::<Runtime>::mbsm_address(0).unwrap(), MBSM);

		// Add a new revision
		let new_mbsm = {
			let mut v = MBSM;
			v.randomize();
			v
		};

		assert_ok!(Services::update_master_blueprint_service_manager(
			RuntimeOrigin::root(),
			new_mbsm
		));

		assert_eq!(Pallet::<Runtime>::mbsm_latest_revision(), 1);
		assert_eq!(Pallet::<Runtime>::mbsm_address(1).unwrap(), new_mbsm);
		// Old one should still be there
		assert_eq!(Pallet::<Runtime>::mbsm_address(0).unwrap(), MBSM);
		// Doesn't exist
		assert!(Pallet::<Runtime>::mbsm_address(2).is_err());
	});
}

#[test]
fn update_mbsm_not_root() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);
		assert_err!(
			Services::update_master_blueprint_service_manager(RuntimeOrigin::signed(alice), MBSM),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn create_service_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint,));

		let next_id = Services::next_blueprint_id();
		assert_eq!(next_id, 1);
		assert_events(vec![RuntimeEvent::Services(crate::Event::BlueprintCreated {
			owner: alice,
			blueprint_id: next_id - 1,
		})]);

		let (_, blueprint) = Services::blueprints(next_id - 1).unwrap();

		// The MBSM should be set on the blueprint
		assert_eq!(Pallet::<Runtime>::mbsm_address_of(&blueprint).unwrap(), MBSM);
		// The master manager revision should pinned to a specific revision that is equal to the
		// latest revision of the MBSM.
		assert_eq!(
			blueprint.master_manager_revision,
			MasterBlueprintServiceManagerRevision::Specific(0)
		);
	});
}
