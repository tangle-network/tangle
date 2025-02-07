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
use sp_core::{H160, U256};
use sp_runtime::Percent;

#[test]
fn test_payment_refunds_on_failure() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		// Create blueprint
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Register operator
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

		// Test Case 1: Refund on operator rejection
		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie.clone()),
			None,
			0,
			vec![],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(USDC, &[10, 20])],
			100,
			Asset::Custom(USDC),
			payment,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Verify payment is held by pallet
		assert_eq!(Assets::balance(USDC, Services::pallet_account()), payment);
		assert_eq!(Assets::balance(USDC, charlie.clone()), before_balance - payment);

		// Bob rejects the request
		assert_ok!(Services::reject(RuntimeOrigin::signed(bob.clone()), 0));

		// Verify payment is refunded
		assert_eq!(Assets::balance(USDC, Services::pallet_account()), 0);
		assert_eq!(Assets::balance(USDC, charlie.clone()), before_balance);

		// Test Case 2: Refund on ERC20 payment
		let charlie_address = mock_address(CHARLIE);
		let charlie_evm_account_id = address_to_account_id(charlie_address);
		let before_erc20_balance = Services::query_erc20_balance_of(USDC_ERC20, charlie_address)
			.map(|(b, _)| b)
			.unwrap_or_default();

		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie_evm_account_id.clone()),
			Some(charlie_address),
			0,
			vec![],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(USDC, &[10, 20])],
			100,
			Asset::Erc20(USDC_ERC20),
			payment,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Verify ERC20 payment is held by pallet
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, Services::pallet_evm_account())
				.map(|(b, _)| b),
			U256::from(payment)
		);

		// Bob rejects the request
		assert_ok!(Services::reject(RuntimeOrigin::signed(bob.clone()), 0));

		// Verify ERC20 payment is refunded
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, Services::pallet_evm_account())
				.map(|(b, _)| b),
			U256::from(0)
		);
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, charlie_address).map(|(b, _)| b),
			before_erc20_balance
		);

		// Test Case 3: Refund on native currency payment
		let before_native_balance = Balances::free_balance(charlie.clone());
		let native_payment = 1000000000000000000u128; // 1 TNT

		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie.clone()),
			None,
			0,
			vec![],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(USDC, &[10, 20])],
			100,
			Asset::Custom(0),
			native_payment,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Verify native payment is held by pallet
		assert_eq!(Balances::free_balance(Services::pallet_account()), native_payment);
		assert_eq!(Balances::free_balance(charlie.clone()), before_native_balance - native_payment);

		// Bob rejects the request
		assert_ok!(Services::reject(RuntimeOrigin::signed(bob.clone()), 0));

		// Verify native payment is refunded
		assert_eq!(Balances::free_balance(Services::pallet_account()), 0);
		assert_eq!(Balances::free_balance(charlie.clone()), before_native_balance);
	});
}

