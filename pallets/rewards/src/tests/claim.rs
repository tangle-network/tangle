use crate::{
	mock::*, tests::reward_calc::setup_test_env, AssetAction, BalanceOf, DecayRate,
	DecayStartPeriod, Error, Pallet as RewardsPallet, RewardConfigForAssetVault,
	TotalRewardVaultDeposit, TotalRewardVaultScore, UserClaimedReward,
};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_runtime::Percent;
use tangle_primitives::{
	rewards::UserDepositWithLocks,
	services::Asset,
	types::rewards::{LockInfo, LockMultiplier},
};

// Mock values for consistent testing
const EIGHTEEN_DECIMALS: u128 = 1_000_000_000_000_000_000_000;
const MOCK_DEPOSIT_CAP: u128 = 1_000_000 * EIGHTEEN_DECIMALS; // 1M tokens with 18 decimals
const MOCK_TOTAL_ISSUANCE: u128 = 100_000_000 * EIGHTEEN_DECIMALS; // 100M tokens with 18 decimals
const MOCK_INCENTIVE_CAP: u128 = 10_000 * EIGHTEEN_DECIMALS; // 10k tokens with 18 decimals
const MOCK_APY: u8 = 10; // 10% APY
const MOCK_DEPOSIT: u128 = 100_000 * EIGHTEEN_DECIMALS; // 100k tokens with 18 decimals

fn run_to_block(n: u64) {
	while System::block_number() < n {
		System::set_block_number(System::block_number() + 1);
	}
}

fn setup_vault(
	account: AccountId,
	vault_id: u32,
	asset: Asset<u128>,
) -> Result<(), Error<Runtime>> {
	// Setup test environment
	setup_test_env();

	// Configure the reward vault
	assert_ok!(RewardsPallet::<Runtime>::create_reward_vault(
		RuntimeOrigin::root(),
		vault_id,
		RewardConfigForAssetVault {
			apy: Percent::from_percent(MOCK_APY),
			deposit_cap: MOCK_DEPOSIT_CAP,
			incentive_cap: MOCK_INCENTIVE_CAP,
			boost_multiplier: Some(1),
		}
	));

	// Add asset to vault
	assert_ok!(RewardsPallet::<Runtime>::manage_asset_reward_vault(
		RuntimeOrigin::root(),
		vault_id,
		asset,
		AssetAction::Add,
	));

	// Set deposit in mock delegation info
	MOCK_DELEGATION_INFO.with(|m| {
		m.borrow_mut().deposits.insert(
			(account.clone(), asset),
			UserDepositWithLocks { unlocked_amount: MOCK_DEPOSIT, amount_with_locks: None },
		);
	});

	// Set total deposit and total score for the vault
	TotalRewardVaultDeposit::<Runtime>::insert(vault_id, MOCK_DEPOSIT);
	TotalRewardVaultScore::<Runtime>::insert(vault_id, MOCK_DEPOSIT);

	// set last claim to zero
	let default_balance: BalanceOf<Runtime> = 0_u32.into();
	UserClaimedReward::<Runtime>::insert(account, vault_id, (0, default_balance));

	// Finally fund the pot account with rewards
	let vault_pot_account = RewardsPallet::<Runtime>::reward_vaults_pot_account(vault_id)
		.expect("Vault pot account not found");
	let initial_funding = Percent::from_percent(MOCK_APY) * MOCK_TOTAL_ISSUANCE;
	Balances::make_free_balance_be(&vault_pot_account, initial_funding);

	// Set total issuance for APY calculations
	pallet_balances::TotalIssuance::<Runtime>::set(MOCK_TOTAL_ISSUANCE);

	Ok(())
}

#[test]
fn test_claim_rewards_zero_deposit() {
	new_test_ext().execute_with(|| {
		let account: AccountId = AccountId::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(1);

		setup_vault(account.clone(), vault_id, asset).unwrap();

		// Mock deposit with zero amount
		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert(
				(account.clone(), asset),
				UserDepositWithLocks { unlocked_amount: 0, amount_with_locks: None },
			);
		});

		// Try to claim rewards with zero deposit
		assert_noop!(
			RewardsPallet::<Runtime>::claim_rewards(RuntimeOrigin::signed(account.clone()), asset),
			Error::<Runtime>::NoRewardsAvailable
		);
	});
}

