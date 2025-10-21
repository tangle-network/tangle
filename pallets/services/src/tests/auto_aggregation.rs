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

//! Tests for auto-aggregation fix - verifying rewards aggregate per service_id

use super::*;
use crate::mock::MockRewardsManager;
use frame_support::assert_ok;

#[test]
fn rewards_aggregate_for_same_service() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

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

		mint_tokens(USDC, alice.clone(), eve.clone(), 1000 * 10u128.pow(6));

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

		// Make 10 job calls to the SAME service and process payments
		let payment_amount = 100; // Blueprint pricing is 100 native tokens
		for i in 0..10 {
			assert_ok!(Services::call(
				RuntimeOrigin::signed(eve.clone()),
				service_id,
				KEYGEN_JOB_ID,
				vec![Field::Uint8(i as u8)].try_into().unwrap()
			));

			// Process payment for each job call
			assert_ok!(Services::process_job_pay_once_payment(
				service_id,
				KEYGEN_JOB_ID,
				i, // call_id
				&eve,
				&eve,
				payment_amount,
			));
		}

		// Verify operator has ONLY ONE pending reward entry (aggregated)
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		assert_eq!(
			bob_rewards.len(),
			1,
			"WITHOUT aggregation, this would be 10 entries. WITH aggregation, it's 1."
		);

		let (reward_service_id, total_amount) = bob_rewards[0];
		assert_eq!(reward_service_id, service_id, "Reward should be for correct service");

		// Each job payment is 100 native tokens, operator gets 85%
		let payment_amount = 100;
		let operator_share_per_job = payment_amount * 85 / 100;
		let expected_total = operator_share_per_job * 10;

		assert_eq!(
			total_amount, expected_total,
			"Total should be sum of all 10 payments aggregated"
		);
	});
}

#[test]
fn aggregation_works_across_different_services() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let eve = mock_pub_key(EVE);

		// Create two blueprints
		let blueprint1 = cggmp21_blueprint();
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint1));

		let mut blueprint2 = cggmp21_blueprint();
		blueprint2.metadata.name = "Service2".try_into().unwrap();
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint2));

		// Register operator for both
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

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

		mint_tokens(USDC, alice.clone(), eve.clone(), 1000 * 10u128.pow(6));

		// Give eve native tokens to pay for services
		use frame_support::traits::Currency;
		let _ = Balances::make_free_balance_be(&eve, 20_000);

		// Request both services
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

		// Make 5 calls to service 0 with payments
		let payment_amount = 100; // Blueprint pricing is 100 native tokens
		for i in 0..5 {
			assert_ok!(Services::call(
				RuntimeOrigin::signed(eve.clone()),
				service_id_0,
				KEYGEN_JOB_ID,
				vec![Field::Uint8(i as u8)].try_into().unwrap()
			));

			assert_ok!(Services::process_job_pay_once_payment(
				service_id_0,
				KEYGEN_JOB_ID,
				i, // call_id
				&eve,
				&eve,
				payment_amount,
			));
		}

		// Make 3 calls to service 1 with payments
		for i in 0..3 {
			assert_ok!(Services::call(
				RuntimeOrigin::signed(eve.clone()),
				service_id_1,
				KEYGEN_JOB_ID,
				vec![Field::Uint8(i as u8)].try_into().unwrap()
			));

			assert_ok!(Services::process_job_pay_once_payment(
				service_id_1,
				KEYGEN_JOB_ID,
				5 + i, // call_id continues from first service
				&eve,
				&eve,
				payment_amount,
			));
		}

		// Verify operator has exactly 2 entries (one per service), NOT 8
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		assert_eq!(
			bob_rewards.len(),
			2,
			"WITHOUT aggregation: 8 entries. WITH aggregation: 2 entries (one per service)"
		);

		// Verify amounts for each service
		let payment_amount = 100;
		let operator_share_per_job = payment_amount * 85 / 100;

		let service_0_reward = bob_rewards
			.iter()
			.find(|(sid, _)| *sid == service_id_0)
			.map(|(_, amt)| *amt)
			.expect("Should have reward for service 0");

		let service_1_reward = bob_rewards
			.iter()
			.find(|(sid, _)| *sid == service_id_1)
			.map(|(_, amt)| *amt)
			.expect("Should have reward for service 1");

		assert_eq!(service_0_reward, operator_share_per_job * 5, "Service 0: 5 jobs aggregated");
		assert_eq!(service_1_reward, operator_share_per_job * 3, "Service 1: 3 jobs aggregated");
	});
}

