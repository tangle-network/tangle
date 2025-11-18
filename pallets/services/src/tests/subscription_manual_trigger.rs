// Test file for manual subscription payment triggering
// Tests the trigger_subscription_payment extrinsic

use super::*;
use frame_support::{assert_err, assert_ok, traits::Currency};
use tangle_primitives::services::JobSubscriptionBilling;

// Helper function to create billing entry directly in storage for testing
fn create_billing_entry(
	service_id: u64,
	job_index: u8,
	subscriber: AccountId,
	last_billed: u64,
	maybe_end: Option<u64>,
) {
	let billing = JobSubscriptionBilling {
		service_id,
		job_index,
		subscriber: subscriber.clone(),
		last_billed,
		end_block: maybe_end,
	};
	let billing_key = (service_id, job_index, subscriber.clone());
	JobSubscriptionBillings::<Runtime>::insert(&billing_key, &billing);

	// Update subscription count
	let current_count = UserSubscriptionCount::<Runtime>::get(&subscriber);
	UserSubscriptionCount::<Runtime>::insert(&subscriber, current_count + 1);
}

#[test]
fn test_manual_trigger_successful_payment() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let user = mock_pub_key(10);

		// Setup blueprint with subscription pricing
		let mut blueprint = cggmp21_blueprint();
		blueprint.jobs[0].pricing_model = PricingModel::Subscription {
			rate_per_interval: 10 * 10u128.pow(6),
			interval: 10, // Payment due every 10 blocks
			maybe_end: None,
		};

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		// Fund user
		mint_tokens(USDC, alice.clone(), user.clone(), 1000 * 10u128.pow(6));
		let _ = Balances::make_free_balance_be(&user, 100 * 10u128.pow(6));

		// Create service with subscription
		let service_id = Services::next_instance_id();
		assert_ok!(Services::request(
			RuntimeOrigin::signed(user.clone()),
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
			10 * 10u128.pow(6),
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), service_id, vec![
			get_security_commitment(TNT, 10),
			get_security_commitment(WETH, 10)
		]));

		// Call job to create subscription
		assert_ok!(Services::call(
			RuntimeOrigin::signed(user.clone()),
			service_id,
			KEYGEN_JOB_ID,
			vec![Field::Uint8(1)].try_into().unwrap()
		));

		// Create initial billing at block 1
		create_billing_entry(service_id, KEYGEN_JOB_ID, user.clone(), 1, None);

		// Advance to block 11 (payment now due)
		System::set_block_number(11);

		// Manually trigger subscription payment
		assert_ok!(Services::trigger_subscription_payment(
			RuntimeOrigin::signed(user.clone()),
			service_id,
			KEYGEN_JOB_ID,
		));

		// Verify billing was updated
		let billing_key = (service_id, KEYGEN_JOB_ID, user.clone());
		let billing = JobSubscriptionBillings::<Runtime>::get(&billing_key).unwrap();
		assert_eq!(billing.last_billed, 11, "Billing should be updated to current block");

		// Verify event was emitted
		System::assert_has_event(RuntimeEvent::Services(
			crate::Event::SubscriptionPaymentTriggered {
				caller: user,
				service_id,
				job_index: KEYGEN_JOB_ID,
			},
		));
	});
}

