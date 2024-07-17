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

use crate::{
	error::Error,
	proto::{
		self, client_message, ClientMessage as RawClientMessage, Header as RawHeader,
		Misbehaviour as RawMisbehaviour,
	},
};
use alloc::{collections::BTreeMap, vec::Vec};
use anyhow::anyhow;
use grandpa_client_primitives::{FinalityProof, ParachainHeaderProofs};
use ibc::Height;
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_runtime::traits::BlakeTwo256;
use tendermint_proto::Protobuf;

/// Protobuf type url for GRANDPA header
pub const GRANDPA_CLIENT_MESSAGE_TYPE_URL: &str = "/ibc.lightclients.grandpa.v1.ClientMessage";
pub const GRANDPA_HEADER_TYPE_URL: &str = "/ibc.lightclients.grandpa.v1.Header";
pub const GRANDPA_MISBEHAVIOUR_TYPE_URL: &str = "/ibc.lightclients.grandpa.v1.Misbehaviour";

/// Relay chain substrate header type
pub type RelayChainHeader = sp_runtime::generic::Header<u32, BlakeTwo256>;

/// Parachain headers with a Grandpa finality proof.
#[derive(Clone, Debug)]
pub struct Header {
	/// The grandpa finality proof: contains relay chain headers from the
	/// last known finalized grandpa block.
	pub finality_proof: FinalityProof<RelayChainHeader>,
	/// Contains a map of relay chain header hashes to parachain headers
	/// finalzed at the relay chain height. We check for this parachain header finalization
	/// via state proofs. Also contains extrinsic proof for timestamp.
	pub parachain_headers: BTreeMap<H256, ParachainHeaderProofs>,
	/// Lazily initialized height
	pub height: Height,
}

impl Header {
	pub fn height(&self) -> Height {
		self.height
	}
}

/// Misbehaviour type for GRANDPA. If both first and second proofs are valid
/// (that is, form a valid canonical chain of blocks where on of the chain is a fork of
/// the main one)
#[derive(Clone, Debug)]
pub struct Misbehaviour {
	/// first proof of misbehaviour
	pub first_finality_proof: FinalityProof<RelayChainHeader>,
	/// second proof of misbehaviour
	pub second_finality_proof: FinalityProof<RelayChainHeader>,
}

/// [`ClientMessage`] for Ics10-GRANDPA
#[derive(Clone, Debug)]
pub enum ClientMessage {
	/// This is the variant for header updates
	Header(Header),
	/// This is for submitting misbehaviors.
	Misbehaviour(Misbehaviour),
}

impl ibc::core::ics02_client::client_message::ClientMessage for ClientMessage {
	fn encode_to_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error> {
		self.encode_vec()
	}
}

impl Protobuf<RawHeader> for Header {}

impl TryFrom<RawHeader> for Header {
	type Error = Error;

	fn try_from(raw_header: RawHeader) -> Result<Self, Self::Error> {
		let finality_proof = raw_header
			.finality_proof
			.ok_or_else(|| anyhow!("Grandpa finality proof is required!"))?;
		let block = if finality_proof.block.len() == 32 {
			H256::from_slice(&*finality_proof.block)
		} else {
			Err(anyhow!("Invalid hash type with length: {}", finality_proof.block.len()))?
		};

		let parachain_headers = raw_header
			.parachain_headers
			.into_iter()
			.map(|header| {
				let block = if header.relay_hash.len() == 32 {
					H256::from_slice(&*header.relay_hash)
				} else {
					Err(anyhow!("Invalid hash type with length: {}", header.relay_hash.len()))?
				};
				let proto::ParachainHeaderProofs { state_proof, extrinsic_proof, extrinsic } =
					header
						.parachain_header
						.ok_or_else(|| anyhow!("Parachain header is required!"))?;
				let parachain_header_proofs =
					ParachainHeaderProofs { state_proof, extrinsic, extrinsic_proof };
				Ok((block, parachain_header_proofs))
			})
			.collect::<Result<_, Error>>()?;

		let unknown_headers = finality_proof
			.unknown_headers
			.into_iter()
			.map(|h| {
				let header = parity_scale_codec::Decode::decode(&mut &h[..])?;
				Ok(header)
			})
			.collect::<Result<_, Error>>()?;

		Ok(Header {
			finality_proof: FinalityProof {
				block,
				justification: finality_proof.justification,
				unknown_headers,
			},
			parachain_headers,
			height: Height::new(raw_header.para_id as u64, raw_header.para_height as u64),
		})
	}
}

impl From<Header> for RawHeader {
	fn from(header: Header) -> Self {
		let parachain_headers = header
			.parachain_headers
			.into_iter()
			.map(|(hash, parachain_header_proofs)| proto::ParachainHeaderWithRelayHash {
				relay_hash: hash.as_bytes().to_vec(),
				parachain_header: Some(proto::ParachainHeaderProofs {
					state_proof: parachain_header_proofs.state_proof,
					extrinsic: parachain_header_proofs.extrinsic,
					extrinsic_proof: parachain_header_proofs.extrinsic_proof,
				}),
			})
			.collect();
		let finality_proof = proto::FinalityProof {
			block: header.finality_proof.block.as_bytes().to_vec(),
			justification: header.finality_proof.justification,
			unknown_headers: header
				.finality_proof
				.unknown_headers
				.into_iter()
				.map(|h| h.encode())
				.collect(),
		};

		RawHeader {
			finality_proof: Some(finality_proof),
			parachain_headers,
			para_id: header.height.revision_number as u32,
			para_height: header.height.revision_height as u32,
		}
	}
}

impl Protobuf<RawMisbehaviour> for Misbehaviour {}

impl TryFrom<RawMisbehaviour> for Misbehaviour {
	type Error = Error;

	fn try_from(value: RawMisbehaviour) -> Result<Self, Self::Error> {
		Ok(Misbehaviour {
			first_finality_proof: Decode::decode(&mut &*value.first_finality_proof)?,
			second_finality_proof: Decode::decode(&mut &*value.second_finality_proof)?,
		})
	}
}

impl From<Misbehaviour> for RawMisbehaviour {
	fn from(value: Misbehaviour) -> Self {
		RawMisbehaviour {
			first_finality_proof: value.first_finality_proof.encode(),
			second_finality_proof: value.second_finality_proof.encode(),
		}
	}
}

impl Protobuf<RawClientMessage> for ClientMessage {}

impl TryFrom<RawClientMessage> for ClientMessage {
	type Error = Error;

	fn try_from(raw_client_message: RawClientMessage) -> Result<Self, Self::Error> {
		let message = match raw_client_message
			.message
			.ok_or_else(|| anyhow!("Must supply either Header or Misbehaviour type!"))?
		{
			client_message::Message::Header(raw_header) =>
				ClientMessage::Header(Header::try_from(raw_header)?),
			client_message::Message::Misbehaviour(raw_misbehaviour) =>
				ClientMessage::Misbehaviour(Misbehaviour::try_from(raw_misbehaviour)?),
		};

		Ok(message)
	}
}

impl From<ClientMessage> for RawClientMessage {
	fn from(client_message: ClientMessage) -> Self {
		match client_message {
			ClientMessage::Header(header) =>
				RawClientMessage { message: Some(client_message::Message::Header(header.into())) },
			ClientMessage::Misbehaviour(misbehaviior) => RawClientMessage {
				message: Some(client_message::Message::Misbehaviour(misbehaviior.into())),
			},
		}
	}
}
