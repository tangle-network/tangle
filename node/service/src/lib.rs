use client::{Client, RuntimeApiCollection};
use eth::{
	new_frontier_partial, spawn_frontier_tasks, BackendType, EthApi, EthConfiguration,
	FrontierPartialComponents, RpcConfig,
};
use fc_consensus::FrontierBlockImport as TFrontierBlockImport;
use fc_db::DatabaseSource;
use futures::{channel::mpsc, FutureExt};
use sc_chain_spec::ChainSpec;
use sc_client_api::{AuxStore, Backend, BlockBackend, StateBackend, StorageProvider};
use sc_consensus_grandpa::SharedVoterState;
pub use sc_executor::NativeElseWasmExecutor;
use sc_service::{
	error::Error as ServiceError, ChainType, Configuration, PartialComponents, TFullBackend,
	TFullClient, TaskManager,
};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_api::{ConstructRuntimeApi, ProvideRuntimeApi};
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_core::U256;
use tangle_primitives::Block;

use sc_consensus_aura::ImportQueueParams;
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;

use std::{path::Path, sync::Arc, time::Duration};

use sp_runtime::{traits::BlakeTwo256, Percent};

pub mod chainspec;
pub mod client;
pub mod distributions;
pub mod eth;
pub mod fixtures;
pub mod rpc;
pub mod utils;

#[cfg(feature = "testnet")]
pub mod aura;

#[cfg(feature = "tangle")]
pub mod babe;

/// The minimum period of blocks on which justifications will be
/// imported and generated.
const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

type FullClient<RuntimeApi, Executor> =
	TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>;
type FullBackend = TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
type FrontierBlockImport<RuntimeApi, Executor> = TFrontierBlockImport<
	Block,
	Arc<FullClient<RuntimeApi, Executor>>,
	FullClient<RuntimeApi, Executor>,
>;
type GrandpaLinkHalf<Client> = sc_consensus_grandpa::LinkHalf<Block, Client, FullSelectChain>;
type BoxBlockImport = sc_consensus::BoxBlockImport<Block>;
type FrontierBackend = fc_db::Backend<Block>;
type PartialComponentsResult<RuntimeApi, Executor> = Result<
	PartialComponents<
		FullClient<RuntimeApi, Executor>,
		FullBackend,
		FullSelectChain,
		sc_consensus::DefaultImportQueue<Block>,
		sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
		(
			Option<Telemetry>,
			BoxBlockImport,
			GrandpaLinkHalf<FullClient<RuntimeApi, Executor>>,
			FrontierBackend,
			Arc<fc_rpc::OverrideHandle<Block>>,
		),
	>,
	ServiceError,
>;

/// Host functions required for kitchensink runtime and Substrate node.
#[cfg(not(feature = "runtime-benchmarks"))]
pub type HostFunctions =
	(sp_io::SubstrateHostFunctions, sp_statement_store::runtime_api::HostFunctions);

/// Host functions required for kitchensink runtime and Substrate node.
#[cfg(feature = "runtime-benchmarks")]
pub type HostFunctions = (
	sp_io::SubstrateHostFunctions,
	sp_statement_store::runtime_api::HostFunctions,
	frame_benchmarking::benchmarking::HostFunctions,
);

#[cfg(feature = "tangle")]
pub struct TangleExecutor;

#[cfg(feature = "tangle")]
impl sc_executor::NativeExecutionDispatch for TangleExecutor {
	type ExtendHostFunctions = HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		tangle_mainnet_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		tangle_mainnet_runtime::native_version()
	}
}

#[cfg(feature = "testnet")]
pub struct TestnetExecutor;

#[cfg(feature = "testnet")]
impl sc_executor::NativeExecutionDispatch for TestnetExecutor {
	type ExtendHostFunctions = HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		tangle_testnet_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		tangle_testnet_runtime::native_version()
	}
}