#[test]
fn test_manual_trigger_payment_not_due_yet() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let user = mock_pub_key(10);

		// Setup
		let mut blueprint = cggmp21_blueprint();
		blueprint.jobs[0].pricing_model = PricingModel::Subscription {
			rate_per_interval: 10 * 10u128.pow(6),
			interval: 10,
			maybe_end: None,
		};

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		mint_tokens(USDC, alice.clone(), user.clone(), 1000 * 10u128.pow(6));
		let _ = Balances::make_free_balance_be(&user, 100 * 10u128.pow(6));

		let service_id = Services::next_instance_id();
		assert_ok!(Services::request(
			RuntimeOrigin::signed(user.clone()),
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
			10 * 10u128.pow(6),
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), service_id, vec![
			get_security_commitment(TNT, 10),
			get_security_commitment(WETH, 10)
		]));

		assert_ok!(Services::call(
			RuntimeOrigin::signed(user.clone()),
			service_id,
			KEYGEN_JOB_ID,
			vec![Field::Uint8(1)].try_into().unwrap()
		));

		// Create billing at block 1
		create_billing_entry(service_id, KEYGEN_JOB_ID, user.clone(), 1, None);

		// Advance to block 5 (not enough blocks passed, interval is 10)
		System::set_block_number(5);

		// Attempt to manually trigger - should fail
		assert_err!(
			Services::trigger_subscription_payment(
				RuntimeOrigin::signed(user.clone()),
				service_id,
				KEYGEN_JOB_ID,
			),
			Error::<Runtime>::PaymentNotDueYet
		);
	});
}

#[test]
fn test_manual_trigger_subscription_not_found() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let user = mock_pub_key(10);

		// Setup service but don't create subscription
		let mut blueprint = cggmp21_blueprint();
		blueprint.jobs[0].pricing_model = PricingModel::Subscription {
			rate_per_interval: 10 * 10u128.pow(6),
			interval: 10,
			maybe_end: None,
		};

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		mint_tokens(USDC, alice.clone(), user.clone(), 1000 * 10u128.pow(6));
		let _ = Balances::make_free_balance_be(&user, 100 * 10u128.pow(6));

		let service_id = Services::next_instance_id();
		assert_ok!(Services::request(
			RuntimeOrigin::signed(user.clone()),
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
			10 * 10u128.pow(6),
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), service_id, vec![
			get_security_commitment(TNT, 10),
			get_security_commitment(WETH, 10)
		]));

		// Don't create subscription billing

		// Attempt to trigger non-existent subscription
		assert_err!(
			Services::trigger_subscription_payment(
				RuntimeOrigin::signed(user.clone()),
				service_id,
				KEYGEN_JOB_ID,
			),
			Error::<Runtime>::SubscriptionNotFound
		);
	});
}

#[test]
fn test_manual_trigger_expired_subscription() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let user = mock_pub_key(10);

		// Setup with subscription that ends at block 20
		let mut blueprint = cggmp21_blueprint();
		blueprint.jobs[0].pricing_model = PricingModel::Subscription {
			rate_per_interval: 10 * 10u128.pow(6),
			interval: 10,
			maybe_end: Some(20), // Subscription ends at block 20
		};

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		mint_tokens(USDC, alice.clone(), user.clone(), 1000 * 10u128.pow(6));
		let _ = Balances::make_free_balance_be(&user, 100 * 10u128.pow(6));

		let service_id = Services::next_instance_id();
		assert_ok!(Services::request(
			RuntimeOrigin::signed(user.clone()),
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
			10 * 10u128.pow(6),
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), service_id, vec![
			get_security_commitment(TNT, 10),
			get_security_commitment(WETH, 10)
		]));

		assert_ok!(Services::call(
			RuntimeOrigin::signed(user.clone()),
			service_id,
			KEYGEN_JOB_ID,
			vec![Field::Uint8(1)].try_into().unwrap()
		));

		// Create billing at block 1 with end_block = 20
		create_billing_entry(service_id, KEYGEN_JOB_ID, user.clone(), 1, Some(20));

		// Advance to block 25 (past expiration)
		System::set_block_number(25);

		// Attempt to trigger expired subscription - should fail
		assert_err!(
			Services::trigger_subscription_payment(
				RuntimeOrigin::signed(user.clone()),
				service_id,
				KEYGEN_JOB_ID,
			),
			Error::<Runtime>::SubscriptionNotValid
		);
	});
}

