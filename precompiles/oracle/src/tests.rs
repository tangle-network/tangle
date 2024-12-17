use super::*;
use crate::{mock::*, OraclePrecompile, OraclePrecompileCall};
use frame_support::{assert_ok, BoundedVec};
use precompile_utils::{
	precompile_set::{AddressU64, PrecompileAt, PrecompileSetBuilder},
	testing::PrecompileTesterExt,
};
use sp_core::{H160, U256};

pub type PCall = OraclePrecompileCall<Runtime>;
pub type Precompiles =
	PrecompileSetBuilder<Runtime, (PrecompileAt<AddressU64<1>, OraclePrecompile<Runtime>>,)>;

fn precompiles() -> Precompiles {
	Precompiles::new()
}

#[test]
fn feed_values_works() {
	ExtBuilder::default().build().execute_with(|| {
		// Create a bounded vector for feed values
		let mut feed_values = Vec::new();
		feed_values.push((1u32, 100u64));
		feed_values.push((2u32, 200u64));
		let bounded_feed_values: BoundedVec<_, _> = feed_values.try_into().unwrap();

		assert_ok!(Oracle::feed_values(
			RuntimeOrigin::signed(TestAccount::Alice),
			bounded_feed_values
		));
	});
}

#[test]
fn precompile_feed_values_works() {
	ExtBuilder::default().build().execute_with(|| {
		// Test data
		let key1: U256 = U256::from(1u32);
		let value1: U256 = U256::from(100u64);
		let key2: U256 = U256::from(2u32);
		let value2: U256 = U256::from(200u64);

		precompiles()
			.prepare_test(
				TestAccount::Alice,
				H160::from_low_u64_be(1),
				PCall::feed_values { keys: vec![key1, key2], values: vec![value1, value2] },
			)
			.execute_returns(());
	});
}
