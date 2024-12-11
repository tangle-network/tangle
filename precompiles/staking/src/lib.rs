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

//! This file contains the implementation of the StakingPrecompile struct which provides an
//! interface between the EVM and the native staking pallet of the runtime. It allows EVM contracts
//! to call functions of the staking pallet, and to query its state.
//!
//! The StakingPrecompile struct implements several methods that correspond to the functions of the
//! staking pallet. These methods can be called from EVM contracts. They include functions to get
//! the current era, the minimum bond for nominators and validators, the number of validators,
//! whether a given account is a validator or nominator, and others.
//!
//! Each method records the gas cost for the operation, performs the requested operation, and
//! returns the result in a format that can be used by the EVM.
//!
//! The StakingPrecompile struct is generic over the Runtime type, which is the type of the runtime
//! that includes the staking pallet. This allows the precompile to work with any runtime that
//! includes the staking pallet and meets the other trait bounds required by the precompile.

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
use sp_runtime::traits::{Dispatchable, StaticLookup};
use sp_std::{convert::TryInto, marker::PhantomData, vec, vec::Vec};
use tangle_primitives::types::WrappedAccountId32;

type BalanceOf<Runtime> = <<Runtime as pallet_staking::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

pub struct StakingPrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> StakingPrecompile<Runtime>
where
	Runtime: pallet_staking::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_staking::Call<Runtime>>,
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
	fn convert_to_reward_destination(
		payee: H256,
	) -> EvmResult<pallet_staking::RewardDestination<Runtime::AccountId>> {
		let payee = match payee {
			H256(
				[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
			) => pallet_staking::RewardDestination::Staked,
			H256(
				[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
			) => pallet_staking::RewardDestination::Stash,

			H256(
				[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _],
			) => {
				let ethereum_address = Address(H160::from_slice(&payee.0[12..]));
				pallet_staking::RewardDestination::Account(
					Runtime::AddressMapping::into_account_id(ethereum_address.0),
				)
			},
			H256(account) => pallet_staking::RewardDestination::Account(
				Self::parse_32byte_address(account.to_vec())?,
			),
		};

		Ok(payee)
	}
}

#[precompile_utils::precompile]
impl<Runtime> StakingPrecompile<Runtime>
where
	Runtime: pallet_staking::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_staking::Call<Runtime>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
	Runtime::AccountId: From<WrappedAccountId32>,
{
	#[precompile::public("currentEra()")]
	#[precompile::public("current_era()")]
	#[precompile::view]
	fn current_era(handle: &mut impl PrecompileHandle) -> EvmResult<u32> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let current_era = pallet_staking::CurrentEra::<Runtime>::get().unwrap_or_default();

		Ok(current_era)
	}

	#[precompile::public("minNominatorBond()")]
	#[precompile::public("min_nominator_bond()")]
	#[precompile::view]
	fn min_nominator_bond(handle: &mut impl PrecompileHandle) -> EvmResult<u128> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let min_nominator_bond: u128 = pallet_staking::MinNominatorBond::<Runtime>::get()
			.try_into()
			.map_err(|_| revert("Amount is too large for provided balance type"))?;
		Ok(min_nominator_bond)
	}

	#[precompile::public("minValidatorBond()")]
	#[precompile::public("min_validator_bond()")]
	#[precompile::view]
	fn min_validator_bond(handle: &mut impl PrecompileHandle) -> EvmResult<u128> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let min_validator_bond: u128 = pallet_staking::MinValidatorBond::<Runtime>::get()
			.try_into()
			.map_err(|_| revert("Amount is too large for provided balance type"))?;
		Ok(min_validator_bond)
	}

	#[precompile::public("minActiveStake()")]
	#[precompile::public("min_active_stake()")]
	#[precompile::view]
	fn min_active_stake(handle: &mut impl PrecompileHandle) -> EvmResult<u128> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let min_active_stake: u128 = pallet_staking::MinimumActiveStake::<Runtime>::get()
			.try_into()
			.map_err(|_| revert("Amount is too large for provided balance type"))?;
		Ok(min_active_stake)
	}

	#[precompile::public("validatorCount()")]
	#[precompile::public("validator_count()")]
	#[precompile::view]
	fn validator_count(handle: &mut impl PrecompileHandle) -> EvmResult<u32> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let validator_count: u32 = pallet_staking::ValidatorCount::<Runtime>::get();
		Ok(validator_count)
	}

	#[precompile::public("maxValidatorCount()")]
	#[precompile::public("max_validator_count()")]
	#[precompile::view]
	fn max_validator_count(handle: &mut impl PrecompileHandle) -> EvmResult<u32> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let max_validator_count: u32 =
			pallet_staking::MaxValidatorsCount::<Runtime>::get().unwrap_or_default();
		Ok(max_validator_count)
	}

	#[precompile::public("maxNominatorCount()")]
	#[precompile::public("max_nominator_count()")]
	#[precompile::view]
	fn max_nominator_count(handle: &mut impl PrecompileHandle) -> EvmResult<u32> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let max_nominator_count: u32 =
			pallet_staking::MaxNominatorsCount::<Runtime>::get().unwrap_or_default();
		Ok(max_nominator_count)
	}

	#[precompile::public("isNominator(address)")]
	#[precompile::public("is_nominator(address)")]
	#[precompile::view]
	fn is_nominator(handle: &mut impl PrecompileHandle, nominator: Address) -> EvmResult<bool> {
		let nominator_account = Runtime::AddressMapping::into_account_id(nominator.0);
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_nominator = pallet_staking::Validators::<Runtime>::contains_key(nominator_account);
		Ok(is_nominator)
	}

	#[precompile::public("erasTotalStake(uint32)")]
	#[precompile::public("eras_total_stake(uint32)")]
	#[precompile::view]
	fn eras_total_stake(handle: &mut impl PrecompileHandle, era_index: u32) -> EvmResult<u128> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let total_stake: u128 = <pallet_staking::Pallet<Runtime>>::eras_total_stake(era_index)
			.try_into()
			.map_err(|_| revert("Amount is too large for provided balance type"))?;

		Ok(total_stake)
	}

	#[precompile::public("erasTotalRewardPoints(uint32)")]
	#[precompile::public("eras_total_reward_points(uint32)")]
	#[precompile::view]
	fn eras_total_reward_points(
		handle: &mut impl PrecompileHandle,
		era_index: u32,
	) -> EvmResult<u32> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let total_reward_points: u32 =
			<pallet_staking::Pallet<Runtime>>::eras_reward_points(era_index).total;
		Ok(total_reward_points)
	}

	#[precompile::public("nominate(bytes32[])")]
	fn nominate(handle: &mut impl PrecompileHandle, targets: Vec<H256>) -> EvmResult {
		handle.record_log_costs_manual(2, 32 * targets.len())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let mut converted_targets: Vec<<Runtime::Lookup as StaticLookup>::Source> = vec![];
		for tgt in targets {
			let target: Runtime::AccountId = Self::parse_32byte_address(tgt.0.to_vec())?;
			let converted_target = <Runtime::Lookup as StaticLookup>::unlookup(target);
			converted_targets.push(converted_target);
		}
		let call = pallet_staking::Call::<Runtime>::nominate { targets: converted_targets };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("bond(uint256,bytes32)")]
	fn bond(handle: &mut impl PrecompileHandle, value: U256, payee: H256) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let value: BalanceOf<Runtime> = value
			.try_into()
			.map_err(|_| revert("Value is too large for provided balance type"))?;
		let payee = Self::convert_to_reward_destination(payee)?;

		let call = pallet_staking::Call::<Runtime>::bond { value, payee };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("bondExtra(uint256)")]
	#[precompile::public("bond_extra(uint256)")]
	fn bond_extra(handle: &mut impl PrecompileHandle, max_additional: U256) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let max_additional: BalanceOf<Runtime> = max_additional
			.try_into()
			.map_err(|_| revert("Value is too large for provided balance type"))?;

		let call = pallet_staking::Call::<Runtime>::bond_extra { max_additional };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("unbond(uint256)")]
	fn unbond(handle: &mut impl PrecompileHandle, value: U256) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let value: BalanceOf<Runtime> = value
			.try_into()
			.map_err(|_| revert("Value is too large for provided balance type"))?;

		let call = pallet_staking::Call::<Runtime>::unbond { value };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("withdrawUnbonded(uint32)")]
	#[precompile::public("withdraw_unbonded(uint32)")]
	fn withdraw_unbonded(handle: &mut impl PrecompileHandle, num_slashing_spans: u32) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = pallet_staking::Call::<Runtime>::withdraw_unbonded { num_slashing_spans };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("chill()")]
	fn chill(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = pallet_staking::Call::<Runtime>::chill {};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("setPayee(uint8)")]
	#[precompile::public("set_payee(uint8)")]
	fn set_payee(handle: &mut impl PrecompileHandle, payee: u8) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let payee = match payee {
			1 => pallet_staking::RewardDestination::Staked,
			2 => pallet_staking::RewardDestination::Stash,
			_ => return Err(revert("Invalid payee")),
		};

		let call = pallet_staking::Call::<Runtime>::set_payee { payee };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// (Re-)sets the controller of a stash to the stash itself. This function previously
	/// accepted a `controller` argument to set the controller to an account other than the
	/// stash itself. This functionality has now been removed, now only setting the controller
	/// to the stash, if it is not already.
	#[precompile::public("setController()")]
	#[precompile::public("set_controller()")]
	fn set_controller(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = pallet_staking::Call::<Runtime>::set_controller {};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("payoutStakers(bytes32,uint32)")]
	#[precompile::public("payout_stakers(bytes32,uint32)")]
	fn payout_stakers(
		handle: &mut impl PrecompileHandle,
		validator_stash: H256,
		era: u32,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let validator_stash: Runtime::AccountId =
			Self::parse_32byte_address(validator_stash.0.to_vec())?;

		let call = pallet_staking::Call::<Runtime>::payout_stakers { validator_stash, era };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("rebond(uint256)")]
	fn rebond(handle: &mut impl PrecompileHandle, value: U256) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let value: BalanceOf<Runtime> = value
			.try_into()
			.map_err(|_| revert("Value is too large for provided balance type"))?;

		let call = pallet_staking::Call::<Runtime>::rebond { value };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}
}