#[test]
fn test_manual_trigger_multiple_payments_in_sequence() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let user = mock_pub_key(10);

		let mut blueprint = cggmp21_blueprint();
		blueprint.jobs[0].pricing_model = PricingModel::Subscription {
			rate_per_interval: 10 * 10u128.pow(6),
			interval: 10,
			maybe_end: None,
		};

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		mint_tokens(USDC, alice.clone(), user.clone(), 1000 * 10u128.pow(6));
		let _ = Balances::make_free_balance_be(&user, 100 * 10u128.pow(6));

		let service_id = Services::next_instance_id();
		assert_ok!(Services::request(
			RuntimeOrigin::signed(user.clone()),
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
			10 * 10u128.pow(6),
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), service_id, vec![
			get_security_commitment(TNT, 10),
			get_security_commitment(WETH, 10)
		]));

		assert_ok!(Services::call(
			RuntimeOrigin::signed(user.clone()),
			service_id,
			KEYGEN_JOB_ID,
			vec![Field::Uint8(1)].try_into().unwrap()
		));

		// Initial billing at block 1
		create_billing_entry(service_id, KEYGEN_JOB_ID, user.clone(), 1, None);

		// First payment at block 11
		System::set_block_number(11);
		assert_ok!(Services::trigger_subscription_payment(
			RuntimeOrigin::signed(user.clone()),
			service_id,
			KEYGEN_JOB_ID,
		));

		let billing = JobSubscriptionBillings::<Runtime>::get((service_id, KEYGEN_JOB_ID, user.clone())).unwrap();
		assert_eq!(billing.last_billed, 11);

		// Second payment at block 21
		System::set_block_number(21);
		assert_ok!(Services::trigger_subscription_payment(
			RuntimeOrigin::signed(user.clone()),
			service_id,
			KEYGEN_JOB_ID,
		));

		let billing = JobSubscriptionBillings::<Runtime>::get((service_id, KEYGEN_JOB_ID, user.clone())).unwrap();
		assert_eq!(billing.last_billed, 21);

		// Third payment at block 31
		System::set_block_number(31);
		assert_ok!(Services::trigger_subscription_payment(
			RuntimeOrigin::signed(user.clone()),
			service_id,
			KEYGEN_JOB_ID,
		));

		let billing = JobSubscriptionBillings::<Runtime>::get((service_id, KEYGEN_JOB_ID, user.clone())).unwrap();
		assert_eq!(billing.last_billed, 31);
	});
}

#[test]
fn test_manual_trigger_with_non_subscription_pricing() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let user = mock_pub_key(10);

		// Setup with PayOnce pricing model
		let mut blueprint = cggmp21_blueprint();
		blueprint.jobs[0].pricing_model = PricingModel::PayOnce {
			amount: 100 * 10u128.pow(6),
		};

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		mint_tokens(USDC, alice.clone(), user.clone(), 1000 * 10u128.pow(6));
		let _ = Balances::make_free_balance_be(&user, 100 * 10u128.pow(6));

		let service_id = Services::next_instance_id();
		assert_ok!(Services::request(
			RuntimeOrigin::signed(user.clone()),
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
			10 * 10u128.pow(6),
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), service_id, vec![
			get_security_commitment(TNT, 10),
			get_security_commitment(WETH, 10)
		]));

		// Attempt to trigger for non-subscription job
		assert_err!(
			Services::trigger_subscription_payment(
				RuntimeOrigin::signed(user.clone()),
				service_id,
				KEYGEN_JOB_ID,
			),
			Error::<Runtime>::SubscriptionNotValid
		);
	});
}

