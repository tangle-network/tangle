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

use beefy_light_client_primitives::{
	ClientState as LightClientState, ParachainHeader, ParachainsUpdateProof,
};
use core::{fmt::Debug, marker::PhantomData};
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_mmr_primitives::Proof;
use tendermint_proto::Protobuf;

use crate::{
	client_message::ClientMessage, client_state::ClientState, consensus_state::ConsensusState,
	error::Error,
};
use ibc::{
	core::{
		ics02_client::{
			client_consensus::ConsensusState as _,
			client_def::{ClientDef, ConsensusUpdateResult},
			client_state::ClientState as _,
			error::Error as Ics02Error,
		},
		ics03_connection::connection::ConnectionEnd,
		ics04_channel::{
			channel::ChannelEnd,
			commitment::{AcknowledgementCommitment, PacketCommitment},
			packet::Sequence,
		},
		ics23_commitment::commitment::{CommitmentPrefix, CommitmentProofBytes, CommitmentRoot},
		ics24_host::{
			identifier::{ChannelId, ClientId, ConnectionId, PortId},
			path::{
				AcksPath, ChannelEndsPath, ClientConsensusStatePath, ClientStatePath,
				CommitmentsPath, ConnectionsPath, ReceiptsPath, SeqRecvsPath,
			},
		},
		ics26_routing::context::ReaderContext,
	},
	prelude::*,
	Height,
};
use light_client_common::{verify_delay_passed, verify_membership, verify_non_membership};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct BeefyClient<T>(PhantomData<T>);

