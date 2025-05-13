use super::*;
use crate::mock::*;
use frame_support::assert_ok;
use precompile_utils::testing::*;
use sp_core::{H160, U256};
use crate::mock_evm::PrecompilesValue;
use crate::mock_evm::PCall;

#[test]
fn test_burn_success() {
	ExtBuilder::default().build().execute_with(|| {
		let caller = H160::from_low_u64_be(1);
		let amount = U256::from(1000);

		PrecompilesValue::get()
			.prepare_test(TestAccount::Alex, caller, PCall::burn { amount })
			.execute_returns(true);
	});
}

#[test]
fn test_claim_credits_success() {
	ExtBuilder::default().build().execute_with(|| {
		let caller = H160::from_low_u64_be(1);
		let amount = U256::from(500);
		let account_id = b"offchain_user".to_vec();

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				caller,
				PCall::claim_credits {
					amount_to_claim: amount,
					offchain_account_id: account_id.clone().into(),
				},
			)
			.execute_returns(true);
	});
}

#[test]
fn test_claim_credits_fails_on_long_offchain_id() {
	ExtBuilder::default().build().execute_with(|| {
		let caller = H160::from_low_u64_be(1);
		let long_id = vec![0u8; 100]; // Make sure this exceeds OffchainAccountIdOf length limit

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				caller,
				PCall::claim_credits {
					amount_to_claim: U256::from(500),
					offchain_account_id: long_id.clone().into(),
				},
			)
			.execute_reverts(|output| output == b"Offchain account ID too long");
	});
}

#[test]
fn test_get_current_rate_returns_value() {
	ExtBuilder::default().build().execute_with(|| {
		let caller = H160::from_low_u64_be(1);
		let staked = U256::from(1000);

		let result = PrecompilesValue::get()
			.prepare_test(TestAccount::Alex, caller, PCall::get_current_rate { staked_amount: staked })
			.execute_and_decode_output::<U256>();

		assert!(result > U256::zero(), "Rate should be positive");
	});
}

#[test]
fn test_calculate_accrued_credits() {
	ExtBuilder::default().build().execute_with(|| {
		let caller = H160::from_low_u64_be(1);

		let accrued = PrecompilesValue::get()
			.prepare_test(TestAccount::Alex, caller, PCall::calculate_accrued_credits { account: caller })
			.execute_and_decode_output::<U256>();

		assert!(accrued >= U256::zero());
	});
}

#[test]
fn test_get_stake_tiers_returns_thresholds_and_rates() {
	ExtBuilder::default().build().execute_with(|| {
		let caller = H160::from_low_u64_be(1);

		let (thresholds, rates) = PrecompilesValue::get()
			.prepare_test(TestAccount::Alex, caller, PCall::get_stake_tiers {})
			.execute_and_decode_output::<(Vec<U256>, Vec<U256>)>();

		assert_eq!(thresholds.len(), rates.len());
	});
}