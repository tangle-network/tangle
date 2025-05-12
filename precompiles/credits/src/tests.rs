use super::*;
use mock::*;
use precompile_utils::testing::*;
use frame_support::{assert_ok, BoundedVec};
use sp_core::U256;
use sp_runtime::TokenError;

// Using the address from mock.rs

#[precompile_utils::solidity::codec::sol_interface]
pub trait Credits {
    #[sol(function)]
    fn burn(amount: U256) -> bool;

    #[sol(function)]
    fn claim_credits(amount_to_claim: U256, offchain_account_id: Vec<u8>) -> bool;

    #[sol(function)]
    fn get_current_rate(staked_amount: U256) -> U256;

    #[sol(function)]
    fn calculate_accrued_credits(account: Address) -> U256;

    #[sol(function)]
    fn get_stake_tiers() -> (Vec<U256>, Vec<U256>);
}

fn precompiles() -> Precompiles<Runtime> {
    PrecompilesValue::get()
}

#[test]
fn burn_works() {
    ExtBuilder::default()
        .balances(vec![(
            accounts().get("alice").unwrap().into(),
            10_000_000,
        )])
        .build()
        .execute_with(|| {
            let alice = accounts().get("alice").unwrap();
            let amount = 1000_u64;

            // Call the burn precompile function
            precompiles()
                .prepare_test(
                    alice,
                    PRECOMPILE_ADDRESS,
                    EvmDataWriter::new_with_selector(Action::Burn)
                        .write(U256::from(amount))
                        .build(),
                )
                .expect_success()
                .expect_log(pallet_credits::Event::CreditsGrantedFromBurn {
                    who: alice.into(),
                    tnt_burned: amount as u128,
                    credits_granted: amount as u128 * 10, // Based on BurnConversionRate = 10
                }
                .into_log())
                .execute_returns(EvmDataWriter::new().write(true).build());

            // Check alice's balance was reduced by the burned amount
            assert_eq!(
                Balances::free_balance(alice.into()),
                10_000_000 - amount as u128
            );
        });
}

#[test]
fn burn_insufficient_balance_fails() {
    ExtBuilder::default()
        .balances(vec![(
            accounts().get("alice").unwrap().into(),
            1000,
        )])
        .build()
        .execute_with(|| {
            let alice = accounts().get("alice").unwrap();
            let amount = 10_000_u64; // More than Alice has

            // Call the burn precompile function
            precompiles()
                .prepare_test(
                    alice,
                    PRECOMPILE_ADDRESS,
                    EvmDataWriter::new_with_selector(Action::Burn)
                        .write(U256::from(amount))
                        .build(),
                )
                .execute_reverts(|output| {
                    from_utf8_lossy(&output)
                        .contains("InsufficientTntBalance")
                });
        });
}

#[test]
fn claim_credits_works() {
    ExtBuilder::default()
        .balances(vec![(
            accounts().get("alice").unwrap().into(),
            10_000_000,
        )])
        .build()
        .execute_with(|| {
            let alice = accounts().get("alice").unwrap();
            let alice_account: AccountId = alice.into();
            
            // First, stake some TNT with MultiAssetDelegation
            assert_ok!(MultiAssetDelegation::deposit(
                RuntimeOrigin::signed(alice_account.clone()),
                TntAssetId::get(),
                500_000
            ));

            // Advance 10 blocks for credits accrual
            for _ in 0..10 {
                System::set_block_number(System::block_number() + 1);
            }

            // Prepare offchain_account_id
            let offchain_account_id = b"alice_offchain".to_vec();
            let amount_to_claim = 500; // Based on alice's stake tier and elapsed blocks

            // Call claim_credits
            precompiles()
                .prepare_test(
                    alice,
                    PRECOMPILE_ADDRESS,
                    EvmDataWriter::new_with_selector(Action::ClaimCredits)
                        .write(U256::from(amount_to_claim))
                        .write(offchain_account_id.clone())
                        .build(),
                )
                .expect_success()
                .expect_log(pallet_credits::Event::CreditsClaimed {
                    who: alice.into(),
                    amount_claimed: amount_to_claim,
                    offchain_account_id: BoundedVec::try_from(offchain_account_id).unwrap(),
                }
                .into_log())
                .execute_returns(EvmDataWriter::new().write(true).build());
        });
}

