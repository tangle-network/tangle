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
	client_def::GrandpaClient,
	client_message::{ClientMessage, GRANDPA_CLIENT_MESSAGE_TYPE_URL},
	client_state::{ClientState, UpgradeOptions, GRANDPA_CLIENT_STATE_TYPE_URL},
	consensus_state::{ConsensusState, GRANDPA_CONSENSUS_STATE_TYPE_URL},
};
use ibc::{
	core::{
		ics02_client,
		ics02_client::{
			client_consensus::ConsensusState as _, client_state::ClientState as _,
			context::ClientTypes,
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
use sp_core::ed25519;
use sp_runtime::{
	app_crypto::RuntimePublic,
	traits::{BlakeTwo256, Header},
};
use std::{cell::RefCell, collections::BTreeSet};
use tendermint_proto::Protobuf;

pub const MOCK_CLIENT_STATE_TYPE_URL: &str = "/ibc.mock.ClientState";
pub const MOCK_CLIENT_MESSAGE_TYPE_URL: &str = "/ibc.mock.ClientMessage";
pub const MOCK_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.mock.ConsensusState";

#[derive(Clone, Default, PartialEq, Debug, Eq)]
pub struct HostFunctionsManager;

thread_local! {
	static HEADER_HASHES: RefCell<BTreeSet<sp_core::H256>> = RefCell::new(BTreeSet::new());
}

impl grandpa_client_primitives::RelayHostFunctions for HostFunctionsManager {
	type Header = sp_runtime::generic::Header<u32, BlakeTwo256>;

	fn ed25519_verify(sig: &ed25519::Signature, msg: &[u8], pub_key: &ed25519::Public) -> bool {
		pub_key.verify(&msg, sig)
	}

	fn insert_relay_header_hashes(headers: &[<Self::Header as Header>::Hash]) {
		HEADER_HASHES.with(|set| {
			let mut set_mut = set.borrow_mut();
			for hash in headers {
				set_mut.insert(hash.clone());
			}
		})
	}

	fn contains_relay_header_hash(hash: <Self::Header as Header>::Hash) -> bool {
		HEADER_HASHES.with(|set| set.borrow().contains(&hash))
	}
}

impl light_client_common::HostFunctions for HostFunctionsManager {
	type BlakeTwo256 = BlakeTwo256;
}

#[derive(Clone, Debug, PartialEq, Eq, ClientDef)]
pub enum AnyClient {
	Mock(MockClient),
	Grandpa(GrandpaClient<HostFunctionsManager>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnyUpgradeOptions {
	Mock(()),
	Grandpa(UpgradeOptions),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, ClientState, Protobuf)]
#[serde(tag = "type")]
pub enum AnyClientState {
	#[ibc(proto_url = "MOCK_CLIENT_STATE_TYPE_URL")]
	Mock(MockClientState),
	#[serde(skip)]
	#[ibc(proto_url = "GRANDPA_CLIENT_STATE_TYPE_URL")]
	Grandpa(ClientState<HostFunctionsManager>),
}

#[derive(Clone, Debug, Deserialize, Serialize, ClientMessage)]
#[allow(clippy::large_enum_variant)]
pub enum AnyClientMessage {
	#[ibc(proto_url = "MOCK_CLIENT_MESSAGE_TYPE_URL")]
	Mock(MockClientMessage),
	#[serde(skip)]
	#[ibc(proto_url = "GRANDPA_CLIENT_MESSAGE_TYPE_URL")]
	Grandpa(ClientMessage),
}

impl Protobuf<Any> for AnyClientMessage {}

impl TryFrom<Any> for AnyClientMessage {
	type Error = ics02_client::error::Error;

	fn try_from(value: Any) -> Result<Self, Self::Error> {
		match value.type_url.as_str() {
			MOCK_CLIENT_MESSAGE_TYPE_URL => {
				Ok(Self::Mock(panic!("MockClientMessage doesn't implement Protobuf")))
			},
			GRANDPA_CLIENT_MESSAGE_TYPE_URL => Ok(Self::Grandpa(
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
			AnyClientMessage::Grandpa(msg) => Any {
				type_url: GRANDPA_CLIENT_MESSAGE_TYPE_URL.to_string(),
				value: msg.encode_vec().unwrap(),
			},
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, ConsensusState, Protobuf)]
#[serde(tag = "type")]
pub enum AnyConsensusState {
	#[ibc(proto_url = "GRANDPA_CONSENSUS_STATE_TYPE_URL")]
	Grandpa(ConsensusState),
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
