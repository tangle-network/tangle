// End-to-end payment integration tests
// Verifies the full customer payment → reward distribution flow

use super::*;
use crate::mock::MockRewardsManager;
use frame_support::{assert_ok, assert_err};
use sp_runtime::Perbill;
use tangle_primitives::{services::{Asset, PricingModel}, traits::RewardRecorder};

/// Helper to advance blocks and process subscription payments
fn advance_blocks(n: u64) {
	for _ in 0..n {
		let current = System::block_number();
		System::set_block_number(current + 1);
		// Manually call on_initialize to process subscription payments
		Services::on_initialize(System::block_number());
	}
}

#[test]
fn test_subscription_payment_e2e_flow() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
		MockRewardsManager::clear_all();
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE); // Blueprint developer
		let bob = mock_pub_key(BOB);     // Operator
		let charlie = mock_pub_key(CHARLIE); // Customer

		// Give customer extra funds for multiple subscription payments
		Balances::make_free_balance_be(&charlie, 100_000);

		// Ensure rewards pallet account exists
		let rewards_account = MockRewardsManager::account_id();
		Balances::make_free_balance_be(&rewards_account, 1000);

		// Setup: Create blueprint with subscription pricing
		let subscription_rate = 1_000u128; // 1,000 tokens per interval
		let interval_blueprint = 10u32; // Every 10 blocks (for blueprint)
		let interval_runtime = 10u64; // Every 10 blocks (for runtime calls)

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint_with_pricing(
			RuntimeOrigin::signed(alice.clone()),
			blueprint,
			PricingModel::Subscription {
				rate_per_interval: subscription_rate,
				interval: interval_blueprint,
				maybe_end: Some(50u32), // End after block 50 (blueprint uses u32)
			}
		));

		// Register operator
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		// Customer requests service
		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(TNT, &[50, 100])],
			100, // TTL (independent from subscription interval!)
			Asset::Custom(0),
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Operator approves with 50% exposure commitment
		let security_commitments = vec![get_security_commitment(TNT, 50)];
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			security_commitments
		));

		// Service is now active at block 1
		// Customer initiates subscription job
		let service_id = 0;
		let job_index = 0;

		// Manually trigger subscription payment (in production this happens automatically)
		assert_ok!(Services::process_job_subscription_payment(
			service_id,
			job_index,
			0, // call_id
			&charlie,
			&charlie,
			subscription_rate,
			interval_runtime,
			Some(50u64), // Runtime uses u64 for block numbers
			1, // current_block
		));

		// Verify first payment was distributed
		// Operator share: 85% of 1,000 = 850 tokens
		// Developer share: 10% of 1,000 = 100 tokens
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 850, "First payment: Bob should receive 850 tokens (85%)");

		let alice_rewards = MockRewardsManager::get_pending_rewards(&alice);
		let alice_total: u128 = alice_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(alice_total, 100, "First payment: Alice (developer) should receive 100 tokens (10%)");

		// Advance blocks to trigger next payment
		advance_blocks(10);

		// Manually trigger second subscription payment
		assert_ok!(Services::process_job_subscription_payment(
			service_id,
			job_index,
			0,
			&charlie,
			&charlie,
			subscription_rate,
			interval_runtime,
			Some(50u64),
			11, // current_block
		));

		// Verify second payment was distributed
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 1_700, "Second payment: Bob should have 1,700 tokens (2 * 850)");

		let alice_rewards = MockRewardsManager::get_pending_rewards(&alice);
		let alice_total: u128 = alice_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(alice_total, 200, "Second payment: Alice should have 200 tokens (2 * 100)");

		// Advance blocks to trigger third payment
		advance_blocks(10);

		assert_ok!(Services::process_job_subscription_payment(
			service_id,
			job_index,
			0,
			&charlie,
			&charlie,
			subscription_rate,
			interval_runtime,
			Some(50u64),
			21,
		));

		// Verify third payment
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 2_550, "Third payment: Bob should have 2,550 tokens (3 * 850)");

		// Advance blocks past subscription end
		advance_blocks(30); // Now at block 51, past end block 50

		// Try to process payment after subscription end - should not add new rewards
		assert_ok!(Services::process_job_subscription_payment(
			service_id,
			job_index,
			0,
			&charlie,
			&charlie,
			subscription_rate,
			interval_runtime,
			Some(50u64),
			51,
		));

		// Rewards should not have increased (subscription ended)
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 2_550, "After end: Bob's rewards should not increase");
	});
}

