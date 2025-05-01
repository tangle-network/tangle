// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.

use crate::Weight;
use fp_evm::{CallInfo, ExitReason, ExitSucceed, UsedGas};
use frame_system::Config;
use sp_core::{H160, U256};
use sp_std::vec::Vec;
use scale_info::prelude::vec;

#[derive(Debug)]
pub struct RunnerError<E: Into<sp_runtime::DispatchError>> {
	pub error: E,
	pub weight: Weight,
}

#[allow(clippy::too_many_arguments)]
pub trait EvmRunner<T: Config> {
	type Error: Into<sp_runtime::DispatchError>;

	fn call(
		source: H160,
		target: H160,
		input: Vec<u8>,
		value: U256,
		gas_limit: u64,
		is_transactional: bool,
		validate: bool,
	) -> Result<CallInfo, RunnerError<Self::Error>>;
}

impl<T: Config> EvmRunner<T> for () {
	type Error = sp_runtime::DispatchError;

	fn call(
		_source: H160,
		_target: H160,
		_input: Vec<u8>,
		_value: U256,
		_gas_limit: u64,
		_is_transactional: bool,
		_validate: bool,
	) -> Result<CallInfo, RunnerError<Self::Error>> {
		Ok(CallInfo {
			exit_reason: ExitReason::Succeed(ExitSucceed::Returned),
			value: vec![],
			used_gas: UsedGas { standard: U256::from(0), effective: U256::from(0) },
			weight_info: None,
			logs: vec![],
		})
	}
}

/// A mapping function that converts EVM gas to Substrate weight and vice versa
pub trait EvmGasWeightMapping {
	/// Convert EVM gas to Substrate weight
	fn gas_to_weight(gas: u64, without_base_weight: bool) -> Weight;
	/// Convert Substrate weight to EVM gas
	fn weight_to_gas(weight: Weight) -> u64;
}

impl EvmGasWeightMapping for () {
	fn gas_to_weight(_gas: u64, _without_base_weight: bool) -> Weight {
		Default::default()
	}
	fn weight_to_gas(_weight: Weight) -> u64 {
		Default::default()
	}
}

/// Trait to be implemented for evm address mapping.
pub trait EvmAddressMapping<A> {
	/// Convert an address to an account id.
	fn into_account_id(address: H160) -> A;

	/// Convert an account id to an address.
	fn into_address(account_id: A) -> H160;
}
