use super::*;
use mock::*;
use precompile_utils::testing::*;
use sp_core::H160;
use sp_runtime::Percent;
use frame_support::assert_ok;

fn precompiles() -> TestPrecompileSet<Runtime> {
    PrecompilesValue::get()
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented() {
    for file in ["Rewards.sol"] {
        precompiles()
            .process_selectors(file, |fn_selector, fn_signature| {
                assert!(
                    DOCUMENTED_FUNCTIONS.contains(&fn_selector),
                    "documented_functions must contain {fn_selector:?} ({fn_signature})",
                );
            });
    }
}

#[test]
fn test_claim_rewards() {
    ExtBuilder::default().build().execute_with(|| {
        let vault_id = 1u32;
        let asset_id = Asset::Custom(1);
        let alice: AccountId = TestAccount::Alice.into();
        let deposit_amount = 1_000u128;

        // Setup vault and add asset
        assert_ok!(Rewards::manage_asset_reward_vault(
            RuntimeOrigin::root(),
            vault_id,
            asset_id,
            AssetAction::Add,
        ));

        // Setup reward config
        let config = RewardConfigForAssetVault {
            apy: Percent::from_percent(10),
            deposit_cap: 1_000_000,
            incentive_cap: 100_000,
            boost_multiplier: Some(200),
        };
        assert_ok!(Rewards::udpate_vault_reward_config(
            RuntimeOrigin::root(),
            vault_id,
            config,
        ));

        // Setup mock deposit
        MOCK_DELEGATION_INFO.with(|m| {
            m.borrow_mut().deposits.insert(
                (alice.clone(), asset_id),
                UserDepositWithLocks { unlocked_amount: deposit_amount, amount_with_locks: None },
            );
        });

        PrecompilesValue::get()
            .prepare_test(
                TestAccount::Alice,
                H160::from_low_u64_be(1),
                PrecompileCall::claim_rewards {
                    asset_id: U256::from(1),
                    token_address: Address::zero(),
                },
            )
            .execute_returns(());

        // Check that rewards were claimed
        let claimed = UserClaimedReward::<Runtime>::get(alice, vault_id);
        assert!(claimed.is_some());
    });
}

#[test]
fn test_force_claim_rewards() {
    ExtBuilder::default().build().execute_with(|| {
        let vault_id = 1u32;
        let asset_id = Asset::Custom(1);
        let alice: AccountId = TestAccount::Alice.into();
        let bob: AccountId = TestAccount::Bob.into();
        let deposit_amount = 1_000u128;

        // Setup vault and add asset
        assert_ok!(Rewards::manage_asset_reward_vault(
            RuntimeOrigin::root(),
            vault_id,
            asset_id,
            AssetAction::Add,
        ));

        // Setup reward config
        let config = RewardConfigForAssetVault {
            apy: Percent::from_percent(10),
            deposit_cap: 1_000_000,
            incentive_cap: 100_000,
            boost_multiplier: Some(200),
        };
        assert_ok!(Rewards::udpate_vault_reward_config(
            RuntimeOrigin::root(),
            vault_id,
            config,
        ));

        // Setup mock deposit for Bob
        MOCK_DELEGATION_INFO.with(|m| {
            m.borrow_mut().deposits.insert(
                (bob.clone(), asset_id),
                UserDepositWithLocks { unlocked_amount: deposit_amount, amount_with_locks: None },
            );
        });

        // Alice force claims rewards for Bob
        PrecompilesValue::get()
            .prepare_test(
                TestAccount::Alice,
                H160::from_low_u64_be(1),
                PrecompileCall::force_claim_rewards {
                    account: Address(TestAccount::Bob.into()),
                    asset_id: U256::from(1),
                    token_address: Address::zero(),
                },
            )
            .execute_returns(());

        // Check that rewards were claimed for Bob
        let claimed = UserClaimedReward::<Runtime>::get(bob, vault_id);
        assert!(claimed.is_some());
    });
}

#[test]
fn test_update_vault_reward_config() {
    ExtBuilder::default().build().execute_with(|| {
        let vault_id = 1u32;

        // Update vault config
        PrecompilesValue::get()
            .prepare_test(
                TestAccount::Alice,
                H160::from_low_u64_be(1),
                PrecompileCall::update_vault_reward_config {
                    vault_id: U256::from(vault_id),
                    apy: 10,
                    deposit_cap: U256::from(1_000_000u128),
                    incentive_cap: U256::from(100_000u128),
                    boost_multiplier: 200,
                },
            )
            .execute_returns(());

        // Verify config was updated
        let config = RewardConfigStorage::<Runtime>::get(vault_id);
        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!(config.apy, Percent::from_percent(10));
        assert_eq!(config.deposit_cap, 1_000_000u128);
        assert_eq!(config.incentive_cap, 100_000u128);
        assert_eq!(config.boost_multiplier, Some(200));
    });
}

#[test]
fn test_manage_asset_reward_vault() {
    ExtBuilder::default().build().execute_with(|| {
        let vault_id = 1u32;
        let asset_id = 1u128;

        // Add asset to vault
        PrecompilesValue::get()
            .prepare_test(
                TestAccount::Alice,
                H160::from_low_u64_be(1),
                PrecompileCall::manage_asset_reward_vault {
                    vault_id: U256::from(vault_id),
                    asset_id: U256::from(asset_id),
                    token_address: Address::zero(),
                    add: true,
                },
            )
            .execute_returns(());

        // Verify asset was added
        let vault = AssetLookupRewardVaults::<Runtime>::get(Asset::Custom(asset_id));
        assert_eq!(vault, Some(vault_id));

        // Remove asset from vault
        PrecompilesValue::get()
            .prepare_test(
                TestAccount::Alice,
                H160::from_low_u64_be(1),
                PrecompileCall::manage_asset_reward_vault {
                    vault_id: U256::from(vault_id),
                    asset_id: U256::from(asset_id),
                    token_address: Address::zero(),
                    add: false,
                },
            )
            .execute_returns(());

        // Verify asset was removed
        let vault = AssetLookupRewardVaults::<Runtime>::get(Asset::Custom(asset_id));
        assert_eq!(vault, None);
    });
}

#[test]
fn test_claim_rewards_no_vault() {
    ExtBuilder::default().build().execute_with(|| {
        PrecompilesValue::get()
            .prepare_test(
                TestAccount::Alice,
                H160::from_low_u64_be(1),
                PrecompileCall::claim_rewards {
                    asset_id: U256::from(1),
                    token_address: Address::zero(),
                },
            )
            .execute_reverts(|output| output == b"AssetNotInVault");
    });
}

#[test]
fn test_claim_rewards_no_deposit() {
    ExtBuilder::default().build().execute_with(|| {
        let vault_id = 1u32;
        let asset_id = Asset::Custom(1);

        // Setup vault and add asset
        assert_ok!(Rewards::manage_asset_reward_vault(
            RuntimeOrigin::root(),
            vault_id,
            asset_id,
            AssetAction::Add,
        ));

        PrecompilesValue::get()
            .prepare_test(
                TestAccount::Alice,
                H160::from_low_u64_be(1),
                PrecompileCall::claim_rewards {
                    asset_id: U256::from(1),
                    token_address: Address::zero(),
                },
            )
            .execute_reverts(|output| output == b"NoRewardsAvailable");
    });
}

#[test]
fn test_update_vault_reward_config_invalid_values() {
    ExtBuilder::default().build().execute_with(|| {
        // Test APY > 100%
        PrecompilesValue::get()
            .prepare_test(
                TestAccount::Alice,
                H160::from_low_u64_be(1),
                PrecompileCall::update_vault_reward_config {
                    vault_id: U256::from(1),
                    apy: 101,
                    deposit_cap: U256::from(1_000_000u128),
                    incentive_cap: U256::from(100_000u128),
                    boost_multiplier: 200,
                },
            )
            .execute_returns(());

        // Verify APY was capped at 100%
        let config = RewardConfigStorage::<Runtime>::get(1);
        assert!(config.is_some());
        assert_eq!(config.unwrap().apy, Percent::from_percent(100));

        // Test boost multiplier > 500%
        PrecompilesValue::get()
            .prepare_test(
                TestAccount::Alice,
                H160::from_low_u64_be(1),
                PrecompileCall::update_vault_reward_config {
                    vault_id: U256::from(1),
                    apy: 50,
                    deposit_cap: U256::from(1_000_000u128),
                    incentive_cap: U256::from(100_000u128),
                    boost_multiplier: 600,
                },
            )
            .execute_returns(());

        // Verify boost multiplier was capped at 500%
        let config = RewardConfigStorage::<Runtime>::get(1);
        assert!(config.is_some());
        assert_eq!(config.unwrap().boost_multiplier, Some(500));
    });
}
