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

use core::str::FromStr;
use fp_evm::{PrecompileHandle, PrecompileOutput};
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::Get,
};
use pallet_evm::AddressMapping;
use pallet_oracle::TimestampedValue;
use precompile_utils::{
	prelude::*,
	solidity::{codec::Writer, modifier::FunctionModifier, revert::revert},
};
use sp_core::U256;
use sp_std::{marker::PhantomData, prelude::*};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	GetValue = "getValue(uint256)",
	FeedValues = "feedValues(uint256[],uint256[])",
}

/// A precompile to wrap the functionality from pallet_oracle.
pub struct OraclePrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> OraclePrecompile<Runtime>
where
	Runtime: pallet_oracle::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_oracle::Call<Runtime>>,
{
	pub fn new() -> Self {
		Self(PhantomData)
	}

	fn get_value(handle: &mut impl PrecompileHandle, key: U256) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let key: u32 = key.try_into().map_err(|_| revert("Invalid key"))?;

		let value = <pallet_oracle::Pallet<Runtime>>::get(&key);

		let mut writer = Writer::new_with_selector(Action::GetValue);

		if let Some(TimestampedValue { value, timestamp }) = value {
			writer.write(U256::from(value));
			writer.write(U256::from(timestamp));
		} else {
			writer.write(U256::zero());
			writer.write(U256::zero());
		}

		Ok(PrecompileOutput { exit_status: ExitSucceed::Returned, output: writer.build() })
	}

	fn feed_values(
		handle: &mut impl PrecompileHandle,
		keys: Vec<U256>,
		values: Vec<U256>,
	) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let caller = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let mut feed_values = Vec::new();
		for (key, value) in keys.iter().zip(values.iter()) {
			let key: u32 = key.try_into().map_err(|_| revert("Invalid key"))?;
			let value: u64 = value.try_into().map_err(|_| revert("Invalid value"))?;
			feed_values.push((key, value));
		}

		let bounded_feed_values = feed_values.try_into().map_err(|_| revert("Too many values"))?;

		let call = pallet_oracle::Call::<Runtime>::feed_values { feed_values: bounded_feed_values };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(caller).into(), call)?;

		Ok(PrecompileOutput { exit_status: ExitSucceed::Returned, output: vec![] })
	}
}

impl<Runtime> Precompile for OraclePrecompile<Runtime>
where
	Runtime: pallet_oracle::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_oracle::Call<Runtime>>,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let selector = handle.read_selector()?;

		match selector {
			Action::GetValue => Self::get_value(handle, handle.read_u256()?),
			Action::FeedValues => {
				let keys = handle.read_u256_array()?;
				let values = handle.read_u256_array()?;
				Self::feed_values(handle, keys, values)
			},
		}
	}
}
