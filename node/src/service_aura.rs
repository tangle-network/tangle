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

//! Parachain Node Service
use crate::rpc;
use codec::Codec;
use core::marker::PhantomData;
use cumulus_client_cli::CollatorOptions;
use cumulus_client_consensus_aura::{AuraConsensus, BuildAuraConsensusParams, SlotProportion};
use cumulus_client_consensus_common::{
	ParachainBlockImport, ParachainCandidate, ParachainConsensus,
};
use cumulus_client_consensus_relay_chain::Verifier as RelayChainVerifier;
use cumulus_client_network::BlockAnnounceValidator;
use cumulus_client_service::{
	prepare_node_config, start_collator, start_full_node, SharedImportQueue, StartCollatorParams,
	StartFullNodeParams,
};
use cumulus_primitives_core::{
	relay_chain::v2::{Hash as PHash, PersistedValidationData},
	ParaId,
};
use cumulus_relay_chain_inprocess_interface::build_inprocess_relay_chain;
use cumulus_relay_chain_interface::{RelayChainError, RelayChainInterface, RelayChainResult};
use jsonrpsee::RpcModule;
use polkadot_service::CollatorPair;
use sc_network::NetworkBlock;

use futures::lock::Mutex;
use sc_consensus::{
	import_queue::{BasicQueue, Verifier as VerifierT},
	BlockImportParams,
};
use sc_executor::{NativeExecutionDispatch, WasmExecutor};
use sc_network::NetworkService;
pub use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sc_service::{Configuration, Error, Role, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sp_api::{ApiExt, ConstructRuntimeApi};
use sp_consensus::CacheKeyId;
use sp_consensus_aura::AuraApi;
use sp_core::crypto::Pair;
use sp_keystore::SyncCryptoStorePtr;
use sp_offchain::OffchainWorkerApi;
use sp_runtime::{
	app_crypto::AppKey,
	generic::BlockId,
	traits::{BlakeTwo256, Header as HeaderT},
};
use sp_session::SessionKeys;
use sp_transaction_pool::runtime_api::TaggedTransactionQueue;
use std::sync::Arc;
use substrate_prometheus_endpoint::Registry;
pub use tangle_rococo_runtime::{opaque::Block, AccountId, Balance, Hash, Header, Index as Nonce};
use cumulus_relay_chain_rpc_interface::{create_client_and_start_worker, RelayChainRpcInterface};

#[cfg(not(feature = "runtime-benchmarks"))]
type HostFunctions = sp_io::SubstrateHostFunctions;

#[cfg(feature = "runtime-benchmarks")]
type HostFunctions =
	(sp_io::SubstrateHostFunctions, frame_benchmarking::benchmarking::HostFunctions);

/// Native Manta Parachain executor instance.
pub struct MantaRuntimeExecutor;
impl NativeExecutionDispatch for MantaRuntimeExecutor {
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		tangle_rococo_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		tangle_rococo_runtime::native_version()
	}
}

/// We use wasm executor only now.
pub type DefaultExecutorType = WasmExecutor<HostFunctions>;

/// Full Client Implementation Type
pub type Client<RuntimeApi> = TFullClient<Block, RuntimeApi, DefaultExecutorType>;

/// Default Import Queue Type
pub type ImportQueue<RuntimeApi> = sc_consensus::DefaultImportQueue<Block, Client<RuntimeApi>>;

/// Full Transaction Pool Type
pub type TransactionPool<RuntimeApi> = sc_transaction_pool::FullPool<Block, Client<RuntimeApi>>;

/// Components Needed for Chain Ops Subcommands
pub type PartialComponents<RuntimeApi> = sc_service::PartialComponents<
	Client<RuntimeApi>,
	TFullBackend<Block>,
	(),
	ImportQueue<RuntimeApi>,
	TransactionPool<RuntimeApi>,
	(Option<Telemetry>, Option<TelemetryWorkerHandle>),
>;

