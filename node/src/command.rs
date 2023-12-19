// Copyright 2022 Webb Technologies Inc.
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
	benchmarking::{inherent_benchmark_data, RemarkBuilder, TransferKeepAliveBuilder},
	cli::{Cli, Subcommand},
};
use fc_db::DatabaseSource;
use frame_benchmarking_cli::{BenchmarkCmd, ExtrinsicFactory};
use futures::TryFutureExt;
use sc_cli::SubstrateCli;
use sc_service::PartialComponents;
use sp_keyring::Sr25519Keyring;
use std::sync::Arc;
use tangle_service::{chainspec, frontier_database_dir};

#[cfg(feature = "tangle")]
use tangle_mainnet_runtime::EXISTENTIAL_DEPOSIT;
#[cfg(feature = "testnet")]
use tangle_testnet_runtime::EXISTENTIAL_DEPOSIT;

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
			"" | "local" => Box::new(chainspec::testnet::local_testnet_config(4006)?),
			// generates the spec for testnet
			"testnet" => Box::new(chainspec::testnet::tangle_testnet_config(4006)?),
			// generates the spec for mainnet
			"dev" | "mainnet-local" => Box::new(chainspec::mainnet::local_testnet_config(4006)?),
			"mainnet" => Box::new(chainspec::mainnet::tangle_mainnet_config(4006)?),
			"tangle-testnet" => Box::new(chainspec::testnet::ChainSpec::from_json_bytes(
				&include_bytes!("../../chainspecs/testnet/tangle-standalone.json")[..],
			)?),
			path => Box::new(chainspec::testnet::ChainSpec::from_json_file(
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
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				#[cfg(feature = "tangle")]
				let (client, _, import_queue, task_manager) =
					tangle_service::babe::new_chain_ops(&mut config, &cli.eth)?;
				#[cfg(feature = "testnet")]
				let (client, _, import_queue, task_manager) =
					tangle_service::babe::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				#[cfg(feature = "tangle")]
				let (client, _, _, task_manager) = tangle_service::babe::new_chain_ops(&mut config, &cli.eth)?;
				#[cfg(feature = "testnet")]
				let (client, _, _, task_manager) = tangle_service::babe::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		},
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				#[cfg(feature = "tangle")]
				let (client, _, _, task_manager) = tangle_service::babe::new_chain_ops(&mut config, &cli.eth)?;
				#[cfg(feature = "testnet")]
				let (client, _, _, task_manager) = tangle_service::babe::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				#[cfg(feature = "tangle")]
				let (client, _, import_queue, task_manager) =
					tangle_service::babe::new_chain_ops(&mut config, &cli.eth)?;
				#[cfg(feature = "testnet")]
				let (client, _, import_queue, task_manager) =
					tangle_service::babe::new_chain_ops(&mut config, &cli.eth)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| {
				// Remove Frontier offchain db
				let frontier_database_config = match config.database {
					DatabaseSource::RocksDb { .. } => DatabaseSource::RocksDb {
						path: frontier_database_dir(&config, "db"),
						cache_size: 0,
					},
					DatabaseSource::ParityDb { .. } => DatabaseSource::ParityDb {
						path: frontier_database_dir(&config, "paritydb"),
					},
					_ =>
						return Err(format!("Cannot purge `{:?}` database", config.database).into()),
				};
				cmd.run(frontier_database_config)?;

				// Remove Tangle db
				cmd.run(config.database)
			})
		},
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;
			match chain_spec {
				#[cfg(feature = "tangle")]
				spec if spec.is_mainnet() => runner.async_run(|mut config| {
					let params = tangle_service::babe::new_partial::<
						tangle_mainnet_runtime::RuntimeApi,
						tangle_service::TangleExecutor,
					>(&mut config, &cli.eth)?;

					Ok((cmd.run(params.client, params.backend, None), params.task_manager))
				}),
				#[cfg(feature = "testnet")]
				spec if spec.is_testnet() => runner.async_run(|mut config| {
					let params = tangle_service::babe::new_partial::<
						tangle_testnet_runtime::RuntimeApi,
						tangle_service::TestnetExecutor,
					>(&mut config, &cli.eth)?;

					Ok((cmd.run(params.client, params.backend, None), params.task_manager))
				}),
				_ => panic!("invalid chain spec"),
			}
		},
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			// This switch needs to be in the client, since the client decides
			// which sub-commands it wants to support.
			match cmd {
				BenchmarkCmd::Pallet(cmd) =>
					if cfg!(feature = "runtime-benchmarks") {
						let chain_spec = &runner.config().chain_spec;
						match chain_spec {
							#[cfg(feature = "tangle")]
							spec if spec.is_mainnet() =>
								return runner.sync_run(|config| {
									cmd.run::<tangle_mainnet_runtime::Block, tangle_service::HostFunctions>(
										config,
									)
								}),
							#[cfg(feature = "testnet")]
							spec if spec.is_testnet() =>
								return runner.sync_run(|config| {
									cmd.run::<tangle_testnet_runtime::Block, tangle_service::HostFunctions>(
										config,
									)
								}),
							_ => panic!("invalid chain spec"),
						}
					} else {
						Err("Benchmarking wasn't enabled when building the node. \
					You can enable it with `--features runtime-benchmarks`."
							.into())
					},
				BenchmarkCmd::Block(cmd) => {
					let chain_spec = &runner.config().chain_spec;
					match chain_spec {
						#[cfg(feature = "tangle")]
						spec if spec.is_mainnet() =>
							return runner.sync_run(|mut config| {
								let params = tangle_service::babe::new_partial::<
									tangle_mainnet_runtime::RuntimeApi,
									tangle_service::TangleExecutor,
								>(&mut config, &cli.eth)?;

								cmd.run(params.client)
							}),
						#[cfg(feature = "testnet")]
						spec if spec.is_testnet() =>
							return runner.sync_run(|mut config| {
								let params = tangle_service::babe::new_partial::<
									tangle_testnet_runtime::RuntimeApi,
									tangle_service::TestnetExecutor,
								>(&mut config, &cli.eth)?;

								cmd.run(params.client)
							}),
						_ => panic!("invalid chain spec"),
					}
				},
				#[cfg(not(feature = "runtime-benchmarks"))]
				BenchmarkCmd::Storage(_) => Err(sc_cli::Error::Input(
					"Compile with --features=runtime-benchmarks \
					to enable storage benchmarks."
						.into(),
				)),
				#[cfg(feature = "runtime-benchmarks")]
				BenchmarkCmd::Storage(cmd) => {
					let chain_spec = &runner.config().chain_spec;
					match chain_spec {
						#[cfg(feature = "tangle")]
						spec if spec.is_mainnet() =>
							return runner.sync_run(|mut config| {
								let params = new_partial::<
									tangle_mainnet_runtime::RuntimeApi,
									tangle_service::TangleExecutor,
								>(&mut config, &cli.eth)?;

								let db = params.backend.expose_db();
								let storage = params.backend.expose_storage();

								cmd.run(config, params.client, db, storage)
							}),
						#[cfg(feature = "testnet")]
						spec if spec.is_testnet() =>
							return runner.sync_run(|mut config| {
								let params = new_partial::<
									tangle_testnet_runtime::RuntimeApi,
									tangle_service::TestnetExecutor,
								>(&mut config, &cli.eth)?;

								let db = params.backend.expose_db();
								let storage = params.backend.expose_storage();

								cmd.run(config, params.client, db, storage)
							}),
						_ => panic!("invalid chain spec"),
					}
				},
				BenchmarkCmd::Overhead(cmd) => {
					let chain_spec = &runner.config().chain_spec;
					match chain_spec {
						#[cfg(feature = "tangle")]
						spec if spec.is_mainnet() =>
							return runner.sync_run(|mut config| {
								let PartialComponents { client, .. } =
									tangle_service::babe::new_partial::<
										tangle_mainnet_runtime::RuntimeApi,
										tangle_service::TangleExecutor,
									>(&mut config, &cli.eth)?;

								let c = Arc::new(tangle_service::client::Client::Tangle(
									client.clone(),
								));
								let ext_builder = RemarkBuilder::new(c);
								cmd.run(
									config,
									client,
									inherent_benchmark_data()?,
									Vec::new(),
									&ext_builder,
								)
							}),
						#[cfg(feature = "testnet")]
						spec if spec.is_testnet() =>
							return runner.sync_run(|mut config| {
								let PartialComponents { client, .. } =
									tangle_service::babe::new_partial::<
										tangle_testnet_runtime::RuntimeApi,
										tangle_service::TestnetExecutor,
									>(&mut config, &cli.eth)?;

								let c = Arc::new(tangle_service::client::Client::Testnet(
									client.clone(),
								));
								let ext_builder = RemarkBuilder::new(c);
								cmd.run(
									config,
									client,
									inherent_benchmark_data()?,
									Vec::new(),
									&ext_builder,
								)
							}),
						_ => panic!("invalid chain spec"),
					}
				},
				BenchmarkCmd::Extrinsic(cmd) => {
					let chain_spec = &runner.config().chain_spec;
					match chain_spec {
						#[cfg(feature = "tangle")]
						spec if spec.is_mainnet() => {
							return runner.sync_run(|mut config| {
								let PartialComponents { client, .. } =
									tangle_service::babe::new_partial::<
										tangle_mainnet_runtime::RuntimeApi,
										tangle_service::TangleExecutor,
									>(&mut config, &cli.eth)?;
								// Create a client.
								let c = Arc::new(tangle_service::client::Client::Tangle(
									client.clone(),
								));
								// Register the *Remark* and *TKA* builders.
								let ext_factory = ExtrinsicFactory(vec![
									Box::new(RemarkBuilder::new(c.clone())),
									Box::new(TransferKeepAliveBuilder::new(
										c,
										Sr25519Keyring::Alice.to_account_id(),
										EXISTENTIAL_DEPOSIT,
									)),
								]);

								cmd.run(
									client,
									inherent_benchmark_data()?,
									Vec::new(),
									&ext_factory,
								)
							})
						},
						#[cfg(feature = "testnet")]
						spec if spec.is_testnet() => {
							return runner.sync_run(|mut config| {
								let PartialComponents { client, .. } =
									tangle_service::babe::new_partial::<
										tangle_testnet_runtime::RuntimeApi,
										tangle_service::TestnetExecutor,
									>(&mut config, &cli.eth)?;
								// Create a client.
								let c = Arc::new(tangle_service::client::Client::Testnet(
									client.clone(),
								));
								// Register the *Remark* and *TKA* builders.
								let ext_factory = ExtrinsicFactory(vec![
									Box::new(RemarkBuilder::new(c.clone())),
									Box::new(TransferKeepAliveBuilder::new(
										c,
										Sr25519Keyring::Alice.to_account_id(),
										EXISTENTIAL_DEPOSIT,
									)),
								]);

								cmd.run(
									client,
									inherent_benchmark_data()?,
									Vec::new(),
									&ext_factory,
								)
							})
						},
						_ => panic!("invalid chain spec"),
					}
				},
				BenchmarkCmd::Machine(cmd) =>
					return runner.sync_run(|config| {
						cmd.run(
							&config,
							frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE.clone(),
						)
					}),
			}
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
				#[cfg(feature = "tangle")]
				Ok((cmd.run::<Block, tangle_service::ExecutorDispatch>(config), task_manager))
			})
		},
		#[cfg(not(feature = "try-runtime"))]
		Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. \
				You can enable it with `--features try-runtime`."
			.into()),
		None => {
			let rpc_config = tangle_service::eth::RpcConfig {
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
			let chain_spec = &runner.config().chain_spec;
			match chain_spec {
				#[cfg(feature = "tangle")]
				spec if spec.is_mainnet() =>
					return runner.run_node_until_exit(|config| async move {
						tangle_service::babe::new_full::<
							tangle_mainnet_runtime::RuntimeApi,
							tangle_service::TangleExecutor,
						>(tangle_service::RunFullParams {
							config,
							rpc_config,
							eth_config: cli.eth,
							debug_output: cli.output_path,
							auto_insert_keys: cli.auto_insert_keys,
							disable_hardware_benchmarks: cli.no_hardware_benchmarks,
						})
						.map_err(Into::into)
						.await
					}),
				#[cfg(feature = "testnet")]
				spec if spec.is_testnet() =>
					return runner.run_node_until_exit(|config| async move {
						tangle_service::babe::new_full::<
							tangle_testnet_runtime::RuntimeApi,
							tangle_service::TestnetExecutor,
						>(tangle_service::RunFullParams {
							config,
							rpc_config,
							eth_config: cli.eth,
							debug_output: cli.output_path,
							auto_insert_keys: cli.auto_insert_keys,
							disable_hardware_benchmarks: cli.no_hardware_benchmarks,
						})
						.map_err(Into::into)
						.await
					}),
				_ => panic!("invalid chain spec"),
			}
		},
	}
}
