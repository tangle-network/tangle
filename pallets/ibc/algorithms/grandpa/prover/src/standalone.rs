// Copyright (C) 2022 ComposableFi.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#![allow(clippy::all)]
#![deny(missing_docs)]

//! GRANDPA standalone (non relay chain) prover utilities

use anyhow::anyhow;
pub use beefy_prover;
use grandpa_light_client_primitives::{FinalityProof, StandaloneClientState};
use jsonrpsee::{async_client::Client, tracing::log, ws_client::WsClientBuilder};
use light_client_common::config::RuntimeStorage;
use parity_scale_codec::{Decode, Encode};
use sc_consensus_grandpa_rpc::GrandpaApiClient;
use serde::{Deserialize, Serialize};
use sp_consensus_grandpa::{AuthorityId, AuthoritySignature};
use sp_core::H256;
use sp_runtime::{
	traits::{One, Zero},
	SaturatedConversion,
};
use std::{collections::BTreeSet, sync::Arc, time::Duration};
use subxt::{
	backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
	config::Header,
	Config, OnlineClient,
};

/// The maximum number of authority set changes to request at once
pub const PROCESS_CHANGES_SET_BATCH_SIZE: usize = 100;
/// The maximum number of blocks to request at once
pub const PROCESS_BLOCKS_BATCH_SIZE: usize = 100;

/// Contains methods useful for proving parachain header finality using GRANDPA
pub struct GrandpaStandaloneProver<T: Config> {
	/// Subxt client for the relay chain
	pub client: OnlineClient<T>,
	/// Standalone chain jsonrpsee client for typed rpc requests, which subxt lacks support for.
	pub ws_client: Arc<Client>,
	/// Standalone chain rpc client
	pub rpc_client: RpcClient,
	/// ChainId of the associated parachain
	pub chain_id: u32,
	/// Delay between rpc calls to the RPC
	pub rpc_call_delay: Duration,
}

// We redefine these here because we want the header to be bounded by subxt::config::Header in the
// prover
/// Commit
pub type Commit = finality_grandpa::Commit<H256, u32, AuthoritySignature, AuthorityId>;

/// Justification
#[cfg_attr(any(feature = "std", test), derive(Debug))]
#[derive(Clone, Encode, Decode)]
pub struct GrandpaJustification<H: Header + parity_scale_codec::Decode> {
	/// Current voting round number, monotonically increasing
	pub round: u64,
	/// Contains block hash & number that's being finalized and the signatures.
	pub commit: Commit,
	/// Contains the path from a [`PreCommit`]'s target hash to the GHOST finalized block.
	pub votes_ancestries: Vec<H>,
}

/// An encoded justification proving that the given header has been finalized
#[derive(Clone, Serialize, Deserialize)]
pub struct JustificationNotification(pub sp_core::Bytes);

impl<T: Config> Clone for GrandpaStandaloneProver<T> {
	fn clone(&self) -> Self {
		Self {
			client: self.client.clone(),
			rpc_client: self.rpc_client.clone(),
			ws_client: self.ws_client.clone(),
			chain_id: self.chain_id,
			rpc_call_delay: self.rpc_call_delay,
		}
	}
}

