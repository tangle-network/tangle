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
use crate::{types::StakeTier, BalanceOf, Config, Pallet as Credits};
use frame_benchmarking::{account, v2::*, BenchmarkError};
use frame_support::{
	assert_ok,
	traits::{Currency, Get},
	BoundedVec,
};
use frame_system::RawOrigin;
use sp_runtime::{traits::Zero, Saturating};
use sp_std::prelude::*;
use tangle_primitives::{
	services::Asset,
	traits::{
		MultiAssetDelegationBenchmarkingHelperDelegation,
		MultiAssetDelegationBenchmarkingHelperOperator,
	},
};

const SEED: u32 = 0;
const INITIAL_BALANCE: u32 = 1_000_000;

/// Helper function to prepare an account with the given amount of TNT
fn setup_account<T: Config>(acc: &'static str, account_index: u32) -> T::AccountId {
	let account: T::AccountId = account(acc, account_index, SEED);
	T::Currency::make_free_balance_be(
		&account,
		T::Currency::minimum_balance().saturating_mul(INITIAL_BALANCE.into()),
	);
	account
}

/// Helper function to setup delegation for benchmarking
fn setup_nominator<T: Config>(
	delegator: T::AccountId,
	bond_amount: BalanceOf<T>,
	operator: T::AccountId,
	asset: Asset<T::AssetId>,
	amount: BalanceOf<T>,
) -> Result<(), &'static str> {
	assert_ok!(<T::BenchmarkingHelper as MultiAssetDelegationBenchmarkingHelperOperator<
		T::AccountId,
		BalanceOf<T>,
	>>::handle_deposit_and_create_operator_be(operator.clone(), bond_amount));

	assert_ok!(<T::BenchmarkingHelper as MultiAssetDelegationBenchmarkingHelperDelegation<
		T::AccountId,
		BalanceOf<T>,
		T::AssetId,
	>>::process_delegate_be(delegator, operator, asset, amount));

	Ok(())
}

/// Create stake tiers for benchmarking
fn create_stake_tiers<T: Config>(tiers_count: u32) -> Vec<StakeTier<BalanceOf<T>>> {
	let mut tiers = Vec::new();
	for i in 0..tiers_count {
		// Create increasing thresholds and rates
		let threshold: BalanceOf<T> = ((i + 1) * 1000u32).into();
		let rate: BalanceOf<T> = ((i + 1) * 10u32).into();

		tiers.push(StakeTier { threshold, rate_per_block: rate });
	}
	tiers
}

