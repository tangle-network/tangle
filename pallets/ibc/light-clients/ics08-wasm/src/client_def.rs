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
	client_message::ClientMessage, client_state::ClientState, consensus_state::ConsensusState,
};
use alloc::{boxed::Box, vec::Vec};
use core::{
	fmt::{Debug, Display},
	marker::PhantomData,
};
use ibc::{
	core::{
		ics02_client::{
			client_consensus::ConsensusState as IbcConsensusState,
			client_def::{ClientDef, ConsensusUpdateResult},
			client_state::ClientState as IbcClientState,
			error::Error,
		},
		ics03_connection::connection::ConnectionEnd,
		ics04_channel::{
			channel::ChannelEnd,
			commitment::{AcknowledgementCommitment, PacketCommitment},
			packet::Sequence,
		},
		ics23_commitment::commitment::{CommitmentPrefix, CommitmentProofBytes, CommitmentRoot},
		ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
		ics26_routing::context::ReaderContext,
	},
	Height,
};
use ibc_proto::google::protobuf::Any;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WasmClient<AnyClient, AnyClientState, AnyConsensusState> {
	pub inner: Box<AnyClient>,
	pub _phantom: PhantomData<(AnyClientState, AnyConsensusState)>,
}

impl<AnyClient, AnyClientState, AnyConsensusState> ClientDef
	for WasmClient<AnyClient, AnyClientState, AnyConsensusState>