#[test]
fn test_claim_rewards_only_unlocked() {
	new_test_ext().execute_with(|| {
		let account: AccountId = AccountId::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(1);
		let user_deposit = 10_000 * EIGHTEEN_DECIMALS; // 10k tokens

		setup_vault(account.clone(), vault_id, asset).unwrap();

		// Mock deposit with only unlocked amount
		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert(
				(account.clone(), asset),
				UserDepositWithLocks { unlocked_amount: user_deposit, amount_with_locks: None },
			);
		});

		// Initial balance should be 0
		assert_eq!(Balances::free_balance(&account), 0);

		// Run to block 1000
		run_to_block(1000);

		// Claim rewards
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards(
			RuntimeOrigin::signed(account.clone()),
			asset
		));

		// Check that rewards were received
		let balance = Balances::free_balance(&account);

		// Verify approximate expected rewards (19 tokens with some precision loss)
		let expected_reward = 191 * EIGHTEEN_DECIMALS / 10;
		let diff = if balance > expected_reward {
			balance - expected_reward
		} else {
			expected_reward - balance
		};
		println!("diff: {:?} {:?}", diff, diff / EIGHTEEN_DECIMALS);
		assert!(diff <= 2 * EIGHTEEN_DECIMALS);
	});
}

#[test]
fn test_claim_rewards_with_expired_lock() {
	new_test_ext().execute_with(|| {
		let account: AccountId = AccountId::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(1);
		let user_deposit = 10_000 * EIGHTEEN_DECIMALS;

		setup_vault(account.clone(), vault_id, asset).unwrap();

		// Mock deposit with expired lock
		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert(
				(account.clone(), asset),
				UserDepositWithLocks {
					unlocked_amount: user_deposit,
					amount_with_locks: Some(vec![LockInfo {
						amount: user_deposit,
						lock_multiplier: LockMultiplier::TwoMonths,
						expiry_block: 900,
					}]),
				},
			);
		});

		// Run to block 1000 (after lock expiry)
		run_to_block(1000);

		// Claim rewards
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards(
			RuntimeOrigin::signed(account.clone()),
			asset
		));

		// Verify rewards
		let balance = Balances::free_balance(&account);
		assert!(balance > 0);

		// Expected rewards should reflect the lock multipliers
		// Total TNT in system = 100M
		// APY = 10%
		// deposit_cap = 1M
		// blocks = 1000
		// user deposit = 10k
		// user score with locks = 2x20k + 3x30k + 10k = 140k
		// Effective APY = total_deposit / deposit_cap * apy = 1%
		// Expected reward = 100M * 1% = 1M
		// Rewards per block = Expected reward / 5_256_000 = 1M / 5_256_000 = 0.1902587519
		// Claiming for block 1000
		// reward for unlocked 10k = 0.01902587519 * 1000 = 19.2587519
		// reward for locked 10k = 0.038051750761035007610 * 900 = 34.246575342465753424500
		// reward for expired locked 10k = 0.01902587519 * 100 = 1.92587519
		let expected_reward =
			19 * EIGHTEEN_DECIMALS + 34 * EIGHTEEN_DECIMALS + 2 * EIGHTEEN_DECIMALS;
		let diff = if balance > expected_reward {
			balance - expected_reward
		} else {
			expected_reward - balance
		};
		assert!(diff < EIGHTEEN_DECIMALS);
	});
}

#[test]
fn test_claim_rewards_with_active_locks() {
	new_test_ext().execute_with(|| {
		let account: AccountId = AccountId::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(1);
		let user_deposit = 10_000 * EIGHTEEN_DECIMALS;

		setup_vault(account.clone(), vault_id, asset).unwrap();

		// Mock deposit with active locks
		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert(
				(account.clone(), asset),
				UserDepositWithLocks {
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
				},
			);
		});

		// Run to block 1000
		run_to_block(1000);

		// Claim rewards
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards(
			RuntimeOrigin::signed(account.clone()),
			asset
		));

		// Check rewards
		let balance = Balances::free_balance(&account);
		assert!(balance > 0);

		// Expected rewards should reflect the lock multipliers
		// Total TNT in system = 100M
		// APY = 10%
		// deposit_cap = 1M
		// blocks = 1000
		// user deposit = 10k
		// user score with locks = 2x20k + 3x30k + 10k = 140k
		// Effective APY = total_deposit / deposit_cap * apy = 1%
		// Expected reward = 100M * 1% = 1M
		// Rewards per block = Expected reward / 5_256_000 = 1M / 5_256_000 = 0.1902587519
		// Claiming for block 1000
		// reward for unlocked 10k = 0.01902587519 * 1000 = 19.2587519
		// reward for locked 40k = 0.076103500761035007610 * 1000 = 76.103500761035007610
		// reward for locked 90k = 0.171232876712328767122 * 1000 = 171.232876712328767122
		let expected_reward =
			19 * EIGHTEEN_DECIMALS + 76 * EIGHTEEN_DECIMALS + 171 * EIGHTEEN_DECIMALS;
		let diff = if balance > expected_reward {
			balance - expected_reward
		} else {
			expected_reward - balance
		};
		println!("diff {:?} {:?}", diff, diff / EIGHTEEN_DECIMALS);
		assert!(diff < 2 * EIGHTEEN_DECIMALS); // allow for 1TNT precision loss
	});
}