#[benchmarks(where
	T::AssetId: From<u32>,
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn burn() -> Result<(), BenchmarkError> {
		// Setup: Create an account with sufficient balance for worst case scenario
		// Following the pattern from multi-asset-delegation benchmarks
		let account: T::AccountId = setup_account::<T>("account", 1);

		// For worst case, use a large burn amount relative to minimum balance
		// This ensures we test the maximum burn scenario
		let burn_amount: BalanceOf<T> = T::Currency::minimum_balance() * 1000u32.into();

		#[extrinsic_call]
		burn(RawOrigin::Signed(account.clone()), burn_amount);

		Ok(())
	}

	#[benchmark]
	fn claim_credits() -> Result<(), BenchmarkError> {
		// Setup: Use maximum stake tier threshold for worst case scenario
		let stored_tiers = Credits::<T>::stake_tiers();
		let max_stake_amount = if stored_tiers.is_empty() {
			10_000u32.into() // Fallback if no tiers configured
		} else {
			// Use the highest tier threshold
			stored_tiers.iter().map(|t| t.threshold).max().unwrap_or(10_000u32.into())
		};
		let account = setup_account::<T>("account", 1);
		let operator = setup_account::<T>("operator", 1);

		// asset to delegate
		let asset_id_u32 = 0_u32;
		let asset_id = Asset::Custom(asset_id_u32.into());

		// Setup delegation to enable credit accrual
		setup_nominator::<T>(
			account.clone(),
			max_stake_amount,
			operator.clone(),
			asset_id,
			max_stake_amount,
		)
		.unwrap();

		// Setup global stake tiers for the benchmark with maximum rate
		// claim_credits uses get_current_rate which reads from StoredStakeTiers (global tiers)
		let max_tiers = T::MaxStakeTiers::get() as u32;
		let global_tiers = create_stake_tiers::<T>(max_tiers.min(10)); // Limit to reasonable size
		Credits::<T>::set_stake_tiers(RawOrigin::Root.into(), global_tiers).unwrap();

		// Advance blocks by the full claim window for worst case scenario
		let window = T::ClaimWindowBlocks::get();
		let start_block = frame_system::Pallet::<T>::block_number();
		let end_block = start_block.saturating_add(window);
		frame_system::Pallet::<T>::set_block_number(end_block);

		// Get the actual max claimable amount within the window
		// This ensures we don't exceed what's actually available
		let max_claimable = Credits::<T>::get_accrued_amount(&account, Some(end_block))
			.map_err(|_| BenchmarkError::Weightless)?;

		// For worst case scenario, we must have credits available
		// If setup results in zero credits, the benchmark setup is wrong
		assert!(!max_claimable.is_zero());

		// Use the maximum claimable amount for worst case
		let claim_amount = max_claimable;

		// Create a bounded ID for the claim
		let id_str = b"benchmark_claim_id".to_vec();
		let bounded_id: BoundedVec<u8, T::MaxOffchainAccountIdLength> =
			id_str.try_into().expect("ID should not be too long");

		#[extrinsic_call]
		claim_credits(RawOrigin::Signed(account.clone()), claim_amount, bounded_id.clone());

		Ok(())
	}

	#[benchmark]
	fn set_stake_tiers() -> Result<(), BenchmarkError> {
		// Use the maximum allowed number of tiers to benchmark worst-case scenario
		let max_tiers = T::MaxStakeTiers::get() as u32;

		// Create a set of stake tiers with increasing thresholds and rates
		let new_tiers = create_stake_tiers::<T>(max_tiers);

		#[extrinsic_call]
		set_stake_tiers(RawOrigin::Root, new_tiers);

		Ok(())
	}

	#[benchmark]
	fn claim_credits_with_asset() -> Result<(), BenchmarkError> {
		// Setup: Use maximum stake tier threshold for worst case scenario
		let stored_tiers = Credits::<T>::stake_tiers();
		let max_stake_amount = if stored_tiers.is_empty() {
			10_000u32.into() // Fallback if no tiers configured
		} else {
			// Use the highest tier threshold
			stored_tiers.iter().map(|t| t.threshold).max().unwrap_or(10_000u32.into())
		};
		let account = setup_account::<T>("account", 1);
		let operator = setup_account::<T>("operator", 1);
		let asset_id = 0_u32;
		let asset = Asset::Custom(asset_id.into());

		// Setup delegation to enable credit accrual
		setup_nominator::<T>(
			account.clone(),
			max_stake_amount,
			operator.clone(),
			asset,
			max_stake_amount,
		)
		.unwrap();

		// Setup asset-specific stake tiers for the benchmark with maximum rate
		let max_tiers = T::MaxStakeTiers::get() as u32;
		let asset_tiers = create_stake_tiers::<T>(max_tiers.min(10)); // Limit to reasonable size
		Credits::<T>::set_asset_stake_tiers(RawOrigin::Root.into(), asset_id.into(), asset_tiers)
			.unwrap();

		// Advance blocks by the full claim window for worst case scenario
		let window = T::ClaimWindowBlocks::get();
		let start_block = frame_system::Pallet::<T>::block_number();
		let end_block = start_block.saturating_add(window);
		frame_system::Pallet::<T>::set_block_number(end_block);

		// Get the actual max claimable amount within the window for the specific asset
		// This ensures we don't exceed what's actually available
		let max_claimable =
			Credits::<T>::get_accrued_amount_for_asset(&account, Some(end_block), asset_id.into())
				.map_err(|_| BenchmarkError::Weightless)?;

		// For worst case scenario, we must have credits available
		// If setup results in zero credits, the benchmark setup is wrong
		assert!(!max_claimable.is_zero());

		// Use the maximum claimable amount for worst case
		let claim_amount = max_claimable;

		// Create a bounded ID for the claim
		let id_str = b"benchmark_asset_claim_id".to_vec();
		let bounded_id: BoundedVec<u8, T::MaxOffchainAccountIdLength> =
			id_str.try_into().expect("ID should not be too long");

		#[extrinsic_call]
		claim_credits_with_asset(
			RawOrigin::Signed(account.clone()),
			claim_amount,
			bounded_id.clone(),
			asset_id.into(),
		);

		Ok(())
	}

	#[benchmark]
	fn set_asset_stake_tiers() -> Result<(), BenchmarkError> {
		// Use the maximum allowed number of tiers to benchmark worst-case scenario
		let max_tiers = T::MaxStakeTiers::get() as u32;
		let asset_id = T::AssetId::default(); // Use default asset ID

		// Create a set of stake tiers with increasing thresholds and rates
		let new_tiers = create_stake_tiers::<T>(max_tiers);

		#[extrinsic_call]
		set_asset_stake_tiers(RawOrigin::Root, asset_id, new_tiers);

		Ok(())
	}

	impl_benchmark_test_suite!(Credits, crate::mock::new_test_ext(vec![]), crate::mock::Runtime);
}
