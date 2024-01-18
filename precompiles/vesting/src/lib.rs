// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
//
// This file is part of pallet-evm-precompile-staking package, originally developed by Purestake
// Inc. Pallet-evm-precompile-staking package used in Tangle Network in terms of GPLv3.

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

//! This file contains the implementation of the VestingPrecompile struct which provides an
//! interface between the EVM and the native vesting pallet of the runtime. It allows EVM contracts
//! to call functions of the vesting pallet, in order to allow EVM accounts to claim vested funds.
//!
//! The VestingPrecompile struct implements core methods that correspond to the functions of the
//! vesting pallet. These methods can be called from EVM contracts. They include functions to get
//! the claim vested funds, claim vested funds on behalf of an account, and transfer a vesting
//! schedule.
//!
//! Each method records the gas cost for the operation, performs the requested operation, and
//! returns the result in a format that can be used by the EVM.
//!
//! The VestingPrecompile struct is generic over the Runtime type, which is the type of the runtime
//! that includes the staking pallet. This allows the precompile to work with any runtime that
//! includes the staking pallet and meets the other trait bounds required by the precompile.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use evm::Runtime;
use fp_evm::PrecompileHandle;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	ensure,
	traits::Currency,
};

use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{H160, H256, U256};
use sp_runtime::{
	traits::{AccountIdLookup, Dispatchable, StaticLookup},
	MultiAddress,
};
use sp_std::{convert::TryInto, marker::PhantomData, vec, vec::Vec};
use tangle_primitives::types::WrappedAccountId32;

type BalanceOf<Runtime> = <<Runtime as pallet_vesting::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

pub struct VestingPrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> VestingPrecompile<Runtime>
where
	Runtime: pallet_vesting::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_vesting::Call<Runtime>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
	Runtime::AccountId: From<WrappedAccountId32>,
{
	/// Helper method to parse SS58 address
	fn parse_32byte_address(addr: Vec<u8>) -> EvmResult<Runtime::AccountId> {
		let addr: Runtime::AccountId = match addr.len() {
			// public address of the ss58 account has 32 bytes
			32 => {
				let mut addr_bytes = [0_u8; 32];
				addr_bytes[..].clone_from_slice(&addr[0..32]);

				WrappedAccountId32(addr_bytes).into()
			},
			_ => {
				// Return err if account length is wrong
				return Err(revert("Error while parsing staker's address"))
			},
		};

		Ok(addr)
	}

	/// Helper for converting from u8 to RewardDestination
	fn convert_to_account_id(payee: H256) -> EvmResult<Runtime::AccountId> {
		let payee = match payee {
			H256(
				[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _],
			) => {
				let ethereum_address = Address(H160::from_slice(&payee.0[10..]));
				Runtime::AddressMapping::into_account_id(ethereum_address.0)
			},
			H256(account) => Self::parse_32byte_address(account.to_vec())?,
		};

		Ok(payee)
	}
}

#[precompile_utils::precompile]
impl<Runtime> VestingPrecompile<Runtime>
where
	Runtime: pallet_vesting::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_vesting::Call<Runtime>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
	Runtime::AccountId: From<WrappedAccountId32>,
{
	#[precompile::public("vest()")]
	fn vest(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Make the call to vest the `msg.sender` account.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_vesting::Call::<Runtime>::vest {};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("vestOther(bytes32)")]
	#[precompile::public("vest_other(bytes32)")]
	fn vest_other(handle: &mut impl PrecompileHandle, target: H256) -> EvmResult<u8> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Make the call to vest the `target` account.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let target = Self::convert_to_account_id(target)?;
		let tgt = <<Runtime as frame_system::Config>::Lookup as StaticLookup>::unlookup(target);
		let call = pallet_vesting::Call::<Runtime>::vest_other { target: tgt };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok((0))
	}

	#[precompile::public("vestedTransfer(bytes32,uint8)")]
	#[precompile::public("vested_transfer(bytes32,uint8)")]
	fn vested_transfer(
		handle: &mut impl PrecompileHandle,
		target: H256,
		index: u8,
	) -> EvmResult<u8> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// First get the vesting schedule of the `msg.sender`
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		match pallet_vesting::Vesting::<Runtime>::get(origin.clone()) {
			Some(schedules) => {
				if index >= schedules.len() as u8 {
					return Err(revert("Invalid vesting schedule index"))
				}
				// Make the call to transfer the vested funds to the `target` account.
				let target = Self::convert_to_account_id(target)?;
				let tgt =
					<<Runtime as frame_system::Config>::Lookup as StaticLookup>::unlookup(target);
				let call = pallet_vesting::Call::<Runtime>::vested_transfer {
					target: tgt,
					schedule: schedules[index as usize],
				};

				// Dispatch call (if enough gas).
				RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

				return Ok((0))
			},
			None => Err(revert("No vesting schedule found for the sender")),
		}
	}
}
