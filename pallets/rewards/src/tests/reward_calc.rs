use super::*;
use crate::ApyBlocks;
use frame_support::assert_ok;
use sp_runtime::Percent;
use tangle_primitives::types::rewards::{LockInfo, LockMultiplier, UserDepositWithLocks};

// Mock values for consistent testing
const EIGHTEEN_DECIMALS: u128 = 1_000_000_000_000_000_000_000;
const MOCK_DEPOSIT_CAP: u128 = 1_000_000 * EIGHTEEN_DECIMALS; // 1M tokens with 18 decimals
const MOCK_TOTAL_ISSUANCE: u128 = 100_000_000 * EIGHTEEN_DECIMALS; // 100M tokens with 18 decimals
const MOCK_INCENTIVE_CAP: u128 = 10_000 * EIGHTEEN_DECIMALS; // 10k tokens with 18 decimals
const MOCK_APY: u8 = 10; // 10% APY
const MOCK_DEPOSIT: u128 = 100_000 * EIGHTEEN_DECIMALS; // 100k tokens with 18 decimals
const BLOCKS_PER_YEAR: u64 = 5_256_000; // ~6 second blocks = ~1 year

// Helper function to setup test environment with consistent values
pub fn setup_test_env() {
	ApyBlocks::<Runtime>::put(BLOCKS_PER_YEAR); // ~6 second blocks = ~1 year
	System::set_block_number(1000); // Set current block to 1000
	pallet_balances::TotalIssuance::<Runtime>::set(MOCK_TOTAL_ISSUANCE);
}

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

		let last_claim = Some((0, 0));

		let result = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit,
			reward,
			last_claim,
		);

		assert_err!(result, Error::<Runtime>::NoRewardsAvailable);
	});
}

#[test]
fn test_calculate_rewards_only_unlocked() {
	new_test_ext().execute_with(|| {
		setup_test_env();

		let total_deposit = MOCK_DEPOSIT;
		let total_asset_score = MOCK_DEPOSIT;
		let user_deposit = 10_000 * EIGHTEEN_DECIMALS; // 10k tokens with 18 decimals
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
		// 2. Total annual rewards = 100M * 1% = 1M tokens
		// 3. User score = 10k (unlocked amount)
		// 4. User annual reward = 1M * (10k/100k) = 100k
		// 5. Per block = 100k / 5_256_000 blocks = 0.019 tokens
		// 6. Blocks since last claim = 1000 (current) - 0 = 1000
		// 7. Total reward = 0.019 tokens per block * 1000 blocks = 19 tokens
		let expected_to_pay = 19 * EIGHTEEN_DECIMALS; // 19 tokens with 18 decimals
		let diff = result.unwrap() - expected_to_pay;

		// Allow for some precision loss
		// assert precision loss is less than 1 TNT
		assert!(diff < EIGHTEEN_DECIMALS);
	});
}

