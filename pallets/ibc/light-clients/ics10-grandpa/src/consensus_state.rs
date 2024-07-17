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

use alloc::{format, vec, vec::Vec};
use anyhow::anyhow;
use core::{convert::Infallible, fmt::Debug};
use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use tendermint::time::Time;
use tendermint_proto::{google::protobuf as tpb, Protobuf};

use crate::proto::ConsensusState as RawConsensusState;

use crate::{alloc::string::ToString, error::Error};
use grandpa_client_primitives::{parachain_header_storage_key, ParachainHeaderProofs};
use ibc::{core::ics23_commitment::commitment::CommitmentRoot, timestamp::Timestamp, Height};
use ibc_proto::google::protobuf::Any;
use light_client_common::{decode_timestamp_extrinsic, state_machine};
use sp_core::H256;
use sp_runtime::{generic, traits::BlakeTwo256, SaturatedConversion};
use sp_trie::StorageProof;

/// Protobuf type url for GRANDPA Consensus State
pub const GRANDPA_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.lightclients.grandpa.v1.ConsensusState";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusState {
	pub timestamp: Time,
	pub root: CommitmentRoot,
}

impl ConsensusState {
	pub fn new(root: Vec<u8>, timestamp: Time) -> Self {
		Self { timestamp, root: root.into() }
	}

	pub fn to_any(&self) -> Any {
		Any {
			type_url: GRANDPA_CONSENSUS_STATE_TYPE_URL.to_string(),
			value: self.encode_vec().expect("encode ConsensusState"),
		}
	}

	pub fn from_header<H>(
		parachain_header_proof: ParachainHeaderProofs,
		para_id: u32,
		relay_state_root: H256,
	) -> Result<(Height, Self), Error>
	where
		H: grandpa_client_primitives::RelayHostFunctions,
	{
		let key = parachain_header_storage_key(para_id);
		let proof = StorageProof::new(parachain_header_proof.state_proof);
		let parachain_header_bytes = state_machine::read_proof_check::<H::BlakeTwo256, _>(
			&relay_state_root,
			proof,
			vec![parachain_header_storage_key(para_id)],
		)
		.map_err(anyhow::Error::msg)?
		.remove(key.as_ref())
		.flatten()
		.ok_or_else(|| anyhow!("Invalid state proof for parachain header"))?;

		let parachain_header =
			generic::Header::<u32, BlakeTwo256>::decode(&mut &parachain_header_bytes[..])?;
		let root = parachain_header.state_root.0.to_vec();

		let timestamp = decode_timestamp_extrinsic(&parachain_header_proof.extrinsic)?;
		let duration = core::time::Duration::from_millis(timestamp);
		let timestamp = Timestamp::from_nanoseconds(duration.as_nanos().saturated_into::<u64>())?
			.into_tm_time()
			.ok_or_else(|| anyhow!("Error decoding Timestamp, timestamp cannot be zero"))?;

		Ok((
			Height::new(para_id as u64, parachain_header.number as u64),
			Self { root: root.into(), timestamp },
		))
	}
}

impl ibc::core::ics02_client::client_consensus::ConsensusState for ConsensusState {
	type Error = Infallible;

	fn root(&self) -> &CommitmentRoot {
		&self.root
	}

	fn timestamp(&self) -> Timestamp {
		self.timestamp.into()
	}

	fn encode_to_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error> {
		self.encode_vec()
	}
}

impl Protobuf<RawConsensusState> for ConsensusState {}

impl TryFrom<RawConsensusState> for ConsensusState {
	type Error = Error;

	fn try_from(raw: RawConsensusState) -> Result<Self, Self::Error> {
		let prost_types::Timestamp { seconds, nanos } = raw
			.timestamp
			.ok_or_else(|| Error::Custom(format!("Invalid consensus state: missing timestamp")))?;
		let proto_timestamp = tpb::Timestamp { seconds, nanos };
		let timestamp = proto_timestamp.try_into().map_err(|e| {
			Error::Custom(format!("Invalid consensus state: invalid timestamp {e}"))
		})?;

		Ok(Self { root: raw.root.into(), timestamp })
	}
}

impl From<ConsensusState> for RawConsensusState {
	fn from(value: ConsensusState) -> Self {
		let tpb::Timestamp { seconds, nanos } = value.timestamp.into();
		let timestamp = prost_types::Timestamp { seconds, nanos };

		RawConsensusState { timestamp: Some(timestamp), root: value.root.into_vec() }
	}
}

#[cfg(any(test, feature = "mocks"))]
pub mod test_util {
	use super::*;
	use crate::mock::AnyConsensusState;

	pub fn get_dummy_beefy_consensus_state() -> AnyConsensusState {
		AnyConsensusState::Grandpa(ConsensusState {
			timestamp: Time::now(),
			root: vec![0; 32].into(),
		})
	}
}
