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

use tendermint_proto::Protobuf;

use crate::{
	error::Error,
	proto::{
		client_message, BeefyAuthoritySet as RawBeefyAuthoritySet, BeefyMmrLeaf as RawBeefyMmrLeaf,
		BeefyMmrLeafPartial as RawBeefyMmrLeafPartial, ClientMessage as RawClientMessage,
		ClientStateUpdateProof as RawMmrUpdateProof, Commitment as RawCommitment,
		CommitmentSignature, ConsensusStateUpdateProof, Header as RawBeefyHeader,
		Misbehaviour as RawMisbehaviour, PayloadItem, SignedCommitment as RawSignedCommitment,
	},
};
use alloc::{format, vec, vec::Vec};
use anyhow::anyhow;
use beefy_light_client_primitives::{
	BeefyNextAuthoritySet, Hash, MmrUpdateProof, PartialMmrLeaf, SignatureWithAuthorityIndex,
	SignedCommitment,
};
use parity_scale_codec::{Decode, Encode};
use sp_consensus_beefy::{
	known_payloads::MMR_ROOT_ID,
	mmr::{MmrLeaf, MmrLeafVersion},
	Commitment, Payload,
};
use sp_core::H256;
use sp_mmr_primitives::Proof;
use sp_runtime::{
	generic::Header as SubstrateHeader,
	traits::{BlakeTwo256, SaturatedConversion},
};

/// Protobuf type url for Beefy header
pub const BEEFY_CLIENT_MESSAGE_TYPE_URL: &str = "/ibc.lightclients.beefy.v1.ClientMessage";

/// Beefy consensus header
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BeefyHeader {
	pub headers_with_proof: Option<ParachainHeadersWithProof>,
	pub mmr_update_proof: Option<MmrUpdateProof>, // Proof for updating the latest mmr root hash
}