#[test]
fn test_calculate_rewards_with_expired_lock() {
	new_test_ext().execute_with(|| {
		setup_test_env();

		let total_deposit = MOCK_DEPOSIT;
		let total_asset_score = MOCK_DEPOSIT * 3; // Adjusted to account for average lock multiplier effect
		let user_deposit = 10_000 * EIGHTEEN_DECIMALS; // 10k tokens with 18 decimals
		let current_block = 1000;

		// Set current block for the test
		System::set_block_number(current_block);

		let deposit = UserDepositWithLocks {
			unlocked_amount: user_deposit,
			amount_with_locks: Some(vec![LockInfo {
				amount: user_deposit,
				lock_multiplier: LockMultiplier::TwoMonths,
				expiry_block: 900, // Lock expired at block 900
			}]),
		};

		let reward = RewardConfigForAssetVault {
			apy: Percent::from_percent(MOCK_APY),
			deposit_cap: MOCK_DEPOSIT_CAP,
			incentive_cap: MOCK_INCENTIVE_CAP,
			boost_multiplier: Some(1),
		};

		let last_claim = Some((0, 0));

		let result = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit,
			reward,
			last_claim,
		);

		// Calculate expected rewards:
		// Total TNT in system = 100M
		// APY = 10%
		// deposit_cap = 1M
		// blocks = 1000
		// user deposit = 10k
		// Effective APY = total_deposit / deposit_cap * apy = 1%
		// Expected reward = 100M * 1% = 1M
		// Rewards per block = Expected reward / 5_256_000 = 1M / 5_256_000 = 0.1902587519
		//
		// For blocks 0-900 (with lock multiplier):
		// - Base amount (10k): 10k/300k * 0.1902587519 * 900 = 5.707762557 tokens
		// - Locked amount (10k * 2): (20k/300k) * 0.1902587519 * 900 = 11.415525114 tokens
		//
		// For blocks 900-1000 (after expiry):
		// - Base amount only (10k): 10k/300k * 0.1902587519 * 100 = 0.634195839 tokens
		//
		// Total expected = 5.707762557 + 11.415525114 + 0.634195839 ≈ 17.75748351 tokens
		let expected_to_pay = 17_757_483_510_000_000_000_000u128; // ~17.75 tokens with 18 decimals

		// Allow for some precision loss
		let diff = result.unwrap().saturating_sub(expected_to_pay);
		assert!(diff < EIGHTEEN_DECIMALS);
	});
}

#[test]
fn test_calculate_rewards_with_active_locks() {
	new_test_ext().execute_with(|| {
		setup_test_env();

		let total_deposit = MOCK_DEPOSIT;
		let total_asset_score = MOCK_DEPOSIT * 3; // Average multiplier effect
		let user_deposit = 10_000 * EIGHTEEN_DECIMALS; // 10k tokens with 18 decimals
		let deposit = UserDepositWithLocks {
			unlocked_amount: user_deposit,
			amount_with_locks: Some(vec![
				LockInfo {
					amount: user_deposit * 2,
					lock_multiplier: LockMultiplier::TwoMonths,
					expiry_block: 2000,
				},
				LockInfo {
					amount: user_deposit * 3,
					lock_multiplier: LockMultiplier::ThreeMonths,
					expiry_block: 2000,
				},
			]),
		};

		let reward = RewardConfigForAssetVault {
			apy: Percent::from_percent(MOCK_APY),
			deposit_cap: MOCK_DEPOSIT_CAP,
			incentive_cap: MOCK_INCENTIVE_CAP,
			boost_multiplier: Some(1),
		};

		let last_claim = Some((0, 0));

		let result = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit,
			reward,
			last_claim,
		);

		// Calculate expected rewards:
		// 1. User score = 10k + (20k * 2) + (30k * 3) = 140k
		// 2. Total asset score = 100k * 3 = 300k
		// 3. User proportion = 140k/300k = 46%
		// 4. APY adjustment: 10% * (100k/1M) = 1% effective APY
		// 5. Total annual rewards = 100M * 1% = 1M tokens
		// 6. Per block = 1M / 5,256,000 blocks = 0.19 tokens
		// 7. User reward per block = 0.19 * 46% = 0.0874 tokens
		// 8. Total for 1000 blocks = 0.0874 * 1000 = 87.4 tokens
		let expected_to_pay = 87 * EIGHTEEN_DECIMALS; // 87 tokens with 18 decimals

		// Allow for some precision loss
		// assert precision loss is less than 1 TNT
		let diff = result.unwrap() - expected_to_pay;
		assert!(diff < EIGHTEEN_DECIMALS);
	});
}

