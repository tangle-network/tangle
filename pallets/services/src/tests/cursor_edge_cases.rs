//! Critical Edge Case Tests for Subscription Cursor System
//!
//! These tests validate the fixes for the critical bugs identified in code review:
//! 1. Cursor deletion scenario (subscription cancelled while cursor points to it)
//! 2. Cursor cleanup with exactly 50 subscriptions
//! 3. Cursor cleanup with 100 subscriptions (50+50 batches)

use super::*;
use frame_support::{assert_ok, weights::Weight};

/// Test: Cursor deletion scenario - subscription cancelled while cursor points to it
///
/// This test validates the fix for the critical bug where if a subscription is deleted
/// while the cursor points to it, the iteration would break permanently, causing all
/// remaining subscriptions to never be billed again.
///
/// ## Bug Description
/// Original code would skip entries until cursor matched, but if cursor was deleted,
/// the match never happened, causing skip_until_cursor to remain true forever.
///
/// ## Fix Validation
/// The fix validates cursor existence before iteration and resets if deleted.
#[test]
fn test_cursor_deletion_recovery() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);

		// Setup blueprint with subscription pricing
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

		use frame_support::traits::Currency;

		// Create 10 subscriptions
		for user_id in 10..20 {
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

			assert_ok!(Services::approve(
				RuntimeOrigin::signed(bob.clone()),
				service_id,
				vec![get_security_commitment(TNT, 10), get_security_commitment(WETH, 10)]
			));

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
				user_id as u64,
				&user,
				&user,
				10 * 10u128.pow(6),
				1,
				None,
				1,
			));
		}

		// Verify 10 subscriptions exist
		let billing_count = JobSubscriptionBillings::<Runtime>::iter().count();
		assert_eq!(billing_count, 10, "Should have 10 billing entries");

		// Advance to block 2 and process first 5 subscriptions (simulate weight limit)
		System::set_block_number(2);

		// Get the 5th subscription key (this will be our cursor position)
		let fifth_key = JobSubscriptionBillings::<Runtime>::iter().nth(4).map(|(k, _)| k);
		assert!(fifth_key.is_some(), "5th subscription should exist");

		// Manually set cursor to 5th position to simulate processing first 4
		if let Some(key) = fifth_key.clone() {
			SubscriptionProcessingCursor::<Runtime>::put(key);
		}

		// Now DELETE the subscription at cursor position (critical scenario!)
		if let Some(key) = fifth_key.clone() {
			JobSubscriptionBillings::<Runtime>::remove(&key);
			println!("❌ DELETED subscription at cursor position: service={}, job={}", key.0, key.1);
		}

		// Verify subscription was deleted
		let billing_count_after_delete = JobSubscriptionBillings::<Runtime>::iter().count();
		assert_eq!(
			billing_count_after_delete, 9,
			"Should have 9 billing entries after deletion"
		);

		// NOW TEST THE FIX: Process subscriptions with cursor pointing to deleted entry
		// The fix should detect the deleted cursor and reset to beginning
		System::set_block_number(3);
		let generous_weight = Weight::from_parts(500_000_000_000, 64 * 1024);

		let _weight_used = Services::process_subscription_payments_on_idle(3, generous_weight);

		// Count how many were processed
		let mut processed_count = 0;
		for (_key, billing) in JobSubscriptionBillings::<Runtime>::iter() {
			if billing.last_billed == 3 {
				processed_count += 1;
			}
		}

		// CRITICAL ASSERTION: Without the fix, processed_count would be 0
		// With the fix, all remaining 9 subscriptions should be processed
		assert!(
			processed_count > 0,
			"CRITICAL FIX VALIDATION FAILED: Cursor deletion caused permanent billing freeze! \
			No subscriptions were processed. This proves the bug exists."
		);

		println!("✓ Cursor deletion fix validated: {} subscriptions processed after cursor deletion", processed_count);

		// Verify cursor was cleared after processing
		let cursor_after = SubscriptionProcessingCursor::<Runtime>::get();
		assert!(
			cursor_after.is_none() || processed_count == 9,
			"Cursor should be cleared or all subscriptions processed"
		);

		println!("✓ TEST PASSED: Cursor deletion recovery working correctly");
	});
}