where
	AnyClient: ClientDef<ClientState = AnyClientState, ConsensusState = AnyConsensusState>
		+ Debug
		+ Send
		+ Sync
		+ Eq,
	AnyClientState: Clone + Eq + IbcClientState<ClientDef = AnyClient>,
	AnyClientState: TryFrom<Any>,
	<AnyClientState as TryFrom<Any>>::Error: Display,
	AnyConsensusState: IbcConsensusState + Eq,
	AnyConsensusState: TryFrom<Any>,
	<AnyConsensusState as TryFrom<Any>>::Error: Display,
	AnyClient::ClientMessage: TryFrom<Any>,
	<AnyClient::ClientMessage as TryFrom<Any>>::Error: Display,
{
	type ClientMessage = ClientMessage<AnyClient::ClientMessage>;
	type ClientState = ClientState<AnyClient, AnyClientState, AnyConsensusState>;
	type ConsensusState = ConsensusState<AnyConsensusState>;

	fn verify_client_message<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_id: ClientId,
		client_state: Self::ClientState,
		client_msg: Self::ClientMessage,
	) -> Result<(), Error> {
		self.inner.verify_client_message(
			ctx,
			client_id,
			*client_state.inner,
			client_msg.into_inner(),
		)
	}

	fn update_state<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_id: ClientId,
		client_state: Self::ClientState,
		client_msg: Self::ClientMessage,
	) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Error> {
		let (inner_client_state, inner_consensus_update_result) = self.inner.update_state(
			ctx,
			client_id,
			*client_state.inner,
			client_msg.into_inner(),
		)?;
		let client_state = ClientState {
			data: client_state.data.clone(),
			code_id: client_state.code_id.clone(),
			inner: Box::new(inner_client_state),
			latest_height: client_state.latest_height,
			_phantom: PhantomData,
		};
		Ok((client_state, inner_consensus_update_result))
	}

	fn update_state_on_misbehaviour(
		&self,
		client_state: Self::ClientState,
		client_msg: Self::ClientMessage,
	) -> Result<Self::ClientState, Error> {
		let inner_client_state = self
			.inner
			.update_state_on_misbehaviour(*client_state.inner, client_msg.into_inner())?;
		Ok(ClientState {
			data: client_state.data.clone(),
			code_id: client_state.code_id.clone(),
			inner: Box::new(inner_client_state),
			latest_height: client_state.latest_height,
			_phantom: PhantomData,
		})
	}

	fn check_for_misbehaviour<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_id: ClientId,
		client_state: Self::ClientState,
		client_msg: Self::ClientMessage,
	) -> Result<bool, Error> {
		self.inner.check_for_misbehaviour(
			ctx,
			client_id,
			*client_state.inner,
			client_msg.into_inner(),
		)
	}

	fn verify_upgrade_and_update_state<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_id: ClientId,
		old_client_state: &Self::ClientState,
		upgrade_client_state: &Self::ClientState,
		upgrade_consensus_state: &Self::ConsensusState,
		proof_upgrade_client: Vec<u8>,
		proof_upgrade_consensus_state: Vec<u8>,
	) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Error> {
		self.inner
			.verify_upgrade_and_update_state(
				ctx,
				client_id,
				&old_client_state.inner,
				&upgrade_client_state.inner,
				&upgrade_consensus_state.inner,
				proof_upgrade_client,
				proof_upgrade_consensus_state,
			)
			.map(|(client_state, result)| {
				(
					ClientState {
						inner: Box::new(client_state),
						data: old_client_state.data.clone(),
						code_id: old_client_state.code_id.clone(),
						latest_height: old_client_state.latest_height,
						_phantom: Default::default(),
					},
					result,
				)
			})
	}

	fn check_substitute_and_update_state<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		subject_client_id: ClientId,
		substitute_client_id: ClientId,
		old_client_state: Self::ClientState,
		substitute_client_state: Self::ClientState,
	) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Error> {
		let (inner_client_state, inner_consensus_update_result) =
			self.inner.check_substitute_and_update_state(
				ctx,
				subject_client_id,
				substitute_client_id,
				*old_client_state.inner,
				*substitute_client_state.inner,
			)?;
		let client_state = ClientState {
			data: old_client_state.data.clone(),
			code_id: old_client_state.code_id.clone(),
			inner: Box::new(inner_client_state),
			latest_height: old_client_state.latest_height,
			_phantom: PhantomData,
		};
		Ok((client_state, inner_consensus_update_result))
	}

	fn verify_client_consensus_state<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_state: &Self::ClientState,
		height: Height,
		prefix: &CommitmentPrefix,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		client_id: &ClientId,
		consensus_height: Height,
		expected_consensus_state: &Ctx::AnyConsensusState,
	) -> Result<(), Error> {
		self.inner.verify_client_consensus_state(
			ctx,
			&client_state.inner,
			height,
			prefix,
			proof,
			root,
			client_id,
			consensus_height,
			expected_consensus_state,
		)
	}

	fn verify_connection_state<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_id: &ClientId,
		client_state: &Self::ClientState,
		height: Height,
		prefix: &CommitmentPrefix,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		connection_id: &ConnectionId,
		expected_connection_end: &ConnectionEnd,
	) -> Result<(), Error> {
		self.inner.verify_connection_state(
			ctx,
			client_id,
			&client_state.inner,
			height,
			prefix,
			proof,
			root,
			connection_id,
			expected_connection_end,
		)
	}

	fn verify_channel_state<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_id: &ClientId,
		client_state: &Self::ClientState,
		height: Height,
		prefix: &CommitmentPrefix,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		port_id: &PortId,
		channel_id: &ChannelId,
		expected_channel_end: &ChannelEnd,
	) -> Result<(), Error> {
		self.inner.verify_channel_state(
			ctx,
			client_id,
			&client_state.inner,
			height,
			prefix,
			proof,
			root,
			port_id,
			channel_id,
			expected_channel_end,
		)
	}

	fn verify_client_full_state<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_state: &Self::ClientState,
		height: Height,
		prefix: &CommitmentPrefix,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		client_id: &ClientId,
		expected_client_state: &Ctx::AnyClientState,
	) -> Result<(), Error> {
		self.inner.verify_client_full_state(
			ctx,
			&client_state.inner,
			height,
			prefix,
			proof,
			root,
			client_id,
			expected_client_state,
		)
	}

	fn verify_packet_data<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_id: &ClientId,
		client_state: &Self::ClientState,
		height: Height,
		connection_end: &ConnectionEnd,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: Sequence,
		commitment: PacketCommitment,
	) -> Result<(), Error> {
		self.inner.verify_packet_data(
			ctx,
			client_id,
			&client_state.inner,
			height,
			connection_end,
			proof,
			root,
			port_id,
			channel_id,
			sequence,
			commitment,
		)
	}

	fn verify_packet_acknowledgement<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_id: &ClientId,
		client_state: &Self::ClientState,
		height: Height,
		connection_end: &ConnectionEnd,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: Sequence,
		ack: AcknowledgementCommitment,
	) -> Result<(), Error> {
		self.inner.verify_packet_acknowledgement(
			ctx,
			client_id,
			&client_state.inner,
			height,
			connection_end,
			proof,
			root,
			port_id,
			channel_id,
			sequence,
			ack,
		)
	}

	fn verify_next_sequence_recv<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_id: &ClientId,
		client_state: &Self::ClientState,
		height: Height,
		connection_end: &ConnectionEnd,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: Sequence,
	) -> Result<(), Error> {
		self.inner.verify_next_sequence_recv(
			ctx,
			client_id,
			&client_state.inner,
			height,
			connection_end,
			proof,
			root,
			port_id,
			channel_id,
			sequence,
		)
	}

	fn verify_packet_receipt_absence<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_id: &ClientId,
		client_state: &Self::ClientState,
		height: Height,
		connection_end: &ConnectionEnd,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: Sequence,
	) -> Result<(), Error> {
		self.inner.verify_packet_receipt_absence(
			ctx,
			client_id,
			&client_state.inner,
			height,
			connection_end,
			proof,
			root,
			port_id,
			channel_id,
			sequence,
		)
	}
}