#[test]
fn test_payment_distribution_operators() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		// Create blueprint
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(
			RuntimeOrigin::signed(alice.clone()),
			blueprint.clone()
		));

		// Register operators
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

		// Test Case 1: Custom Asset Payment (USDC)
		let payment = 5 * 10u128.pow(6); // 5 USDC
		let eve = mock_pub_key(EVE);
		let before_balance = Assets::balance(USDC, eve.clone());

		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![],
			vec![bob.clone(), charlie.clone()],
			Default::default(),
			vec![get_security_requirement(USDC, &[10, 20])],
			100,
			Asset::Custom(USDC),
			payment,
			MembershipModel::Fixed { min_operators: 2 },
		));

		// Verify payment is held by pallet
		assert_eq!(Assets::balance(USDC, Services::pallet_account()), payment);
		assert_eq!(Assets::balance(USDC, eve.clone()), before_balance - payment);

		// Approve service request
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			Percent::from_percent(10),
			vec![get_security_commitment(USDC, 10)],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			Percent::from_percent(20),
			vec![get_security_commitment(USDC, 15)],
		));

		// Verify payment is transferred to MBSM
		let mbsm_address = Services::mbsm_address_of(&blueprint).unwrap();
		let mbsm_account_id = address_to_account_id(mbsm_address);
		assert_eq!(Assets::balance(USDC, mbsm_account_id), payment);
		assert_eq!(Assets::balance(USDC, Services::pallet_account()), 0);

		// Test Case 2: ERC20 Token Payment
		let charlie_address = mock_address(CHARLIE);
		let charlie_evm_account_id = address_to_account_id(charlie_address);
		let before_erc20_balance = Services::query_erc20_balance_of(USDC_ERC20, charlie_address)
			.map(|(b, _)| b)
			.unwrap_or_default();

		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie_evm_account_id.clone()),
			Some(charlie_address),
			0,
			vec![],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(USDC, &[10, 20])],
			100,
			Asset::Erc20(USDC_ERC20),
			payment,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Verify ERC20 payment is held by pallet
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, Services::pallet_evm_account())
				.map(|(b, _)| b),
			U256::from(payment)
		);

		// Bob approves
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			1,
			Percent::from_percent(10),
			vec![get_security_commitment(USDC, 10)],
		));

		// Verify ERC20 payment is transferred to MBSM
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, mbsm_address).map(|(b, _)| b),
			U256::from(payment)
		);
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, Services::pallet_evm_account())
				.map(|(b, _)| b),
			U256::from(0)
		);

		// Test Case 3: Native Currency Payment
		let native_payment = 1000000000000000000u128; // 1 TNT
		let before_native_balance = Balances::free_balance(eve.clone());

		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(USDC, &[10, 20])],
			100,
			Asset::Custom(0),
			native_payment,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Verify native payment is held by pallet
		assert_eq!(Balances::free_balance(Services::pallet_account()), native_payment);
		assert_eq!(Balances::free_balance(eve.clone()), before_native_balance - native_payment);

		// Bob approves
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			2,
			Percent::from_percent(10),
			vec![get_security_commitment(USDC, 10)],
		));

		// Verify native payment is transferred to MBSM
		assert_eq!(Balances::free_balance(mbsm_account_id), native_payment);
		assert_eq!(Balances::free_balance(Services::pallet_account()), 0);
	});
}

