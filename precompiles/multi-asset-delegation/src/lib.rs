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

use fp_evm::{PrecompileFailure, PrecompileHandle};
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::Currency,
};
use pallet_evm::AddressMapping;
use pallet_multi_asset_delegation::types::DelegatorBlueprintSelection;
use parity_scale_codec::Decode;
use precompile_utils::prelude::*;
use sp_core::{H160, H256, U256};
use sp_runtime::traits::{Dispatchable, TryConvert};
use sp_std::{marker::PhantomData, vec::Vec};
use tangle_primitives::{services::Asset, types::WrappedAccountId32};

type BalanceOf<Runtime> =
	<<Runtime as pallet_multi_asset_delegation::Config>::Currency as Currency<
		<Runtime as frame_system::Config>::AccountId,
	>>::Balance;

type AssetIdOf<Runtime> = <Runtime as pallet_multi_asset_delegation::Config>::AssetId;

pub struct MultiAssetDelegationPrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> MultiAssetDelegationPrecompile<Runtime>
where
	Runtime: pallet_multi_asset_delegation::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_multi_asset_delegation::Call<Runtime>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
	AssetIdOf<Runtime>: TryFrom<U256> + Into<U256>,
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

	fn u256_to_amount(amount: U256) -> EvmResult<BalanceOf<Runtime>> {
		amount.try_into().map_err(|_| revert("Invalid amount"))
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
	AssetIdOf<Runtime>: TryFrom<U256> + Into<U256> + From<u32>,
	Runtime::AccountId: From<WrappedAccountId32>,
{
	// Errors for the `MultiAssetDelegation` precompile.

	/// Found an invalid amount / value.
	const INVALID_AMOUNT: [u8; 32] = keccak256!("InvalidAmount()");
	/// Value must be zero for ERC20 payment asset.
	const VALUE_NOT_ZERO_FOR_ERC20: [u8; 32] = keccak256!("ValueMustBeZeroForERC20()");
	/// Value must be zero for custom payment asset.
	const VALUE_NOT_ZERO_FOR_CUSTOM_ASSET: [u8; 32] = keccak256!("ValueMustBeZeroForCustomAsset()");
	/// Payment asset should be either custom or ERC20.
	const PAYMENT_ASSET_SHOULD_BE_CUSTOM_OR_ERC20: [u8; 32] =
		keccak256!("PaymentAssetShouldBeCustomOrERC20()");

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
	fn deposit(
		handle: &mut impl PrecompileHandle,
		asset_id: U256,
		token_address: Address,
		amount: U256,
	) -> EvmResult {
		let amount = Self::u256_to_amount(amount)?;

		let (deposit_asset, amount) = match (asset_id.as_u32(), token_address.0 .0) {
			(0, erc20_token) => (Asset::Erc20(erc20_token.into()), amount),
			(other_asset_id, ZERO_ADDRESS) => (Asset::Custom(other_asset_id.into()), amount),
			(_other_asset_id, _erc20_token) => {
				return Err(revert_custom_error(Self::PAYMENT_ASSET_SHOULD_BE_CUSTOM_OR_ERC20))
			},
		};

		// Get origin account.
		let msg_sender = handle.context().caller;
		let origin = Runtime::AddressMapping::into_account_id(msg_sender);

		// Build call with origin.
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let call = pallet_multi_asset_delegation::Call::<Runtime>::deposit {
			asset_id: deposit_asset,
			amount,
			evm_address: Some(msg_sender),
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("schedule_withdraw(uint256,uint256)")]
	fn schedule_withdraw(
		handle: &mut impl PrecompileHandle,
		asset_id: U256,
		token_address: Address,
		amount: U256,
	) -> EvmResult {
		let amount = Self::u256_to_amount(amount)?;

		// Get origin account.
		let msg_sender = handle.context().caller;
		let origin = Runtime::AddressMapping::into_account_id(msg_sender);

		// Build call with origin.
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let (deposit_asset, amount) = match (asset_id.as_u32(), token_address.0 .0) {
			(0, erc20_token) => (Asset::Erc20(erc20_token.into()), amount),
			(other_asset_id, ZERO_ADDRESS) => (Asset::Custom(other_asset_id.into()), amount),
			(_other_asset_id, _erc20_token) => {
				return Err(revert_custom_error(Self::PAYMENT_ASSET_SHOULD_BE_CUSTOM_OR_ERC20))
			},
		};

		let call = pallet_multi_asset_delegation::Call::<Runtime>::schedule_withdraw {
			asset_id: deposit_asset,
			amount,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("executeWithdraw()")]
	fn execute_withdraw(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_multi_asset_delegation::Call::<Runtime>::execute_withdraw {
			evm_address: Some(handle.context().caller),
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("cancel_withdraw(uint256,uint256)")]
	fn cancel_withdraw(
		handle: &mut impl PrecompileHandle,
		asset_id: U256,
		token_address: Address,
		amount: U256,
	) -> EvmResult {
		let amount = Self::u256_to_amount(amount)?;

		// Get origin account.
		let msg_sender = handle.context().caller;
		let origin = Runtime::AddressMapping::into_account_id(msg_sender);

		let (deposit_asset, amount) = match (asset_id.as_u32(), token_address.0 .0) {
			(0, erc20_token) => (Asset::Erc20(erc20_token.into()), amount),
			(other_asset_id, ZERO_ADDRESS) => (Asset::Custom(other_asset_id.into()), amount),
			(_other_asset_id, _erc20_token) => {
				return Err(revert_custom_error(Self::PAYMENT_ASSET_SHOULD_BE_CUSTOM_OR_ERC20))
			},
		};

		// Build call with origin.
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let call = pallet_multi_asset_delegation::Call::<Runtime>::cancel_withdraw {
			asset_id: deposit_asset,
			amount,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("delegate(bytes32,uint256,uint256,uint64[])")]
	fn delegate(
		handle: &mut impl PrecompileHandle,
		operator: H256,
		asset_id: U256,
		token_address: Address,
		amount: U256,
		blueprint_selection: Vec<u64>,
	) -> EvmResult {
		let amount = Self::u256_to_amount(amount)?;

		// Get origin account.
		let msg_sender = handle.context().caller;
		let origin = Runtime::AddressMapping::into_account_id(msg_sender);

		// Parse operator address
		let operator = Self::convert_to_account_id(operator)?;

		// Parse blueprint selection
		let bounded_blueprint_selection: frame_support::BoundedVec<_, _> =
			frame_support::BoundedVec::try_from(blueprint_selection)
				.map_err(|_| revert("error converting blueprint selection"))?;

		let blueprint_selection = DelegatorBlueprintSelection::Fixed(bounded_blueprint_selection);

		let (deposit_asset, amount) = match (asset_id.as_u32(), token_address.0 .0) {
			(0, erc20_token) => (Asset::Erc20(erc20_token.into()), amount),
			(other_asset_id, ZERO_ADDRESS) => (Asset::Custom(other_asset_id.into()), amount),
			(_other_asset_id, _erc20_token) => {
				return Err(revert_custom_error(Self::PAYMENT_ASSET_SHOULD_BE_CUSTOM_OR_ERC20))
			},
		};

		// Build call with origin.
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let call = pallet_multi_asset_delegation::Call::<Runtime>::delegate {
			operator,
			asset_id: deposit_asset,
			amount,
			blueprint_selection,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("schedule_delegator_unstake(bytes32,uint256,uint256)")]
	fn schedule_delegator_unstake(
		handle: &mut impl PrecompileHandle,
		operator: H256,
		asset_id: U256,
		token_address: Address,
		amount: U256,
	) -> EvmResult {
		let amount = Self::u256_to_amount(amount)?;

		// Get origin account.
		let msg_sender = handle.context().caller;
		let origin = Runtime::AddressMapping::into_account_id(msg_sender);

		// Parse operator address
		let operator = Self::convert_to_account_id(operator)?;

		let (deposit_asset, amount) = match (asset_id.as_u32(), token_address.0 .0) {
			(0, erc20_token) => (Asset::Erc20(erc20_token.into()), amount),
			(other_asset_id, ZERO_ADDRESS) => (Asset::Custom(other_asset_id.into()), amount),
			(_other_asset_id, _erc20_token) => {
				return Err(revert_custom_error(Self::PAYMENT_ASSET_SHOULD_BE_CUSTOM_OR_ERC20))
			},
		};

		// Build call with origin.
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let call = pallet_multi_asset_delegation::Call::<Runtime>::schedule_delegator_unstake {
			operator,
			asset_id: deposit_asset,
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
		token_address: Address,
		amount: U256,
	) -> EvmResult {
		let amount = Self::u256_to_amount(amount)?;

		// Get origin account.
		let msg_sender = handle.context().caller;
		let origin = Runtime::AddressMapping::into_account_id(msg_sender);

		// Parse operator address
		let operator = Self::convert_to_account_id(operator)?;

		let (deposit_asset, amount) = match (asset_id.as_u32(), token_address.0 .0) {
			(0, erc20_token) => (Asset::Erc20(erc20_token.into()), amount),
			(other_asset_id, ZERO_ADDRESS) => (Asset::Custom(other_asset_id.into()), amount),
			(_other_asset_id, _erc20_token) => {
				return Err(revert_custom_error(Self::PAYMENT_ASSET_SHOULD_BE_CUSTOM_OR_ERC20))
			},
		};

		// Build call with origin.
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let call = pallet_multi_asset_delegation::Call::<Runtime>::cancel_delegator_unstake {
			operator,
			asset_id: deposit_asset,
			amount,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}
}

/// Revert with Custom Error Selector
fn revert_custom_error(err: [u8; 32]) -> PrecompileFailure {
	let selector = &err[0..4];
	let mut output = sp_std::vec![0u8; 32];
	output[0..4].copy_from_slice(selector);
	PrecompileFailure::Revert { exit_status: fp_evm::ExitRevert::Reverted, output }
}
