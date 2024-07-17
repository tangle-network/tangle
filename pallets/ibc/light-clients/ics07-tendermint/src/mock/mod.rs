#![allow(unreachable_code)]
// Copyright 2022 ComposableFi
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod context;
pub mod host;

use crate::{
	client_def::TendermintClient,
	client_state::{
		ClientState as TendermintClientState, UpgradeOptions as TendermintUpgradeOptions,
	},
	consensus_state::ConsensusState as TendermintConsensusState,
	HostFunctionsProvider,
};
use tendermint::{
	crypto::{
		signature::{Error, Verifier},
		Sha256,
	},
	merkle::{Hash, MerkleHash, NonIncremental, HASH_SIZE},
	PublicKey, Signature,
};
use tendermint_light_client_verifier::operations::CommitValidator;

use crate::{
	client_message::{ClientMessage, TENDERMINT_CLIENT_MESSAGE_TYPE_URL},
	client_state::TENDERMINT_CLIENT_STATE_TYPE_URL,
	consensus_state::TENDERMINT_CONSENSUS_STATE_TYPE_URL,
	mock::host::MockHostBlock,
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
	},
	prelude::*,
};
use ibc_derive::{ClientDef, ClientMessage, ClientState, ConsensusState};
use ibc_proto::google::protobuf::Any;
use tendermint_proto::Protobuf;

pub const MOCK_CLIENT_STATE_TYPE_URL: &str = "/ibc.mock.ClientState";
pub const MOCK_CLIENT_MESSAGE_TYPE_URL: &str = "/ibc.mock.ClientMessage";
pub const MOCK_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.mock.ConsensusState";

#[derive(Clone, Debug, PartialEq, Eq, ClientDef)]
pub enum AnyClient {
	Mock(MockClient),
	Tendermint(TendermintClient<Crypto>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnyUpgradeOptions {
	Mock(()),
	Tendermint(TendermintUpgradeOptions),
}

#[derive(Clone, Debug, PartialEq, Eq, ClientState, Protobuf)]
pub enum AnyClientState {
	#[ibc(proto_url = "MOCK_CLIENT_STATE_TYPE_URL")]
	Mock(MockClientState),
	#[ibc(proto_url = "TENDERMINT_CLIENT_STATE_TYPE_URL")]
	Tendermint(TendermintClientState<Crypto>),
}
#[derive(Clone, Debug, PartialEq, Eq, ClientMessage)]
#[allow(clippy::large_enum_variant)]
pub enum AnyClientMessage {
	#[ibc(proto_url = "MOCK_CLIENT_MESSAGE_TYPE_URL")]
	Mock(MockClientMessage),
	#[ibc(proto_url = "TENDERMINT_CLIENT_MESSAGE_TYPE_URL")]
	Tendermint(ClientMessage),
}

impl Protobuf<Any> for AnyClientMessage {}

impl TryFrom<Any> for AnyClientMessage {
	type Error = ics02_client::error::Error;

	fn try_from(value: Any) -> Result<Self, Self::Error> {
		match value.type_url.as_str() {
			MOCK_CLIENT_MESSAGE_TYPE_URL => {
				Ok(Self::Mock(panic!("MockClientMessage doesn't implement Protobuf")))
			},
			TENDERMINT_CLIENT_MESSAGE_TYPE_URL => Ok(Self::Tendermint(
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
			AnyClientMessage::Tendermint(msg) => Any {
				type_url: TENDERMINT_CLIENT_MESSAGE_TYPE_URL.to_string(),
				value: msg.encode_vec().unwrap(),
			},
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, ConsensusState, Protobuf)]
pub enum AnyConsensusState {
	#[ibc(proto_url = "TENDERMINT_CONSENSUS_STATE_TYPE_URL")]
	Tendermint(TendermintConsensusState),
	#[ibc(proto_url = "MOCK_CONSENSUS_STATE_TYPE_URL")]
	Mock(MockConsensusState),
}

impl From<MockConsensusState> for AnyConsensusState {
	fn from(mcs: MockConsensusState) -> Self {
		Self::Mock(mcs)
	}
}

impl From<MockClientState> for AnyClientState {
	fn from(mcs: MockClientState) -> Self {
		Self::Mock(mcs)
	}
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Crypto;

impl ics23::HostFunctionsProvider for Crypto {
	fn sha2_256(message: &[u8]) -> [u8; 32] {
		sha2::Sha256::digest(message)
	}

	fn sha2_512(_message: &[u8]) -> [u8; 64] {
		unimplemented!()
	}

	fn sha2_512_truncated(_message: &[u8]) -> [u8; 32] {
		unimplemented!()
	}

	fn sha3_512(_message: &[u8]) -> [u8; 64] {
		unimplemented!()
	}

	fn ripemd160(_message: &[u8]) -> [u8; 20] {
		unimplemented!()
	}
}

impl Sha256 for Crypto {
	fn digest(_data: impl AsRef<[u8]>) -> [u8; HASH_SIZE] {
		unimplemented!()
	}
}

impl MerkleHash for Crypto {
	fn empty_hash(&mut self) -> Hash {
		NonIncremental::<Self>::default().empty_hash()
	}

	fn leaf_hash(&mut self, bytes: &[u8]) -> Hash {
		NonIncremental::<Self>::default().leaf_hash(bytes)
	}

	fn inner_hash(&mut self, left: Hash, right: Hash) -> Hash {
		NonIncremental::<Self>::default().inner_hash(left, right)
	}
}

impl Verifier for Crypto {
	fn verify(_pubkey: PublicKey, _msg: &[u8], _signature: &Signature) -> Result<(), Error> {
		unimplemented!()
	}
}

impl CommitValidator for Crypto {}

impl HostFunctionsProvider for Crypto {}