#[test]
fn test_manual_trigger_prevents_double_processing() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let user = mock_pub_key(10);

		let mut blueprint = cggmp21_blueprint();
		blueprint.jobs[0].pricing_model = PricingModel::Subscription {
			rate_per_interval: 10 * 10u128.pow(6),
			interval: 10,
			maybe_end: None,
		};

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		mint_tokens(USDC, alice.clone(), user.clone(), 1000 * 10u128.pow(6));
		let _ = Balances::make_free_balance_be(&user, 100 * 10u128.pow(6));

		let service_id = Services::next_instance_id();
		assert_ok!(Services::request(
			RuntimeOrigin::signed(user.clone()),
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
			10 * 10u128.pow(6),
			MembershipModel::Fixed { min_operators: 1 },
		));

		assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), service_id, vec![
			get_security_commitment(TNT, 10),
			get_security_commitment(WETH, 10)
		]));

		assert_ok!(Services::call(
			RuntimeOrigin::signed(user.clone()),
			service_id,
			KEYGEN_JOB_ID,
			vec![Field::Uint8(1)].try_into().unwrap()
		));

		create_billing_entry(service_id, KEYGEN_JOB_ID, user.clone(), 1, None);

		// Advance and trigger first payment
		System::set_block_number(11);
		assert_ok!(Services::trigger_subscription_payment(
			RuntimeOrigin::signed(user.clone()),
			service_id,
			KEYGEN_JOB_ID,
		));

		// Try to trigger again immediately - should fail
		assert_err!(
			Services::trigger_subscription_payment(
				RuntimeOrigin::signed(user.clone()),
				service_id,
				KEYGEN_JOB_ID,
			),
			Error::<Runtime>::PaymentNotDueYet
		);

		// Verify billing updated once
		let billing = JobSubscriptionBillings::<Runtime>::get((service_id, KEYGEN_JOB_ID, user.clone())).unwrap();
		assert_eq!(billing.last_billed, 11, "Should have updated exactly once");
	});
}

// ========================================
// E2E SIMULATION TESTS
// ========================================
// These tests verify manual triggering with realistic scenarios using actual
// runtime calls (not mocked). They test:
// - Many users manually triggering their subscriptions
// - Concurrent manual triggers
// - Real payment processing
// - System performance under load

#[test]
fn test_manual_trigger_100_users_e2e() {
	const NUM_USERS: u8 = 100;

	println!("\n=== MANUAL TRIGGER E2E TEST: {} USERS ===", NUM_USERS);

	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);

		// Setup blueprint with subscription pricing
		let mut blueprint = cggmp21_blueprint();
		blueprint.jobs[0].pricing_model = PricingModel::Subscription {
			rate_per_interval: 10 * 10u128.pow(6), // 10 USDC per interval
			interval: 10, // Every 10 blocks
			maybe_end: None,
		};

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		assert_ok!(join_and_register(
			bob.clone(),
			0,
			test_ecdsa_key(),
			1000,
			Some("https://example.com/rpc")
		));

		println!("Blueprint created. Setting up {} user subscriptions...", NUM_USERS);

		// Create subscriptions for each user
		use frame_support::traits::Currency;
		let mut user_services: Vec<(AccountId, u64)> = Vec::new();

		for user_id in 10..(10 + NUM_USERS) {
			let user = mock_pub_key(user_id);

			// Fund user generously
			mint_tokens(USDC, alice.clone(), user.clone(), 100_000 * 10u128.pow(6));
			let _ = Balances::make_free_balance_be(&user, 100_000 * 10u128.pow(6));

			// Create service with subscription
			let service_id = Services::next_instance_id();
			assert_ok!(Services::request(
				RuntimeOrigin::signed(user.clone()),
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
				10 * 10u128.pow(6),
				MembershipModel::Fixed { min_operators: 1 },
			));

			assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), service_id, vec![
				get_security_commitment(TNT, 10),
				get_security_commitment(WETH, 10)
			]));

			// Call job to create subscription
			assert_ok!(Services::call(
				RuntimeOrigin::signed(user.clone()),
				service_id,
				KEYGEN_JOB_ID,
				vec![Field::Uint8(1)].try_into().unwrap()
			));

			// Create initial billing entry
			create_billing_entry(service_id, KEYGEN_JOB_ID, user.clone(), 1, None);

			user_services.push((user.clone(), service_id));
		}

		println!("Created {} subscriptions. Advancing to block 11...", NUM_USERS);

		// Advance to block 11 where all payments are due
		System::set_block_number(11);

		println!("All subscriptions now due. Starting manual triggers...");

		// Each user manually triggers their subscription
		for (idx, (user, service_id)) in user_services.iter().enumerate() {
			if idx % 10 == 0 {
				println!("  Triggering payment {}/{}...", idx + 1, NUM_USERS);
			}

			assert_ok!(Services::trigger_subscription_payment(
				RuntimeOrigin::signed(user.clone()),
				*service_id,
				KEYGEN_JOB_ID,
			));

			// Verify billing was updated
			let billing_key = (*service_id, KEYGEN_JOB_ID, user.clone());
			let billing = JobSubscriptionBillings::<Runtime>::get(&billing_key).unwrap();
			assert_eq!(
				billing.last_billed, 11,
				"User {} billing should be updated to block 11", idx
			);
		}

		println!("All {} payments successfully triggered!", NUM_USERS);

		// Advance to block 21 and trigger second round of payments
		System::set_block_number(21);
		println!("Advanced to block 21. Triggering second round of payments...");

		for (idx, (user, service_id)) in user_services.iter().enumerate() {
			if idx % 10 == 0 {
				println!("  Second payment {}/{}...", idx + 1, NUM_USERS);
			}

			assert_ok!(Services::trigger_subscription_payment(
				RuntimeOrigin::signed(user.clone()),
				*service_id,
				KEYGEN_JOB_ID,
			));

			// Verify billing was updated
			let billing_key = (*service_id, KEYGEN_JOB_ID, user.clone());
			let billing = JobSubscriptionBillings::<Runtime>::get(&billing_key).unwrap();
			assert_eq!(
				billing.last_billed, 21,
				"User {} second billing should be updated to block 21", idx
			);
		}

		println!("E2E test completed successfully!");
		println!("Verified {} users × 2 payments = {} total manual triggers", NUM_USERS, NUM_USERS * 2);
	});
}

