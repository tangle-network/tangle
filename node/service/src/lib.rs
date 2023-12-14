mod chainspec;
mod client;
mod distributions;
mod eth;
mod fixtures;
mod rpc;

use crate::eth::{
	new_frontier_partial, spawn_frontier_tasks, BackendType, EthApi, EthConfiguration,
	FrontierPartialComponents,
};
use client::{Client, RuntimeApiCollection};
use fc_consensus::FrontierBlockImport as TFrontierBlockImport;
use fc_db::DatabaseSource;
use futures::{channel::mpsc, FutureExt};
use parity_scale_codec::Encode;
use sc_chain_spec::ChainSpec;
use sc_client_api::{AuxStore, Backend, BlockBackend, StateBackend, StorageProvider};
use sc_consensus::BasicQueue;
use sc_consensus_aura::ImportQueueParams;
use sc_consensus_grandpa::SharedVoterState;
pub use sc_executor::NativeElseWasmExecutor;
use sc_executor::NativeExecutionDispatch;
use sc_service::{
	error::Error as ServiceError, ChainType, Configuration, PartialComponents, TFullBackend,
	TFullClient, TaskManager,
};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_api::{ConstructRuntimeApi, ProvideRuntimeApi};
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
use sp_core::{Pair, U256};
use sp_runtime::{
	generic::{self, Era},
	OpaqueExtrinsic, SaturatedConversion,
};
use tangle_primitives::{Block, BlockNumber};
use tokio::runtime::Runtime;

use std::{path::Path, sync::Arc, time::Duration};
use substrate_frame_rpc_system::AccountNonceApi;

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

pub type HostFunctions = (frame_benchmarking::benchmarking::HostFunctions, ());

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

use sp_runtime::{traits::BlakeTwo256, DigestItem, Percent};

pub const SOFT_DEADLINE_PERCENT: Percent = Percent::from_percent(100);

/// Builds a new object suitable for chain operations.
#[allow(clippy::type_complexity)]
pub fn new_chain_ops(
	config: &mut Configuration,
	eth_config: &EthConfiguration,
) -> Result<
	(Arc<Client>, Arc<FullBackend>, sc_consensus::BasicQueue<Block>, TaskManager),
	ServiceError,
> {
	match &config.chain_spec {
		#[cfg(feature = "testnet")]
		spec if spec.is_testnet() => new_chain_ops_inner::<
			tangle_testnet_runtime::RuntimeApi,
			TestnetExecutor,
		>(config, eth_config),
		#[cfg(feature = "tangle")]
		spec if spec.is_tangle() => new_chain_ops_inner::<
			tangle_mainnet_runtime::RuntimeApi,
			TangleExecutor,
		>(config, eth_config),
		_ => panic!("invalid chain spec"),
	}
}

#[allow(clippy::type_complexity)]
fn new_chain_ops_inner<RuntimeApi, Executor>(
	config: &mut Configuration,
	eth_config: &EthConfiguration,
) -> Result<
	(Arc<Client>, Arc<FullBackend>, sc_consensus::BasicQueue<Block>, TaskManager),
	ServiceError,
>
where
	Client: From<Arc<crate::FullClient<RuntimeApi, Executor>>>,
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	Executor: sc_executor::NativeExecutionDispatch + 'static,
{
	config.keystore = sc_service::config::KeystoreConfig::InMemory;
	let PartialComponents { client, backend, import_queue, task_manager, .. } =
		new_partial::<RuntimeApi, Executor>(config, eth_config)?;
	Ok((Arc::new(Client::from(client)), backend, import_queue, task_manager))
}

pub fn new_partial<RuntimeApi, Executor>(
	config: &Configuration,
	eth_config: &EthConfiguration,
) -> PartialComponentsResult<RuntimeApi, Executor>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	Executor: sc_executor::NativeExecutionDispatch + 'static,
{
	println!("    ++++++++++++++++++++++++                                                                          
   +++++++++++++++++++++++++++                                                                        
   +++++++++++++++++++++++++++                                                                        
   +++        ++++++      +++         @%%%%%%%%%%%                                     %%%
   ++++++      ++++      +++++        %%%%%%%%%%%%                                     %%%@
   ++++++++++++++++++++++++++            %%%%      %%%%@     %%% %%@       @%%%%%%%   %%%@    %%%%@
	  ++++++++                       %%%%    @%%%%%%%@   %%%%%%%%%   @%%%%%%%%%   %%%@  %%%%%%%%%
	  ++++++++                       %%%%    %%%%%%%%%   %%%% @%%%@  %%%%  %%%%   %%%@  %%%%%%%%%%
   ++++++++++++++++++++++++++            %%%%    %%%%%%%%%   %%%   %%%%  %%%   @%%%   %%%@ @%%%%%  %%%%%
   ++++++      ++++      ++++++          %%%%    %%%%%%%%%   %%%   %%%%  %%%%%%%%%%   %%%@  %%%%%%%%%@
   +++        ++++++        +++          %%%%    %%%%%%%%%   %%%   %%%@   %%%%%%%%%   %%%    %%%%%%%@
   ++++      +++++++++      +++                                           %%%%  %%%%               
   ++++++++++++++++++++++++++++                                           %%%%%%%%%         
     +++++++++++++++++++++++                                                 %%%%% \n");
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let executor = sc_service::new_native_or_wasm_executor(config);

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let client = Arc::new(client);

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	let (grandpa_block_import, grandpa_link) = sc_consensus_grandpa::block_import(
		client.clone(),
		GRANDPA_JUSTIFICATION_PERIOD,
		&client,
		select_chain.clone(),
		telemetry.as_ref().map(|x| x.handle()),
	)?;

	let overrides = crate::rpc::overrides_handle(client.clone());
	let frontier_backend = open_frontier_backend(client.clone(), config, &eth_config)?;

	let frontier_block_import = FrontierBlockImport::new(client.clone(), client.clone());

	let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
	let target_gas_price = eth_config.target_gas_price;
	let create_inherent_data_providers = move |_, ()| async move {
		let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
		let slot =
			sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
				*timestamp,
				slot_duration,
			);
		let dynamic_fee = fp_dynamic_fee::InherentDataProvider(U256::from(target_gas_price));
		Ok((slot, timestamp, dynamic_fee))
	};

	let import_queue =
		sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _>(ImportQueueParams {
			block_import: frontier_block_import.clone(),
			justification_import: Some(Box::new(grandpa_block_import.clone())),
			client: client.clone(),
			create_inherent_data_providers,
			spawner: &task_manager.spawn_essential_handle(),
			registry: config.prometheus_registry(),
			check_for_equivocation: Default::default(),
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			compatibility_mode: Default::default(),
		})?;

	Ok(sc_service::PartialComponents {
		client,
		backend,
		keystore_container,
		task_manager,
		select_chain,
		import_queue,
		transaction_pool,
		other: (
			telemetry,
			Box::new(frontier_block_import),
			grandpa_link,
			frontier_backend,
			overrides,
		),
	})
}
