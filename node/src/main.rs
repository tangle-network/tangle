//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod benchmarking;
mod cli;
mod command;

fn main() -> sc_cli::Result<()> {
	command::run()
}