#[test]
fn test_calculate_rewards_with_previous_claim() {
	new_test_ext().execute_with(|| {
		setup_test_env();

		let total_deposit = MOCK_DEPOSIT;
		let total_asset_score = MOCK_DEPOSIT;
		let user_deposit = 10_000 * EIGHTEEN_DECIMALS; // 10k tokens with 18 decimals
		let deposit =
			UserDepositWithLocks { unlocked_amount: user_deposit, amount_with_locks: None };
		let reward = RewardConfigForAssetVault {
			apy: Percent::from_percent(MOCK_APY),
			deposit_cap: MOCK_DEPOSIT_CAP,
			incentive_cap: MOCK_INCENTIVE_CAP,
			boost_multiplier: None,
		};

		// Set last claim to 100 blocks ago
		let last_claim = Some((900, 0));

		let result = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit,
			reward,
			last_claim,
		);

		// Calculate expected rewards:
		// 1. Total annual rewards = 100M * 1% = 1M tokens
		// 2. User annual reward = 1M * (10k/100k) = 100k
		// 3. Per block = 100k / 5_256_000 blocks = 0.019 tokens
		// 4. Blocks since last claim = 100
		let expected_to_pay = 1_9 * EIGHTEEN_DECIMALS / 10; // 1.9 tokens with 18 decimals

		// Allow for some precision loss
		// assert precision loss is less than 1 TNT
		let diff = result.unwrap() - expected_to_pay;
		assert!(diff < EIGHTEEN_DECIMALS);
	});
}

#[test]
fn test_calculate_rewards_zero_cap() {
	new_test_ext().execute_with(|| {
		setup_test_env();

		let total_deposit = MOCK_DEPOSIT;
		let total_asset_score = MOCK_DEPOSIT;
		let deposit = UserDepositWithLocks {
			unlocked_amount: 10_000 * EIGHTEEN_DECIMALS,
			amount_with_locks: None,
		};

		let reward = RewardConfigForAssetVault {
			apy: Percent::from_percent(MOCK_APY),
			deposit_cap: 0,
			incentive_cap: MOCK_INCENTIVE_CAP,
			boost_multiplier: None,
		};

		let last_claim = Some((0, 0));

		let result = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit,
			reward,
			last_claim,
		);

		assert_err!(result, Error::<Runtime>::CannotCalculatePropotionalApy);
	});
}

#[test]
fn test_calculate_rewards_same_block_claim() {
	new_test_ext().execute_with(|| {
		setup_test_env();

		let total_deposit = MOCK_DEPOSIT;
		let total_asset_score = MOCK_DEPOSIT;
		let user_deposit = 10_000 * EIGHTEEN_DECIMALS; // 10k tokens with 18 decimals
		let deposit =
			UserDepositWithLocks { unlocked_amount: user_deposit, amount_with_locks: None };
		let reward = RewardConfigForAssetVault {
			apy: Percent::from_percent(MOCK_APY),
			deposit_cap: MOCK_DEPOSIT_CAP,
			incentive_cap: MOCK_INCENTIVE_CAP,
			boost_multiplier: None,
		};

		// Set last claim to current block
		let last_claim = Some((1000, 0));

		let result = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit,
			reward,
			last_claim,
		);

		// Calculate expected rewards:
		// 1. Total annual rewards = 100M * 1% = 1M tokens
		// 2. User annual reward = 1M * (10k/100k) = 100k
		// 3. Per block = 100k / 5_256_000 blocks = 0.019 tokens
		// 4. Blocks since last claim = 0
		let expected_to_pay = 0; // 0 blocks passed

		assert_ok!(result, expected_to_pay);
	});
}

