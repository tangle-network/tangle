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
#![allow(unreachable_code)]

use crate::{
	client_def::BeefyClient,
	client_message::{ClientMessage, BEEFY_CLIENT_MESSAGE_TYPE_URL},
	client_state::{
		ClientState as BeefyClientState, UpgradeOptions as BeefyUpgradeOptions,
		BEEFY_CLIENT_STATE_TYPE_URL,
	},
	consensus_state::{ConsensusState as BeefyConsensusState, BEEFY_CONSENSUS_STATE_TYPE_URL},
};
use ibc::{
	core::{
		ics02_client,
		ics02_client::{
			client_consensus::ConsensusState, client_state::ClientState, context::ClientTypes,
		},
	},
	mock::{
		client_def::MockClient,
		client_state::{MockClientState, MockConsensusState},
		context::HostBlockType,
		header::MockClientMessage,
		host::MockHostBlock,
	},
	prelude::*,
};
use ibc_derive::{ClientDef, ClientMessage, ClientState, ConsensusState, Protobuf};
use ibc_proto::google::protobuf::Any;
use serde::{Deserialize, Serialize};
use sp_runtime::traits::BlakeTwo256;
use tendermint_proto::Protobuf;

pub const MOCK_CLIENT_STATE_TYPE_URL: &str = "/ibc.mock.ClientState";
pub const MOCK_CLIENT_MESSAGE_TYPE_URL: &str = "/ibc.mock.ClientMessage";
pub const MOCK_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.mock.ConsensusState";

#[derive(Clone, Default, PartialEq, Debug, Eq)]
pub struct HostFunctionsManager;

impl beefy_light_client_primitives::HostFunctions for HostFunctionsManager {
	fn keccak_256(input: &[u8]) -> [u8; 32] {
		beefy_prover::Crypto::keccak_256(input)
	}

	fn secp256k1_ecdsa_recover_compressed(
		signature: &[u8; 65],
		value: &[u8; 32],
	) -> Option<Vec<u8>> {
		beefy_prover::Crypto::secp256k1_ecdsa_recover_compressed(signature, value)
	}
}

impl light_client_common::HostFunctions for HostFunctionsManager {
	type BlakeTwo256 = BlakeTwo256;
}

#[derive(Clone, Debug, PartialEq, Eq, ClientDef)]
pub enum AnyClient {
	Mock(MockClient),
	Beefy(BeefyClient<HostFunctionsManager>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnyUpgradeOptions {
	Mock(()),
	Beefy(BeefyUpgradeOptions),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, ClientState, Protobuf)]
#[serde(tag = "type")]
pub enum AnyClientState {
	#[ibc(proto_url = "MOCK_CLIENT_STATE_TYPE_URL")]
	Mock(MockClientState),
	#[serde(skip)]
	#[ibc(proto_url = "BEEFY_CLIENT_STATE_TYPE_URL")]
	Beefy(BeefyClientState<HostFunctionsManager>),
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, ClientMessage)]
#[allow(clippy::large_enum_variant)]
pub enum AnyClientMessage {
	#[ibc(proto_url = "MOCK_CLIENT_MESSAGE_TYPE_URL")]
	Mock(MockClientMessage),
	#[serde(skip)]
	#[ibc(proto_url = "BEEFY_CLIENT_MESSAGE_TYPE_URL")]
	Beefy(ClientMessage),
}

impl Protobuf<Any> for AnyClientMessage {}

impl TryFrom<Any> for AnyClientMessage {
	type Error = ics02_client::error::Error;

	fn try_from(value: Any) -> Result<Self, Self::Error> {
		match value.type_url.as_str() {
			MOCK_CLIENT_MESSAGE_TYPE_URL => {
				Ok(Self::Mock(panic!("MockClientMessage doesn't implement Protobuf")))
			},
			BEEFY_CLIENT_MESSAGE_TYPE_URL => Ok(Self::Beefy(
				ClientMessage::decode_vec(&value.value)
					.map_err(ics02_client::error::Error::decode_raw_header)?,
			)),
			_ => Err(ics02_client::error::Error::unknown_consensus_state_type(value.type_url)),
		}
	}
}

impl From<AnyClientMessage> for Any {
	fn from(client_msg: AnyClientMessage) -> Self {
		match client_msg {
			AnyClientMessage::Mock(_mock) => {
				panic!("MockClientMessage doesn't implement Protobuf");
			},
			AnyClientMessage::Beefy(beefy) => Any {
				type_url: BEEFY_CLIENT_MESSAGE_TYPE_URL.to_string(),
				value: beefy.encode_vec().unwrap(),
			},
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, ConsensusState, Protobuf)]
#[serde(tag = "type")]
pub enum AnyConsensusState {
	#[ibc(proto_url = "BEEFY_CONSENSUS_STATE_TYPE_URL")]
	Beefy(BeefyConsensusState),
	#[ibc(proto_url = "MOCK_CONSENSUS_STATE_TYPE_URL")]
	Mock(MockConsensusState),
}

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct MockClientTypes;

impl ClientTypes for MockClientTypes {
	type AnyClientMessage = AnyClientMessage;
	type AnyClientState = AnyClientState;
	type AnyConsensusState = AnyConsensusState;
	type ClientDef = AnyClient;
}

impl HostBlockType for MockClientTypes {
	type HostBlock = MockHostBlock;
}

impl From<MockHostBlock> for AnyClientMessage {
	fn from(block: MockHostBlock) -> Self {
		let MockHostBlock::Mock(header) = block;
		AnyClientMessage::Mock(MockClientMessage::Header(header))
	}
}

impl From<MockHostBlock> for AnyConsensusState {
	fn from(block: MockHostBlock) -> Self {
		let MockHostBlock::Mock(header) = block;
		AnyConsensusState::Mock(MockConsensusState::new(header))
	}
}
