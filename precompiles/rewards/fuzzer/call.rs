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

use fp_evm::Context;
use fp_evm::PrecompileSet;
use frame_support::traits::{Currency, Get};
use honggfuzz::fuzz;
use pallet_evm::AddressMapping;
use pallet_evm_precompile_rewards::{
    mock::*, mock_evm::PrecompilesValue, RewardsPrecompileCall as RewardsCall,
};
use pallet_rewards::pallet as rewards;
use precompile_utils::prelude::*;
use precompile_utils::testing::*;
use rand::{seq::SliceRandom, Rng};
use sp_runtime::traits::{Scale, Zero, One};
use sp_runtime::Percent;

const MAX_APY: u32 = 1000; // 10% in basis points
const MAX_REWARDS: u128 = u128::MAX;
const MAX_VAULT_ID: u128 = 1000;
const MAX_BLUEPRINT_ID: u64 = 1000;

type PCall = RewardsCall<Runtime>;

fn random_address<R: Rng>(rng: &mut R) -> Address {
    Address(rng.gen::<[u8; 20]>().into())
}

/// Generate a random signed origin
fn random_signed_origin<R: Rng>(rng: &mut R) -> (RuntimeOrigin, Address) {
    let addr = random_address(rng);
    let signer = <TestAccount as AddressMapping<AccountId>>::into_account_id(addr.0);
    (RuntimeOrigin::signed(signer), addr)
}

/// Generate a random asset ID
fn random_asset<R: Rng>(rng: &mut R) -> u128 {
    rng.gen_range(1..u128::MAX)
}

/// Generate a random APY value (max 10%)
fn random_apy<R: Rng>(rng: &mut R) -> u32 {
    rng.gen_range(1..=MAX_APY)
}

/// Generate random rewards amount
fn random_rewards<R: Rng>(rng: &mut R) -> u128 {
    rng.gen_range(1..MAX_REWARDS)
}

/// Generate a random vault ID
fn random_vault_id<R: Rng>(rng: &mut R) -> u128 {
    rng.gen_range(1..MAX_VAULT_ID)
}

/// Generate a random blueprint ID
fn random_blueprint_id<R: Rng>(rng: &mut R) -> u64 {
    rng.gen_range(1..MAX_BLUEPRINT_ID)
}

/// Update asset rewards call
fn update_asset_rewards_call<R: Rng>(rng: &mut R) -> (PCall, Address) {
    let (origin, who) = random_signed_origin(rng);
    let asset_id = random_asset(rng).into();
    let rewards = random_rewards(rng).into();
    (PCall::update_asset_rewards { asset_id, rewards }, who)
}

/// Update asset APY call
fn update_asset_apy_call<R: Rng>(rng: &mut R) -> (PCall, Address) {
    let (origin, who) = random_signed_origin(rng);
    let asset_id = random_asset(rng).into();
    let apy = random_apy(rng);
    (PCall::update_asset_apy { asset_id, apy }, who)
}

/// Set incentive APY and cap call
fn set_incentive_apy_and_cap_call<R: Rng>(rng: &mut R) -> (PCall, Address) {
    let (origin, who) = random_signed_origin(rng);
    let vault_id = random_vault_id(rng).into();
    let apy = random_apy(rng);
    let cap = random_rewards(rng).into();
    (PCall::set_incentive_apy_and_cap { vault_id, apy, cap }, who)
}

/// Whitelist blueprint call
fn whitelist_blueprint_call<R: Rng>(rng: &mut R) -> (PCall, Address) {
    let (origin, who) = random_signed_origin(rng);
    let blueprint_id = random_blueprint_id(rng);
    (PCall::whitelist_blueprint_for_rewards { blueprint_id }, who)
}

/// Manage asset in vault call
fn manage_asset_in_vault_call<R: Rng>(rng: &mut R) -> (PCall, Address) {
    let (origin, who) = random_signed_origin(rng);
    let vault_id = random_vault_id(rng).into();
    let asset_id = random_asset(rng).into();
    let action = rng.gen_range(0..=1);
    (PCall::manage_asset_in_vault { vault_id, asset_id, action }, who)
}

