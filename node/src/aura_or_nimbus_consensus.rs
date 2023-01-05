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

//! Implements a consensus that can verify blocks with both Nimbus and Aura
//! by adding a custom import_queue and Verifier that delegate based on
//! what type of seal a block has
//! NOTE: Does not change block *proposing*, set to Nimbus in service.rs
//! NOTE: Assumes running as a Parachain. Sovereign chain mode NOT SUPPORTED

use futures::TryFutureExt;
use log::debug;
use sc_client_api::HeaderBackend;
use sc_consensus::{
	import_queue::{BasicQueue, Verifier as VerifierT},
	BlockImport, BlockImportParams,
};
use sc_consensus_aura::BuildVerifierParams;
use sc_consensus_slots::InherentDataProviderExt;
use sc_telemetry::TelemetryHandle;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::Result as ClientResult;
use sp_consensus::{error::Error as ConsensusError, CacheKeyId};
use sp_consensus_aura::AuraApi;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::{
	app_crypto::AppKey,
	traits::{Block as BlockT, Header as HeaderT},
};
use std::sync::Arc;
use tangle_rococo_runtime::AuraId;

use nimbus_primitives::CompatibleDigestItem as NimbusDigestItem;
use sc_consensus_aura::CompatibleDigestItem as AuraDigestItem;

const LOG_TARGET: &str = "aura-nimbus-consensus";

struct AuraOrNimbusVerifier<Client, Block: BlockT, AuraCIDP, NimbusCIDP> {
	aura_verifier: sc_consensus_aura::AuraVerifier<Client, <AuraId as AppKey>::Pair, AuraCIDP, <<Block as BlockT>::Header as HeaderT>::Number>,
	nimbus_verifier: nimbus_consensus::Verifier<Client, Block, NimbusCIDP>,
}
impl<Client, Block, AuraCIDP, NimbusCIDP> AuraOrNimbusVerifier<Client, Block, AuraCIDP, NimbusCIDP>
where
	Block: BlockT,
{
	pub fn new(
		client: Arc<Client>,
		create_inherent_data_providers_aura: AuraCIDP,
		create_inherent_data_providers_nimbus: NimbusCIDP,
		telemetry: Option<TelemetryHandle>,
	) -> Self
	where
		Client: ProvideRuntimeApi<Block> + Send + Sync + 'static,
		<Client as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block>,
		AuraCIDP: CreateInherentDataProviders<Block, ()> + 'static,
		NimbusCIDP: CreateInherentDataProviders<Block, ()> + 'static,
	{
		Self {
			aura_verifier: sc_consensus_aura::build_verifier(BuildVerifierParams {
				client: client.clone(),
				create_inherent_data_providers: create_inherent_data_providers_aura,
				check_for_equivocation: sc_consensus_aura::CheckForEquivocation::Yes,
				telemetry,
				compatibility_mode: Default::default()
			}),
			nimbus_verifier: nimbus_consensus::Verifier::new(
				client,
				create_inherent_data_providers_nimbus,
			),
		}
	}
}

#[async_trait::async_trait]
impl<Client, Block, AuraCIDP, NimbusCIDP> VerifierT<Block>
	for AuraOrNimbusVerifier<Client, Block, AuraCIDP, NimbusCIDP>
where
	Block: BlockT,
	Client: ProvideRuntimeApi<Block> + Send + Sync,
	<Client as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block> + AuraApi<Block, AuraId>,
	AuraCIDP: CreateInherentDataProviders<Block, ()> + 'static,
	<AuraCIDP as CreateInherentDataProviders<Block, ()>>::InherentDataProviders:
		InherentDataProviderExt,
	NimbusCIDP: CreateInherentDataProviders<Block, ()>,
	Client: sc_client_api::AuxStore + sc_client_api::BlockOf,
{
	async fn verify(
		&mut self,
		block_params: BlockImportParams<Block, ()>,
	) -> Result<(BlockImportParams<Block, ()>, Option<Vec<(CacheKeyId, Vec<u8>)>>), String> {
		// We assume the outermost digest item is the block seal ( we have no two-step consensus )
		let seal = block_params
			.header
			.digest()
			.logs()
			.last()
			.ok_or("Block should have at least one digest/seal on it")?;

		// delegate verification to Aura or Nimbus verifiers
		if NimbusDigestItem::as_nimbus_seal(seal).is_some() {
			debug!(target: LOG_TARGET, "Verifying block with Nimbus");
			self.nimbus_verifier.verify(block_params).map_err(Into::into).await
		} else if AuraDigestItem::<<AuraId as AppKey>::Signature>::as_aura_seal(seal).is_some() {
			debug!(target: LOG_TARGET, "Verifying block with Aura");
			self.aura_verifier.verify(block_params).map_err(Into::into).await
		} else {
			Err("No Aura or Nimbus seal on block".to_string())
		}
	}
}

pub fn import_queue<Client, Block: BlockT, InnerBI>(
	client: Arc<Client>,
	block_import: InnerBI,
	spawner: &impl sp_core::traits::SpawnEssentialNamed,
	registry: Option<&substrate_prometheus_endpoint::Registry>,
	telemetry: Option<TelemetryHandle>,
) -> ClientResult<BasicQueue<Block, InnerBI::Transaction>>
where
	InnerBI: BlockImport<Block, Error = ConsensusError> + Send + Sync + 'static,
	InnerBI::Transaction: Send,
	Client::Api: BlockBuilderApi<Block>,
	Client: ProvideRuntimeApi<Block> + Send + Sync + 'static,
	Client: sc_client_api::AuxStore + sc_client_api::UsageProvider<Block>,
	Client: HeaderBackend<Block> + sc_client_api::BlockOf,
	<Client as ProvideRuntimeApi<Block>>::Api: BlockBuilderApi<Block> + AuraApi<Block, AuraId>,
{
	let verifier = AuraOrNimbusVerifier::new(
		client.clone(),
		move |_, _| {
			let client2 = client.clone();
			async move {
				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
				let slot_duration =
					cumulus_client_consensus_aura::slot_duration(&*client2).unwrap();
				let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                            *timestamp,
                            slot_duration,
                        );
				Ok((slot, timestamp))
			}
		},
		move |_, _| async move {
			let time = sp_timestamp::InherentDataProvider::from_system_time();
			Ok((time,))
		},
		telemetry,
	);
	Ok(BasicQueue::new(
		verifier,
		// NOTE: As of Aug2022, nimbus and aura simply delegate block import
		//       to cumulus. We're skipping wrapping both by using this directly.
		//       If in the future either of them diverge from this,
		//       we'll have to adapt to the change here and in
		//       node/src/service.rs:L467 aka. BuildNimbusConsensusParams
		Box::new(cumulus_client_consensus_common::ParachainBlockImport::new(block_import)),
		None,
		spawner,
		registry,
	))
}