#[test]
fn claim_credits_exceeds_allowance_fails() {
    ExtBuilder::default()
        .balances(vec![(
            accounts().get("alice").unwrap().into(),
            10_000_000,
        )])
        .build()
        .execute_with(|| {
            let alice = accounts().get("alice").unwrap();
            let alice_account: AccountId = alice.into();
            
            // First, stake some TNT with MultiAssetDelegation
            assert_ok!(MultiAssetDelegation::deposit(
                RuntimeOrigin::signed(alice_account.clone()),
                TntAssetId::get(),
                100_000
            ));

            // Advance just 1 block for minimal credits accrual
            System::set_block_number(System::block_number() + 1);

            // Prepare offchain_account_id
            let offchain_account_id = b"alice_offchain".to_vec();
            let amount_to_claim = 10000; // More than would be accrued with the stake in 1 block

            // Call claim_credits - should fail due to exceeded allowance
            precompiles()
                .prepare_test(
                    alice,
                    PRECOMPILE_ADDRESS,
                    EvmDataWriter::new_with_selector(Action::ClaimCredits)
                        .write(U256::from(amount_to_claim))
                        .write(offchain_account_id.clone())
                        .build(),
                )
                .execute_reverts(|output| {
                    from_utf8_lossy(&output)
                        .contains("ClaimAmountExceedsWindowAllowance")
                });
        });
}

#[test]
fn get_current_rate_works() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let alice = accounts().get("alice").unwrap();
            
            // Test different stake amounts and their expected rates
            let test_cases = vec![
                (50_000, 0),        // Below any tier
                (100_000, 10),      // Tier 1
                (500_000, 50),      // Tier 2
                (1_000_000, 100),   // Tier 3
                (2_000_000, 100),   // Above highest tier, still gets highest rate
            ];
            
            for (stake_amount, expected_rate) in test_cases {
                let result = precompiles()
                    .prepare_test(
                        alice,
                        PRECOMPILE_ADDRESS,
                        EvmDataWriter::new_with_selector(Action::GetCurrentRate)
                            .write(U256::from(stake_amount))
                            .build(),
                    )
                    .expect_success()
                    .execute_returns_raw();

                let actual_rate = EvmDataReader::new(&result).read::<U256>().unwrap();
                assert_eq!(actual_rate, U256::from(expected_rate));
            }
        });
}

#[test]
fn calculate_accrued_credits_works() {
    ExtBuilder::default()
        .balances(vec![(
            accounts().get("alice").unwrap().into(),
            10_000_000,
        )])
        .build()
        .execute_with(|| {
            let alice = accounts().get("alice").unwrap();
            let alice_account: AccountId = alice.into();
            
            // First, stake some TNT with MultiAssetDelegation
            assert_ok!(MultiAssetDelegation::deposit(
                RuntimeOrigin::signed(alice_account.clone()),
                TntAssetId::get(),
                500_000
            ));

            // Advance 10 blocks for credits accrual
            for _ in 0..10 {
                System::set_block_number(System::block_number() + 1);
            }

            // Check accrued credits - should be 10 blocks * 50 credits per block = 500
            let expected_accrued = 500;
            
            let result = precompiles()
                .prepare_test(
                    alice,
                    PRECOMPILE_ADDRESS,
                    EvmDataWriter::new_with_selector(Action::CalculateAccruedCredits)
                        .write(Address::from(alice.address()))
                        .build(),
                )
                .expect_success()
                .execute_returns_raw();

            let actual_accrued = EvmDataReader::new(&result).read::<U256>().unwrap();
            assert_eq!(actual_accrued, U256::from(expected_accrued));
        });
}

#[test]
fn get_stake_tiers_works() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let alice = accounts().get("alice").unwrap();
            
            // Call get_stake_tiers
            let result = precompiles()
                .prepare_test(
                    alice,
                    PRECOMPILE_ADDRESS,
                    EvmDataWriter::new_with_selector(Action::GetStakeTiers)
                        .build(),
                )
                .expect_success()
                .execute_returns_raw();

            // Decode the result
            let mut reader = EvmDataReader::new(&result);
            let thresholds = reader.read::<Vec<U256>>().unwrap();
            let rates = reader.read::<Vec<U256>>().unwrap();
            
            // Check against the predefined tiers
            assert_eq!(thresholds.len(), 3);
            assert_eq!(rates.len(), 3);
            
            // Check tier values
            assert_eq!(thresholds[0], U256::from(100_000));
            assert_eq!(rates[0], U256::from(10));
            
            assert_eq!(thresholds[1], U256::from(500_000));
            assert_eq!(rates[1], U256::from(50));
            
            assert_eq!(thresholds[2], U256::from(1_000_000));
            assert_eq!(rates[2], U256::from(100));
        });
}