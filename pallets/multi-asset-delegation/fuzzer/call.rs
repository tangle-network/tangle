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
//! Running this fuzzer can be done with `cargo hfuzz run mad-fuzzer`. `honggfuzz` CLI
//! options can be used by setting `HFUZZ_RUN_ARGS`, such as `-n 4` to use 4 threads.
//!
//! # Debugging a panic
//! Once a panic is found, it can be debugged with
//! `cargo hfuzz run-debug mad-fuzzer hfuzz_workspace/mad-fuzzer/*.fuzz`.

use frame_support::traits::{Currency, GetCallName, Hooks, UnfilteredDispatchable};
use honggfuzz::fuzz;
use pallet_multi_asset_delegation::{mock::*, pallet as mad, types::*};
use rand::{seq::SliceRandom, Rng};
use sp_runtime::Percent;

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

fn fund_account<R: Rng>(rng: &mut R, account: &AccountId) {
	let target_amount = random_ed_multiple(rng);
	if let Some(top_up) = target_amount.checked_sub(Balances::free_balance(account)) {
		let _ = Balances::deposit_creating(account, top_up);
	}
	assert!(Balances::free_balance(account) >= target_amount);
}

fn random_call<R: Rng>(mut rng: &mut R) -> (mad::Call<Runtime>, RuntimeOrigin) {
	let op = <mad::Call<Runtime> as GetCallName>::get_call_names()
		.choose(rng)
		.cloned()
		.unwrap();

	match op {
		"join_operators" => {
			// join_operators
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let bond_amount = random_ed_multiple(&mut rng);
			(mad::Call::join_operators { bond_amount }, origin)
		},
		"schedule_leave_operators" => {
			// Schedule leave operators
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			(mad::Call::schedule_leave_operators {}, origin)
		},
		"cancel_leave_operators" => {
			// Cancel leave operators
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			(mad::Call::cancel_leave_operators {}, origin)
		},
		"execute_leave_operators" => {
			// Execute leave operators
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			(mad::Call::execute_leave_operators {}, origin)
		},
		"operator_bond_more" => {
			// Operator bond more
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let additional_bond = random_ed_multiple(&mut rng);
			(mad::Call::operator_bond_more { additional_bond }, origin)
		},
		"schedule_operator_unstake" => {
			// Schedule operator unstake
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let unstake_amount = random_ed_multiple(&mut rng);
			(mad::Call::schedule_operator_unstake { unstake_amount }, origin)
		},
		"execute_operator_unstake" => {
			// Execute operator unstake
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			(mad::Call::execute_operator_unstake {}, origin)
		},
		"cancel_operator_unstake" => {
			// Cancel operator unstake
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			(mad::Call::cancel_operator_unstake {}, origin)
		},
		"go_offline" => {
			// Go offline
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			(mad::Call::go_offline {}, origin)
		},
		"go_online" => {
			// Go online
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			(mad::Call::go_online {}, origin)
		},
		"deposit" => {
			// Deposit
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let asset_id = random_asset(&mut rng);
			let amount = random_ed_multiple(&mut rng);
			let evm_address =
				if rng.gen_bool(0.5) { Some(rng.gen::<[u8; 20]>().into()) } else { None };
			(mad::Call::deposit { asset_id, amount, evm_address }, origin)
		},
		"schedule_withdraw" => {
			// Schedule withdraw
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let asset_id = random_asset(&mut rng);
			let amount = random_ed_multiple(&mut rng);
			(mad::Call::schedule_withdraw { asset_id, amount }, origin)
		},
		"execute_withdraw" => {
			// Execute withdraw
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let evm_address =
				if rng.gen_bool(0.5) { Some(rng.gen::<[u8; 20]>().into()) } else { None };
			(mad::Call::execute_withdraw { evm_address }, origin)
		},
		"cancel_withdraw" => {
			// Cancel withdraw
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let asset_id = random_asset(&mut rng);
			let amount = random_ed_multiple(&mut rng);
			(mad::Call::cancel_withdraw { asset_id, amount }, origin)
		},
		"delegate" => {
			// Delegate
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let operator = random_account_id(&mut rng);
			let asset_id = random_asset(&mut rng);
			let amount = random_ed_multiple(&mut rng);
			let blueprint_selection = {
				let all = rng.gen_bool(0.5);
				if all {
					DelegatorBlueprintSelection::All
				} else {
					let count = rng.gen_range(1..MaxDelegatorBlueprints::get());
					DelegatorBlueprintSelection::Fixed(
						(0..count)
							.map(|_| rng.gen::<u64>())
							.collect::<Vec<_>>()
							.try_into()
							.unwrap(),
					)
				}
			};
			(mad::Call::delegate { operator, asset_id, amount, blueprint_selection }, origin)
		},
		"schedule_delegator_unstake" => {
			// Schedule delegator unstakes
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let operator = random_account_id(&mut rng);
			let asset_id = random_asset(&mut rng);
			let amount = random_ed_multiple(&mut rng);
			(mad::Call::schedule_delegator_unstake { operator, asset_id, amount }, origin)
		},
		"execute_delegator_unstake" => {
			// Execute delegator unstake
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			(mad::Call::execute_delegator_unstake {}, origin)
		},
		"cancel_delegator_unstake" => {
			// Cancel delegator unstake
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let operator = random_account_id(&mut rng);
			let asset_id = random_asset(&mut rng);
			let amount = random_ed_multiple(&mut rng);
			(mad::Call::cancel_delegator_unstake { operator, asset_id, amount }, origin)
		},
		"set_incentive_apy_and_cap" => {
			// Set incentive APY and cap
			let is_root = rng.gen_bool(0.5);
			let (origin, who) = if is_root {
				(RuntimeOrigin::root(), [0u8; 32].into())
			} else {
				random_signed_origin(&mut rng)
			};
			fund_account(&mut rng, &who);
			let vault_id = rng.gen();
			let apy = Percent::from_percent(rng.gen_range(0..100));
			let cap = rng.gen_range(0..Balance::MAX);
			(mad::Call::set_incentive_apy_and_cap { vault_id, apy, cap }, origin)
		},
		"whitelist_blueprint_for_rewards" => {
			// Whitelist blueprint for rewards
			let is_root = rng.gen_bool(0.5);
			let (origin, who) = if is_root {
				(RuntimeOrigin::root(), [0u8; 32].into())
			} else {
				random_signed_origin(&mut rng)
			};
			fund_account(&mut rng, &who);
			let blueprint_id = rng.gen::<u64>();
			(mad::Call::whitelist_blueprint_for_rewards { blueprint_id }, origin)
		},
		"manage_asset_in_vault" => {
			// Manage asset in vault
			let is_root = rng.gen_bool(0.5);
			let (origin, who) = if is_root {
				(RuntimeOrigin::root(), [0u8; 32].into())
			} else {
				random_signed_origin(&mut rng)
			};
			fund_account(&mut rng, &who);
			let asset_id = random_asset(&mut rng);
			let vault_id = rng.gen();
			let action = if rng.gen() { AssetAction::Add } else { AssetAction::Remove };
			(mad::Call::manage_asset_in_vault { asset_id, vault_id, action }, origin)
		},
		"add_blueprint_id" => {
			// Add blueprint ID
			let is_root = rng.gen_bool(0.5);
			let (origin, who) = if is_root {
				(RuntimeOrigin::root(), [0u8; 32].into())
			} else {
				random_signed_origin(&mut rng)
			};
			fund_account(&mut rng, &who);
			let blueprint_id = rng.gen::<u64>();
			(mad::Call::add_blueprint_id { blueprint_id }, origin)
		},
		"remove_blueprint_id" => {
			// Remove blueprint ID
			let is_root = rng.gen_bool(0.5);
			let (origin, who) = if is_root {
				(RuntimeOrigin::root(), [0u8; 32].into())
			} else {
				random_signed_origin(&mut rng)
			};
			fund_account(&mut rng, &who);
			let blueprint_id = rng.gen::<u64>();
			(mad::Call::remove_blueprint_id { blueprint_id }, origin)
		},
		_ => {
			unimplemented!("unknown call name: {}", op)
		},
	}
}

