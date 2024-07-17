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

use crate::prelude::*;

use alloc::collections::btree_map::BTreeMap as HashMap;

use core::{convert::Infallible, fmt::Debug, time::Duration};

use ibc_proto::{google::protobuf::Any, ibc::core::client::v1::ConsensusStateWithHeight};
use serde::{Deserialize, Serialize};
use tendermint_proto::Protobuf;

use crate::{
	core::{
		ics02_client::{
			client_consensus::ConsensusState,
			client_state::{ClientState, ClientType},
			context::ClientTypes,
			error::{Error as Ics02Error, Error},
		},
		ics23_commitment::commitment::CommitmentRoot,
		ics24_host::identifier::ChainId,
	},
	downcast,
	mock::{
		client_def::{AnyClient, MockClient},
		context::HostBlockType,
		header::{MockClientMessage, MockHeader},
	},
	timestamp::Timestamp,
	Height,
};
use ibc_proto::ibc::mock::{
	ClientState as RawMockClientState, ConsensusState as RawMockConsensusState,
};

pub const MOCK_CLIENT_STATE_TYPE_URL: &str = "/ibc.mock.ClientState";

/// A mock of an IBC client record as it is stored in a mock context.
/// For testing ICS02 handlers mostly, cf. `MockClientContext`.
#[derive(Clone, Debug)]
pub struct MockClientRecord<C: ClientTypes> {
	/// The type of this client.
	pub client_type: ClientType,

	/// The client state (representing only the latest height at the moment).
	pub client_state: Option<C::AnyClientState>,

	/// Mapping of heights to consensus states for this client.
	pub consensus_states: HashMap<Height, C::AnyConsensusState>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnyUpgradeOptions {
	Mock(()),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, ClientState, Protobuf)]
#[serde(tag = "type")]
pub enum AnyClientState {
	#[ibc(proto_url = "MOCK_CLIENT_STATE_TYPE_URL")]
	Mock(MockClientState),
}

/// A mock of a client state. For an example of a real structure that this mocks, you can see
/// `ClientState` of ics07_tendermint/client_state.rs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub struct MockClientState {
	pub header: MockHeader,
	pub frozen_height: Option<Height>,
}

impl Protobuf<RawMockClientState> for MockClientState {}

impl MockClientState {
	pub fn new(client_message: MockClientMessage) -> Self {
		Self { header: client_message.header(), frozen_height: None }
	}

	pub fn refresh_time(&self) -> Option<Duration> {
		None
	}
}

impl From<MockClientState> for AnyClientState {
	fn from(mcs: MockClientState) -> Self {
		Self::Mock(mcs)
	}
}

impl TryFrom<RawMockClientState> for MockClientState {
	type Error = Error;

	fn try_from(raw: RawMockClientState) -> Result<Self, Self::Error> {
		Ok(Self::new(MockHeader::try_from(raw.header.unwrap())?.into()))
	}
}

impl From<MockClientState> for RawMockClientState {
	fn from(value: MockClientState) -> Self {
		RawMockClientState {
			header: Some(ibc_proto::ibc::mock::Header {
				height: Some(value.header.height().into()),
				timestamp: value.header.timestamp.nanoseconds(),
			}),
		}
	}
}

impl ClientState for MockClientState {
	type UpgradeOptions = ();
	type ClientDef = MockClient;

	fn chain_id(&self) -> ChainId {
		self.chain_id()
	}

	fn client_def(&self) -> Self::ClientDef {
		MockClient::default()
	}

	fn client_type(&self) -> ClientType {
		Self::client_type()
	}

	fn latest_height(&self) -> Height {
		self.latest_height()
	}

	fn frozen_height(&self) -> Option<Height> {
		self.frozen_height
	}

	fn upgrade(self, _upgrade_height: Height, _upgrade_options: (), _chain_id: ChainId) -> Self {
		self.upgrade(_upgrade_height, _upgrade_options, _chain_id)
	}

	fn expired(&self, elapsed: Duration) -> bool {
		self.expired(elapsed)
	}

	fn encode_to_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error> {
		self.encode_vec()
	}
}

impl MockClientState {
	pub fn chain_id(&self) -> ChainId {
		ChainId::default()
	}

	pub fn client_type() -> ClientType {
		"9999-mock".to_string()
	}

	pub fn latest_height(&self) -> Height {
		self.header.height()
	}

	pub fn frozen_height(&self) -> Option<Height> {
		self.frozen_height
	}

	pub fn upgrade(
		self,
		_upgrade_height: Height,
		_upgrade_options: (),
		_chain_id: ChainId,
	) -> Self {
		todo!()
	}

	pub fn expired(&self, _elapsed: Duration) -> bool {
		false
	}
}

