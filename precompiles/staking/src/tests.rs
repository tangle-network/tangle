use crate::mock::{self, new_test_ext, PCall, Precompiles, PrecompilesValue, Runtime, TestAccount};
use core::str::from_utf8;
use frame_support::{assert_ok, dispatch::Dispatchable, sp_runtime::Percent};
use pallet_evm::Call as EvmCall;
use pallet_staking::Event as StakingEvent;
use precompile_utils::{prelude::*, testing::*};
use sp_core::{H160, U256};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

fn evm_call(source: impl Into<H160>, input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: source.into(),
		target: Precompile1.into(),
		input,
		value: U256::zero(), // No value sent in EVM
		gas_limit: u64::max_value(),
		max_fee_per_gas: 0.into(),
		max_priority_fee_per_gas: Some(U256::zero()),
		nonce: None, // Use the next nonce
		access_list: Vec::new(),
	}
}

#[test]
fn selectors() {
	print!(" Hello this is selector {:?}", PCall::is_nominator_selectors());
	assert_eq!(true, false);
}

#[test]
fn minimum_validator_count_works() {
	new_test_ext(vec![1, 2, 3, 4, 5]).execute_with(|| {
		precompiles()
			.prepare_test(
				TestAccount::Alex,
				TestAccount::PrecompileAddress,
				PCall::max_validator_count {},
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(3u32)
	});
}
