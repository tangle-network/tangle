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

use crate::{
	context::Context,
	contract::{CLIENT_COUNTER, CONSENSUS_STATES_HEIGHTS, HOST_CONSENSUS_STATE},
	ics23::{
		ClientStates, ConsensusStates, FakeInner, ReadonlyClientStates, ReadonlyClients,
		ReadonlyConsensusStates,
	},
	log,
};
use grandpa_light_client_primitives::RelayHostFunctions;
use ibc::{
	core::{
		ics02_client::{
			client_consensus::ConsensusState as _,
			client_state::ClientType,
			context::{ClientKeeper, ClientReader, ClientTypes},
			error::Error,
		},
		ics24_host::identifier::ClientId,
	},
	protobuf::Protobuf,
	timestamp::Timestamp,
	Height,
};
use ibc_proto::google::protobuf::Any;
use ics10_grandpa::{
	client_def::GrandpaClient,
	client_message::{ClientMessage, RelayChainHeader},
	client_state::ClientState,
	consensus_state::ConsensusState,
};
use prost::Message;
use std::str::FromStr;

impl<'a, H: RelayHostFunctions<Header = RelayChainHeader>> ClientTypes for Context<'a, H> {
	type AnyClientMessage = ClientMessage;
	type AnyClientState = ClientState<H>;
	type AnyConsensusState = ConsensusState;
	type ClientDef = GrandpaClient<H>;
}

