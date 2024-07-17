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

//! BEEFY prover utilities

#![allow(clippy::all)]
#![deny(missing_docs)]

/// Errors that can be encountered by the prover
pub mod error;
/// Helper functions and types
pub mod helpers;
/// Methods for querying the relay chain
pub mod relay_chain_queries;

use std::marker::PhantomData;

use beefy_light_client_primitives::{
	ClientState, HostFunctions, MmrUpdateProof, ParachainHeader, PartialMmrLeaf,
};
use error::Error;
use helpers::{
	fetch_timestamp_extrinsic_with_proof, prove_parachain_headers, ParaHeadsProof,
	TimeStampExtWithProof,
};
use hex_literal::hex;
use parity_scale_codec::Decode;
use sp_consensus_beefy::mmr::{BeefyNextAuthoritySet, MmrLeaf};
use sp_core::{hexdisplay::AsBytesRef, keccak_256, H256};
use sp_io::crypto;
use sp_mmr_primitives::Proof;
use sp_runtime::traits::BlakeTwo256;
use subxt::{
	backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
	config::Header,
	rpc_params, Config, OnlineClient,
};

use crate::relay_chain_queries::parachain_header_storage_key;
use light_client_common::config::{AsInner, BeefyAuthoritySetT, RuntimeStorage};
use relay_chain_queries::{fetch_finalized_parachain_heads, fetch_mmr_proof, FinalizedParaHeads};

/// Host function implementation for beefy light client.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Crypto;

impl light_client_common::HostFunctions for Crypto {
	type BlakeTwo256 = BlakeTwo256;
}

impl HostFunctions for Crypto {
	fn keccak_256(input: &[u8]) -> [u8; 32] {
		keccak_256(input)
	}

	fn secp256k1_ecdsa_recover_compressed(
		signature: &[u8; 65],
		value: &[u8; 32],
	) -> Option<Vec<u8>> {
		crypto::secp256k1_ecdsa_recover_compressed(signature, value)
			.ok()
			.map(|val| val.to_vec())
	}
}

/// This contains methods for fetching BEEFY proofs for parachain headers.
pub struct Prover<T: Config> {
	/// Subxt client for the relay chain
	pub relay_rpc_client: RpcClient,
	/// Subxt client for the parachain
	pub para_rpc_client: RpcClient,
	/// Para Id for the associated parachain.
	pub para_id: u32,
	/// Phantom
	pub phantom: PhantomData<T>,
}

impl<T: light_client_common::config::Config> Prover<T> {
	/// Returns the initial state for bootstrapping a BEEFY light client.
	pub async fn get_initial_client_state(client: Option<&RpcClient>) -> ClientState {
		if client.is_none() {
			return ClientState {
				latest_beefy_height: 0,
				mmr_root_hash: Default::default(),
				current_authorities: BeefyNextAuthoritySet {
					id: 0,
					len: 5,
					keyset_commitment: H256::from(hex!(
						"baa93c7834125ee3120bac6e3342bd3f28611110ad21ab6075367abdffefeb09"
					)),
				},
				next_authorities: BeefyNextAuthoritySet {
					id: 1,
					len: 5,
					keyset_commitment: H256::from(hex!(
						"baa93c7834125ee3120bac6e3342bd3f28611110ad21ab6075367abdffefeb09"
					)),
				},
			}
		}
		// Get initial validator set
		// In development mode validators are the same for all sessions only validator set_id
		// changes
		let client = client.expect("Client should be defined");
		let online_client = OnlineClient::<T>::from_rpc_client(client.clone())
			.await
			.expect("OnlineClient should be defined");
		let legacy_rpc_methods = LegacyRpcMethods::<T>::new(client.clone());
		let latest_beefy_finalized: <T as Config>::Hash =
			client.request("beefy_getFinalizedHead", rpc_params!()).await.unwrap();
		let header = legacy_rpc_methods
			.chain_get_header(Some(latest_beefy_finalized))
			.await
			.unwrap()
			.unwrap();
		let validator_set_id = {
			let key = T::Storage::beefy_validator_set_id();
			online_client
				.storage()
				.at(latest_beefy_finalized)
				.fetch(&key)
				.await
				.unwrap()
				.unwrap()
		};

		let next_val_set = {
			let key = T::Storage::mmr_leaf_beefy_next_authorities();
			let data = online_client
				.storage()
				.at(latest_beefy_finalized)
				.fetch(&key)
				.await
				.unwrap()
				.expect("Authorirty set should be defined");
			<T::Storage as RuntimeStorage>::BeefyAuthoritySet::from_inner(data)
		};
		let latest_beefy_height: u64 = (header.number()).into();
		ClientState {
			latest_beefy_height: latest_beefy_height as u32,
			mmr_root_hash: Default::default(),
			current_authorities: BeefyNextAuthoritySet {
				id: validator_set_id,
				len: next_val_set.len(),
				keyset_commitment: next_val_set.root(),
			},
			next_authorities: BeefyNextAuthoritySet {
				id: validator_set_id + 1,
				len: next_val_set.len(),
				keyset_commitment: next_val_set.root(),
			},
		}
	}

