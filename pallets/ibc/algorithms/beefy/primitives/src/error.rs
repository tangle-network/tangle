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

//! Light client error definition

use parity_scale_codec::alloc::string::String;
use sp_core::H256;
use sp_std::prelude::*;

#[derive(sp_std::fmt::Debug, derive_more::From, derive_more::Display)]
/// Error definition for the light client
pub enum BeefyClientError {
	/// Outdated commitment
	#[from(ignore)]
	#[display(
		fmt = "OutdatedCommitment: latest_beefy_height {}, commitment_block_number {}",
		latest_beefy_height,
		commitment_block_number
	)]
	OutdatedCommitment {
		/// Latest beefy height stored
		latest_beefy_height: u32,
		/// Commitment block number received
		commitment_block_number: u32,
	},
	/// Expected proof for latest leaf, found a batch proof
	ExpectedSingleLeafIndex,
	/// Mmr root hash not found in commitment
	MmrRootHashNotFound,
	/// Invalid Authority set id received
	#[from(ignore)]
	#[display(
		fmt = "AuthoritySetMismatch: current_set_id {}, next_set_id {}, commitment_set_id {}",
		current_set_id,
		next_set_id,
		commitment_set_id
	)]
	AuthoritySetMismatch {
		/// Current authority set id
		current_set_id: u64,
		/// Next authority set id
		next_set_id: u64,
		/// Authority set id in commitment
		commitment_set_id: u64,
	},
	/// Incomplete Signature threshold
	IncompleteSignatureThreshold,
	/// Error recovering public key from signature
	InvalidSignature,
	/// Some invalid merkle root hash
	#[from(ignore)]
	#[display(fmt = "InvalidRootHash with len: {}", len)]
	InvalidRootHash {
		/// Root hash
		root_hash: Vec<u8>,
		/// Root hash length
		len: u64,
	},
	/// Some invalid mmr proof
	#[display(
		fmt = "InvalidMmrProof, expected: {}, found: {}, during: {}",
		expected,
		found,
		location
	)]
	InvalidMmrProof {
		/// The root hash that was expected
		expected: H256,
		/// The root hash that was derived
		found: H256,
		/// The location of the error
		location: &'static str,
	},
	/// Invalid authority proof
	InvalidAuthorityProof,
	/// Invalid merkle proof
	InvalidMerkleProof,
	/// Mmr Error
	MmrVerificationError(ckb_merkle_mountain_range::Error),
	/// Codec error
	Codec(parity_scale_codec::Error),
	/// Custom error
	Custom(String),
}