/// State Backend Type
pub type StateBackend = sc_client_api::StateBackendFor<TFullBackend<Block>, Block>;

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
pub fn new_partial<RuntimeApi, BIQ>(
	config: &Configuration,
	build_import_queue: BIQ,
) -> Result<PartialComponents<RuntimeApi>, Error>
where
	RuntimeApi: ConstructRuntimeApi<Block, Client<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ SessionKeys<Block>
		+ ApiExt<Block, StateBackend = StateBackend>
		+ OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>,
	StateBackend: sp_api::StateBackend<BlakeTwo256>,
	BIQ: FnOnce(
		Arc<Client<RuntimeApi>>,
		&Configuration,
		Option<TelemetryHandle>,
		&TaskManager,
	) -> Result<ImportQueue<RuntimeApi>, Error>,
{
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
	let executor = sc_executor::WasmExecutor::<HostFunctions>::new(
		config.wasm_method,
		config.default_heap_pages,
		config.max_runtime_instances,
		None,
		config.runtime_cache_size,
	);
	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let client = Arc::new(client);
	let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());
	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});
	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);
	let import_queue = build_import_queue(
		client.clone(),
		config,
		telemetry.as_ref().map(|telemetry| telemetry.handle()),
		&task_manager,
	)?;
	Ok(PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		select_chain: (),
		other: (telemetry, telemetry_worker_handle),
	})
}

async fn build_relay_chain_interface(
	polkadot_config: Configuration,
	parachain_config: &Configuration,
	telemetry_worker_handle: Option<TelemetryWorkerHandle>,
	task_manager: &mut TaskManager,
	collator_options: CollatorOptions,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> RelayChainResult<(Arc<(dyn RelayChainInterface + 'static)>, Option<CollatorPair>)> {
	match collator_options.relay_chain_rpc_url {
		Some(relay_chain_url) => {
			let client = create_client_and_start_worker(relay_chain_url, task_manager).await?;
			Ok((Arc::new(RelayChainRpcInterface::new(client)) as Arc<_>, None))
		},
		None => build_inprocess_relay_chain(
			polkadot_config,
			parachain_config,
			telemetry_worker_handle,
			task_manager,
			hwbench,
		),
	}
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
#[allow(clippy::too_many_arguments)]

async fn start_node_impl<RuntimeApi, BIQ, BIC, RB>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	id: ParaId,
	rpc_ext_builder: RB,
	build_import_queue: BIQ,
	build_consensus: BIC,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<Client<RuntimeApi>>)>
where
	RuntimeApi: ConstructRuntimeApi<Block, Client<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ SessionKeys<Block>
		+ ApiExt<Block, StateBackend = StateBackend>
		+ OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ cumulus_primitives_core::CollectCollationInfo<Block>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	StateBackend: sp_api::StateBackend<BlakeTwo256>,

	RB: Fn(
			rpc::FullDeps<Client<RuntimeApi>, TransactionPool<RuntimeApi>>,
		) -> Result<RpcModule<()>, Error>
		+ 'static,
	BIQ: FnOnce(
			Arc<Client<RuntimeApi>>,
			&Configuration,
			Option<TelemetryHandle>,
			&TaskManager,
		) -> Result<ImportQueue<RuntimeApi>, Error>
		+ 'static,
	BIC: FnOnce(
		Arc<Client<RuntimeApi>>,
		Option<&Registry>,
		Option<TelemetryHandle>,
		&TaskManager,
		Arc<dyn RelayChainInterface>,
		Arc<TransactionPool<RuntimeApi>>,
		Arc<NetworkService<Block, Hash>>,
		SyncCryptoStorePtr,
		bool,
	) -> Result<Box<dyn ParachainConsensus<Block>>, Error>,
{
	let parachain_config = prepare_node_config(parachain_config);

	let params = new_partial::<RuntimeApi, BIQ>(&parachain_config, build_import_queue)?;
	let (mut telemetry, telemetry_worker_handle) = params.other;

	let mut task_manager = params.task_manager;
	let (relay_chain_interface, collator_key) = build_relay_chain_interface(
		polkadot_config,
		&parachain_config,
		telemetry_worker_handle,
		&mut task_manager,
		collator_options.clone(),
		hwbench.clone(),
	)
	.await
	.map_err(|e| match e {
		RelayChainError::ServiceError(polkadot_service::Error::Sub(x)) => x,
		s => s.to_string().into(),
	})?;

	let client = params.client.clone();
	let backend = params.backend.clone();
	let block_announce_validator = BlockAnnounceValidator::new(relay_chain_interface.clone(), id);

	let force_authoring = parachain_config.force_authoring;
	let validator = parachain_config.role.is_authority();
	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = params.transaction_pool.clone();
	let import_queue = SharedImportQueue::new(params.import_queue);
	let (network, system_rpc_tx, tx_handler_controller, start_network) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &parachain_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue: import_queue.clone(),
			block_announce_validator_builder: Some(Box::new(|_| {
				Box::new(block_announce_validator)
			})),
			warp_sync: None,
		})?;

	let rpc_builder = {
		let client = client.clone();
		let transaction_pool = transaction_pool.clone();

		Box::new(move |deny_unsafe, _| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				deny_unsafe,
			};

			rpc_ext_builder(deps)
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_builder,
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: params.keystore_container.sync_keystore(),
		backend: backend.clone(),
		network: network.clone(),
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	let announce_block = {
		let network = network.clone();
		Arc::new(move |hash, data| network.announce_block(hash, data))
	};

	let relay_chain_slot_duration = core::time::Duration::from_secs(6);
	if validator {
		let parachain_consensus = build_consensus(
			client.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t| t.handle()),
			&task_manager,
			relay_chain_interface.clone(),
			transaction_pool,
			network,
			params.keystore_container.sync_keystore(),
			force_authoring,
		)?;
		let spawner = task_manager.spawn_handle();
		start_collator(StartCollatorParams {
			para_id: id,
			block_status: client.clone(),
			announce_block,
			client: client.clone(),
			task_manager: &mut task_manager,
			relay_chain_interface,
			spawner,
			parachain_consensus,
			import_queue,
			collator_key: collator_key.expect("Command line arguments do not allow this. qed"),
			relay_chain_slot_duration,
		})
		.await?;
	} else {
		start_full_node(StartFullNodeParams {
			client: client.clone(),
			announce_block,
			task_manager: &mut task_manager,
			para_id: id,
			relay_chain_interface,
			relay_chain_slot_duration,
			import_queue,
			collator_options,
		})?;
	}

	start_network.start_network();
	Ok((task_manager, client))
}

