// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tangle. If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::{
    ExitError, ExitSucceed, LinearCostPrecompile, PrecompileFailure, PrecompileHandle,
    PrecompileOutput,
};
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use pallet_oracle::TimestampedValue;
use precompile_utils::{
    prelude::*,
    solidity::{codec::Writer, revert::revert},
};
use sp_core::U256;
use sp_runtime::traits::{Dispatchable, UniqueSaturatedInto};
use sp_std::{marker::PhantomData, prelude::*};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// The selector values for each function in the precompile.
/// These are calculated using the first 4 bytes of the Keccak hash of the function signature.
pub mod selectors {
    /// Selector for the getValue(uint256) function
    pub const GET_VALUE: u32 = 0x20965255;
    /// Selector for the feedValues(uint256[],uint256[]) function
    pub const FEED_VALUES: u32 = 0x983b2d56;
}

/// A precompile to wrap the functionality from pallet_oracle.
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

    fn get_value_impl(input: &[u8]) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
        let key = U256::from_big_endian(&input[4..]);
        let key: u32 = key.low_u32();
        let value = <pallet_oracle::Pallet<Runtime>>::get(&key);

        let mut writer = Writer::default();

        if let Some(TimestampedValue { value, timestamp }) = value {
            let mut writer = writer.write(U256::from(value));
            let timestamp_u64: u64 = UniqueSaturatedInto::<u64>::unique_saturated_into(timestamp);
            writer = writer.write(U256::from(timestamp_u64));
            Ok((ExitSucceed::Returned, writer.build()))
        } else {
            let mut writer = writer.write(U256::zero());
            writer = writer.write(U256::zero());
            Ok((ExitSucceed::Returned, writer.build()))
        }
    }

    fn feed_values_impl(input: &[u8]) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
        let mut keys = Vec::new();
        let mut values = Vec::new();

        // Read the length of the arrays
        let keys_len = U256::from_big_endian(&input[4..36]).as_usize();
        let mut pos = 36;

        // Read keys
        for _ in 0..keys_len {
            keys.push(U256::from_big_endian(&input[pos..pos + 32]));
            pos += 32;
        }

        // Read values
        let values_len = U256::from_big_endian(&input[pos..pos + 32]).as_usize();
        pos += 32;

        for _ in 0..values_len {
            values.push(U256::from_big_endian(&input[pos..pos + 32]));
            pos += 32;
        }

        let mut feed_values = Vec::new();
        for (key, value) in keys.iter().zip(values.iter()) {
            let key: u32 = key.low_u32();
            let value: u64 = value.low_u64();
            feed_values.push((key, value));
        }

        let bounded_feed_values = feed_values.try_into().map_err(|_| PrecompileFailure::Error {
            exit_status: ExitError::Other("Too many values".into()),
        })?;

        let call = pallet_oracle::Call::<Runtime>::feed_values { values: bounded_feed_values };

        Ok((ExitSucceed::Returned, vec![]))
    }
}

impl<Runtime> LinearCostPrecompile for OraclePrecompile<Runtime>
where
    Runtime: pallet_oracle::Config<OracleKey = u32, OracleValue = u64> + pallet_evm::Config,
    Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    <Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
    Runtime::RuntimeCall: From<pallet_oracle::Call<Runtime>>,
{
    const BASE: u64 = 20;
    const WORD: u64 = 10;

    fn execute(
        input: &[u8],
        _target_gas: u64,
    ) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
        let selector = input.get(0..4).ok_or_else(|| {
            PrecompileFailure::Error { exit_status: ExitError::Other("Invalid selector".into()) }
        })?;
        let selector = u32::from_be_bytes(selector.try_into().unwrap());

        match selector {
            selectors::GET_VALUE => Self::get_value_impl(input),
            selectors::FEED_VALUES => Self::feed_values_impl(input),
            _ => Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("Unknown selector".into()),
            }),
        }
    }
}
