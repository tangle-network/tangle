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
use sp_core::U256;
use sp_runtime::traits::{BlakeTwo256, Hash};

#[test]
fn request_service() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Register multiple operators
		let bob = mock_pub_key(BOB);
		let bob_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			bob_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		let charlie = mock_pub_key(CHARLIE);
		let charlie_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			charlie.clone(),
			0,
			charlie_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		let dave = mock_pub_key(DAVE);
		let dave_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			dave.clone(),
			0,
			dave_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		let eve = mock_pub_key(EVE);

		// Native asset exposure too low should fail
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(eve.clone()),
				None,
				0,
				vec![alice.clone()],
				vec![bob.clone(), charlie.clone(), dave.clone()],
				Default::default(),
				vec![
					get_security_requirement(USDC, &[10, 20]),
					get_security_requirement(WETH, &[15, 25]),
					get_security_requirement(TNT, &[5, 10]),
				],
				100,
				Asset::Custom(USDC),
				0,
				MembershipModel::Fixed { min_operators: 1 },
			),
			Error::<Runtime>::NativeAssetExposureTooLow,
		);

		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			Default::default(),
			vec![
				get_security_requirement(USDC, &[10, 20]),
				get_security_requirement(WETH, &[10, 20])
			],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 3 },
		));

		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// Bob approves the request (must match requirements order: USDC, WETH, TNT)
		// Note: TNT (native asset) is automatically added by the system if not present
		let security_commitments = vec![
			get_security_commitment(USDC, 10),
			get_security_commitment(WETH, 10),
			get_security_commitment(TNT, 10),
		];
		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 0, security_commitments));

		let events: Vec<RuntimeEvent> = System::events()
			.into_iter()
			.map(|e| e.event)
			.filter(|e| matches!(e, RuntimeEvent::Services(_)))
			.collect();

		assert!(events.contains(&RuntimeEvent::Services(crate::Event::ServiceRequestApproved {
			operator: bob.clone(),
			request_id: 0,
			blueprint_id: 0,
			approved: vec![bob.clone()],
			pending_approvals: vec![charlie.clone(), dave.clone()],
		})));

		// Charlie approves the request with different security commitments (order: USDC, WETH, TNT)
		let security_commitments_charlie = vec![
			get_security_commitment(USDC, 15),
			get_security_commitment(WETH, 15),
			get_security_commitment(TNT, 15),
		];
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			security_commitments_charlie
		));

		let events: Vec<RuntimeEvent> = System::events()
			.into_iter()
			.map(|e| e.event)
			.filter(|e| matches!(e, RuntimeEvent::Services(_)))
			.collect();

		assert!(events.contains(&RuntimeEvent::Services(crate::Event::ServiceRequestApproved {
			operator: charlie.clone(),
			request_id: 0,
			blueprint_id: 0,
			approved: vec![bob.clone(), charlie.clone()],
			pending_approvals: vec![dave.clone()],
		})));

		// Now let's try to approve with misordered security commitments for Dave. This should fail
		// because the security commitments are misordered. They must be in the same order as the
		// security requirements.
		let invalid_security_commitments = vec![
			get_security_commitment(TNT, 20),
			get_security_commitment(USDC, 20),
			get_security_commitment(WETH, 20),
		];
		assert_err!(
			Services::approve(RuntimeOrigin::signed(dave.clone()), 0, invalid_security_commitments),
			Error::<Runtime>::InvalidSecurityCommitments,
		);

		// Dave approves the request with security commitments (order: USDC, WETH, TNT)
		let security_commitments_dave = vec![
			get_security_commitment(USDC, 20),
			get_security_commitment(WETH, 20),
			get_security_commitment(TNT, 20),
		];
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(dave.clone()),
			0,
			security_commitments_dave
		));

		let service = Services::services(0).unwrap();
		let operator_security_commitments = service.operator_security_commitments;

		let events: Vec<RuntimeEvent> = System::events()
			.into_iter()
			.map(|e| e.event)
			.filter(|e| matches!(e, RuntimeEvent::Services(_)))
			.collect();

		assert!(events.contains(&RuntimeEvent::Services(crate::Event::ServiceRequestApproved {
			operator: dave.clone(),
			request_id: 0,
			blueprint_id: 0,
			approved: vec![bob.clone(), charlie.clone(), dave.clone()],
			pending_approvals: vec![],
		})));

		assert!(events.contains(&RuntimeEvent::Services(crate::Event::ServiceInitiated {
			owner: eve,
			request_id: 0,
			service_id: 0,
			blueprint_id: 0,
			operator_security_commitments,
		})));

		// The request is now fully approved
		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		// Now the service should be initiated
		assert!(Instances::<Runtime>::contains_key(0));

		// The service should also be added to the services for each operator.
		let profile = OperatorsProfile::<Runtime>::get(bob).unwrap();
		assert!(profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(charlie).unwrap();
		assert!(profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(dave).unwrap();
		assert!(profile.services.contains(&0));
	});
}

