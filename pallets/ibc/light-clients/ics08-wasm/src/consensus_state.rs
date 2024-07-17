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

#[cfg(feature = "cosmwasm")]
use crate::msg::Base64;
use crate::Bytes;
use alloc::{
	boxed::Box,
	string::{String, ToString},
	vec::Vec,
};
use core::{
	convert::Infallible,
	fmt::{Debug, Display},
};
#[cfg(feature = "cosmwasm")]
use cosmwasm_schema::cw_serde;
use ibc::{
	core::{
		ics02_client::client_consensus::ConsensusState as IbcConsensusState,
		ics23_commitment::commitment::CommitmentRoot,
	},
	protobuf::Protobuf,
	timestamp::Timestamp,
};
use ibc_proto::{
	google::protobuf::Any, ibc::lightclients::wasm::v1::ConsensusState as RawConsensusState,
};
use prost::Message;

pub const WASM_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.ConsensusState";

#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Clone, Debug, PartialEq))]
#[derive(Eq)]
pub struct ConsensusState<AnyConsensusState> {
	#[cfg_attr(feature = "cosmwasm", schemars(with = "String"))]
	#[cfg_attr(feature = "cosmwasm", serde(with = "Base64", default))]
	pub data: Bytes,
	pub timestamp: u64,
	#[cfg_attr(feature = "cosmwasm", serde(skip))]
	#[cfg_attr(feature = "cosmwasm", schemars(skip))]
	pub inner: Box<AnyConsensusState>,
}

impl<AnyConsensusState: IbcConsensusState> IbcConsensusState for ConsensusState<AnyConsensusState>
where
	AnyConsensusState: Clone + Debug + Send + Sync,
	AnyConsensusState: TryFrom<Any>,
	<AnyConsensusState as TryFrom<Any>>::Error: Display,
{
	type Error = Infallible;

	fn root(&self) -> &CommitmentRoot {
		unimplemented!()
	}

	fn timestamp(&self) -> Timestamp {
		Timestamp::from_nanoseconds(self.timestamp).expect("timestamp is valid")
	}

	fn encode_to_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error> {
		self.encode_vec()
	}
}

impl<AnyConsensusState> ConsensusState<AnyConsensusState>
where
	AnyConsensusState: Clone + Debug + Send + Sync,
	AnyConsensusState: TryFrom<Any> + IbcConsensusState,
	<AnyConsensusState as TryFrom<Any>>::Error: Display,
{
	pub fn to_any(&self) -> Any {
		Any {
			type_url: WASM_CONSENSUS_STATE_TYPE_URL.to_string(),
			value: self.encode_to_vec().expect(
				"ConsensusState<AnyConsensusState> is always valid and can be encoded to Any",
			),
		}
	}
}

impl<AnyConsensusState> TryFrom<RawConsensusState> for ConsensusState<AnyConsensusState>
where
	AnyConsensusState: TryFrom<Any>,
	<AnyConsensusState as TryFrom<Any>>::Error: Display,
{
	type Error = String;

	fn try_from(raw: RawConsensusState) -> Result<Self, Self::Error> {
		let any = Any::decode(&mut &raw.data[..])
			.map_err(|e| format!("failed to decode ConsensusState::data into Any: {e}"))?;
		let inner = AnyConsensusState::try_from(any).map_err(|e| {
			format!("failed to decode ConsensusState::data into ConsensusState: {e}")
		})?;
		Ok(Self { data: raw.data, timestamp: raw.timestamp, inner: Box::new(inner) })
	}
}

impl<AnyConsensusState> From<ConsensusState<AnyConsensusState>> for RawConsensusState {
	fn from(value: ConsensusState<AnyConsensusState>) -> Self {
		Self { data: value.data, timestamp: value.timestamp }
	}
}

impl<AnyConsensusState> Protobuf<RawConsensusState> for ConsensusState<AnyConsensusState>
where
	AnyConsensusState: Clone,
	AnyConsensusState: TryFrom<Any>,
	<AnyConsensusState as TryFrom<Any>>::Error: Display,
{
}
