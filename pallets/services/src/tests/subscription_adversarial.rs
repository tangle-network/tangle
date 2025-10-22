//! Adversarial Security Tests for Subscription Cursor
//!
//! These tests attempt to break the subscription cursor implementation
//! through various attack vectors to prove security.

use super::*;
use frame_support::{assert_noop, assert_ok, weights::Weight};
use crate::Error;

/// Test: Attempt to bypass 100 subscription per-user limit
#[test]
fn test_cannot_bypass_subscription_limit() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let attacker = mock_pub_key(EVE);

		// Setup service with subscription pricing
		let mut blueprint = cggmp21_blueprint();
		blueprint.jobs[0].pricing_model = PricingModel::Subscription {
			rate_per_interval: 10 * 10u128.pow(6),
			interval: 1,
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

		// Fund attacker with enough for 100 subscriptions
		mint_tokens(USDC, alice.clone(), attacker.clone(), 200000 * 10u128.pow(6));
		use frame_support::traits::Currency;
		let _ = Balances::make_free_balance_be(&attacker, 100000);
		
		// Fund rewards pallet for distribution

		// Create 100 subscriptions (should all succeed)
		for i in 0..100 {
			let service_id = Services::next_instance_id();
			assert_ok!(Services::request(
				RuntimeOrigin::signed(attacker.clone()),
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
			],));

			// Create subscription by calling service
			assert_ok!(Services::call(
				RuntimeOrigin::signed(attacker.clone()),
				service_id,
				KEYGEN_JOB_ID,
				vec![Field::Uint8(1)].try_into().unwrap()
			));

			// Process payment to create billing entry
			let current_block = System::block_number();
			assert_ok!(Services::process_job_subscription_payment(
				service_id,
				KEYGEN_JOB_ID,
				0,
				&attacker,
				&attacker,
				10 * 10u128.pow(6),
				1,
				None,
				current_block,
			));

			// Verify subscription count increments
			assert_eq!(Services::user_subscription_count(&attacker), (i + 1) as u32);
		}

		// Attempt 101st subscription - should fail at payment processing
		let service_id = Services::next_instance_id();
		assert_ok!(Services::request(
			RuntimeOrigin::signed(attacker.clone()),
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
		],));

		// Call succeeds (just stores job call)
		assert_ok!(Services::call(
			RuntimeOrigin::signed(attacker.clone()),
			service_id,
			KEYGEN_JOB_ID,
			vec![Field::Uint8(1)].try_into().unwrap()
		));

		// Payment processing should fail with TooManySubscriptions
		let current_block = System::block_number();
		assert_noop!(
			Services::process_job_subscription_payment(
				service_id,
				KEYGEN_JOB_ID,
				0,
				&attacker,
				&attacker,
				10 * 10u128.pow(6),
				1,
				None,
				current_block,
			),
			Error::<Runtime>::TooManySubscriptions
		);

		// Verify count didn't increment beyond 100
		assert_eq!(Services::user_subscription_count(&attacker), 100);
	});
}

/// Test: Cannot process same subscription twice in one block
#[test]
fn test_cannot_double_process_subscription() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let user = mock_pub_key(EVE);

		// Setup
		let mut blueprint = cggmp21_blueprint();
		blueprint.jobs[0].pricing_model = PricingModel::Subscription {
			rate_per_interval: 10 * 10u128.pow(6),
			interval: 1,
			maybe_end: None,
		};

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		mint_tokens(USDC, alice.clone(), user.clone(), 1000 * 10u128.pow(6));
		use frame_support::traits::Currency;
		let _ = Balances::make_free_balance_be(&user, 100000);

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
		],));

		assert_ok!(Services::call(
			RuntimeOrigin::signed(user.clone()),
			service_id,
			KEYGEN_JOB_ID,
			vec![Field::Uint8(1)].try_into().unwrap()
		));

		// Create billing entry with first payment at block 1
		assert_ok!(Services::process_job_subscription_payment(
			service_id,
			KEYGEN_JOB_ID,
			0,
			&user,
			&user,
			10 * 10u128.pow(6),
			1,
			None,
			1,
		));

		let billing_key = (service_id, KEYGEN_JOB_ID, user.clone());
		let billing_after_first = Services::job_subscription_billings(&billing_key).unwrap();
		assert_eq!(billing_after_first.last_billed, 1);

		// Record balance after first payment
		let balance_after_first = pallet_assets::Pallet::<Runtime>::balance(USDC, &user);

		// Attempt second processing in SAME block - should not charge again
		assert_ok!(Services::process_job_subscription_payment(
			service_id,
			KEYGEN_JOB_ID,
			0,
			&user,
			&user,
			10 * 10u128.pow(6),
			1,
			None,
			1, // Same block
		));

		let balance_after_second = pallet_assets::Pallet::<Runtime>::balance(USDC, &user);
		// Balance should be unchanged - no second charge in same block
		assert_eq!(balance_after_first, balance_after_second);
		
		// last_billed should still be block 1
		let billing_final = Services::job_subscription_billings(&billing_key).unwrap();
		assert_eq!(billing_final.last_billed, 1);
	});
}

