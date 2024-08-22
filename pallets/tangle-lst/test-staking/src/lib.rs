//! Contains benchmarks and tests that use `pallet-staking`

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
#[cfg(any(test, feature = "test-utils"))]
pub mod mock;
#[cfg(test)]
mod tests;
