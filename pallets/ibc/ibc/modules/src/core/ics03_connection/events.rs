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

//! Types for the IBC events emitted from Tendermint Websocket by the connection module.

use serde_derive::{Deserialize, Serialize};
use tendermint::abci::{Event as AbciEvent, EventAttribute};

use crate::{
	core::{
		ics02_client::{error::Error as Ics02Error, height::Height},
		ics03_connection::error::Error,
		ics24_host::identifier::{ClientId, ConnectionId},
	},
	events::{IbcEvent, IbcEventType},
	prelude::*,
};

/// The content of the `key` field for the attribute containing the connection identifier.
pub const HEIGHT_ATTRIBUTE_KEY: &str = "height";
pub const CONN_ID_ATTRIBUTE_KEY: &str = "connection_id";
pub const CLIENT_ID_ATTRIBUTE_KEY: &str = "client_id";
pub const COUNTERPARTY_CONN_ID_ATTRIBUTE_KEY: &str = "counterparty_connection_id";
pub const COUNTERPARTY_CLIENT_ID_ATTRIBUTE_KEY: &str = "counterparty_client_id";

pub fn try_from_tx(event: &tendermint::abci::Event) -> Option<IbcEvent> {
	match event.kind.parse() {
		Ok(IbcEventType::OpenInitConnection) => extract_attributes_from_tx(event)
			.map(OpenInit::from)
			.map(IbcEvent::OpenInitConnection)
			.ok(),
		Ok(IbcEventType::OpenTryConnection) => extract_attributes_from_tx(event)
			.map(OpenTry::from)
			.map(IbcEvent::OpenTryConnection)
			.ok(),
		Ok(IbcEventType::OpenAckConnection) => extract_attributes_from_tx(event)
			.map(OpenAck::from)
			.map(IbcEvent::OpenAckConnection)
			.ok(),
		Ok(IbcEventType::OpenConfirmConnection) => extract_attributes_from_tx(event)
			.map(OpenConfirm::from)
			.map(IbcEvent::OpenConfirmConnection)
			.ok(),
		_ => None,
	}
}

