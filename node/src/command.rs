// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use crate::{
	chainspec,
	cli::{Cli, Subcommand},
	service::{self, tangle},
};
use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};
use futures::TryFutureExt;
use sc_cli::SubstrateCli;
use sc_service::PartialComponents;
use tangle_primitives::types::Block;

#[allow(dead_code)]
trait IdentifyChain {
	fn is_mainnet(&self) -> bool;
	fn is_testnet(&self) -> bool;
}

impl IdentifyChain for dyn sc_service::ChainSpec {
	fn is_mainnet(&self) -> bool {
		!self.id().starts_with("testnet")
	}
	fn is_testnet(&self) -> bool {
		self.id().starts_with("testnet")
	}
}

impl<T: sc_service::ChainSpec + 'static> IdentifyChain for T {
	fn is_mainnet(&self) -> bool {
		<dyn sc_service::ChainSpec>::is_mainnet(self)
	}
	fn is_testnet(&self) -> bool {
		<dyn sc_service::ChainSpec>::is_testnet(self)
	}
}

impl SubstrateCli for Cli {
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
				&include_bytes!("../../chainspecs/testnet/tangle-testnet.json")[..],
			)?),
			// generates the spec for mainnet
			"mainnet-local" => Box::new(chainspec::mainnet::local_mainnet_config(
				tangle_primitives::MAINNET_CHAIN_ID,
			)?),
			"mainnet" => Box::new(chainspec::mainnet::tangle_mainnet_config(
				tangle_primitives::MAINNET_CHAIN_ID,
			)?),
			"tangle-mainnet" => Box::new(chainspec::mainnet::ChainSpec::from_json_bytes(
				&include_bytes!("../../chainspecs/mainnet/tangle-mainnet.json")[..],
			)?),

			path => Box::new(chainspec::mainnet::ChainSpec::from_json_file(
				std::path::PathBuf::from(path),
			)?),
		})
	}
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		#[cfg(not(feature = "manual-seal"))]
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, import_queue, .. } =
					service::new_partial(&config, &cli.eth)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		#[cfg(feature = "manual-seal")]
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager, _) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		#[cfg(not(feature = "manual-seal"))]
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, .. } =
					service::new_partial(&config, &cli.eth)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		},
		#[cfg(feature = "manual-seal")]
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager, _) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		},
		#[cfg(not(feature = "manual-seal"))]
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, .. } =
					service::new_partial(&config, &cli.eth)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		},
		#[cfg(feature = "manual-seal")]
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager, _) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		},
		#[cfg(not(feature = "manual-seal"))]
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, import_queue, .. } =
					service::new_partial(&config, &cli.eth)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		#[cfg(feature = "manual-seal")]
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager, _) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		},
		#[cfg(not(feature = "manual-seal"))]
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, backend, .. } =
					service::new_partial(&config, &cli.eth)?;
				let aux_revert = Box::new(|client, _, blocks| {
					sc_consensus_grandpa::revert(client, blocks)?;
					Ok(())
				});
				Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
			})
		},
		#[cfg(feature = "manual-seal")]
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, backend, import_queue, task_manager, _) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				let aux_revert = Box::new(|client, _, blocks| {
					sc_consensus_grandpa::revert(client, blocks)?;
					Ok(())
				});
				Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
			})
		},
		#[cfg(not(feature = "manual-seal"))]
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				// This switch needs to be in the client, since the client decides
				// which sub-commands it wants to support.
				match cmd {
					BenchmarkCmd::Pallet(cmd) => {
						if !cfg!(feature = "runtime-benchmarks") {
							return Err(
								"Runtime benchmarking wasn't enabled when building the node. \
							You can enable it with `--features runtime-benchmarks`."
									.into(),
							);
						}

						cmd.run_with_spec::<sp_runtime::traits::HashingFor<Block>,
						<tangle::ExecutorDispatch as sc_executor::NativeExecutionDispatch>::ExtendHostFunctions>(Some(
							config.chain_spec,
						))
					},
					BenchmarkCmd::Block(cmd) => {
						let PartialComponents { client, .. } =
							service::new_partial(&config, &cli.eth)?;
						cmd.run(client)
					},
					#[cfg(not(feature = "runtime-benchmarks"))]
					BenchmarkCmd::Storage(_) => Err(sc_cli::Error::Input(
						"Compile with --features=runtime-benchmarks \
						to enable storage benchmarks."
							.into(),
					)),
					#[cfg(feature = "runtime-benchmarks")]
					BenchmarkCmd::Storage(cmd) => {
						let PartialComponents { client, backend, .. } =
							service::new_partial(&config, &cli.eth)?;
						let db = backend.expose_db();
						let storage = backend.expose_storage();

						cmd.run(config, client, db, storage)
					},
					BenchmarkCmd::Overhead(_cmd) => Err("Unsupported benchmarking command".into()),
					BenchmarkCmd::Extrinsic(_cmd) => Err("Unsupported benchmarking command".into()),
					BenchmarkCmd::Machine(cmd) => {
						cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())
					},
				}
			})
		},
		#[cfg(feature = "manual-seal")]
		Some(Subcommand::Benchmark(cmd)) => {
			unimplemented!()
		},
		Some(Subcommand::FrontierDb(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|mut config| {
				let (client, _, _, _, frontier_backend) =
					service::new_chain_ops(&mut config, &cli.eth)?;
				let frontier_backend = match frontier_backend {
					fc_db::Backend::KeyValue(kv) => kv,
					_ => panic!("Only fc_db::Backend::KeyValue supported"),
				};
				cmd.run(client, frontier_backend)
			})
		},
		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				// we don't need any of the components of new_partial, just a runtime, or a task
				// manager to do `async_run`.
				let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
				let task_manager =
					sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
						.map_err(|e| sc_cli::Error::Service(sc_service::Error::Prometheus(e)))?;
				Ok((cmd.run::<Block, service::ExecutorDispatch>(config), task_manager))
			})
		},
		#[cfg(not(feature = "try-runtime"))]
		Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. \
				You can enable it with `--features try-runtime`."
			.into()),
		Some(Subcommand::ChainInfo(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run::<Block>(&config))
		},
		None => {
			let rpc_config = crate::eth::RpcConfig {
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
			let runner = cli.create_runner(&cli.run)?;

			runner.run_node_until_exit(|config| async move {
				service::new_full::<
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
				.map_err(Into::into)
				.await
			})
		},
	}
}
