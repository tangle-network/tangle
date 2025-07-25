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

//! Autogenerated weights for multi_asset_delegation
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 42.0.0
//! DATE: 2025-07-08, STEPS: `10`, REPEAT: `2`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// target/release/tangle
// benchmark
// --chain=dev
// --steps=10
// --repeat=2
// --pallet=multi_asset_delegation
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for multi_asset_delegation.
pub trait WeightInfo {
	fn join_operators() -> Weight;
	fn schedule_leave_operators() -> Weight;
	fn cancel_leave_operators() -> Weight;
	fn execute_leave_operators() -> Weight;
	fn operator_bond_more() -> Weight;
	fn schedule_operator_unstake() -> Weight;
	fn execute_operator_unstake() -> Weight;
	fn cancel_operator_unstake() -> Weight;
	fn go_offline() -> Weight;
	fn go_online() -> Weight;
	fn deposit() -> Weight;
	fn schedule_withdraw() -> Weight;
	fn execute_withdraw() -> Weight;
	fn cancel_withdraw() -> Weight;
	fn delegate() -> Weight;
	fn schedule_delegator_unstake() -> Weight;
	fn execute_delegator_unstake() -> Weight;
	fn cancel_delegator_unstake() -> Weight;
	fn delegate_nomination() -> Weight;
	fn schedule_nomination_unstake() -> Weight;
	fn execute_nomination_unstake() -> Weight;
	fn cancel_nomination_unstake() -> Weight;
	fn add_blueprint_id() -> Weight;
	fn remove_blueprint_id() -> Weight;
}