enum BuildOnAccess<R> {
	Uninitialized(Option<Box<dyn FnOnce() -> R + Send + Sync>>),
	Initialized(R),
}

impl<R> BuildOnAccess<R> {
	fn get_mut(&mut self) -> &mut R {
		loop {
			match self {
				Self::Uninitialized(f) => {
					*self = Self::Initialized((f.take().unwrap())());
				},
				Self::Initialized(ref mut r) => return r,
			}
		}
	}
}

/// Special [`ParachainConsensus`] implementation that waits for the upgrade from
/// shell to a parachain runtime that implements Aura.
struct WaitForAuraConsensus<Client, AuraId> {
	client: Arc<Client>,
	aura_consensus: Arc<Mutex<BuildOnAccess<Box<dyn ParachainConsensus<Block>>>>>,
	relay_chain_consensus: Arc<Mutex<Box<dyn ParachainConsensus<Block>>>>,
	_phantom: PhantomData<AuraId>,
}

impl<Client, AuraId> Clone for WaitForAuraConsensus<Client, AuraId> {
	fn clone(&self) -> Self {
		Self {
			client: self.client.clone(),
			aura_consensus: self.aura_consensus.clone(),
			relay_chain_consensus: self.relay_chain_consensus.clone(),
			_phantom: PhantomData,
		}
	}
}

#[async_trait::async_trait]
impl<Client, AuraId> ParachainConsensus<Block> for WaitForAuraConsensus<Client, AuraId>
where
	Client: sp_api::ProvideRuntimeApi<Block> + Send + Sync,
	Client::Api: AuraApi<Block, AuraId>,
	AuraId: Send + Codec + Sync,
{
	async fn produce_candidate(
		&mut self,
		parent: &Header,
		relay_parent: PHash,
		validation_data: &PersistedValidationData,
	) -> Option<ParachainCandidate<Block>> {
		let block_id = BlockId::hash(parent.hash());
		if self
			.client
			.runtime_api()
			.has_api::<dyn AuraApi<Block, AuraId>>(&block_id)
			.unwrap_or(false)
		{
			self.aura_consensus
				.lock()
				.await
				.get_mut()
				.produce_candidate(parent, relay_parent, validation_data)
				.await
		} else {
			self.relay_chain_consensus
				.lock()
				.await
				.produce_candidate(parent, relay_parent, validation_data)
				.await
		}
	}
}

