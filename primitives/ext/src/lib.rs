// Copyright 2022-2025 Tangle Foundation.
// This file is part of Tangle.
// This file originated in Moonbeam's codebase.

// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tangle. If not, see <http://www.gnu.org/licenses/>.

//! Environmental-aware externalities for EVM tracing in Wasm runtime. This enables
//! capturing the - potentially large - trace output data in the host and keep
//! a low memory footprint in `--execution=wasm`.
//!
//! - The original trace Runtime Api call is wrapped `using` environmental (thread local).
//! - Arguments are scale-encoded known types in the host.
//! - Host functions will decode the input and emit an event `with` environmental.

#![cfg_attr(not(feature = "std"), no_std)]

// TEMPORARY STUB: The runtime_interface macro is not working properly with the new polkadot-sdk
// version This is a minimal stub to allow compilation to proceed

use evm_tracing_events::StepEventFilter;
use sp_std::vec::Vec;

pub mod ext {
	use super::*;

	pub fn raw_step(_data: Vec<u8>) {}
	pub fn raw_gas(_data: Vec<u8>) {}
	pub fn raw_return_value(_data: Vec<u8>) {}
	pub fn call_list_entry(_index: u32, _value: Vec<u8>) {}
	pub fn call_list_new() {}
	pub fn evm_event(_event: Vec<u8>) {}
	pub fn gasometer_event(_event: Vec<u8>) {}
	pub fn runtime_event(_event: Vec<u8>) {}
	pub fn step_event_filter() -> StepEventFilter {
		StepEventFilter::default()
	}
}
