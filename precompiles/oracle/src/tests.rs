use super::*;
use crate::{mock::*, selectors, OraclePrecompile};
use frame_support::{assert_ok, BoundedVec};
use sp_core::{H160, U256};

fn evm_test_context() -> Context {
    Context {
        address: Default::default(),
        caller: H160::from_low_u64_be(1),
        apparent_value: Default::default(),
    }
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

        // Build input data
        let mut input = Vec::new();
        input.extend_from_slice(&selectors::FEED_VALUES.to_be_bytes());
        
        // Write array lengths and data
        let mut key_data = vec![0u8; 32];
        U256::from(2u32).to_big_endian(&mut key_data);
        input.extend_from_slice(&key_data);

        let mut key1_data = vec![0u8; 32];
        key1.to_big_endian(&mut key1_data);
        input.extend_from_slice(&key1_data);

        let mut key2_data = vec![0u8; 32];
        key2.to_big_endian(&mut key2_data);
        input.extend_from_slice(&key2_data);

        let mut value_data = vec![0u8; 32];
        U256::from(2u32).to_big_endian(&mut value_data);
        input.extend_from_slice(&value_data);

        let mut value1_data = vec![0u8; 32];
        value1.to_big_endian(&mut value1_data);
        input.extend_from_slice(&value1_data);

        let mut value2_data = vec![0u8; 32];
        value2.to_big_endian(&mut value2_data);
        input.extend_from_slice(&value2_data);

        let result = OraclePrecompile::<Runtime>::execute(&input, u64::MAX);
        assert!(result.is_ok());
    });
}
