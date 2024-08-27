// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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

//! This file contains the implementation of the MultiAssetDelegationPrecompile struct which
//! provides an interface between the EVM and the native MultiAssetDelegation pallet of the runtime.
//! It allows EVM contracts to call functions of the MultiAssetDelegation pallet, in order to enable
//! EVM accounts to interact with the delegation system.
//!
//! The MultiAssetDelegationPrecompile struct implements core methods that correspond to the
//! functions of the MultiAssetDelegation pallet. These methods can be called from EVM contracts.
//! They include functions to join as an operator, delegate assets, withdraw assets, etc.
//!
//! Each method records the gas cost for the operation, performs the requested operation, and
//! returns the result in a format that can be used by the EVM.
//!
//! The MultiAssetDelegationPrecompile struct is generic over the Runtime type, which is the type of
//! the runtime that includes the MultiAssetDelegation pallet. This allows the precompile to work
//! with any runtime that includes the MultiAssetDelegation pallet and meets the other trait bounds
//! required by the precompile.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use fp_evm::PrecompileHandle;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::Currency,
};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{H160, H256, U256};
use sp_runtime::traits::Dispatchable;
use sp_std::{marker::PhantomData, vec::Vec};
use tangle_primitives::types::WrappedAccountId32;

type BalanceOf<Runtime> =
	<<Runtime as pallet_multi_asset_delegation::Config>::Currency as Currency<
		<Runtime as frame_system::Config>::AccountId,
	>>::Balance;

pub struct MultiAssetDelegationPrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> MultiAssetDelegationPrecompile<Runtime>
where
	Runtime: pallet_multi_asset_delegation::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_multi_asset_delegation::Call<Runtime>>,
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
				return Err(revert("Error while parsing staker's address"));
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
				let ethereum_address = Address(H160::from_slice(&payee.0[12..]));
				Runtime::AddressMapping::into_account_id(ethereum_address.0)
			},
			H256(account) => Self::parse_32byte_address(account.to_vec())?,
		};

		Ok(payee)
	}
}

