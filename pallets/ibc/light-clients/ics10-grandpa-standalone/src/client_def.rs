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
	client_message::StandaloneChainHeader, client_state::ClientState,
	consensus_state::ConsensusState, error::Error,
};
use ibc::core::ics02_client::{
	client_consensus::ConsensusState as _, client_state::ClientState as _,
};

use crate::client_message::ClientMessage;
use alloc::{format, string::ToString, vec, vec::Vec};
use core::marker::PhantomData;
use finality_grandpa::Chain;
use grandpa_client_primitives::standalone::{
	find_forced_change, find_scheduled_change, AncestryChain, GrandpaStandaloneJustification,
};
use ibc::{
	core::{
		ics02_client::{
			client_def::{ClientDef, ConsensusUpdateResult},
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
	Height,
};
use light_client_common::{
	state_machine, verify_delay_passed, verify_membership, verify_non_membership,
};
use parity_scale_codec::Decode;
use sp_core::H256;
use sp_runtime::traits::Header;
use sp_trie::StorageProof;
use tendermint_proto::Protobuf;

const CLIENT_STATE_UPGRADE_PATH: &[u8] = b"client-state-upgrade-path";
const CONSENSUS_STATE_UPGRADE_PATH: &[u8] = b"consensus-state-upgrade-path";

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct GrandpaClient<T>(PhantomData<T>);

impl<H> ClientDef for GrandpaClient<H>
where
	H: grandpa_client_primitives::StandaloneHostFunctions<Header = StandaloneChainHeader>,
{
	type ClientMessage = ClientMessage;
	type ClientState = ClientState<H>;
	type ConsensusState = ConsensusState;

	fn verify_client_message<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_id: ClientId,
		client_state: Self::ClientState,
		client_message: Self::ClientMessage,
	) -> Result<(), Ics02Error> {
		match client_message {
			ClientMessage::Header(header) => {
				if client_state.chain_id as u64 != header.height.revision_number {
					return Err(Error::Custom(format!(
						"Para id mismatch: expected {}, got {}",
						client_state.chain_id, header.height.revision_number
					))
					.into());
				}

				grandpa_client::verify_standalone_grandpa_finality_proof::<
					StandaloneChainHeader,
					H,
				>(client_state.into(), header.finality_proof)
				.map_err(Error::GrandpaPrimitives)?;
			},
			ClientMessage::Misbehaviour(misbehavior) => {
				let first_proof = misbehavior.first_finality_proof;
				let second_proof = misbehavior.second_finality_proof;

				if first_proof.block == second_proof.block {
					return Err(
						Error::Custom("Misbehaviour proofs are for the same block".into()).into()
					);
				}

				let first_headers =
					AncestryChain::<StandaloneChainHeader>::new(&first_proof.unknown_headers);
				let first_target =
					first_proof.unknown_headers.iter().max_by_key(|h| *h.number()).ok_or_else(
						|| Error::Custom("Unknown headers can't be empty!".to_string()),
					)?;

				let second_headers =
					AncestryChain::<StandaloneChainHeader>::new(&second_proof.unknown_headers);
				let second_target =
					second_proof.unknown_headers.iter().max_by_key(|h| *h.number()).ok_or_else(
						|| Error::Custom("Unknown headers can't be empty!".to_string()),
					)?;

				if first_target.hash() != first_proof.block
					|| second_target.hash() != second_proof.block
				{
					return Err(Error::Custom(
						"Misbehaviour proofs are not for the same chain".into(),
					)
					.into());
				}

				let first_base =
					first_proof.unknown_headers.iter().min_by_key(|h| *h.number()).ok_or_else(
						|| Error::Custom("Unknown headers can't be empty!".to_string()),
					)?;
				first_headers
					.ancestry(first_base.hash(), first_target.hash())
					.map_err(|_| Error::Custom("Invalid ancestry!".to_string()))?;

				let second_base =
					second_proof.unknown_headers.iter().min_by_key(|h| *h.number()).ok_or_else(
						|| Error::Custom("Unknown headers can't be empty!".to_string()),
					)?;
				second_headers
					.ancestry(second_base.hash(), second_target.hash())
					.map_err(|_| Error::Custom("Invalid ancestry!".to_string()))?;

				let first_parent = first_base.parent_hash;
				let second_parent = second_base.parent_hash;

				if first_parent != second_parent {
					return Err(Error::Custom(
						"Misbehaviour proofs are not for the same ancestor".into(),
					)
					.into());
				}

				// TODO: should we handle genesis block here somehow?
				if !H::contains_header_hash(first_parent) {
					Err(Error::Custom(
						"Could not find the known header for first finality proof".to_string(),
					))?
				}

				let first_justification =
					GrandpaStandaloneJustification::<StandaloneChainHeader>::decode(
						&mut &first_proof.justification[..],
					)
					.map_err(|_| {
						Error::Custom("Could not decode first justification".to_string())
					})?;
				let second_justification =
					GrandpaStandaloneJustification::<StandaloneChainHeader>::decode(
						&mut &second_proof.justification[..],
					)
					.map_err(|_| {
						Error::Custom("Could not decode second justification".to_string())
					})?;

				if first_proof.block != first_justification.commit.target_hash
					|| second_proof.block != second_justification.commit.target_hash
				{
					Err(Error::Custom(
						"First or second finality proof block hash does not match justification target hash"
							.to_string(),
					))?
				}

				let first_valid = first_justification
					.verify::<H>(client_state.current_set_id, &client_state.current_authorities)
					.is_ok();
				let second_valid = second_justification
					.verify::<H>(client_state.current_set_id, &client_state.current_authorities)
					.is_ok();

				if !first_valid || !second_valid {
					Err(Error::Custom("Invalid justification".to_string()))?
				}

				// whoops equivocation is valid.
			},
		}

		Ok(())
	}

	fn update_state<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_id: ClientId,
		mut client_state: Self::ClientState,
		client_message: Self::ClientMessage,
	) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Ics02Error> {
		let header = match client_message {
			ClientMessage::Header(header) => header,
			_ => unreachable!(
				"02-client will check for misbehaviour before calling update_state; qed"
			),
		};
		let timestamp_proof = header.timestamp_proof;
		let ancestry =
			AncestryChain::<StandaloneChainHeader>::new(&header.finality_proof.unknown_headers);

		let from = client_state.latest_hash;

		let finalized = ancestry
			.ancestry(from, header.finality_proof.block)
			.map_err(|_| Error::Custom(format!("[update_state] Invalid ancestry!")))?;
		let mut finalized_sorted = finalized.clone();
		finalized_sorted.sort();

		// we really shouldn't set consensus states for parachain headers not in the finalized
		// chain.
		let block_hash = header.finality_proof.block;
		if finalized_sorted.binary_search(&block_hash).is_err() {
			return Err(Error::Custom(format!(
				"Block hash {block_hash:?} is not in the finalized chain"
			)))?;
		}

		let header = ancestry.header(&block_hash).ok_or_else(|| {
			Error::Custom(format!("No standalone chain header found for hash: {block_hash:?}"))
		})?;

		let (height, consensus_state) = ConsensusState::from_header::<H>(
			timestamp_proof,
			client_state.chain_id,
			header.clone(),
		)?;

		// Error if duplicate consensus states
		if ctx.consensus_state(&client_id, height).is_ok() {
			return Err(Error::Custom(format!(
				"Consensus state for height {height} already exists"
			)))?;
		}

		let wrapped = Ctx::AnyConsensusState::wrap(&consensus_state)
			.expect("AnyConsenusState is type checked; qed");

		// updates
		let target = ancestry
			.header(&block_hash)
			.expect("target header has already been checked in verify_client_message; qed");

		// can't try to rewind relay chain
		if target.number <= client_state.latest_height {
			Err(Ics02Error::implementation_specific(format!(
				"Light client can only be updated to new standalone chain height."
			)))?
		}

		client_state.latest_hash = block_hash;
		client_state.latest_height = target.number;

		if let Some(scheduled_change) = find_scheduled_change(target) {
			client_state.current_set_id += 1;
			client_state.current_authorities = scheduled_change.next_authorities;
		}

		H::insert_header_hashes(&finalized);

		Ok((client_state, ConsensusUpdateResult::Single(wrapped)))
	}

	fn update_state_on_misbehaviour(
		&self,
		mut client_state: Self::ClientState,
		_client_message: Self::ClientMessage,
	) -> Result<Self::ClientState, Ics02Error> {
		client_state.frozen_height =
			Some(Height::new(client_state.chain_id as u64, client_state.latest_height as u64));
		Ok(client_state)
	}

	fn check_for_misbehaviour<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_id: ClientId,
		client_state: Self::ClientState,
		client_message: Self::ClientMessage,
	) -> Result<bool, Ics02Error> {
		if matches!(client_message, ClientMessage::Misbehaviour(_)) {
			return Ok(true);
		}

		// we also check that this update doesn't include competing consensus states for heights we
		// already processed.
		let header = match client_message {
			ClientMessage::Header(header) => header,
			_ => unreachable!("We've checked for misbehavior in line 180; qed"),
		};
		let timestamp_proof = header.timestamp_proof;
		//forced authority set change is handled as a misbehaviour

		let ancestry =
			AncestryChain::<StandaloneChainHeader>::new(&header.finality_proof.unknown_headers);

		let block_hash = header.finality_proof.block;
		let header = ancestry.header(&block_hash).ok_or_else(|| {
			Error::Custom(format!("No standalone chain header found for hash: {block_hash:?}"))
		})?;

		if find_forced_change(header).is_some() {
			return Ok(true);
		}

		let (height, consensus_state) = ConsensusState::from_header::<H>(
			timestamp_proof,
			client_state.chain_id,
			header.clone(),
		)?;

		match ctx.maybe_consensus_state(&client_id, height)? {
			Some(cs) => {
				let cs: ConsensusState = cs
					.downcast()
					.ok_or(Ics02Error::client_args_type_mismatch(client_state.client_type()))?;

				if cs != consensus_state {
					// Houston we have a problem
					return Ok(true);
				}
			},
			None => {},
		};

		Ok(false)
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
	) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Ics02Error> {
		let height =
			Height::new(old_client_state.chain_id as u64, old_client_state.latest_height as u64);

		let consenus_state = ctx.consensus_state(&client_id, height)?
			.downcast::<Self::ConsensusState>()
			.ok_or_else(|| Error::Custom(format!("Wrong consensus state type stored for Grandpa client with {client_id} at {height}")))?;

		let root = H256::from_slice(consenus_state.root.as_bytes());

		// verify client state upgrade proof
		{
			let proof_upgrade_client = {
				let nodes: Vec<Vec<u8>> =
					Decode::decode(&mut &proof_upgrade_client[..]).map_err(Error::Codec)?;
				StorageProof::new(nodes)
			};

			let encoded = Ctx::AnyClientState::wrap(&upgrade_client_state.clone())
				.expect("AnyConsensusState is type-checked; qed")
				.encode_to_vec()
				.map_err(Ics02Error::encode)?;

			let value = state_machine::read_proof_check::<H::BlakeTwo256, _>(
				&root,
				proof_upgrade_client,
				vec![CLIENT_STATE_UPGRADE_PATH],
			)
			.map_err(|err| Error::Custom(format!("{err}")))?
			.remove(CLIENT_STATE_UPGRADE_PATH)
			.flatten()
			.ok_or_else(|| Error::Custom(format!("Invalid proof for client state upgrade")))?;

			if value != encoded {
				Err(Error::Custom(format!("Invalid proof for client state upgrade")))?
			}
		}

		// verify consensus state upgrade proof
		{
			let proof_upgrade_consensus_state = {
				let nodes: Vec<Vec<u8>> = Decode::decode(&mut &proof_upgrade_consensus_state[..])
					.map_err(Error::Codec)?;
				StorageProof::new(nodes)
			};

			let encoded = Ctx::AnyConsensusState::wrap(upgrade_client_state)
				.expect("AnyConsensusState is type-checked; qed")
				.encode_to_vec()
				.map_err(Ics02Error::encode)?;

			let value = state_machine::read_proof_check::<H::BlakeTwo256, _>(
				&root,
				proof_upgrade_consensus_state,
				vec![CONSENSUS_STATE_UPGRADE_PATH],
			)
			.map_err(|err| Error::Custom(format!("{err}")))?
			.remove(CONSENSUS_STATE_UPGRADE_PATH)
			.flatten()
			.ok_or_else(|| Error::Custom(format!("Invalid proof for client state upgrade")))?;

			if value != encoded {
				Err(Error::Custom(format!("Invalid proof for client state upgrade")))?
			}
		}

		Ok((
			upgrade_client_state.clone(),
			ConsensusUpdateResult::Single(
				Ctx::AnyConsensusState::wrap(upgrade_consensus_state)
					.expect("AnyConsensusState is type-checked; qed"),
			),
		))
	}

	/// Will try to update the client with the state of the substitute.
	///
	/// The following must always be true:
	///   - The substitute client is the same type as the subject client
	///   - The subject and substitute client states match in all parameters (expect `relay_chain`,
	/// `chain_id`, `latest_height`, `latest_height`, `latest_hash`,
	/// `frozen_height`, `latest_height`, `current_set_id` and `current_authorities`).
	fn check_substitute_and_update_state<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_subject_client_id: ClientId,
		_substitute_client_id: ClientId,
		_old_client_state: Self::ClientState,
		_substitute_client_state: Self::ClientState,
	) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Ics02Error> {
		unimplemented!("check_substitute_and_update_state not implemented for Grandpa client")
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
