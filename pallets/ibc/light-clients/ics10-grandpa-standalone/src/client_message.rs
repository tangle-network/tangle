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
use alloc::vec::Vec;
use anyhow::anyhow;
use grandpa_client_primitives::{FinalityProof, StandaloneTimestampProof};
use ibc::Height;
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_runtime::traits::BlakeTwo256;
use tendermint_proto::Protobuf;

/// Protobuf type url for GRANDPA header
pub const GRANDPA_STANDALONE_CLIENT_MESSAGE_TYPE_URL: &str =
	"/ibc.lightclients.grandpa_standalone.v1.ClientMessage";
pub const GRANDPA_STANDALONE_HEADER_TYPE_URL: &str =
	"/ibc.lightclients.grandpa_standalone.v1.Header";
pub const GRANDPA_STANDALONE_MISBEHAVIOUR_TYPE_URL: &str =
	"/ibc.lightclients.grandpa_standalone.v1.Misbehaviour";

/// Relay chain substrate header type
pub type StandaloneChainHeader = sp_runtime::generic::Header<u32, BlakeTwo256>;

/// Parachain headers with a Grandpa finality proof.
#[derive(Clone, Debug)]
pub struct Header {
	/// The grandpa finality proof: contains relay chain headers from the
	/// last known finalized grandpa block.
	pub finality_proof: FinalityProof<StandaloneChainHeader>,

	/// Header timestamp roof
	pub timestamp_proof: StandaloneTimestampProof,

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
	pub first_finality_proof: FinalityProof<StandaloneChainHeader>,
	/// second proof of misbehaviour
	pub second_finality_proof: FinalityProof<StandaloneChainHeader>,
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

		let timestamp_proof = raw_header
			.timestamp_proof
			.map(|p| {
				let proto::TimestampProof { extrinsic_proof, extrinsic } = p;
				StandaloneTimestampProof { extrinsic, extrinsic_proof }
			})
			.ok_or_else(|| anyhow!("Timestamp proof is required!"))?;

		Ok(Header {
			finality_proof: FinalityProof {
				block,
				justification: finality_proof.justification,
				unknown_headers: vec![],
			},
			timestamp_proof,
			height: Height::new(raw_header.chain_id as u64, raw_header.chain_height as u64),
		})
	}
}

impl From<Header> for RawHeader {
	fn from(header: Header) -> Self {
		let finality_proof = proto::FinalityProof {
			block: header.finality_proof.block.as_bytes().to_vec(),
			justification: header.finality_proof.justification,
		};

		RawHeader {
			finality_proof: Some(finality_proof),
			timestamp_proof: Some(proto::TimestampProof {
				extrinsic_proof: header.timestamp_proof.extrinsic_proof,
				extrinsic: header.timestamp_proof.extrinsic,
			}),
			chain_id: header.height.revision_number as u32,
			chain_height: header.height.revision_height as u32,
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
			client_message::Message::Header(raw_header) => {
				ClientMessage::Header(Header::try_from(raw_header)?)
			},
			client_message::Message::Misbehaviour(raw_misbehaviour) => {
				ClientMessage::Misbehaviour(Misbehaviour::try_from(raw_misbehaviour)?)
			},
		};

		Ok(message)
	}
}

impl From<ClientMessage> for RawClientMessage {
	fn from(client_message: ClientMessage) -> Self {
		match client_message {
			ClientMessage::Header(header) => {
				RawClientMessage { message: Some(client_message::Message::Header(header.into())) }
			},
			ClientMessage::Misbehaviour(misbehaviior) => RawClientMessage {
				message: Some(client_message::Message::Misbehaviour(misbehaviior.into())),
			},
		}
	}
}