#[precompile_utils::precompile]
impl<Runtime> MultiAssetDelegationPrecompile<Runtime>
where
	Runtime: pallet_multi_asset_delegation::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_multi_asset_delegation::Call<Runtime>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
	Runtime::AccountId: From<WrappedAccountId32>,
	<Runtime as pallet_multi_asset_delegation::Config>::AssetId:
		TryFrom<U256> + Into<U256> + solidity::Codec,
{
	#[precompile::public("joinOperators(uint256)")]
	fn join_operators(handle: &mut impl PrecompileHandle, bond_amount: U256) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let bond_amount: BalanceOf<Runtime> =
			bond_amount.try_into().map_err(|_| revert("Invalid bond amount"))?;
		let call = pallet_multi_asset_delegation::Call::<Runtime>::join_operators { bond_amount };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("scheduleLeaveOperators()")]
	fn schedule_leave_operators(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_multi_asset_delegation::Call::<Runtime>::schedule_leave_operators {};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("cancelLeaveOperators()")]
	fn cancel_leave_operators(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_multi_asset_delegation::Call::<Runtime>::cancel_leave_operators {};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("executeLeaveOperators()")]
	fn execute_leave_operators(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_multi_asset_delegation::Call::<Runtime>::execute_leave_operators {};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("operatorBondMore(uint256)")]
	fn operator_bond_more(handle: &mut impl PrecompileHandle, additional_bond: U256) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let additional_bond: BalanceOf<Runtime> =
			additional_bond.try_into().map_err(|_| revert("Invalid bond amount"))?;
		let call =
			pallet_multi_asset_delegation::Call::<Runtime>::operator_bond_more { additional_bond };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("scheduleOperatorUnstake(uint256)")]
	fn schedule_operator_unstake(
		handle: &mut impl PrecompileHandle,
		unstake_amount: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let unstake_amount: BalanceOf<Runtime> =
			unstake_amount.try_into().map_err(|_| revert("Invalid unstake amount"))?;
		let call = pallet_multi_asset_delegation::Call::<Runtime>::schedule_operator_unstake {
			unstake_amount,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("executeOperatorUnstake()")]
	fn execute_operator_unstake(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_multi_asset_delegation::Call::<Runtime>::execute_operator_unstake {};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("cancelOperatorUnstake()")]
	fn cancel_operator_unstake(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_multi_asset_delegation::Call::<Runtime>::cancel_operator_unstake {};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("goOffline()")]
	fn go_offline(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_multi_asset_delegation::Call::<Runtime>::go_offline {};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("goOnline()")]
	fn go_online(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_multi_asset_delegation::Call::<Runtime>::go_online {};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("deposit(uint256,uint256)")]
	fn deposit(handle: &mut impl PrecompileHandle, asset_id: U256, amount: U256) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let asset_id: <Runtime as pallet_multi_asset_delegation::Config>::AssetId =
			asset_id.try_into().map_err(|_| revert("Invalid asset id"))?;
		let amount: BalanceOf<Runtime> = amount.try_into().map_err(|_| revert("Invalid amount"))?;
		let call = pallet_multi_asset_delegation::Call::<Runtime>::deposit { asset_id, amount };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("scheduleWithdraw(uint256,uint256)")]
	fn schedule_withdraw(
		handle: &mut impl PrecompileHandle,
		asset_id: U256,
		amount: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let asset_id: <Runtime as pallet_multi_asset_delegation::Config>::AssetId =
			asset_id.try_into().map_err(|_| revert("Invalid asset id"))?;
		let amount: BalanceOf<Runtime> = amount.try_into().map_err(|_| revert("Invalid amount"))?;
		let call =
			pallet_multi_asset_delegation::Call::<Runtime>::schedule_withdraw { asset_id, amount };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("executeWithdraw()")]
	fn execute_withdraw(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_multi_asset_delegation::Call::<Runtime>::execute_withdraw {};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("cancelWithdraw(uint256,uint256)")]
	fn cancel_withdraw(
		handle: &mut impl PrecompileHandle,
		asset_id: U256,
		amount: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let asset_id: <Runtime as pallet_multi_asset_delegation::Config>::AssetId =
			asset_id.try_into().map_err(|_| revert("Invalid asset id"))?;
		let amount: BalanceOf<Runtime> = amount.try_into().map_err(|_| revert("Invalid amount"))?;
		let call =
			pallet_multi_asset_delegation::Call::<Runtime>::cancel_withdraw { asset_id, amount };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("delegate(bytes32,uint256,uint256)")]
	fn delegate(
		handle: &mut impl PrecompileHandle,
		operator: H256,
		asset_id: U256,
		amount: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let operator = Self::convert_to_account_id(operator)?;
		let asset_id: <Runtime as pallet_multi_asset_delegation::Config>::AssetId =
			asset_id.try_into().map_err(|_| revert("Invalid asset id"))?;
		let amount: BalanceOf<Runtime> = amount.try_into().map_err(|_| revert("Invalid amount"))?;
		let call =
			pallet_multi_asset_delegation::Call::<Runtime>::delegate { operator, asset_id, amount };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("scheduleDelegatorUnstake(bytes32,uint256,uint256)")]
	fn schedule_delegator_unstake(
		handle: &mut impl PrecompileHandle,
		operator: H256,
		asset_id: U256,
		amount: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let operator = Self::convert_to_account_id(operator)?;
		let asset_id: <Runtime as pallet_multi_asset_delegation::Config>::AssetId =
			asset_id.try_into().map_err(|_| revert("Invalid asset id"))?;
		let amount: BalanceOf<Runtime> = amount.try_into().map_err(|_| revert("Invalid amount"))?;
		let call = pallet_multi_asset_delegation::Call::<Runtime>::schedule_delegator_unstake {
			operator,
			asset_id,
			amount,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("executeDelegatorUnstake()")]
	fn execute_delegator_unstake(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_multi_asset_delegation::Call::<Runtime>::execute_delegator_unstake {};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("cancelDelegatorUnstake(bytes32,uint256,uint256)")]
	fn cancel_delegator_unstake(
		handle: &mut impl PrecompileHandle,
		operator: H256,
		asset_id: U256,
		amount: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let operator = Self::convert_to_account_id(operator)?;
		let asset_id: <Runtime as pallet_multi_asset_delegation::Config>::AssetId =
			asset_id.try_into().map_err(|_| revert("Invalid asset id"))?;
		let amount: BalanceOf<Runtime> = amount.try_into().map_err(|_| revert("Invalid amount"))?;
		let call = pallet_multi_asset_delegation::Call::<Runtime>::cancel_delegator_unstake {
			operator,
			asset_id,
			amount,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}
}
