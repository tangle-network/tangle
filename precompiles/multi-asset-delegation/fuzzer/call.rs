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
use sp_runtime::traits::Scale;
use sp_runtime::DispatchResult;

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
fn join_operators<R: Rng>(rng: &mut R, who: &Address) -> DispatchResult {
	let minimum_bond = <<Runtime as mad::Config>::MinOperatorBondAmount as Get<Balance>>::get();
	let multiplier = rng.gen_range(1..50u64);
	let who_account_id = <TestAccount as AddressMapping<AccountId>>::into_account_id(who.0);
	let _ = Balances::deposit_creating(&who_account_id, minimum_bond.mul(multiplier));
	let bond_amount = minimum_bond.mul(multiplier);
	MultiAssetDelegation::join_operators(RuntimeOrigin::signed(who_account_id), bond_amount)
}

fn random_calls<R: Rng>(mut rng: &mut R) -> impl IntoIterator<Item = (PCall, Address)> {
	let op = PCall::selectors().choose(rng).cloned().unwrap();
	match op {
		_ if op == PCall::deposit_selectors()[0] => {
			// Deposit
			let who = random_address(&mut rng);
			fund_account(&mut rng, &who);
			let (asset_id, token_address) = match random_asset(&mut rng) {
				Asset::Custom(id) => (id.into(), Default::default()),
				Asset::Erc20(token) => (0.into(), token.into()),
			};
			let amount = random_ed_multiple(&mut rng).into();
			vec![(PCall::deposit { asset_id, amount, token_address, lock_multiplier: 0 }, who)]
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
			join_operators(&mut rng, &operator).unwrap();
			vec![(
				PCall::delegate {
					operator: operator.0.into(),
					asset_id,
					token_address,
					amount,
					blueprint_selection,
				},
				who,
			)]
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
			join_operators(&mut rng, &operator).unwrap();
			vec![(
				PCall::schedule_delegator_unstake {
					operator: operator.0.into(),
					asset_id,
					token_address,
					amount,
				},
				who,
			)]
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
			join_operators(&mut rng, &operator).unwrap();
			vec![(
				PCall::cancel_delegator_unstake {
					operator: operator.0.into(),
					asset_id,
					token_address,
					amount,
				},
				who,
			)]
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
		PCall::deposit { asset_id, amount, token_address, lock_multiplier: 0 } => {
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
