// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.

//! # Running
//! Running this fuzzer can be done with `cargo hfuzz run rewards-fuzzer`. `honggfuzz` CLI
//! options can be used by setting `HFUZZ_RUN_ARGS`, such as `-n 4` to use 4 threads.
//!
//! # Debugging a panic
//! Once a panic is found, it can be debugged with
//! `cargo hfuzz run-debug rewards-fuzzer hfuzz_workspace/rewards-fuzzer/*.fuzz`.

use crate::runtime::*;
use frame_support::pallet_prelude::*;
use pallet_rewards::{Call as RewardsCall, Config, Error, RewardConfigForAssetVault};
use sp_runtime::{traits::Zero, Perbill};
use tangle_primitives::types::rewards::LockMultiplier;

#[derive(Debug)]
pub enum RewardsFuzzCall {
    ClaimRewards(u32),
    ForceClaimRewards(AccountId, u32),
    UpdateVaultRewardConfig(u32, u8, u128, u128, Option<u32>),
}

impl RewardsFuzzCall {
    pub fn generate(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        // Use first byte to determine call type
        match data[0] % 3 {
            0 => Some(RewardsFuzzCall::ClaimRewards(
                u32::from_le_bytes(data.get(1..5)?.try_into().ok()?),
            )),
            1 => Some(RewardsFuzzCall::ForceClaimRewards(
                AccountId::new(data.get(1..33)?.try_into().ok()?),
                u32::from_le_bytes(data.get(33..37)?.try_into().ok()?),
            )),
            2 => Some(RewardsFuzzCall::UpdateVaultRewardConfig(
                u32::from_le_bytes(data.get(1..5)?.try_into().ok()?),
                data.get(5)?.clone(),
                u128::from_le_bytes(data.get(6..22)?.try_into().ok()?),
                u128::from_le_bytes(data.get(22..38)?.try_into().ok()?),
                if data.get(38)? % 2 == 0 {
                    Some(u32::from_le_bytes(data.get(39..43)?.try_into().ok()?))
                } else {
                    None
                },
            )),
            _ => None,
        }
    }

    pub fn execute(&self) -> DispatchResultWithPostInfo {
        match self {
            RewardsFuzzCall::ClaimRewards(vault_id) => {
                RewardsCallExecutor::execute_claim_rewards(*vault_id)
            }
            RewardsFuzzCall::ForceClaimRewards(account, vault_id) => {
                RewardsCallExecutor::execute_force_claim_rewards(account.clone(), *vault_id)
            }
            RewardsFuzzCall::UpdateVaultRewardConfig(vault_id, apy, deposit_cap, incentive_cap, boost_multiplier) => {
                RewardsCallExecutor::execute_update_vault_reward_config(
                    *vault_id,
                    *apy,
                    *deposit_cap,
                    *incentive_cap,
                    *boost_multiplier,
                )
            }
        }
    }

    pub fn verify(&self) -> bool {
        match self {
            RewardsFuzzCall::ClaimRewards(vault_id) => {
                RewardsCallVerifier::verify_claim_rewards(*vault_id)
            }
            RewardsFuzzCall::ForceClaimRewards(account, vault_id) => {
                RewardsCallVerifier::verify_force_claim_rewards(account.clone(), *vault_id)
            }
            RewardsFuzzCall::UpdateVaultRewardConfig(vault_id, apy, deposit_cap, incentive_cap, boost_multiplier) => {
                RewardsCallVerifier::verify_update_vault_reward_config(
                    *vault_id,
                    *apy,
                    *deposit_cap,
                    *incentive_cap,
                    *boost_multiplier,
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct RewardsCallGenerator;

impl RewardsCallGenerator {
    pub fn claim_rewards(vault_id: u32) -> RewardsCall<Runtime> {
        RewardsCall::claim_rewards { vault_id }
    }

    pub fn force_claim_rewards(account: AccountId, vault_id: u32) -> RewardsCall<Runtime> {
        RewardsCall::force_claim_rewards { account, vault_id }
    }

    pub fn update_vault_reward_config(
        vault_id: u32,
        apy: u8,
        deposit_cap: u128,
        incentive_cap: u128,
        boost_multiplier: Option<u32>,
    ) -> RewardsCall<Runtime> {
        let config = RewardConfigForAssetVault {
            apy: Perbill::from_percent(apy.min(100)),
            deposit_cap,
            incentive_cap,
            boost_multiplier: boost_multiplier.map(|m| m.min(500)), // Cap at 5x
        };
        RewardsCall::update_vault_reward_config { vault_id, new_config: config }
    }
}

#[derive(Debug)]
pub struct RewardsCallExecutor;

impl RewardsCallExecutor {
    pub fn execute_claim_rewards(vault_id: u32) -> DispatchResultWithPostInfo {
        Rewards::claim_rewards(RuntimeOrigin::signed(ALICE), vault_id)
    }

    pub fn execute_force_claim_rewards(account: AccountId, vault_id: u32) -> DispatchResultWithPostInfo {
        Rewards::force_claim_rewards(RuntimeOrigin::root(), account, vault_id)
    }

    pub fn execute_update_vault_reward_config(
        vault_id: u32,
        apy: u8,
        deposit_cap: u128,
        incentive_cap: u128,
        boost_multiplier: Option<u32>,
    ) -> DispatchResultWithPostInfo {
        let config = RewardConfigForAssetVault {
            apy: Perbill::from_percent(apy.min(100)),
            deposit_cap,
            incentive_cap,
            boost_multiplier: boost_multiplier.map(|m| m.min(500)), // Cap at 5x
        };
        Rewards::update_vault_reward_config(RuntimeOrigin::root(), vault_id, config)
    }
}

#[derive(Debug)]
pub struct RewardsCallVerifier;

impl RewardsCallVerifier {
    pub fn verify_claim_rewards(vault_id: u32) -> bool {
        if let Ok(_) = RewardsCallExecutor::execute_claim_rewards(vault_id) {
            // Verify that rewards were claimed by checking storage
            UserClaimedReward::<Runtime>::contains_key(&ALICE, vault_id)
        } else {
            false
        }
    }

    pub fn verify_force_claim_rewards(account: AccountId, vault_id: u32) -> bool {
        if let Ok(_) = RewardsCallExecutor::execute_force_claim_rewards(account.clone(), vault_id) {
            // Verify that rewards were claimed by checking storage
            UserClaimedReward::<Runtime>::contains_key(&account, vault_id)
        } else {
            false
        }
    }

    pub fn verify_update_vault_reward_config(
        vault_id: u32,
        apy: u8,
        deposit_cap: u128,
        incentive_cap: u128,
        boost_multiplier: Option<u32>,
    ) -> bool {
        if let Ok(_) = RewardsCallExecutor::execute_update_vault_reward_config(
            vault_id,
            apy,
            deposit_cap,
            incentive_cap,
            boost_multiplier,
        ) {
            // Verify that config was updated by checking storage
            if let Some(config) = RewardConfigStorage::<Runtime>::get(vault_id) {
                config.apy == Perbill::from_percent(apy.min(100))
                    && config.deposit_cap == deposit_cap
                    && config.incentive_cap == incentive_cap
                    && config.boost_multiplier == boost_multiplier.map(|m| m.min(500))
            } else {
                false
            }
        } else {
            false
        }
    }
}

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            if let Some(call) = RewardsFuzzCall::generate(data) {
                // Execute the call and verify its effects
                let _ = call.execute();
                let _ = call.verify();
            }
        });
    }
}
