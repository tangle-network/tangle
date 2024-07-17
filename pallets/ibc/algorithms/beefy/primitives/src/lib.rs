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

//! Primitive BEEFY types used by verifier and prover

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::all)]
#![deny(missing_docs)]

pub mod error;
use core::marker::PhantomData;
use parity_scale_codec::{Decode, Encode};
use sp_consensus_beefy::mmr::MmrLeafVersion;
pub use sp_consensus_beefy::mmr::{BeefyNextAuthoritySet, MmrLeaf};
use sp_core::H256;
use sp_std::prelude::*;

#[derive(sp_std::fmt::Debug, Encode, Decode, PartialEq, Eq, Clone)]
/// Client state definition for the light client
pub struct ClientState {
	/// Latest beefy height
	pub latest_beefy_height: u32,
	/// Latest mmr root hash
	pub mmr_root_hash: H256,
	/// Authorities for the current session
	pub current_authorities: BeefyNextAuthoritySet<H256>,
	/// Authorities for the next session
	pub next_authorities: BeefyNextAuthoritySet<H256>,
}

/// Host functions that allow the light client perform cryptographic operations in native.
pub trait HostFunctions: light_client_common::HostFunctions {
	/// Keccak 256 hash function
	fn keccak_256(input: &[u8]) -> [u8; 32];

	/// Compressed Ecdsa public key recovery from a signature
	fn secp256k1_ecdsa_recover_compressed(
		signature: &[u8; 65],
		value: &[u8; 32],
	) -> Option<Vec<u8>>;
}

/// Hash length definition for hashing algorithms used
pub const HASH_LENGTH: usize = 32;
/// Authority Signature type
pub type TSignature = [u8; 65];
/// Represents a Hash in this library
pub type Hash = [u8; 32];

#[derive(Clone, sp_std::fmt::Debug, PartialEq, Eq, Encode, Decode)]
/// Authority signature and its index in the signatures array
pub struct SignatureWithAuthorityIndex {
	/// Authority signature
	pub signature: TSignature,
	/// Index in signatures vector
	pub index: u32,
}

#[derive(Clone, sp_std::fmt::Debug, PartialEq, Eq, Encode, Decode)]
/// Signed commitment
pub struct SignedCommitment {
	/// Commitment
	pub commitment: sp_consensus_beefy::Commitment<u32>,
	/// Signatures for this commitment
	pub signatures: Vec<SignatureWithAuthorityIndex>,
}

#[derive(sp_std::fmt::Debug, Clone, PartialEq, Eq, Encode, Decode)]
/// Mmr Update with proof
pub struct MmrUpdateProof {
	/// Signed commitment
	pub signed_commitment: SignedCommitment,
	/// Latest leaf added to mmr
	pub latest_mmr_leaf: MmrLeaf<u32, H256, H256, H256>,
	/// Proof for the latest mmr leaf
	pub mmr_proof: sp_mmr_primitives::Proof<H256>,
	/// Proof for authorities in current session
	pub authority_proof: Vec<Hash>,
}

#[derive(sp_std::fmt::Debug, Clone, PartialEq, Eq, Encode, Decode)]
/// A partial representation of the mmr leaf
pub struct PartialMmrLeaf {
	/// Leaf version
	pub version: MmrLeafVersion,
	/// Parent block number and hash
	pub parent_number_and_hash: (u32, H256),
	/// Next beefy authorities
	pub beefy_next_authority_set: BeefyNextAuthoritySet<H256>,
}

#[derive(sp_std::fmt::Debug, Clone, PartialEq, Eq, Encode, Decode)]
/// Parachain header definition
pub struct ParachainHeader {
	/// scale encoded parachain header
	pub parachain_header: Vec<u8>,
	/// Reconstructed mmr leaf
	pub partial_mmr_leaf: PartialMmrLeaf,
	/// parachain id
	pub para_id: u32,
	/// Proof for our parachain header inclusion in the parachain headers root
	pub parachain_heads_proof: Vec<Hash>,
	/// leaf index for parachain heads proof
	pub heads_leaf_index: u32,
	/// Total number of parachain heads
	pub heads_total_count: u32,
	/// Trie merkle proof of inclusion of the set timestamp extrinsic in header.extrinsic_root
	/// this already encodes the actual extrinsic
	pub extrinsic_proof: Vec<Vec<u8>>,
	/// Timestamp extrinsic
	pub timestamp_extrinsic: Vec<u8>,
}

#[derive(sp_std::fmt::Debug, Clone, PartialEq, Eq, Encode, Decode)]
/// Parachain headers update with proof
pub struct ParachainsUpdateProof {
	/// Parachai headers
	pub parachain_headers: Vec<ParachainHeader>,
	/// Mmr Batch proof for parachain headers
	pub mmr_proof: sp_mmr_primitives::Proof<H256>,
}

#[cfg(feature = "std")]
#[derive(Clone, serde::Serialize, serde::Deserialize)]
/// finality proof
pub struct EncodedVersionedFinalityProof(pub sp_core::Bytes);

/// MMR nodes & size -related utilities.
pub struct NodesUtils {
	no_of_leaves: u64,
}

impl NodesUtils {
	/// Create new instance of MMR nodes utilities for given number of leaves.
	pub fn new(no_of_leaves: u64) -> Self {
		Self { no_of_leaves }
	}

	/// Calculate number of peaks in the MMR.
	pub fn number_of_peaks(&self) -> u64 {
		self.number_of_leaves().count_ones() as u64
	}

	/// Return the number of leaves in the MMR.
	pub fn number_of_leaves(&self) -> u64 {
		self.no_of_leaves
	}

	/// Calculate the total size of MMR (number of nodes).
	pub fn size(&self) -> u64 {
		2 * self.no_of_leaves - self.number_of_peaks()
	}
}

/// Merkle Hasher for mmr library
#[derive(Clone)]
pub struct MerkleHasher<T: HostFunctions>(PhantomData<T>);

impl<T: HostFunctions + Clone> ckb_merkle_mountain_range::Merge for MerkleHasher<T> {
	type Item = H256;

	fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
		let mut concat = left.as_bytes().to_vec();
		concat.extend_from_slice(right.as_bytes());
		T::keccak_256(&*concat).into()
	}
}

impl<T: HostFunctions + Clone> rs_merkle::Hasher for MerkleHasher<T> {
	type Hash = [u8; 32];
	fn hash(data: &[u8]) -> Self::Hash {
		T::keccak_256(data)
	}
}
