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
fn test_zero_percentage_slash() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let operator = mock_pub_key(BOB);

		// Try to slash zero stake
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				operator.clone(),
				service_id,
				Percent::from_percent(0)
			),
			Error::<Runtime>::InvalidSlashPercentage
		);
	});
}

#[test]
fn unapplied_slash() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, .. } = deploy();
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
			slash_percent,
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
fn dispute_and_verify_event() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, .. } = deploy();
		let bob = mock_pub_key(BOB);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Create a slash
		let slash_percent = Percent::from_percent(50);
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		// Get the unapplied slash
		let (era, index) = UnappliedSlashes::<Runtime>::iter_keys().next().unwrap();

		// Dispute the slash
		let dispute_origin =
			Services::query_dispute_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::dispute(RuntimeOrigin::signed(dispute_origin.clone()), era, index));

		// Verify the slash was removed
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		// Verify the correct event was emitted
		System::assert_has_event(RuntimeEvent::Services(crate::Event::SlashDiscarded {
			era,
			index,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			slash_percent,
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

#[test]
fn test_slash_with_multiple_asset_types() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);

		// Setup native stake
		let native_stake = 10_000;
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			native_stake,
			Default::default(),
		));

		// Setup USDC stake
		let usdc_stake = 100_000;
		mint_tokens(USDC, mock_pub_key(ALICE), delegator.clone(), usdc_stake * 10u128.pow(6));
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(delegator.clone()),
			Asset::Custom(USDC),
			usdc_stake,
			None,
			None,
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			Asset::Custom(USDC),
			usdc_stake,
			Default::default(),
		));

		// Setup WETH stake
		let weth_stake = 100_000;
		mint_tokens(WETH, mock_pub_key(BOB), delegator.clone(), weth_stake * 10u128.pow(18));
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(delegator.clone()),
			Asset::Custom(WETH),
			weth_stake,
			None,
			None,
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			Asset::Custom(WETH),
			weth_stake,
			Default::default(),
		));

		// Apply slash
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();
		let slash_percent = Percent::from_percent(50);

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			slash_percent
		));

		// Get the unapplied slash and verify amounts
		let unapplied_slash = UnappliedSlashes::<Runtime>::get(0, 0).unwrap();

		// TODO: Verify slash is applied correctly
		assert_ok!(MultiAssetDelegation::slash_operator(&unapplied_slash));
	});
}

#[test]
fn test_slash_with_no_blueprint_selection() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);

		// Initial stake amounts
		let native_stake = 10_000;

		// Delegate assets but don't select any blueprints (uses default empty selection)
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			native_stake,
			Default::default(), // Default blueprint selection is empty
		));

		// Get service and slashing origin
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();
		let slash_percent = Percent::from_percent(50);

		// Execute slash
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			slash_percent
		));

		// Verify the unapplied slash record
		let unapplied_slash = UnappliedSlashes::<Runtime>::get(0, 0).unwrap();

		// TODO: Ensure that the slash is applied correctly
		assert_ok!(MultiAssetDelegation::slash_operator(&unapplied_slash));
	});
}

#[test]
fn test_slash_with_native_delegation() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, blueprint_id, .. } = deploy();
		let operator = mock_pub_key(BOB);
		let delegator1 = mock_pub_key(CHARLIE);

		// Initial setup
		let initial_stake = 10_000;
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(delegator1.clone()),
			operator.clone(),
			initial_stake,
			vec![blueprint_id].into(),
		));

		// Verify initial delegation storage
		let delegator1_metadata = MultiAssetDelegation::delegators(delegator1.clone()).unwrap();
		let initial_delegation = delegator1_metadata
			.delegations
			.iter()
			.find(|d| d.operator == operator)
			.map(|d| d.amount)
			.unwrap_or(0);
		assert_eq!(initial_delegation, initial_stake);

		assert_eq!(delegator1_metadata.total_nomination_delegations(), initial_stake);
		assert_eq!(delegator1_metadata.total_non_nomination_delegations(), 0);

		// Start a slash
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();
		let slash_percent = Percent::from_percent(50);

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			slash_percent
		));

		// Verify slash amount matches security commitment
		let unapplied_slashes: Vec<_> = UnappliedSlashes::<Runtime>::iter_prefix(0).collect();
		assert_eq!(unapplied_slashes.len(), 1, "Should be exactly one unapplied slash");

		// TODO: Apply the slash
	});
}

#[test]
fn test_slash_with_partial_amounts() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let operator = mock_pub_key(BOB);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Test various partial slash percentages
		let slash_percentages = vec![
			Percent::from_percent(33), // 33%
			Percent::from_percent(17), // 17%
			Percent::from_percent(7),  // 7%
		];

		for slash_percent in slash_percentages {
			assert_ok!(Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				operator.clone(),
				service_id,
				slash_percent
			));

			// Apply the slash
			let slash = UnappliedSlashes::<Runtime>::get(0, 0).unwrap();
			assert_ok!(MultiAssetDelegation::slash_operator(&slash));
		}
	});
}

#[test]
fn test_slash_with_invalid_operator() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let invalid_operator = mock_pub_key(99); // Non-existent operator
		let delegator = mock_pub_key(CHARLIE);

		// Attempt to slash an unregistered operator
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();
		let slash_percent = Percent::from_percent(50);

		// Should fail because operator is not registered
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				invalid_operator.clone(),
				service_id,
				slash_percent
			),
			Error::<Runtime>::OffenderNotOperator
		);

		// Try to slash with an account that is registered but not as an operator
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				delegator.clone(),
				service_id,
				slash_percent
			),
			Error::<Runtime>::OffenderNotOperator
		);
	});
}

#[test]
fn test_slash_with_multiple_services() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, .. } = deploy();

		// Create second service
		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let eve = mock_pub_key(EVE);
		let service2_id = Services::next_instance_id();
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			blueprint_id,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(USDC, &[10, 20])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			service2_id,
			vec![get_security_commitment(USDC, 10), get_security_commitment(TNT, 10)],
		));

		// Create slashes for both services
		let service1 = Services::services(service_id).unwrap();
		let slashing_origin1 =
			Services::query_slashing_origin(&service1).map(|(o, _)| o.unwrap()).unwrap();

		let service2 = Services::services(service2_id).unwrap();
		let slashing_origin2 =
			Services::query_slashing_origin(&service2).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin1.clone()),
			bob.clone(),
			service_id,
			Percent::from_percent(50)
		));

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin2.clone()),
			bob.clone(),
			service2_id,
			Percent::from_percent(25)
		));

		// Apply slashes
		let slashes: Vec<_> = UnappliedSlashes::<Runtime>::iter_prefix(0).collect();
		for (_, slash) in slashes {
			assert_ok!(MultiAssetDelegation::slash_operator(&slash));
		}
	});
}

#[test]
fn test_slash_with_rewards_distribution() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let operator = mock_pub_key(BOB);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Create and apply slash
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			Percent::from_percent(50)
		));

		let slash = UnappliedSlashes::<Runtime>::get(0, 0).unwrap();
		assert_ok!(MultiAssetDelegation::slash_operator(&slash));
	});
}

#[test]
fn test_slash_with_unauthorized_origin() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let bob = mock_pub_key(BOB);
		let eve = mock_pub_key(EVE);
		let slash_percent = Percent::from_percent(50);

		// Try to slash with unauthorized origin (EVE)
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(eve.clone()),
				bob.clone(),
				service_id,
				slash_percent
			),
			DispatchError::BadOrigin
		);
	});
}