#[test]
fn request_service_with_no_assets() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		let bob_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			bob_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));
		let eve = mock_pub_key(EVE);
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(eve.clone()),
				None,
				0,
				vec![alice.clone()],
				vec![bob.clone()],
				Default::default(),
				vec![], // no assets
				100,
				Asset::Custom(USDC),
				0,
				MembershipModel::Fixed { min_operators: 1 },
			),
			Error::<Runtime>::NoAssetsProvided
		);
	});
}

#[test]
fn request_service_with_payment_asset() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();

		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint.clone()));
		let bob = mock_pub_key(BOB);
		let bob_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			bob_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		let payment = 5 * 10u128.pow(6); // 5 USDC
		let charlie = mock_pub_key(CHARLIE);
		let before_balance = Assets::balance(USDC, charlie.clone());
		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie.clone()),
			None,
			0,
			vec![],
			vec![bob.clone()],
			Default::default(),
			vec![
				get_security_requirement(TNT, &[10, 20]),
				get_security_requirement(USDC, &[10, 20]),
				get_security_requirement(WETH, &[10, 20])
			],
			100,
			Asset::Custom(USDC),
			payment,
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// The Pallet account now has 5 USDC
		assert_eq!(Assets::balance(USDC, Services::pallet_account()), payment);
		// Charlie Balance should be decreased by 5 USDC
		assert_eq!(Assets::balance(USDC, charlie.clone()), before_balance - payment);

		// Bob approves the request (must match requirements order: TNT, USDC, WETH)
		let security_commitments = vec![
			get_security_commitment(TNT, 10),
			get_security_commitment(USDC, 10),
			get_security_commitment(WETH, 10),
		];
		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 0, security_commitments));

		// The request is now fully approved
		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		// The Payment should be now transferred to the MBSM.
		let mbsm_address = Pallet::<Runtime>::mbsm_address_of(&blueprint).unwrap();
		let mbsm_account_id = address_to_account_id(mbsm_address);
		assert_eq!(Assets::balance(USDC, mbsm_account_id), payment);
		// Pallet account should have 0 USDC
		assert_eq!(Assets::balance(USDC, Services::pallet_account()), 0);

		// Now the service should be initiated
		assert!(Instances::<Runtime>::contains_key(0));
	});
}

#[test]
fn request_service_with_payment_erc20_token() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();

		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint.clone()));
		let bob = mock_pub_key(BOB);
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		let payment = 5 * 10u128.pow(6); // 5 USDC
		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::request(
			RuntimeOrigin::signed(address_to_account_id(mock_address(CHARLIE))),
			Some(account_id_to_address(charlie.clone())),
			0,
			vec![],
			vec![bob.clone()],
			Default::default(),
			vec![
				get_security_requirement(TNT, &[10, 20]),
				get_security_requirement(USDC, &[10, 20]),
				get_security_requirement(WETH, &[10, 20])
			],
			100,
			Asset::Erc20(USDC_ERC20),
			payment,
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// The Pallet address now has 5 USDC
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, Services::pallet_evm_account())
				.map(|(b, _)| b),
			U256::from(payment)
		);

		// Bob approves the request (must match requirements order: TNT, USDC, WETH)
		let security_commitments = vec![
			get_security_commitment(TNT, 10),
			get_security_commitment(USDC, 10),
			get_security_commitment(WETH, 10),
		];
		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 0, security_commitments));

		// The request is now fully approved
		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		// The Payment should be now transferred to the MBSM.
		let mbsm_address = Pallet::<Runtime>::mbsm_address_of(&blueprint).unwrap();
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, mbsm_address).map(|(b, _)| b),
			U256::from(payment)
		);
		// Pallet account should have 0 USDC
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, Services::pallet_evm_account())
				.map(|(b, _)| b),
			U256::from(0)
		);

		// Now the service should be initiated
		assert!(Instances::<Runtime>::contains_key(0));
	});
}

