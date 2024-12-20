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
//! Running this fuzzer can be done with `cargo hfuzz run call`. `honggfuzz` CLI
//! options can be used by setting `HFUZZ_RUN_ARGS`, such as `-n 4` to use 4 threads.
//!
//! # Debugging a panic
//! Once a panic is found, it can be debugged with
//! `cargo hfuzz run-debug per_thing_rational hfuzz_workspace/call/*.fuzz`.

use frame_support::traits::{Currency, GetCallName, Hooks, UnfilteredDispatchable};
use honggfuzz::fuzz;
use pallet_multi_asset_delegation::{mock::*, pallet as mad};
use rand::Rng;

const MAX_ED_MULTIPLE: Balance = 10_000;
const MIN_ED_MULTIPLE: Balance = 10;

/// Grab random accounts.
fn random_signed_origin<R: Rng>(rng: &mut R) -> (RuntimeOrigin, AccountId) {
	let acc: AccountId = rng.gen::<[u8; 32]>().into();
	(RuntimeOrigin::signed(acc.clone()), acc)
}

fn random_ed_multiple<R: Rng>(rng: &mut R) -> Balance {
	let multiple = rng.gen_range(MIN_ED_MULTIPLE..MAX_ED_MULTIPLE);
	ExistentialDeposit::get() * multiple
}

fn fund_account<R: Rng>(rng: &mut R, account: &AccountId) {
	let target_amount = random_ed_multiple(rng);
	if let Some(top_up) = target_amount.checked_sub(Balances::free_balance(account)) {
		let _ = Balances::deposit_creating(account, top_up);
	}
	assert!(Balances::free_balance(account) >= target_amount);
}

fn random_call<R: Rng>(mut rng: &mut R) -> (mad::Call<Runtime>, RuntimeOrigin) {
	let op = rng.gen::<usize>();
	let op_count = <mad::Call<Runtime> as GetCallName>::get_call_names().len();

	match op % op_count {
		0 => {
			// join_operators
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let bond_amount = random_ed_multiple(&mut rng);
			(mad::Call::join_operators { bond_amount }, origin)
		},
		1 => {
			// Schedule leave operators
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			(mad::Call::schedule_leave_operators {}, origin)
		},
		_ => {
			// Do nothing for now.
			(mad::Call::schedule_leave_operators {}, RuntimeOrigin::none())
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
					"iteration {}, call {:?}, origin {:?}, outcome: {:?}, so far {} ok {} err",
					iteration,
					call,
					origin,
					outcome,
					ok,
					err,
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