#[test]
fn test_claim_rewards_multiple_claims() {
	new_test_ext().execute_with(|| {
		let account: AccountId = AccountId::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(1);
		let user_deposit = 10_000 * EIGHTEEN_DECIMALS;

		setup_vault(account.clone(), vault_id, asset).unwrap();

		// Mock deposit with active locks
		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert(
				(account.clone(), asset),
				UserDepositWithLocks {
					unlocked_amount: user_deposit,
					amount_with_locks: Some(vec![LockInfo {
						amount: user_deposit,
						lock_multiplier: LockMultiplier::TwoMonths,
						expiry_block: 2000,
					}]),
				},
			);
		});

		// First claim at block 1000
		run_to_block(1000);
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards(
			RuntimeOrigin::signed(account.clone()),
			asset
		));
		let first_claim_balance = Balances::free_balance(&account);

		// Second claim at block 1500
		run_to_block(1500);
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards(
			RuntimeOrigin::signed(account.clone()),
			asset
		));
		let second_claim_balance = Balances::free_balance(&account);

		// Verify that second claim added more rewards
		assert!(second_claim_balance > first_claim_balance);

		// Verify that claiming in the same block gives no rewards
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards(
			RuntimeOrigin::signed(account.clone()),
			asset
		));
		assert_eq!(Balances::free_balance(&account), second_claim_balance);
	});
}

#[test]
fn test_claim_rewards_with_zero_cap() {
	new_test_ext().execute_with(|| {
		let account: AccountId = AccountId::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(1);
		let user_deposit = 10_000 * EIGHTEEN_DECIMALS;

		// Setup vault with zero incentive cap
		let rewards_account = RewardsPallet::<Runtime>::account_id();
		Balances::make_free_balance_be(&rewards_account, MOCK_TOTAL_ISSUANCE);

		assert_ok!(RewardsPallet::<Runtime>::create_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			RewardConfigForAssetVault {
				apy: Percent::from_percent(MOCK_APY),
				deposit_cap: MOCK_DEPOSIT_CAP,
				incentive_cap: 0, // Zero incentive cap
				boost_multiplier: Some(1),
			}
		));

		assert_ok!(RewardsPallet::<Runtime>::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			asset,
			AssetAction::Add,
		));

		// Mock deposit
		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert(
				(account.clone(), asset),
				UserDepositWithLocks { unlocked_amount: user_deposit, amount_with_locks: None },
			);
		});

		run_to_block(1000);

		// Should not be able to claim rewards with zero incentive cap
		assert_noop!(
			RewardsPallet::<Runtime>::claim_rewards(RuntimeOrigin::signed(account.clone()), asset),
			Error::<Runtime>::CannotCalculateRewardPerBlock
		);
	});
}