fn extract_attributes_from_tx(event: &tendermint::abci::Event) -> Result<Attributes, Error> {
	let mut attr = Attributes::default();

	for tag in &event.attributes {
		let key = tag.key.as_str();
		let value = tag.value.as_str();
		match key {
			HEIGHT_ATTRIBUTE_KEY => {
				attr.height = value.parse().map_err(|e| {
					Error::ics02_client(Ics02Error::invalid_string_as_height(value.to_string(), e))
				})?;
			},
			CONN_ID_ATTRIBUTE_KEY => {
				attr.connection_id = value.parse().ok();
			},
			CLIENT_ID_ATTRIBUTE_KEY => {
				attr.client_id = value.parse().map_err(Error::invalid_identifier)?;
			},
			COUNTERPARTY_CONN_ID_ATTRIBUTE_KEY => {
				attr.counterparty_connection_id = value.parse().ok();
			},
			COUNTERPARTY_CLIENT_ID_ATTRIBUTE_KEY => {
				attr.counterparty_client_id = value.parse().map_err(Error::invalid_identifier)?;
			},
			_ => {},
		}
	}

	Ok(attr)
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Attributes {
	pub height: Height,
	pub connection_id: Option<ConnectionId>,
	pub client_id: ClientId,
	pub counterparty_connection_id: Option<ConnectionId>,
	pub counterparty_client_id: ClientId,
}

/// Convert attributes to Tendermint ABCI tags
impl From<Attributes> for Vec<EventAttribute> {
	fn from(a: Attributes) -> Self {
		let mut attributes = vec![];
		let height = EventAttribute {
			key: HEIGHT_ATTRIBUTE_KEY.to_string(),
			value: a.height.to_string(),
			index: false,
		};
		attributes.push(height);
		if let Some(conn_id) = a.connection_id {
			let conn_id = EventAttribute {
				key: CONN_ID_ATTRIBUTE_KEY.to_string(),
				value: conn_id.to_string(),
				index: false,
			};
			attributes.push(conn_id);
		}
		let client_id = EventAttribute {
			key: CLIENT_ID_ATTRIBUTE_KEY.to_string(),
			value: a.client_id.to_string(),
			index: false,
		};
		attributes.push(client_id);
		if let Some(conn_id) = a.counterparty_connection_id {
			let conn_id = EventAttribute {
				key: COUNTERPARTY_CONN_ID_ATTRIBUTE_KEY.to_string(),
				value: conn_id.to_string(),
				index: false,
			};
			attributes.push(conn_id);
		}
		let counterparty_client_id = EventAttribute {
			key: COUNTERPARTY_CLIENT_ID_ATTRIBUTE_KEY.to_string(),
			value: a.counterparty_client_id.to_string(),
			index: false,
		};
		attributes.push(counterparty_client_id);
		attributes
	}
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct OpenInit(pub Attributes);

impl OpenInit {
	pub fn attributes(&self) -> &Attributes {
		&self.0
	}
	pub fn connection_id(&self) -> Option<&ConnectionId> {
		self.0.connection_id.as_ref()
	}
	pub fn height(&self) -> Height {
		self.0.height
	}
	pub fn set_height(&mut self, height: Height) {
		self.0.height = height;
	}
}

impl From<Attributes> for OpenInit {
	fn from(attrs: Attributes) -> Self {
		OpenInit(attrs)
	}
}

impl From<OpenInit> for IbcEvent {
	fn from(v: OpenInit) -> Self {
		IbcEvent::OpenInitConnection(v)
	}
}

impl From<OpenInit> for AbciEvent {
	fn from(v: OpenInit) -> Self {
		let attributes = Vec::<EventAttribute>::from(v.0);
		AbciEvent { kind: IbcEventType::OpenInitConnection.as_str().to_string(), attributes }
	}
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct OpenTry(pub Attributes);

impl OpenTry {
	pub fn attributes(&self) -> &Attributes {
		&self.0
	}
	pub fn connection_id(&self) -> Option<&ConnectionId> {
		self.0.connection_id.as_ref()
	}
	pub fn height(&self) -> Height {
		self.0.height
	}
	pub fn set_height(&mut self, height: Height) {
		self.0.height = height;
	}
}

impl From<Attributes> for OpenTry {
	fn from(attrs: Attributes) -> Self {
		OpenTry(attrs)
	}
}

impl From<OpenTry> for IbcEvent {
	fn from(v: OpenTry) -> Self {
		IbcEvent::OpenTryConnection(v)
	}
}

impl From<OpenTry> for AbciEvent {
	fn from(v: OpenTry) -> Self {
		let attributes = Vec::<EventAttribute>::from(v.0);
		AbciEvent { kind: IbcEventType::OpenTryConnection.as_str().to_string(), attributes }
	}
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct OpenAck(pub Attributes);

impl OpenAck {
	pub fn attributes(&self) -> &Attributes {
		&self.0
	}
	pub fn connection_id(&self) -> Option<&ConnectionId> {
		self.0.connection_id.as_ref()
	}
	pub fn height(&self) -> Height {
		self.0.height
	}
	pub fn set_height(&mut self, height: Height) {
		self.0.height = height;
	}
}

impl From<Attributes> for OpenAck {
	fn from(attrs: Attributes) -> Self {
		OpenAck(attrs)
	}
}

impl From<OpenAck> for IbcEvent {
	fn from(v: OpenAck) -> Self {
		IbcEvent::OpenAckConnection(v)
	}
}

impl From<OpenAck> for AbciEvent {
	fn from(v: OpenAck) -> Self {
		let attributes = Vec::<EventAttribute>::from(v.0);
		AbciEvent { kind: IbcEventType::OpenAckConnection.as_str().to_string(), attributes }
	}
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct OpenConfirm(pub Attributes);

impl OpenConfirm {
	pub fn attributes(&self) -> &Attributes {
		&self.0
	}
	pub fn connection_id(&self) -> Option<&ConnectionId> {
		self.0.connection_id.as_ref()
	}
	pub fn height(&self) -> Height {
		self.0.height
	}
	pub fn set_height(&mut self, height: Height) {
		self.0.height = height;
	}
}

impl From<Attributes> for OpenConfirm {
	fn from(attrs: Attributes) -> Self {
		OpenConfirm(attrs)
	}
}

impl From<OpenConfirm> for IbcEvent {
	fn from(v: OpenConfirm) -> Self {
		IbcEvent::OpenConfirmConnection(v)
	}
}

impl From<OpenConfirm> for AbciEvent {
	fn from(v: OpenConfirm) -> Self {
		let attributes = Vec::<EventAttribute>::from(v.0);
		AbciEvent { kind: IbcEventType::OpenConfirmConnection.as_str().to_string(), attributes }
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn connection_event_to_abci_event() {
		let height = Height::new(1, 1);
		let attributes = Attributes {
			height,
			connection_id: Some("test_connection".parse().unwrap()),
			client_id: "test_client".parse().unwrap(),
			counterparty_connection_id: Some("counterparty_test_conn".parse().unwrap()),
			counterparty_client_id: "counterparty_test_client".parse().unwrap(),
		};
		let mut abci_events = vec![];
		let open_init = OpenInit::from(attributes.clone());
		abci_events.push(AbciEvent::from(open_init.clone()));
		let open_try = OpenTry::from(attributes.clone());
		abci_events.push(AbciEvent::from(open_try.clone()));
		let open_ack = OpenAck::from(attributes.clone());
		abci_events.push(AbciEvent::from(open_ack.clone()));
		let open_confirm = OpenConfirm::from(attributes);
		abci_events.push(AbciEvent::from(open_confirm.clone()));

		for event in abci_events {
			match try_from_tx(&event) {
				Some(e) => match e {
					IbcEvent::OpenInitConnection(e) => assert_eq!(e.0, open_init.0),
					IbcEvent::OpenTryConnection(e) => assert_eq!(e.0, open_try.0),
					IbcEvent::OpenAckConnection(e) => assert_eq!(e.0, open_ack.0),
					IbcEvent::OpenConfirmConnection(e) => assert_eq!(e.0, open_confirm.0),
					_ => panic!("unexpected event type"),
				},
				None => panic!("converted event was wrong"),
			}
		}
	}
}