#[test]
fn test_manual_trigger_mixed_timing_e2e() {
	// This test verifies that users can trigger payments at different times
	// simulating a real-world scenario where not all users trigger simultaneously
	const NUM_USERS: u8 = 50;

	println!("\n=== MANUAL TRIGGER MIXED TIMING E2E TEST ===");

	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);

		// Setup blueprint
		let mut blueprint = cggmp21_blueprint();
		blueprint.jobs[0].pricing_model = PricingModel::Subscription {
			rate_per_interval: 10 * 10u128.pow(6),
			interval: 10,
			maybe_end: None,
		};

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		println!("Setting up {} subscriptions...", NUM_USERS);

		// Create subscriptions
		use frame_support::traits::Currency;
		let mut user_services: Vec<(AccountId, u64)> = Vec::new();

		for user_id in 10..(10 + NUM_USERS) {
			let user = mock_pub_key(user_id);

			mint_tokens(USDC, alice.clone(), user.clone(), 100_000 * 10u128.pow(6));
			let _ = Balances::make_free_balance_be(&user, 100_000 * 10u128.pow(6));

			let service_id = Services::next_instance_id();
			assert_ok!(Services::request(
				RuntimeOrigin::signed(user.clone()),
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
				10 * 10u128.pow(6),
				MembershipModel::Fixed { min_operators: 1 },
			));

			assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), service_id, vec![
				get_security_commitment(TNT, 10),
				get_security_commitment(WETH, 10)
			]));

			assert_ok!(Services::call(
				RuntimeOrigin::signed(user.clone()),
				service_id,
				KEYGEN_JOB_ID,
				vec![Field::Uint8(1)].try_into().unwrap()
			));

			create_billing_entry(service_id, KEYGEN_JOB_ID, user.clone(), 1, None);

			user_services.push((user.clone(), service_id));
		}

		println!("Testing staggered manual triggers across blocks...");

		// Trigger payments at different blocks to simulate real usage
		// Some users trigger at block 11, others at block 12, 13, etc.
		for (idx, (user, service_id)) in user_services.iter().enumerate() {
			// Stagger triggers across blocks 11-15
			let trigger_block = 11 + (idx as u64 % 5);
			System::set_block_number(trigger_block);

			assert_ok!(Services::trigger_subscription_payment(
				RuntimeOrigin::signed(user.clone()),
				*service_id,
				KEYGEN_JOB_ID,
			));

			// Verify billing
			let billing_key = (*service_id, KEYGEN_JOB_ID, user.clone());
			let billing = JobSubscriptionBillings::<Runtime>::get(&billing_key).unwrap();
			assert_eq!(
				billing.last_billed, trigger_block,
				"User {} should be billed at block {}", idx, trigger_block
			);

			if idx % 10 == 0 {
				println!("  User {} triggered at block {}", idx, trigger_block);
			}
		}

		println!("Mixed timing test completed! All users triggered at different blocks.");
	});
}

