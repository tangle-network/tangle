// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
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

use core::marker::PhantomData;
use frame_support::traits::Currency;
use fp_evm::{PrecompileHandle, PrecompileOutput};
use pallet_evm::Precompile;
use pallet_rewards::Config;
use precompile_utils::{
	prelude::*,
	solidity::{
		codec::{Address, BoundedVec},
		modifier::FunctionModifier,
		revert::InjectBacktrace,
	},
};
use sp_core::{H160, U256};
use sp_runtime::traits::StaticLookup;
use sp_std::{marker::PhantomData, prelude::*};
use tangle_primitives::services::Asset;

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_REWARDS_CLAIMED: [u8; 32] = keccak256!("RewardsClaimed(address,uint256)");

/// A precompile to wrap the functionality from pallet-rewards.
pub struct RewardsPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> RewardsPrecompile<Runtime>
where
	Runtime: Config + pallet_evm::Config,
	Runtime::AccountId: From<H160> + Into<H160>,
{
	#[precompile::public("claimRewards(uint256,address)")]
	fn claim_rewards(
		handle: &mut impl PrecompileHandle,
		asset_id: U256,
		token_address: Address,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let caller = handle.context().caller;
		let who = Runtime::AddressMapping::into_account_id(caller);

		let (asset, _) = match (asset_id.as_u128(), token_address.0 .0) {
			(0, erc20_token) if erc20_token != [0; 20] => {
				(Asset::Erc20(erc20_token.into()), U256::zero())
			},
			(other_asset_id, _) => (Asset::Custom(other_asset_id.into()), U256::zero()),
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(who).into(),
			pallet_rewards::Call::<Runtime>::claim_rewards { asset },
		)?;

		Ok(())
	}

	#[precompile::public("forceClaimRewards(address,uint256,address)")]
	fn force_claim_rewards(
		handle: &mut impl PrecompileHandle,
		account: Address,
		asset_id: U256,
		token_address: Address,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let caller = handle.context().caller;
		let who = Runtime::AddressMapping::into_account_id(caller);
		let target = Runtime::AddressMapping::into_account_id(account.0);

		let (asset, _) = match (asset_id.as_u128(), token_address.0 .0) {
			(0, erc20_token) if erc20_token != [0; 20] => {
				(Asset::Erc20(erc20_token.into()), U256::zero())
			},
			(other_asset_id, _) => (Asset::Custom(other_asset_id.into()), U256::zero()),
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(who).into(),
			pallet_rewards::Call::<Runtime>::force_claim_rewards { account: target, asset },
		)?;

		Ok(())
	}

	#[precompile::public("updateVaultRewardConfig(uint256,uint8,uint256,uint256,uint32)")]
	fn update_vault_reward_config(
		handle: &mut impl PrecompileHandle,
		vault_id: U256,
		apy: u8,
		deposit_cap: U256,
		incentive_cap: U256,
		boost_multiplier: u32,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let caller = handle.context().caller;
		let who = Runtime::AddressMapping::into_account_id(caller);

		let config = pallet_rewards::RewardConfigForAssetVault {
			apy: sp_runtime::Percent::from_percent(apy.min(100)),
			deposit_cap: deposit_cap.try_into().map_err(|_| RevertReason::value_is_too_large("deposit_cap"))?,
			incentive_cap: incentive_cap.try_into().map_err(|_| RevertReason::value_is_too_large("incentive_cap"))?,
			boost_multiplier: Some(boost_multiplier.min(500)), // Cap at 5x
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(who).into(),
			pallet_rewards::Call::<Runtime>::udpate_vault_reward_config {
				vault_id: vault_id.try_into().map_err(|_| RevertReason::value_is_too_large("vault_id"))?,
				new_config: config,
			},
		)?;

		Ok(())
	}

	#[precompile::public("manageAssetRewardVault(uint256,uint256,address,bool)")]
	fn manage_asset_reward_vault(
		handle: &mut impl PrecompileHandle,
		vault_id: U256,
		asset_id: U256,
		token_address: Address,
		add: bool,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let caller = handle.context().caller;
		let who = Runtime::AddressMapping::into_account_id(caller);

		let (asset, _) = match (asset_id.as_u128(), token_address.0 .0) {
			(0, erc20_token) if erc20_token != [0; 20] => {
				(Asset::Erc20(erc20_token.into()), U256::zero())
			},
			(other_asset_id, _) => (Asset::Custom(other_asset_id.into()), U256::zero()),
		};

		let action = if add {
			pallet_rewards::AssetAction::Add
		} else {
			pallet_rewards::AssetAction::Remove
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(who).into(),
			pallet_rewards::Call::<Runtime>::manage_asset_reward_vault {
				vault_id: vault_id.try_into().map_err(|_| RevertReason::value_is_too_large("vault_id"))?,
				asset_id: asset,
				action,
			},
		)?;

		Ok(())
	}
}