#[test]
fn test_claim_frequency_with_decay() {
	new_test_ext().execute_with(|| {
		let frequent_claimer = AccountId::new([1u8; 32]);
		let infrequent_claimer = AccountId::new([2u8; 32]);
		let deposit_amount = 10_000 * EIGHTEEN_DECIMALS;
		let asset = Asset::Custom(1);
		let vault_id = 1u32;

		setup_test_env();

		// Configure the reward vault
		assert_ok!(RewardsPallet::<Runtime>::create_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			RewardConfigForAssetVault {
				apy: Percent::from_percent(MOCK_APY),
				deposit_cap: MOCK_DEPOSIT_CAP,
				incentive_cap: MOCK_INCENTIVE_CAP,
				boost_multiplier: Some(1),
			}
		));

		// Add asset to vault
		assert_ok!(RewardsPallet::<Runtime>::manage_asset_reward_vault(
			RuntimeOrigin::root(),
			vault_id,
			asset,
			AssetAction::Add,
		));

		// Set deposit in mock delegation info
		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert(
				(frequent_claimer.clone(), asset),
				UserDepositWithLocks { unlocked_amount: deposit_amount, amount_with_locks: None },
			);
		});

		// Mock deposit for infrequent claimer
		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert(
				(infrequent_claimer.clone(), asset),
				UserDepositWithLocks { unlocked_amount: deposit_amount, amount_with_locks: None },
			);
		});

		// Set total deposit and total score for the vault
		TotalRewardVaultDeposit::<Runtime>::insert(vault_id, MOCK_DEPOSIT * 2); // Both users
		TotalRewardVaultScore::<Runtime>::insert(vault_id, MOCK_DEPOSIT * 2); // Both users

		// Set last claim to zero
		let default_balance: BalanceOf<Runtime> = 0_u32.into();
		UserClaimedReward::<Runtime>::insert(
			frequent_claimer.clone(),
			vault_id,
			(0, default_balance),
		);
		UserClaimedReward::<Runtime>::insert(
			infrequent_claimer.clone(),
			vault_id,
			(0, default_balance),
		);

		// Fund the pot account with rewards
		let vault_pot_account = RewardsPallet::<Runtime>::reward_vaults_pot_account(vault_id)
			.expect("Vault pot account not found");
		let initial_funding = Percent::from_percent(MOCK_APY) * MOCK_TOTAL_ISSUANCE * 2; // Double funding to ensure enough rewards
		Balances::make_free_balance_be(&vault_pot_account, initial_funding);

		// Set total issuance for APY calculations
		pallet_balances::TotalIssuance::<Runtime>::set(MOCK_TOTAL_ISSUANCE);

		// Set decay to start after 30 days (144000 blocks) with 5% decay
		DecayStartPeriod::<Runtime>::set(144_000);
		// decay rate to counteract 1% permonth inflation
		DecayRate::<Runtime>::set(Percent::from_percent(10));

		let blocks_per_month = 144_000_u64;
		let total_months = 10;
		let mut current_block = 1000;

		// Frequent claimer claims every month for 10 months
		let frequent_starting_balance = Balances::free_balance(&frequent_claimer);
		for _ in 0..total_months {
			System::set_block_number(current_block + blocks_per_month);
			current_block += blocks_per_month;

			assert_ok!(RewardsPallet::<Runtime>::claim_rewards(
				RuntimeOrigin::signed(frequent_claimer.clone()),
				asset,
			));

			// simulate inflation, 1% per month
			let supply = pallet_balances::TotalIssuance::<Runtime>::get();
			let inflation = Percent::from_percent(1).mul_floor(supply);
			pallet_balances::TotalIssuance::<Runtime>::set(supply + inflation);
		}
		let frequent_total_rewards =
			Balances::free_balance(&frequent_claimer) - frequent_starting_balance;

		// Infrequent claimer claims after 10 months
		let infrequent_starting_balance = Balances::free_balance(&infrequent_claimer);
		System::set_block_number(blocks_per_month * total_months);
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards(
			RuntimeOrigin::signed(infrequent_claimer.clone()),
			asset,
		));
		let infrequent_total_rewards =
			Balances::free_balance(&infrequent_claimer) - infrequent_starting_balance;

		let difference = frequent_total_rewards.saturating_sub(infrequent_total_rewards);
		let difference_percent = (difference / frequent_total_rewards) * 100;
		assert!(difference_percent < 1);
	});
}

#[test]
fn test_claim_rewards_other() {
	new_test_ext().execute_with(|| {
		let account: AccountId = AccountId::new([1u8; 32]);
		let other_account: AccountId = AccountId::new([2u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(1);
		let user_deposit = 10_000 * EIGHTEEN_DECIMALS; // 10k tokens

		setup_vault(account.clone(), vault_id, asset).unwrap();

		// Mock deposit with only unlocked amount
		MOCK_DELEGATION_INFO.with(|m| {
			m.borrow_mut().deposits.insert(
				(account.clone(), asset),
				UserDepositWithLocks { unlocked_amount: user_deposit, amount_with_locks: None },
			);
		});

		// Initial balance should be 0
		assert_eq!(Balances::free_balance(&account), 0);

		// Run to block 1000
		run_to_block(1000);

		// Claim rewards for account from account 2
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards_other(
			RuntimeOrigin::signed(other_account.clone()),
			account.clone(),
			asset
		));

		// Check that rewards were received
		let balance = Balances::free_balance(&account);

		// Verify approximate expected rewards (19 tokens with some precision loss)
		let expected_reward = 191 * EIGHTEEN_DECIMALS / 10;
		let diff = if balance > expected_reward {
			balance - expected_reward
		} else {
			expected_reward - balance
		};
		println!("diff: {:?} {:?}", diff, diff / EIGHTEEN_DECIMALS);
		assert!(diff <= 2 * EIGHTEEN_DECIMALS);
	});
}