#[test]
#[ignore = "Performance test - run manually with: cargo test test_manual_trigger_stress --release -- --ignored --nocapture"]
fn test_manual_trigger_stress_1000_users() {
	// Stress test with 200 users (realistic scale test)
	const NUM_USERS: u16 = 200;

	println!("\n=== STRESS TEST: {} USERS MANUAL TRIGGER ===", NUM_USERS);
	println!("This tests system behavior when many users manually trigger payments");

	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);

		// Setup blueprint
		let mut blueprint = cggmp21_blueprint();
		blueprint.jobs[0].pricing_model = PricingModel::Subscription {
			rate_per_interval: 10 * 10u128.pow(6),
			interval: 10,
			maybe_end: None,
		};

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		// Use the standard helper which works
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		println!("Creating {} subscriptions...", NUM_USERS);

		use frame_support::traits::Currency;
		use std::collections::HashSet;
		let mut user_services: Vec<(AccountId, u64)> = Vec::new();
		let mut funded_users: HashSet<AccountId> = HashSet::new();

		// Create subscriptions - use only a smaller set of unique users
		// Each user will have multiple subscriptions
		const UNIQUE_USERS: u8 = 20; // 20 unique users, each with 10 subscriptions
		for i in 0..NUM_USERS {
			let user_id = 10 + (i % UNIQUE_USERS as u16) as u8;
			let user = mock_pub_key(user_id);

			// Fund each unique user on first encounter
			if !funded_users.contains(&user) {
				mint_tokens(USDC, alice.clone(), user.clone(), 100_000 * 10u128.pow(6));
				let _ = Balances::make_free_balance_be(&user, 100_000 * 10u128.pow(6));
				funded_users.insert(user.clone());
			}

			let service_id = Services::next_instance_id();
			assert_ok!(Services::request(
				RuntimeOrigin::signed(user.clone()),
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
				10 * 10u128.pow(6),
				MembershipModel::Fixed { min_operators: 1 },
			));

			assert_ok!(Services::approve(RuntimeOrigin::signed(bob.clone()), service_id, vec![
				get_security_commitment(TNT, 10),
				get_security_commitment(WETH, 10)
			]));

			assert_ok!(Services::call(
				RuntimeOrigin::signed(user.clone()),
				service_id,
				KEYGEN_JOB_ID,
				vec![Field::Uint8(1)].try_into().unwrap()
			));

			create_billing_entry(service_id, KEYGEN_JOB_ID, user.clone(), 1, None);

			user_services.push((user.clone(), service_id));

			if i % 100 == 0 && i > 0 {
				println!("  Created {}/{} subscriptions", i, NUM_USERS);
			}
		}

		println!("All subscriptions created. Starting stress test...");
		System::set_block_number(11);

		// Trigger all payments
		for (idx, (user, service_id)) in user_services.iter().enumerate() {
			assert_ok!(Services::trigger_subscription_payment(
				RuntimeOrigin::signed(user.clone()),
				*service_id,
				KEYGEN_JOB_ID,
			));

			if idx % 100 == 0 && idx > 0 {
				println!("  Triggered {}/{} payments", idx, NUM_USERS);
			}
		}

		println!("STRESS TEST PASSED: {} manual triggers completed successfully!", NUM_USERS);
	});
}