impl<H> ClientDef for BeefyClient<H>
where
	H: light_client_common::HostFunctions + beefy_light_client_primitives::HostFunctions,
{
	type ClientMessage = ClientMessage;
	type ClientState = ClientState<H>;
	type ConsensusState = ConsensusState;

	fn verify_client_message<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_id: ClientId,
		client_state: Self::ClientState,
		message: Self::ClientMessage,
	) -> Result<(), Ics02Error> {
		match message {
			ClientMessage::Header(header) => {
				let light_client_state = LightClientState {
					latest_beefy_height: client_state.latest_beefy_height,
					mmr_root_hash: client_state.mmr_root_hash,
					current_authorities: client_state.authority.clone(),
					next_authorities: client_state.next_authority_set.clone(),
				};
				// If mmr update exists verify it and return the new light client state
				// or else return existing light client state
				let light_client_state = if let Some(mmr_update) = header.mmr_update_proof {
					beefy_light_client::verify_mmr_root_with_proof::<H>(
						light_client_state,
						mmr_update,
					)
					.map_err(Error::from)?
				} else {
					light_client_state
				};

				// Extract parachain headers from the beefy header if they exist
				if let Some(headers_with_proof) = header.headers_with_proof {
					let parachain_headers = headers_with_proof
						.headers
						.into_iter()
						.map(|header| ParachainHeader {
							parachain_header: header.parachain_header.encode(),
							partial_mmr_leaf: header.partial_mmr_leaf,
							para_id: client_state.para_id,
							parachain_heads_proof: header.parachain_heads_proof,
							heads_leaf_index: header.heads_leaf_index,
							heads_total_count: header.heads_total_count,
							extrinsic_proof: header.extrinsic_proof,
							timestamp_extrinsic: header.timestamp_extrinsic,
						})
						.collect::<Vec<_>>();

					let parachain_update_proof = ParachainsUpdateProof {
						parachain_headers,
						mmr_proof: Proof {
							leaf_indices: headers_with_proof.leaf_indices,
							leaf_count: headers_with_proof.leaf_count,
							items: headers_with_proof
								.mmr_proofs
								.into_iter()
								.map(|item| H256::decode(&mut &*item))
								.collect::<Result<Vec<_>, _>>()
								.map_err(Error::from)?,
						},
					};

					// Perform the parachain header verification
					beefy_light_client::verify_parachain_headers::<H>(
						light_client_state,
						parachain_update_proof,
					)
					.map_err(Error::from)?
				}
			},
			ClientMessage::Misbehaviour(_) => unimplemented!(),
		}
		Ok(())
	}

	fn update_state<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_id: ClientId,
		client_state: Self::ClientState,
		message: Self::ClientMessage,
	) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Ics02Error> {
		let header = match message {
			ClientMessage::Header(header) => header,
			_ => unreachable!(
				"02-client will check for misbehaviour before calling update_state; qed"
			),
		};
		let mut parachain_cs_states = vec![];
		// Extract the new client state from the verified header
		let mut client_state = client_state.from_header(header.clone()).map_err(Error::from)?;
		let mut latest_para_height = client_state.latest_para_height;

		if let Some(parachain_headers) = header.headers_with_proof {
			for header in parachain_headers.headers {
				// Skip genesis block of parachains since it has no timestamp or ibc root
				if header.parachain_header.number == 0 {
					continue;
				}
				if latest_para_height < header.parachain_header.number {
					latest_para_height = header.parachain_header.number;
				}
				let height =
					Height::new(client_state.para_id as u64, header.parachain_header.number as u64);
				// Skip duplicate consensus states
				if ctx.consensus_state(&client_id, height).is_ok() {
					continue;
				}
				parachain_cs_states.push((
					height,
					Ctx::AnyConsensusState::wrap(
						&ConsensusState::from_header(header).map_err(Error::from)?,
					)
					.ok_or_else(|| Error::Custom("Ctx::AnyConsensusState".to_string()))?,
				))
			}
		}

		client_state.latest_para_height = latest_para_height;

		Ok((client_state, ConsensusUpdateResult::Batch(parachain_cs_states)))
	}

	fn update_state_on_misbehaviour(
		&self,
		mut client_state: Self::ClientState,
		_header: Self::ClientMessage,
	) -> Result<Self::ClientState, Ics02Error> {
		client_state.frozen_height =
			Some(Height::new(client_state.para_id as u64, client_state.latest_para_height as u64));
		Ok(client_state)
	}

	fn check_for_misbehaviour<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_id: ClientId,
		client_state: Self::ClientState,
		message: Self::ClientMessage,
	) -> Result<bool, Ics02Error> {
		match message {
			ClientMessage::Header(header) => {
				// we simply check that this update doesn't include competing consensus states for
				// heights we already processed.
				if let Some(parachain_headers) = header.headers_with_proof {
					for header in parachain_headers.headers {
						let height = Height::new(
							client_state.para_id as u64,
							header.parachain_header.number as u64,
						);

						let consensus_state =
							ConsensusState::from_header(header).map_err(Error::from)?;
						match ctx.maybe_consensus_state(&client_id, height)? {
							Some(cs) => {
								let cs: ConsensusState =
									cs.downcast().ok_or(Ics02Error::client_args_type_mismatch(
										client_state.client_type().to_owned(),
									))?;

								if cs != consensus_state {
									// Houston we have a problem
									return Ok(true);
								}
							},
							None => {},
						};
					}
				}
			},
			// todo: Beefy protocol hasn't yet defined it's equivocation protocol
			// blocked on https://github.com/paritytech/grandpa-bridge-gadget/issues/101
			ClientMessage::Misbehaviour(_) => {},
		}

		Ok(false)
	}

	fn verify_upgrade_and_update_state<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_id: ClientId,
		_old_client_state: &Self::ClientState,
		_upgrade_client_state: &Self::ClientState,
		_upgrade_consensus_state: &Self::ConsensusState,
		_proof_upgrade_client: Vec<u8>,
		_proof_upgrade_consensus_state: Vec<u8>,
	) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Ics02Error> {
		Err(Error::Custom("Beefy Client doesn't need client upgrades".to_string()).into())
	}

	fn check_substitute_and_update_state<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_subject_client_id: ClientId,
		_substitute_client_id: ClientId,
		_old_client_state: Self::ClientState,
		_substitute_client_state: Self::ClientState,
	) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Ics02Error> {
		Err(Error::Custom("Beefy Client doesn't need client upgrades".to_string()).into())
	}

	fn verify_client_consensus_state<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		client_state: &Self::ClientState,
		height: Height,
		prefix: &CommitmentPrefix,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		client_id: &ClientId,
		consensus_height: Height,
		expected_consensus_state: &Ctx::AnyConsensusState,
	) -> Result<(), Ics02Error> {
		client_state.verify_height(height)?;
		let path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: consensus_height.revision_number,
			height: consensus_height.revision_height,
		};
		let value = expected_consensus_state.encode_to_vec().map_err(Ics02Error::encode)?;
		verify_membership::<H::BlakeTwo256, _>(prefix, proof, root, path, value)
			.map_err(Error::Anyhow)?;
		Ok(())
	}

	// Consensus state will be verified in the verification functions  before these are called
	fn verify_connection_state<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_id: &ClientId,
		client_state: &Self::ClientState,
		height: Height,
		prefix: &CommitmentPrefix,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		connection_id: &ConnectionId,
		expected_connection_end: &ConnectionEnd,
	) -> Result<(), Ics02Error> {
		client_state.verify_height(height)?;
		let path = ConnectionsPath(connection_id.clone());
		let value = expected_connection_end.encode_vec().map_err(Ics02Error::encode)?;
		verify_membership::<H::BlakeTwo256, _>(prefix, proof, root, path, value)
			.map_err(Error::Anyhow)?;
		Ok(())
	}

	fn verify_channel_state<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_id: &ClientId,
		client_state: &Self::ClientState,
		height: Height,
		prefix: &CommitmentPrefix,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		port_id: &PortId,
		channel_id: &ChannelId,
		expected_channel_end: &ChannelEnd,
	) -> Result<(), Ics02Error> {
		client_state.verify_height(height)?;
		let path = ChannelEndsPath(port_id.clone(), *channel_id);
		let value = expected_channel_end.encode_vec().map_err(Ics02Error::encode)?;
		verify_membership::<H::BlakeTwo256, _>(prefix, proof, root, path, value)
			.map_err(Error::Anyhow)?;
		Ok(())
	}

	fn verify_client_full_state<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		client_state: &Self::ClientState,
		height: Height,
		prefix: &CommitmentPrefix,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		client_id: &ClientId,
		expected_client_state: &Ctx::AnyClientState,
	) -> Result<(), Ics02Error> {
		client_state.verify_height(height)?;
		let path = ClientStatePath(client_id.clone());
		let value = expected_client_state.encode_to_vec().map_err(Ics02Error::encode)?;
		verify_membership::<H::BlakeTwo256, _>(prefix, proof, root, path, value)
			.map_err(Error::Anyhow)?;
		Ok(())
	}

	fn verify_packet_data<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		_client_id: &ClientId,
		client_state: &Self::ClientState,
		height: Height,
		connection_end: &ConnectionEnd,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: Sequence,
		commitment: PacketCommitment,
	) -> Result<(), Ics02Error> {
		client_state.verify_height(height)?;
		verify_delay_passed::<H, _>(ctx, height, connection_end).map_err(Error::Anyhow)?;

		let commitment_path =
			CommitmentsPath { port_id: port_id.clone(), channel_id: *channel_id, sequence };

		verify_membership::<H::BlakeTwo256, _>(
			connection_end.counterparty().prefix(),
			proof,
			root,
			commitment_path,
			commitment.into_vec(),
		)
		.map_err(Error::Anyhow)?;
		Ok(())
	}

	fn verify_packet_acknowledgement<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		_client_id: &ClientId,
		client_state: &Self::ClientState,
		height: Height,
		connection_end: &ConnectionEnd,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: Sequence,
		ack: AcknowledgementCommitment,
	) -> Result<(), Ics02Error> {
		client_state.verify_height(height)?;
		verify_delay_passed::<H, _>(ctx, height, connection_end).map_err(Error::Anyhow)?;

		let ack_path = AcksPath { port_id: port_id.clone(), channel_id: *channel_id, sequence };
		verify_membership::<H::BlakeTwo256, _>(
			connection_end.counterparty().prefix(),
			proof,
			root,
			ack_path,
			ack.into_vec(),
		)
		.map_err(Error::Anyhow)?;
		Ok(())
	}

	fn verify_next_sequence_recv<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		_client_id: &ClientId,
		client_state: &Self::ClientState,
		height: Height,
		connection_end: &ConnectionEnd,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: Sequence,
	) -> Result<(), Ics02Error> {
		client_state.verify_height(height)?;
		verify_delay_passed::<H, _>(ctx, height, connection_end).map_err(Error::Anyhow)?;

		let seq_bytes = parity_scale_codec::Encode::encode(&u64::from(sequence));

		let seq_path = SeqRecvsPath(port_id.clone(), *channel_id);
		verify_membership::<H::BlakeTwo256, _>(
			connection_end.counterparty().prefix(),
			proof,
			root,
			seq_path,
			seq_bytes,
		)
		.map_err(Error::Anyhow)?;
		Ok(())
	}

	fn verify_packet_receipt_absence<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		_client_id: &ClientId,
		client_state: &Self::ClientState,
		height: Height,
		connection_end: &ConnectionEnd,
		proof: &CommitmentProofBytes,
		root: &CommitmentRoot,
		port_id: &PortId,
		channel_id: &ChannelId,
		sequence: Sequence,
	) -> Result<(), Ics02Error> {
		client_state.verify_height(height)?;
		verify_delay_passed::<H, _>(ctx, height, connection_end).map_err(Error::Anyhow)?;

		let receipt_path =
			ReceiptsPath { port_id: port_id.clone(), channel_id: *channel_id, sequence };
		verify_non_membership::<H::BlakeTwo256, _>(
			connection_end.counterparty().prefix(),
			proof,
			root,
			receipt_path,
		)
		.map_err(Error::Anyhow)?;
		Ok(())
	}
}