#[test]
fn test_payment_multiple_asset_types() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		// Create blueprint
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(
			RuntimeOrigin::signed(alice.clone()),
			blueprint.clone()
		));

		// Register operator
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		// Test Case 1: Multiple asset security requirements
		let eve = mock_pub_key(EVE);
		let payment = 5 * 10u128.pow(6); // 5 USDC
		let before_balance = Assets::balance(USDC, eve.clone());

		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![],
			vec![bob.clone()],
			Default::default(),
			vec![
				get_security_requirement(USDC, &[10, 20]),
				get_security_requirement(WETH, &[15, 25]),
				get_security_requirement(TNT, &[5, 10]),
			],
			100,
			Asset::Custom(USDC),
			payment,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Verify payment is held by pallet
		assert_eq!(Assets::balance(USDC, Services::pallet_account()), payment);
		assert_eq!(Assets::balance(USDC, eve.clone()), before_balance - payment);

		// Bob approves with security commitments for all assets
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			Percent::from_percent(10),
			vec![
				get_security_commitment(USDC, 10),
				get_security_commitment(WETH, 15),
				get_security_commitment(TNT, 5),
			],
		));

		// Verify payment is transferred to MBSM
		let mbsm_address = Services::mbsm_address_of(&blueprint).unwrap();
		let mbsm_account_id = address_to_account_id(mbsm_address);
		assert_eq!(Assets::balance(USDC, mbsm_account_id), payment);
		assert_eq!(Assets::balance(USDC, Services::pallet_account()), 0);

		// Test Case 2: Multiple asset types with ERC20 payment
		let charlie_address = mock_address(CHARLIE);
		let charlie_evm_account_id = address_to_account_id(charlie_address);
		let before_erc20_balance = Services::query_erc20_balance_of(USDC_ERC20, charlie_address)
			.map(|(b, _)| b)
			.unwrap_or_default();

		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie_evm_account_id.clone()),
			Some(charlie_address),
			0,
			vec![],
			vec![bob.clone()],
			Default::default(),
			vec![
				get_security_requirement(USDC, &[10, 20]),
				get_security_requirement(WETH, &[15, 25]),
				get_security_requirement(TNT, &[5, 10]),
			],
			100,
			Asset::Erc20(USDC_ERC20),
			payment,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Verify ERC20 payment is held by pallet
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, Services::pallet_evm_account())
				.map(|(b, _)| b),
			U256::from(payment)
		);

		// Bob approves with security commitments for all assets
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			1,
			Percent::from_percent(10),
			vec![
				get_security_commitment(USDC, 10),
				get_security_commitment(WETH, 15),
				get_security_commitment(TNT, 5),
			],
		));

		// Verify ERC20 payment is transferred to MBSM
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, mbsm_address).map(|(b, _)| b),
			U256::from(payment)
		);
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, Services::pallet_evm_account())
				.map(|(b, _)| b),
			U256::from(0)
		);

		// Test Case 3: Multiple asset types with native currency payment
		let native_payment = 1000000000000000000u128; // 1 TNT
		let before_native_balance = Balances::free_balance(eve.clone());

		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![],
			vec![bob.clone()],
			Default::default(),
			vec![
				get_security_requirement(USDC, &[10, 20]),
				get_security_requirement(WETH, &[15, 25]),
				get_security_requirement(TNT, &[5, 10]),
			],
			100,
			Asset::Custom(0),
			native_payment,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Verify native payment is held by pallet
		assert_eq!(Balances::free_balance(Services::pallet_account()), native_payment);
		assert_eq!(Balances::free_balance(eve.clone()), before_native_balance - native_payment);

		// Bob approves with security commitments for all assets
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			2,
			Percent::from_percent(10),
			vec![
				get_security_commitment(USDC, 10),
				get_security_commitment(WETH, 15),
				get_security_commitment(TNT, 5),
			],
		));

		// Verify native payment is transferred to MBSM
		assert_eq!(Balances::free_balance(mbsm_account_id), native_payment);
		assert_eq!(Balances::free_balance(Services::pallet_account()), 0);
	});
}

#[test]
fn test_payment_zero_amount() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		// Create blueprint
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Register operator
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		let charlie = mock_pub_key(CHARLIE);

		// Test Case 1: Zero amount for Custom Asset
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(charlie.clone()),
				None,
				0,
				vec![],
				vec![bob.clone()],
				Default::default(),
				vec![get_security_requirement(USDC, &[10, 20])],
				100,
				Asset::Custom(USDC),
				0,
				MembershipModel::Fixed { min_operators: 1 },
			),
			Error::<Runtime>::InvalidAmount
		);

		// Test Case 2: Zero amount for ERC20 Token
		let charlie_address = mock_address(CHARLIE);
		let charlie_evm_account_id = address_to_account_id(charlie_address);
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(charlie_evm_account_id.clone()),
				Some(charlie_address),
				0,
				vec![],
				vec![bob.clone()],
				Default::default(),
				vec![get_security_requirement(USDC, &[10, 20])],
				100,
				Asset::Erc20(USDC_ERC20),
				0,
				MembershipModel::Fixed { min_operators: 1 },
			),
			Error::<Runtime>::InvalidAmount
		);

		// Test Case 3: Zero amount for Native Currency
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(charlie.clone()),
				None,
				0,
				vec![],
				vec![bob.clone()],
				Default::default(),
				vec![get_security_requirement(USDC, &[10, 20])],
				100,
				Asset::Custom(0),
				0,
				MembershipModel::Fixed { min_operators: 1 },
			),
			Error::<Runtime>::InvalidAmount
		);
	});
}

