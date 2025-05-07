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
use crate::{Pallet as Credits, *};
use frame_benchmarking::v2::*;
use frame_support::{assert_ok, traits::Currency};
use frame_system::RawOrigin;
use sp_runtime::traits::Zero;
use sp_std::prelude::*;
use tangle_primitives::services::Asset;

const SEED: u32 = 0;

/// Helper function to prepare an account with the given amount of TNT
fn setup_account<T: Config>(account_index: u32, balance: BalanceOf<T>) -> T::AccountId {
	let account: T::AccountId = account("account", account_index, SEED);
	let _ = T::Currency::make_free_balance_be(&account, balance);
	account
}

/// Helper function to setup delegation for an account
fn setup_delegation<T: Config>(
	account: &T::AccountId,
	stake_amount: BalanceOf<T>,
) -> Result<(), &'static str> {
	// This is a simplified version - in a real implementation, you would need to
	// interact with the multi-asset delegation pallet to set up actual delegation

	// For benchmarking purposes, we'll just ensure the account has enough balance
	let min_balance = stake_amount.saturating_mul(2u32.into());
	let _ = T::Currency::make_free_balance_be(account, min_balance);

	// In a complete implementation, you would set up the delegation here
	// This might involve calling into the multi-asset delegation pallet

	Ok(())
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn burn() {
		// Setup: Create an account with sufficient balance
		let burn_amount: BalanceOf<T> = 1000u32.into();
		let account = setup_account::<T>(1, burn_amount.saturating_mul(2u32.into()));

		#[extrinsic_call]
		burn(RawOrigin::Signed(account.clone()), burn_amount);

		// Verify the burn was successful by checking the last event
		let conversion_rate = T::BurnConversionRate::get();
		let credits_granted = burn_amount.saturating_mul(conversion_rate);
		System::<T>::assert_last_event(
			Event::CreditsGrantedFromBurn {
				who: account,
				tnt_burned: burn_amount,
				credits_granted,
			}
			.into(),
		);
	}

	#[benchmark]
	fn claim_credits() {
		// Setup: Create an account with sufficient stake to earn credits
		let stake_amount: BalanceOf<T> = 1000u32.into();
		let account = setup_account::<T>(1, stake_amount.saturating_mul(2u32.into()));

		// Setup delegation to enable credit accrual
		setup_delegation::<T>(&account, stake_amount)?;

		// Advance blocks to accrue some credits
		let start_block = frame_system::Pallet::<T>::block_number();
		let blocks_to_advance = 100u32.into();
		let end_block = start_block + blocks_to_advance;
		frame_system::Pallet::<T>::set_block_number(end_block);

		// Calculate a reasonable claim amount
		let rate = Credits::<T>::get_current_rate(stake_amount);
		let claim_amount = if rate.is_zero() {
			1u32.into() // Fallback to a minimal amount if rate is zero
		} else {
			rate.saturating_mul(blocks_to_advance.into())
		};

		// Create a bounded ID for the claim
		let id_str = b"benchmark_claim_id".to_vec();
		let bounded_id: BoundedVec<u8, T::MaxOffchainAccountIdLength> =
			id_str.try_into().map_err(|_| "ID too long")?;

		#[extrinsic_call]
		claim_credits(RawOrigin::Signed(account.clone()), claim_amount, bounded_id.clone());

		// Verify the claim was successful by checking the last event
		System::<T>::assert_last_event(
			Event::CreditsClaimed {
				who: account,
				amount_claimed: claim_amount,
				offchain_account_id: bounded_id,
			}
			.into(),
		);
	}

	impl_benchmark_test_suite!(Credits, crate::mock::new_test_ext(vec![]), crate::mock::Runtime);
}
