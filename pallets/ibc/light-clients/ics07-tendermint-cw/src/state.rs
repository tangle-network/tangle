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

use crate::ics23::FakeInner;
use cosmwasm_std::Deps;
use ibc::{
	core::{ics02_client::error::Error, ics24_host::identifier::ClientId},
	protobuf::Protobuf,
	Height,
};
use ibc_proto::google::protobuf::Any;
use ics07_tendermint::{client_state::ClientState, consensus_state::ConsensusState};
use prost::Message;

/// Retrieves raw bytes from storage and deserializes them into [`ClientState`]
pub fn get_client_state<H: Clone>(deps: Deps) -> Result<ClientState<H>, Error> {
	deps.storage
		.get(&"clientState".to_string().into_bytes())
		.ok_or_else(|| Error::unknown_client_state_type("08-wasm-0".to_string()))
		.and_then(|client_state| deserialize_client_state(client_state, deps))
}

fn deserialize_client_state<H: Clone>(
	client_state: Vec<u8>,
	_deps: Deps,
) -> Result<ClientState<H>, Error> {
	let any = Any::decode(&*client_state).map_err(Error::decode)?;

	let wasm_state =
		ics08_wasm::client_state::ClientState::<FakeInner, FakeInner, FakeInner>::decode_vec(
			&any.value,
		)
		.map_err(|e| {
			Error::implementation_specific(format!(
				"[client_state]: error decoding client state bytes to WasmClientState {e}"
			))
		})?;
	let any = Any::decode(&*wasm_state.data).map_err(Error::decode)?;
	let state =
		ClientState::<H>::decode_vec(&any.value).map_err(Error::invalid_any_client_state)?;
	Ok(state)
}

pub fn get_consensus_state(
	deps: Deps,
	client_id: &ClientId,
	height: Height,
) -> Result<ConsensusState, Error> {
	deps.storage
		.get(&get_consensus_state_key(height))
		.ok_or_else(|| Error::consensus_state_not_found(client_id.clone(), height))
		.and_then(deserialize_consensus_state)
}

fn deserialize_consensus_state(consensus_state: Vec<u8>) -> Result<ConsensusState, Error> {
	let any = Any::decode(&*consensus_state).map_err(Error::decode)?;
	let wasm_consensus_state =
		ics08_wasm::consensus_state::ConsensusState::<FakeInner>::decode_vec(&any.value).map_err(
			|e| {
				Error::implementation_specific(format!(
				"[consensus_state]: error decoding consensus state bytes to WasmConsensusState {e}"
			))
			},
		)?;
	let any = Any::decode(&*wasm_consensus_state.data).map_err(Error::decode)?;
	let consensus =
		ConsensusState::decode_vec(&any.value).map_err(Error::invalid_any_consensus_state)?;
	Ok(consensus)
}

pub fn get_consensus_state_key(height: Height) -> Vec<u8> {
	["consensusStates/".to_string().into_bytes(), format!("{height}").into_bytes()].concat()
}
