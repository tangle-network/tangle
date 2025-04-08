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

use frame_support::{
	dispatch::PostDispatchInfo,
	traits::{Currency, Get, GetCallName, Hooks, UnfilteredDispatchable},
};
use frame_system::ensure_signed_or_root;
use honggfuzz::fuzz;
use pallet_multi_asset_delegation::{mock::*, pallet as mad, types::*};
use rand::{Rng, seq::SliceRandom};
use sp_runtime::traits::{Scale, Zero};

const MAX_ED_MULTIPLE: Balance = 10_000;
const MIN_ED_MULTIPLE: Balance = 10;

fn random_account_id<R: Rng>(rng: &mut R) -> AccountId {
	rng.r#gen::<[u8; 32]>().into()
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
	let asset = rng.gen_range(1..u128::MAX);
	let is_evm = rng.gen_bool(0.5);
	if is_evm {
		let evm_address = rng.r#gen::<[u8; 20]>().into();
		Asset::Erc20(evm_address)
	} else {
		Asset::Custom(asset)
	}
}

fn fund_account<R: Rng>(rng: &mut R, account: &AccountId) {
	let target_amount = random_ed_multiple(rng);
	if let Some(top_up) = target_amount.checked_sub(Balances::free_balance(account)) {
		let _ = Balances::deposit_creating(account, top_up);
	}
	assert!(Balances::free_balance(account) >= target_amount);
}

/// Join operators call.
fn join_operators_call<R: Rng>(
	rng: &mut R,
	origin: RuntimeOrigin,
	who: &AccountId,
) -> (mad::Call<Runtime>, RuntimeOrigin) {
	let minimum_bond = <<Runtime as mad::Config>::MinOperatorBondAmount as Get<Balance>>::get();
	let multiplier = rng.gen_range(1..50u128);
	let _ = Balances::deposit_creating(who, minimum_bond.mul(multiplier));
	let bond_amount = minimum_bond.mul(multiplier);
	(mad::Call::join_operators { bond_amount }, origin)
}

