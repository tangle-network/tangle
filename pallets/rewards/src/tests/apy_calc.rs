use super::*;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::Percent;

#[test]
fn test_calculate_proportional_apy_zero_deposit() {
    new_test_ext().execute_with(|| {
        let total_deposit = 0;
        let deposit_cap = 1000;
        let original_apy = Percent::from_percent(10);

        let result = RewardsPallet::<Runtime>::calculate_propotional_apy(
            total_deposit,
            deposit_cap,
            original_apy,
        );

        // With zero deposit, APY should be zero
        assert_eq!(result, Some(Percent::zero()));
    });
}

#[test]
fn test_calculate_proportional_apy_full_cap() {
    new_test_ext().execute_with(|| {
        let deposit_cap = 1000;
        let total_deposit = deposit_cap; // Full capacity
        let original_apy = Percent::from_percent(10);

        let result = RewardsPallet::<Runtime>::calculate_propotional_apy(
            total_deposit,
            deposit_cap,
            original_apy,
        );

        // At full capacity, should return original APY
        assert_eq!(result, Some(original_apy));
    });
}

#[test]
fn test_calculate_proportional_apy_half_cap() {
    new_test_ext().execute_with(|| {
        let deposit_cap = 1000;
        let total_deposit = deposit_cap / 2; // Half capacity
        let original_apy = Percent::from_percent(10);

        let result = RewardsPallet::<Runtime>::calculate_propotional_apy(
            total_deposit,
            deposit_cap,
            original_apy,
        );

        // At half capacity, should return half of original APY
        assert_eq!(result, Some(Percent::from_percent(5)));
    });
}

#[test]
fn test_calculate_proportional_apy_zero_cap() {
    new_test_ext().execute_with(|| {
        let deposit_cap = 0;
        let total_deposit = 100;
        let original_apy = Percent::from_percent(10);

        let result = RewardsPallet::<Runtime>::calculate_propotional_apy(
            total_deposit,
            deposit_cap,
            original_apy,
        );

        // With zero cap, should return None (division by zero)
        assert_eq!(result, None);
    });
}

#[test]
fn test_calculate_proportional_apy_over_cap() {
    new_test_ext().execute_with(|| {
        let deposit_cap = 1000;
        let total_deposit = deposit_cap * 2; // Double the cap
        let original_apy = Percent::from_percent(10);

        let result = RewardsPallet::<Runtime>::calculate_propotional_apy(
            total_deposit,
            deposit_cap,
            original_apy,
        );

        // Over capacity should still work, but will return full APY
        // This is because Percent::from_rational clamps to 100%
        assert_eq!(result, Some(original_apy));
    });
}

#[test]
fn test_calculate_proportional_apy_max_values() {
    new_test_ext().execute_with(|| {
        let deposit_cap = u128::MAX;
        let total_deposit = deposit_cap; // Max value
        let original_apy = Percent::from_percent(100);

        let result = RewardsPallet::<Runtime>::calculate_propotional_apy(
            total_deposit,
            deposit_cap,
            original_apy,
        );

        // Even with max values, should handle calculation correctly
        assert_eq!(result, Some(original_apy));
    });
}

#[test]
fn test_calculate_proportional_apy_small_values() {
    new_test_ext().execute_with(|| {
        let deposit_cap = 1_000_000;
        let total_deposit = 1; // Minimal deposit
        let original_apy = Percent::from_percent(10);

        let result = RewardsPallet::<Runtime>::calculate_propotional_apy(
            total_deposit,
            deposit_cap,
            original_apy,
        );

        // Should handle very small proportions
        // Expected: 0.0001% of 10% = ~0%
        assert!(result.unwrap().is_zero());
    });
}

