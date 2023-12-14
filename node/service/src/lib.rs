mod chainspec;
mod client;
mod distributions;
mod eth;
mod fixtures;
mod rpc;
mod utils;

use crate::eth::{
	new_frontier_partial, spawn_frontier_tasks, BackendType, EthApi, EthConfiguration,
	FrontierPartialComponents,
};
use client::{Client, RuntimeApiCollection};
use eth::RpcConfig;
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

pub struct RunFullParams {
	pub config: Configuration,
	pub eth_config: EthConfiguration,
	pub rpc_config: RpcConfig,
	pub debug_output: Option<std::path::PathBuf>,
	#[cfg(feature = "relayer")]
	pub relayer_cmd: tangle_relayer_gadget_cli::RelayerCmd,
	#[cfg(feature = "light-client")]
	pub light_client_relayer_cmd:
		pallet_eth2_light_client_relayer_gadget_cli::LightClientRelayerCmd,
	pub auto_insert_keys: bool,
}

/// Builds a new service for a full client.
pub async fn new_full(
	RunFullParams {
		mut config,
		eth_config,
		rpc_config,
		debug_output: _,
		#[cfg(feature = "relayer")]
		relayer_cmd,
		#[cfg(feature = "light-client")]
		light_client_relayer_cmd,
		auto_insert_keys,
	}: RunFullParams,
) -> Result<TaskManager, ServiceError> {
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (mut telemetry, block_import, grandpa_link, frontier_backend, overrides),
	} = new_partial(&config, &eth_config)?;

	if config.role.is_authority() {
		if auto_insert_keys {
			crate::utils::insert_controller_account_keys_into_keystore(
				&config,
				Some(keystore_container.keystore()),
			);
		} else {
			crate::utils::insert_dev_controller_account_keys_into_keystore(
				&config,
				Some(keystore_container.keystore()),
			);
		}

		// finally check if keys are inserted correctly
		if config.chain_spec.chain_type() != ChainType::Development {
			if crate::utils::ensure_all_keys_exist_in_keystore(keystore_container.keystore())
				.is_err()
			{
				println!("   
			++++++++++++++++++++++++++++++++++++++++++++++++                                                                          
				Validator keys not found, validator keys are essential to run a validator on
				Tangle Network, refer to https://docs.webb.tools/docs/ecosystem-roles/validator/required-keys/ on
				how to generate and insert keys. OR start the node with --auto-insert-keys to automatically generate the keys.
			++++++++++++++++++++++++++++++++++++++++++++++++   							
			\n");
				panic!("Keys not detected!")
			}
		}
	}

	let FrontierPartialComponents { filter_pool, fee_history_cache, fee_history_cache_limit } =
		new_frontier_partial(&eth_config)?;

	let mut net_config = sc_network::config::FullNetworkConfiguration::new(&config.network);

	let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
		&client.block_hash(0).ok().flatten().expect("Genesis block exists; qed"),
		&config.chain_spec,
	);

	net_config.add_notification_protocol(sc_consensus_grandpa::grandpa_peers_set_config(
		grandpa_protocol_name.clone(),
	));

	let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
		backend.clone(),
		grandpa_link.shared_authority_set().clone(),
		Vec::default(),
	));

	let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_params: Some(sc_service::WarpSyncParams::WithProvider(warp_sync)),
		})?;

	let role = config.role.clone();
	let force_authoring = config.force_authoring;
	let _backoff_authoring_blocks: Option<()> = None;
	let name = config.network.node_name.clone();
	let enable_grandpa = !config.disable_grandpa;
	let prometheus_registry = config.prometheus_registry().cloned();

	if config.offchain_worker.enabled {
		task_manager.spawn_handle().spawn(
			"offchain-workers-runner",
			"offchain-work",
			sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
				runtime_api_provider: client.clone(),
				keystore: Some(keystore_container.keystore()),
				offchain_db: backend.offchain_storage(),
				transaction_pool: Some(OffchainTransactionPoolFactory::new(
					transaction_pool.clone(),
				)),
				network_provider: network.clone(),
				is_validator: role.is_authority(),
				enable_http_requests: true,
				custom_extensions: move |_| vec![],
			})
			.run(client.clone(), task_manager.spawn_handle())
			.boxed(),
		);
	}

	// Channel for the rpc handler to communicate with the authorship task.
	let (command_sink, _commands_stream) = mpsc::channel(1000);

	// Sinks for pubsub notifications.
	// Everytime a new subscription is created, a new mpsc channel is added to the sink pool.
	// The MappingSyncWorker sends through the channel on block import and the subscription emits a
	// notification to the subscriber on receiving a message through this channel. This way we avoid
	// race conditions when using native substrate block import notification stream.
	let pubsub_notification_sinks: fc_mapping_sync::EthereumBlockNotificationSinks<
		fc_mapping_sync::EthereumBlockNotification<Block>,
	> = Default::default();
	let pubsub_notification_sinks = Arc::new(pubsub_notification_sinks);

	// for ethereum-compatibility rpc.
	config.rpc_id_provider = Some(Box::new(fc_rpc::EthereumSubIdProvider));

	let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
	let target_gas_price = eth_config.target_gas_price;
	let pending_create_inherent_data_providers = move |_, ()| async move {
		let current = sp_timestamp::InherentDataProvider::from_system_time();
		let next_slot = current.timestamp().as_millis() + slot_duration.as_millis();
		let timestamp = sp_timestamp::InherentDataProvider::new(next_slot.into());
		let slot =
			sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
				*timestamp,
				slot_duration,
			);
		let dynamic_fee = fp_dynamic_fee::InherentDataProvider(U256::from(target_gas_price));
		Ok((slot, timestamp, dynamic_fee))
	};

	let ethapi_cmd = rpc_config.ethapi.clone();
	let tracing_requesters =
		if ethapi_cmd.contains(&EthApi::Debug) || ethapi_cmd.contains(&EthApi::Trace) {
			crate::rpc::tracing::spawn_tracing_tasks(
				&task_manager,
				client.clone(),
				backend.clone(),
				frontier_backend.clone(),
				overrides.clone(),
				&rpc_config,
				prometheus_registry.clone(),
			)
		} else {
			crate::rpc::tracing::RpcRequesters { debug: None, trace: None }
		};
	let eth_rpc_params = crate::rpc::EthDeps {
		client: client.clone(),
		pool: transaction_pool.clone(),
		graph: transaction_pool.pool().clone(),
		converter: Some(TransactionConver),
		is_authority: config.role.is_authority(),
		enable_dev_signer: eth_config.enable_dev_signer,
		network: network.clone(),
		sync: sync_service.clone(),
		frontier_backend: match frontier_backend.clone() {
			fc_db::Backend::KeyValue(b) => Arc::new(b),
			fc_db::Backend::Sql(b) => Arc::new(b),
		},
		overrides: overrides.clone(),
		block_data_cache: Arc::new(fc_rpc::EthBlockDataCacheTask::new(
			task_manager.spawn_handle(),
			overrides.clone(),
			eth_config.eth_log_block_cache,
			eth_config.eth_statuses_cache,
			prometheus_registry.clone(),
		)),
		filter_pool: filter_pool.clone(),
		max_past_logs: eth_config.max_past_logs,
		fee_history_cache: fee_history_cache.clone(),
		fee_history_cache_limit,
		execute_gas_limit_multiplier: eth_config.execute_gas_limit_multiplier,
		forced_parent_hashes: None,
		tracing_config: Some(crate::rpc::eth::TracingConfig {
			tracing_requesters: tracing_requesters.clone(),
			trace_filter_max_count: rpc_config.ethapi_trace_max_count,
		}),
		pending_create_inherent_data_providers,
	};

	let rpc_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let pubsub_notification_sinks = pubsub_notification_sinks.clone();
		Box::new(move |deny_unsafe, subscription_task_executor| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				deny_unsafe,
				command_sink: Some(command_sink.clone()),
				eth: eth_rpc_params.clone(),
			};
			if ethapi_cmd.contains(&EthApi::Debug) || ethapi_cmd.contains(&EthApi::Trace) {
				crate::rpc::create_full(
					deps,
					subscription_task_executor,
					pubsub_notification_sinks.clone(),
				)
				.map_err(Into::into)
			} else {
				crate::rpc::create_full(
					deps,
					subscription_task_executor,
					pubsub_notification_sinks.clone(),
				)
				.map_err(Into::into)
			}
		})
	};

	spawn_frontier_tasks(
		&task_manager,
		client.clone(),
		backend.clone(),
		frontier_backend,
		filter_pool,
		overrides,
		fee_history_cache,
		fee_history_cache_limit,
		sync_service.clone(),
		pubsub_notification_sinks,
	)
	.await;

	if role.is_authority() {
		// setup relayer gadget params
		#[cfg(feature = "relayer")]
		let relayer_params = tangle_relayer_gadget::RelayerParams {
			local_keystore: keystore_container.local_keystore(),
			config_dir: relayer_cmd.relayer_config_dir,
			database_path: config
				.database
				.path()
				.and_then(|path| path.parent())
				.map(|p| p.to_path_buf()),
			rpc_addr: config.rpc_addr,
		};

		// Start Webb Relayer Gadget as non-essential task.
		#[cfg(feature = "relayer")]
		task_manager.spawn_handle().spawn(
			"relayer-gadget",
			None,
			tangle_relayer_gadget::start_relayer_gadget(
				relayer_params,
				sp_application_crypto::KeyTypeId(*b"role"),
			),
		);

		// Start Eth2 Light client Relayer Gadget - (MAINNET RELAYER)
		#[cfg(feature = "light-client")]
		task_manager.spawn_handle().spawn(
			"mainnet-relayer-gadget",
			None,
			pallet_eth2_light_client_relayer_gadget::start_gadget(
				pallet_eth2_light_client_relayer_gadget::Eth2LightClientParams {
					lc_relay_config_path: light_client_relayer_cmd
						.light_client_relay_config_path
						.clone(),
					lc_init_config_path: light_client_relayer_cmd
						.light_client_init_pallet_config_path
						.clone(),
					eth2_chain_id: webb_proposals::TypedChainId::Evm(1),
				},
			),
		);
	}
	let params = sc_service::SpawnTasksParams {
		network: network.clone(),
		client: client.clone(),
		keystore: keystore_container.keystore(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		rpc_builder,
		backend: backend.clone(),
		system_rpc_tx,
		tx_handler_controller,
		sync_service: sync_service.clone(),
		config,
		telemetry: telemetry.as_mut(),
	};
	let _rpc_handlers = sc_service::spawn_tasks(params)?;

	if role.is_authority() {
		let proposer_factory = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
		let target_gas_price = eth_config.target_gas_price;
		let create_inherent_data_providers = move |_, ()| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
			let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
				*timestamp,
				slot_duration,
			);
			let dynamic_fee = fp_dynamic_fee::InherentDataProvider(U256::from(target_gas_price));
			Ok((slot, timestamp, dynamic_fee))
		};

		let aura = sc_consensus_aura::start_aura::<AuraPair, _, _, _, _, _, _, _, _, _, _>(
			sc_consensus_aura::StartAuraParams {
				slot_duration,
				client,
				select_chain,
				block_import,
				proposer_factory,
				sync_oracle: sync_service.clone(),
				justification_sync_link: sync_service.clone(),
				create_inherent_data_providers,
				force_authoring,
				backoff_authoring_blocks: Option::<()>::None,
				keystore: keystore_container.keystore(),
				block_proposal_slot_portion: sc_consensus_aura::SlotProportion::new(2f32 / 3f32),
				max_block_proposal_slot_portion: None,
				telemetry: telemetry.as_ref().map(|x| x.handle()),
				compatibility_mode: sc_consensus_aura::CompatibilityMode::None,
			},
		)?;

		// the AURA authoring task is considered essential, i.e. if it
		// fails we take down the service with it.
		task_manager
			.spawn_essential_handle()
			.spawn_blocking("aura", Some("block-authoring"), aura);
	}

	// if the node isn't actively participating in consensus then it doesn't
	// need a keystore, regardless of which protocol we use below.
	let keystore = if role.is_authority() { Some(keystore_container.keystore()) } else { None };

	let grandpa_config = sc_consensus_grandpa::Config {
		// FIXME #1578 make this available through chainspec
		gossip_duration: Duration::from_millis(333),
		justification_generation_period: GRANDPA_JUSTIFICATION_PERIOD,
		name: Some(name),
		observer_enabled: false,
		keystore,
		local_role: role,
		telemetry: telemetry.as_ref().map(|x| x.handle()),
		protocol_name: grandpa_protocol_name,
	};

	if enable_grandpa {
		// start the full GRANDPA voter
		// NOTE: non-authorities could run the GRANDPA observer protocol, but at
		// this point the full voter should provide better guarantees of block
		// and vote data availability than the observer. The observer has not
		// been tested extensively yet and having most nodes in a network run it
		// could lead to finality stalls.
		let grandpa_config = sc_consensus_grandpa::GrandpaParams {
			config: grandpa_config,
			link: grandpa_link,
			network,
			sync: Arc::new(sync_service),
			voting_rule: sc_consensus_grandpa::VotingRulesBuilder::default().build(),
			prometheus_registry,
			shared_voter_state: SharedVoterState::empty(),
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(transaction_pool),
		};

		// the GRANDPA voter task is considered infallible, i.e.
		// if it fails we take down the service with it.
		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			None,
			sc_consensus_grandpa::run_grandpa_voter(grandpa_config)?,
		);
	}

	network_starter.start_network();
	Ok(task_manager)
}