#[test]
fn reject_service_with_payment_token() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();

		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint.clone()));
		let bob = mock_pub_key(BOB);
		let bob_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			bob_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		let payment = 5 * 10u128.pow(6); // 5 USDC
		let charlie_address = mock_address(CHARLIE);
		let charlie_evm_account_id = address_to_account_id(charlie_address);
		let before_balance = Services::query_erc20_balance_of(USDC_ERC20, charlie_address)
			.map(|(b, _)| b)
			.unwrap_or_default();
		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie_evm_account_id),
			Some(charlie_address),
			0,
			vec![],
			vec![bob.clone()],
			Default::default(),
			vec![
				get_security_requirement(TNT, &[10, 20]),
				get_security_requirement(USDC, &[10, 20]),
				get_security_requirement(WETH, &[10, 20])
			],
			100,
			Asset::Erc20(USDC_ERC20),
			payment,
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// The Pallet address now has 5 USDC
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, Services::pallet_evm_account())
				.map(|(b, _)| b),
			U256::from(payment)
		);
		// Charlie Balance should be decreased by 5 USDC
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, charlie_address).map(|(b, _)| b),
			before_balance - U256::from(payment)
		);

		// Bob rejects the request
		assert_ok!(Services::reject(RuntimeOrigin::signed(bob.clone()), 0));

		// The Payment should be now refunded to the requester.
		// Pallet account should have 0 USDC
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, Services::pallet_evm_account())
				.map(|(b, _)| b),
			U256::from(0)
		);
		// Charlie Balance should be back to the original
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, charlie_address).map(|(b, _)| b),
			before_balance
		);
	});
}

#[test]
fn reject_service_with_payment_asset() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();

		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint.clone()));
		let bob = mock_pub_key(BOB);
		let bob_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			bob_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		let payment = 5 * 10u128.pow(6); // 5 USDC
		let charlie = mock_pub_key(CHARLIE);
		let before_balance = Assets::balance(USDC, charlie.clone());
		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie.clone()),
			None,
			0,
			vec![],
			vec![bob.clone()],
			Default::default(),
			vec![
				get_security_requirement(TNT, &[10, 20]),
				get_security_requirement(USDC, &[10, 20]),
				get_security_requirement(WETH, &[10, 20])
			],
			100,
			Asset::Custom(USDC),
			payment,
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// The Pallet account now has 5 USDC
		assert_eq!(Assets::balance(USDC, Services::pallet_account()), payment);
		// Charlie Balance should be decreased by 5 USDC
		assert_eq!(Assets::balance(USDC, charlie.clone()), before_balance - payment);

		// Bob rejects the request
		assert_ok!(Services::reject(RuntimeOrigin::signed(bob.clone()), 0));

		// The Payment should be now refunded to the requester.
		// Pallet account should have 0 USDC
		assert_eq!(Assets::balance(USDC, Services::pallet_account()), 0);
		// Charlie Balance should be back to the original
		assert_eq!(Assets::balance(USDC, charlie), before_balance);
	});
}

#[test]
fn test_service_creation_dynamic_max_operators() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		// Create blueprint with free pricing since this test is about operator validation, not
		// payment
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint_with_pricing(
			RuntimeOrigin::signed(alice.clone()),
			blueprint,
			PricingModel::PayOnce { amount: 0 }
		));

		// Register maximum number of operators (using mock accounts)
		let max_operators = 10;
		let mut operators = Vec::new();

		// Create 10 operators with sequential keys
		for i in 1..=10 {
			let operator = mock_pub_key_from_fixed_bytes([i as u8; 32]);
			// Give operator sufficient balance to join
			Balances::make_free_balance_be(&operator, 10_000_000);
			assert_ok!(join_and_register(
				operator.clone(),
				0,
				test_ecdsa_key(),
				1000,
				Some("https://example.com/rpc")
			));
			operators.push(operator);
		}

		let eve = mock_pub_key(EVE);

		// Try to create service with exactly 10 operators - should succeed
		assert_ok!(Services::request(
			RuntimeOrigin::signed(alice.clone()),
			None,
			0,
			vec![alice.clone()],
			operators.clone(),
			Default::default(),
			vec![get_security_requirement(USDC, &[10, 20])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Dynamic { min_operators: 1, max_operators: Some(max_operators) },
		));

		// Try to create service with 11 operators - should fail
		let extra_operator = mock_pub_key_from_fixed_bytes([11u8; 32]);
		// Give extra operator sufficient balance to join
		Balances::make_free_balance_be(&extra_operator, 10_000_000);
		assert_ok!(join_and_register(
			extra_operator.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));
		operators.push(extra_operator);

		assert_err!(
			Services::request(
				RuntimeOrigin::signed(alice.clone()),
				None,
				0,
				vec![alice.clone()],
				operators,
				Default::default(),
				vec![get_security_requirement(USDC, &[10, 20])],
				100,
				Asset::Custom(USDC),
				0,
				MembershipModel::Dynamic { min_operators: 1, max_operators: Some(max_operators) },
			),
			Error::<Runtime>::TooManyOperators
		);
	});
}

