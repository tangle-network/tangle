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
fn unapplied_slash() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let eve = mock_pub_key(EVE);
		let bob = mock_pub_key(BOB);

		// Set up a job call that will result in an invalid submission
		let job_call_id = Services::next_job_call_id();
		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			service_id,
			KEYGEN_JOB_ID,
			bounded_vec![Field::Uint8(1)],
		));

		// Submit an invalid result that should trigger slashing
		let mut dkg = vec![0u8; 33];
		dkg[32] = 1;
		assert_ok!(Services::submit_result(
			RuntimeOrigin::signed(bob.clone()),
			0,
			job_call_id,
			bounded_vec![Field::from(BoundedVec::try_from(dkg).unwrap())],
		));

		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Slash the operator for the invalid result
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		// Verify the slash was recorded but not yet applied
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// Verify the correct event was emitted
		System::assert_has_event(RuntimeEvent::Services(crate::Event::UnappliedSlash {
			era: 0,
			index: 0,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * bob_exposed_restake_percentage).mul_floor(
				<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob),
			),
		}));
	});
}

#[test]
fn slash_account_not_an_operator() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let karen = mock_pub_key(23);

		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		let slash_percent = Percent::from_percent(50);

		// Try to slash an operator that is not active in this service
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				karen.clone(),
				service_id,
				slash_percent
			),
			Error::<Runtime>::OffenderNotOperator
		);
	});
}

#[test]
fn dispute() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let bob = mock_pub_key(BOB);
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Create a slash
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		let era = 0;
		let slash_index = 0;

		// Dispute the slash
		let dispute_origin =
			Services::query_dispute_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::dispute(
			RuntimeOrigin::signed(dispute_origin.clone()),
			era,
			slash_index
		));

		// Verify the slash was removed
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		// Calculate expected slash amount
		let bob_stake = <Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob);
		let expected_slash_amount =
			(slash_percent * bob_exposed_restake_percentage).mul_floor(bob_stake);

		// Verify the correct event was emitted
		System::assert_has_event(RuntimeEvent::Services(crate::Event::SlashDiscarded {
			era,
			index: slash_index,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: expected_slash_amount,
		}));
	});
}

#[test]
fn dispute_with_unauthorized_origin() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let eve = mock_pub_key(EVE);
		let bob = mock_pub_key(BOB);
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Create a slash
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		let era = 0;
		let slash_index = 0;

		// Try to dispute with an invalid origin
		assert_err!(
			Services::dispute(RuntimeOrigin::signed(eve.clone()), era, slash_index),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn dispute_an_already_applied_slash() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let eve = mock_pub_key(EVE);
		let bob = mock_pub_key(BOB);
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Create a slash
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		let era = 0;
		let slash_index = 0;

		// Simulate a slash being applied by removing it
		UnappliedSlashes::<Runtime>::remove(era, slash_index);

		// Try to dispute an already applied slash
		assert_err!(
			Services::dispute(RuntimeOrigin::signed(eve.clone()), era, slash_index),
			Error::<Runtime>::UnappliedSlashNotFound
		);
	});
}
