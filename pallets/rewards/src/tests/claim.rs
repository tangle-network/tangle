use crate::{
    mock::*,
    tests::reward_calc::setup_test_env,
    AssetAction, Error, Pallet as RewardsPallet, TotalRewardVaultDeposit, TotalRewardVaultScore,
};
use crate::BalanceOf;
use crate::UserClaimedReward;
use frame_support::{assert_err, assert_ok, traits::Currency};
use sp_runtime::Percent;
use tangle_primitives::{
    services::Asset,
    types::rewards::{LockInfo, LockMultiplier, UserDepositWithLocks},
};
use crate::RewardConfigForAssetVault;
use frame_support::assert_noop;

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

fn setup_vault(account: AccountId, vault_id: u32, asset: Asset<u128>) -> Result<(), Error<Runtime>> {
    // Setup test environment
    setup_test_env();

    // Fund rewards pallet with initial balance
    let rewards_account = RewardsPallet::<Runtime>::account_id();
    let initial_funding = 1_000_000_000_000_000_000_000u128; // 1M tokens
    Balances::make_free_balance_be(&rewards_account, initial_funding);

    // Set total issuance for APY calculations
    pallet_balances::TotalIssuance::<Runtime>::set(MOCK_TOTAL_ISSUANCE);

    // Configure the reward vault
    assert_ok!(RewardsPallet::<Runtime>::update_vault_reward_config(
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
            UserDepositWithLocks {
                unlocked_amount: MOCK_DEPOSIT,
                amount_with_locks: None,
            },
        );
    });

    // Set total deposit and total score for the vault
    TotalRewardVaultDeposit::<Runtime>::insert(vault_id, MOCK_DEPOSIT);
    TotalRewardVaultScore::<Runtime>::insert(vault_id, MOCK_DEPOSIT);

	// set last claim to zero
	let default_balance : BalanceOf<Runtime> = 0_u32.into();
	UserClaimedReward::<Runtime>::insert(
		account,
		vault_id,
		(0, default_balance),
	);

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
        assert!(balance > 0);

        // Verify approximate expected rewards (19 tokens with some precision loss)
        let expected_reward = 19 * EIGHTEEN_DECIMALS;
        let diff = if balance > expected_reward {
            balance - expected_reward
        } else {
            expected_reward - balance
        };
        assert!(diff < EIGHTEEN_DECIMALS); // Less than 1 token difference
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

        // Expected around 28 tokens with some precision loss
        let expected_reward = 28 * EIGHTEEN_DECIMALS;
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
        let expected_reward = 46 * EIGHTEEN_DECIMALS; // ~46% of annual rewards for 1000 blocks
        let diff = if balance > expected_reward {
            balance - expected_reward
        } else {
            expected_reward - balance
        };
        assert!(diff < EIGHTEEN_DECIMALS);
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
                    amount_with_locks: Some(vec![
                        LockInfo {
                            amount: user_deposit,
                            lock_multiplier: LockMultiplier::TwoMonths,
                            expiry_block: 2000,
                        },
                    ]),
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

        assert_ok!(RewardsPallet::<Runtime>::update_vault_reward_config(
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
            Error::<Runtime>::ArithmeticError
        );
    });
}