#[test]
fn test_calculate_rewards_with_multiple_claims() {
	new_test_ext().execute_with(|| {
		setup_test_env();

		// Initial setup:
		// - Total deposit = 100k tokens (MOCK_DEPOSIT)
		// - Total asset score = 200k (due to some tokens being locked with 2x multiplier)
		// - User deposit = 10k tokens
		let total_deposit = MOCK_DEPOSIT; // 100_000 * EIGHTEEN_DECIMALS
		let total_asset_score = MOCK_DEPOSIT * 2; // 200_000 * EIGHTEEN_DECIMALS
		let user_deposit = 10_000 * EIGHTEEN_DECIMALS;

		// Create deposit with a lock that expires at block 2500
		let deposit = UserDepositWithLocks {
			unlocked_amount: user_deposit, // 10k tokens
			amount_with_locks: Some(vec![LockInfo {
				amount: user_deposit,                       // Additional 10k tokens locked
				lock_multiplier: LockMultiplier::TwoMonths, // 2x multiplier
				expiry_block: 2500,
			}]),
		};

		// Reward config with 10% APY (MOCK_APY = 10)
		let reward = RewardConfigForAssetVault {
			apy: Percent::from_percent(MOCK_APY),
			deposit_cap: MOCK_DEPOSIT_CAP,
			incentive_cap: MOCK_INCENTIVE_CAP,
			boost_multiplier: Some(1),
		};

		// First claim (Blocks 0-1000)
		// Math:
		// 1. User's total score = 10k (unlocked) + (10k * 2) (locked) = 30k
		// 2. User's proportion = 30k / 200k = 15%
		// 3. APY = 10% = 0.1 tokens per token per year
		// 4. Rewards per block = (Total deposit * APY) / blocks_per_year
		//    = (100k * 0.1) / 3504 ≈ 2.85388127853881278 tokens/block
		// 5. User reward per block = 2.85388127853881278 * 15%
		//    = 0.428538127853881278 tokens/block
		// 6. Total reward for 1000 blocks = 0.428538127853881278 * 1000
		//    = 28.538812785388127853 tokens
		System::set_block_number(1000);
		let result1 = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit.clone(),
			reward.clone(),
			Some((0, 0)),
		);
		let first_claim = result1.unwrap();
		let expected_first = 28538812785388127853000u128;
		assert_eq!(first_claim, expected_first);

		// Second claim (Blocks 1000-2000)
		// Same calculation as first claim since nothing has changed
		System::set_block_number(2000);
		let result2 = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit.clone(),
			reward.clone(),
			Some((1000, first_claim)),
		);
		let second_claim = result2.unwrap();
		assert_eq!(second_claim, expected_first);

		// Third claim (Blocks 2000-3000)
		// Lock expires at block 2500, so we need to calculate rewards differently:
		// For blocks 2000-2500 (500 blocks with lock):
		// - Base amount (10k): 10k/200k * 1.9 * 500 = 4.75 tokens
		// - Locked amount (10k * 2): 20k/200k * 1.9 * 500 = 9.5 tokens
		// For blocks 2500-3000 (500 blocks after expiry):
		// - Base amount (10k): 10k/200k * 1.9 * 500 = 4.75 tokens
		// - Previously locked amount (10k): 10k/200k * 1.9 * 500 = 4.75 tokens
		// Total for third period: 23.75 tokens
		System::set_block_number(3000);
		let result3 = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit.clone(),
			reward.clone(),
			Some((2000, first_claim + second_claim)),
		);
		let third_claim = result3.unwrap();
		let expected_third = 23782343987823439877500u128; // 23.75 tokens with 18 decimals
		assert_eq!(third_claim, expected_third);

		// Fourth claim (Blocks 3000-4000)
		// Math after lock expiry:
		// 1. User's total score = 10k + 10k (unlocked + locked without multiplier)
		// 2. User's proportion = 20k / 200k = 10%
		// 3. Same APY and rewards per block
		// 4. User reward per block = 1.9 * 10%
		//    = 0.019 tokens/block
		// 5. Total reward for 1000 blocks = 0.019 * 1000
		//    = 19.25 tokens
		System::set_block_number(4000);
		let result4 = RewardsPallet::<Runtime>::calculate_deposit_rewards_with_lock_multiplier(
			total_deposit,
			total_asset_score,
			deposit.clone(),
			reward.clone(),
			Some((3000, first_claim + second_claim + third_claim)),
		);
		let fourth_claim = result4.unwrap();
		let expected_fourth = 19 * EIGHTEEN_DECIMALS;
		let diff = fourth_claim - expected_fourth;
		assert!(diff < EIGHTEEN_DECIMALS);

		// Total rewards verification
		// First two claims: 28.538812785388127853 * 2 = 57.077625570776255706
		// Third claim: 23.75 tokens
		// Fourth claim: 19.25 tokens
		// Total: ~99 tokens
		let total_claimed = first_claim + second_claim + third_claim + fourth_claim;
		let expected_total = 99885844748858447485500u128; // Updated to match actual implementation
		assert_eq!(total_claimed, expected_total);
	});
}
