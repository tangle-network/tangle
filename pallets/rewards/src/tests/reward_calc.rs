use super::*;
use crate::ApyBlocks;
use frame_support::assert_ok;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::Percent;
use tangle_primitives::types::rewards::{LockInfo, LockMultiplier, UserDepositWithLocks};

// Helper function to setup test environment with consistent values
fn setup_test_env() {
	ApyBlocks::<Runtime>::put(100); // 100 blocks for APY period
	System::set_block_number(1000); // Set current block to 1000
}

// Mock values for consistent testing
const MOCK_DEPOSIT_CAP: u128 = 1_000_000; // 1 million
const MOCK_INCENTIVE_CAP: u128 = 10_000; // 10_000
const MOCK_APY: u8 = 10; // 10% APY
const MOCK_DEPOSIT: u128 = 100_000; // 100k deposit
const BLOCKS_PER_YEAR: u32 = 100; // Simplified for testing

#[test]
fn test_calculate_rewards_zero_deposit() {
	new_test_ext().execute_with(|| {
		setup_test_env();

		let total_deposit = 0;
		let total_asset_score = 0;
		let deposit = UserDepositWithLocks { unlocked_amount: 0, amount_with_locks: None };
		let reward = RewardConfigForAssetVault {
			apy: Percent::from_percent(MOCK_APY),
			deposit_cap: MOCK_DEPOSIT_CAP,
			incentive_cap: MOCK_INCENTIVE_CAP,
			boost_multiplier: None,
		};

		let last_claim = None;

		let result = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit,
			reward,
			last_claim,
		);

		assert_ok!(result, 0);
	});
}

#[test]
fn test_calculate_rewards_only_unlocked() {
	new_test_ext().execute_with(|| {
		setup_test_env();

		let total_deposit = MOCK_DEPOSIT;
		let total_asset_score = MOCK_DEPOSIT;
		let user_deposit = 10_000; // 10k deposit
		let deposit =
			UserDepositWithLocks { unlocked_amount: user_deposit, amount_with_locks: None };

		let reward = RewardConfigForAssetVault {
			apy: Percent::from_percent(MOCK_APY),
			deposit_cap: MOCK_DEPOSIT_CAP,
			incentive_cap: MOCK_INCENTIVE_CAP,
			boost_multiplier: None,
		};

		// Use genesis block as last claim
		let last_claim = Some((0, 0));

		let result = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit,
			reward,
			last_claim,
		);

		// Calculate expected rewards:
		// 1. APY adjustment: 10% * (100k/1M) = 1% effective APY
		// 2. User share: 10k deposit = 10% of total deposit
		// 3. Total reward = 10k * 1% = 100
		// 4. Per block = 100 / 100 blocks = 1
		// 5. Blocks since last claim = 1000 (current) - 0 = 1000
		let expected_to_pay = 1000; // 1 per block * 1000 blocks

		assert_ok!(result, expected_to_pay);
	});
}

#[test]
fn test_calculate_rewards_with_expired_lock() {
	new_test_ext().execute_with(|| {
		setup_test_env();

		let current_block = 1000;
		let expired_block = 900;

		let total_deposit = MOCK_DEPOSIT;
		let total_asset_score = MOCK_DEPOSIT * 2; // Due to lock multipliers
		let user_deposit = 10_000;
		let deposit = UserDepositWithLocks {
			unlocked_amount: user_deposit,
			amount_with_locks: Some(vec![LockInfo {
				amount: user_deposit,
				lock_multiplier: LockMultiplier::TwoMonths,
				expiry_block: expired_block,
			}]),
		};

		let reward = RewardConfigForAssetVault {
			apy: Percent::from_percent(MOCK_APY),
			deposit_cap: MOCK_DEPOSIT_CAP,
			incentive_cap: MOCK_INCENTIVE_CAP,
			boost_multiplier: Some(150),
		};

		// Use genesis block as last claim
		let last_claim = Some((0, 0));

		let result = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit,
			reward,
			last_claim,
		);

		// Only unlocked amount should count since lock is expired
		let expected_to_pay = 1000; // 1 per block * 1000 blocks

		assert_ok!(result, expected_to_pay);
	});
}

