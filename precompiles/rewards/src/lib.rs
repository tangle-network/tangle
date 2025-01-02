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

#[cfg(any(test, feature = "fuzzing"))]
pub mod mock;
#[cfg(test)]
mod tests;

use fp_evm::PrecompileHandle;
use frame_support::{
    dispatch::{GetDispatchInfo, PostDispatchInfo},
    traits::Currency,
};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{H160, U256};
use sp_runtime::traits::Dispatchable;
use sp_std::{marker::PhantomData, vec::Vec};
use tangle_primitives::{
    services::Asset,
    types::{rewards::{LockMultiplier, RewardType}, WrappedAccountId32},
};

type BalanceOf<Runtime> =
    <<Runtime as pallet_rewards::Config>::Currency as Currency<
        <Runtime as frame_system::Config>::AccountId,
    >>::Balance;

type AssetIdOf<Runtime> = <Runtime as pallet_rewards::Config>::AssetId;

pub struct RewardsPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> RewardsPrecompile<Runtime>
where
    Runtime: pallet_rewards::Config + pallet_evm::Config,
    Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    <Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
    Runtime::RuntimeCall: From<pallet_rewards::Call<Runtime>>,
    BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
    AssetIdOf<Runtime>: TryFrom<U256> + Into<U256> + From<u128>,
    Runtime::AccountId: From<WrappedAccountId32>,
{
    /// Set APY for an asset (admin only)
    #[precompile::public("setAssetApy(uint256,address,uint256)")]
    fn set_asset_apy(
        handle: &mut impl PrecompileHandle,
        asset_id: U256,
        token_address: Address,
        apy_basis_points: U256,
    ) -> EvmResult {
        handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
        let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
        
        let asset = if token_address == Address::zero() {
            Asset::Custom(
                asset_id.try_into().map_err(|_| revert("Invalid asset ID"))?,
            )
        } else {
            Asset::Erc20(token_address.into())
        };

        let apy: u32 = apy_basis_points
            .try_into()
            .map_err(|_| revert("Invalid APY basis points"))?;

        let call = pallet_rewards::Call::<Runtime>::set_asset_apy { 
            asset,
            apy_basis_points: apy,
        };

        RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;
        Ok(())
    }

    /// Claim rewards for a specific asset and reward type
    #[precompile::public("claimRewards(uint256,address,uint8)")]
    fn claim_rewards(
        handle: &mut impl PrecompileHandle,
        asset_id: U256,
        token_address: Address,
        reward_type: u8,
    ) -> EvmResult {
        handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
        let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
        
        let asset = if token_address == Address::zero() {
            Asset::Custom(
                asset_id.try_into().map_err(|_| revert("Invalid asset ID"))?,
            )
        } else {
            Asset::Erc20(token_address.into())
        };

        let reward_type = match reward_type {
            0 => RewardType::Boost,
            1 => RewardType::Service,
            2 => RewardType::Restaking,
            _ => return Err(revert("Invalid reward type")),
        };

        let call = pallet_rewards::Call::<Runtime>::claim_rewards { 
            asset,
            reward_type,
        };

        RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;
        Ok(())
    }

    /// Get user's reward info for an asset
    #[precompile::public("getUserRewards(address,uint256,address)")]
    fn get_user_rewards(
        handle: &mut impl PrecompileHandle,
        user: Address,
        asset_id: U256,
        token_address: Address,
    ) -> EvmResult<(U256, U256, U256, U256)> {
        handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
        
        let user = Runtime::AddressMapping::into_account_id(user.into());
        let asset = if token_address == Address::zero() {
            Asset::Custom(
                asset_id.try_into().map_err(|_| revert("Invalid asset ID"))?,
            )
        } else {
            Asset::Erc20(token_address.into())
        };

        let rewards = pallet_rewards::Pallet::<Runtime>::user_rewards(&user, asset);
        
        Ok((
            rewards.boost_rewards.amount.into(),
            rewards.boost_rewards.expiry.into(),
            rewards.service_rewards.into(),
            rewards.restaking_rewards.into(),
        ))
    }

    /// Get asset APY and capacity
    #[precompile::public("getAssetInfo(uint256,address)")]
    fn get_asset_info(
        handle: &mut impl PrecompileHandle,
        asset_id: U256,
        token_address: Address,
    ) -> EvmResult<(U256, U256)> {
        handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
        
        let asset = if token_address == Address::zero() {
            Asset::Custom(
                asset_id.try_into().map_err(|_| revert("Invalid asset ID"))?,
            )
        } else {
            Asset::Erc20(token_address.into())
        };

        let apy = pallet_rewards::Pallet::<Runtime>::asset_apy(asset);
        let capacity = pallet_rewards::Pallet::<Runtime>::asset_capacity(asset);
        
        Ok((apy.into(), capacity.into()))
    }

    /// Update APY for an asset (admin only)
    #[precompile::public("updateAssetApy(uint256,address,uint32)")]
    fn update_asset_apy(
        handle: &mut impl PrecompileHandle,
        asset_id: U256,
        token_address: Address,
        apy_basis_points: u32,
    ) -> EvmResult {
        handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
        let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
        
        let asset = if token_address == Address::zero() {
            Asset::Custom(
                asset_id.try_into().map_err(|_| revert("Invalid asset ID"))?,
            )
        } else {
            Asset::Erc20(token_address.into())
        };

        let call = pallet_rewards::Call::<Runtime>::update_asset_apy { 
            asset,
            apy_basis_points,
        };

        RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mock::*;
    use precompile_utils::testing::*;
    use sp_core::H160;

    fn precompiles() -> TestPrecompileSet<Runtime> {
        PrecompilesValue::get()
    }

    #[test]
    fn test_solidity_interface_has_all_function_selectors_documented() {
        for file in ["Rewards.sol"] {
            precompiles()
                .process_selectors(file, |fn_selector, fn_signature| {
                    assert!(
                        DOCUMENTED_FUNCTIONS.contains(&fn_selector),
                        "documented_functions must contain {fn_selector:?} ({fn_signature})",
                    );
                });
        }
    }
}
