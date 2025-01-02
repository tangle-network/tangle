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

use frame_support::{
    dispatch::PostDispatchInfo,
    traits::{Currency, Get, GetCallName, Hooks, UnfilteredDispatchable},
};
use frame_system::ensure_signed_or_root;
use honggfuzz::fuzz;
use pallet_rewards::{mock::*, pallet as rewards, types::*, RewardType};
use rand::{seq::SliceRandom, Rng};
use sp_runtime::{
    traits::{Scale, Zero},
    Percent,
};
use tangle_primitives::{services::Asset, types::rewards::LockMultiplier};

const MAX_ED_MULTIPLE: Balance = 10_000;
const MIN_ED_MULTIPLE: Balance = 10;

fn random_account_id<R: Rng>(rng: &mut R) -> AccountId {
    rng.gen::<[u8; 32]>().into()
}

/// Grab random accounts.
fn random_signed_origin<R: Rng>(rng: &mut R) -> (RuntimeOrigin, AccountId) {
    let acc = random_account_id(rng);
    (RuntimeOrigin::signed(acc.clone()), acc)
}

fn random_ed_multiple<R: Rng>(rng: &mut R) -> Balance {
    let multiple = rng.gen_range(MIN_ED_MULTIPLE..MAX_ED_MULTIPLE);
    ExistentialDeposit::get() * multiple
}

fn random_asset<R: Rng>(rng: &mut R) -> Asset<AssetId> {
    let asset_id = rng.gen_range(1..u128::MAX);
    let is_evm = rng.gen_bool(0.5);
    if is_evm {
        let evm_address = rng.gen::<[u8; 20]>().into();
        Asset::Erc20(evm_address)
    } else {
        Asset::Custom(asset_id)
    }
}

fn random_lock_multiplier<R: Rng>(rng: &mut R) -> LockMultiplier {
    let multipliers = [
        LockMultiplier::OneMonth,
        LockMultiplier::TwoMonths,
        LockMultiplier::ThreeMonths,
        LockMultiplier::SixMonths,
    ];
    *multipliers.choose(rng).unwrap()
}

fn random_reward_type<R: Rng>(rng: &mut R) -> RewardType {
    let reward_types = [
        RewardType::Boost,
        RewardType::Service,
        RewardType::Restaking,
    ];
    *reward_types.choose(rng).unwrap()
}

fn fund_account<R: Rng>(rng: &mut R, account: &AccountId) {
    let target_amount = random_ed_multiple(rng);
    if let Some(top_up) = target_amount.checked_sub(Balances::free_balance(account)) {
        let _ = Balances::deposit_creating(account, top_up);
    }
    assert!(Balances::free_balance(account) >= target_amount);
}

/// Initialize an asset with random APY and capacity
fn init_asset_call<R: Rng>(
    rng: &mut R,
    origin: RuntimeOrigin,
) -> (rewards::Call<Runtime>, RuntimeOrigin) {
    let asset = random_asset(rng);
    let apy = rng.gen_range(1..10000); // 0.01% to 100%
    let capacity = random_ed_multiple(rng);
    
    (rewards::Call::whitelist_asset { asset }, origin.clone());
    (
        rewards::Call::set_asset_apy { 
            asset: asset.clone(),
            apy_basis_points: apy,
        },
        origin.clone(),
    );
    (
        rewards::Call::set_asset_capacity {
            asset,
            capacity,
        },
        origin,
    )
}

/// Claim rewards call
fn claim_rewards_call<R: Rng>(
    rng: &mut R,
    origin: RuntimeOrigin,
) -> (rewards::Call<Runtime>, RuntimeOrigin) {
    let asset = random_asset(rng);
    let reward_type = random_reward_type(rng);
    (
        rewards::Call::claim_rewards {
            asset,
            reward_type,
        },
        origin,
    )
}

fn random_calls<R: Rng>(
    mut rng: &mut R,
) -> impl IntoIterator<Item = (rewards::Call<Runtime>, RuntimeOrigin)> {
    let op = <rewards::Call<Runtime> as GetCallName>::get_call_names()
        .choose(rng)
        .cloned()
        .unwrap();

    match op {
        "whitelist_asset" | "set_asset_apy" | "set_asset_capacity" => {
            let origin = RuntimeOrigin::root();
            [init_asset_call(&mut rng, origin)].to_vec()
        }
        "claim_rewards" => {
            let (origin, who) = random_signed_origin(&mut rng);
            fund_account(&mut rng, &who);
            [claim_rewards_call(&mut rng, origin)].to_vec()
        }
        _ => vec![],
    }
}

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            let mut rng = rand::rngs::SmallRng::from_slice(data).unwrap();
            let calls = random_calls(&mut rng);

            new_test_ext().execute_with(|| {
                // Run to block 1 to initialize
                System::set_block_number(1);
                Rewards::on_initialize(1);

                for (call, origin) in calls {
                    let _ = call.dispatch_bypass_filter(origin.clone());
                    System::assert_last_event(Event::Rewards(rewards::Event::RewardsClaimed { 
                        account: who, 
                        asset, 
                        amount,
                        reward_type,
                    }));
                }
            });
        });
    }
}

/// Perform sanity checks on the state after a call is executed successfully.
fn do_sanity_checks(
    call: rewards::Call<Runtime>,
    origin: RuntimeOrigin,
    outcome: PostDispatchInfo,
) {
    match call {
        rewards::Call::claim_rewards { asset, reward_type } => {
            // Verify the asset is whitelisted
            assert!(Rewards::is_asset_whitelisted(asset));

            // Get the account that made the call
            let who = ensure_signed_or_root(origin).unwrap();

            // Get user rewards
            let rewards = Rewards::user_rewards(&who, asset);

            // Verify rewards were properly reset after claiming
            match reward_type {
                RewardType::Boost => {
                    // For boost rewards, verify expiry was updated
                    assert_eq!(
                        rewards.boost_rewards.expiry,
                        System::block_number(),
                    );
                }
                RewardType::Service => {
                    // Verify service rewards were reset
                    assert!(rewards.service_rewards.is_zero());
                }
                RewardType::Restaking => {
                    // Verify restaking rewards were reset
                    assert!(rewards.restaking_rewards.is_zero());
                }
            }

            // Verify total score is updated correctly
            let user_score = rewards::functions::calculate_user_score::<Runtime>(asset, &rewards);
            assert!(Rewards::total_asset_score(asset) >= user_score);
        }
        _ => {}
    }
}
