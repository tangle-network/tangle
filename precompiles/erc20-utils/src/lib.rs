// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// This file is part of pallet-evm-precompile-multi-asset-delegation package.
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

use ethabi::Function;
use fp_evm::PrecompileFailure;
use precompile_utils::prelude::*;
use sp_core::U256;
use sp_std::prelude::*;

#[cfg(not(feature = "std"))]
use alloc::format;

/// Executes an ERC20 token transfer by calling the `transfer` function on the specified ERC20
/// contract
///
/// # Arguments
///
/// * `handle` - Handle for the precompile execution context
/// * `erc20` - Address of the ERC20 token contract
/// * `to` - Destination address to transfer tokens to
/// * `amount` - Amount of tokens to transfer
///
/// # Returns
///
/// * `Ok(bool)` - Returns true if transfer succeeded, false otherwise
/// * `Err(PrecompileFailure)` - If the transfer call fails or returns invalid data
pub fn erc20_transfer(
	handle: &mut impl PrecompileHandle,
	erc20: Address,
	caller: Address,
	to: Address,
	amount: U256,
) -> EvmResult<bool> {
	#[allow(deprecated)]
	let transfer_fn = Function {
		name: String::from("transfer"),
		inputs: Vec::from([
			ethabi::Param {
				name: String::from("to"),
				kind: ethabi::ParamType::Address,
				internal_type: None,
			},
			ethabi::Param {
				name: String::from("value"),
				kind: ethabi::ParamType::Uint(256),
				internal_type: None,
			},
		]),
		outputs: Vec::from([ethabi::Param {
			name: String::from("success"),
			kind: ethabi::ParamType::Bool,
			internal_type: None,
		}]),
		constant: None,
		state_mutability: ethabi::StateMutability::NonPayable,
	};

	let args = [ethabi::Token::Address(to.0), ethabi::Token::Uint(ethabi::Uint::from(amount))];

	let data = transfer_fn
		.encode_input(&args)
		.map_err(|e| revert(format!("failed to encode IERC20.transfer call: {e:?}")))?;
	// let gas_limit = Some(handle.remaining_gas());
	let gas_limit = None;
	let is_static = false;
	let context =
		fp_evm::Context { address: erc20.0, caller: caller.0, apparent_value: U256::zero() };
	let (exit_reason, output) = handle.call(erc20.0, None, data, gas_limit, is_static, &context);

	log::debug!(
		target: "evm",
		"ERC20.transfer: context: {:?}, exit_reason: {:?}, input: ({:?}, {}), output: 0x{}",
		context,
		exit_reason,
		to.0,
		amount,
		hex::encode(&output),
	);

	match exit_reason {
		fp_evm::ExitReason::Succeed(_) => {
			// decode the result and return it
			let result = transfer_fn
				.decode_output(&output)
				.map_err(|e| revert(format!("failed to decode IERC20.transfer result: {e:?}")))?;
			let first_token = result.first().ok_or(RevertReason::custom("no return value"))?;
			let s = if let ethabi::Token::Bool(val) = first_token { *val } else { false };
			Ok(s)
		},
		fp_evm::ExitReason::Error(e) => Err(PrecompileFailure::Error { exit_status: e }),
		fp_evm::ExitReason::Revert(e) => Err(PrecompileFailure::Revert { exit_status: e, output }),
		fp_evm::ExitReason::Fatal(e) => Err(PrecompileFailure::Fatal { exit_status: e }),
	}
}
