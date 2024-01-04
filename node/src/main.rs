//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod chainspec;
#[macro_use]
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

fn main() -> sc_cli::Result<()> {
	command::run()
}
