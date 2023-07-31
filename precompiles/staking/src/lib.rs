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

#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::PrecompileHandle;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::Currency,
};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{H160, U256};
use sp_std::{convert::TryInto, marker::PhantomData};

type BalanceOf<Runtime> = <<Runtime as pallet_staking::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

pub struct StakingPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> StakingPrecompile<Runtime>
where
	Runtime: pallet_staking::Config + pallet_evm::Config,
	Runtime::AccountId: Into<H160>,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_staking::Call<Runtime>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
{
	#[precompile::public("currentEra()")]
	#[precompile::view]
	fn current_era(handle: &mut impl PrecompileHandle) -> EvmResult<u32> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let current_era = pallet_staking::CurrentEra::<Runtime>::get().unwrap_or_default();

		Ok(current_era)
	}

	#[precompile::public("minNominatorBond()")]
	#[precompile::view]
	fn min_nominator_bond(handle: &mut impl PrecompileHandle) -> EvmResult<u128> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let min_nominator_bond: u128 = pallet_staking::MinNominatorBond::<Runtime>::get()
			.try_into()
			.map_err(|_| revert("Amount is too large for provided balance type"))?;
		Ok(min_nominator_bond)
	}

	#[precompile::public("minValidatorBond()")]
	#[precompile::view]
	fn min_validator_bond(handle: &mut impl PrecompileHandle) -> EvmResult<u128> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let min_validator_bond: u128 = pallet_staking::MinValidatorBond::<Runtime>::get()
			.try_into()
			.map_err(|_| revert("Amount is too large for provided balance type"))?;
		Ok(min_validator_bond)
	}

	#[precompile::public("minActiveStake()")]
	#[precompile::view]
	fn min_active_stake(handle: &mut impl PrecompileHandle) -> EvmResult<u128> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let min_active_stake: u128 = pallet_staking::MinimumActiveStake::<Runtime>::get()
			.try_into()
			.map_err(|_| revert("Amount is too large for provided balance type"))?;
		Ok(min_active_stake)
	}

	#[precompile::public("validatorCount()")]
	#[precompile::view]
	fn validator_count(handle: &mut impl PrecompileHandle) -> EvmResult<u32> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let validator_count: u32 = pallet_staking::ValidatorCount::<Runtime>::get();
		Ok(validator_count)
	}

	#[precompile::public("maxValidatorCount()")]
	#[precompile::view]
	fn max_validator_count(handle: &mut impl PrecompileHandle) -> EvmResult<u32> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let max_validator_count: u32 =
			pallet_staking::MaxValidatorsCount::<Runtime>::get().unwrap_or_default();
		Ok(max_validator_count)
	}

	#[precompile::public("isValidator(address)")]
	#[precompile::view]
	fn is_validators(handle: &mut impl PrecompileHandle, validator: Address) -> EvmResult<bool> {
		let validator_account = Runtime::AddressMapping::into_account_id(validator.0);
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_validator = pallet_staking::Validators::<Runtime>::contains_key(validator_account);
		Ok(is_validator)
	}

	#[precompile::public("maxNominatorCount()")]
	#[precompile::view]
	fn max_nominator_count(handle: &mut impl PrecompileHandle) -> EvmResult<u32> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let max_nominator_count: u32 =
			pallet_staking::MaxNominatorsCount::<Runtime>::get().unwrap_or_default();
		Ok(max_nominator_count)
	}

	#[precompile::public("isNominator(address)")]
	#[precompile::view]
	fn is_nominator(handle: &mut impl PrecompileHandle, nominator: Address) -> EvmResult<bool> {
		let nominator_account = Runtime::AddressMapping::into_account_id(nominator.0);
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_nominator = pallet_staking::Validators::<Runtime>::contains_key(nominator_account);
		Ok(is_nominator)
	}

	#[precompile::public("erasTotalStake(uint32)")]
	#[precompile::view]
	fn eras_total_stake(handle: &mut impl PrecompileHandle, era_index: u32) -> EvmResult<u128> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let total_stake: u128 = <pallet_staking::Pallet<Runtime>>::eras_total_stake(era_index)
			.try_into()
			.map_err(|_| revert("Amount is too large for provided balance type"))?;

		Ok(total_stake)
	}
}