/// Generate random calls for fuzzing
fn random_calls<R: Rng>(mut rng: &mut R) -> impl IntoIterator<Item = (PCall, Address)> {
    let op = PCall::selectors().choose(rng).cloned().unwrap();
    match op {
        _ if op == PCall::update_asset_rewards_selectors()[0] => {
            vec![update_asset_rewards_call(&mut rng)]
        },
        _ if op == PCall::update_asset_apy_selectors()[0] => {
            vec![update_asset_apy_call(&mut rng)]
        },
        _ if op == PCall::set_incentive_apy_and_cap_selectors()[0] => {
            vec![set_incentive_apy_and_cap_call(&mut rng)]
        },
        _ if op == PCall::whitelist_blueprint_for_rewards_selectors()[0] => {
            vec![whitelist_blueprint_call(&mut rng)]
        },
        _ if op == PCall::manage_asset_in_vault_selectors()[0] => {
            vec![manage_asset_in_vault_call(&mut rng)]
        },
        _ => vec![],
    }
}

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            let mut rng = rand::thread_rng();
            let calls = random_calls(&mut rng);

            // Create test externalities
            let mut ext = ExtBuilder::default().build();
            ext.execute_with(|| {
                let precompiles = PrecompilesValue::get();

                // Execute each call
                for (call, who) in calls {
                    let input = call.encode();
                    let context = Context {
                        address: Default::default(),
                        caller: who.0,
                        apparent_value: Default::default(),
                    };

                    let info = call.estimate_gas(input.clone(), &mut EvmDataWriter::new(), context);
                    match info {
                        Ok((_, estimate)) => {
                            let mut gasometer = Gasometer::new(estimate);
                            let outcome = precompiles
                                .execute(&context.address, &mut gasometer, &context, &input)
                                .expect("Precompile failed");

                            // Perform sanity checks
                            do_sanity_checks(call, who, outcome);
                        },
                        Err(e) => {
                            // Expected errors are ok
                            println!("Expected error: {:?}", e);
                        },
                    }
                }
            });
        });
    }
}

/// Perform sanity checks on the state after a call is executed successfully
fn do_sanity_checks(call: PCall, origin: Address, outcome: PrecompileOutput) {
    match call {
        PCall::update_asset_rewards { asset_id, rewards } => {
            // Check that rewards were updated
            assert_eq!(Rewards::asset_rewards(asset_id), rewards);
        },
        PCall::update_asset_apy { asset_id, apy } => {
            // Check that APY was updated and is within bounds
            assert!(apy <= MAX_APY);
            assert_eq!(Rewards::asset_apy(asset_id), apy);
        },
        PCall::set_incentive_apy_and_cap { vault_id, apy, cap } => {
            // Check APY is within bounds
            assert!(apy <= MAX_APY);
            
            // Check config was updated
            if let Some(config) = Rewards::reward_config() {
                if let Some(vault_config) = config.configs.get(&vault_id) {
                    assert_eq!(vault_config.apy, Percent::from_parts(apy as u8));
                    assert_eq!(vault_config.cap, cap);
                }
            }
        },
        PCall::whitelist_blueprint_for_rewards { blueprint_id } => {
            // Check blueprint was whitelisted
            if let Some(config) = Rewards::reward_config() {
                assert!(config.whitelisted_blueprints.contains(&blueprint_id));
            }
        },
        PCall::manage_asset_in_vault { vault_id, asset_id, action } => {
            // Check asset was added/removed from vault
            if let Some(config) = Rewards::reward_config() {
                if let Some(vault_config) = config.configs.get(&vault_id) {
                    match action {
                        0 => assert!(vault_config.assets.contains(&asset_id)),
                        1 => assert!(!vault_config.assets.contains(&asset_id)),
                        _ => panic!("Invalid action"),
                    }
                }
            }
        },
        _ => {},
    }
}