fn main() {
	sp_tracing::try_init_simple();
	let mut ext = sp_io::TestExternalities::new_empty();
	let mut events_histogram = Vec::<(mad::Event<Runtime>, u32)>::default();
	let mut iteration = 0 as BlockNumber;
	let mut ok = 0;
	let mut err = 0;

	ext.execute_with(|| {
		System::set_block_number(1);
		Session::on_initialize(1);
		<Staking as Hooks<u64>>::on_initialize(1);
	});

	loop {
		fuzz!(|seed: [u8; 32]| {
			use ::rand::{rngs::SmallRng, SeedableRng};
			let mut rng = SmallRng::from_seed(seed);

			ext.execute_with(|| {
				let (call, origin) = random_call(&mut rng);
				let outcome = call.clone().dispatch_bypass_filter(origin.clone());
				iteration += 1;
				match outcome {
					Ok(_) => ok += 1,
					Err(_) => err += 1,
				};

				sp_tracing::trace!(
					%iteration,
					?call,
					?origin,
					?outcome,
					%ok,
					%err,
					"fuzzed call"
				);

				// execute sanity checks at a fixed interval, possibly on every block.
				if iteration
					% (std::env::var("SANITY_CHECK_INTERVAL")
						.ok()
						.and_then(|x| x.parse::<u64>().ok()))
					.unwrap_or(1) == 0
				{
					sp_tracing::info!("running sanity checks at {}", iteration);
					// TODO
				}

				// collect and reset events.
				System::events()
					.into_iter()
					.map(|r| r.event)
					.filter_map(|e| {
						if let RuntimeEvent::MultiAssetDelegation(inner) = e {
							Some(inner)
						} else {
							None
						}
					})
					.for_each(|e| {
						if let Some((_, c)) = events_histogram
							.iter_mut()
							.find(|(x, _)| std::mem::discriminant(x) == std::mem::discriminant(&e))
						{
							*c += 1;
						} else {
							events_histogram.push((e, 1))
						}
					});
				System::reset_events();
			})
		})
	}
}
