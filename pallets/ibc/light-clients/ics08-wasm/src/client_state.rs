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
use crate::{client_def::WasmClient, Bytes};
use alloc::{
	boxed::Box,
	string::{String, ToString},
	vec::Vec,
};
use core::{
	fmt::{Debug, Display},
	marker::PhantomData,
	time::Duration,
};
#[cfg(feature = "cosmwasm")]
use cosmwasm_schema::cw_serde;
use ibc::{
	core::{
		ics02_client::{
			client_consensus::ConsensusState as IbcConsensusState, client_def::ClientDef,
			client_state::ClientState as IbcClientState,
		},
		ics24_host::identifier::ChainId,
	},
	protobuf::Protobuf,
	Height,
};
use ibc_proto::{
	google::protobuf::Any, ibc::lightclients::wasm::v1::ClientState as RawClientState,
};
use prost::Message;

pub const WASM_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.ClientState";

#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Clone, Debug, PartialEq))]
#[derive(Eq)]
pub struct ClientState<AnyClient, AnyClientState, AnyConsensusState> {
	#[cfg_attr(feature = "cosmwasm", schemars(with = "String"))]
	#[cfg_attr(feature = "cosmwasm", serde(with = "Base64", default))]
	pub data: Bytes,
	#[cfg_attr(feature = "cosmwasm", schemars(with = "String"))]
	#[cfg_attr(feature = "cosmwasm", serde(with = "Base64", default))]
	pub code_id: Bytes,
	pub latest_height: Height,
	#[cfg_attr(feature = "cosmwasm", serde(skip))]
	#[cfg_attr(feature = "cosmwasm", schemars(skip))]
	pub inner: Box<AnyClientState>,
	#[cfg_attr(feature = "cosmwasm", serde(skip))]
	#[cfg_attr(feature = "cosmwasm", schemars(skip))]
	pub _phantom: PhantomData<(AnyConsensusState, AnyClient)>,
}

impl<AnyClient, AnyClientState, AnyConsensusState> IbcClientState
	for ClientState<AnyClient, AnyClientState, AnyConsensusState>
where
	AnyClientState: TryFrom<Any>,
	<AnyClientState as TryFrom<Any>>::Error: Display,
	AnyConsensusState: IbcConsensusState + Eq,
	AnyConsensusState: TryFrom<Any>,
	<AnyConsensusState as TryFrom<Any>>::Error: Display,
	AnyClient: ClientDef<ClientState = AnyClientState, ConsensusState = AnyConsensusState>
		+ Debug
		+ Send
		+ Sync
		+ Eq,
	AnyClientState: IbcClientState<ClientDef = AnyClient> + Eq,
	AnyClient::ClientMessage: TryFrom<Any>,
	<AnyClient::ClientMessage as TryFrom<Any>>::Error: Display,
{
	type UpgradeOptions = Box<AnyClientState::UpgradeOptions>;
	type ClientDef = WasmClient<AnyClient, AnyClientState, AnyConsensusState>;

	fn chain_id(&self) -> ChainId {
		// self.inner.chain_id()
		self.inner.chain_id()
	}

	fn client_def(&self) -> Self::ClientDef {
		let inner = self.inner.client_def();
		WasmClient { inner: Box::new(inner), _phantom: Default::default() }
	}

	fn client_type(&self) -> String {
		"08-wasm".to_string()
	}

	fn latest_height(&self) -> Height {
		self.latest_height
	}

	fn frozen_height(&self) -> Option<Height> {
		self.inner.frozen_height()
	}

	fn upgrade(
		self,
		upgrade_height: Height,
		upgrade_options: Self::UpgradeOptions,
		chain_id: ChainId,
	) -> Self {
		let inner = self.inner.upgrade(upgrade_height, *upgrade_options, chain_id);
		Self { inner: Box::new(inner), ..self }
	}

	fn expired(&self, elapsed: Duration) -> bool {
		self.inner.expired(elapsed)
	}

	fn encode_to_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error> {
		self.encode_vec()
	}
}

impl<AnyClient, AnyClientState, AnyConsensusState>
	ClientState<AnyClient, AnyClientState, AnyConsensusState>
where
	AnyClientState: TryFrom<Any>,
	<AnyClientState as TryFrom<Any>>::Error: Display,
	AnyConsensusState: IbcConsensusState + Eq,
	AnyConsensusState: TryFrom<Any>,
	<AnyConsensusState as TryFrom<Any>>::Error: Display,
	AnyClient: ClientDef<ClientState = AnyClientState, ConsensusState = AnyConsensusState>
		+ Debug
		+ Send
		+ Sync
		+ Eq,
	AnyClientState: IbcClientState<ClientDef = AnyClient> + Eq,
	AnyClient::ClientMessage: TryFrom<Any>,
	<AnyClient::ClientMessage as TryFrom<Any>>::Error: Display,
{
	pub fn to_any(&self) -> Any {
		Any {
			type_url: WASM_CLIENT_STATE_TYPE_URL.to_string(),
			value: self
				.encode_to_vec()
				.expect("ClientState<AnyClientState> is always valid and can be encoded to Any"),
		}
	}
}

impl<AnyClient, AnyClientState, AnyConsensusState> TryFrom<RawClientState>
	for ClientState<AnyClient, AnyClientState, AnyConsensusState>
where
	AnyClientState: TryFrom<Any>,
	<AnyClientState as TryFrom<Any>>::Error: Display,
{
	type Error = String;

	fn try_from(raw: RawClientState) -> Result<Self, Self::Error> {
		let any = Any::decode(&mut &raw.data[..]).map_err(|e| e.to_string())?;
		let inner = AnyClientState::try_from(any).map_err(|e| e.to_string())?;
		Ok(Self {
			data: raw.data,
			code_id: raw.code_id,
			inner: Box::new(inner),
			latest_height: raw
				.latest_height
				.map(|h| Height::new(h.revision_number, h.revision_height))
				.unwrap_or_default(),
			_phantom: Default::default(),
		})
	}
}

impl<AnyClient, AnyClientState, AnyConsensusState>
	From<ClientState<AnyClient, AnyClientState, AnyConsensusState>> for RawClientState
where
	AnyClientState: TryFrom<Any>,
	<AnyClientState as TryFrom<Any>>::Error: Display,
{
	fn from(client_state: ClientState<AnyClient, AnyClientState, AnyConsensusState>) -> Self {
		Self {
			data: client_state.data,
			code_id: client_state.code_id,
			latest_height: Some(client_state.latest_height.into()),
		}
	}
}

impl<AnyClient, AnyClientState, AnyConsensusState> Protobuf<RawClientState>
	for ClientState<AnyClient, AnyClientState, AnyConsensusState>
where
	AnyClientState: Clone,
	AnyClientState: TryFrom<Any>,
	<AnyClientState as TryFrom<Any>>::Error: Display,
	AnyConsensusState: Clone,
	AnyClient: Clone,
{
}
