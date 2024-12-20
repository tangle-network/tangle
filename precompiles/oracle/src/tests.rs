use super::*;
use crate::mock::*;
use frame_support::{assert_ok, BoundedVec};
use precompile_utils::{testing::*, prelude::*};
use sp_core::{H160, U256};

fn precompiles() -> PrecompileSet {
    PrecompilesValue::get()
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
            RuntimeOrigin::signed(H160::repeat_byte(0x01)),
            bounded_feed_values
        ));
    });
}

#[test]
fn precompile_feed_values_works() {
    ExtBuilder::default().build().execute_with(|| {
        let mut tester = PrecompileTester::new(precompiles(), H160::repeat_byte(0x01), PRECOMPILE_ADDRESS);

        // Test data
        let keys = vec![U256::from(1u32), U256::from(2u32)];
        let values = vec![U256::from(100u64), U256::from(200u64)];

        // Call the precompile
        tester.call("feedValues(uint256[],uint256[])", (keys.clone(), values.clone()))
            .expect_no_logs()
            .execute_returns(());

        // Verify values were stored
        let value1 = Oracle::get(1u32).unwrap();
        let value2 = Oracle::get(2u32).unwrap();
        assert_eq!(value1.value, 100u64);
        assert_eq!(value2.value, 200u64);
    });
}

#[test]
fn get_value_works() {
    ExtBuilder::default().build().execute_with(|| {
        let mut tester = PrecompileTester::new(precompiles(), H160::repeat_byte(0x01), PRECOMPILE_ADDRESS);

        // First feed a value
        let mut feed_values = Vec::new();
        feed_values.push((1u32, 100u64));
        let bounded_feed_values: BoundedVec<_, _> = feed_values.try_into().unwrap();
        assert_ok!(Oracle::feed_values(
            RuntimeOrigin::signed(H160::repeat_byte(0x01)),
            bounded_feed_values
        ));

        // Now try to read it through the precompile
        let (value, timestamp): (U256, U256) = tester
            .call("getValue(uint256)", U256::from(1u32))
            .expect_no_logs()
            .execute_returns();

        assert_eq!(value, U256::from(100u64));
        assert!(timestamp > U256::zero());
    });
}

#[test]
fn get_value_fails_for_non_existent_key() {
    ExtBuilder::default().build().execute_with(|| {
        let mut tester = PrecompileTester::new(precompiles(), H160::repeat_byte(0x01), PRECOMPILE_ADDRESS);

        // Try to read a non-existent key
        tester
            .call("getValue(uint256)", U256::from(999u32))
            .expect_no_logs()
            .execute_reverts(|output| output == b"No value found for key");
    });
}