impl From<MockConsensusState> for MockClientState {
	fn from(cs: MockConsensusState) -> Self {
		Self::new(cs.header.into())
	}
}

pub const MOCK_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.mock.ConsensusState";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, ConsensusState, Protobuf)]
#[serde(tag = "type")]
pub enum AnyConsensusState {
	#[ibc(proto_url = "MOCK_CONSENSUS_STATE_TYPE_URL")]
	Mock(MockConsensusState),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AnyConsensusStateWithHeight<C: ClientTypes> {
	pub height: Height,
	pub consensus_state: C::AnyConsensusState,
}

impl<C: HostBlockType> Protobuf<ConsensusStateWithHeight> for AnyConsensusStateWithHeight<C>
where
	C::AnyClientMessage: TryFrom<Any, Error = Ics02Error> + Into<Any> + From<C::HostBlock>,
	C::AnyClientState: Eq + TryFrom<Any, Error = Ics02Error> + Into<Any>,
	C::AnyConsensusState:
		Eq + TryFrom<Any, Error = Ics02Error> + Into<Any> + From<C::HostBlock> + 'static,
{
}

impl<C: HostBlockType> TryFrom<ConsensusStateWithHeight> for AnyConsensusStateWithHeight<C>
where
	C::AnyClientMessage: TryFrom<Any, Error = Ics02Error> + Into<Any> + From<C::HostBlock>,
	C::AnyClientState: Eq + TryFrom<Any, Error = Ics02Error> + Into<Any>,
	C::AnyConsensusState:
		Eq + TryFrom<Any, Error = Ics02Error> + Into<Any> + From<C::HostBlock> + 'static,
{
	type Error = Error;

	fn try_from(value: ConsensusStateWithHeight) -> Result<Self, Self::Error> {
		let state = value.consensus_state.map(C::AnyConsensusState::try_from).unwrap().unwrap();

		Ok(AnyConsensusStateWithHeight {
			height: value.height.ok_or_else(Error::missing_height)?.into(),
			consensus_state: state,
		})
	}
}

impl<C: HostBlockType> From<AnyConsensusStateWithHeight<C>> for ConsensusStateWithHeight
where
	C::AnyClientMessage: TryFrom<Any, Error = Ics02Error> + Into<Any> + From<C::HostBlock>,
	C::AnyClientState: Eq + TryFrom<Any, Error = Ics02Error> + Into<Any>,
	C::AnyConsensusState:
		Eq + TryFrom<Any, Error = Ics02Error> + Into<Any> + From<C::HostBlock> + 'static,
{
	fn from(value: AnyConsensusStateWithHeight<C>) -> Self {
		ConsensusStateWithHeight {
			height: Some(value.height.into()),
			consensus_state: Some(value.consensus_state.into()),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MockConsensusState {
	pub header: MockHeader,
	pub root: CommitmentRoot,
}

impl MockConsensusState {
	pub fn new(header: MockHeader) -> Self {
		MockConsensusState { header, root: CommitmentRoot::from(vec![0]) }
	}

	pub fn timestamp(&self) -> Timestamp {
		self.header.timestamp
	}
}

impl Protobuf<RawMockConsensusState> for MockConsensusState {}

impl TryFrom<RawMockConsensusState> for MockConsensusState {
	type Error = Error;

	fn try_from(raw: RawMockConsensusState) -> Result<Self, Self::Error> {
		let raw_header = raw.header.ok_or_else(Error::missing_raw_consensus_state)?;

		Ok(Self { header: MockHeader::try_from(raw_header)?, root: CommitmentRoot::from(vec![0]) })
	}
}

impl From<MockConsensusState> for RawMockConsensusState {
	fn from(value: MockConsensusState) -> Self {
		RawMockConsensusState {
			header: Some(ibc_proto::ibc::mock::Header {
				height: Some(value.header.height().into()),
				timestamp: value.header.timestamp.nanoseconds(),
			}),
		}
	}
}

impl From<MockConsensusState> for AnyConsensusState {
	fn from(mcs: MockConsensusState) -> Self {
		Self::Mock(mcs)
	}
}

impl TryFrom<AnyConsensusState> for MockConsensusState {
	type Error = Error;

	fn try_from(value: AnyConsensusState) -> Result<Self, Self::Error> {
		downcast!(
			value => AnyConsensusState::Mock
		)
		.ok_or_else(|| Error::client_args_type_mismatch(MockClientState::client_type().to_owned()))
	}
}

impl ConsensusState for MockConsensusState {
	type Error = Infallible;

	fn root(&self) -> &CommitmentRoot {
		&self.root
	}

	fn timestamp(&self) -> Timestamp {
		self.timestamp()
	}

	fn encode_to_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error> {
		self.encode_vec()
	}
}
