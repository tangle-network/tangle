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
use frame_support::{assert_err, assert_noop, assert_ok, traits::ConstU128};
use sp_core::{H160, U256};
use sp_runtime::{
	TokenError,
	traits::{BlakeTwo256, Hash},
};
use tangle_primitives::services::PricingModel;

#[test]
fn test_payment_refunds_on_failure() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		// Create blueprint
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

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

		// Bob cannot reject the same request again
		assert_noop!(
			Services::reject(RuntimeOrigin::signed(bob.clone()), 0),
			Error::<Runtime>::ApprovalNotRequested
		);

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
		assert_ok!(Services::reject(RuntimeOrigin::signed(bob.clone()), 1));

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
		let native_payment = 20000u128; // 0.00002 TNT
		let initial_balance = native_payment * 100;
		Balances::make_free_balance_be(&charlie, initial_balance);
		let before_native_balance = Balances::free_balance(charlie.clone());
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

		assert_eq!(Balances::free_balance(charlie.clone()), before_native_balance - native_payment);

		// Bob rejects the request
		assert_ok!(Services::reject(RuntimeOrigin::signed(bob.clone()), 2));

		// Verify native payment is refunded
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
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint.clone()));

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

		// Test Case 1: Custom Asset Payment (USDC)
		let payment = 5 * 10u128.pow(6); // 5 USDC
		let eve = mock_pub_key(EVE);
		mint_tokens(USDC, alice.clone(), eve.clone(), 1000000000000000000u128);
		let before_balance = Assets::balance(USDC, eve.clone());

		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![],
			vec![bob.clone(), charlie.clone()],
			Default::default(),
			vec![
				get_security_requirement(TNT, &[10, 30]), // Include native asset requirement
				get_security_requirement(USDC, &[10, 20])
			],
			100,
			Asset::Custom(USDC),
			payment,
			MembershipModel::Fixed { min_operators: 2 },
		));

		// Verify payment is held by pallet
		assert_eq!(Assets::balance(USDC, Services::pallet_account()), payment);
		assert_eq!(Assets::balance(USDC, eve.clone()), before_balance - payment);

		// Bob and Charlie approve the request with security commitments
		let security_commitments_bob =
			vec![get_security_commitment(TNT, 20), get_security_commitment(USDC, 10)];
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			security_commitments_bob
		));

		let security_commitments_charlie =
			vec![get_security_commitment(TNT, 25), get_security_commitment(USDC, 15)];
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			security_commitments_charlie
		));

		// Verify payment is transferred to MBSM
		let mbsm_address = Services::mbsm_address_of(&blueprint).unwrap();
		let mbsm_account_id = PalletEVMAddressMapping::into_account_id(mbsm_address);
		assert_eq!(Assets::balance(USDC, mbsm_account_id.clone()), payment);
		assert_eq!(Assets::balance(USDC, Services::pallet_account()), 0);

		// Test Case 2: ERC20 Token Payment
		let charlie_address = mock_address(CHARLIE);
		let charlie_evm_account_id = PalletEVMAddressMapping::into_account_id(charlie_address);

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

		// Bob approves with security commitments (USDC + TNT auto-added by system)
		let security_commitments2 =
			vec![get_security_commitment(USDC, 10), get_security_commitment(TNT, 10)];
		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 1, security_commitments2));

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
		let existential_deposit = <ConstU128<1> as sp_core::Get<u128>>::get();
		// Ensure enough balance for payment + existential deposit
		let required_balance = native_payment * 10;

		// Setup accounts with sufficient balances
		Balances::make_free_balance_be(&eve, required_balance);
		let pallet_account = Services::pallet_account();
		Balances::make_free_balance_be(&pallet_account, required_balance);

		let mbsm_address = Services::mbsm_address_of(&blueprint).unwrap();
		let mbsm_account_id = PalletEVMAddressMapping::into_account_id(mbsm_address);
		Balances::make_free_balance_be(&mbsm_account_id, required_balance);

		// Verify initial balances
		assert_eq!(
			Balances::free_balance(eve.clone()),
			required_balance,
			"Eve's balance not set correctly"
		);
		let initial_pallet_balance = Balances::free_balance(pallet_account.clone());
		let initial_mbsm_balance = Balances::free_balance(mbsm_account_id.clone());
		assert!(
			initial_pallet_balance >= existential_deposit,
			"Pallet account needs existential deposit"
		);
		assert!(
			initial_mbsm_balance >= existential_deposit,
			"MBSM account needs existential deposit"
		);

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

		// Bob approves with security commitments (USDC + TNT auto-added by system)
		let security_commitments3 =
			vec![get_security_commitment(USDC, 10), get_security_commitment(TNT, 10)];
		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 2, security_commitments3));

		assert_eq!(
			Balances::free_balance(eve.clone()),
			native_payment * 9,
			"Eve should retain rest of the balance"
		);
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
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint.clone()));

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

		// Test Case 1: Multiple asset security requirements
		let eve = mock_pub_key(EVE);
		let payment = 5 * 10u128.pow(6); // 5 USDC
		mint_tokens(USDC, alice.clone(), eve.clone(), payment * 10u128.pow(6));
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
			],
			100,
			Asset::Custom(USDC),
			payment,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Verify payment is held by pallet
		assert_eq!(Assets::balance(USDC, Services::pallet_account()), payment);
		assert_eq!(Assets::balance(USDC, eve.clone()), before_balance - payment);

		// Bob approves with security commitments for all assets (USDC, WETH + TNT auto-added)
		let security_commitments = vec![
			get_security_commitment(USDC, 10),
			get_security_commitment(WETH, 15),
			get_security_commitment(TNT, 10),
		];
		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 0, security_commitments));

		// Verify payment is transferred to MBSM
		let mbsm_address = Services::mbsm_address_of(&blueprint).unwrap();
		let mbsm_account_id = address_to_account_id(mbsm_address);
		assert_eq!(Assets::balance(USDC, mbsm_account_id.clone()), payment);
		assert_eq!(Assets::balance(USDC, Services::pallet_account()), 0);

		// Test Case 2: Multiple asset types with ERC20 payment
		let charlie_address = mock_address(CHARLIE);
		let charlie_evm_account_id = address_to_account_id(charlie_address);

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

		// Bob approves with security commitments for all assets (USDC, WETH + TNT auto-added)
		let security_commitments2 = vec![
			get_security_commitment(USDC, 10),
			get_security_commitment(WETH, 15),
			get_security_commitment(TNT, 10),
		];
		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 1, security_commitments2));

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
		let existential_deposit = <ConstU128<1> as sp_core::Get<u128>>::get();
		// Ensure enough balance for payment + existential deposit
		let required_balance = native_payment * 10;

		// Setup accounts with sufficient balances
		Balances::make_free_balance_be(&eve, required_balance);
		let pallet_account = Services::pallet_account();
		Balances::make_free_balance_be(&pallet_account, required_balance);

		let mbsm_address = Services::mbsm_address_of(&blueprint).unwrap();
		let mbsm_account_id = PalletEVMAddressMapping::into_account_id(mbsm_address);
		Balances::make_free_balance_be(&mbsm_account_id, required_balance);

		// Verify initial balances
		assert_eq!(
			Balances::free_balance(eve.clone()),
			required_balance,
			"Eve's balance not set correctly"
		);
		let initial_pallet_balance = Balances::free_balance(pallet_account.clone());
		let initial_mbsm_balance = Balances::free_balance(mbsm_account_id.clone());
		assert!(
			initial_pallet_balance >= existential_deposit,
			"Pallet account needs existential deposit"
		);
		assert!(
			initial_mbsm_balance >= existential_deposit,
			"MBSM account needs existential deposit"
		);

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
			],
			100,
			Asset::Custom(0),
			native_payment,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Bob approves with security commitments for all assets (USDC, WETH + TNT auto-added)
		let security_commitments3 = vec![
			get_security_commitment(USDC, 10),
			get_security_commitment(WETH, 15),
			get_security_commitment(TNT, 10),
		];
		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 2, security_commitments3));

		assert_eq!(
			Balances::free_balance(eve.clone()),
			native_payment * 9,
			"Eve should retain rest of the balance"
		);
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
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

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

		let charlie = mock_pub_key(CHARLIE);

		// Test Case 1: Zero amount for Custom Asset
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
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Test Case 2: Zero amount for ERC20 Token
		let charlie_address = mock_address(CHARLIE);
		let charlie_evm_account_id = address_to_account_id(charlie_address);
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
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Test Case 3: Zero amount for Native Currency
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
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));
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
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

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
			TokenError::FundsUnavailable,
		);

		// Test Case 2: Maximum amount for ERC20 Token (more than balance)
		let charlie_address = mock_address(CHARLIE);
		let charlie_evm_account_id = address_to_account_id(charlie_address);
		let max_erc20_amount = Services::query_erc20_balance_of(USDC_ERC20, charlie_address)
			.map(|(b, _)| b)
			.unwrap_or_default()
			.as_u128() +
			1;
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
			Error::<Runtime>::ERC20TransferFailed
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
			TokenError::FundsUnavailable,
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
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

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
			TokenError::UnknownAsset,
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
			Error::<Runtime>::ERC20TransferFailed
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
			Error::<Runtime>::ERC20TransferFailed
		);
	});
}