#[test]
fn test_service_creation_fixed_min_operators() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		// Create blueprint with free pricing since this test is about operator validation, not
		// payment
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint_with_pricing(
			RuntimeOrigin::signed(alice.clone()),
			blueprint,
			PricingModel::PayOnce { amount: 0 }
		));

		// Register some operators
		let bob = mock_pub_key(BOB);
		let bob_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			bob_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		let charlie = mock_pub_key(CHARLIE);
		let charlie_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			charlie.clone(),
			0,
			charlie_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		let eve = mock_pub_key(EVE);

		// Try to create service with zero operators - should fail
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(alice.clone()),
				None,
				0,
				vec![alice.clone()],
				vec![],
				Default::default(),
				vec![get_security_requirement(USDC, &[10, 20])],
				100,
				Asset::Custom(USDC),
				0,
				MembershipModel::Fixed { min_operators: 0 },
			),
			Error::<Runtime>::TooFewOperators
		);

		// Try to create service with fewer operators than min_operators - should fail
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(alice.clone()),
				None,
				0,
				vec![alice.clone()],
				vec![bob.clone()],
				Default::default(),
				vec![get_security_requirement(USDC, &[10, 20])],
				100,
				Asset::Custom(USDC),
				0,
				MembershipModel::Fixed { min_operators: 2 },
			),
			Error::<Runtime>::TooFewOperators
		);

		// Try to create service with exactly min_operators - should succeed
		assert_ok!(Services::request(
			RuntimeOrigin::signed(alice.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone()],
			Default::default(),
			vec![get_security_requirement(USDC, &[10, 20])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 2 },
		));
	});
}

#[test]
fn test_service_creation_invalid_operators() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		// Create blueprint with free pricing since this test is about operator validation, not
		// payment
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint_with_pricing(
			RuntimeOrigin::signed(alice.clone()),
			blueprint,
			PricingModel::PayOnce { amount: 0 }
		));

		// Register one valid operator
		let bob = mock_pub_key(BOB);
		let bob_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			bob_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		// Create an unregistered operator
		let unregistered = mock_pub_key(CHARLIE);
		let eve = mock_pub_key(EVE);

		// Try to create service with an unregistered operator - should fail
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(alice.clone()),
				None,
				0,
				vec![alice.clone()],
				vec![bob.clone(), unregistered.clone()],
				Default::default(),
				vec![get_security_requirement(USDC, &[10, 20])],
				100,
				Asset::Custom(USDC),
				0,
				MembershipModel::Fixed { min_operators: 2 },
			),
			Error::<Runtime>::OperatorNotActive
		);
	});
}

#[test]
fn test_service_creation_duplicate_operators() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		// Create blueprint with free pricing since this test is about operator validation, not
		// payment
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint_with_pricing(
			RuntimeOrigin::signed(alice.clone()),
			blueprint,
			PricingModel::PayOnce { amount: 0 }
		));

		// Register operators
		let bob = mock_pub_key(BOB);
		let bob_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			bob_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		let charlie = mock_pub_key(CHARLIE);
		let charlie_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			charlie.clone(),
			0,
			charlie_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		let eve = mock_pub_key(EVE);

		// Try to create service with duplicate operators - should fail
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(alice.clone()),
				None,
				0,
				vec![alice.clone()],
				vec![bob.clone(), bob.clone()],
				Default::default(),
				vec![get_security_requirement(USDC, &[10, 20])],
				100,
				Asset::Custom(USDC),
				0,
				MembershipModel::Fixed { min_operators: 2 },
			),
			Error::<Runtime>::DuplicateOperator
		);
	});
}

