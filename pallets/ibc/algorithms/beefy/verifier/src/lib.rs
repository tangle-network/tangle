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

//! BEEFY light client verification functions
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::all)]
#![deny(missing_docs)]

extern crate alloc;

#[cfg(test)]
mod tests;

use beefy_light_client_primitives::{
	error::BeefyClientError, BeefyNextAuthoritySet, ClientState, HostFunctions, MerkleHasher,
	MmrUpdateProof, NodesUtils, ParachainsUpdateProof, SignatureWithAuthorityIndex, HASH_LENGTH,
};
use parity_scale_codec::{Decode, Encode};
use sp_consensus_beefy::{known_payloads::MMR_ROOT_ID, mmr::MmrLeaf};
use sp_core::H256;
use sp_runtime::traits::Convert;

use alloc::{format, string::ToString};
use sp_core::ByteArray;
use sp_runtime::{generic::Header, traits::BlakeTwo256};
use sp_std::{prelude::*, vec};
use sp_trie::LayoutV0;

/// This should verify the signed commitment signatures, and reconstruct the
/// authority merkle root, confirming known authorities signed the [`crate::primitives::Commitment`]
/// then using the mmr proofs, verify the latest mmr leaf,
/// using the latest mmr leaf to rotate its view of the next authorities.
pub fn verify_mmr_root_with_proof<H>(
	mut trusted_client_state: ClientState,
	mmr_update: MmrUpdateProof,
) -> Result<ClientState, BeefyClientError>
where
	H: HostFunctions + Clone,
{
	let current_authority_set = &trusted_client_state.current_authorities;
	let next_authority_set = &trusted_client_state.next_authorities;
	let signatures_len = mmr_update.signed_commitment.signatures.len();
	let validator_set_id = mmr_update.signed_commitment.commitment.validator_set_id;

	// If signature threshold is not satisfied, return
	if !validate_sigs_against_threshold(current_authority_set, signatures_len)
		&& !validate_sigs_against_threshold(next_authority_set, signatures_len)
	{
		return Err(BeefyClientError::IncompleteSignatureThreshold);
	}

	if current_authority_set.id != validator_set_id && next_authority_set.id != validator_set_id {
		return Err(BeefyClientError::AuthoritySetMismatch {
			current_set_id: current_authority_set.id,
			next_set_id: next_authority_set.id,
			commitment_set_id: validator_set_id,
		});
	}

	// Extract root hash from signed commitment and validate it
	let mmr_root_vec = {
		if let Some(root) = mmr_update.signed_commitment.commitment.payload.get_raw(&MMR_ROOT_ID) {
			if root.len() == HASH_LENGTH {
				root
			} else {
				return Err(BeefyClientError::InvalidRootHash {
					root_hash: root.clone(),
					len: root.len() as u64,
				});
			}
		} else {
			return Err(BeefyClientError::MmrRootHashNotFound);
		}
	};

	let mmr_root_hash = H256::from_slice(&*mmr_root_vec);

	// Beefy validators sign the keccak_256 hash of the scale encoded commitment
	let encoded_commitment = mmr_update.signed_commitment.commitment.encode();
	let commitment_hash = H::keccak_256(&*encoded_commitment);

	let mut authority_indices = Vec::new();
	let authority_leaves = mmr_update
		.signed_commitment
		.signatures
		.into_iter()
		.map(|SignatureWithAuthorityIndex { index, signature }| {
			H::secp256k1_ecdsa_recover_compressed(&signature, &commitment_hash)
				.and_then(|public_key_bytes| {
					sp_consensus_beefy::ecdsa_crypto::AuthorityId::from_slice(&public_key_bytes)
						.ok()
				})
				.map(|pub_key| {
					authority_indices.push(index as usize);
					H::keccak_256(&pallet_beefy_mmr::BeefyEcdsaToEthereum::convert(pub_key))
				})
				.ok_or(BeefyClientError::InvalidSignature)
		})
		.collect::<Result<Vec<_>, BeefyClientError>>()?;

	let mut authorities_changed = false;

	let authorities_merkle_proof =
		rs_merkle::MerkleProof::<MerkleHasher<H>>::new(mmr_update.authority_proof);
	// Verify mmr_update.authority_proof against store root hash
	match validator_set_id {
		id if id == current_authority_set.id => {
			let root_hash = current_authority_set.keyset_commitment;
			if !authorities_merkle_proof.verify(
				root_hash.into(),
				&authority_indices,
				&authority_leaves,
				current_authority_set.len as usize,
			) {
				return Err(BeefyClientError::InvalidAuthorityProof);
			}
		},
		id if id == next_authority_set.id => {
			let root_hash = next_authority_set.keyset_commitment;
			if !authorities_merkle_proof.verify(
				root_hash.into(),
				&authority_indices,
				&authority_leaves,
				next_authority_set.len as usize,
			) {
				return Err(BeefyClientError::InvalidAuthorityProof);
			}
			authorities_changed = true;
		},
		_ => {
			return Err(BeefyClientError::AuthoritySetMismatch {
				current_set_id: current_authority_set.id,
				next_set_id: next_authority_set.id,
				commitment_set_id: validator_set_id,
			})
		},
	}

	let latest_beefy_height = trusted_client_state.latest_beefy_height;

	let commitment_block_number = mmr_update.signed_commitment.commitment.block_number;
	if commitment_block_number <= latest_beefy_height {
		return Err(BeefyClientError::OutdatedCommitment {
			latest_beefy_height,
			commitment_block_number,
		});
	}

	// Move on to verify mmr_proof
	let node = mmr_update.latest_mmr_leaf.using_encoded(|leaf| H::keccak_256(leaf));

	let mmr_size = NodesUtils::new(mmr_update.mmr_proof.leaf_count).size();
	let proof = ckb_merkle_mountain_range::MerkleProof::<_, MerkleHasher<H>>::new(
		mmr_size,
		mmr_update.mmr_proof.items,
	);

	// We are trying to verify the proof for the latest mmr leaf so we expect the proof to contain a
	// singular leaf index
	let leaf_index = mmr_update
		.mmr_proof
		.leaf_indices
		.get(0)
		.ok_or(BeefyClientError::ExpectedSingleLeafIndex)?;

	let leaf_pos = ckb_merkle_mountain_range::leaf_index_to_pos(*leaf_index);

	let root = proof.calculate_root(vec![(leaf_pos, node.into())])?;
	if root != mmr_root_hash {
		return Err(BeefyClientError::InvalidMmrProof {
			expected: mmr_root_hash,
			found: root,
			location: "verifying_latest_mmr_leaf",
		});
	}

	trusted_client_state.latest_beefy_height = mmr_update.signed_commitment.commitment.block_number;
	trusted_client_state.mmr_root_hash = mmr_root_hash;

	if authorities_changed {
		trusted_client_state.current_authorities = next_authority_set.clone();
		trusted_client_state.next_authorities = mmr_update.latest_mmr_leaf.beefy_next_authority_set;
	}
	Ok(trusted_client_state)
}

