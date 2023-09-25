// Copyright 2022 Webb Technologies Inc.
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
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::Currency,
};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{H256, U256};
use sp_runtime::traits::StaticLookup;
use sp_std::{convert::TryInto, marker::PhantomData};

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
	Runtime::AccountId: From<[u8; 32]>,
{
	/// Helper method to parse H160 or SS58 address
	fn parse_input_address(staker_vec: Vec<u8>) -> EvmResult<Runtime::AccountId> {
		let staker: Runtime::AccountId = match staker_vec.len() {
			// public address of the ss58 account has 32 bytes
			32 => {
				let mut staker_bytes = [0_u8; 32];
				staker_bytes[..].clone_from_slice(&staker_vec[0..32]);

				staker_bytes.into()
			},
			// public address of the H160 account has 20 bytes
			20 => {
				let mut staker_bytes = [0_u8; 20];
				staker_bytes[..].clone_from_slice(&staker_vec[0..20]);

				Runtime::AddressMapping::into_account_id(staker_bytes.into())
			},
			_ => {
				// Return err if account length is wrong
				return Err(revert("Error while parsing staker's address"))
			},
		};

		Ok(staker)
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
	Runtime::AccountId: From<[u8; 32]>,
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

	#[precompile::public("isValidator(address)")]
	#[precompile::public("is_validator(address)")]
	#[precompile::view]
	fn is_validator(handle: &mut impl PrecompileHandle, validator: Address) -> EvmResult<bool> {
		let validator_account = Runtime::AddressMapping::into_account_id(validator.0);
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_validator = pallet_staking::Validators::<Runtime>::contains_key(validator_account);
		Ok(is_validator)
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

	#[precompile::public("nominate(address[])")]
	fn nominate(handle: &mut impl PrecompileHandle, targets: Vec<H256>) -> EvmResult {
		handle.record_log_costs_manual(2, 32 * targets.len())?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let mut converted_targets: Vec<<Runtime::Lookup as StaticLookup>::Source> = vec![];
		for i in 0..targets.len() {
			let target: Runtime::AccountId = Self::parse_input_address(targets[i].0.to_vec())?;
			let converted_target = <Runtime::Lookup as StaticLookup>::unlookup(target);
			converted_targets.push(converted_target);
		}
		let call = pallet_staking::Call::<Runtime>::nominate { targets: converted_targets };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}
}
