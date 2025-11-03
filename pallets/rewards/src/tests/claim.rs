use crate::{
	AssetAction, BalanceOf, DecayRate, DecayStartPeriod, Error, Pallet as RewardsPallet,
	RewardConfigForAssetVault, TotalRewardVaultDeposit, TotalRewardVaultScore, UserClaimedReward,
	mock::*, tests::reward_calc::setup_test_env,
};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_runtime::Perbill;
use tangle_primitives::{
	services::Asset,
	traits::RewardRecorder,
	types::rewards::{LockInfo, LockMultiplier},
};

// Mock values for consistent testing
const EIGHTEEN_DECIMALS: u128 = 1_000_000_000_000_000_000_000;
const MOCK_DEPOSIT_CAP: u128 = 1_000_000 * EIGHTEEN_DECIMALS; // 1M tokens with 18 decimals
const MOCK_TOTAL_ISSUANCE: u128 = 100_000_000 * EIGHTEEN_DECIMALS; // 100M tokens with 18 decimals
const MOCK_INCENTIVE_CAP: u128 = 10_000 * EIGHTEEN_DECIMALS; // 10k tokens with 18 decimals
const MOCK_APY: u32 = 10; // 10% APY
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
			apy: Perbill::from_percent(MOCK_APY),
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
	insert_user_deposit(account.clone(), asset, MOCK_DEPOSIT, None);

	// Set total deposit and total score for the vault
	TotalRewardVaultDeposit::<Runtime>::insert(vault_id, MOCK_DEPOSIT);
	TotalRewardVaultScore::<Runtime>::insert(vault_id, MOCK_DEPOSIT);

	// set last claim to zero
	let default_balance: BalanceOf<Runtime> = 0_u32.into();
	UserClaimedReward::<Runtime>::insert(account, vault_id, (0, default_balance));

	// Finally fund the pot account with rewards
	let vault_pot_account = RewardsPallet::<Runtime>::reward_vaults_pot_account(vault_id)
		.expect("Vault pot account not found");
	let initial_funding = Perbill::from_percent(MOCK_APY) * MOCK_TOTAL_ISSUANCE;
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
		insert_user_deposit(account.clone(), asset, 0, None);

		// Try to claim rewards for the account with zero deposit - should fail
		assert_noop!(
			RewardsPallet::<Runtime>::claim_rewards_other(
				RuntimeOrigin::signed(account.clone()),
				account.clone(),
				asset
			),
			Error::<Runtime>::NoRewardsAvailable
		);

		// Try to claim rewards again with zero deposit - should still fail
		assert_noop!(
			RewardsPallet::<Runtime>::claim_rewards_other(
				RuntimeOrigin::signed(account.clone()),
				account.clone(),
				asset
			),
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
		insert_user_deposit(account.clone(), asset, user_deposit, None);

		// Initial balance should be 0
		assert_eq!(Balances::free_balance(&account), 0);

		// Run to block 1000
		run_to_block(1000);

		// Try to claim rewards for the account
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards_other(
			RuntimeOrigin::signed(account.clone()),
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

#[test]
fn test_claim_rewards_with_expired_lock() {
	new_test_ext().execute_with(|| {
		let account: AccountId = AccountId::new([1u8; 32]);
		let vault_id = 1u32;
		let asset = Asset::Custom(1);
		let user_deposit = 10_000 * EIGHTEEN_DECIMALS;

		setup_vault(account.clone(), vault_id, asset).unwrap();

		// Mock deposit with expired lock
		insert_user_deposit(
			account.clone(),
			asset,
			user_deposit,
			Some(vec![LockInfo {
				amount: user_deposit,
				lock_multiplier: LockMultiplier::TwoMonths,
				expiry_block: 900,
			}]),
		);

		// Run to block 1000 (after lock expiry)
		run_to_block(1000);

		// Try to claim rewards for the account
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards_other(
			RuntimeOrigin::signed(account.clone()),
			account.clone(),
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
		insert_user_deposit(
			account.clone(),
			asset,
			user_deposit,
			Some(vec![
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
		);

		// Run to block 1000
		run_to_block(1000);

		// Try to claim rewards for the account
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards_other(
			RuntimeOrigin::signed(account.clone()),
			account.clone(),
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
		insert_user_deposit(
			account.clone(),
			asset,
			user_deposit,
			Some(vec![LockInfo {
				amount: user_deposit,
				lock_multiplier: LockMultiplier::TwoMonths,
				expiry_block: 2000,
			}]),
		);

		// First claim at block 1000
		run_to_block(1000);
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards_other(
			RuntimeOrigin::signed(account.clone()),
			account.clone(),
			asset
		));
		let first_claim_balance = Balances::free_balance(&account);

		// Second claim at block 1500
		run_to_block(1500);
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards_other(
			RuntimeOrigin::signed(account.clone()),
			account.clone(),
			asset
		));
		let second_claim_balance = Balances::free_balance(&account);

		// Verify that second claim added more rewards
		assert!(second_claim_balance > first_claim_balance);

		// Verify that claiming in the same block gives no rewards
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards_other(
			RuntimeOrigin::signed(account.clone()),
			account.clone(),
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
				apy: Perbill::from_percent(MOCK_APY),
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
		insert_user_deposit(account.clone(), asset, user_deposit, None);

		run_to_block(1000);

		// Should not be able to claim rewards with zero incentive cap
		assert_noop!(
			RewardsPallet::<Runtime>::claim_rewards_other(
				RuntimeOrigin::signed(account.clone()),
				account.clone(),
				asset
			),
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
				apy: Perbill::from_percent(MOCK_APY),
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
		insert_user_deposit(frequent_claimer.clone(), asset, deposit_amount, None);

		// Mock deposit for infrequent claimer
		insert_user_deposit(infrequent_claimer.clone(), asset, deposit_amount, None);

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
		let initial_funding = Perbill::from_percent(MOCK_APY) * MOCK_TOTAL_ISSUANCE * 2; // Double funding to ensure enough rewards
		Balances::make_free_balance_be(&vault_pot_account, initial_funding);

		// Set total issuance for APY calculations
		pallet_balances::TotalIssuance::<Runtime>::set(MOCK_TOTAL_ISSUANCE);

		// Set decay to start after 30 days (144000 blocks) with 5% decay
		DecayStartPeriod::<Runtime>::set(144_000);
		// decay rate to counteract 1% permonth inflation
		DecayRate::<Runtime>::set(Perbill::from_percent(10));

		let blocks_per_month = 144_000_u64;
		let total_months = 10;
		let mut current_block = 1000;

		// Frequent claimer claims every month for 10 months
		let frequent_starting_balance = Balances::free_balance(&frequent_claimer);
		for _ in 0..total_months {
			System::set_block_number(current_block + blocks_per_month);
			current_block += blocks_per_month;

			assert_ok!(RewardsPallet::<Runtime>::claim_rewards_other(
				RuntimeOrigin::signed(frequent_claimer.clone()),
				frequent_claimer.clone(),
				asset
			));

			// simulate inflation, 1% per month
			let supply = pallet_balances::TotalIssuance::<Runtime>::get();
			let inflation = Perbill::from_percent(1).mul_floor(supply);
			pallet_balances::TotalIssuance::<Runtime>::set(supply + inflation);
		}
		let frequent_total_rewards =
			Balances::free_balance(&frequent_claimer) - frequent_starting_balance;

		// Infrequent claimer claims after 10 months
		let infrequent_starting_balance = Balances::free_balance(&infrequent_claimer);
		System::set_block_number(blocks_per_month * total_months);
		assert_ok!(RewardsPallet::<Runtime>::claim_rewards_other(
			RuntimeOrigin::signed(infrequent_claimer.clone()),
			infrequent_claimer.clone(),
			asset
		));
		let infrequent_total_rewards =
			Balances::free_balance(&infrequent_claimer) - infrequent_starting_balance;

		let difference = frequent_total_rewards.saturating_sub(infrequent_total_rewards);
		let difference_perbill = (difference / frequent_total_rewards) * 100;
		assert!(difference_perbill < 1);
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
		insert_user_deposit(account.clone(), asset, user_deposit, None);

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

#[test]
fn test_update_apy_blocks() {
	new_test_ext_raw_authorities().execute_with(|| {
		// Try updating APY blocks with non-root (should fail)
		assert_noop!(
			RewardsPallet::<Runtime>::update_apy_blocks(
				RuntimeOrigin::signed(AccountId::new([1u8; 32])),
				1000
			),
			sp_runtime::DispatchError::BadOrigin,
		);

		// Update APY blocks with root (should succeed)
		assert_ok!(RewardsPallet::<Runtime>::update_apy_blocks(RuntimeOrigin::root(), 1000));

		// Verify the storage was updated
		assert_eq!(RewardsPallet::<Runtime>::blocks_for_apy(), 1000);

		// Update to a different value
		assert_ok!(RewardsPallet::<Runtime>::update_apy_blocks(RuntimeOrigin::root(), 2000));
		assert_eq!(RewardsPallet::<Runtime>::blocks_for_apy(), 2000);
	});
}

// ═══════════════════════════════════════════════════════════════════════════
// DELEGATOR REWARD TESTS WITH COMMISSION SPLIT
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_commission_split_on_reward_recording() {
	new_test_ext().execute_with(|| {
		let operator: AccountId = AccountId::new([1u8; 32]);
		let service_id = 0u64;
		let payment = 1000u128;

		let pricing_model = tangle_primitives::services::PricingModel::default();

		// Record a reward for the operator
		assert_ok!(RewardsPallet::<Runtime>::record_reward(
			&operator,
			service_id,
			payment,
			&pricing_model
		));

		// Verify commission was recorded (15% of 1000 = 150)
		let pending = RewardsPallet::<Runtime>::pending_operator_rewards(&operator);
		assert!(!pending.is_empty());
		assert_eq!(pending.len(), 1);
		assert_eq!(pending[0].1, 150); // 15% commission

		// Verify pool was updated with remaining 85% (850)
		let pool = RewardsPallet::<Runtime>::operator_reward_pools(&operator);
		// Pool accumulator should be 850 / 0 = undefined, so pool should have zero total_staked
		// This is expected when no delegators exist yet
		assert_eq!(pool.total_staked, 0);
	});
}

#[test]
fn test_delegator_reward_distribution_proportional() {
	new_test_ext().execute_with(|| {
		let operator: AccountId = AccountId::new([1u8; 32]);
		let delegator_a: AccountId = AccountId::new([2u8; 32]);
		let delegator_b: AccountId = AccountId::new([3u8; 32]);

		// Initialize delegator debts with different stake amounts (60/40 split)
		assert_ok!(RewardsPallet::<Runtime>::init_delegator_reward_debt(
			&delegator_a,
			&operator,
			600u128
		));
		assert_ok!(RewardsPallet::<Runtime>::init_delegator_reward_debt(
			&delegator_b,
			&operator,
			400u128
		));

		// Verify pool total stake
		let pool = RewardsPallet::<Runtime>::operator_reward_pools(&operator);
		assert_eq!(pool.total_staked, 1000);

		// Record a reward (commission + pool distribution)
		let service_id = 0u64;
		let payment = 1000u128;
		let pricing_model = tangle_primitives::services::PricingModel::default();
		assert_ok!(RewardsPallet::<Runtime>::record_reward(
			&operator,
			service_id,
			payment,
			&pricing_model
		));

		// Calculate pending rewards for each delegator
		let pending_a =
			RewardsPallet::<Runtime>::calculate_pending_delegator_rewards(&delegator_a, &operator);
		let pending_b =
			RewardsPallet::<Runtime>::calculate_pending_delegator_rewards(&delegator_b, &operator);

		assert_ok!(&pending_a);
		assert_ok!(&pending_b);

		// Verify proportional distribution of the 85% pool (850 tokens)
		// Delegator A should get 60% of 850 = 510
		// Delegator B should get 40% of 850 = 340
		assert_eq!(pending_a.unwrap(), 510);
		assert_eq!(pending_b.unwrap(), 340);
	});
}

#[test]
fn test_operator_receives_commission_plus_pool_share() {
	new_test_ext().execute_with(|| {
		let operator: AccountId = AccountId::new([1u8; 32]);
		let service_id = 0u64;
		let payment = 1000u128;

		// Operator self-delegates 600, delegator has 400 (60/40 split)
		assert_ok!(RewardsPallet::<Runtime>::init_delegator_reward_debt(
			&operator, &operator, 600u128
		));

		let delegator: AccountId = AccountId::new([2u8; 32]);
		assert_ok!(RewardsPallet::<Runtime>::init_delegator_reward_debt(
			&delegator, &operator, 400u128
		));

		// Fund the rewards pallet account for transfers
		let rewards_account = RewardsPallet::<Runtime>::account_id();
		Balances::make_free_balance_be(&rewards_account, 10_000u128);

		// Record reward
		let pricing_model = tangle_primitives::services::PricingModel::default();
		assert_ok!(RewardsPallet::<Runtime>::record_reward(
			&operator,
			service_id,
			payment,
			&pricing_model
		));

		// Verify operator's commission (15% of 1000 = 150)
		let pending_commission = RewardsPallet::<Runtime>::pending_operator_rewards(&operator);
		assert!(!pending_commission.is_empty());
		assert_eq!(pending_commission[0].1, 150);

		// Verify operator's pool share (60% of 850 = 510)
		let pool_share =
			RewardsPallet::<Runtime>::calculate_pending_delegator_rewards(&operator, &operator);
		assert_ok!(&pool_share);
		assert_eq!(pool_share.unwrap(), 510);

		// Total operator earnings should be 150 + 510 = 660
		// This is approximately 66% of the original 1000 payment
		// Operator has 60% stake but gets extra 15% commission = ~66% total
	});
}

#[test]
fn test_delegator_only_receives_pool_share_no_commission() {
	new_test_ext().execute_with(|| {
		let operator: AccountId = AccountId::new([1u8; 32]);
		let delegator: AccountId = AccountId::new([2u8; 32]);
		let service_id = 0u64;
		let payment = 1000u128;

		// Setup delegation: operator has 600, delegator has 400
		assert_ok!(RewardsPallet::<Runtime>::init_delegator_reward_debt(
			&operator, &operator, 600u128
		));
		assert_ok!(RewardsPallet::<Runtime>::init_delegator_reward_debt(
			&delegator, &operator, 400u128
		));

		// Record reward
		let pricing_model = tangle_primitives::services::PricingModel::default();
		assert_ok!(RewardsPallet::<Runtime>::record_reward(
			&operator,
			service_id,
			payment,
			&pricing_model
		));

		// Verify delegator has NO commission rewards
		let delegator_commission = RewardsPallet::<Runtime>::pending_operator_rewards(&delegator);
		assert!(delegator_commission.is_empty());

		// Verify delegator only has pool share (40% of 850 = 340)
		let pool_share =
			RewardsPallet::<Runtime>::calculate_pending_delegator_rewards(&delegator, &operator);
		assert_ok!(&pool_share);
		assert_eq!(pool_share.unwrap(), 340);
	});
}

#[test]
fn test_claim_delegator_rewards_updates_balance() {
	new_test_ext().execute_with(|| {
		let operator: AccountId = AccountId::new([1u8; 32]);
		let delegator: AccountId = AccountId::new([2u8; 32]);
		let service_id = 0u64;
		let payment = 1000u128;

		// Setup delegation
		assert_ok!(RewardsPallet::<Runtime>::init_delegator_reward_debt(
			&delegator, &operator, 1000u128 // 100% stake for simplicity
		));

		// Fund the rewards pallet
		let rewards_account = RewardsPallet::<Runtime>::account_id();
		Balances::make_free_balance_be(&rewards_account, 10_000u128);

		// Record reward
		let pricing_model = tangle_primitives::services::PricingModel::default();
		assert_ok!(RewardsPallet::<Runtime>::record_reward(
			&operator,
			service_id,
			payment,
			&pricing_model
		));

		// Get delegator's initial balance
		let balance_before = Balances::free_balance(&delegator);

		// Claim delegator rewards
		let claimed =
			RewardsPallet::<Runtime>::calculate_and_claim_delegator_rewards(&delegator, &operator);
		assert_ok!(&claimed);

		// Verify balance increased by pool share (85% of 1000 = 850)
		let balance_after = Balances::free_balance(&delegator);
		assert_eq!(balance_after - balance_before, 850);
		assert_eq!(claimed.unwrap(), 850);

		// Verify pending rewards are now zero after claim
		let pending_after =
			RewardsPallet::<Runtime>::calculate_pending_delegator_rewards(&delegator, &operator);
		assert_ok!(&pending_after);
		assert_eq!(pending_after.unwrap(), 0);
	});
}

#[test]
fn test_multiple_rewards_accumulate_in_pool() {
	new_test_ext().execute_with(|| {
		let operator: AccountId = AccountId::new([1u8; 32]);
		let delegator: AccountId = AccountId::new([2u8; 32]);

		// Setup delegation (50/50 split)
		assert_ok!(RewardsPallet::<Runtime>::init_delegator_reward_debt(
			&operator, &operator, 500u128
		));
		assert_ok!(RewardsPallet::<Runtime>::init_delegator_reward_debt(
			&delegator, &operator, 500u128
		));

		// Record multiple rewards
		let pricing_model = tangle_primitives::services::PricingModel::default();
		for service_id in 0..3 {
			assert_ok!(RewardsPallet::<Runtime>::record_reward(
				&operator,
				service_id,
				100u128,
				&pricing_model
			));
		}

		// Total rewards: 3 × 100 = 300
		// Commission per reward: 15 (total 45)
		// Pool per reward: 85 (total 255)
		// Each delegator should get 50% of 255 = 127.5 ≈ 127

		let delegator_pending =
			RewardsPallet::<Runtime>::calculate_pending_delegator_rewards(&delegator, &operator);
		assert_ok!(&delegator_pending);
		assert_eq!(delegator_pending.unwrap(), 127); // 50% of 255

		// Verify operator has both commission and pool share
		let operator_commission = RewardsPallet::<Runtime>::pending_operator_rewards(&operator);
		assert!(!operator_commission.is_empty());
		let total_commission: u128 = operator_commission.iter().map(|r| r.1).sum();
		assert_eq!(total_commission, 45); // 3 × 15

		let operator_pool =
			RewardsPallet::<Runtime>::calculate_pending_delegator_rewards(&operator, &operator);
		assert_ok!(&operator_pool);
		assert_eq!(operator_pool.unwrap(), 127); // 50% of 255
	});
}

#[test]
fn test_delegator_joins_mid_period_no_historical_rewards() {
	new_test_ext().execute_with(|| {
		let operator: AccountId = AccountId::new([1u8; 32]);
		let early_delegator: AccountId = AccountId::new([2u8; 32]);
		let late_delegator: AccountId = AccountId::new([3u8; 32]);

		// Early delegator joins with 100% stake
		assert_ok!(RewardsPallet::<Runtime>::init_delegator_reward_debt(
			&early_delegator,
			&operator,
			100u128
		));

		// Record first reward (early delegator gets 100%)
		let pricing_model = tangle_primitives::services::PricingModel::default();
		assert_ok!(RewardsPallet::<Runtime>::record_reward(
			&operator,
			0u64,
			1000u128,
			&pricing_model
		));

		// Late delegator joins (now 50/50 split)
		assert_ok!(RewardsPallet::<Runtime>::init_delegator_reward_debt(
			&late_delegator,
			&operator,
			100u128
		));

		// Record second reward (both get 50%)
		assert_ok!(RewardsPallet::<Runtime>::record_reward(
			&operator,
			1u64,
			1000u128,
			&pricing_model
		));

		// Early delegator should have:
		// - 100% of first 850 = 850
		// - 50% of second 850 = 425
		// - Total = 1275
		let early_pending = RewardsPallet::<Runtime>::calculate_pending_delegator_rewards(
			&early_delegator,
			&operator,
		);
		assert_ok!(&early_pending);
		assert_eq!(early_pending.unwrap(), 1275);

		// Late delegator should have:
		// - 0 from first reward (not delegated yet)
		// - 50% of second 850 = 425
		// - Total = 425
		let late_pending = RewardsPallet::<Runtime>::calculate_pending_delegator_rewards(
			&late_delegator,
			&operator,
		);
		assert_ok!(&late_pending);
		assert_eq!(late_pending.unwrap(), 425);
	});
}