	/// Use this fetch all parachain headers finalized at this new
	/// beefy height.
	pub async fn query_finalized_parachain_headers_at(
		&self,
		commitment_block_number: u32,
		latest_beefy_height: u32,
	) -> Result<Vec<T::Header>, Error>
	where
		<<T as Config>::Header as Header>::Number: From<u32>,
		<T as Config>::Header: Decode,
	{
		let subxt_block_number: subxt::backend::legacy::rpc_methods::BlockNumber =
			commitment_block_number.into();
		let online_client =
			OnlineClient::<T>::from_rpc_client(self.relay_rpc_client.clone()).await?;
		let legacy_rpc_methods = LegacyRpcMethods::<T>::new(self.relay_rpc_client.clone());
		let block_hash = legacy_rpc_methods.chain_get_block_hash(Some(subxt_block_number)).await?;
		let previous_finalized_block_number: subxt::backend::legacy::rpc_methods::BlockNumber =
			(latest_beefy_height + 1).into();
		let previous_finalized_hash = legacy_rpc_methods
			.chain_get_block_hash(Some(previous_finalized_block_number))
			.await?
			.ok_or_else(|| {
				Error::Custom(
					"Failed to get previous finalized beefy block hash from block number"
						.to_string(),
				)
			})?;

		let change_set = legacy_rpc_methods
			.state_query_storage(
				// we are interested only in the blocks where our parachain header changes.
				vec![parachain_header_storage_key(self.para_id).as_bytes_ref()],
				previous_finalized_hash,
				block_hash,
			)
			.await?;
		let mut headers = vec![];
		for changes in change_set {
			let header = legacy_rpc_methods
				.chain_get_header(Some(changes.block))
				.await?
				.ok_or_else(|| {
					Error::Custom(format!(
						"[get_parachain_headers] block not found {:?}",
						changes.block
					))
				})?;

			let key = T::Storage::paras_heads(self.para_id);
			let head = <<T::Storage as RuntimeStorage>::HeadData as AsInner>::from_inner(
				online_client
					.storage()
					.at(header.hash())
					.fetch(&key)
					.await?
					.expect("Header exists in its own changeset; qed"),
			);

			let para_header = T::Header::decode(&mut head.as_ref())
				.map_err(|_| Error::Custom(format!("Failed to decode header")))?;
			headers.push(para_header);
		}

		Ok(headers)
	}

	/// This will query the finalized parachain headers between the given relay chain blocks
	/// Only including the given parachain headers into the [`Proof`]
	pub async fn query_finalized_parachain_headers_with_proof(
		&self,
		commitment_block_number: u32,
		latest_beefy_height: u32,
		header_numbers: Vec<<<T as Config>::Header as Header>::Number>,
	) -> Result<(Vec<ParachainHeader>, Proof<H256>), Error>
	where
		<<T as Config>::Header as Header>::Number: Ord + sp_runtime::traits::Zero,
		<T as subxt::Config>::Header: Decode,
	{
		let legacy_rpc_methods = LegacyRpcMethods::<T>::new(self.relay_rpc_client.clone());
		let header_numbers = header_numbers.into_iter().map(From::from).collect();
		let FinalizedParaHeads { block_numbers, raw_finalized_heads: finalized_blocks } =
			fetch_finalized_parachain_heads::<T>(
				&self.relay_rpc_client,
				commitment_block_number,
				latest_beefy_height,
				self.para_id,
				&header_numbers,
			)
			.await?;

		let subxt_block_number: subxt::backend::legacy::rpc_methods::BlockNumber =
			commitment_block_number.into();
		let block_hash = legacy_rpc_methods.chain_get_block_hash(Some(subxt_block_number)).await?;

		let batch_proof =
			fetch_mmr_proof::<T>(&self.relay_rpc_client, block_numbers, block_hash).await?;

		let leaves: Vec<Vec<u8>> = Decode::decode(&mut &*batch_proof.leaves.to_vec())?;

		let mut parachain_headers = vec![];
		for leaf_bytes in leaves {
			let leaf: MmrLeaf<u32, H256, H256, H256> = Decode::decode(&mut &*leaf_bytes)?;
			let parent_block: u32 = leaf.parent_number_and_hash.0.into();
			let leaf_block_number = (parent_block + 1) as u64;
			let para_headers = finalized_blocks.get(&leaf_block_number).ok_or_else(|| {
				Error::Custom(format!(
					"[get_parachain_headers] Para Headers not found for relay chain block {}",
					leaf_block_number
				))
			})?;
			let ParaHeadsProof {
				parachain_heads_proof,
				para_head,
				heads_leaf_index,
				heads_total_count,
			} = prove_parachain_headers(&para_headers, self.para_id)?;

			let decoded_para_head = T::Header::decode(&mut &para_head[..])?;
			let TimeStampExtWithProof { ext: timestamp_extrinsic, proof: extrinsic_proof } =
				fetch_timestamp_extrinsic_with_proof::<T>(
					&self.para_rpc_client,
					Some(decoded_para_head.hash()),
				)
				.await?;

			let header = ParachainHeader {
				parachain_header: para_head,
				partial_mmr_leaf: PartialMmrLeaf {
					version: leaf.version,
					parent_number_and_hash: leaf.parent_number_and_hash,
					beefy_next_authority_set: leaf.beefy_next_authority_set.clone(),
				},
				para_id: self.para_id,
				parachain_heads_proof,
				heads_leaf_index,
				heads_total_count,
				extrinsic_proof,
				timestamp_extrinsic,
			};

			parachain_headers.push(header);
		}

		let batch_proof: Proof<H256> = Decode::decode(&mut batch_proof.proof.0.as_slice())?;
		Ok((parachain_headers, batch_proof))
	}

