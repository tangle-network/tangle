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

use fp_evm::Context;
use fp_evm::PrecompileSet;
use frame_support::traits::{Currency, Get};
use honggfuzz::fuzz;
use pallet_evm::AddressMapping;
use pallet_evm_precompile_multi_asset_delegation::{
	mock::*, mock_evm::PrecompilesValue, MultiAssetDelegationPrecompileCall as MADPrecompileCall,
};
use pallet_multi_asset_delegation::{
	mock::{Asset, AssetId},
	pallet as mad,
	types::*,
};
use precompile_utils::prelude::*;
use precompile_utils::testing::*;
use rand::{seq::SliceRandom, Rng};
use sp_runtime::traits::{Scale, Zero};

const MAX_ED_MULTIPLE: Balance = 10_000;
const MIN_ED_MULTIPLE: Balance = 10;

type PCall = MADPrecompileCall<Runtime>;

fn random_address<R: Rng>(rng: &mut R) -> Address {
	Address(rng.gen::<[u8; 20]>().into())
}

/// Grab random accounts.
fn random_signed_origin<R: Rng>(rng: &mut R) -> (RuntimeOrigin, Address) {
	let addr = random_address(rng);
	let signer = <TestAccount as AddressMapping<AccountId>>::into_account_id(addr.0);
	(RuntimeOrigin::signed(signer), addr)
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

fn fund_account<R: Rng>(rng: &mut R, address: &Address) {
	let target_amount = random_ed_multiple(rng);
	let signer = <TestAccount as AddressMapping<AccountId>>::into_account_id(address.0);
	if let Some(top_up) = target_amount.checked_sub(Balances::free_balance(signer)) {
		let _ = Balances::deposit_creating(&signer, top_up);
	}
	assert!(Balances::free_balance(signer) >= target_amount);
}

/// Join operators call.
fn join_operators_call<R: Rng>(rng: &mut R, who: &Address) -> (PCall, Address) {
	let minimum_bond = <<Runtime as mad::Config>::MinOperatorBondAmount as Get<Balance>>::get();
	let multiplier = rng.gen_range(1..50u64);
	let who_account_id = <TestAccount as AddressMapping<AccountId>>::into_account_id(who.0);
	let _ = Balances::deposit_creating(&who_account_id, minimum_bond.mul(multiplier));
	let bond_amount = minimum_bond.mul(multiplier).into();
	(PCall::join_operators { bond_amount }, *who)
}

fn random_calls<R: Rng>(mut rng: &mut R) -> impl IntoIterator<Item = (PCall, Address)> {
	let op = PCall::selectors().choose(rng).cloned().unwrap();
	match op {
		_ if op == PCall::join_operators_selectors()[0] => {
			// join_operators
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			vec![join_operators_call(&mut rng, &who)]
		},
		_ if op == PCall::schedule_leave_operators_selectors()[0] => {
			// Schedule leave operators
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			vec![join_operators_call(rng, &who), (PCall::schedule_leave_operators {}, who)]
		},
		_ if op == PCall::cancel_leave_operators_selectors()[0] => {
			// Cancel leave operators
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			vec![join_operators_call(rng, &who), (PCall::cancel_leave_operators {}, who)]
		},
		_ if op == PCall::execute_leave_operators_selectors()[0] => {
			// Execute leave operators
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			vec![join_operators_call(rng, &who), (PCall::execute_leave_operators {}, who)]
		},
		_ if op == PCall::operator_bond_more_selectors()[0] => {
			// Operator bond more
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			let additional_bond = random_ed_multiple(&mut rng).into();
			vec![
				join_operators_call(rng, &who),
				(PCall::operator_bond_more { additional_bond }, who),
			]
		},
		_ if op == PCall::schedule_operator_unstake_selectors()[0] => {
			// Schedule operator unstake
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			let unstake_amount = random_ed_multiple(&mut rng).into();
			vec![
				join_operators_call(rng, &who),
				(PCall::schedule_operator_unstake { unstake_amount }, who),
			]
		},
		_ if op == PCall::execute_operator_unstake_selectors()[0] => {
			// Execute operator unstake
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			vec![join_operators_call(rng, &who), (PCall::execute_operator_unstake {}, who)]
		},
		_ if op == PCall::cancel_operator_unstake_selectors()[0] => {
			// Cancel operator unstake
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			vec![join_operators_call(rng, &who), (PCall::cancel_operator_unstake {}, who)]
		},
		_ if op == PCall::go_offline_selectors()[0] => {
			// Go offline
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			vec![join_operators_call(rng, &who), (PCall::go_offline {}, who)]
		},
		_ if op == PCall::go_online_selectors()[0] => {
			// Go online
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			vec![join_operators_call(rng, &who), (PCall::go_online {}, who)]
		},
		_ if op == PCall::deposit_selectors()[0] => {
			// Deposit
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			let (asset_id, token_address) = match random_asset(&mut rng) {
				Asset::Custom(id) => (id.into(), Default::default()),
				Asset::Erc20(token) => (0.into(), token.into()),
			};
			let amount = random_ed_multiple(&mut rng).into();
			vec![(PCall::deposit { asset_id, amount, token_address }, who)]
		},
		_ if op == PCall::schedule_withdraw_selectors()[0] => {
			// Schedule withdraw
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			let (asset_id, token_address) = match random_asset(&mut rng) {
				Asset::Custom(id) => (id.into(), Default::default()),
				Asset::Erc20(token) => (0.into(), token.into()),
			};
			let amount = random_ed_multiple(&mut rng).into();
			vec![(PCall::schedule_withdraw { asset_id, token_address, amount }, who)]
		},
		_ if op == PCall::execute_withdraw_selectors()[0] => {
			// Execute withdraw
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			vec![(PCall::execute_withdraw {}, who)]
		},
		_ if op == PCall::cancel_withdraw_selectors()[0] => {
			// Cancel withdraw
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			let (asset_id, token_address) = match random_asset(&mut rng) {
				Asset::Custom(id) => (id.into(), Default::default()),
				Asset::Erc20(token) => (0.into(), token.into()),
			};
			let amount = random_ed_multiple(&mut rng).into();
			vec![(PCall::cancel_withdraw { asset_id, amount, token_address }, who)]
		},
		_ if op == PCall::delegate_selectors()[0] => {
			// Delegate
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			let (_, operator) = random_signed_origin(&mut rng);
			let (asset_id, token_address) = match random_asset(&mut rng) {
				Asset::Custom(id) => (id.into(), Default::default()),
				Asset::Erc20(token) => (0.into(), token.into()),
			};
			let amount = random_ed_multiple(&mut rng).into();
			let blueprint_selection = {
				let count = rng.gen_range(1..MaxDelegatorBlueprints::get());
				(0..count).map(|_| rng.gen::<u64>()).collect::<Vec<_>>()
			};
			vec![
				join_operators_call(&mut rng, &operator),
				(
					PCall::delegate {
						operator: operator.0.into(),
						asset_id,
						token_address,
						amount,
						blueprint_selection,
					},
					who,
				),
			]
		},
		_ if op == PCall::schedule_delegator_unstake_selectors()[0] => {
			// Schedule delegator unstakes
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			let (_, operator) = random_signed_origin(&mut rng);
			let (asset_id, token_address) = match random_asset(&mut rng) {
				Asset::Custom(id) => (id.into(), Default::default()),
				Asset::Erc20(token) => (0.into(), token.into()),
			};
			let amount = random_ed_multiple(&mut rng).into();
			vec![
				join_operators_call(&mut rng, &operator),
				(
					PCall::schedule_delegator_unstake {
						operator: operator.0.into(),
						asset_id,
						token_address,
						amount,
					},
					who,
				),
			]
		},
		_ if op == PCall::execute_delegator_unstake_selectors()[0] => {
			// Execute delegator unstake
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			vec![(PCall::execute_delegator_unstake {}, who)]
		},
		_ if op == PCall::cancel_delegator_unstake_selectors()[0] => {
			// Cancel delegator unstake
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			let (_, operator) = random_signed_origin(&mut rng);
			let (asset_id, token_address) = match random_asset(&mut rng) {
				Asset::Custom(id) => (id.into(), Default::default()),
				Asset::Erc20(token) => (0.into(), token.into()),
			};
			let amount = random_ed_multiple(&mut rng).into();
			vec![
				join_operators_call(&mut rng, &operator),
				(
					PCall::cancel_delegator_unstake {
						operator: operator.0.into(),
						asset_id,
						token_address,
						amount,
					},
					who,
				),
			]
		},
		_ => {
			unimplemented!("unknown call name: {}", op)
		},
	}
}

fn main() {
	sp_tracing::try_init_simple();
	let mut ext = ExtBuilder::default().build();
	let mut block_number = 1;
	let to = Precompile1.into();
	loop {
		fuzz!(|seed: [u8; 32]| {
			use ::rand::{rngs::SmallRng, SeedableRng};
			let mut rng = SmallRng::from_seed(seed);

			ext.execute_with(|| {
				System::set_block_number(block_number);
				for (call, who) in random_calls(&mut rng) {
					let mut handle = MockHandle::new(
						to,
						Context {
							address: to,
							caller: who.into(),
							apparent_value: Default::default(),
						},
					);
					let mut handle_clone = MockHandle::new(
						to,
						Context {
							address: to,
							caller: who.into(),
							apparent_value: Default::default(),
						},
					);
					let encoded = call.encode();
					handle.input = encoded.clone();
					let call_clone = PCall::parse_call_data(&mut handle).unwrap();
					handle_clone.input = encoded;
					let outcome = PrecompilesValue::get().execute(&mut handle).unwrap();
					sp_tracing::trace!(?who, ?outcome, "fuzzed call");

					// execute sanity checks at a fixed interval, possibly on every block.
					if let Ok(out) = outcome {
						sp_tracing::info!("running sanity checks..");
						do_sanity_checks(call_clone, who, out);
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
fn do_sanity_checks(call: PCall, origin: Address, outcome: PrecompileOutput) {
	let caller = <TestAccount as AddressMapping<AccountId>>::into_account_id(origin.0);
	match call {
		PCall::join_operators { bond_amount } => {
			assert!(mad::Operators::<Runtime>::contains_key(caller), "operator not found");
			assert_eq!(
				MultiAssetDelegation::operator_info(caller).unwrap_or_default().stake,
				bond_amount.as_u64()
			);
			assert!(
				Balances::reserved_balance(caller).ge(&bond_amount.as_u64()),
				"bond amount not reserved"
			);
		},
		PCall::schedule_leave_operators {} => {
			assert!(mad::Operators::<Runtime>::contains_key(caller), "operator not found");
			let current_round = mad::CurrentRound::<Runtime>::get();
			let leaving_time =
				<<Runtime as mad::Config>::LeaveOperatorsDelay as Get<u32>>::get() + current_round;
			assert_eq!(
				mad::Operators::<Runtime>::get(caller).unwrap_or_default().status,
				OperatorStatus::Leaving(leaving_time)
			);
		},
		PCall::cancel_leave_operators {} => {
			assert_eq!(
				mad::Operators::<Runtime>::get(caller).unwrap_or_default().status,
				OperatorStatus::Active
			);
		},
		PCall::execute_leave_operators {} => {
			assert!(!mad::Operators::<Runtime>::contains_key(caller), "operator not removed");
			assert!(Balances::reserved_balance(caller).is_zero(), "bond amount not unreserved");
		},
		PCall::operator_bond_more { additional_bond } => {
			let info = MultiAssetDelegation::operator_info(caller).unwrap_or_default();
			assert!(info.stake.ge(&additional_bond.as_u64()), "bond amount not increased");
			assert!(
				Balances::reserved_balance(caller).ge(&additional_bond.as_u64()),
				"bond amount not reserved"
			);
		},
		PCall::schedule_operator_unstake { unstake_amount } => {
			let info = MultiAssetDelegation::operator_info(caller).unwrap_or_default();
			let current_round = MultiAssetDelegation::current_round();
			let unstake_request = OperatorBondLessRequest {
				amount: unstake_amount.as_u64(),
				request_time: current_round,
			};
			assert_eq!(info.request, Some(unstake_request), "unstake request not set");
		},
		PCall::execute_operator_unstake {} => {
			let info = MultiAssetDelegation::operator_info(caller).unwrap_or_default();
			assert!(info.request.is_none(), "unstake request not removed");
			// reserved balance should be reduced and equal to the stake
			assert!(
				Balances::reserved_balance(caller).eq(&info.stake),
				"reserved balance not equal to stake"
			);
		},
		PCall::cancel_operator_unstake {} => {
			let info = MultiAssetDelegation::operator_info(caller).unwrap_or_default();
			assert!(info.request.is_none(), "unstake request not removed");
		},
		PCall::go_offline {} => {
			let info = MultiAssetDelegation::operator_info(caller).unwrap_or_default();
			assert_eq!(info.status, OperatorStatus::Inactive, "status not set to inactive");
		},
		PCall::go_online {} => {
			let info = MultiAssetDelegation::operator_info(caller).unwrap_or_default();
			assert_eq!(info.status, OperatorStatus::Active, "status not set to active");
		},
		PCall::deposit { asset_id, amount, token_address } => {
			let (deposit_asset, amount) = match (asset_id.as_u32(), token_address.0 .0) {
				(0, erc20_token) if erc20_token != [0; 20] => {
					(Asset::Erc20(erc20_token.into()), amount)
				},
				(other_asset_id, _) => (Asset::Custom(other_asset_id.into()), amount),
			};
			match deposit_asset {
				Asset::Custom(id) => {
					let pallet_balance =
						Assets::balance(id, MultiAssetDelegation::pallet_account());
					assert!(pallet_balance.ge(&amount.as_u64()), "pallet balance not enough");
				},
				Asset::Erc20(token) => {
					let pallet_balance = MultiAssetDelegation::query_erc20_balance_of(
						token,
						MultiAssetDelegation::pallet_evm_account(),
					)
					.unwrap_or_default()
					.0;
					assert!(pallet_balance.ge(&amount), "pallet balance not enough");
				},
			};

			assert_eq!(
				MultiAssetDelegation::delegators(caller)
					.unwrap_or_default()
					.calculate_delegation_by_asset(deposit_asset),
				amount.as_u64()
			);
		},
		PCall::schedule_withdraw { asset_id, amount, token_address } => {
			let (deposit_asset, amount) = match (asset_id.as_u32(), token_address.0 .0) {
				(0, erc20_token) if erc20_token != [0; 20] => {
					(Asset::Erc20(erc20_token.into()), amount)
				},
				(other_asset_id, _) => (Asset::Custom(other_asset_id.into()), amount),
			};
			let round = MultiAssetDelegation::current_round();
			assert!(
				MultiAssetDelegation::delegators(caller)
					.unwrap_or_default()
					.get_withdraw_requests()
					.contains(&WithdrawRequest {
						asset_id: deposit_asset,
						amount: amount.as_u64(),
						requested_round: round
					}),
				"withdraw request not found"
			);
		},
		PCall::execute_withdraw { .. } => {
			assert!(
				MultiAssetDelegation::delegators(caller)
					.unwrap_or_default()
					.get_withdraw_requests()
					.is_empty(),
				"withdraw requests not removed"
			);
		},
		PCall::cancel_withdraw { asset_id, amount, token_address } => {
			let round = MultiAssetDelegation::current_round();

			let (deposit_asset, amount) = match (asset_id.as_u32(), token_address.0 .0) {
				(0, erc20_token) if erc20_token != [0; 20] => {
					(Asset::Erc20(erc20_token.into()), amount)
				},
				(other_asset_id, _) => (Asset::Custom(other_asset_id.into()), amount),
			};
			assert!(
				!MultiAssetDelegation::delegators(caller)
					.unwrap_or_default()
					.get_withdraw_requests()
					.contains(&WithdrawRequest {
						asset_id: deposit_asset,
						amount: amount.as_u64(),
						requested_round: round
					}),
				"withdraw request not removed"
			);
		},
		PCall::delegate { operator, asset_id, amount, token_address, .. } => {
			let (deposit_asset, amount) = match (asset_id.as_u32(), token_address.0 .0) {
				(0, erc20_token) if erc20_token != [0; 20] => {
					(Asset::Erc20(erc20_token.into()), amount)
				},
				(other_asset_id, _) => (Asset::Custom(other_asset_id.into()), amount),
			};
			let operator_account = AccountId::from(operator.0);
			let delegator = MultiAssetDelegation::delegators(caller).unwrap_or_default();
			let operator_info =
				MultiAssetDelegation::operator_info(operator_account).unwrap_or_default();
			assert!(
				delegator
					.calculate_delegation_by_operator(operator_account)
					.iter()
					.find_map(|x| {
						if x.asset_id == deposit_asset {
							Some(x.amount)
						} else {
							None
						}
					})
					.ge(&Some(amount.as_u64())),
				"delegation amount not set"
			);
			assert!(
				operator_info
					.delegations
					.iter()
					.find_map(|x| {
						if x.delegator == caller && x.asset_id == deposit_asset {
							Some(x.amount)
						} else {
							None
						}
					})
					.ge(&Some(amount.as_u64())),
				"delegator not added to operator"
			);
		},
		_ => {
			// ignore other calls
		},
	}
}