#[test]
fn aggregation_prevents_bounded_vec_overflow() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

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

		mint_tokens(USDC, alice.clone(), eve.clone(), 10000 * 10u128.pow(6));

		// Give eve native tokens to pay for services (50 payments of 100)
		use frame_support::traits::Currency;
		let _ = Balances::make_free_balance_be(&eve, 50_000);

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

		// Make 50 job calls - WITHOUT aggregation, this would overflow BoundedVec
		// WITH aggregation, all 50 collapse into 1 entry
		let payment_amount = 100; // Blueprint pricing is 100 native tokens
		for i in 0..50 {
			assert_ok!(Services::call(
				RuntimeOrigin::signed(eve.clone()),
				service_id,
				KEYGEN_JOB_ID,
				vec![Field::Uint8((i % 256) as u8)].try_into().unwrap()
			));

			assert_ok!(Services::process_job_pay_once_payment(
				service_id,
				KEYGEN_JOB_ID,
				i, // call_id
				&eve,
				&eve,
				payment_amount,
			));
		}

		// Verify operator still has ONLY ONE entry after 50 calls
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		assert_eq!(
			bob_rewards.len(),
			1,
			"Aggregation prevents BoundedVec overflow: 50 calls -> 1 entry"
		);

		// Verify total is correct
		let payment_amount = 100;
		let operator_share_per_job = payment_amount * 85 / 100;
		let expected_total = operator_share_per_job * 50;

		let (_, total_amount) = bob_rewards[0];
		assert_eq!(total_amount, expected_total, "All 50 payments aggregated correctly");
	});
}

#[test]
fn aggregation_works_with_claim_in_between() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

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

		mint_tokens(USDC, alice.clone(), eve.clone(), 1000 * 10u128.pow(6));

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

		// Make 5 calls with payments
		let payment_amount = 100; // Blueprint pricing is 100 native tokens
		for i in 0..5 {
			assert_ok!(Services::call(
				RuntimeOrigin::signed(eve.clone()),
				service_id,
				KEYGEN_JOB_ID,
				vec![Field::Uint8(i as u8)].try_into().unwrap()
			));

			assert_ok!(Services::process_job_pay_once_payment(
				service_id,
				KEYGEN_JOB_ID,
				i, // call_id
				&eve,
				&eve,
				payment_amount,
			));
		}

		// Verify aggregation: 1 entry
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		assert_eq!(bob_rewards.len(), 1, "5 calls aggregated into 1 entry");

		// Claim rewards
		// Simulate claim
		MockRewardsManager::clear_pending_rewards(&bob);

		// After claim, pending should be cleared
		let bob_rewards_after_claim = MockRewardsManager::get_pending_rewards(&bob);
		assert_eq!(bob_rewards_after_claim.len(), 0, "Pending rewards cleared after claim");

		// Make 3 more calls to same service with payments
		for i in 5..8 {
			assert_ok!(Services::call(
				RuntimeOrigin::signed(eve.clone()),
				service_id,
				KEYGEN_JOB_ID,
				vec![Field::Uint8(i as u8)].try_into().unwrap()
			));

			assert_ok!(Services::process_job_pay_once_payment(
				service_id,
				KEYGEN_JOB_ID,
				i, // call_id
				&eve,
				&eve,
				payment_amount,
			));
		}

		// Should have 1 new entry for the 3 new calls
		let bob_rewards_final = MockRewardsManager::get_pending_rewards(&bob);
		assert_eq!(bob_rewards_final.len(), 1, "New calls after claim create new aggregated entry");

		let payment_amount = 100;
		let operator_share_per_job = payment_amount * 85 / 100;
		let expected_amount = operator_share_per_job * 3;

		let (_, amount) = bob_rewards_final[0];
		assert_eq!(amount, expected_amount, "New entry has correct aggregated amount");
	});
}