/// Trivial enum representing runtime variant
#[derive(Clone)]
pub enum RuntimeVariant {
	#[cfg(feature = "tangle")]
	Tangle,
	#[cfg(feature = "testnet")]
	Testnet,
	Unrecognized,
}

impl RuntimeVariant {
	pub fn from_chain_spec(chain_spec: &Box<dyn ChainSpec>) -> Self {
		match chain_spec {
			#[cfg(feature = "tangle")]
			spec if spec.is_tangle() => Self::Tangle,
			#[cfg(feature = "testnet")]
			spec if spec.is_testnet() => Self::Testnet,
			_ => Self::Unrecognized,
		}
	}
}

/// Can be called for a `Configuration` to check if it is a configuration for
/// the `Testnet` network.
pub trait IdentifyVariant {
	/// Returns `true` if this is a configuration for the `Tangle` network.
	fn is_tangle(&self) -> bool;

	/// Returns `true` if this is a configuration for the `Testnet` network.
	fn is_testnet(&self) -> bool;
}

impl IdentifyVariant for Box<dyn ChainSpec> {
	fn is_tangle(&self) -> bool {
		self.id().starts_with("tangle")
	}

	fn is_testnet(&self) -> bool {
		self.id().starts_with("testnet")
	}
}

pub fn frontier_database_dir(config: &Configuration, path: &str) -> std::path::PathBuf {
	config.base_path.config_dir(config.chain_spec.id()).join("frontier").join(path)
}

// TODO This is copied from frontier. It should be imported instead after
// https://github.com/paritytech/frontier/issues/333 is solved
pub fn open_frontier_backend<C, BE>(
	client: Arc<C>,
	config: &Configuration,
	eth_config: &EthConfiguration,
) -> Result<fc_db::Backend<Block>, String>
where
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
	C: Send + Sync + 'static,
	C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
{
	let frontier_backend = match eth_config.frontier_backend_type {
		BackendType::KeyValue => fc_db::Backend::KeyValue(fc_db::kv::Backend::<Block>::new(
			client,
			&fc_db::kv::DatabaseSettings {
				source: match config.database {
					DatabaseSource::RocksDb { .. } => DatabaseSource::RocksDb {
						path: frontier_database_dir(config, "db"),
						cache_size: 0,
					},
					DatabaseSource::ParityDb { .. } =>
						DatabaseSource::ParityDb { path: frontier_database_dir(config, "paritydb") },
					DatabaseSource::Auto { .. } => DatabaseSource::Auto {
						rocksdb_path: frontier_database_dir(config, "db"),
						paritydb_path: frontier_database_dir(config, "paritydb"),
						cache_size: 0,
					},
					_ =>
						return Err(
							"Supported db sources: `rocksdb` | `paritydb` | `auto`".to_string()
						),
				},
			},
		)?),
		BackendType::Sql => {
			let overrides = crate::rpc::overrides_handle(client.clone());
			let sqlite_db_path = frontier_database_dir(config, "sql");
			std::fs::create_dir_all(&sqlite_db_path).expect("failed creating sql db directory");
			let backend = futures::executor::block_on(fc_db::sql::Backend::new(
				fc_db::sql::BackendConfig::Sqlite(fc_db::sql::SqliteBackendConfig {
					path: Path::new("sqlite:///")
						.join(sqlite_db_path)
						.join("frontier.db3")
						.to_str()
						.expect("frontier sql path error"),
					create_if_missing: true,
					thread_count: eth_config.frontier_sql_backend_thread_count,
					cache_size: eth_config.frontier_sql_backend_cache_size,
				}),
				eth_config.frontier_sql_backend_pool_size,
				std::num::NonZeroU32::new(eth_config.frontier_sql_backend_num_ops_timeout),
				overrides.clone(),
			))
			.unwrap_or_else(|err| panic!("failed creating sql backend: {:?}", err));
			fc_db::Backend::Sql(backend)
		},
	};

	Ok(frontier_backend)
}