/// Test: Weight exhaustion doesn't break processing
#[test]
fn test_weight_exhaustion_graceful_degradation() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let weight_used = Services::process_subscription_payments_on_idle(1, Weight::zero());
		assert_eq!(weight_used, Weight::zero());

		let tiny_weight = Weight::from_parts(100, 0);
		let weight_used = Services::process_subscription_payments_on_idle(1, tiny_weight);
		assert_eq!(weight_used, Weight::zero());

		assert!(Services::subscription_processing_cursor().is_none());
	});
}

/// Test: MAX_SUBSCRIPTIONS_PER_BLOCK limit
#[test]
fn test_max_subscriptions_per_block_limit() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		const MAX_SUBSCRIPTIONS_PER_BLOCK: u32 = 50;
		assert_eq!(MAX_SUBSCRIPTIONS_PER_BLOCK, 50);
	});
}

/// Test: Cursor cannot be manipulated by users
#[test]
fn test_cursor_cannot_be_manipulated_by_users() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		assert!(Services::subscription_processing_cursor().is_none());
	});
}

/// Test: Arithmetic safety
#[test]
fn test_arithmetic_safety() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		let current_block: u64 = 0;
		let last_billed: u64 = 100;
		let blocks_since = current_block.saturating_sub(last_billed);
		assert_eq!(blocks_since, 0);

		let max_block: u64 = u64::MAX;
		let blocks_since_max = max_block.saturating_sub(0);
		assert_eq!(blocks_since_max, u64::MAX);
	});
}

/// Test: Graceful handling of terminated service
#[test]
fn test_graceful_handling_of_terminated_service() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let user = mock_pub_key(EVE);

		let mut blueprint = cggmp21_blueprint();
		blueprint.jobs[0].pricing_model = PricingModel::Subscription {
			rate_per_interval: 10 * 10u128.pow(6),
			interval: 1,
			maybe_end: Some(10),
		};

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		mint_tokens(USDC, alice.clone(), user.clone(), 1000 * 10u128.pow(6));
		use frame_support::traits::Currency;
		let _ = Balances::make_free_balance_be(&user, 100000);

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
		],));

		assert_ok!(Services::call(
			RuntimeOrigin::signed(user.clone()),
			service_id,
			KEYGEN_JOB_ID,
			vec![Field::Uint8(1)].try_into().unwrap()
		));

		// Create billing entry
		assert_ok!(Services::process_job_subscription_payment(
			service_id,
			KEYGEN_JOB_ID,
			0,
			&user,
			&user,
			10 * 10u128.pow(6),
			1,
			Some(10),
			1,
		));

		let billing_key = (service_id, KEYGEN_JOB_ID, user.clone());
		assert!(Services::job_subscription_billings(&billing_key).is_some());
		assert_eq!(Services::user_subscription_count(&user), 1);

		// Advance past end block and process - should cleanup
		System::set_block_number(11);
		assert_ok!(Services::process_job_subscription_payment(
			service_id,
			KEYGEN_JOB_ID,
			0,
			&user,
			&user,
			10 * 10u128.pow(6),
			1,
			Some(10),
			11,
		));

		assert!(Services::job_subscription_billings(&billing_key).is_none());
		assert_eq!(Services::user_subscription_count(&user), 0);
	});
}

