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

use fp_evm::PrecompileHandle;
use frame_support::{
    dispatch::{GetDispatchInfo, PostDispatchInfo},
    pallet_prelude::BoundedVec,
    traits::Currency,
};
use pallet_evm::AddressMapping;
use parity_scale_codec::Codec;
use precompile_utils::prelude::*;
use sp_core::{H160, U256};
use sp_std::{marker::PhantomData, vec::Vec};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub struct OraclePrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> OraclePrecompile<Runtime>
where
    Runtime: pallet_oracle::Config + pallet_evm::Config,
    Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    <Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
    Runtime::RuntimeCall: From<pallet_oracle::Call<Runtime>>,
    Runtime::OracleKey: TryFrom<U256> + Into<U256>,
    Runtime::OracleValue: TryFrom<U256> + Into<U256>,
{
    #[precompile::public("feedValues(uint256[],uint256[])")]
    fn feed_values(
        handle: &mut impl PrecompileHandle,
        keys: Vec<U256>,
        values: Vec<U256>,
    ) -> EvmResult {
        handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

        // Check that keys and values have the same length
        ensure!(keys.len() == values.len(), revert("Keys and values must have the same length"));

        // Convert keys and values to the required types
        let mut oracle_values = Vec::with_capacity(keys.len());
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            let oracle_key = key
                .try_into()
                .map_err(|_| RevertReason::value_is_too_large("oracle key"))?;
            let oracle_value = value
                .try_into()
                .map_err(|_| RevertReason::value_is_too_large("oracle value"))?;
            oracle_values.push((oracle_key, oracle_value));
        }

        // Convert to BoundedVec
        let bounded_values: BoundedVec<_, Runtime::MaxFeedValues> = oracle_values
            .try_into()
            .map_err(|_| revert("Too many values"))?;

        let caller = handle.context().caller;
        let who = Runtime::AddressMapping::into_account_id(caller);

        RuntimeHelper::<Runtime>::try_dispatch(
            handle,
            Some(who).into(),
            pallet_oracle::Call::<Runtime>::feed_values { values: bounded_values },
        )?;

        Ok(())
    }
}
