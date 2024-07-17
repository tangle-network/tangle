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

use super::{
	client_state::NearClientState,
	consensus_state::ConsensusState,
	error::Error as NearError,
	header::NearHeader,
	types::{ApprovalInner, CryptoHash, LightClientBlockView},
};
use crate::header::NearClientMessage;
use borsh::BorshSerialize;
use core::fmt::Debug;
use ibc::{
	core::{
		ics02_client::{
			client_def::{ClientDef, ConsensusUpdateResult},
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
	prelude::*,
	Height,
};
use ics23::HostFunctionsProvider;
use std::marker::PhantomData;

pub trait HostFunctionsTrait:
	HostFunctions + HostFunctionsProvider + Clone + Debug + PartialEq + Eq + Default + Send + Sync
{
}

/// This trait captures all the functions that the host chain should provide for
/// crypto operations.
pub trait HostFunctions: Clone + Send + Sync + Default {
	/// Keccak 256 hash function
	fn keccak_256(input: &[u8]) -> [u8; 32];

	/// Compressed Ecdsa public key recovery from a signature
	fn secp256k1_ecdsa_recover_compressed(
		signature: &[u8; 65],
		value: &[u8; 32],
	) -> Option<Vec<u8>>;

	/// Recover the ED25519 pubkey that produced this signature, given a arbitrarily sized message
	fn ed25519_verify(signature: &[u8; 64], msg: &[u8], pubkey: &[u8]) -> bool;

	/// This function should verify membership in a trie proof using sp_state_machine's
	/// read_child_proof_check
	fn verify_membership_trie_proof(
		root: &[u8; 32],
		proof: &[Vec<u8>],
		key: &[u8],
		value: &[u8],
	) -> Result<(), Error>;

	/// This function should verify non membership in a trie proof using sp_state_machine's
	/// read_child_proof_check
	fn verify_non_membership_trie_proof(
		root: &[u8; 32],
		proof: &[Vec<u8>],
		key: &[u8],
	) -> Result<(), Error>;

	/// This function should verify membership in a trie proof using parity's sp-trie package
	/// with a BlakeTwo256 Hasher
	fn verify_timestamp_extrinsic(
		root: &[u8; 32],
		proof: &[Vec<u8>],
		value: &[u8],
	) -> Result<(), Error>;

	/// Conduct a 256-bit Sha2 hash
	fn sha256_digest(data: &[u8]) -> [u8; 32];

	/// The SHA-256 hash algorithm
	fn sha2_256(message: &[u8]) -> [u8; 32];

	/// The SHA-512 hash algorithm
	fn sha2_512(message: &[u8]) -> [u8; 64];

	/// The SHA-512 hash algorithm with its output truncated to 256 bits.
	fn sha2_512_truncated(message: &[u8]) -> [u8; 32];

	/// SHA-3-512 hash function.
	fn sha3_512(message: &[u8]) -> [u8; 64];

	/// Ripemd160 hash function.
	fn ripemd160(message: &[u8]) -> [u8; 20];
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NearClient<H>(PhantomData<H>);

impl<H: HostFunctionsTrait> ClientDef for NearClient<H> {
	/// The data that we need to update the [`ClientState`] to a new block height
	type ClientMessage = NearClientMessage;

	/// The data that we need to know, to validate incoming headers and update the state
	/// of our [`ClientState`]. Ususally this will store:
	///    - The current epoch
	///    - The current validator set
	///
	/// ```rust,ignore
	/// pub struct NearLightClientState {
	///     head: LightClientBlockView,
	///     current_validators: Vec<ValidatorStakeView>,
	///     next_validators:  Vec<ValidatorStakeView>,
	/// }
	/// ```
	type ClientState = NearClientState<H>;

	/// This is usually just two things, that should be derived from the header:
	///    - The ibc commitment root hash as described by ics23 (possibly from tx outcome/ state
	///      proof)
	///    - The timestamp of the header.
	type ConsensusState = ConsensusState;

	// rehydrate client from its own storage, then call this function
	fn verify_client_message<Ctx>(
		&self,
		_ctx: &Ctx,
		_client_id: ClientId,
		client_state: Self::ClientState,
		client_message: Self::ClientMessage,
	) -> Result<(), Error>
	where
		Ctx: ReaderContext,
	{
		match client_message {
			NearClientMessage::Header(header) => {
				// your light client, shouldn't do storage anymore, it should just do verification
				// here.
				validate_light_block::<H>(&header, client_state)
			},
		}
	}

	fn update_state<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_id: ClientId,
		_client_state: Self::ClientState,
		_client_message: Self::ClientMessage,
	) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Error> {
		// 1. create new client state from this header, return that.
		// 2. as well as all the neccessary consensus states.
		//
		//
		// []--[]--[]--[]--[]--[]--[]--[]--[]--[]
		// 11  12  13  14  15  16  17  18  19  20 <- block merkle root
		// ^                                    ^
		// |    <-------consensus states----->  |
		// current state                       new state

		todo!()
	}

	fn update_state_on_misbehaviour(
		&self,
		_client_state: Self::ClientState,
		_client_message: Self::ClientMessage,
	) -> Result<Self::ClientState, Error> {
		todo!()
	}

	fn check_for_misbehaviour<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_id: ClientId,
		_client_state: Self::ClientState,
		_client_message: Self::ClientMessage,
	) -> Result<bool, Error> {
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
	) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Error> {
		todo!()
	}

	fn check_substitute_and_update_state<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_subject_client_id: ClientId,
		_substitute_client_id: ClientId,
		_old_client_state: Self::ClientState,
		_substitute_client_state: Self::ClientState,
	) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Error> {
		todo!()
	}

	fn verify_client_consensus_state<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_state: &Self::ClientState,
		_height: Height,
		_prefix: &CommitmentPrefix,
		_proof: &CommitmentProofBytes,
		_root: &CommitmentRoot,
		_client_id: &ClientId,
		_consensus_height: Height,
		_expected_consensus_state: &Ctx::AnyConsensusState,
	) -> Result<(), Error> {
		todo!()
	}

	// Consensus state will be verified in the verification functions  before these are called
	fn verify_connection_state<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_id: &ClientId,
		_client_state: &Self::ClientState,
		_height: Height,
		_prefix: &CommitmentPrefix,
		_proof: &CommitmentProofBytes,
		_root: &CommitmentRoot,
		_connection_id: &ConnectionId,
		_expected_connection_end: &ConnectionEnd,
	) -> Result<(), Error> {
		todo!()
	}

	fn verify_channel_state<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_id: &ClientId,
		_client_state: &Self::ClientState,
		_height: Height,
		_prefix: &CommitmentPrefix,
		_proof: &CommitmentProofBytes,
		_root: &CommitmentRoot,
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_expected_channel_end: &ChannelEnd,
	) -> Result<(), Error> {
		todo!()
	}

	fn verify_client_full_state<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_state: &Self::ClientState,
		_height: Height,
		_prefix: &CommitmentPrefix,
		_proof: &CommitmentProofBytes,
		_root: &CommitmentRoot,
		_client_id: &ClientId,
		_expected_client_state: &Ctx::AnyClientState,
	) -> Result<(), Error> {
		todo!()
	}

	fn verify_packet_data<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_id: &ClientId,
		_client_state: &Self::ClientState,
		_height: Height,
		_connection_end: &ConnectionEnd,
		_proof: &CommitmentProofBytes,
		_root: &CommitmentRoot,
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_sequence: Sequence,
		_commitment: PacketCommitment,
	) -> Result<(), Error> {
		todo!()
	}

	fn verify_packet_acknowledgement<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_id: &ClientId,
		_client_state: &Self::ClientState,
		_height: Height,
		_connection_end: &ConnectionEnd,
		_proof: &CommitmentProofBytes,
		_root: &CommitmentRoot,
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_sequence: Sequence,
		_ack: AcknowledgementCommitment,
	) -> Result<(), Error> {
		todo!()
	}

	fn verify_next_sequence_recv<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_id: &ClientId,
		_client_state: &Self::ClientState,
		_height: Height,
		_connection_end: &ConnectionEnd,
		_proof: &CommitmentProofBytes,
		_root: &CommitmentRoot,
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_sequence: Sequence,
	) -> Result<(), Error> {
		todo!()
	}

	fn verify_packet_receipt_absence<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_id: &ClientId,
		_client_state: &Self::ClientState,
		_height: Height,
		_connection_end: &ConnectionEnd,
		_proof: &CommitmentProofBytes,
		_root: &CommitmentRoot,
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_sequence: Sequence,
	) -> Result<(), Error> {
		todo!()
	}
}

