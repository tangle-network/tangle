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

//! Types for the IBC events emitted from Tendermint Websocket by the client module.

use crate::{
	core::{ics02_client::height::Height, ics24_host::identifier::ClientId},
	events::IbcEvent,
	prelude::*,
};
use serde_derive::{Deserialize, Serialize};
use tendermint::abci::EventAttribute;

/// The content of the `key` field for the attribute containing the height.
pub const HEIGHT_ATTRIBUTE_KEY: &str = "height";

/// The content of the `key` field for the attribute containing the client identifier.
pub const CLIENT_ID_ATTRIBUTE_KEY: &str = "client_id";

/// The content of the `key` field for the attribute containing the client type.
pub const CLIENT_TYPE_ATTRIBUTE_KEY: &str = "client_type";

/// The content of the `key` field for the attribute containing the height.
pub const CONSENSUS_HEIGHT_ATTRIBUTE_KEY: &str = "consensus_height";

/// The content of the `key` field for the attribute containing WASM code ID.
pub const WASM_CODE_ID_ATTRIBUTE_KEY: &str = "wasm_code_id";

/// NewBlock event signals the committing & execution of a new block.
// TODO - find a better place for NewBlock
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub struct NewBlock {
	pub height: Height,
}

impl NewBlock {
	pub fn new(h: Height) -> NewBlock {
		NewBlock { height: h }
	}
	pub fn set_height(&mut self, height: Height) {
		self.height = height;
	}
	pub fn height(&self) -> Height {
		self.height
	}
}

impl From<NewBlock> for IbcEvent {
	fn from(v: NewBlock) -> Self {
		IbcEvent::NewBlock(v)
	}
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Attributes {
	pub height: Height,
	pub client_id: ClientId,
	pub client_type: String,
	pub consensus_height: Height,
}

#[cfg(not(test))]
impl Default for Attributes {
	fn default() -> Self {
		Attributes {
			height: Height::default(),
			client_id: Default::default(),
			client_type: "00-uninitialized".to_owned(),
			consensus_height: Height::default(),
		}
	}
}

/// Convert attributes to Tendermint ABCI tags
impl From<Attributes> for Vec<EventAttribute> {
	fn from(a: Attributes) -> Self {
		let height = EventAttribute {
			key: HEIGHT_ATTRIBUTE_KEY.to_string(),
			value: a.height.to_string(),
			index: false,
		};
		let client_id = EventAttribute {
			key: CLIENT_ID_ATTRIBUTE_KEY.to_string(),
			value: a.client_id.to_string(),
			index: false,
		};
		let client_type = EventAttribute {
			key: CLIENT_TYPE_ATTRIBUTE_KEY.to_string(),
			value: a.client_type.to_owned(),
			index: false,
		};
		let consensus_height = EventAttribute {
			key: CONSENSUS_HEIGHT_ATTRIBUTE_KEY.to_string(),
			value: a.height.to_string(),
			index: false,
		};
		vec![height, client_id, client_type, consensus_height]
	}
}

impl core::fmt::Display for Attributes {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		write!(
			f,
			"height: {}, client_id: {}, consensus_height: {}, client_type: {}",
			self.height, self.client_id, self.consensus_height, self.client_type
		)
	}
}

/// CreateClient event signals the creation of a new on-chain client (IBC client).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct CreateClient(pub Attributes);

impl CreateClient {
	pub fn client_id(&self) -> &ClientId {
		&self.0.client_id
	}
	pub fn height(&self) -> Height {
		self.0.height
	}
	pub fn set_height(&mut self, height: Height) {
		self.0.height = height;
	}
}

impl From<Attributes> for CreateClient {
	fn from(attrs: Attributes) -> Self {
		CreateClient(attrs)
	}
}

impl From<CreateClient> for IbcEvent {
	fn from(v: CreateClient) -> Self {
		IbcEvent::CreateClient(v)
	}
}

impl core::fmt::Display for CreateClient {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		write!(f, "{}", self.0)
	}
}

/// UpdateClient event signals a recent update of an on-chain client (IBC Client).
// TODO: use generic header type
#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct UpdateClient {
	pub common: Attributes,
	pub header: Option<Vec<u8>>,
}

impl UpdateClient {
	pub fn client_id(&self) -> &ClientId {
		&self.common.client_id
	}

	pub fn client_type(&self) -> &str {
		&self.common.client_type
	}

	pub fn height(&self) -> Height {
		self.common.height
	}

	pub fn set_height(&mut self, height: Height) {
		self.common.height = height;
	}

	pub fn consensus_height(&self) -> Height {
		self.common.consensus_height
	}
}

impl From<Attributes> for UpdateClient {
	fn from(attrs: Attributes) -> Self {
		UpdateClient { common: attrs, header: None }
	}
}

impl From<UpdateClient> for IbcEvent {
	fn from(v: UpdateClient) -> Self {
		IbcEvent::UpdateClient(v)
	}
}

impl core::fmt::Display for UpdateClient {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		write!(f, "{}", self.common)
	}
}

impl core::fmt::Debug for UpdateClient {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{}", self.common)
	}
}

/// ClientMisbehaviour event signals the update of an on-chain client (IBC Client) with evidence of
/// misbehaviour.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ClientMisbehaviour(pub Attributes);

impl ClientMisbehaviour {
	pub fn client_id(&self) -> &ClientId {
		&self.0.client_id
	}
	pub fn height(&self) -> Height {
		self.0.height
	}
	pub fn set_height(&mut self, height: Height) {
		self.0.height = height;
	}
}

impl From<Attributes> for ClientMisbehaviour {
	fn from(attrs: Attributes) -> Self {
		ClientMisbehaviour(attrs)
	}
}

impl From<ClientMisbehaviour> for IbcEvent {
	fn from(v: ClientMisbehaviour) -> Self {
		IbcEvent::ClientMisbehaviour(v)
	}
}

/// Signals a recent upgrade of an on-chain client (IBC Client).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct UpgradeClient(pub Attributes);

impl UpgradeClient {
	pub fn set_height(&mut self, height: Height) {
		self.0.height = height;
	}
	pub fn height(&self) -> Height {
		self.0.height
	}
	pub fn client_id(&self) -> &ClientId {
		&self.0.client_id
	}
}

impl From<Attributes> for UpgradeClient {
	fn from(attrs: Attributes) -> Self {
		UpgradeClient(attrs)
	}
}

impl From<UpgradeClient> for IbcEvent {
	fn from(v: UpgradeClient) -> Self {
		IbcEvent::UpgradeClient(v)
	}
}

pub type CodeId = Vec<u8>;

/// Signals a recent pushed WASM code to the chain.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct PushWasmCode(pub CodeId);

impl From<PushWasmCode> for IbcEvent {
	fn from(v: PushWasmCode) -> Self {
		IbcEvent::PushWasmCode(v)
	}
}
