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

use crate::{
	client_state::{test_util::get_dummy_tendermint_client_state, ClientState},
	mock::{host::MockHostBlock, AnyConsensusState, MockClientTypes},
};
use ibc::{
	core::{ics02_client::client_state::ClientType, ics24_host::identifier::ClientId},
	mock::{
		client_state::{MockClientRecord, MockClientState, MockConsensusState},
		context::MockContext,
		header::{MockClientMessage, MockHeader},
	},
	timestamp::Timestamp,
	Height,
};
use log::debug;
use std::ops::Sub;

/// Similar to `with_client`, this function associates a client record to this context, but
/// additionally permits to parametrize two details of the client. If `client_type` is None,
/// then the client will have type Mock, otherwise the specified type. If
/// `consensus_state_height` is None, then the client will be initialized with a consensus
/// state matching the same height as the client state (`client_state_height`).
pub fn with_client_parametrized(
	ctx: MockContext<MockClientTypes>,
	client_id: &ClientId,
	client_state_height: Height,
	client_type: Option<ClientType>,
	consensus_state_height: Option<Height>,
) -> MockContext<MockClientTypes> {
	let cs_height = consensus_state_height.unwrap_or(client_state_height);

	let client_type = client_type.unwrap_or(MockClientState::client_type());
	let (client_state, consensus_state) = match client_type.clone() {
		// If it's a mock client, create the corresponding mock states.
		client_type if client_type == MockClientState::client_type() => (
			Some(
				MockClientState::new(MockClientMessage::Header(MockHeader::new(
					client_state_height,
				)))
				.into(),
			),
			MockConsensusState::new(MockHeader::new(cs_height)).into(),
		),
		// If it's a Tendermint client, we need TM states.
		client_type if client_type == ClientState::<()>::client_type() => {
			let light_block = MockHostBlock::generate_tm_block(
				ctx.host_chain_id.clone(),
				cs_height.revision_height,
				Timestamp::now(),
			);

			let consensus_state = AnyConsensusState::from(light_block.clone());
			let client_state = get_dummy_tendermint_client_state(light_block.signed_header.header);

			// Return the tuple.
			(Some(client_state), consensus_state)
		},
		_ => unimplemented!(),
	};
	let consensus_states = vec![(cs_height, consensus_state)].into_iter().collect();

	debug!("consensus states: {:?}", consensus_states);

	let client_record = MockClientRecord { client_type, client_state, consensus_states };
	ctx.ibc_store.lock().unwrap().clients.insert(client_id.clone(), client_record);
	ctx
}

pub fn with_client_parametrized_history(
	ctx: MockContext<MockClientTypes>,
	client_id: &ClientId,
	client_state_height: Height,
	client_type: Option<ClientType>,
	consensus_state_height: Option<Height>,
) -> MockContext<MockClientTypes> {
	let cs_height = consensus_state_height.unwrap_or(client_state_height);
	let prev_cs_height = cs_height.clone().sub(1).unwrap_or(client_state_height);

	let client_type = client_type.unwrap_or(MockClientState::client_type());
	let now = Timestamp::now();

	let (client_state, consensus_state) = match client_type.clone() {
		// If it's a mock client, create the corresponding mock states.
		client_type if client_type == MockClientState::client_type() => (
			Some(
				MockClientState::new(MockClientMessage::Header(MockHeader::new(
					client_state_height,
				)))
				.into(),
			),
			MockConsensusState::new(MockHeader::new(cs_height)).into(),
		),
		// If it's a Tendermint client, we need TM states.
		client_type if client_type == ClientState::<()>::client_type() => {
			let light_block = MockHostBlock::generate_tm_block(
				ctx.host_chain_id.clone(),
				cs_height.revision_height,
				now,
			);

			let consensus_state = AnyConsensusState::from(light_block.clone());
			let client_state = get_dummy_tendermint_client_state(light_block.signed_header.header);

			// Return the tuple.
			(Some(client_state), consensus_state)
		},
		_ => unimplemented!(),
	};

	let prev_consensus_state = match client_type.clone() {
		// If it's a mock client, create the corresponding mock states.
		client_type if client_type == MockClientState::client_type() =>
			MockConsensusState::new(MockHeader::new(prev_cs_height)).into(),
		client_type if client_type == ClientState::<()>::client_type() => {
			let light_block = MockHostBlock::generate_tm_block(
				ctx.host_chain_id.clone(),
				prev_cs_height.revision_height,
				now.sub(ctx.block_time).unwrap(),
			);
			AnyConsensusState::from(light_block)
		},
		_ => unimplemented!(),
	};

	let consensus_states =
		vec![(prev_cs_height, prev_consensus_state), (cs_height, consensus_state)]
			.into_iter()
			.collect();

	debug!("consensus states: {:?}", consensus_states);

	let client_record = MockClientRecord { client_type, client_state, consensus_states };

	ctx.ibc_store.lock().unwrap().clients.insert(client_id.clone(), client_record);
	ctx
}