	/// This will fetch the latest leaf in the mmr as well as a proof for this leaf in the latest
	/// mmr root hash.
	pub async fn fetch_mmr_update_proof_for(
		&self,
		_signed_commitment: sp_consensus_beefy::SignedCommitment<
			u32,
			sp_consensus_beefy::ecdsa_crypto::Signature,
		>,
	) -> Result<MmrUpdateProof, Error> {
		todo!("fetch beefy authorities")
		/*
		let subxt_block_number: subxt::backend::legacy::rpc_methods::BlockNumber =
			signed_commitment.commitment.block_number.into();
		let block_hash =
			legacy_rpc_methods.chain_get_block_hash(Some(subxt_block_number)).await?.ok_or_else(
				|| {
					Error::Custom(format!(
						"Failed to fetch block hash for block number {}",
						signed_commitment.commitment.block_number,
					))
				},
			)?;

		let current_authorities: Vec<Public> = {
			let key = T::Storage::beefy_authorities();
			self.relay_rpc_client
				.storage()
				.at(block_hash)
				.fetch(&key)
				.await?
				.ok_or_else(|| Error::Custom(format!("No beefy authorities found!")))?
		};

		// Current LeafIndex
		let block_number = signed_commitment.commitment.block_number;
		let leaf_proof =
			fetch_mmr_proof(&self.relay_rpc_client, vec![block_number.into()], Some(block_hash))
				.await?;
		let leaves: Vec<Vec<u8>> = parity_scale_codec::Decode::decode(&mut &*leaf_proof.leaves.0)?;
		let latest_leaf: MmrLeaf<u32, H256, H256, H256> = parity_scale_codec::Decode::decode(&mut &*leaves[0])?;
		let mmr_proof: sp_mmr_primitives::Proof<H256> =
			parity_scale_codec::Decode::decode(&mut &*leaf_proof.proof.0)?;

		let authority_address_hashes = hash_authority_addresses(
			current_authorities.into_iter().map(|x| x.encode()).collect(),
		)?;

		let AuthorityProofWithSignatures { authority_proof, signatures } =
			prove_authority_set(&signed_commitment, authority_address_hashes)?;

		Ok(MmrUpdateProof {
			signed_commitment: SignedCommitment {
				commitment: signed_commitment.commitment.clone(),
				signatures,
			},
			latest_mmr_leaf: latest_leaf.clone(),
			mmr_proof,
			authority_proof,
		})
		 */
	}

	/// Construct a beefy client state to be submitted to the counterparty chain
	pub async fn construct_beefy_client_state(&self) -> Result<ClientState, Error> {
		todo!("fetch beefy authorities")
		/*
		let (signed_commitment, latest_beefy_finalized) =
			fetch_beefy_justification(&self.relay_rpc_client).await?;

		// Encoding and decoding to fix dependency version conflicts
		let next_authority_set = {
			let key = T::Storage::mmr_leaf_beefy_next_authorities();
			self.relay_rpc_client
				.storage()
				.at(latest_beefy_finalized)
				.fetch(&key)
				.await?
				.expect("Should retrieve next authority set")
				.encode()
		};
		let next_authority_set = BeefyNextAuthoritySet::decode(&mut &*next_authority_set)
			.expect("Should decode next authority set correctly");

		let current_authorities: Vec<Public> = {
			let key = T::Storage::beefy_authorities();
			self.relay_rpc_client
				.storage()
				.at(latest_beefy_finalized)
				.fetch(&key)
				.await?
				.expect("Should retrieve next authority set")
		};

		let authority_address_hashes = hash_authority_addresses(
			current_authorities.into_iter().map(|x| x.encode()).collect(),
		)?;
		let tree =
			rs_merkle::MerkleTree::<MerkleHasher<Crypto>>::from_leaves(&authority_address_hashes);

		let authority_root = tree.root().expect("Should generate root");
		let authority_root: H256 = authority_root.into();
		let current_authority_set = BeefyNextAuthoritySet {
			id: next_authority_set.id - 1,
			len: authority_address_hashes.len() as u32,
			root: authority_root,
		};

		let mmr_root_hash = signed_commitment
			.commitment
			.payload
			.get_decoded::<H256>(&MMR_ROOT_ID)
			.expect("Mmr root hash should decode correctly");

		let client_state = ClientState {
			mmr_root_hash,
			latest_beefy_height: signed_commitment.commitment.block_number as u32,
			current_authorities: current_authority_set.clone(),
			next_authorities: next_authority_set.clone(),
		};

		Ok(client_state)
		 */
	}
}