// Payment Processing Tests for different pricing models

#[test]
fn test_validate_payment_amount_pay_once() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		let blueprint = create_blueprint_with_pricing(PricingModel::PayOnce { amount: 1000u128 });

		// Valid payment amount (equal to required)
		assert_ok!(Services::validate_payment_amount(&blueprint, 1000));

		// Valid payment amount (more than required)
		assert_ok!(Services::validate_payment_amount(&blueprint, 1500));

		// Invalid payment amount (less than required)
		assert_err!(
			Services::validate_payment_amount(&blueprint, 500),
			Error::<Runtime>::InvalidRequestInput
		);
	});
}

#[test]
fn test_validate_payment_amount_subscription() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		let blueprint = create_blueprint_with_pricing(PricingModel::Subscription {
			rate_per_interval: 100u128,
			interval: 10u32,
			maybe_end: None,
		});

		// Valid payment amount (equal to rate)
		assert_ok!(Services::validate_payment_amount(&blueprint, 100));

		// Valid payment amount (more than rate)
		assert_ok!(Services::validate_payment_amount(&blueprint, 200));

		// Invalid payment amount (less than rate)
		assert_err!(
			Services::validate_payment_amount(&blueprint, 50),
			Error::<Runtime>::InvalidRequestInput
		);
	});
}

