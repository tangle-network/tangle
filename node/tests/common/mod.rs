//! Common utilities for end-to-end tests.
#![cfg(feature = "e2e")]

use core::future::Future;

use sc_cli::{CliConfiguration, SubstrateCli};
use tangle::{chainspec, cli, eth, service};
use tangle_primitives::types::Block;
use tangle_subxt::{subxt, subxt_signer};

pub struct CliWrapper(pub cli::Cli);

impl clap::CommandFactory for CliWrapper {
	fn command() -> clap::Command {
		<cli::Cli as clap::CommandFactory>::command()
	}

	fn command_for_update() -> clap::Command {
		<cli::Cli as clap::CommandFactory>::command_for_update()
	}
}

impl clap::FromArgMatches for CliWrapper {
	fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
		<cli::Cli as clap::FromArgMatches>::from_arg_matches(matches).map(CliWrapper)
	}

	fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
		<cli::Cli as clap::FromArgMatches>::update_from_arg_matches(&mut self.0, matches)
	}
}

impl SubstrateCli for CliWrapper {
	fn impl_name() -> String {
		"Tangle Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/webb-tools/tangle/issues".into()
	}

	fn copyright_start_year() -> i32 {
		2023
	}

	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(match id {
			"" | "dev" | "local" => Box::new(chainspec::testnet::local_testnet_config(
				tangle_primitives::TESTNET_LOCAL_CHAIN_ID,
			)?),
			// generates the spec for benchmarking.
			"benchmark" => Box::new(chainspec::testnet::local_benchmarking_config(
				tangle_primitives::TESTNET_CHAIN_ID,
			)?),
			// generates the spec for testnet
			"testnet" => Box::new(chainspec::testnet::tangle_testnet_config(
				tangle_primitives::TESTNET_CHAIN_ID,
			)?),
			"tangle-testnet" => Box::new(chainspec::testnet::ChainSpec::from_json_bytes(
				&include_bytes!("../../../chainspecs/testnet/tangle-testnet.json")[..],
			)?),
			// generates the spec for mainnet
			"mainnet-local" => Box::new(chainspec::mainnet::local_mainnet_config(
				tangle_primitives::MAINNET_CHAIN_ID,
			)?),
			"mainnet" => Box::new(chainspec::mainnet::tangle_mainnet_config(
				tangle_primitives::MAINNET_CHAIN_ID,
			)?),
			"tangle-mainnet" => Box::new(chainspec::mainnet::ChainSpec::from_json_bytes(
				&include_bytes!("../../../chainspecs/mainnet/tangle-mainnet.json")[..],
			)?),

			path => Box::new(chainspec::mainnet::ChainSpec::from_json_file(
				std::path::PathBuf::from(path),
			)?),
		})
	}
}

impl clap::Parser for CliWrapper {}

#[derive(Debug, Clone, Copy)]
pub enum TestAccount {
	Alice,
	Bob,
	Charlie,
	Dave,
	Eve,
	Ferdie,
}

impl TestAccount {
	pub fn address(&self) -> alloy::primitives::Address {
		self.evm_signer().address()
	}

	pub fn account_id(&self) -> subxt::utils::AccountId32 {
		use subxt::tx::Signer;
		use subxt::PolkadotConfig;

		let signer = self.substrate_signer();
		Signer::<PolkadotConfig>::account_id(&signer)
	}

	pub fn evm_signer(&self) -> alloy::signers::local::PrivateKeySigner {
		let private_key = match self {
			Self::Alice => subxt_signer::ecdsa::dev::alice().0.secret_bytes(),
			Self::Bob => subxt_signer::ecdsa::dev::bob().0.secret_bytes(),
			Self::Charlie => subxt_signer::ecdsa::dev::charlie().0.secret_bytes(),
			Self::Dave => subxt_signer::ecdsa::dev::dave().0.secret_bytes(),
			Self::Eve => subxt_signer::ecdsa::dev::eve().0.secret_bytes(),
			Self::Ferdie => subxt_signer::ecdsa::dev::ferdie().0.secret_bytes(),
		};
		alloy::signers::local::PrivateKeySigner::from_bytes((&private_key).into()).unwrap()
	}

	pub fn evm_wallet(&self) -> alloy::network::EthereumWallet {
		alloy::network::EthereumWallet::from(self.evm_signer())
	}

