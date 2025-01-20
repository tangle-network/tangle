use super::*;
use crate::ApyBlocks;
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

#[test]
fn test_calculate_reward_per_block_zero_blocks() {
	new_test_ext().execute_with(|| {
		ApyBlocks::<Runtime>::put(0);
		let total_reward = 1000;

		let result = RewardsPallet::<Runtime>::calculate_reward_per_block(total_reward);

		// With zero blocks, should return None (division by zero)
		assert_eq!(result, None);
	});
}

#[test]
fn test_calculate_reward_per_block_normal_case() {
	new_test_ext().execute_with(|| {
		ApyBlocks::<Runtime>::put(100);
		let total_reward = 1000;

		let result = RewardsPallet::<Runtime>::calculate_reward_per_block(total_reward);

		// Should evenly distribute rewards across blocks
		assert_eq!(result, Some(10)); // 1000/100 = 10
	});
}

#[test]
fn test_calculate_reward_per_block_zero_reward() {
	new_test_ext().execute_with(|| {
		ApyBlocks::<Runtime>::put(100);
		let total_reward = 0;

		let result = RewardsPallet::<Runtime>::calculate_reward_per_block(total_reward);

		// Zero reward should result in zero per block
		assert_eq!(result, Some(0));
	});
}

#[test]
fn test_calculate_reward_per_block_one_block() {
	new_test_ext().execute_with(|| {
		ApyBlocks::<Runtime>::put(1);
		let total_reward = 1000;

		let result = RewardsPallet::<Runtime>::calculate_reward_per_block(total_reward);

		// With one block, should return full reward
		assert_eq!(result, Some(total_reward));
	});
}

#[test]
fn test_calculate_reward_per_block_large_numbers() {
	new_test_ext().execute_with(|| {
		ApyBlocks::<Runtime>::put(u64::MAX);
		let total_reward = u128::MAX;

		let result = RewardsPallet::<Runtime>::calculate_reward_per_block(total_reward);

		// Should handle large numbers without overflow
		assert!(result.is_some());
		if let Some(per_block) = result {
			assert!(per_block > 0);
			// Verify the per-block reward times blocks doesn't exceed total
			let blocks_balance = BalanceOf::<Runtime>::from(u32::MAX);
			assert!(per_block.saturating_mul(blocks_balance) <= total_reward);
		}
	});
}

#[test]
fn test_calculate_reward_per_block_uneven_division() {
	new_test_ext().execute_with(|| {
		ApyBlocks::<Runtime>::put(3);
		let total_reward = 10;

		let result = RewardsPallet::<Runtime>::calculate_reward_per_block(total_reward);

		// Should handle uneven division (10/3 = 3 with remainder)
		assert_eq!(result, Some(3));
	});
}