#[test]
fn test_service_creation_inactive_operators() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		// Create blueprint with free pricing since this test is about operator validation, not
		// payment
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint_with_pricing(
			RuntimeOrigin::signed(alice.clone()),
			blueprint,
			PricingModel::PayOnce { amount: 0 }
		));

		// Register operators
		let bob = mock_pub_key(BOB);
		let bob_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			bob_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		let charlie = mock_pub_key(CHARLIE);
		let charlie_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			charlie.clone(),
			0,
			charlie_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		// Deactivate one operator
		assert_ok!(MultiAssetDelegation::go_offline(RuntimeOrigin::signed(charlie.clone())));

		let eve = mock_pub_key(EVE);

		// Try to create service with an inactive operator - should fail
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(alice.clone()),
				None,
				0,
				vec![alice.clone()],
				vec![bob.clone(), charlie.clone()],
				Default::default(),
				vec![get_security_requirement(USDC, &[10, 20])],
				100,
				Asset::Custom(USDC),
				0,
				MembershipModel::Fixed { min_operators: 2 },
			),
			Error::<Runtime>::OperatorNotActive
		);

		// Service creation with only active operators should succeed
		assert_ok!(Services::request(
			RuntimeOrigin::signed(alice.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(USDC, &[10, 20])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));
	});
}

#[test]
fn test_termination_with_partial_approvals() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		// Create blueprint with free pricing since this test is about termination logic, not
		// payment
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint_with_pricing(
			RuntimeOrigin::signed(alice.clone()),
			blueprint,
			PricingModel::PayOnce { amount: 0 }
		));

		// Register operators
		let bob = mock_pub_key(BOB);
		let bob_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			bob_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		let charlie = mock_pub_key(CHARLIE);
		let charlie_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			charlie.clone(),
			0,
			charlie_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		let dave = mock_pub_key(DAVE);
		let dave_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			dave.clone(),
			0,
			dave_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		// Create service request
		let eve = mock_pub_key(EVE);
		assert_ok!(Services::request(
			RuntimeOrigin::signed(alice.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			Default::default(),
			vec![get_security_requirement(USDC, &[10, 20])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 3 },
		));

		// Only two operators approve (must include TNT as system auto-adds it)
		let security_commitments_bob =
			vec![get_security_commitment(USDC, 10), get_security_commitment(TNT, 10)];
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			security_commitments_bob
		));

		let security_commitments_charlie =
			vec![get_security_commitment(USDC, 15), get_security_commitment(TNT, 15)];
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			security_commitments_charlie
		));

		// Attempt to terminate service with partial approvals - should fail
		assert_err!(
			Services::terminate(RuntimeOrigin::signed(alice.clone()), 0),
			Error::<Runtime>::ServiceNotFound
		);

		// Complete the approvals
		let security_commitments_dave =
			vec![get_security_commitment(USDC, 20), get_security_commitment(TNT, 20)];
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(dave.clone()),
			0,
			security_commitments_dave
		));

		// Now termination should succeed - service owner (alice) terminates
		assert_ok!(Services::terminate(RuntimeOrigin::signed(alice.clone()), 0));

		// Verify service is terminated
		assert!(!Instances::<Runtime>::contains_key(0));
	});
}

#[test]
fn test_operator_offline_during_active_service() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		// Create blueprint with free pricing since this test is about operator offline logic, not
		// payment
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint_with_pricing(
			RuntimeOrigin::signed(alice.clone()),
			blueprint,
			PricingModel::PayOnce { amount: 0 }
		));

		// Register operator
		let bob = mock_pub_key(BOB);
		let bob_ecdsa_key = test_ecdsa_key();
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			bob_ecdsa_key,
			1000,
			Some("https://example.com/rpc")
		));

		// Create service
		let eve = mock_pub_key(EVE);
		assert_ok!(Services::request(
			RuntimeOrigin::signed(alice.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(USDC, &[10, 20])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Approve service request (must include TNT as system auto-adds it)
		let security_commitments =
			vec![get_security_commitment(USDC, 10), get_security_commitment(TNT, 10)];
		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 0, security_commitments));

		// Verify service is active
		assert!(Instances::<Runtime>::contains_key(0));

		// Attempt to go offline while service is active - should fail
		assert_err!(
			MultiAssetDelegation::go_offline(RuntimeOrigin::signed(bob.clone())),
			pallet_multi_asset_delegation::Error::<Runtime>::CannotGoOfflineWithActiveServices
		);

		// Terminate the service - service owner (alice) terminates
		assert_ok!(Services::terminate(RuntimeOrigin::signed(alice.clone()), 0));

		// Now operator should be able to go offline
		assert_ok!(MultiAssetDelegation::go_offline(RuntimeOrigin::signed(bob.clone())));
	});
}
