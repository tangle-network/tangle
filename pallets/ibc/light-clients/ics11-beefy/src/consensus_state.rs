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

use ibc::prelude::*;

use core::{convert::Infallible, fmt::Debug};
use serde::Serialize;
use tendermint::time::Time;
use tendermint_proto::{google::protobuf as tpb, Protobuf};

use crate::proto::ConsensusState as RawConsensusState;

use crate::{client_message::ParachainHeader, error::Error};
use ibc::{core::ics23_commitment::commitment::CommitmentRoot, timestamp::Timestamp};
use light_client_common::decode_timestamp_extrinsic;

/// Protobuf type url for Beefy Consensus State
pub const BEEFY_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.lightclients.beefy.v1.ConsensusState";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConsensusState {
	pub timestamp: Time,
	pub root: CommitmentRoot,
}

impl ConsensusState {
	pub fn new(root: Vec<u8>, timestamp: Time) -> Self {
		Self { timestamp, root: root.into() }
	}

	pub fn from_header(header: ParachainHeader) -> Result<Self, Error> {
		use sp_runtime::SaturatedConversion;
		let root = header.parachain_header.state_root.0.to_vec();

		let timestamp = decode_timestamp_extrinsic(&header.timestamp_extrinsic)?;
		let duration = core::time::Duration::from_millis(timestamp);
		let timestamp = Timestamp::from_nanoseconds(duration.as_nanos().saturated_into::<u64>())?
			.into_tm_time()
			.ok_or_else(|| {
				Error::Custom("Error decoding Timestamp, timestamp cannot be zero".to_string())
			})?;

		Ok(Self { root: root.into(), timestamp })
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

#[cfg(test)]
pub mod test_util {
	use super::*;
	use crate::mock::AnyConsensusState;

	pub fn get_dummy_beefy_consensus_state() -> AnyConsensusState {
		AnyConsensusState::Beefy(ConsensusState {
			timestamp: Time::now(),
			root: vec![0; 32].into(),
		})
	}
}