#[test]
fn test_validate_payment_amount_event_driven() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		let blueprint =
			create_blueprint_with_pricing(PricingModel::EventDriven { reward_per_event: 10u128 });

		// Any payment amount should be valid for event-driven services
		assert_ok!(Services::validate_payment_amount(&blueprint, 0));
		assert_ok!(Services::validate_payment_amount(&blueprint, 100));
		assert_ok!(Services::validate_payment_amount(&blueprint, 1000));
	});
}

// Helper functions for payment processing tests
fn create_blueprint_with_pricing(
	_pricing_model: PricingModel<u32, u128>,
) -> ServiceBlueprint<ConstraintsOf<Runtime>> {
	use frame_support::BoundedVec;
	use sp_core::bounded_vec;
	use tangle_primitives::services::{
		BlueprintServiceManager, JobDefinition, JobMetadata, MasterBlueprintServiceManagerRevision,
		MembershipModelType, ServiceBlueprint, ServiceMetadata,
	};

	ServiceBlueprint {
		metadata: ServiceMetadata::default(),
		jobs: bounded_vec![JobDefinition {
			metadata: JobMetadata::default(),
			params: BoundedVec::default(),
			result: BoundedVec::default(),
			pricing_model: _pricing_model.clone(),
		}],
		registration_params: BoundedVec::default(),
		request_params: BoundedVec::default(),
		manager: BlueprintServiceManager::default(),
		master_manager_revision: MasterBlueprintServiceManagerRevision::default(),
		sources: Default::default(),
		supported_membership_models: vec![MembershipModelType::Fixed].try_into().unwrap(),
	}
}