impl<T> GrandpaStandaloneProver<T>
where
	T: light_client_common::config::Config + Send + Sync,
	<<T as subxt::Config>::Header as Header>::Number: Ord + Zero,
	sp_core::H256: From<T::Hash>,
{
	/// Initializes the parachain and relay chain clients given the ws urls.
	pub async fn new(
		ws_url: &str,
		chain_id: u32,
		rpc_call_delay: Duration,
	) -> Result<Self, anyhow::Error> {
		let ws_client = Arc::new(WsClientBuilder::default().build(ws_url).await?);
		let rpc_client = RpcClient::from_url(ws_url).await?;
		let client = OnlineClient::<T>::from_rpc_client(rpc_client.to_owned().clone()).await?;

		Ok(Self { ws_client, rpc_client, client, chain_id, rpc_call_delay })
	}

	/// Construct the inital client state.
	pub async fn initialize_client_state(&self) -> Result<StandaloneClientState, anyhow::Error>
	where
		<T as subxt::Config>::Header: Decode,
	{
		use sp_consensus_grandpa::AuthorityList;
		let latest_hash = LegacyRpcMethods::<T>::new(self.rpc_client.clone())
			.chain_get_finalized_head()
			.await
			.unwrap();
		log::debug!(target: "hyperspace", "Latest relay hash: {:?}", latest_hash);
		let header = LegacyRpcMethods::<T>::new(self.rpc_client.clone())
			.chain_get_header(Some(latest_hash))
			.await
			.unwrap()
			.ok_or_else(|| anyhow!("Header not found for hash: {latest_hash:?}"))
			.unwrap();

		let current_set_id = {
			let key = T::Storage::grandpa_current_set_id();
			self.client
				.storage()
				.at(latest_hash)
				.fetch(&key)
				.await
				.unwrap()
				.expect("Failed to fetch current set id")
		};

		let current_authorities = {
			let bytes = self
				.rpc_client
				.request::<String>(
					"state_call",
					subxt::rpc_params!(
						"GrandpaApi_grandpa_authorities",
						"0x",
						Some(format!("{:?}", latest_hash))
					),
				)
				.await
				.map(|res| hex::decode(&res[2..]))
				.unwrap()
				.unwrap();

			AuthorityList::decode(&mut &bytes[..]).expect("Failed to scale decode authorities")
		};

		// Ensure there are no duplicates in authority list
		let mut set = BTreeSet::new();
		for (id, ..) in &current_authorities {
			if !set.insert(id) {
				Err(anyhow!("Duplicate entries found in current authority set"))?
			}
		}

		let latest_height = header.number().into().saturated_into::<u32>();

		Ok(StandaloneClientState {
			current_authorities,
			current_set_id,
			latest_height,
			latest_hash: latest_hash.into(),
			chain_id: self.chain_id,
		})
	}

	/// Returns a tuple of the finality proof for the given parachain `header_numbers` finalized by
	/// `latest_finalized_height`.
	pub async fn query_finality_proof<H>(
		&self,
		latest_finalized_height: u32,
		latest_justification: Option<Vec<u8>>,
	) -> Result<FinalityProof<H>, anyhow::Error>
	where
		H: Header + parity_scale_codec::Decode + Send + 'static,
		<H::Hasher as subxt::config::Hasher>::Output: From<T::Hash>,
		T::Hash: From<<H::Hasher as subxt::config::Hasher>::Output>,
		H::Number: finality_grandpa::BlockNumberOps,
		<<T as subxt::Config>::Header as Header>::Number: One + Clone + Sync + Send,
		<T as subxt::Config>::Header: Decode + Sync,
	{
		let finality_proof = if let Some(justification) = latest_justification {
			let justification = GrandpaJustification::<H>::decode(&mut &*justification)?;

			FinalityProof::<H> {
				block: justification.commit.target_hash,
				justification: justification.encode(),
				unknown_headers: vec![],
			}
		} else {
			let encoded = GrandpaApiClient::<JustificationNotification, H256, u32>::prove_finality(
				// we cast between the same type but different crate versions.
				&*self.ws_client.clone(),
				latest_finalized_height,
			)
			.await?
			.ok_or_else(|| {
				anyhow!("No justification found for block: {:?}", latest_finalized_height)
			})?
			.0;

			let mut finality_proof = FinalityProof::<H>::decode(&mut &encoded[..])?;

			let justification =
				GrandpaJustification::<H>::decode(&mut &finality_proof.justification[..])?;

			finality_proof.block = justification.commit.target_hash;
			finality_proof
		};

		Ok(finality_proof)
	}

	/// Queries the block at which the epoch for the given block belongs to ends.
	pub async fn session_start_and_end_for_block(
		&self,
		block: u32,
	) -> Result<(u32, u32), anyhow::Error> {
		let epoch_addr = T::Storage::babe_epoch_start();
		let block_hash = LegacyRpcMethods::<T>::new(self.rpc_client.clone())
			.chain_get_block_hash(Some(block.into()))
			.await?
			.ok_or_else(|| anyhow!("Failed to fetch block hash for block number {}", block))?;
		let (previous_epoch_start, current_epoch_start) = self
			.client
			.storage()
			.at(block_hash)
			.fetch(&epoch_addr)
			.await?
			.ok_or_else(|| anyhow!("Failed to fetch epoch information"))?;
		Ok((
			current_epoch_start,
			current_epoch_start + (current_epoch_start - previous_epoch_start),
		))
	}

	/// Returns the session length in blocks
	pub async fn session_length(&self) -> Result<u32, anyhow::Error> {
		let metadata = LegacyRpcMethods::<T>::new(self.rpc_client.clone())
			.state_get_metadata(None)
			.await?;
		let metadata = metadata
			.pallet_by_name("Babe")
			.expect("pallet exists")
			.constant_by_name("EpochDuration")
			.expect("constant exists");
		let md = metadata.value().to_vec();
		Ok(Decode::decode(&mut &md[..])?)
	}
}