#[test]
fn test_payment_maximum_amount() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		// Create blueprint
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Register operator
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		let charlie = mock_pub_key(CHARLIE);

		// Test Case 1: Maximum amount for Custom Asset (more than balance)
		let max_custom_amount = Assets::balance(USDC, charlie.clone()) + 1;
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(charlie.clone()),
				None,
				0,
				vec![],
				vec![bob.clone()],
				Default::default(),
				vec![get_security_requirement(USDC, &[10, 20])],
				100,
				Asset::Custom(USDC),
				max_custom_amount,
				MembershipModel::Fixed { min_operators: 1 },
			),
			Error::<Runtime>::InsufficientBalance
		);

		// Test Case 2: Maximum amount for ERC20 Token (more than balance)
		let charlie_address = mock_address(CHARLIE);
		let charlie_evm_account_id = address_to_account_id(charlie_address);
		let max_erc20_amount = Services::query_erc20_balance_of(USDC_ERC20, charlie_address)
			.map(|(b, _)| b)
			.unwrap_or_default()
			.as_u128()
			+ 1;
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(charlie_evm_account_id.clone()),
				Some(charlie_address),
				0,
				vec![],
				vec![bob.clone()],
				Default::default(),
				vec![get_security_requirement(USDC, &[10, 20])],
				100,
				Asset::Erc20(USDC_ERC20),
				max_erc20_amount,
				MembershipModel::Fixed { min_operators: 1 },
			),
			Error::<Runtime>::InsufficientBalance
		);

		// Test Case 3: Maximum amount for Native Currency (more than balance)
		let max_native_amount = Balances::free_balance(charlie.clone()) + 1;
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(charlie.clone()),
				None,
				0,
				vec![],
				vec![bob.clone()],
				Default::default(),
				vec![get_security_requirement(USDC, &[10, 20])],
				100,
				Asset::Custom(0),
				max_native_amount,
				MembershipModel::Fixed { min_operators: 1 },
			),
			Error::<Runtime>::InsufficientBalance
		);
	});
}

#[test]
fn test_payment_invalid_asset_types() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		// Create blueprint
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Register operator
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		let charlie = mock_pub_key(CHARLIE);
		let payment = 5 * 10u128.pow(6); // 5 USDC

		// Test Case 1: Non-existent Custom Asset
		let non_existent_asset_id = 999999;
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(charlie.clone()),
				None,
				0,
				vec![],
				vec![bob.clone()],
				Default::default(),
				vec![get_security_requirement(USDC, &[10, 20])],
				100,
				Asset::Custom(non_existent_asset_id),
				payment,
				MembershipModel::Fixed { min_operators: 1 },
			),
			Error::<Runtime>::AssetNotFound
		);

		// Test Case 2: Non-existent ERC20 Token
		let charlie_address = mock_address(CHARLIE);
		let charlie_evm_account_id = address_to_account_id(charlie_address);
		let non_existent_erc20 = H160::from_low_u64_be(999999);
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(charlie_evm_account_id.clone()),
				Some(charlie_address),
				0,
				vec![],
				vec![bob.clone()],
				Default::default(),
				vec![get_security_requirement(USDC, &[10, 20])],
				100,
				Asset::Erc20(non_existent_erc20),
				payment,
				MembershipModel::Fixed { min_operators: 1 },
			),
			Error::<Runtime>::ERC20NotFound
		);

		// Test Case 3: Invalid ERC20 Token (not a contract)
		let invalid_erc20 = H160::from_low_u64_be(1); // Random address that's not a contract
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(charlie_evm_account_id.clone()),
				Some(charlie_address),
				0,
				vec![],
				vec![bob.clone()],
				Default::default(),
				vec![get_security_requirement(USDC, &[10, 20])],
				100,
				Asset::Erc20(invalid_erc20),
				payment,
				MembershipModel::Fixed { min_operators: 1 },
			),
			Error::<Runtime>::InvalidERC20Contract
		);
	});
}
