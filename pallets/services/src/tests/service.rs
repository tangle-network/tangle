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
use sp_core::U256;
use sp_runtime::Percent;
use tangle_primitives::services::*;

#[test]
fn request_service() {
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

		// Bob approves the request with security commitments
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			Percent::from_percent(10),
			vec![get_security_commitment(USDC, 10), get_security_commitment(WETH, 10)],
		));

		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceRequestApproved {
			operator: bob.clone(),
			request_id: 0,
			blueprint_id: 0,
			approved: vec![bob.clone()],
			pending_approvals: vec![charlie.clone(), dave.clone()],
		})]);

		// Charlie approves the request with security commitments
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			Percent::from_percent(20),
			vec![get_security_commitment(USDC, 15), get_security_commitment(WETH, 15)],
		));

		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceRequestApproved {
			operator: charlie.clone(),
			request_id: 0,
			blueprint_id: 0,
			approved: vec![bob.clone(), charlie.clone()],
			pending_approvals: vec![dave.clone()],
		})]);

		// Dave approves the request with security commitments
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(dave.clone()),
			0,
			Percent::from_percent(30),
			vec![get_security_commitment(USDC, 20), get_security_commitment(WETH, 20)],
		));

		assert_events(vec![
			RuntimeEvent::Services(crate::Event::ServiceRequestApproved {
				operator: dave.clone(),
				request_id: 0,
				blueprint_id: 0,
				approved: vec![bob.clone(), charlie.clone(), dave.clone()],
				pending_approvals: vec![],
			}),
			RuntimeEvent::Services(crate::Event::ServiceInitiated {
				owner: eve,
				request_id: 0,
				service_id: 0,
				blueprint_id: 0,
				assets: vec![Asset::Custom(USDC), Asset::Custom(WETH)],
			}),
		]);

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
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
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

		assert_ok!(Services::create_blueprint(
			RuntimeOrigin::signed(alice.clone()),
			blueprint.clone()
		));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
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
		assert_eq!(Assets::balance(USDC, Services::account_id()), payment);
		// Charlie Balance should be decreased by 5 USDC
		assert_eq!(Assets::balance(USDC, charlie.clone()), before_balance - payment);

		// Bob approves the request with security commitments
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			Percent::from_percent(10),
			vec![
				get_security_commitment(TNT, 10),
				get_security_commitment(USDC, 10),
				get_security_commitment(WETH, 10)
			],
		));

		// The request is now fully approved
		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		// The Payment should be now transferred to the MBSM.
		let mbsm_address = Pallet::<Runtime>::mbsm_address_of(&blueprint).unwrap();
		let mbsm_account_id = address_to_account_id(mbsm_address);
		assert_eq!(Assets::balance(USDC, mbsm_account_id), payment);
		// Pallet account should have 0 USDC
		assert_eq!(Assets::balance(USDC, Services::account_id()), 0);

		// Now the service should be initiated
		assert!(Instances::<Runtime>::contains_key(0));
	});
}

#[test]
fn request_service_with_payment_token() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(
			RuntimeOrigin::signed(alice.clone()),
			blueprint.clone()
		));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
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

		// Bob approves the request with security commitments
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			Percent::from_percent(10),
			vec![
				get_security_commitment(TNT, 10),
				get_security_commitment(USDC, 10),
				get_security_commitment(WETH, 10)
			],
		));

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

		assert_ok!(Services::create_blueprint(
			RuntimeOrigin::signed(alice.clone()),
			blueprint.clone()
		));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
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

		assert_ok!(Services::create_blueprint(
			RuntimeOrigin::signed(alice.clone()),
			blueprint.clone()
		));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
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
		assert_eq!(Assets::balance(USDC, Services::account_id()), payment);
		// Charlie Balance should be decreased by 5 USDC
		assert_eq!(Assets::balance(USDC, charlie.clone()), before_balance - payment);

		// Bob rejects the request
		assert_ok!(Services::reject(RuntimeOrigin::signed(bob.clone()), 0));

		// The Payment should be now refunded to the requester.
		// Pallet account should have 0 USDC
		assert_eq!(Assets::balance(USDC, Services::account_id()), 0);
		// Charlie Balance should be back to the original
		assert_eq!(Assets::balance(USDC, charlie), before_balance);
	});
}