fn random_calls<R: Rng>(
	mut rng: &mut R,
) -> impl IntoIterator<Item = (mad::Call<Runtime>, RuntimeOrigin)> {
	let op = <mad::Call<Runtime> as GetCallName>::get_call_names()
		.choose(rng)
		.cloned()
		.unwrap();

	match op {
		"join_operators" => {
			// join_operators
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			[join_operators_call(&mut rng, origin, &who)].to_vec()
		},
		"schedule_leave_operators" => {
			// Schedule leave operators
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			[
				join_operators_call(&mut rng, origin.clone(), &who),
				(mad::Call::schedule_leave_operators {}, origin),
			]
			.to_vec()
		},
		"cancel_leave_operators" => {
			// Cancel leave operators
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			[
				join_operators_call(&mut rng, origin.clone(), &who),
				(mad::Call::cancel_leave_operators {}, origin),
			]
			.to_vec()
		},
		"execute_leave_operators" => {
			// Execute leave operators
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			[
				join_operators_call(&mut rng, origin.clone(), &who),
				(mad::Call::execute_leave_operators {}, origin),
			]
			.to_vec()
		},
		"operator_bond_more" => {
			// Operator bond more
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let additional_bond = random_ed_multiple(&mut rng);
			[
				join_operators_call(&mut rng, origin.clone(), &who),
				(mad::Call::operator_bond_more { additional_bond }, origin),
			]
			.to_vec()
		},
		"schedule_operator_unstake" => {
			// Schedule operator unstake
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let unstake_amount = random_ed_multiple(&mut rng);
			[
				join_operators_call(&mut rng, origin.clone(), &who),
				(mad::Call::schedule_operator_unstake { unstake_amount }, origin),
			]
			.to_vec()
		},
		"execute_operator_unstake" => {
			// Execute operator unstake
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			[
				join_operators_call(&mut rng, origin.clone(), &who),
				(mad::Call::execute_operator_unstake {}, origin),
			]
			.to_vec()
		},
		"cancel_operator_unstake" => {
			// Cancel operator unstake
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			[
				join_operators_call(&mut rng, origin.clone(), &who),
				(mad::Call::cancel_operator_unstake {}, origin),
			]
			.to_vec()
		},
		"go_offline" => {
			// Go offline
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			[
				join_operators_call(&mut rng, origin.clone(), &who),
				(mad::Call::go_offline {}, origin),
			]
			.to_vec()
		},
		"go_online" => {
			// Go online
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			[join_operators_call(&mut rng, origin.clone(), &who), (mad::Call::go_online {}, origin)]
				.to_vec()
		},
		"deposit" => {
			// Deposit
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let asset = random_asset(&mut rng);
			let amount = random_ed_multiple(&mut rng);
			let evm_address =
				if rng.gen_bool(0.5) { Some(rng.r#gen::<[u8; 20]>().into()) } else { None };
			[(mad::Call::deposit { asset, amount, evm_address, lock_multiplier: None }, origin)]
				.to_vec()
		},
		"schedule_withdraw" => {
			// Schedule withdraw
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let asset = random_asset(&mut rng);
			let amount = random_ed_multiple(&mut rng);
			[(mad::Call::schedule_withdraw { asset, amount }, origin)].to_vec()
		},
		"execute_withdraw" => {
			// Execute withdraw
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let evm_address =
				if rng.gen_bool(0.5) { Some(rng.r#gen::<[u8; 20]>().into()) } else { None };
			[(mad::Call::execute_withdraw { evm_address }, origin)].to_vec()
		},
		"cancel_withdraw" => {
			// Cancel withdraw
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let asset = random_asset(&mut rng);
			let amount = random_ed_multiple(&mut rng);
			[(mad::Call::cancel_withdraw { asset, amount }, origin)].to_vec()
		},
		"delegate" => {
			// Delegate
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let (operator_origin, operator) = random_signed_origin(&mut rng);
			let asset = random_asset(&mut rng);
			let amount = random_ed_multiple(&mut rng);
			let blueprint_selection = {
				let all = rng.gen_bool(0.5);
				if all {
					DelegatorBlueprintSelection::All
				} else {
					let count = rng.gen_range(1..MaxDelegatorBlueprints::get());
					DelegatorBlueprintSelection::Fixed(
						(0..count)
							.map(|_| rng.r#gen::<u64>())
							.collect::<Vec<_>>()
							.try_into()
							.unwrap(),
					)
				}
			};
			[
				join_operators_call(&mut rng, operator_origin.clone(), &operator),
				(mad::Call::delegate { operator, asset, amount, blueprint_selection }, origin),
			]
			.to_vec()
		},
		"schedule_delegator_unstake" => {
			// Schedule delegator unstakes
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let (operator_origin, operator) = random_signed_origin(&mut rng);
			let asset = random_asset(&mut rng);
			let amount = random_ed_multiple(&mut rng);
			[
				join_operators_call(&mut rng, operator_origin.clone(), &operator),
				(mad::Call::schedule_delegator_unstake { operator, asset, amount }, origin),
			]
			.to_vec()
		},
		"execute_delegator_unstake" => {
			// Execute delegator unstake
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			[(mad::Call::execute_delegator_unstake {}, origin)].to_vec()
		},
		"cancel_delegator_unstake" => {
			// Cancel delegator unstake
			let (origin, who) = random_signed_origin(&mut rng);
			fund_account(&mut rng, &who);
			let (operator_origin, operator) = random_signed_origin(&mut rng);
			let asset = random_asset(&mut rng);
			let amount = random_ed_multiple(&mut rng);
			[
				join_operators_call(&mut rng, operator_origin.clone(), &operator),
				(mad::Call::cancel_delegator_unstake { operator, asset, amount }, origin),
			]
			.to_vec()
		},
		_ => {
			unimplemented!("unknown call name: {}", op)
		},
	}
}

fn main() {
	sp_tracing::try_init_simple();
	let mut ext = sp_io::TestExternalities::new_empty();
	let mut block_number = 1;
	loop {
		fuzz!(|seed: [u8; 32]| {
			use ::rand::{SeedableRng, rngs::SmallRng};
			let mut rng = SmallRng::from_seed(seed);

			ext.execute_with(|| {
				System::set_block_number(block_number);
				Session::on_initialize(block_number);
				Staking::on_initialize(block_number);

				for (call, origin) in random_calls(&mut rng) {
					let outcome = call.clone().dispatch_bypass_filter(origin.clone());
					sp_tracing::trace!(?call, ?origin, ?outcome, "fuzzed call");

					// execute sanity checks at a fixed interval, possibly on every block.
					if let Ok(out) = outcome {
						sp_tracing::info!("running sanity checks..");
						do_sanity_checks(call.clone(), origin.clone(), out);
					}
				}

				System::reset_events();
				block_number += 1;
			});
		})
	}
}

/// Perform sanity checks on the state after a call is executed successfully.
#[allow(unused)]
fn do_sanity_checks(call: mad::Call<Runtime>, origin: RuntimeOrigin, outcome: PostDispatchInfo) {
	let caller = match ensure_signed_or_root(origin).unwrap() {
		Some(signer) => signer,
		None =>
		/*Root */
		{
			[0u8; 32].into()
		},
	};
	match call {
		mad::Call::join_operators { bond_amount } => {
			assert!(mad::Operators::<Runtime>::contains_key(&caller), "operator not found");
			assert_eq!(
				MultiAssetDelegation::operator_info(&caller).unwrap_or_default().stake,
				bond_amount
			);
			assert!(
				Balances::reserved_balance(&caller).ge(&bond_amount),
				"bond amount not reserved"
			);
		},
		mad::Call::schedule_leave_operators {} => {
			assert!(mad::Operators::<Runtime>::contains_key(&caller), "operator not found");
			let current_round = mad::CurrentRound::<Runtime>::get();
			let leaving_time =
				<<Runtime as mad::Config>::LeaveOperatorsDelay as Get<u32>>::get() + current_round;
			assert_eq!(
				mad::Operators::<Runtime>::get(&caller).unwrap_or_default().status,
				OperatorStatus::Leaving(leaving_time)
			);
		},
		mad::Call::cancel_leave_operators {} => {
			assert_eq!(
				mad::Operators::<Runtime>::get(&caller).unwrap_or_default().status,
				OperatorStatus::Active
			);
		},
		mad::Call::execute_leave_operators {} => {
			assert!(!mad::Operators::<Runtime>::contains_key(&caller), "operator not removed");
			assert!(Balances::reserved_balance(&caller).is_zero(), "bond amount not unreserved");
		},
		mad::Call::operator_bond_more { additional_bond } => {
			let info = MultiAssetDelegation::operator_info(&caller).unwrap_or_default();
			assert!(info.stake.ge(&additional_bond), "bond amount not increased");
			assert!(
				Balances::reserved_balance(&caller).ge(&additional_bond),
				"bond amount not reserved"
			);
		},
		mad::Call::schedule_operator_unstake { unstake_amount } => {
			let info = MultiAssetDelegation::operator_info(&caller).unwrap_or_default();
			let current_round = MultiAssetDelegation::current_round();
			let unstake_request =
				OperatorBondLessRequest { amount: unstake_amount, request_time: current_round };
			assert_eq!(info.request, Some(unstake_request), "unstake request not set");
		},
		mad::Call::execute_operator_unstake {} => {
			let info = MultiAssetDelegation::operator_info(&caller).unwrap_or_default();
			assert!(info.request.is_none(), "unstake request not removed");
			// reserved balance should be reduced and equal to the stake
			assert!(
				Balances::reserved_balance(&caller).eq(&info.stake),
				"reserved balance not equal to stake"
			);
		},
		mad::Call::cancel_operator_unstake {} => {
			let info = MultiAssetDelegation::operator_info(&caller).unwrap_or_default();
			assert!(info.request.is_none(), "unstake request not removed");
		},
		mad::Call::go_offline {} => {
			let info = MultiAssetDelegation::operator_info(&caller).unwrap_or_default();
			assert_eq!(info.status, OperatorStatus::Inactive, "status not set to inactive");
		},
		mad::Call::go_online {} => {
			let info = MultiAssetDelegation::operator_info(&caller).unwrap_or_default();
			assert_eq!(info.status, OperatorStatus::Active, "status not set to active");
		},
		mad::Call::deposit { asset, amount, .. } => {
			match asset {
				Asset::Custom(id) => {
					let pallet_balance =
						Assets::balance(id, MultiAssetDelegation::pallet_account());
					assert!(pallet_balance.ge(&amount), "pallet balance not enough");
				},
				Asset::Erc20(token) => {
					let pallet_balance = MultiAssetDelegation::query_erc20_balance_of(
						token,
						MultiAssetDelegation::pallet_evm_account(),
					)
					.unwrap_or_default()
					.0;
					assert!(pallet_balance.ge(&amount.into()), "pallet balance not enough");
				},
			};

			assert_eq!(
				MultiAssetDelegation::delegators(&caller)
					.unwrap_or_default()
					.calculate_delegation_by_asset(asset),
				amount
			);
		},
		mad::Call::schedule_withdraw { asset, amount } => {
			let round = MultiAssetDelegation::current_round();
			assert!(
				MultiAssetDelegation::delegators(&caller)
					.unwrap_or_default()
					.get_withdraw_requests()
					.contains(&WithdrawRequest { asset, amount, requested_round: round }),
				"withdraw request not found"
			);
		},
		mad::Call::execute_withdraw { .. } => {
			assert!(
				MultiAssetDelegation::delegators(&caller)
					.unwrap_or_default()
					.get_withdraw_requests()
					.is_empty(),
				"withdraw requests not removed"
			);
		},
		mad::Call::cancel_withdraw { asset, amount } => {
			let round = MultiAssetDelegation::current_round();
			assert!(
				!MultiAssetDelegation::delegators(&caller)
					.unwrap_or_default()
					.get_withdraw_requests()
					.contains(&WithdrawRequest { asset, amount, requested_round: round }),
				"withdraw request not removed"
			);
		},
		mad::Call::delegate { operator, asset, amount, .. } => {
			let delegator = MultiAssetDelegation::delegators(&caller).unwrap_or_default();
			let operator_info = MultiAssetDelegation::operator_info(&operator).unwrap_or_default();
			assert!(
				delegator
					.calculate_delegation_by_operator(operator)
					.iter()
					.find_map(|x| { if x.asset == asset { Some(x.amount) } else { None } })
					.ge(&Some(amount)),
				"delegation amount not set"
			);
			assert!(
				operator_info
					.delegations
					.iter()
					.find_map(|x| {
						if x.delegator == caller && x.asset == asset {
							Some(x.amount)
						} else {
							None
						}
					})
					.ge(&Some(amount)),
				"delegator not added to operator"
			);
		},
		mad::Call::schedule_delegator_unstake { operator, asset, amount } => {},
		mad::Call::execute_delegator_unstake {} => {},
		mad::Call::cancel_delegator_unstake { operator, asset, amount } => {},
		other => unimplemented!("sanity checks for call: {other:?} not implemented"),
	}
}