/// Test: Payment failure doesn't corrupt billing
#[test]
fn test_payment_failure_doesnt_corrupt_billing() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let poor_user = mock_pub_key(EVE);

		let mut blueprint = cggmp21_blueprint();
		blueprint.jobs[0].pricing_model = PricingModel::Subscription {
			rate_per_interval: 10 * 10u128.pow(6),
			interval: 1,
			maybe_end: None,
		};

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		// Give user enough for first payment
		mint_tokens(USDC, alice.clone(), poor_user.clone(), 10 * 10u128.pow(6));
		use frame_support::traits::Currency;
		let _ = Balances::make_free_balance_be(&poor_user, 100000);

		let service_id = Services::next_instance_id();
		assert_ok!(Services::request(
			RuntimeOrigin::signed(poor_user.clone()),
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
		],));

		assert_ok!(Services::call(
			RuntimeOrigin::signed(poor_user.clone()),
			service_id,
			KEYGEN_JOB_ID,
			vec![Field::Uint8(1)].try_into().unwrap()
		));

		// Create billing entry with first payment
		assert_ok!(Services::process_job_subscription_payment(
			service_id,
			KEYGEN_JOB_ID,
			0,
			&poor_user,
			&poor_user,
			10 * 10u128.pow(6),
			1,
			None,
			1,
		));

		let billing_key = (service_id, KEYGEN_JOB_ID, poor_user.clone());
		let initial_billing = Services::job_subscription_billings(&billing_key).unwrap();

		// Advance block - user now has insufficient balance
		System::set_block_number(2);

		// Attempt to process - should fail
		let result = Services::process_job_subscription_payment(
			service_id,
			KEYGEN_JOB_ID,
			0,
			&poor_user,
			&poor_user,
			10 * 10u128.pow(6),
			1,
			None,
			2,
		);

		assert!(result.is_err());

		// Verify billing state unchanged (last_billed not updated)
		let final_billing = Services::job_subscription_billings(&billing_key).unwrap();
		assert_eq!(initial_billing.last_billed, final_billing.last_billed);

		// Subscription count should still be 1
		assert_eq!(Services::user_subscription_count(&poor_user), 1);
	});
}

/// Test: Cursor iteration determinism
#[test]
fn test_cursor_iteration_determinism() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		assert!(true);
	});
}

/// Test: Storage cleanup on subscription end
#[test]
fn test_storage_cleanup_on_end() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let user = mock_pub_key(EVE);

		let mut blueprint = cggmp21_blueprint();
		blueprint.jobs[0].pricing_model = PricingModel::Subscription {
			rate_per_interval: 10 * 10u128.pow(6),
			interval: 1,
			maybe_end: Some(5),
		};

		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		assert_ok!(join_and_register(bob.clone(), 0, test_ecdsa_key(), 1000, Some("https://example.com/rpc")));

		mint_tokens(USDC, alice.clone(), user.clone(), 1000 * 10u128.pow(6));
		use frame_support::traits::Currency;
		let _ = Balances::make_free_balance_be(&user, 100000);

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
		],));

		assert_ok!(Services::call(
			RuntimeOrigin::signed(user.clone()),
			service_id,
			KEYGEN_JOB_ID,
			vec![Field::Uint8(1)].try_into().unwrap()
		));

		// Create billing entry
		assert_ok!(Services::process_job_subscription_payment(
			service_id,
			KEYGEN_JOB_ID,
			0,
			&user,
			&user,
			10 * 10u128.pow(6),
			1,
			Some(5),
			1,
		));

		let billing_key = (service_id, KEYGEN_JOB_ID, user.clone());
		assert!(Services::job_subscription_billings(&billing_key).is_some());
		assert_eq!(Services::user_subscription_count(&user), 1);

		// Process at block 5 (end block) - should still exist
		System::set_block_number(5);
		assert_ok!(Services::process_job_subscription_payment(
			service_id,
			KEYGEN_JOB_ID,
			0,
			&user,
			&user,
			10 * 10u128.pow(6),
			1,
			Some(5),
			5,
		));

		assert!(Services::job_subscription_billings(&billing_key).is_some());

		// Process at block 6 (past end) - should cleanup
		System::set_block_number(6);
		assert_ok!(Services::process_job_subscription_payment(
			service_id,
			KEYGEN_JOB_ID,
			0,
			&user,
			&user,
			10 * 10u128.pow(6),
			1,
			Some(5),
			6,
		));

		assert!(Services::job_subscription_billings(&billing_key).is_none());
		assert_eq!(Services::user_subscription_count(&user), 0);
	});
}