/// Takes the updated client state and parachains headers update proof
/// and verifies inclusion in mmr
pub fn verify_parachain_headers<H>(
	trusted_client_state: ClientState,
	ParachainsUpdateProof { mmr_proof, parachain_headers }: ParachainsUpdateProof,
) -> Result<(), BeefyClientError>
where
	H: HostFunctions + Clone,
{
	let mut mmr_leaves = Vec::new();

	if mmr_proof.leaf_indices.len() != parachain_headers.len() {
		Err(BeefyClientError::Custom(
			"leaf indices is not equal to number of parachain headers".to_string(),
		))?
	}

	for (index, parachain_header) in parachain_headers.into_iter().enumerate() {
		let decoded_para_header =
			Header::<u32, BlakeTwo256>::decode(&mut &*parachain_header.parachain_header)?;

		// just to be safe skip genesis block if it's included, it has no timestamp
		if decoded_para_header.number == 0 {
			Err(BeefyClientError::Custom(
				"Genesis block found, it should not be included".to_string(),
			))?
		}

		// Verify timestamp extrinsic
		// Timestamp extrinsic should be the first inherent and hence the first extrinsic
		// https://github.com/paritytech/substrate/blob/d602397a0bbb24b5d627795b797259a44a5e29e9/primitives/trie/src/lib.rs#L99-L101
		let timestamp_ext_key = parity_scale_codec::Compact(0u64).encode();
		sp_trie::verify_trie_proof::<LayoutV0<H::BlakeTwo256>, _, _, _>(
			&decoded_para_header.extrinsics_root,
			&&*parachain_header.extrinsic_proof,
			&vec![(timestamp_ext_key, Some(&*parachain_header.timestamp_extrinsic))],
		)
		.map_err(|_| BeefyClientError::Custom(format!("Invalid extrinsic proof")))?;

		let pair = (parachain_header.para_id, parachain_header.parachain_header);
		let leaf_bytes = pair.encode();

		let proof =
			rs_merkle::MerkleProof::<MerkleHasher<H>>::new(parachain_header.parachain_heads_proof);
		let leaf_hash = H::keccak_256(&leaf_bytes);
		let root = proof
			.root(
				&[parachain_header.heads_leaf_index as usize],
				&[leaf_hash],
				parachain_header.heads_total_count as usize,
			)
			.map_err(|_| BeefyClientError::InvalidMerkleProof)?;
		// reconstruct leaf
		let mmr_leaf = MmrLeaf {
			version: parachain_header.partial_mmr_leaf.version,
			parent_number_and_hash: parachain_header.partial_mmr_leaf.parent_number_and_hash,
			beefy_next_authority_set: parachain_header.partial_mmr_leaf.beefy_next_authority_set,
			leaf_extra: H256::from_slice(&root),
		};

		let node = mmr_leaf.using_encoded(|leaf| H::keccak_256(leaf));
		let leaf_index = mmr_proof.leaf_indices[index];
		let leaf_pos = ckb_merkle_mountain_range::leaf_index_to_pos(leaf_index);
		mmr_leaves.push((leaf_pos, H256::from_slice(&node)));
	}

	let mmr_size = NodesUtils::new(mmr_proof.leaf_count).size();
	let proof = ckb_merkle_mountain_range::MerkleProof::<_, MerkleHasher<H>>::new(
		mmr_size,
		mmr_proof.items,
	);

	let root = proof.calculate_root(mmr_leaves)?;
	if root != trusted_client_state.mmr_root_hash {
		return Err(BeefyClientError::InvalidMmrProof {
			expected: trusted_client_state.mmr_root_hash,
			found: root,
			location: "verifying_parachain_headers_inclusion",
		});
	}
	Ok(())
}

/// Validate signatures against threshold
fn validate_sigs_against_threshold(set: &BeefyNextAuthoritySet<H256>, sigs_len: usize) -> bool {
	let threshold = ((2 * set.len) / 3) + 1;
	sigs_len >= threshold as usize
}