/// [`ClientMessage`] for ICS11-BEEFY
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ClientMessage {
	/// Header variant for updating the client
	Header(BeefyHeader),
	/// Misbehaviour variant for freezing the client.
	Misbehaviour(()),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParachainHeadersWithProof {
	pub headers: Vec<ParachainHeader>, // contains the parachain headers
	pub mmr_proofs: Vec<Vec<u8>>,      // mmr proofs for these headers
	pub mmr_size: u64,                 // The latest mmr size
	pub leaf_indices: Vec<u64>,        // The mmr leaf indices for the proof
	pub leaf_count: u64,               // The mmr leaf count when the proof was generated
}

impl ibc::core::ics02_client::client_message::ClientMessage for ClientMessage {
	fn encode_to_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error> {
		self.encode_vec()
	}
}

#[derive(Clone, PartialEq, Eq, Debug, parity_scale_codec::Encode, parity_scale_codec::Decode)]
pub struct ParachainHeader {
	pub parachain_header: SubstrateHeader<u32, BlakeTwo256>,
	/// Reconstructed mmr leaf
	pub partial_mmr_leaf: PartialMmrLeaf,
	/// Proof for our parachain header inclusion in the parachain headers root
	pub parachain_heads_proof: Vec<Hash>,
	/// leaf index for parachain heads proof
	pub heads_leaf_index: u32,
	/// Total number of parachain heads
	pub heads_total_count: u32,
	/// Trie merkle proof of inclusion of the set timestamp extrinsic in header.extrinsic_root
	pub extrinsic_proof: Vec<Vec<u8>>,
	/// this already encodes the actual extrinsic
	pub timestamp_extrinsic: Vec<u8>,
}

pub fn split_leaf_version(version: u8) -> (u8, u8) {
	let major = version >> 5;
	let minor = version & 0b11111;
	(major, minor)
}

pub fn merge_leaf_version(major: u8, minor: u8) -> u8 {
	(major << 5) + minor
}

impl TryFrom<RawClientMessage> for ClientMessage {
	type Error = Error;

	fn try_from(msg: RawClientMessage) -> Result<Self, Self::Error> {
		let message = match msg
			.message
			.ok_or_else(|| anyhow!("Must supply either Header or Misbehaviour type!"))?
		{
			client_message::Message::Header(raw_header) => {
				let headers_with_proof = raw_header
					.consensus_state
					.map(|consensus_update| {
						let parachain_headers = consensus_update
							.parachain_headers
							.into_iter()
							.map(|raw_para_header| {
								let mmr_partial_leaf =
									raw_para_header.mmr_leaf_partial.ok_or_else(|| {
										Error::Custom(format!(
											"Invalid header, missing mmr_leaf_partial"
										))
									})?;
								let parent_hash =
									H256::decode(&mut mmr_partial_leaf.parent_hash.as_slice())
										.unwrap();
								let beefy_next_authority_set = if let Some(next_set) =
									mmr_partial_leaf.beefy_next_authority_set
								{
									BeefyNextAuthoritySet {
										id: next_set.id,
										len: next_set.len,
										keyset_commitment: H256::decode(
											&mut next_set.authority_root.as_slice(),
										)?,
									}
								} else {
									Default::default()
								};
								Ok(ParachainHeader {
									parachain_header: SubstrateHeader::decode(
										&mut &raw_para_header.parachain_header[..],
									)?,
									partial_mmr_leaf: PartialMmrLeaf {
										version: {
											let (major, minor) = split_leaf_version(
												mmr_partial_leaf.version.saturated_into::<u8>(),
											);
											MmrLeafVersion::new(major, minor)
										},
										parent_number_and_hash: (
											mmr_partial_leaf.parent_number,
											parent_hash,
										),
										beefy_next_authority_set,
									},
									parachain_heads_proof: raw_para_header
										.parachain_heads_proof
										.into_iter()
										.map(|item| {
											let mut dest = [0u8; 32];
											if item.len() != 32 {
												return Err(Error::Custom(format!(
													"Invalid proof item with len {}",
													item.len()
												)));
											}
											dest.copy_from_slice(&*item);
											Ok(dest)
										})
										.collect::<Result<Vec<_>, Error>>()?,
									heads_leaf_index: raw_para_header.heads_leaf_index,
									heads_total_count: raw_para_header.heads_total_count,
									extrinsic_proof: raw_para_header.extrinsic_proof,
									timestamp_extrinsic: raw_para_header.timestamp_extrinsic,
								})
							})
							.collect::<Result<Vec<_>, Error>>()
							.ok();
						parachain_headers.map(|parachain_headers| ParachainHeadersWithProof {
							headers: parachain_headers,
							mmr_proofs: consensus_update.mmr_proofs,
							mmr_size: consensus_update.mmr_size,
							leaf_indices: consensus_update.leaf_indices,
							leaf_count: consensus_update.leaf_count,
						})
					})
					.flatten();

				let mmr_update_proof = if let Some(mmr_update) = raw_header.client_state {
					let commitment = mmr_update
						.signed_commitment
						.as_ref()
						.ok_or_else(|| Error::Custom(format!("Signed commitment is missing")))?
						.commitment
						.as_ref()
						.ok_or_else(|| Error::Custom(format!("Commitment is missing")))?;
					let payload = {
						commitment
							.payload
							.iter()
							.filter_map(|item| {
								if item.payload_id.as_slice() != MMR_ROOT_ID {
									return None;
								}
								let mut payload_id = [0u8; 2];
								payload_id.copy_from_slice(&item.payload_id);
								Some(Payload::from_single_entry(
									payload_id,
									item.payload_data.clone(),
								))
							})
							.collect::<Vec<_>>()
							.get(0)
							.ok_or_else(|| {
								Error::Custom(format!("Invalid payload, missing mmr root hash"))
							})?
							.clone()
					};
					let block_number = commitment.block_numer;
					let validator_set_id = commitment.validator_set_id;
					let signatures = mmr_update
						.signed_commitment
						.ok_or_else(|| Error::Custom(format!("Signed Commiment is missing")))?
						.signatures
						.into_iter()
						.map(|commitment_sig| {
							if commitment_sig.signature.len() != 65 {
								return Err(Error::Custom(format!(
									"Invalid signature length: {}",
									commitment_sig.signature.len()
								)));
							}
							Ok(SignatureWithAuthorityIndex {
								signature: {
									let mut sig = [0u8; 65];
									sig.copy_from_slice(&commitment_sig.signature);
									sig
								},
								index: commitment_sig.authority_index,
							})
						})
						.collect::<Result<Vec<_>, Error>>()?;

					let mmr_leaf = mmr_update
						.mmr_leaf
						.as_ref()
						.ok_or_else(|| Error::Custom(format!("Mmr Leaf is missing")))?;
					let beefy_next_authority_set =
						mmr_leaf.beefy_next_authority_set.as_ref().ok_or_else(|| {
							Error::Custom(format!("Beefy Next Authority set is missing"))
						})?;

					Some(MmrUpdateProof {
						signed_commitment: SignedCommitment {
							commitment: Commitment { payload, block_number, validator_set_id },
							signatures,
						},
						latest_mmr_leaf: MmrLeaf {
							version: {
								let (major, minor) =
									split_leaf_version(mmr_leaf.version.saturated_into::<u8>());
								MmrLeafVersion::new(major, minor)
							},
							parent_number_and_hash: {
								let parent_number = mmr_leaf.parent_number;
								let parent_hash =
									H256::decode(&mut mmr_leaf.parent_hash.as_slice())
										.map_err(|e| Error::Custom(format!("{e}")))?;
								(parent_number, parent_hash)
							},
							beefy_next_authority_set: BeefyNextAuthoritySet {
								id: beefy_next_authority_set.id,
								len: beefy_next_authority_set.len,
								keyset_commitment: H256::decode(
									&mut beefy_next_authority_set.authority_root.as_slice(),
								)
								.map_err(|e| Error::Custom(format!("{e}")))?,
							},
							leaf_extra: H256::decode(&mut mmr_leaf.parachain_heads.as_slice())
								.map_err(|e| Error::Custom(format!("{e}")))?,
						},
						mmr_proof: Proof {
							leaf_indices: vec![mmr_update.mmr_leaf_index], // TODO(blas): fix this
							leaf_count: mmr_update.mmr_leaf_index + 1,
							items: mmr_update
								.mmr_proof
								.into_iter()
								.map(|item| {
									H256::decode(&mut &*item)
										.map_err(|e| Error::Custom(format!("{e}")))
								})
								.collect::<Result<Vec<_>, Error>>()?,
						},
						authority_proof: mmr_update
							.authorities_proof
							.into_iter()
							.map(|item| {
								if item.len() != 32 {
									return Err(Error::Custom(format!(
										"Invalid authorities proof item with len: {}",
										item.len()
									)));
								}
								let mut dest = [0u8; 32];
								dest.copy_from_slice(&item);
								Ok(dest)
							})
							.collect::<Result<Vec<_>, Error>>()?,
					})
				} else {
					None
				};

				ClientMessage::Header(BeefyHeader { headers_with_proof, mmr_update_proof })
			},
			client_message::Message::Misbehaviour(_) => ClientMessage::Misbehaviour(()),
		};

		Ok(message)
	}
}

impl From<ClientMessage> for RawClientMessage {
	fn from(client_message: ClientMessage) -> Self {
		match client_message {
			ClientMessage::Header(beefy_header) => RawClientMessage {
				message: Some(client_message::Message::Header(RawBeefyHeader {
					consensus_state: beefy_header.headers_with_proof.map(|headers| {
						let parachain_headers = headers
							.headers
							.into_iter()
							.map(|para_header| crate::proto::ParachainHeader {
								parachain_header: para_header.parachain_header.encode(),
								mmr_leaf_partial: Some(RawBeefyMmrLeafPartial {
									version: {
										let (major, minor) =
											para_header.partial_mmr_leaf.version.split();
										merge_leaf_version(major, minor) as u32
									},
									parent_number: para_header
										.partial_mmr_leaf
										.parent_number_and_hash
										.0,
									parent_hash: para_header
										.partial_mmr_leaf
										.parent_number_and_hash
										.1
										.encode(),
									beefy_next_authority_set: Some(RawBeefyAuthoritySet {
										id: para_header
											.partial_mmr_leaf
											.beefy_next_authority_set
											.id,
										len: para_header
											.partial_mmr_leaf
											.beefy_next_authority_set
											.len,
										authority_root: para_header
											.partial_mmr_leaf
											.beefy_next_authority_set
											.keyset_commitment
											.encode(),
									}),
								}),
								parachain_heads_proof: para_header
									.parachain_heads_proof
									.into_iter()
									.map(|item| item.to_vec())
									.collect(),
								heads_leaf_index: para_header.heads_leaf_index,
								heads_total_count: para_header.heads_total_count,
								extrinsic_proof: para_header.extrinsic_proof,
								timestamp_extrinsic: para_header.timestamp_extrinsic,
							})
							.collect();
						ConsensusStateUpdateProof {
							parachain_headers,
							mmr_proofs: headers.mmr_proofs,
							mmr_size: headers.mmr_size,
							leaf_indices: headers.leaf_indices,
							leaf_count: headers.leaf_count,
						}
					}),
					client_state: if let Some(mmr_update) = beefy_header.mmr_update_proof {
						Some(RawMmrUpdateProof {
							mmr_leaf: Some(RawBeefyMmrLeaf {
								version: {
									let (major, minor) = mmr_update.latest_mmr_leaf.version.split();
									merge_leaf_version(major, minor) as u32
								},
								parent_number: mmr_update.latest_mmr_leaf.parent_number_and_hash.0,
								parent_hash: mmr_update
									.latest_mmr_leaf
									.parent_number_and_hash
									.1
									.encode(),
								beefy_next_authority_set: Some(RawBeefyAuthoritySet {
									id: mmr_update.latest_mmr_leaf.beefy_next_authority_set.id,
									len: mmr_update.latest_mmr_leaf.beefy_next_authority_set.len,
									authority_root: mmr_update
										.latest_mmr_leaf
										.beefy_next_authority_set
										.keyset_commitment
										.encode(),
								}),
								parachain_heads: mmr_update.latest_mmr_leaf.leaf_extra.encode(),
							}),
							mmr_leaf_index: mmr_update.mmr_proof.leaf_indices[0], /* TODO(blas):
							                                                       * fix this */
							mmr_proof: mmr_update
								.mmr_proof
								.items
								.into_iter()
								.map(|item| item.encode())
								.collect(),
							signed_commitment: Some(RawSignedCommitment {
								commitment: Some(RawCommitment {
									payload: vec![PayloadItem {
										payload_id: MMR_ROOT_ID.to_vec(),
										payload_data: mmr_update
											.signed_commitment
											.commitment
											.payload
											.get_raw(&MMR_ROOT_ID)
											.unwrap()
											.clone(),
									}],
									block_numer: mmr_update
										.signed_commitment
										.commitment
										.block_number,
									validator_set_id: mmr_update
										.signed_commitment
										.commitment
										.validator_set_id,
								}),
								signatures: mmr_update
									.signed_commitment
									.signatures
									.into_iter()
									.map(|item| CommitmentSignature {
										signature: item.signature.to_vec(),
										authority_index: item.index,
									})
									.collect(),
							}),
							authorities_proof: mmr_update
								.authority_proof
								.into_iter()
								.map(|item| item.to_vec())
								.collect(),
						})
					} else {
						None
					},
				})),
			},
			ClientMessage::Misbehaviour(_) => RawClientMessage {
				message: Some(client_message::Message::Misbehaviour(RawMisbehaviour {})),
			},
		}
	}
}

impl Protobuf<RawClientMessage> for ClientMessage {}