struct Verifier<Client, AuraId> {
	client: Arc<Client>,
	aura_verifier: BuildOnAccess<Box<dyn VerifierT<Block>>>,
	relay_chain_verifier: Box<dyn VerifierT<Block>>,
	_phantom: PhantomData<AuraId>,
}

#[async_trait::async_trait]
impl<Client, AuraId> VerifierT<Block> for Verifier<Client, AuraId>
where
	Client: sp_api::ProvideRuntimeApi<Block> + Send + Sync,
	Client::Api: AuraApi<Block, AuraId>,
	AuraId: Send + Sync + Codec,
{
	async fn verify(
		&mut self,
		block_import: BlockImportParams<Block, ()>,
	) -> Result<(BlockImportParams<Block, ()>, Option<Vec<(CacheKeyId, Vec<u8>)>>), String> {
		let block_id = BlockId::hash(*block_import.header.parent_hash());

		if self
			.client
			.runtime_api()
			.has_api::<dyn AuraApi<Block, AuraId>>(&block_id)
			.unwrap_or(false)
		{
			self.aura_verifier.get_mut().verify(block_import).await
		} else {
			self.relay_chain_verifier.verify(block_import).await
		}
	}
}

/// Build the import queue for the calamari/manta runtime.
pub fn parachain_build_import_queue<RuntimeApi, AuraId: AppKey>(
	client: Arc<Client<RuntimeApi>>,
	config: &Configuration,
	telemetry_handle: Option<TelemetryHandle>,
	task_manager: &TaskManager,
) -> Result<ImportQueue<RuntimeApi>, Error>
where
	RuntimeApi: ConstructRuntimeApi<Block, Client<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ SessionKeys<Block>
		+ ApiExt<Block, StateBackend = StateBackend>
		+ OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ sp_consensus_aura::AuraApi<Block, <<AuraId as AppKey>::Pair as Pair>::Public>,
	StateBackend: sp_api::StateBackend<BlakeTwo256>,
	<<AuraId as AppKey>::Pair as Pair>::Signature:
		TryFrom<Vec<u8>> + std::hash::Hash + sp_runtime::traits::Member + Codec,
{
	let client2 = client.clone();

	let aura_verifier = move || {
		let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client2).unwrap();
		Box::new(cumulus_client_consensus_aura::build_verifier::<<AuraId as AppKey>::Pair, _, _>(
			cumulus_client_consensus_aura::BuildVerifierParams {
				client: client2.clone(),
				create_inherent_data_providers: move |_, _| async move {
					let time = sp_timestamp::InherentDataProvider::from_system_time();

					let slot =
                    sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                        *time,
                        slot_duration,
                    );

					Ok((slot, time))
				},
				telemetry: telemetry_handle,
			},
		)) as Box<_>
	};

	let relay_chain_verifier =
		Box::new(RelayChainVerifier::new(client.clone(), |_, _| async { Ok(()) }));

	let verifier = Verifier {
		client: client.clone(),
		relay_chain_verifier,
		aura_verifier: BuildOnAccess::Uninitialized(Some(Box::new(aura_verifier))),
		_phantom: PhantomData,
	};

	let registry = config.prometheus_registry();
	let spawner = task_manager.spawn_essential_handle();

	Ok(BasicQueue::new(
		verifier,
		Box::new(ParachainBlockImport::new(client)),
		None,
		&spawner,
		registry,
	))
}

