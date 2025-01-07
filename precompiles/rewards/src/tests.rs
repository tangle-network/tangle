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