impl<'a, H: RelayHostFunctions<Header = RelayChainHeader>> ClientReader for Context<'a, H> {
	fn client_type(&self, client_id: &ClientId) -> Result<ClientType, Error> {
		log!(self, "in client : [client_type] >> client_id = {:?}", client_id);

		let clients = ReadonlyClients::new(self.storage());
		if !clients.contains_key(client_id) {
			log!(self, "in client : [client_type] >> read client_type is None");
			return Err(Error::client_not_found(client_id.clone()));
		}

		let data = clients
			.get(client_id)
			.ok_or_else(|| Error::client_not_found(client_id.clone()))?;
		let data = String::from_utf8(data).map_err(|e| {
			Error::implementation_specific(format!(
				"[client_type]: error decoding client type bytes to string {e}"
			))
		})?;
		match ClientType::from_str(&data) {
			Err(_err) => Err(Error::unknown_client_type(data.to_string())),
			Ok(val) => {
				log!(self, "in client : [client_type] >> client_type : {:?}", val);
				Ok(val)
			},
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Result<ClientState<H>, Error> {
		log!(self, "in client : [client_state] >> client_id = {:?}", client_id);
		let client_states = ReadonlyClientStates::new(self.storage());
		let data = client_states.get().ok_or_else(|| Error::client_not_found(client_id.clone()))?;
		let state = Self::decode_client_state(&data)?;
		log!(self, "in client : [client_state] >> any client_state: {:?}", state);
		Ok(state)
	}

	fn consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<ConsensusState, Error> {
		log!(self, "in client : [consensus_state] >> height = {:?}", height);

		let consensus_states = ReadonlyConsensusStates::new(self.storage());
		let value = consensus_states
			.get(height)
			.ok_or_else(|| Error::consensus_state_not_found(client_id.clone(), height))?;
		log!(
			self,
			"in client : [consensus_state] >> consensus_state (raw): {}",
			hex::encode(&value)
		);
		let any_consensus_state = Self::decode_consensus_state(&value)?;
		log!(
			self,
			"in client : [consensus_state] >> any consensus state = {:?}",
			any_consensus_state
		);
		Ok(any_consensus_state)
	}

	fn host_client_type(&self) -> String {
		"ics10_grandpa".to_string()
	}

	fn next_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Option<ConsensusState>, Error> {
		CONSENSUS_STATES_HEIGHTS
			.load(self.storage(), client_id.as_bytes().to_owned())
			.unwrap_or_default()
			.range(height..)
			.next()
			.map(|height| self.consensus_state(client_id, *height))
			.transpose()
	}

	fn prev_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Option<ConsensusState>, Error> {
		CONSENSUS_STATES_HEIGHTS
			.load(self.storage(), client_id.as_bytes().to_owned())
			.unwrap_or_default()
			.range(..height)
			.next_back()
			.map(|height| self.consensus_state(client_id, *height))
			.transpose()
	}

	fn host_height(&self) -> Height {
		log!(self, "in client: [host_height]");
		Height::new(self.env.block.height, 0)
	}

	fn host_timestamp(&self) -> Timestamp {
		let time = self.env.block.time;
		Timestamp::from_nanoseconds(time.nanos()).expect("invalid timestamp")
	}

	fn host_consensus_state(
		&self,
		height: Height,
		_proof: Option<Vec<u8>>,
		_client_state: &ClientState<H>,
	) -> Result<ConsensusState, Error> {
		let consensus_state =
			HOST_CONSENSUS_STATE.load(self.storage(), height.revision_height).map_err(|_| {
				Error::implementation_specific(format!(
					"[host_consensus_state]: consensus state not found for host at height {height}"
				))
			})?;
		Ok(consensus_state)
	}

	fn client_counter(&self) -> Result<u64, Error> {
		let count = CLIENT_COUNTER.load(self.storage()).unwrap_or_default();
		log!(self, "in client : [client_counter] >> client_counter: {:?}", count);
		Ok(count as u64)
	}
}

impl<'a, H: RelayHostFunctions<Header = RelayChainHeader>> ClientKeeper for Context<'a, H> {
	fn store_client_type(
		&mut self,
		_client_id: ClientId,
		_client_type: ClientType,
	) -> Result<(), Error> {
		unimplemented!()
	}

	fn store_client_state(
		&mut self,
		client_id: ClientId,
		client_state: Self::AnyClientState,
	) -> Result<(), Error> {
		log!(self, "in client : [store_client_state]");
		let client_states = ReadonlyClientStates::new(self.storage());
		let data = client_states.get().ok_or_else(|| Error::client_not_found(client_id.clone()))?;
		let vec1 = Self::encode_client_state(client_state, data)?;
		log!(self, "in cliden : [store_client_state] >> wasm client state (raw)");
		let mut client_state_storage = ClientStates::new(self.storage_mut());
		client_state_storage.insert(vec1);
		Ok(())
	}

	fn store_consensus_state(
		&mut self,
		client_id: ClientId,
		height: Height,
		consensus_state: Self::AnyConsensusState,
	) -> Result<(), Error> {
		log!(
			self,
			"in client : [store_consensus_state] >> client_id = {:?}, height = {:?}",
			client_id,
			height,
		);

		let encoded = Self::encode_consensus_state(consensus_state);
		log!(
			self,
			"in client : [store_consensus_state] >> wasm consensus state (raw) = {}",
			hex::encode(&encoded)
		);
		let mut consensus_states = ConsensusStates::new(self.storage_mut());
		consensus_states.insert(height, encoded);
		Ok(())
	}

	fn increase_client_counter(&mut self) {
		unimplemented!()
	}

	fn store_update_time(
		&mut self,
		_client_id: ClientId,
		_height: Height,
		_timestamp: Timestamp,
	) -> Result<(), Error> {
		unimplemented!()
	}

	fn store_update_height(
		&mut self,
		_client_id: ClientId,
		_height: Height,
		_host_height: Height,
	) -> Result<(), Error> {
		unimplemented!()
	}

	fn validate_self_client(&self, _client_state: &Self::AnyClientState) -> Result<(), Error> {
		unimplemented!()
	}
}

impl<'a, H: Clone> Context<'a, H> {
	pub fn decode_client_state(data: &[u8]) -> Result<ClientState<H>, Error> {
		let any = Any::decode(data).map_err(Error::decode)?;
		let wasm_state =
			ics08_wasm::client_state::ClientState::<FakeInner, FakeInner, FakeInner>::decode_vec(
				&any.value,
			)
			.map_err(|e| {
				Error::implementation_specific(format!(
					"[client_state]: error decoding client state bytes to WasmConsensusState {e}"
				))
			})?;
		let any = Any::decode(&*wasm_state.data).map_err(Error::decode)?;
		let state =
			ClientState::<H>::decode_vec(&any.value).map_err(Error::invalid_any_client_state)?;
		Ok(state)
	}

	pub fn decode_consensus_state(value: &[u8]) -> Result<ConsensusState, Error> {
		let any = Any::decode(&mut &*value).map_err(Error::decode)?;
		let wasm_consensus_state =
			ics08_wasm::consensus_state::ConsensusState::<FakeInner>::decode_vec(&any.value)
				.map_err(Error::invalid_any_consensus_state)?;
		let any = Any::decode(&mut &wasm_consensus_state.data[..]).map_err(Error::decode)?;
		let any_consensus_state =
			ConsensusState::decode_vec(&any.value).map_err(Error::invalid_any_consensus_state)?;
		Ok(any_consensus_state)
	}

	pub fn encode_client_state(
		client_state: ClientState<H>,
		encoded_wasm_client_state: Vec<u8>,
	) -> Result<Vec<u8>, Error> {
		let any = Any::decode(&*encoded_wasm_client_state).map_err(Error::decode)?;
		let mut wasm_client_state =
			ics08_wasm::client_state::ClientState::<FakeInner, FakeInner, FakeInner>::decode_vec(
				&any.value,
			)
			.map_err(|e| {
				Error::implementation_specific(format!(
					"[client_state]: error decoding client state bytes to WasmConsensusState {e}"
				))
			})?;
		wasm_client_state.data = client_state.to_any().encode_to_vec();
		wasm_client_state.latest_height = client_state.latest_height();
		let vec1 = wasm_client_state.to_any().encode_to_vec();
		Ok(vec1)
	}

	pub fn encode_consensus_state(consensus_state: ConsensusState) -> Vec<u8> {
		let wasm_consensus_state = ics08_wasm::consensus_state::ConsensusState {
			data: consensus_state.to_any().encode_to_vec(),
			timestamp: consensus_state.timestamp().nanoseconds(),
			inner: Box::new(FakeInner),
		};
		wasm_consensus_state.to_any().encode_to_vec()
	}
}
