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

//! This file contains the implementation of the MultiAssetDelegationPrecompile struct which provides an
//! interface between the EVM and the native MultiAssetDelegation pallet of the runtime. It allows EVM contracts
//! to call functions of the MultiAssetDelegation pallet, in order to enable EVM accounts to interact with the delegation system.
//!
//! The MultiAssetDelegationPrecompile struct implements core methods that correspond to the functions of the
//! MultiAssetDelegation pallet. These methods can be called from EVM contracts. They include functions to join as an operator,
//! delegate assets, withdraw assets, etc.
//!
//! Each method records the gas cost for the operation, performs the requested operation, and
//! returns the result in a format that can be used by the EVM.
//!
//! The MultiAssetDelegationPrecompile struct is generic over the Runtime type, which is the type of the runtime
//! that includes the MultiAssetDelegation pallet. This allows the precompile to work with any runtime that
//! includes the MultiAssetDelegation pallet and meets the other trait bounds required by the precompile.

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

use pallet_tangle_lst::{BondExtra, PoolId, PoolState};
use sp_runtime::Perbill;

pub struct TangleLstPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> TangleLstPrecompile<Runtime>
where
	Runtime: pallet_tangle_lst::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_tangle_lst::Call<Runtime>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
	Runtime::AccountId: From<WrappedAccountId32>,
{
	#[precompile::public("join(uint256,uint256)")]
	fn join(handle: &mut impl PrecompileHandle, amount: U256, pool_id: U256) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let amount: BalanceOf<Runtime> = amount.try_into().map_err(|_| revert("Invalid amount"))?;
		let pool_id: PoolId = pool_id.try_into().map_err(|_| revert("Invalid pool id"))?;

		let call = pallet_tangle_lst::Call::<Runtime>::join { amount, pool_id };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("bondExtra(uint256,uint8,uint256)")]
	fn bond_extra(
		handle: &mut impl PrecompileHandle,
		pool_id: U256,
		extra_type: u8,
		extra: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let pool_id: PoolId = pool_id.try_into().map_err(|_| revert("Invalid pool id"))?;
		let extra: BalanceOf<Runtime> =
			extra.try_into().map_err(|_| revert("Invalid extra amount"))?;

		let extra = match extra_type {
			0 => BondExtra::FreeBalance(extra),
			1 => BondExtra::Rewards,
			_ => return Err(revert("Invalid extra type")),
		};

		let call = pallet_tangle_lst::Call::<Runtime>::bond_extra { pool_id, extra };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("unbond(bytes32,uint256,uint256)")]
	fn unbond(
		handle: &mut impl PrecompileHandle,
		member_account: H256,
		pool_id: U256,
		unbonding_points: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let member_account = Self::convert_to_account_id(member_account)?;
		let pool_id: PoolId = pool_id.try_into().map_err(|_| revert("Invalid pool id"))?;
		let unbonding_points: BalanceOf<Runtime> =
			unbonding_points.try_into().map_err(|_| revert("Invalid unbonding points"))?;

		let call = pallet_tangle_lst::Call::<Runtime>::unbond {
			member_account,
			pool_id,
			unbonding_points,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("poolWithdrawUnbonded(uint256,uint32)")]
	fn pool_withdraw_unbonded(
		handle: &mut impl PrecompileHandle,
		pool_id: U256,
		num_slashing_spans: u32,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let pool_id: PoolId = pool_id.try_into().map_err(|_| revert("Invalid pool id"))?;

		let call = pallet_tangle_lst::Call::<Runtime>::pool_withdraw_unbonded {
			pool_id,
			num_slashing_spans,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("withdrawUnbonded(bytes32,uint256,uint32)")]
	fn withdraw_unbonded(
		handle: &mut impl PrecompileHandle,
		member_account: H256,
		pool_id: U256,
		num_slashing_spans: u32,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let member_account = Self::convert_to_account_id(member_account)?;
		let pool_id: PoolId = pool_id.try_into().map_err(|_| revert("Invalid pool id"))?;

		let call = pallet_tangle_lst::Call::<Runtime>::withdraw_unbonded {
			member_account,
			pool_id,
			num_slashing_spans,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("create(uint256,bytes32,bytes32,bytes32)")]
	fn create(
		handle: &mut impl PrecompileHandle,
		amount: U256,
		root: H256,
		nominator: H256,
		bouncer: H256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let amount: BalanceOf<Runtime> = amount.try_into().map_err(|_| revert("Invalid amount"))?;
		let root = Self::convert_to_account_id(root)?;
		let nominator = Self::convert_to_account_id(nominator)?;
		let bouncer = Self::convert_to_account_id(bouncer)?;

		let call = pallet_tangle_lst::Call::<Runtime>::create { amount, root, nominator, bouncer };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("createWithPoolId(uint256,bytes32,bytes32,bytes32,uint256)")]
	fn create_with_pool_id(
		handle: &mut impl PrecompileHandle,
		amount: U256,
		root: H256,
		nominator: H256,
		bouncer: H256,
		pool_id: U256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let amount: BalanceOf<Runtime> = amount.try_into().map_err(|_| revert("Invalid amount"))?;
		let root = Self::convert_to_account_id(root)?;
		let nominator = Self::convert_to_account_id(nominator)?;
		let bouncer = Self::convert_to_account_id(bouncer)?;
		let pool_id: PoolId = pool_id.try_into().map_err(|_| revert("Invalid pool id"))?;

		let call = pallet_tangle_lst::Call::<Runtime>::create_with_pool_id {
			amount,
			root,
			nominator,
			bouncer,
			pool_id,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("nominate(uint256,bytes32[])")]
	fn nominate(
		handle: &mut impl PrecompileHandle,
		pool_id: U256,
		validators: Vec<H256>,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let pool_id: PoolId = pool_id.try_into().map_err(|_| revert("Invalid pool id"))?;
		let validators: Vec<Runtime::AccountId> = validators
			.into_iter()
			.map(Self::convert_to_account_id)
			.collect::<Result<_, _>>()?;

		let call = pallet_tangle_lst::Call::<Runtime>::nominate { pool_id, validators };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("setState(uint256,uint8)")]
	fn set_state(handle: &mut impl PrecompileHandle, pool_id: U256, state: u8) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let pool_id: PoolId = pool_id.try_into().map_err(|_| revert("Invalid pool id"))?;
		let state = match state {
			0 => PoolState::Open,
			1 => PoolState::Blocked,
			2 => PoolState::Destroying,
			_ => return Err(revert("Invalid state")),
		};

		let call = pallet_tangle_lst::Call::<Runtime>::set_state { pool_id, state };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("setMetadata(uint256,bytes)")]
	fn set_metadata(
		handle: &mut impl PrecompileHandle,
		pool_id: U256,
		metadata: Vec<u8>,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let pool_id: PoolId = pool_id.try_into().map_err(|_| revert("Invalid pool id"))?;

		let call = pallet_tangle_lst::Call::<Runtime>::set_metadata { pool_id, metadata };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("setConfigs(uint256,uint256,uint32,uint32)")]
	fn set_configs(
		handle: &mut impl PrecompileHandle,
		min_join_bond: U256,
		min_create_bond: U256,
		max_pools: u32,
		global_max_commission: u32,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		ensure_root(handle)?;

		let min_join_bond: Option<BalanceOf<Runtime>> = if min_join_bond == U256::zero() {
			None
		} else {
			Some(min_join_bond.try_into().map_err(|_| revert("Invalid min join bond"))?)
		};

		let min_create_bond: Option<BalanceOf<Runtime>> = if min_create_bond == U256::zero() {
			None
		} else {
			Some(min_create_bond.try_into().map_err(|_| revert("Invalid min create bond"))?)
		};

		let max_pools = if max_pools == 0 { None } else { Some(max_pools) };

		let global_max_commission = if global_max_commission == 0 {
			None
		} else {
			Some(Perbill::from_percent(global_max_commission))
		};

		let call = pallet_tangle_lst::Call::<Runtime>::set_configs {
			min_join_bond: min_join_bond.map(ConfigOp::Set).unwrap_or(ConfigOp::Noop),
			min_create_bond: min_create_bond.map(ConfigOp::Set).unwrap_or(ConfigOp::Noop),
			max_pools: max_pools.map(ConfigOp::Set).unwrap_or(ConfigOp::Noop),
			global_max_commission: global_max_commission
				.map(ConfigOp::Set)
				.unwrap_or(ConfigOp::Noop),
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, RawOrigin::Root.into(), call)?;

		Ok(())
	}
}
