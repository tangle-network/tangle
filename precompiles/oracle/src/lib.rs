#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use pallet_oracle::TimestampedValue;
use precompile_utils::{prelude::*, solidity};
use sp_core::U256;
use sp_runtime::traits::Dispatchable;
use sp_std::marker::PhantomData;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub struct OraclePrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> OraclePrecompile<Runtime>
where
    Runtime: pallet_oracle::Config<OracleKey = u32, OracleValue = u64> + pallet_evm::Config,
    Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    <Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
    Runtime::RuntimeCall: From<pallet_oracle::Call<Runtime>>,
{
    pub fn new() -> Self {
        Self(PhantomData)
    }

    /// Helper method to convert U256 to OracleKey
    fn u256_to_key(key: U256) -> EvmResult<u32> {
        Ok(key.low_u32())
    }

    /// Helper method to convert U256 to OracleValue
    fn u256_to_value(value: U256) -> EvmResult<u64> {
        Ok(value.low_u64())
    }
}

#[precompile_utils::precompile]
impl<Runtime> OraclePrecompile<Runtime>
where
    Runtime: pallet_oracle::Config<OracleKey = u32, OracleValue = u64> + pallet_evm::Config,
    Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    <Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
    Runtime::RuntimeCall: From<pallet_oracle::Call<Runtime>>,
{
    #[precompile::public("feedValues(uint256[],uint256[])")]
    fn feed_values(
        handle: &mut impl PrecompileHandle,
        keys: Vec<U256>,
        values: Vec<U256>,
    ) -> EvmResult {
        handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

        let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

        // Convert U256 arrays to oracle types
        let mut feed_values = Vec::new();
        for (key, value) in keys.iter().zip(values.iter()) {
            let key = Self::u256_to_key(*key)?;
            let value = Self::u256_to_value(*value)?;
            feed_values.push((key, value));
        }

        let bounded_feed_values = feed_values.try_into().map_err(|_| revert("Too many values"))?;

        let call = pallet_oracle::Call::<Runtime>::feed_values { values: bounded_feed_values };

        RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;
        Ok(())
    }

    #[precompile::public("getValue(uint256)")]
    #[precompile::view]
    fn get_value(
        handle: &mut impl PrecompileHandle,
        key: U256,
    ) -> EvmResult<(U256, U256)> {
        // Record the gas cost for reading from storage
        handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

        let key = Self::u256_to_key(key)?;
        
        // Get the value from the oracle
        let TimestampedValue { value, timestamp } = pallet_oracle::Pallet::<Runtime>::get(key)
            .ok_or_else(|| revert("No value found for key"))?;

        Ok((value.into(), timestamp.into()))
    }
}
