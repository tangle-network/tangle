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

//! Benchmarking for the Credits pallet.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::{BalanceOf, Config, LastRewardUpdateBlock, Pallet as Credits};
use frame_benchmarking::{v2::*, BenchmarkError};
use frame_support::{
	traits::{Currency, Get},
	BoundedVec,
};
use frame_system::RawOrigin;
use sp_runtime::{
	traits::{UniqueSaturatedInto, Zero},
	Saturating,
};
use sp_std::prelude::*;

const SEED: u32 = 0;

/// Helper function to prepare an account with the given amount of TNT
fn setup_account<T: Config>(account_index: u32, balance: BalanceOf<T>) -> T::AccountId {
	let account: T::AccountId = account("account", account_index, SEED);
	let _ = T::Currency::make_free_balance_be(&account, balance);
	account
}

/// Helper function to simulate delegation for an account
fn setup_delegation<T: Config>(
	delegator: &T::AccountId,
	stake_amount: BalanceOf<T>,
) -> Result<(), &'static str> {
	// For benchmarking purposes, we'll just ensure the account has enough balance
	let min_balance = stake_amount.saturating_mul(5u32.into());
	let _ = T::Currency::make_free_balance_be(delegator, min_balance);

	let current_block = frame_system::Pallet::<T>::block_number();
	LastRewardUpdateBlock::<T>::insert(delegator, current_block);

	Ok(())
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn burn() -> Result<(), BenchmarkError> {
		// Setup: Create an account with sufficient balance
		let burn_amount: BalanceOf<T> = 1000u32.into();
		let account = setup_account::<T>(1, burn_amount.saturating_mul(2u32.into()));

		#[extrinsic_call]
		burn(RawOrigin::Signed(account.clone()), burn_amount);

		Ok(())
	}

	#[benchmark]
	fn claim_credits() -> Result<(), BenchmarkError> {
		// Setup: Create an account with sufficient stake to earn credits
		let stake_amount: BalanceOf<T> = 1000u32.into();
		let account = setup_account::<T>(1, stake_amount.saturating_mul(2u32.into()));

		// Setup delegation to enable credit accrual
		setup_delegation::<T>(&account, stake_amount).unwrap();

		// Advance blocks to accrue some credits
		let start_block = frame_system::Pallet::<T>::block_number();
		let blocks_to_advance = 100u32;
		let end_block = start_block + blocks_to_advance.into();
		frame_system::Pallet::<T>::set_block_number(end_block);

		// Calculate a reasonable claim amount
		let rate = Credits::<T>::get_current_rate(stake_amount);
		let claim_amount = if rate.is_zero() {
			1u32.into()
		} else {
			// Convert blocks to the appropriate balance type
			let blocks_as_balance: BalanceOf<T> = blocks_to_advance.into();
			rate.saturating_mul(blocks_as_balance)
		};

		// Create a bounded ID for the claim
		let id_str = b"benchmark_claim_id".to_vec();
		let bounded_id: BoundedVec<u8, T::MaxOffchainAccountIdLength> =
			id_str.try_into().expect("ID should not be too long");

		#[extrinsic_call]
		claim_credits(RawOrigin::Signed(account.clone()), claim_amount, bounded_id.clone());

		Ok(())
	}

	impl_benchmark_test_suite!(Credits, crate::mock::new_test_ext(vec![]), crate::mock::Runtime);
}
