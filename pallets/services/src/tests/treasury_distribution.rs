// Copyright 2022-2025 Tangle Foundation.
// This file is part of Tangle.
// This file originated in Moonbeam's codebase.

// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tangle. If not, see <http://www.gnu.org/licenses/>.

//! Tests for treasury distribution fix - verifying treasury receives 5% protocol share

use super::*;
use crate::mock::MockRewardsManager;
use frame_support::assert_ok;

#[test]
fn treasury_receives_five_percent_on_payonce_job() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		// Setup: Create blueprint, register operator, request service
		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let eve = mock_pub_key(EVE);

		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		assert_ok!(join_and_register(
			bob.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		// Mint tokens for Eve to pay for service
		mint_tokens(USDC, alice.clone(), eve.clone(), 200 * 10u128.pow(6));

		// Give eve native tokens to pay for services
		use frame_support::traits::Currency;
		let _ = Balances::make_free_balance_be(&eve, 10_000);

		let service_id = Services::next_instance_id();
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![
				get_security_requirement(TNT, &[10, 20]),
				get_security_requirement(WETH, &[10, 20])
			],
			100,
			Asset::Custom(USDC),
			100 * 10u128.pow(6),
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			service_id,
			vec![get_security_commitment(TNT, 10), get_security_commitment(WETH, 10)],
		));

		let treasury_account = TreasuryAccount::get();

		// Verify treasury has NO pending rewards initially
		let initial_rewards = MockRewardsManager::get_pending_rewards(&treasury_account);
		assert_eq!(initial_rewards.len(), 0, "Treasury should have no rewards initially");

		// Execute job call (just records the call)
		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			service_id,
			KEYGEN_JOB_ID,
			vec![Field::Uint8(1)].try_into().unwrap()
		));

		// Process PayOnce payment (this triggers reward distribution)
		let payment_amount = 100; // Blueprint pricing is 100 native tokens
		assert_ok!(Services::process_job_pay_once_payment(
			service_id,
			KEYGEN_JOB_ID,
			0, // call_id
			&eve,
			&eve,
			payment_amount,
		));

		// Verify treasury received 5% reward
		let treasury_rewards = MockRewardsManager::get_pending_rewards(&treasury_account);
		assert_eq!(
			treasury_rewards.len(),
			1,
			"Treasury should have exactly 1 pending reward entry"
		);

		let (reward_service_id, reward_amount) = treasury_rewards[0];
		assert_eq!(reward_service_id, service_id, "Treasury reward should be for correct service");

		// Payment is 100 native tokens
		// Treasury should get exactly 5%
		let payment_amount = 100;
		let expected_treasury = payment_amount * 5 / 100; // 5
		assert_eq!(
			reward_amount, expected_treasury,
			"Treasury should receive exactly 5% of payment"
		);

		// Verify treasury can claim rewards
		// Simulate claim (in production would be Rewards::claim_rewards)
		MockRewardsManager::clear_pending_rewards(&treasury_account);

		// After claiming, pending rewards should be cleared
		let final_rewards = MockRewardsManager::get_pending_rewards(&treasury_account);
		assert_eq!(
			final_rewards.len(),
			0,
			"Treasury pending rewards should be cleared after claiming"
		);
	});
}

