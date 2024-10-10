//! Benchmarks for the nomination pools coupled with the staking and bags list pallets.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

extern crate alloc;

#[cfg(feature = "runtime-benchmarks")]
pub mod inner;

#[cfg(feature = "runtime-benchmarks")]
pub use inner::*;

#[cfg(all(feature = "runtime-benchmarks", test))]
pub(crate) mod mock;
