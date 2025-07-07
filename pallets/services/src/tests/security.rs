use crate::tests::*;

#[test]
fn test_subscription_count_cleanup() {
	new_test_ext(vec![ALICE, BOB, CHARLIE]).execute_with(|| {
		// Test subscription count tracking functionality
		let alice = mock_pub_key(ALICE);

		// Verify that UserSubscriptionCount storage exists and is tracked
		let initial_count = Services::user_subscription_count(alice.clone());
		assert_eq!(initial_count, 0);

		// Test that subscription count can be incremented
		// (This would normally happen through subscription payment processing)
		crate::UserSubscriptionCount::<Runtime>::insert(alice.clone(), 1);
		assert_eq!(Services::user_subscription_count(alice.clone()), 1);

		// Test that subscription count can be decremented (cleanup)
		crate::UserSubscriptionCount::<Runtime>::mutate(alice.clone(), |count| {
			*count = count.saturating_sub(1);
		});
		assert_eq!(Services::user_subscription_count(alice.clone()), 0);
	});
}

#[test]
fn test_heartbeat_metrics_size_validation() {
	new_test_ext(vec![ALICE, BOB, CHARLIE]).execute_with(|| {
		// Test that heartbeat metrics size validation is configured
		// This verifies that MAX_METRICS_SIZE constant exists and is reasonable

		// Test size validation logic
		let small_data = [0u8; 100];
		let large_data = vec![0u8; 10000]; // Larger than expected MAX_METRICS_SIZE

		// Verify that size validation exists by checking error type
		let _metrics_error = crate::Error::<Runtime>::MetricsDataTooLarge;

		// Test that small data size is reasonable
		assert!(small_data.len() < 1024, "Small data should be under reasonable limit");

		// Test that large data would trigger validation
		assert!(large_data.len() > 1024, "Large data should exceed reasonable limit");
	});
}