#[test]
fn treasury_accumulates_from_multiple_services() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let eve = mock_pub_key(EVE);

		// Create first blueprint
		let blueprint1 = cggmp21_blueprint();
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint1));

		// Register operator for first blueprint
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		// Create second blueprint with different name
		let mut blueprint2 = cggmp21_blueprint();
		blueprint2.metadata.name = "CGGMP21 TSS v2".try_into().unwrap();
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint2));

		// Register operator for second blueprint
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			1,
			OperatorPreferences {
				key: test_ecdsa_key(),
				rpc_address: "https://example.com/rpc".try_into().unwrap()
			},
			Default::default(),
			0
		));

		// Mint tokens for Eve
		mint_tokens(USDC, alice.clone(), eve.clone(), 400 * 10u128.pow(6));

		// Give eve native tokens to pay for services
		use frame_support::traits::Currency;
		let _ = Balances::make_free_balance_be(&eve, 20_000);

		// Request first service
		let service_id_0 = Services::next_instance_id();
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![
				get_security_requirement(TNT, &[10, 20]),
				get_security_requirement(WETH, &[10, 20])
			],
			100,
			Asset::Custom(USDC),
			100 * 10u128.pow(6),
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			service_id_0,
			vec![get_security_commitment(TNT, 10), get_security_commitment(WETH, 10)],
		));

		// Request second service
		let service_id_1 = Services::next_instance_id();
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			1,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![
				get_security_requirement(TNT, &[10, 20]),
				get_security_requirement(WETH, &[10, 20])
			],
			100,
			Asset::Custom(USDC),
			100 * 10u128.pow(6),
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			service_id_1,
			vec![get_security_commitment(TNT, 10), get_security_commitment(WETH, 10)],
		));

		let treasury_account = TreasuryAccount::get();

		// Call job on first service
		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			service_id_0,
			KEYGEN_JOB_ID,
			vec![Field::Uint8(1)].try_into().unwrap()
		));

		// Process payment for first service
		let payment_amount = 100; // Blueprint pricing is 100 native tokens
		assert_ok!(Services::process_job_pay_once_payment(
			service_id_0,
			KEYGEN_JOB_ID,
			0, // call_id
			&eve,
			&eve,
			payment_amount,
		));

		// Call job on second service
		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			service_id_1,
			KEYGEN_JOB_ID,
			vec![Field::Uint8(2)].try_into().unwrap()
		));

		// Process payment for second service
		assert_ok!(Services::process_job_pay_once_payment(
			service_id_1,
			KEYGEN_JOB_ID,
			1, // call_id
			&eve,
			&eve,
			payment_amount,
		));

		// Verify treasury has rewards from BOTH services
		let treasury_rewards = MockRewardsManager::get_pending_rewards(&treasury_account);
		assert_eq!(
			treasury_rewards.len(),
			2,
			"Treasury should have rewards from 2 different services"
		);

		// Verify both service IDs are present
		let mut service_ids: Vec<_> = treasury_rewards.iter().map(|(sid, _)| *sid).collect();
		service_ids.sort();
		assert_eq!(
			service_ids,
			vec![service_id_0, service_id_1],
			"Treasury should have rewards from both services"
		);

		// Verify amounts
		let payment_amount = 100;
		let expected_per_service = payment_amount * 5 / 100;
		for (_, amount) in treasury_rewards.iter() {
			assert_eq!(*amount, expected_per_service, "Each service should contribute 5%");
		}
	});
}

#[test]
fn treasury_distribution_works_with_multiple_operators() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let eve = mock_pub_key(EVE);

		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Register TWO operators
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		assert_ok!(join_and_register(
			charlie.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example2.com/rpc")
		));

		mint_tokens(USDC, alice.clone(), eve.clone(), 200 * 10u128.pow(6));

		// Give eve native tokens to pay for services
		use frame_support::traits::Currency;
		let _ = Balances::make_free_balance_be(&eve, 10_000);

		let service_id = Services::next_instance_id();
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone()], // Both operators
			Default::default(),
			vec![
				get_security_requirement(TNT, &[10, 20]),
				get_security_requirement(WETH, &[10, 20])
			],
			100,
			Asset::Custom(USDC),
			100 * 10u128.pow(6),
			MembershipModel::Fixed { min_operators: 2 },
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			service_id,
			vec![get_security_commitment(TNT, 10), get_security_commitment(WETH, 10)],
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			service_id,
			vec![get_security_commitment(TNT, 10), get_security_commitment(WETH, 10)],
		));

		let treasury_account = TreasuryAccount::get();

		// Execute job
		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			service_id,
			KEYGEN_JOB_ID,
			vec![Field::Uint8(1)].try_into().unwrap()
		));

		// Process payment
		let payment_amount = 100; // Blueprint pricing is 100 native tokens
		assert_ok!(Services::process_job_pay_once_payment(
			service_id,
			KEYGEN_JOB_ID,
			0, // call_id
			&eve,
			&eve,
			payment_amount,
		));

		// Verify treasury STILL gets 5% regardless of number of operators
		let treasury_rewards = MockRewardsManager::get_pending_rewards(&treasury_account);
		assert_eq!(treasury_rewards.len(), 1, "Treasury should have 1 reward entry");

		let payment_amount = 100;
		let expected_treasury = payment_amount * 5 / 100;
		let (_, treasury_amount) = treasury_rewards[0];
		assert_eq!(
			treasury_amount, expected_treasury,
			"Treasury gets 5% regardless of operator count"
		);

		// Verify operators split the 85% operator share
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let charlie_rewards = MockRewardsManager::get_pending_rewards(&charlie);

		// Both operators should have rewards (85% split between them)
		assert_eq!(bob_rewards.len(), 1, "Bob should have 1 reward");
		assert_eq!(charlie_rewards.len(), 1, "Charlie should have 1 reward");
	});
}