/// Start a calamari/manta parachain node.
pub async fn start_parachain_node<RuntimeApi, AuraId: AppKey, RB>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	id: ParaId,
	hwbench: Option<sc_sysinfo::HwBench>,
	rpc_ext_builder: RB,
) -> sc_service::error::Result<(TaskManager, Arc<Client<RuntimeApi>>)>
where
	RuntimeApi: ConstructRuntimeApi<Block, Client<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ SessionKeys<Block>
		+ ApiExt<Block, StateBackend = StateBackend>
		+ OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ cumulus_primitives_core::CollectCollationInfo<Block>
		+ sp_consensus_aura::AuraApi<Block, <<AuraId as AppKey>::Pair as Pair>::Public>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	StateBackend: sp_api::StateBackend<BlakeTwo256>,
	<<AuraId as AppKey>::Pair as Pair>::Signature:
		TryFrom<Vec<u8>> + std::hash::Hash + sp_runtime::traits::Member + Codec,
	RB: Fn(
			rpc::FullDeps<Client<RuntimeApi>, TransactionPool<RuntimeApi>>,
		) -> Result<RpcModule<()>, Error>
		+ 'static,
{
	start_node_impl::<RuntimeApi, _, _, _>(
		parachain_config,
		polkadot_config,
		collator_options,
		id,
		rpc_ext_builder,
		parachain_build_import_queue::<_, AuraId>,
		|client,
		 prometheus_registry,
		 telemetry,
		 task_manager,
		 relay_chain_interface,
		 transaction_pool,
		 sync_oracle,
		 keystore,
		 force_authoring| {
			let client2 = client.clone();
			let spawn_handle = task_manager.spawn_handle();
			let transaction_pool2 = transaction_pool.clone();
			let telemetry2 = telemetry.clone();
			let prometheus_registry2 = prometheus_registry.map(|r| (*r).clone());
			let relay_chain_for_aura = relay_chain_interface.clone();
			let aura_consensus = BuildOnAccess::Uninitialized(Some(Box::new(move || {
				let slot_duration =
					cumulus_client_consensus_aura::slot_duration(&*client2).unwrap();

				let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
					spawn_handle,
					client2.clone(),
					transaction_pool2,
					prometheus_registry2.as_ref(),
					telemetry2.clone(),
				);

				AuraConsensus::build::<<AuraId as AppKey>::Pair, _, _, _, _, _, _>(
					BuildAuraConsensusParams {
						proposer_factory,
						create_inherent_data_providers:
							move |_, (relay_parent, validation_data)| {
								let relay_chain_for_aura = relay_chain_for_aura.clone();
								async move {
									let parachain_inherent =
										cumulus_primitives_parachain_inherent::ParachainInherentData::create_at(
											relay_parent,
											&relay_chain_for_aura,
											&validation_data,
											id,
										).await;

									let timestamp =
										sp_timestamp::InherentDataProvider::from_system_time();

									let slot =
										sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
											*timestamp,
											slot_duration,
										);

									let parachain_inherent =
										parachain_inherent.ok_or_else(|| {
											Box::<dyn std::error::Error + Send + Sync>::from(
												"Failed to create parachain inherent",
											)
										})?;

									Ok((slot, timestamp, parachain_inherent))
								}
							},
						block_import: client2.clone(),
						para_client: client2.clone(),
						backoff_authoring_blocks: Option::<()>::None,
						sync_oracle,
						keystore,
						force_authoring,
						slot_duration,
						// We got around 500ms for proposing
						block_proposal_slot_portion: SlotProportion::new(1f32 / 24f32),
						// And a maximum of 750ms if slots are skipped
						max_block_proposal_slot_portion: Some(SlotProportion::new(1f32 / 16f32)),
						telemetry: telemetry2,
					},
				)
			})));

			let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
				task_manager.spawn_handle(),
				client.clone(),
				transaction_pool,
				prometheus_registry,
				telemetry,
			);

			let relay_chain_consensus =
				cumulus_client_consensus_relay_chain::build_relay_chain_consensus(
					cumulus_client_consensus_relay_chain::BuildRelayChainConsensusParams {
						para_id: id,
						proposer_factory,
						block_import: client.clone(),
						relay_chain_interface: relay_chain_interface.clone(),
						create_inherent_data_providers:
							move |_, (relay_parent, validation_data)| {
								let relay_chain_interface = relay_chain_interface.clone();
								async move {
									let parachain_inherent =
									cumulus_primitives_parachain_inherent::ParachainInherentData::create_at(
										relay_parent,
										&relay_chain_interface,
										&validation_data,
										id,
									).await;
									let parachain_inherent =
										parachain_inherent.ok_or_else(|| {
											Box::<dyn std::error::Error + Send + Sync>::from(
												"Failed to create parachain inherent",
											)
										})?;
									Ok(parachain_inherent)
								}
							},
					},
				);

			let parachain_consensus = Box::new(WaitForAuraConsensus {
				client,
				aura_consensus: Arc::new(Mutex::new(aura_consensus)),
				relay_chain_consensus: Arc::new(Mutex::new(relay_chain_consensus)),
				_phantom: PhantomData,
			});

			Ok(parachain_consensus)
		},
		hwbench,
	)
	.await
}