/// Weight functions needed for rewards pallet.
/// Weights for `pallet_rewards` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), mode: `Measured`)
	fn join_operators() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1152`
		//  Estimated: `2304`
		// Minimum execution time: 40_823_000 picoseconds.
		Weight::from_parts(41_223_000, 2304)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn schedule_leave_operators() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 39_256_000 picoseconds.
		Weight::from_parts(39_856_000, 2048)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn cancel_leave_operators() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 35_789_000 picoseconds.
		Weight::from_parts(35_789_000, 2048)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `MultiAssetDelegation::CurrentRound` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::CurrentRound` (`max_values`: Some(1), `max_size`: Some(4), mode: `Measured`)
	fn execute_leave_operators() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1028`
		//  Estimated: `2056`
		// Minimum execution time: 45_234_000 picoseconds.
		Weight::from_parts(45_234_000, 2056)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn operator_bond_more() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 39_876_000 picoseconds.
		Weight::from_parts(39_876_000, 2048)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn schedule_operator_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 41_567_000 picoseconds.
		Weight::from_parts(41_567_000, 2048)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `MultiAssetDelegation::CurrentRound` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::CurrentRound` (`max_values`: Some(1), `max_size`: Some(4), mode: `Measured`)
	fn execute_operator_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1028`
		//  Estimated: `2056`
		// Minimum execution time: 44_789_000 picoseconds.
		Weight::from_parts(44_789_000, 2056)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn cancel_operator_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 36_234_000 picoseconds.
		Weight::from_parts(36_234_000, 2048)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn go_offline() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 33_456_000 picoseconds.
		Weight::from_parts(33_456_000, 2048)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn go_online() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 34_123_000 picoseconds.
		Weight::from_parts(34_123_000, 2048)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::AssetConfigs` (r:1 w:0)
	/// Proof: `MultiAssetDelegation::AssetConfigs` (`max_values`: None, `max_size`: Some(128), mode: `Measured`)
	/// Storage: `MultiAssetDelegation::Deposits` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Deposits` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `System::Account` (r:0 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), mode: `Measured`)
	fn deposit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1152`
		//  Estimated: `2304`
		// Minimum execution time: 50_167_000 picoseconds.
		Weight::from_parts(51_067_000, 2304)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Deposits` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Deposits` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), mode: `Measured`)
	fn schedule_withdraw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1152`
		//  Estimated: `2304`
		// Minimum execution time: 43_234_000 picoseconds.
		Weight::from_parts(43_234_000, 2304)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Deposits` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Deposits` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), mode: `Measured`)
	fn execute_withdraw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1152`
		//  Estimated: `2304`
		// Minimum execution time: 45_234_000 picoseconds.
		Weight::from_parts(45_234_000, 2304)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Deposits` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Deposits` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn cancel_withdraw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 36_234_000 picoseconds.
		Weight::from_parts(36_234_000, 2048)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn delegate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2048`
		//  Estimated: `4096`
		// Minimum execution time: 47_267_000 picoseconds.
		Weight::from_parts(47_767_000, 4096)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn schedule_delegator_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 41_567_000 picoseconds.
		Weight::from_parts(41_567_000, 2048)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `MultiAssetDelegation::CurrentRound` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::CurrentRound` (`max_values`: Some(1), `max_size`: Some(4), mode: `Measured`)
	fn execute_delegator_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1028`
		//  Estimated: `2056`
		// Minimum execution time: 44_789_000 picoseconds.
		Weight::from_parts(44_789_000, 2056)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn cancel_delegator_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 36_234_000 picoseconds.
		Weight::from_parts(36_234_000, 2048)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn delegate_nomination() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2048`
		//  Estimated: `4096`
		// Minimum execution time: 48_567_000 picoseconds.
		Weight::from_parts(48_567_000, 4096)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn schedule_nomination_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 41_567_000 picoseconds.
		Weight::from_parts(41_567_000, 2048)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `MultiAssetDelegation::CurrentRound` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::CurrentRound` (`max_values`: Some(1), `max_size`: Some(4), mode: `Measured`)
	fn execute_nomination_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1028`
		//  Estimated: `2056`
		// Minimum execution time: 44_789_000 picoseconds.
		Weight::from_parts(44_789_000, 2056)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn cancel_nomination_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 36_234_000 picoseconds.
		Weight::from_parts(36_234_000, 2048)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::BlueprintIds` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::BlueprintIds` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn add_blueprint_id() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 35_234_000 picoseconds.
		Weight::from_parts(35_234_000, 2048)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::BlueprintIds` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::BlueprintIds` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn remove_blueprint_id() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 35_234_000 picoseconds.
		Weight::from_parts(35_234_000, 2048)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), mode: `Measured`)
	fn join_operators() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1152`
		//  Estimated: `2304`
		// Minimum execution time: 40_823_000 picoseconds.
		Weight::from_parts(41_223_000, 2304)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn schedule_leave_operators() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 38_456_000 picoseconds.
		Weight::from_parts(38_456_000, 2048)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn cancel_leave_operators() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 35_789_000 picoseconds.
		Weight::from_parts(35_789_000, 2048)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `MultiAssetDelegation::CurrentRound` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::CurrentRound` (`max_values`: Some(1), `max_size`: Some(4), mode: `Measured`)
	fn execute_leave_operators() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1028`
		//  Estimated: `2056`
		// Minimum execution time: 45_234_000 picoseconds.
		Weight::from_parts(45_234_000, 2056)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn operator_bond_more() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 39_876_000 picoseconds.
		Weight::from_parts(39_876_000, 2048)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn schedule_operator_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 41_567_000 picoseconds.
		Weight::from_parts(41_567_000, 2048)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `MultiAssetDelegation::CurrentRound` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::CurrentRound` (`max_values`: Some(1), `max_size`: Some(4), mode: `Measured`)
	fn execute_operator_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1028`
		//  Estimated: `2056`
		// Minimum execution time: 44_789_000 picoseconds.
		Weight::from_parts(44_789_000, 2056)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn cancel_operator_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 36_234_000 picoseconds.
		Weight::from_parts(36_234_000, 2048)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn go_offline() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 33_456_000 picoseconds.
		Weight::from_parts(33_456_000, 2048)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn go_online() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 34_123_000 picoseconds.
		Weight::from_parts(34_123_000, 2048)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::AssetConfigs` (r:1 w:0)
	/// Proof: `MultiAssetDelegation::AssetConfigs` (`max_values`: None, `max_size`: Some(128), mode: `Measured`)
	/// Storage: `MultiAssetDelegation::Deposits` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Deposits` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `System::Account` (r:0 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), mode: `Measured`)
	fn deposit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1152`
		//  Estimated: `2304`
		// Minimum execution time: 48_567_000 picoseconds.
		Weight::from_parts(48_567_000, 2304)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Deposits` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Deposits` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), mode: `Measured`)
	fn schedule_withdraw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1152`
		//  Estimated: `2304`
		// Minimum execution time: 43_234_000 picoseconds.
		Weight::from_parts(43_234_000, 2304)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Deposits` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Deposits` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), mode: `Measured`)
	fn execute_withdraw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1152`
		//  Estimated: `2304`
		// Minimum execution time: 45_234_000 picoseconds.
		Weight::from_parts(45_234_000, 2304)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Deposits` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Deposits` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn cancel_withdraw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 36_234_000 picoseconds.
		Weight::from_parts(36_234_000, 2048)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn delegate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2048`
		//  Estimated: `4096`
		// Minimum execution time: 47_267_000 picoseconds.
		Weight::from_parts(47_767_000, 4096)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn schedule_delegator_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 41_567_000 picoseconds.
		Weight::from_parts(41_567_000, 2048)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `MultiAssetDelegation::CurrentRound` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::CurrentRound` (`max_values`: Some(1), `max_size`: Some(4), mode: `Measured`)
	fn execute_delegator_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1028`
		//  Estimated: `2056`
		// Minimum execution time: 44_789_000 picoseconds.
		Weight::from_parts(44_789_000, 2056)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn cancel_delegator_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 36_234_000 picoseconds.
		Weight::from_parts(36_234_000, 2048)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `MultiAssetDelegation::Operators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Operators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn delegate_nomination() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2048`
		//  Estimated: `4096`
		// Minimum execution time: 48_567_000 picoseconds.
		Weight::from_parts(48_567_000, 4096)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn schedule_nomination_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 41_567_000 picoseconds.
		Weight::from_parts(41_567_000, 2048)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	/// Storage: `MultiAssetDelegation::CurrentRound` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::CurrentRound` (`max_values`: Some(1), `max_size`: Some(4), mode: `Measured`)
	fn execute_nomination_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1028`
		//  Estimated: `2056`
		// Minimum execution time: 44_789_000 picoseconds.
		Weight::from_parts(44_789_000, 2056)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}

	/// Storage: `MultiAssetDelegation::Delegators` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::Delegators` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn cancel_nomination_unstake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 36_234_000 picoseconds.
		Weight::from_parts(36_234_000, 2048)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::BlueprintIds` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::BlueprintIds` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn add_blueprint_id() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 35_234_000 picoseconds.
		Weight::from_parts(35_234_000, 2048)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	/// Storage: `MultiAssetDelegation::BlueprintIds` (r:1 w:1)
	/// Proof: `MultiAssetDelegation::BlueprintIds` (`max_values`: None, `max_size`: Some(1024), mode: `Measured`)
	fn remove_blueprint_id() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024`
		//  Estimated: `2048`
		// Minimum execution time: 35_234_000 picoseconds.
		Weight::from_parts(35_234_000, 2048)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}