#[test]
fn test_subscription_payment_multiple_operators() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		MockRewardsManager::clear_all();
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE); // Developer
		let bob = mock_pub_key(BOB);     // Operator 1 (40% exposure)
		let charlie = mock_pub_key(CHARLIE); // Operator 2 (60% exposure)
		let dave = mock_pub_key(DAVE);   // Customer

		// Give customer extra funds for subscription payment
		Balances::make_free_balance_be(&dave, 100_000);

		// Ensure rewards pallet account exists
		let rewards_account = MockRewardsManager::account_id();
		Balances::make_free_balance_be(&rewards_account, 1000);

		let subscription_rate = 10_000u128;
		let interval_blueprint = 5u32;
		let interval_runtime = 5u64;

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint_with_pricing(
			RuntimeOrigin::signed(alice.clone()),
			blueprint,
			PricingModel::Subscription {
				rate_per_interval: subscription_rate,
				interval: interval_blueprint,
				maybe_end: None::<u32>, // No end (blueprint uses u32)
			}
		));

		// Register operators
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));
		assert_ok!(join_and_register(charlie.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		// Customer requests service
		assert_ok!(Services::request(
			RuntimeOrigin::signed(dave.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone()],
			Default::default(),
			vec![
				get_security_requirement(TNT, &[40, 60]),
				get_security_requirement(WETH, &[40, 60])
			],
			100,
			Asset::Custom(0),
			0,
			MembershipModel::Fixed { min_operators: 2 },
		));

		// Operators approve with different exposure levels
		// Bob: 40% TNT + 40% WETH = 80 total exposure
		let bob_commitments = vec![
			get_security_commitment(TNT, 40),
			get_security_commitment(WETH, 40)
		];
		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 0, bob_commitments));

		// Charlie: 60% TNT + 60% WETH = 120 total exposure
		let charlie_commitments = vec![
			get_security_commitment(TNT, 60),
			get_security_commitment(WETH, 60)
		];
		assert_ok!(Services::approve(RuntimeOrigin::signed(charlie.clone()), 0, charlie_commitments));

		// Process subscription payment
		assert_ok!(Services::process_job_subscription_payment(
			0, // service_id
			0, // job_index
			0, // call_id
			&dave,
			&dave,
			subscription_rate,
			interval_runtime,
			None::<u64>, // Runtime uses u64
			1, // current_block
		));

		// Verify distribution:
		// Total exposure: 80 + 120 = 200
		// Operator pool: 85% * 10,000 = 8,500
		// Bob: (80 / 200) * 8,500 = 3,400
		// Charlie: (120 / 200) * 8,500 = 5,100
		// Developer: 10% * 10,000 = 1,000

		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 3_400, "Bob (40% exposure) should receive 3,400 tokens");

		let charlie_rewards = MockRewardsManager::get_pending_rewards(&charlie);
		let charlie_total: u128 = charlie_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(charlie_total, 5_100, "Charlie (60% exposure) should receive 5,100 tokens");

		let alice_rewards = MockRewardsManager::get_pending_rewards(&alice);
		let alice_total: u128 = alice_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(alice_total, 1_000, "Alice (developer) should receive 1,000 tokens");

		// Verify Charlie gets 1.5x Bob's reward (120/80 = 1.5)
		assert_eq!(charlie_total * 2, bob_total * 3, "Charlie should get 1.5x Bob's reward");

		// Verify total distribution is 95% (85% operators + 10% developer)
		let total_distributed = bob_total + charlie_total + alice_total;
		let expected = Perbill::from_percent(95) * subscription_rate;
		assert_eq!(total_distributed, expected, "Total should be 95% of payment");
	});
}

#[test]
fn test_pay_once_payment_distribution() {
	// This test shows that the distribution logic works, but integration with call() is missing
	new_test_ext(vec![ALICE, BOB, CHARLIE]).execute_with(|| {
		MockRewardsManager::clear_all();
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE); // Developer
		let bob = mock_pub_key(BOB);     // Operator
		let charlie = mock_pub_key(CHARLIE); // Customer

		let payment_amount = 5_000u128;

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint_with_pricing(
			RuntimeOrigin::signed(alice.clone()),
			blueprint,
			PricingModel::PayOnce { amount: payment_amount }
		));

		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(TNT, &[75, 100])],
			100,
			Asset::Custom(0),
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));

		let security_commitments = vec![get_security_commitment(TNT, 75)];
		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), 0, security_commitments));

		// Manually call payment processing (in production, this should be triggered by call() extrinsic)
		assert_ok!(Services::process_job_pay_once_payment(
			0, // service_id
			0, // job_index
			0, // call_id
			&charlie,
			&charlie,
			payment_amount,
		));

		// Verify distribution
		// Operator: 85% * 5,000 = 4,250
		// Developer: 10% * 5,000 = 500
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 4_250, "Bob should receive 4,250 tokens (85%)");

		let alice_rewards = MockRewardsManager::get_pending_rewards(&alice);
		let alice_total: u128 = alice_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(alice_total, 500, "Alice should receive 500 tokens (10%)");

		// Verify payment is recorded and cannot be processed twice
		assert!(JobPayments::<Runtime>::contains_key(0, 0));
		assert_err!(
			Services::process_job_pay_once_payment(0, 0, 0, &charlie, &charlie, payment_amount),
			Error::<Runtime>::PaymentAlreadyProcessed
		);
	});
}