/// Test: Cursor cleanup with exactly 50 subscriptions
///
/// This test validates the fix for the cursor cleanup logic bug where
/// the condition `processed_count < MAX_SUBSCRIPTIONS_PER_BLOCK` would
/// fail to clear the cursor when exactly 50 subscriptions exist.
///
/// ## Bug Description
/// If exactly 50 subscriptions exist and all are processed, processed_count = 50.
/// The condition `50 < 50` is false, so cursor persists forever.
///
/// ## Fix Validation
/// The fix tracks iteration completion separately from count.
#[test]
fn test_cursor_cleanup_exactly_50_subscriptions() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);

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

		use frame_support::traits::Currency;

		// Create EXACTLY 50 subscriptions (the edge case!)
		for user_id in 10..60 {
			let user = mock_pub_key(user_id as u8);
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

			assert_ok!(Services::approve(
				RuntimeOrigin::signed(bob.clone()),
				service_id,
				vec![get_security_commitment(TNT, 10), get_security_commitment(WETH, 10)]
			));

			assert_ok!(Services::call(
				RuntimeOrigin::signed(user.clone()),
				service_id,
				KEYGEN_JOB_ID,
				vec![Field::Uint8(1)].try_into().unwrap()
			));

			assert_ok!(Services::process_job_subscription_payment(
				service_id,
				KEYGEN_JOB_ID,
				user_id as u64,
				&user,
				&user,
				10 * 10u128.pow(6),
				1,
				None,
				1,
			));
		}

		// Verify exactly 50 subscriptions
		let billing_count = JobSubscriptionBillings::<Runtime>::iter().count();
		assert_eq!(billing_count, 50, "Should have exactly 50 billing entries");

		// Process all 50 in one block with generous weight
		System::set_block_number(2);
		let generous_weight = Weight::from_parts(500_000_000_000, 64 * 1024);

		let _weight_used = Services::process_subscription_payments_on_idle(2, generous_weight);

		// Count processed
		let mut processed_count = 0;
		for (_key, billing) in JobSubscriptionBillings::<Runtime>::iter() {
			if billing.last_billed == 2 {
				processed_count += 1;
			}
		}

		assert_eq!(processed_count, 50, "All 50 subscriptions should be processed");

		// CRITICAL ASSERTION: Cursor MUST be cleared
		let cursor_after = SubscriptionProcessingCursor::<Runtime>::get();
		assert!(
			cursor_after.is_none(),
			"CRITICAL FIX VALIDATION FAILED: Cursor persists after processing exactly 50 subscriptions! \
			This proves the cursor cleanup bug exists. Cursor: {:?}",
			cursor_after
		);

		println!("✓ TEST PASSED: Cursor correctly cleared after processing exactly 50 subscriptions");
	});
}

/// Test: Cursor cleanup with 100 subscriptions processed in 2 batches of 50
///
/// This test validates that cursor cleanup works correctly when subscriptions
/// are processed in multiple batches where the final batch is exactly 50.
///
/// ## Bug Description
/// Block 1: processes 50, sets cursor
/// Block 2: processes remaining 50 (iteration complete!)
/// Bug: processed_count = 50, NOT < 50, so cursor persists
///
/// ## Fix Validation
/// The fix uses iteration_incomplete flag instead of count comparison.
#[test]
fn test_cursor_cleanup_100_subscriptions_two_batches() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		System::set_block_number(1);

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);

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

		use frame_support::traits::Currency;

		// Create 100 subscriptions across 10 users (10 subscriptions each)
		// User IDs 10-19, each gets 10 subscriptions
		for i in 0..100 {
			let user_idx = 10 + (i / 10); // Users 10-19
			let user = mock_pub_key(user_idx as u8);

			// Fund each user once (when we first encounter them)
			if i % 10 == 0 {
				mint_tokens(USDC, alice.clone(), user.clone(), 1_000_000 * 10u128.pow(6));
				let _ = Balances::make_free_balance_be(&user, 1_000_000 * 10u128.pow(6));
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

			assert_ok!(Services::approve(
				RuntimeOrigin::signed(bob.clone()),
				service_id,
				vec![get_security_commitment(TNT, 10), get_security_commitment(WETH, 10)]
			));

			assert_ok!(Services::call(
				RuntimeOrigin::signed(user.clone()),
				service_id,
				KEYGEN_JOB_ID,
				vec![Field::Uint8(1)].try_into().unwrap()
			));

			assert_ok!(Services::process_job_subscription_payment(
				service_id,
				KEYGEN_JOB_ID,
				i as u64,
				&user,
				&user,
				10 * 10u128.pow(6),
				1,
				None,
				1,
			));
		}

		// Verify 100 subscriptions
		let billing_count = JobSubscriptionBillings::<Runtime>::iter().count();
		assert_eq!(billing_count, 100, "Should have 100 billing entries");

		// Block 2: Process first batch (hits MAX limit of 50)
		System::set_block_number(2);
		let generous_weight = Weight::from_parts(500_000_000_000, 64 * 1024);
		let _weight1 = Services::process_subscription_payments_on_idle(2, generous_weight);

		let mut processed_block2 = 0;
		for (_key, billing) in JobSubscriptionBillings::<Runtime>::iter() {
			if billing.last_billed == 2 {
				processed_block2 += 1;
			}
		}

		assert_eq!(processed_block2, 50, "First batch should process exactly 50");

		// Cursor should be set
		let cursor_after_block2 = SubscriptionProcessingCursor::<Runtime>::get();
		assert!(cursor_after_block2.is_some(), "Cursor should be set after first batch");

		// Block 3: Process second batch (also 50, but iteration completes!)
		System::set_block_number(3);
		let _weight2 = Services::process_subscription_payments_on_idle(3, generous_weight);

		let mut processed_block3 = 0;
		for (_key, billing) in JobSubscriptionBillings::<Runtime>::iter() {
			if billing.last_billed == 3 {
				processed_block3 += 1;
			}
		}

		assert_eq!(processed_block3, 50, "Second batch should process remaining 50");

		// CRITICAL ASSERTION: Cursor MUST be cleared
		let cursor_after_block3 = SubscriptionProcessingCursor::<Runtime>::get();
		assert!(
			cursor_after_block3.is_none(),
			"CRITICAL FIX VALIDATION FAILED: Cursor persists after completing 100 subscriptions in 2 batches! \
			This proves the cursor cleanup bug exists. Cursor: {:?}",
			cursor_after_block3
		);

		println!("✓ TEST PASSED: Cursor correctly cleared after processing 100 subscriptions in 2 batches");
	});
}