/// validates a light block that's contained on the `NearHeader` based on the current
/// state of the light client.
pub fn validate_light_block<H: HostFunctionsTrait>(
	header: &NearHeader,
	client_state: NearClientState<H>,
) -> Result<(), Error>
where
{
	//The light client updates its head with the information from LightClientBlockView iff:

	// 1. The height of the block is higher than the height of the current head;
	// 2. The epoch of the block is equal to the epoch_id or next_epoch_id known for the current
	// head; 3. If the epoch of the block is equal to the next_epoch_id of the head, then next_bps
	// is not None; 4. approvals_after_next contain valid signatures on approval_message from the
	// block producers of the corresponding epoch
	// 5. The signatures present in approvals_after_next correspond to more than 2/3 of the total
	// stake (see next section). 6. If next_bps is not none, sha256(borsh(next_bps)) corresponds to
	// the next_bp_hash in inner_lite.

	// QUESTION: do we also want to pass the block hash received from the RPC?
	// it's not on the spec, but it's an extra validation

	let new_block_view = header.get_light_client_block_view();
	let current_block_view = client_state.get_head();
	let (_current_block_hash, _next_block_hash, approval_message) =
		reconstruct_light_client_block_view_fields::<H>(new_block_view)?;

	// (1)
	if new_block_view.inner_lite.height <= current_block_view.inner_lite.height {
		return Err(NearError::height_too_old().into())
	}

	// (2)
	if ![current_block_view.inner_lite.epoch_id, current_block_view.inner_lite.next_epoch_id]
		.contains(&new_block_view.inner_lite.epoch_id)
	{
		return Err(NearError::invalid_epoch(new_block_view.inner_lite.epoch_id).into())
	}

	// (3)
	if new_block_view.inner_lite.epoch_id == current_block_view.inner_lite.next_epoch_id &&
		new_block_view.next_bps.is_none()
	{
		return Err(NearError::unavailable_block_producers().into())
	}

	//  (4) and (5)
	let mut total_stake = 0;
	let mut approved_stake = 0;

	let epoch_block_producers = client_state
		.get_validators_by_epoch(&new_block_view.inner_lite.epoch_id)
		.ok_or_else(|| Error::from(NearError::invalid_epoch(new_block_view.inner_lite.epoch_id)))?;

	for (maybe_signature, block_producer) in
		new_block_view.approvals_after_next.iter().zip(epoch_block_producers.iter())
	{
		let bp_stake_view = block_producer.clone().into_validator_stake();
		let bp_stake = bp_stake_view.stake;
		total_stake += bp_stake;

		if maybe_signature.is_none() {
			continue
		}

		approved_stake += bp_stake;

		let validator_public_key = &bp_stake_view.public_key;
		let data = H::sha256_digest(&approval_message);
		let signature = maybe_signature.as_ref().unwrap();
		if H::ed25519_verify(signature.get_inner(), &data, validator_public_key.get_inner()) {
			return Err(NearError::invalid_signature().into())
		}
	}

	let threshold = total_stake * 2 / 3;
	if approved_stake <= threshold {
		return Err(NearError::insufficient_staked_amount().into())
	}

	// # (6)
	if new_block_view.next_bps.is_some() {
		let new_block_view_next_bps_serialized = new_block_view
			.next_bps
			.as_deref()
			.unwrap()
			.try_to_vec()
			.map_err(|_| Error::from(NearError::serialization_error()))?;
		if H::sha256_digest(new_block_view_next_bps_serialized.as_ref()).as_slice() !=
			new_block_view.inner_lite.next_bp_hash.as_ref()
		{
			return Err(NearError::serialization_error().into())
		}
	}
	Ok(())
}

pub fn reconstruct_light_client_block_view_fields<H: HostFunctions>(
	block_view: &LightClientBlockView,
) -> Result<(CryptoHash, CryptoHash, Vec<u8>), Error> {
	let current_block_hash = block_view.current_block_hash::<H>();
	let next_block_hash =
		next_block_hash::<H>(block_view.next_block_inner_hash, current_block_hash);
	let approval_message = [
		ApprovalInner::Endorsement(next_block_hash)
			.try_to_vec()
			.map_err(|_| Error::from(NearError::serialization_error()))?,
		(block_view.inner_lite.height + 2)
			.to_le()
			.try_to_vec()
			.map_err(|_| Error::from(NearError::serialization_error()))?,
	]
	.concat();
	Ok((current_block_hash, next_block_hash, approval_message))
}

pub(crate) fn next_block_hash<H: HostFunctions>(
	next_block_inner_hash: CryptoHash,
	current_block_hash: CryptoHash,
) -> CryptoHash {
	H::sha256_digest(
		[next_block_inner_hash.as_ref(), current_block_hash.as_ref()].concat().as_ref(),
	)
	.as_slice()
	.try_into()
	.expect("Could not hash the next block")
}
