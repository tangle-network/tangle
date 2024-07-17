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
use core::fmt::Display;
#[cfg(feature = "cosmwasm")]
use cosmwasm_schema::cw_serde;
use ibc::{
	core::ics02_client::{client_message::ClientMessage as IbcClientMessage, error::Error},
	protobuf::Protobuf,
	Height,
};
use ibc_proto::{
	google::protobuf::Any,
	ibc::lightclients::wasm::v1::{Header as RawHeader, Misbehaviour as RawMisbehaviour},
};
use prost::Message;

pub const WASM_HEADER_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.Header";
pub const WASM_MISBEHAVIOUR_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.Misbehaviour";
pub const WASM_CLIENT_MESSAGE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.ClientMessage";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientMessage<AnyClientMessage> {
	Header(Header<AnyClientMessage>),
	Misbehaviour(Misbehaviour<AnyClientMessage>),
}

impl<AnyClientMessage> ClientMessage<AnyClientMessage> {
	pub fn inner(&self) -> &AnyClientMessage {
		match self {
			ClientMessage::Header(h) => &h.inner,
			ClientMessage::Misbehaviour(m) => &m.inner,
		}
	}

	pub fn into_inner(self) -> AnyClientMessage {
		match self {
			ClientMessage::Header(h) => *h.inner,
			ClientMessage::Misbehaviour(m) => *m.inner,
		}
	}
}

impl<AnyClientMessage> IbcClientMessage for ClientMessage<AnyClientMessage>
where
	AnyClientMessage: IbcClientMessage,
	AnyClientMessage: TryFrom<Any>,
	<AnyClientMessage as TryFrom<Any>>::Error: Display,
{
	fn encode_to_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error> {
		self.encode_vec()
	}
}

impl<AnyClientMessage> Protobuf<Any> for ClientMessage<AnyClientMessage>
where
	AnyClientMessage: Clone,
	AnyClientMessage: TryFrom<Any>,
	<AnyClientMessage as TryFrom<Any>>::Error: Display,
{
}

impl<AnyClientMessage> TryFrom<Any> for ClientMessage<AnyClientMessage>
where
	AnyClientMessage: Clone,
	AnyClientMessage: TryFrom<Any>,
	<AnyClientMessage as TryFrom<Any>>::Error: Display,
{
	type Error = Error;

	fn try_from(any: Any) -> Result<Self, Self::Error> {
		let msg = match &*any.type_url {
			WASM_HEADER_TYPE_URL =>
				Self::Header(Header::decode(&*any.value).map_err(Error::decode_raw_header)?),
			WASM_MISBEHAVIOUR_TYPE_URL => Self::Misbehaviour(
				Misbehaviour::decode(&*any.value).map_err(Error::decode_raw_misbehaviour)?,
			),
			_ => return Err(Error::malformed_header()), // TODO: choose a better error
		};

		Ok(msg)
	}
}

impl<AnyClientMessage> From<ClientMessage<AnyClientMessage>> for Any
where
	AnyClientMessage: Clone,
	AnyClientMessage: TryFrom<Any>,
	<AnyClientMessage as TryFrom<Any>>::Error: Display,
{
	fn from(msg: ClientMessage<AnyClientMessage>) -> Self {
		match msg {
			ClientMessage::Header(header) => Any {
				value: header.encode_vec().expect("encode header"),
				type_url: WASM_HEADER_TYPE_URL.to_string(),
			},
			ClientMessage::Misbehaviour(misbheaviour) => Any {
				value: misbheaviour.encode_vec().expect("encode misbehaviour"),
				type_url: WASM_MISBEHAVIOUR_TYPE_URL.to_string(),
			},
		}
	}
}

impl<AnyClientMessage> Protobuf<RawMisbehaviour> for Misbehaviour<AnyClientMessage>
where
	AnyClientMessage: Clone,
	AnyClientMessage: TryFrom<Any>,
	<AnyClientMessage as TryFrom<Any>>::Error: Display,
{
}

impl<AnyClientMessage> TryFrom<RawMisbehaviour> for Misbehaviour<AnyClientMessage>
where
	AnyClientMessage: TryFrom<Any>,
	<AnyClientMessage as TryFrom<Any>>::Error: Display,
{
	type Error = String;

	fn try_from(raw: RawMisbehaviour) -> Result<Self, Self::Error> {
		let any = Any::decode(&mut &raw.data[..]).map_err(|e| e.to_string())?;
		let inner = AnyClientMessage::try_from(any).map_err(|e| e.to_string())?;
		Ok(Self { inner: Box::new(inner), data: raw.data })
	}
}

impl<AnyClientMessage> Protobuf<RawHeader> for Header<AnyClientMessage>
where
	AnyClientMessage: Clone,
	AnyClientMessage: TryFrom<Any>,
	<AnyClientMessage as TryFrom<Any>>::Error: Display,
{
}

impl<AnyClientMessage> TryFrom<RawHeader> for Header<AnyClientMessage>
where
	AnyClientMessage: TryFrom<Any>,
	<AnyClientMessage as TryFrom<Any>>::Error: Display,
{
	type Error = String;

	fn try_from(raw: RawHeader) -> Result<Self, Self::Error> {
		let any = Any::decode(&mut &raw.data[..])
			.map_err(|e| format!("failed to decode raw header into Any: {e}"))?;
		let inner = AnyClientMessage::try_from(any)
			.map_err(|e| format!("failed to decode raw header into AnyClientMessage: {e}"))?;

		let header = Self {
			inner: Box::new(inner),
			data: raw.data,
			height: raw
				.height
				.ok_or_else(|| {
					"failed to decode raw header into Header: missing height".to_string()
				})?
				.into(),
		};
		Ok(header)
	}
}

#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Clone, Debug, PartialEq))]
#[derive(Eq)]
pub struct Header<AnyClientMessage> {
	#[cfg_attr(feature = "cosmwasm", serde(skip))]
	#[cfg_attr(feature = "cosmwasm", schemars(skip))]
	pub inner: Box<AnyClientMessage>,
	#[cfg_attr(feature = "cosmwasm", schemars(with = "String"))]
	#[cfg_attr(feature = "cosmwasm", serde(with = "Base64", default))]
	pub data: Bytes,
	pub height: Height,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Misbehaviour<AnyClientMessage> {
	// #[schemars(with = "String")]
	// #[serde(with = "Base64", default)]
	pub inner: Box<AnyClientMessage>,
	pub data: Bytes,
}

impl<AnyClientMessage> From<Misbehaviour<AnyClientMessage>> for RawMisbehaviour {
	fn from(value: Misbehaviour<AnyClientMessage>) -> Self {
		RawMisbehaviour { data: value.data }
	}
}

impl<AnyClientMessage> From<Header<AnyClientMessage>> for RawHeader {
	fn from(value: Header<AnyClientMessage>) -> Self {
		RawHeader { data: value.data, height: Some(value.height.into()) }
	}
}