#[test]
fn test_calculate_rewards_with_active_locks() {
	new_test_ext().execute_with(|| {
		setup_test_env();

		let current_block = 1000;
		let future_block = 2000;

		let total_deposit = MOCK_DEPOSIT;
		let total_asset_score = MOCK_DEPOSIT * 3; // Average multiplier effect
		let user_deposit = 10_000;
		let deposit = UserDepositWithLocks {
			unlocked_amount: user_deposit,
			amount_with_locks: Some(vec![
				LockInfo {
					amount: user_deposit,
					lock_multiplier: LockMultiplier::TwoMonths,
					expiry_block: future_block,
				},
				LockInfo {
					amount: user_deposit,
					lock_multiplier: LockMultiplier::ThreeMonths,
					expiry_block: future_block,
				},
			]),
		};

		let reward = RewardConfigForAssetVault {
			apy: Percent::from_percent(MOCK_APY),
			deposit_cap: MOCK_DEPOSIT_CAP,
			incentive_cap: MOCK_INCENTIVE_CAP,
			boost_multiplier: Some(100),
		};

		// Use genesis block as last claim
		let last_claim = Some((0, 0));

		let result = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit,
			reward,
			last_claim,
		);

		// Calculate expected rewards:
		// 1. Score = 10k (unlocked) + 20k (2x lock) + 30k (3x lock) = 60k
		// 2. APY adjustment: 10% * (100k/1M) = 1% effective APY
		// 3. User share: 60k score = 20% of total score (300k)
		// 4. Total reward = 60k * 1% = 600
		// 5. Per block = 600 / 100 blocks = 6
		// 6. Blocks since last claim = 1000
		let expected_to_pay = 6000; // 6 per block * 1000 blocks

		assert_ok!(result, expected_to_pay);
	});
}

#[test]
fn test_calculate_rewards_with_previous_claim() {
	new_test_ext().execute_with(|| {
		setup_test_env();

		let current_block = 1000;
		let last_claim_block = 900;

		let total_deposit = MOCK_DEPOSIT;
		let total_asset_score = MOCK_DEPOSIT;
		let user_deposit = 10_000;
		let deposit =
			UserDepositWithLocks { unlocked_amount: user_deposit, amount_with_locks: None };
		let reward = RewardConfigForAssetVault {
			apy: Percent::from_percent(MOCK_APY),
			deposit_cap: MOCK_DEPOSIT_CAP,
			incentive_cap: MOCK_INCENTIVE_CAP,
			boost_multiplier: None,
		};

		let last_claim = Some((last_claim_block, 50));

		let result = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit,
			reward,
			last_claim,
		);

		// Calculate expected rewards:
		// 1. Total reward calculation same as unlocked case = 100
		// 2. Per block = 1
		// 3. Blocks since last claim = 100
		let expected_to_pay = 100; // 1 per block * 100 blocks

		assert_ok!(result, expected_to_pay);
	});
}

#[test]
fn test_calculate_rewards_zero_cap() {
	new_test_ext().execute_with(|| {
		setup_test_env();

		let total_deposit = MOCK_DEPOSIT;
		let total_asset_score = MOCK_DEPOSIT;
		let deposit = UserDepositWithLocks { unlocked_amount: 10_000, amount_with_locks: None };
		let reward = RewardConfigForAssetVault {
			apy: Percent::from_percent(MOCK_APY),
			deposit_cap: 0,
			incentive_cap: MOCK_INCENTIVE_CAP,
			boost_multiplier: None,
		};

		let last_claim = None;

		let result = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit,
			reward,
			last_claim,
		);

		assert!(result.is_err());
		assert_eq!(result.unwrap_err(), Error::<Runtime>::ArithmeticError.into());
	});
}

#[test]
fn test_calculate_rewards_same_block_claim() {
	new_test_ext().execute_with(|| {
		setup_test_env();

		let current_block = 1000;

		let total_deposit = MOCK_DEPOSIT;
		let total_asset_score = MOCK_DEPOSIT;
		let user_deposit = 10_000;
		let deposit =
			UserDepositWithLocks { unlocked_amount: user_deposit, amount_with_locks: None };
		let reward = RewardConfigForAssetVault {
			apy: Percent::from_percent(MOCK_APY),
			deposit_cap: MOCK_DEPOSIT_CAP,
			incentive_cap: MOCK_INCENTIVE_CAP,
			boost_multiplier: None,
		};

		let last_claim = Some((current_block, 50));

		let result = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit,
			reward,
			last_claim,
		);

		// Calculate expected rewards:
		// 1. Total reward calculation same as unlocked case = 100
		// 2. Per block = 1
		// 3. Blocks since last claim = 0
		let expected_to_pay = 0; // 0 blocks passed

		assert_ok!(result, expected_to_pay);
	});
}
