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

#[cfg(any(test, feature = "fuzzing"))]
pub mod mock;
#[cfg(any(test, feature = "fuzzing"))]
pub mod mock_evm;
#[cfg(test)]
mod tests;
use tangle_primitives::types::rewards::LockMultiplier;

use evm_erc20_utils::*;
use fp_evm::PrecompileHandle;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::Currency,
};
use pallet_evm::AddressMapping;
use pallet_multi_asset_delegation::types::DelegatorBlueprintSelection;
use precompile_utils::prelude::*;
use sp_core::{H160, H256, U256};
use sp_runtime::traits::Dispatchable;
use sp_std::{marker::PhantomData, vec::Vec};
use tangle_primitives::{services::Asset, types::WrappedAccountId32};

type BalanceOf<Runtime> =
	<<Runtime as pallet_multi_asset_delegation::Config>::Currency as Currency<
		<Runtime as frame_system::Config>::AccountId,
	>>::Balance;

type AssetIdOf<Runtime> = <Runtime as pallet_multi_asset_delegation::Config>::AssetId;

pub struct MultiAssetDelegationPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> MultiAssetDelegationPrecompile<Runtime>
where
	Runtime: pallet_multi_asset_delegation::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_multi_asset_delegation::Call<Runtime>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
	AssetIdOf<Runtime>: TryFrom<U256> + Into<U256> + From<u128>,
	Runtime::AccountId: From<WrappedAccountId32>,
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

	#[precompile::public("executeWithdraw()")]
	fn execute_withdraw(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let caller = handle.context().caller;
		let who = Runtime::AddressMapping::into_account_id(caller);

		let pallet_account_id = pallet_multi_asset_delegation::Pallet::<Runtime>::pallet_account();
		let pallet_address = pallet_multi_asset_delegation::Pallet::<Runtime>::pallet_evm_account();

		let snapshot =
			pallet_multi_asset_delegation::Pallet::<Runtime>::ready_withdraw_requests(&who)
				.map_err(|_| revert("Failed to get ready withdraw requests"))?;

		let erc20_transfers = snapshot.filter_map(|request| match request.asset_id {
			Asset::Erc20(token) => Some((token, request.amount)),
			_ => None,
		});

		for (token, amount) in erc20_transfers {
			let v: U256 = amount.try_into().map_err(|_| revert("Invalid amount"))?;
			if !erc20_transfer(handle, token.into(), pallet_address.into(), caller.into(), v)? {
				return Err(revert("Failed to transfer ERC20 tokens"));
			}
		}

		let call = pallet_multi_asset_delegation::Call::<Runtime>::execute_withdraw {
			evm_address: Some(caller),
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(pallet_account_id).into(), call)?;

		// TODO: Execute the ERC20 Transfers here.

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

	#[precompile::public("deposit(uint256,address,uint256,uint8)")]
	fn deposit(
		handle: &mut impl PrecompileHandle,
		asset_id: U256,
		token_address: Address,
		amount: U256,
		lock_multiplier: u8,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let caller = handle.context().caller;

		let (who, deposit_asset, amount) = match (asset_id.as_u128(), token_address.0 .0) {
			(0, erc20_token) if erc20_token != [0; 20] => {
				let who = pallet_multi_asset_delegation::Pallet::<Runtime>::pallet_account();
				let pallet_address =
					pallet_multi_asset_delegation::Pallet::<Runtime>::pallet_evm_account();
				erc20_transfer(
					handle,
					token_address,
					caller.into(),
					pallet_address.into(),
					amount,
				)?;
				(who, Asset::Erc20(erc20_token.into()), amount)
			},
			(other_asset_id, _) => (
				Runtime::AddressMapping::into_account_id(caller),
				Asset::Custom(other_asset_id.into()),
				amount,
			),
		};

		let lock_multiplier = match lock_multiplier {
			0 => None,
			1 => Some(LockMultiplier::OneMonth),
			2 => Some(LockMultiplier::TwoMonths),
			3 => Some(LockMultiplier::ThreeMonths),
			4 => Some(LockMultiplier::SixMonths),
			_ => return Err(RevertReason::custom("Invalid lock multiplier").into()),
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(who).into(),
			pallet_multi_asset_delegation::Call::<Runtime>::deposit {
				asset_id: deposit_asset,
				amount: amount
					.try_into()
					.map_err(|_| RevertReason::value_is_too_large("amount"))?,
				evm_address: Some(caller),
				lock_multiplier,
			},
		)?;

		Ok(())
	}

	#[precompile::public("scheduleWithdraw(uint256,address,uint256)")]
	fn schedule_withdraw(
		handle: &mut impl PrecompileHandle,
		asset_id: U256,
		token_address: Address,
		amount: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let caller = handle.context().caller;

		let (who, deposit_asset, amount) = match (asset_id.as_u128(), token_address.0 .0) {
			(0, erc20_token) if erc20_token != [0; 20] => {
				let who = pallet_multi_asset_delegation::Pallet::<Runtime>::pallet_account();
				(who, Asset::Erc20(erc20_token.into()), amount)
			},
			(other_asset_id, _) => (
				Runtime::AddressMapping::into_account_id(caller),
				Asset::Custom(other_asset_id.into()),
				amount,
			),
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(who).into(),
			pallet_multi_asset_delegation::Call::<Runtime>::schedule_withdraw {
				asset_id: deposit_asset,
				amount: amount
					.try_into()
					.map_err(|_| RevertReason::value_is_too_large("amount"))?,
			},
		)?;

		Ok(())
	}

	#[precompile::public("cancelWithdraw(uint256,address,uint256)")]
	fn cancel_withdraw(
		handle: &mut impl PrecompileHandle,
		asset_id: U256,
		token_address: Address,
		amount: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let caller = handle.context().caller;
		let who = Runtime::AddressMapping::into_account_id(caller);

		let (deposit_asset, amount) = match (asset_id.as_u128(), token_address.0 .0) {
			(0, erc20_token) if erc20_token != [0; 20] => {
				(Asset::Erc20(erc20_token.into()), amount)
			},
			(other_asset_id, _) => (Asset::Custom(other_asset_id.into()), amount),
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(who).into(),
			pallet_multi_asset_delegation::Call::<Runtime>::cancel_withdraw {
				asset_id: deposit_asset,
				amount: amount
					.try_into()
					.map_err(|_| RevertReason::value_is_too_large("amount"))?,
			},
		)?;

		Ok(())
	}

	#[precompile::public("delegate(bytes32,uint256,address,uint256,uint64[])")]
	fn delegate(
		handle: &mut impl PrecompileHandle,
		operator: H256,
		asset_id: U256,
		token_address: Address,
		amount: U256,
		blueprint_selection: Vec<u64>,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let caller = handle.context().caller;
		let who = Runtime::AddressMapping::into_account_id(caller);
		let operator =
			Runtime::AddressMapping::into_account_id(H160::from_slice(&operator.0[12..]));

		let (deposit_asset, amount) = match (asset_id.as_u128(), token_address.0 .0) {
			(0, erc20_token) if erc20_token != [0; 20] => {
				(Asset::Erc20(erc20_token.into()), amount)
			},
			(other_asset_id, _) => (Asset::Custom(other_asset_id.into()), amount),
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(who).into(),
			pallet_multi_asset_delegation::Call::<Runtime>::delegate {
				operator,
				asset_id: deposit_asset,
				amount: amount
					.try_into()
					.map_err(|_| RevertReason::value_is_too_large("amount"))?,
				blueprint_selection: DelegatorBlueprintSelection::Fixed(
					blueprint_selection.try_into().map_err(|_| {
						RevertReason::custom("Too many blueprint ids for fixed selection")
					})?,
				),
			},
		)?;

		Ok(())
	}

	#[precompile::public("scheduleDelegatorUnstake(bytes32,uint256,address,uint256)")]
	fn schedule_delegator_unstake(
		handle: &mut impl PrecompileHandle,
		operator: H256,
		asset_id: U256,
		token_address: Address,
		amount: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let caller = handle.context().caller;
		let who = Runtime::AddressMapping::into_account_id(caller);
		let operator =
			Runtime::AddressMapping::into_account_id(H160::from_slice(&operator.0[12..]));

		let (deposit_asset, amount) = match (asset_id.as_u128(), token_address.0 .0) {
			(0, erc20_token) if erc20_token != [0; 20] => {
				(Asset::Erc20(erc20_token.into()), amount)
			},
			(other_asset_id, _) => (Asset::Custom(other_asset_id.into()), amount),
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(who).into(),
			pallet_multi_asset_delegation::Call::<Runtime>::schedule_delegator_unstake {
				operator,
				asset_id: deposit_asset,
				amount: amount
					.try_into()
					.map_err(|_| RevertReason::value_is_too_large("amount"))?,
			},
		)?;

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

	#[precompile::public("cancelDelegatorUnstake(bytes32,uint256,address,uint256)")]
	fn cancel_delegator_unstake(
		handle: &mut impl PrecompileHandle,
		operator: H256,
		asset_id: U256,
		token_address: Address,
		amount: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let caller = handle.context().caller;
		let who = Runtime::AddressMapping::into_account_id(caller);
		let operator =
			Runtime::AddressMapping::into_account_id(H160::from_slice(&operator.0[12..]));

		let (deposit_asset, amount) = match (asset_id.as_u128(), token_address.0 .0) {
			(0, erc20_token) if erc20_token != [0; 20] => {
				(Asset::Erc20(erc20_token.into()), amount)
			},
			(other_asset_id, _) => (Asset::Custom(other_asset_id.into()), amount),
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(who).into(),
			pallet_multi_asset_delegation::Call::<Runtime>::cancel_delegator_unstake {
				operator,
				asset_id: deposit_asset,
				amount: amount
					.try_into()
					.map_err(|_| RevertReason::value_is_too_large("amount"))?,
			},
		)?;

		Ok(())
	}
}