#[test]
fn test_event_driven_payment_distribution() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE]).execute_with(|| {
		MockRewardsManager::clear_all();
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE); // Developer
		let bob = mock_pub_key(BOB);     // Operator 1
		let charlie = mock_pub_key(CHARLIE); // Operator 2
		let dave = mock_pub_key(DAVE);   // Customer

		let reward_per_event = 100u128;
		let event_count = 50u32; // 50 events occurred

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint_with_pricing(
			RuntimeOrigin::signed(alice.clone()),
			blueprint,
			PricingModel::EventDriven { reward_per_event }
		));

		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));
		assert_ok!(join_and_register(charlie.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		assert_ok!(Services::request(
			RuntimeOrigin::signed(dave.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone()],
			Default::default(),
			vec![get_security_requirement(TNT, &[30, 100])],
			100,
			Asset::Custom(0),
			0,
			MembershipModel::Fixed { min_operators: 2 },
		));

		// Bob: 30% exposure, Charlie: 70% exposure
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			vec![get_security_commitment(TNT, 30)]
		));
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			vec![get_security_commitment(TNT, 70)]
		));

		// Process event-driven payment for 50 events
		// Total: 50 * 100 = 5,000 tokens
		assert_ok!(Services::process_job_event_driven_payment(
			0, // service_id
			0, // job_index
			0, // call_id
			&dave,
			&dave,
			reward_per_event,
			event_count,
		));

		// Verify distribution:
		// Total: 5,000 tokens
		// Operator pool: 85% * 5,000 = 4,250
		// Bob: (30 / 100) * 4,250 = 1,275
		// Charlie: (70 / 100) * 4,250 = 2,975
		// Developer: 10% * 5,000 = 500

		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(bob_total, 1_275, "Bob (30% exposure) should receive 1,275 tokens");

		let charlie_rewards = MockRewardsManager::get_pending_rewards(&charlie);
		let charlie_total: u128 = charlie_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(charlie_total, 2_975, "Charlie (70% exposure) should receive 2,975 tokens");

		let alice_rewards = MockRewardsManager::get_pending_rewards(&alice);
		let alice_total: u128 = alice_rewards.iter().map(|(_, amt)| *amt).sum();
		assert_eq!(alice_total, 500, "Alice (developer) should receive 500 tokens");
	});
}

#[test]
fn test_payment_timing_vs_service_ttl() {
	// This test demonstrates that subscription payment intervals are INDEPENDENT from service TTL
	new_test_ext(vec![ALICE, BOB, CHARLIE]).execute_with(|| {
		MockRewardsManager::clear_all();
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);

		// Give customer extra funds for multiple subscription payments
		Balances::make_free_balance_be(&charlie, 100_000);

		// Ensure rewards pallet account exists
		let rewards_account = MockRewardsManager::account_id();
		Balances::make_free_balance_be(&rewards_account, 1000);

		let subscription_rate = 1_000u128;
		let interval_blueprint = 20u32; // Payment every 20 blocks (for blueprint)
		let interval_runtime = 20u64; // Payment every 20 blocks (for runtime)
		let service_ttl = 100u64; // Service lives 100 blocks

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		let blueprint = cggmp21_blueprint();
		assert_ok!(create_test_blueprint_with_pricing(
			RuntimeOrigin::signed(alice.clone()),
			blueprint,
			PricingModel::Subscription {
				rate_per_interval: subscription_rate,
				interval: interval_blueprint,
				maybe_end: Some(80u32), // Subscription ends before TTL (blueprint uses u32)
			}
		));

		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie.clone()),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(TNT, &[100, 100])],
			service_ttl, // TTL is 100 blocks
			Asset::Custom(0),
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			vec![get_security_commitment(TNT, 100)]
		));

		// Payment 1: Block 1
		assert_ok!(Services::process_job_subscription_payment(0, 0, 0, &charlie, &charlie, subscription_rate, interval_runtime, Some(80u64), 1));

		// Payment 2: Block 21 (after 20 blocks)
		advance_blocks(20);
		assert_ok!(Services::process_job_subscription_payment(0, 0, 0, &charlie, &charlie, subscription_rate, interval_runtime, Some(80u64), 21));

		// Payment 3: Block 41
		advance_blocks(20);
		assert_ok!(Services::process_job_subscription_payment(0, 0, 0, &charlie, &charlie, subscription_rate, interval_runtime, Some(80u64), 41));

		// Payment 4: Block 61
		advance_blocks(20);
		assert_ok!(Services::process_job_subscription_payment(0, 0, 0, &charlie, &charlie, subscription_rate, interval_runtime, Some(80u64), 61));

		// Payment 5 attempt at Block 81: Should not process (past end_block 80)
		advance_blocks(20);
		assert_ok!(Services::process_job_subscription_payment(0, 0, 0, &charlie, &charlie, subscription_rate, interval_runtime, Some(80u64), 81));

		// Verify: Only 4 payments distributed (blocks 1, 21, 41, 61)
		// Note: Service TTL (100) is different from subscription end (80)
		let bob_rewards = MockRewardsManager::get_pending_rewards(&bob);
		let bob_total: u128 = bob_rewards.iter().map(|(_, amt)| *amt).sum();
		let expected_payments = 4;
		let expected_total = expected_payments * 850; // 4 payments * 850 tokens (85% of 1,000)
		assert_eq!(bob_total, expected_total, "Bob should have received 4 payments before subscription ended");
	});
}