	pub fn substrate_signer(&self) -> subxt_signer::sr25519::Keypair {
		match self {
			Self::Alice => subxt_signer::sr25519::dev::alice(),
			Self::Bob => subxt_signer::sr25519::dev::bob(),
			Self::Charlie => subxt_signer::sr25519::dev::charlie(),
			Self::Dave => subxt_signer::sr25519::dev::dave(),
			Self::Eve => subxt_signer::sr25519::dev::eve(),
			Self::Ferdie => subxt_signer::sr25519::dev::ferdie(),
		}
	}
}

pub async fn alloy_provider() -> impl alloy::providers::Provider + Clone {
	alloy::providers::ProviderBuilder::new()
		.with_recommended_fillers()
		.on_builtin("http://127.0.0.1:9944")
		.await
		.unwrap()
}

pub async fn subxt_client() -> subxt::OnlineClient<subxt::PolkadotConfig> {
	subxt::OnlineClient::new().await.unwrap()
}

pub trait AddressConverter {
	fn to_account_id(&self) -> subxt::utils::AccountId32;
}

impl AddressConverter for alloy::primitives::Address {
	fn to_account_id(&self) -> subxt::utils::AccountId32 {
		let mut data = [0u8; 24];
		data[0..4].copy_from_slice(b"evm:");
		data[4..24].copy_from_slice(&self[..]);
		let hash = sp_core::blake2_256(&data);

		subxt::utils::AccountId32(hash)
	}
}

/// Run an end-to-end test with the given future.
#[track_caller]
pub fn run_e2e_test<F>(f: F)
where
	F: Future + Send + 'static,
	F::Output: Send + Sync + 'static,
{
	let wrapper = CliWrapper::from_iter([
		"--tmp",
		"--dev",
		"--validator",
		"--alice",
		"--rpc-cors=all",
		"--rpc-methods=unsafe",
		"--rpc-external",
		"--rpc-port=9944",
		"--sealing=manual",
		"--auto-insert-keys",
		"-linfo",
		"-levm=debug",
		"-lgadget=trace",
	]);

	let runner = {
		let this = &wrapper;
		let command = &wrapper.0.run;
		{
			let tokio_runtime =
				tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

			// `capture` needs to be called in a tokio context.
			// Also capture them as early as possible.
			let signals = tokio_runtime.block_on(async { sc_cli::Signals::capture() }).unwrap();

			let config =
				command.create_configuration(this, tokio_runtime.handle().clone()).unwrap();

			command
				.init(&CliWrapper::support_url(), &CliWrapper::impl_version(), |_, _| {}, &config)
				.unwrap();
			sc_cli::Runner::<CliWrapper>::new(config, tokio_runtime, signals)
		}
	}
	.unwrap();
	let cli = wrapper.0;

	let rpc_config = eth::RpcConfig {
		ethapi: cli.eth.ethapi.clone(),
		ethapi_max_permits: cli.eth.ethapi_max_permits,
		ethapi_trace_max_count: cli.eth.ethapi_trace_max_count,
		ethapi_trace_cache_duration: cli.eth.ethapi_trace_cache_duration,
		eth_log_block_cache: cli.eth.eth_log_block_cache,
		eth_statuses_cache: cli.eth.eth_statuses_cache,
		fee_history_limit: cli.eth.fee_history_limit,
		max_past_logs: cli.eth.max_past_logs,
		tracing_raw_max_memory_usage: cli.eth.tracing_raw_max_memory_usage,
	};

	runner
		.async_run(|config| {
			let tokio_handle = config.tokio_handle.clone();
			tokio_handle.block_on(async move {
				let task_manager = service::new_full::<
					sc_network::NetworkWorker<Block, <Block as sp_runtime::traits::Block>::Hash>,
				>(service::RunFullParams {
					config,
					rpc_config,
					eth_config: cli.eth,
					debug_output: cli.output_path,
					auto_insert_keys: cli.auto_insert_keys,
					#[cfg(feature = "manual-seal")]
					sealing: cli.sealing,
				})
				.await
				.unwrap();

				async fn test_fn<F: Future>(f: F) -> Result<(), sc_cli::Error> {
					// wait till the node is online by checking for port 9944
					let mut port_open = false;
					let addr = ("127.0.0.1", 9944u16);
					while !port_open {
						port_open = tokio::net::TcpStream::connect(addr).await.is_ok();
						tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
					}
					f.await;
					Ok(())
				}

				Ok((test_fn(f), task_manager))
			})
		})
		.unwrap();
}
