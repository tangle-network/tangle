//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod chainspec;
#[macro_use]
#[cfg(not(feature = "manual-seal"))]
mod service;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod cli;
mod command;
mod distributions;
mod eth;
mod mainnet_fixtures;
mod rpc;
mod testnet_fixtures;
mod utils;

// manual seal build
#[cfg(feature = "manual-seal")]
mod manual_seal;
#[cfg(feature = "manual-seal")]
use manual_seal as service;

fn main() -> sc_cli::Result<()> {
	command::run()
}
