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

use crate::error::Error;
// use light_client_common::config::{AsInner, ParaLifecycleT, RuntimeStorage};
use light_client_common::config::{AsInner, ParaLifecycleT, RuntimeStorage};
use mmr_rpc::LeavesProof;
use parity_scale_codec::{Decode, Encode};
use sp_consensus_beefy::{SignedCommitment, VersionedFinalityProof};
use sp_core::{hexdisplay::AsBytesRef, storage::StorageKey, H256};
use sp_runtime::{traits::Zero, SaturatedConversion};
use std::collections::{BTreeMap, BTreeSet};
use subxt::{
	backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
	config::Header,
	rpc_params, Config, OnlineClient,
};

/// This contains the leaf indices of the relay chain blocks and a map of relay chain heights to a
/// map of all parachain headers at those heights Used for generating [`ParaHeadsProof`]
pub struct FinalizedParaHeads {
	/// Block numbers
	pub block_numbers: Vec<u32>,
	/// Map of relay chain heights to map of para ids and parachain headers SCALE-encoded
	pub raw_finalized_heads: BTreeMap<u64, BTreeMap<u32, Vec<u8>>>,
}

/// Get the raw parachain heads finalized in the provided block
pub async fn fetch_finalized_parachain_heads<T: light_client_common::config::Config>(
	client: &RpcClient,
	commitment_block_number: u32,
	latest_beefy_height: u32,
	para_id: u32,
	header_numbers: &BTreeSet<<<T as Config>::Header as Header>::Number>,
) -> Result<FinalizedParaHeads, Error>
where
	<<T as Config>::Header as Header>::Number: Ord + Zero,
	<T as subxt::Config>::Header: Decode,
{
	let legacy_rpc_methods = LegacyRpcMethods::<T>::new(client.clone());
	let subxt_block_number: subxt::backend::legacy::rpc_methods::BlockNumber =
		commitment_block_number.into();
	let block_hash: T::Hash = legacy_rpc_methods
		.chain_get_block_hash(Some(subxt_block_number))
		.await?
		.ok_or_else(|| {
			Error::Custom(format!(
				"Block hash not found for block number {}",
				commitment_block_number
			))
		})?;

	let online_client = OnlineClient::<T>::from_rpc_client(client.clone()).await?;
	let mut para_ids = vec![];
	let key = T::Storage::paras_parachains();
	let ids = online_client
		.storage()
		.at(block_hash)
		.fetch(&key)
		.await?
		.ok_or_else(|| Error::Custom(format!("No ParaIds on relay chain?")))?;
	for id in ids {
		let id = <T::Storage as RuntimeStorage>::Id::from_inner(id.0).into();
		let key = T::Storage::paras_para_lifecycles(id);
		let lifecycle = <T::Storage as RuntimeStorage>::ParaLifecycle::from_inner(
			online_client
				.storage()
				.at(block_hash)
				.fetch(&key)
				.await?
				.expect("ParaId is known"),
		);
		// only care about active parachains.
		if lifecycle.is_parachain() {
			para_ids.push(id);
		}
	}
	let previous_finalized_block_number: subxt::backend::legacy::rpc_methods::BlockNumber =
		(latest_beefy_height + 1).into();
	let previous_finalized_hash = legacy_rpc_methods
		.chain_get_block_hash(Some(previous_finalized_block_number))
		.await?
		.ok_or_else(|| {
			Error::Custom(
				"Failed to get previous finalized beefy block hash from block number".to_string(),
			)
		})?;

	let change_set = legacy_rpc_methods
		.state_query_storage(
			// we are interested only in the blocks where our parachain header changes.
			vec![parachain_header_storage_key(para_id).as_bytes_ref()],
			previous_finalized_hash,
			Some(block_hash),
		)
		.await?;
	let mut finalized_blocks = BTreeMap::new();
	let mut block_numbers = vec![];

	for changes in change_set {
		let header: T::Header =
			legacy_rpc_methods.chain_get_header(Some(changes.block)).await?.ok_or_else(|| {
				Error::Custom(format!(
					"[get_parachain_headers] block not found {:?}",
					changes.block
				))
			})?;

		let mut heads = BTreeMap::new();
		for id in para_ids.iter() {
			let id: u32 = (*id).into();
			let key = T::Storage::paras_heads(id);
			if let Some(head) = online_client.storage().at(header.hash()).fetch(&key).await? {
				heads.insert(
					id,
					Into::<Vec<u8>>::into(<T::Storage as RuntimeStorage>::HeadData::from_inner(
						head,
					)),
				);
			}
		}

		let para_header: T::Header = Decode::decode(&mut &heads[&para_id][..])
			.map_err(|_| Error::Custom(format!("Failed to decode header for {para_id}")))?;
		let para_block_number = para_header.number();
		// skip genesis header or any unknown headers
		if para_block_number == Zero::zero() || !header_numbers.contains(&para_block_number) {
			continue;
		}

		let block_number = header.number().into().saturated_into::<u32>();
		finalized_blocks.insert(block_number as u64, heads);
		block_numbers.push(block_number);
	}

	Ok(FinalizedParaHeads { raw_finalized_heads: finalized_blocks, block_numbers })
}

/// Get beefy justification for latest finalized beefy block
pub async fn fetch_beefy_justification<T: Config>(
	client: &RpcClient,
) -> Result<(SignedCommitment<u32, sp_consensus_beefy::ecdsa_crypto::Signature>, T::Hash), Error> {
	let legacy_rpc_methods = LegacyRpcMethods::<T>::new(client.clone());
	let latest_beefy_finalized: <T as Config>::Hash =
		client.request("beefy_getFinalizedHead", rpc_params!()).await?;
	let block = legacy_rpc_methods
		.chain_get_block(Some(latest_beefy_finalized))
		.await
		.ok()
		.flatten()
		.expect("Should find a valid block");

	let justifications = block.justifications.expect("Block should have valid justifications");

	let beefy_justification = justifications
		.into_iter()
		.find_map(|justfication| {
			(justfication.0 == sp_consensus_beefy::BEEFY_ENGINE_ID).then(|| justfication.1)
		})
		.expect("Should have valid beefy justification");
	let VersionedFinalityProof::V1(signed_commitment) = VersionedFinalityProof::<
		u32,
		sp_consensus_beefy::ecdsa_crypto::Signature,
	>::decode(&mut &*beefy_justification)
	.expect("Beefy justification should decode correctly");

	Ok((signed_commitment, latest_beefy_finalized))
}

/// Query a mmr  proof
pub async fn fetch_mmr_proof<T: Config>(
	client: &RpcClient,
	block_numbers: Vec<u32>,
	block_hash: Option<T::Hash>,
) -> Result<LeavesProof<H256>, Error> {
	let proof: LeavesProof<H256> = client
		.request(
			"mmr_generateProof",
			rpc_params!(block_numbers, Option::<T::Hash>::None, block_hash),
		)
		.await?;
	Ok(proof)
}

/// This returns the storage key under which the parachain header with a given para_id is stored.
pub fn parachain_header_storage_key(para_id: u32) -> StorageKey {
	let mut storage_key = frame_support::storage::storage_prefix(b"Paras", b"Heads").to_vec();
	let encoded_para_id = para_id.encode();
	storage_key.extend_from_slice(sp_core::hashing::twox_64(&encoded_para_id).as_slice());
	storage_key.extend_from_slice(&encoded_para_id);
	StorageKey(storage_key)
}
