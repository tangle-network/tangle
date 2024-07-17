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

use core::{convert::TryInto, fmt::Debug, marker::PhantomData, str::FromStr};

use ibc::core::{
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
	ics23_commitment::{
		commitment::{CommitmentPrefix, CommitmentProofBytes, CommitmentRoot},
		merkle::{apply_prefix, MerkleProof},
	},
	ics24_host::{
		identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId},
		path::{
			AcksPath, ChannelEndsPath, ClientConsensusStatePath, ClientStatePath, CommitmentsPath,
			ConnectionsPath, ReceiptsPath, SeqRecvsPath,
		},
		Path,
	},
	ics26_routing::context::ReaderContext,
};
use ibc_proto::ibc::core::commitment::v1::MerkleProof as RawMerkleProof;
use prost::Message;
use tendermint_light_client_verifier::{
	types::{TrustedBlockState, UntrustedBlockState},
	Verdict, Verifier,
};
use tendermint_proto::Protobuf;

use crate::{
	client_message::{ClientMessage, Header},
	client_state::ClientState,
	consensus_state::ConsensusState,
	error::Error,
	HostFunctionsProvider, ProdVerifier,
};
use ibc::{prelude::*, Height};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TendermintClient<H>(PhantomData<H>);

impl<H> ClientDef for TendermintClient<H>
where
	H: HostFunctionsProvider,
{
	type ClientMessage = ClientMessage;
	type ClientState = ClientState<H>;
	type ConsensusState = ConsensusState;

	fn verify_client_message<Ctx>(
		&self,
		ctx: &Ctx,
		client_id: ClientId,
		client_state: Self::ClientState,
		message: Self::ClientMessage,
	) -> Result<(), Ics02Error>
	where
		Ctx: ReaderContext,
	{
		match message {
			ClientMessage::Header(header) => {
				if header.height().revision_number != client_state.chain_id.version() {
					return Err(Ics02Error::client_error(
						client_state.client_type().to_owned(),
						Error::mismatched_revisions(
							client_state.chain_id.version(),
							header.height().revision_number,
						)
						.to_string(),
					))
				}

				if header.height().revision_number != header.trusted_height.revision_number {
					return Err(Ics02Error::client_error(
						client_state.client_type().to_owned(),
						Error::mismatched_revisions(
							header.trusted_height.revision_number,
							header.height().revision_number,
						)
						.to_string(),
					))
				}

				// Check if a consensus state is already installed; if so skip
				let header_consensus_state = <ConsensusState as From<Header>>::from(header.clone());

				let _ = match ctx.maybe_consensus_state(&client_id.clone(), header.height())? {
					Some(cs) => {
						let cs: ConsensusState =
							cs.downcast().ok_or(Ics02Error::client_args_type_mismatch(
								client_state.client_type().to_owned(),
							))?;
						// If this consensus state matches, skip verification
						// (optimization)
						if cs == header_consensus_state {
							// Header is already installed and matches the incoming
							// header (already verified)
							return Ok(())
						}
						Some(cs)
					},
					None => None,
				};

				let trusted_consensus_state: Self::ConsensusState = ctx
					.consensus_state(&client_id.clone(), header.trusted_height)?
					.downcast()
					.ok_or(Ics02Error::client_args_type_mismatch(
						ClientState::<H>::client_type().to_owned(),
					))?;

				if trusted_consensus_state
					.next_validators_hash
					.ne(&header.trusted_validator_set.hash_with::<H>())
				{
					return Err(Ics02Error::header_verification_failure(
						"next val set mismatch".to_string(),
					))
				}

				let trusted_state = TrustedBlockState {
					// TODO: make sure it's correct
					chain_id: &tendermint::chain::Id::from_str(client_state.chain_id.as_str())
						.unwrap(),
					header_time: trusted_consensus_state.timestamp().into_tm_time().unwrap(),
					height: header.trusted_height.revision_height.try_into().map_err(|_| {
						Ics02Error::client_error(
							client_state.client_type().to_owned(),
							Error::invalid_header_height(header.trusted_height).to_string(),
						)
					})?,
					next_validators: &header.trusted_validator_set,
					next_validators_hash: trusted_consensus_state.next_validators_hash,
				};

				let untrusted_state = UntrustedBlockState {
					signed_header: &header.signed_header,
					validators: &header.validator_set,
					// NB: This will skip the
					// VerificationPredicates::next_validators_match check for the
					// untrusted state.
					next_validators: None,
				};

				let options = client_state.as_light_client_options()?;

				let verifier = ProdVerifier::<H>::default();
				let verdict = verifier.verify(
					untrusted_state,
					trusted_state,
					&options,
					ctx.host_timestamp().into_tm_time().unwrap(),
				);

				match verdict {
					Verdict::Success => {},
					Verdict::NotEnoughTrust(voting_power_tally) =>
						return Err(Error::not_enough_trusted_vals_signed(format!(
							"voting power tally: {}",
							voting_power_tally
						))
						.into()),
					Verdict::Invalid(detail) =>
						return Err(Error::verification_error(detail).into()),
				}
			},
			ClientMessage::Misbehaviour(misbehaviour) => {
				verify_misbehaviour_header(
					ctx,
					client_id.clone(),
					client_state.clone(),
					ClientMessage::Header(misbehaviour.header1),
				)?;
				verify_misbehaviour_header(
					ctx,
					client_id.clone(),
					client_state,
					ClientMessage::Header(misbehaviour.header2),
				)?;
			},
		};

		Ok(())
	}

	fn update_state<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_client_id: ClientId,
		client_state: Self::ClientState,
		client_message: Self::ClientMessage,
	) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Ics02Error> {
		let header = match client_message {
			ClientMessage::Header(header) => header,
			_ => unreachable!("02-client will check for Header before calling update_state; qed"),
		};
		let header_consensus_state = <ConsensusState as From<Header>>::from(header.clone());
		let cs = Ctx::AnyConsensusState::wrap(&header_consensus_state).ok_or_else(|| {
			Ics02Error::unknown_consensus_state_type("Ctx::AnyConsensusState".to_string())
		})?;
		Ok((client_state.with_header(header), ConsensusUpdateResult::Single(cs)))
	}

	fn update_state_on_misbehaviour(
		&self,
		client_state: Self::ClientState,
		client_message: Self::ClientMessage,
	) -> Result<Self::ClientState, Ics02Error> {
		let misbehaviour = match client_message {
			ClientMessage::Misbehaviour(misbehaviour) => misbehaviour,
			_ => unreachable!(
				"02-client will check for misbehaviour before calling update_state_on_misbehaviour; qed"
			),
		};
		client_state
			.with_frozen_height(misbehaviour.header1.height())
			.map_err(|e| e.into())
	}

	fn check_for_misbehaviour<Ctx: ReaderContext>(
		&self,
		ctx: &Ctx,
		client_id: ClientId,
		_client_state: Self::ClientState,
		message: Self::ClientMessage,
	) -> Result<bool, Ics02Error> {
		match message {
			ClientMessage::Header(header) => {
				// Check if a consensus state is already installed; if so it should
				// match the untrusted header.
				let header_consensus_state = <ConsensusState as From<Header>>::from(header.clone());

				let existing_consensus_state =
					match ctx.maybe_consensus_state(&client_id, header.height())? {
						Some(cs) => {
							let cs = cs.downcast::<ConsensusState>().ok_or(
								Ics02Error::client_args_type_mismatch(
									ClientState::<()>::client_type().to_owned(),
								),
							)?;
							// If this consensus state matches, skip verification
							// (optimization)
							if header_consensus_state.eq(&cs) {
								// Header is already installed and matches the incoming
								// header (already verified)
								return Ok(false)
							}
							Some(cs)
						},
						None => None,
					};

				// If the header has verified, but its corresponding consensus state
				// differs from the existing consensus state for that height, freeze the
				// client and return the installed consensus state.
				if let Some(cs) = existing_consensus_state {
					if cs.ne(&header_consensus_state) {
						return Ok(true)
					}
				}

				if let Ok(maybe_next_cs) = ctx.next_consensus_state(&client_id, header.height()) {
					if let Some(next_cs) = maybe_next_cs {
						if next_cs.timestamp().nanoseconds() < header.timestamp().nanoseconds() {
							return Ok(true)
						}
					}
				}

				match ctx.prev_consensus_state(&client_id, header.height())? {
					Some(prev_cs) => {
						if prev_cs.timestamp().nanoseconds() > header.timestamp().nanoseconds() {
							return Ok(true)
						}
					},
					None => {},
				};
			},
			ClientMessage::Misbehaviour(misbehaviour) => {
				if misbehaviour.header1.height().revision_height ==
					misbehaviour.header2.height().revision_height
				{
					let block_id1 = misbehaviour.header1.signed_header.commit.block_id;
					let block_id2 = misbehaviour.header2.signed_header.commit.block_id;
					if block_id1.hash.ne(&block_id2.hash) {
						return Ok(true)
					}
				} else if misbehaviour.header1.signed_header.header.time.le(&misbehaviour
					.header2
					.signed_header
					.header
					.time)
				{
					return Ok(true)
				}
			},
		};

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
		// TODO: tendermint verify_upgrade_and_update_state
		Err(Ics02Error::implementation_specific("Not implemented".to_string()))
	}

	fn check_substitute_and_update_state<Ctx: ReaderContext>(
		&self,
		_ctx: &Ctx,
		_subject_client_id: ClientId,
		_substitute_client_id: ClientId,
		_old_client_state: Self::ClientState,
		_substitute_client_state: Self::ClientState,
	) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Ics02Error> {
		// TODO: tendermint check_substitute_and_update_state
		Err(Ics02Error::implementation_specific("Not implemented".to_string()))
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
		verify_membership::<H, _>(client_state, prefix, proof, root, path, value)
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
		verify_membership::<H, _>(client_state, prefix, proof, root, path, value)
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
		verify_membership::<H, _>(client_state, prefix, proof, root, path, value)
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
		verify_membership::<H, _>(client_state, prefix, proof, root, path, value)
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
		verify_delay_passed(ctx, height, connection_end)?;

		let commitment_path =
			CommitmentsPath { port_id: port_id.clone(), channel_id: *channel_id, sequence };

		verify_membership::<H, _>(
			client_state,
			connection_end.counterparty().prefix(),
			proof,
			root,
			commitment_path,
			commitment.into_vec(),
		)
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
		ack_commitment: AcknowledgementCommitment,
	) -> Result<(), Ics02Error> {
		// client state height = consensus state height
		client_state.verify_height(height)?;
		verify_delay_passed(ctx, height, connection_end)?;

		let ack_path = AcksPath { port_id: port_id.clone(), channel_id: *channel_id, sequence };
		verify_membership::<H, _>(
			client_state,
			connection_end.counterparty().prefix(),
			proof,
			root,
			ack_path,
			ack_commitment.into_vec(),
		)
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
		verify_delay_passed(ctx, height, connection_end)?;

		let mut seq_bytes = Vec::new();
		u64::from(sequence).encode(&mut seq_bytes).expect("buffer size too small");

		let seq_path = SeqRecvsPath(port_id.clone(), *channel_id);
		verify_membership::<H, _>(
			client_state,
			connection_end.counterparty().prefix(),
			proof,
			root,
			seq_path,
			seq_bytes,
		)
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
		verify_delay_passed(ctx, height, connection_end)?;

		let receipt_path =
			ReceiptsPath { port_id: port_id.clone(), channel_id: *channel_id, sequence };
		verify_non_membership::<H, _>(
			client_state,
			connection_end.counterparty().prefix(),
			proof,
			root,
			receipt_path,
		)
	}
}

pub fn verify_membership<H, P>(
	client_state: &ClientState<H>,
	prefix: &CommitmentPrefix,
	proof: &CommitmentProofBytes,
	root: &CommitmentRoot,
	path: P,
	value: Vec<u8>,
) -> Result<(), Ics02Error>
where
	P: Into<Path>,
	H: ics23::HostFunctionsProvider,
{
	let merkle_path = apply_prefix(prefix, vec![path.into().to_string()]);
	let merkle_proof: MerkleProof<H> = RawMerkleProof::try_from(proof.clone())
		.map_err(Ics02Error::invalid_commitment_proof)?
		.into();

	merkle_proof
		.verify_membership(&client_state.proof_specs, root.clone().into(), merkle_path, value, 0)
		.map_err(|e| Error::ics23_error(e).into())
}

pub fn verify_non_membership<H, P>(
	client_state: &ClientState<H>,
	prefix: &CommitmentPrefix,
	proof: &CommitmentProofBytes,
	root: &CommitmentRoot,
	path: P,
) -> Result<(), Ics02Error>
where
	P: Into<Path>,
	H: ics23::HostFunctionsProvider,
{
	let merkle_path = apply_prefix(prefix, vec![path.into().to_string()]);
	let merkle_proof: MerkleProof<H> = RawMerkleProof::try_from(proof.clone())
		.map_err(Ics02Error::invalid_commitment_proof)?
		.into();

	merkle_proof
		.verify_non_membership(&client_state.proof_specs, root.clone().into(), merkle_path)
		.map_err(|e| Error::ics23_error(e).into())
}

fn verify_delay_passed<Ctx: ReaderContext>(
	ctx: &Ctx,
	height: Height,
	connection_end: &ConnectionEnd,
) -> Result<(), Ics02Error> {
	let current_timestamp = ctx.host_timestamp();
	let current_height = ctx.host_height();

	let client_id = connection_end.client_id();
	let processed_time = ctx
		.client_update_time(client_id, height)
		.map_err(|_| Error::processed_time_not_found(height))?;
	let processed_height = ctx
		.client_update_height(client_id, height)
		.map_err(|_| Error::processed_height_not_found(height))?;

	let delay_period_time = connection_end.delay_period();
	let delay_period_height = ctx.block_delay(delay_period_time);
	let delay_period_time_u64 = u64::try_from(delay_period_time.as_nanos()).unwrap();

	ClientState::<()>::verify_delay_passed(
		current_timestamp,
		current_height,
		processed_time.nanoseconds(),
		processed_height.revision_height,
		delay_period_time_u64,
		delay_period_height,
	)
	.map_err(|e| e.into())
}

fn verify_misbehaviour_header<Ctx: ReaderContext, H: HostFunctionsProvider>(
	ctx: &Ctx,
	client_id: ClientId,
	client_state: ClientState<H>,
	message: ClientMessage,
) -> Result<(), Ics02Error> {
	match message {
		ClientMessage::Header(header) => {
			// Check if a consensus state is already installed; if so skip
			let header_consensus_state = <ConsensusState as From<Header>>::from(header.clone());

			let _ = match ctx.maybe_consensus_state(&client_id.clone(), header.height())? {
				Some(cs) => {
					let cs: ConsensusState =
						cs.downcast().ok_or(Ics02Error::client_args_type_mismatch(
							client_state.client_type().to_owned(),
						))?;
					// If this consensus state matches, skip verification
					// (optimization)
					if cs == header_consensus_state {
						// Header is already installed and matches the incoming
						// header (already verified)
						return Ok(())
					}
					Some(cs)
				},
				None => None,
			};

			let trusted_consensus_state: ConsensusState = ctx
				.consensus_state(&client_id.clone(), header.trusted_height)?
				.downcast()
				.ok_or(Ics02Error::client_args_type_mismatch(
					ClientState::<H>::client_type().to_owned(),
				))?;

			if trusted_consensus_state
				.next_validators_hash
				.ne(&header.trusted_validator_set.hash_with::<H>())
			{
				return Err(Ics02Error::header_verification_failure(
					"next val set mismatch".to_string(),
				))
			}

			let chain_id = client_state.chain_id();
			if !ChainId::is_epoch_format(chain_id.as_str()) {
				return Err(Ics02Error::header_verification_failure(
					"client state chain id not in epoch format".to_string(),
				))
			}

			let split: Vec<_> = chain_id.as_str().split('-').collect();
			let chain_name = split.first().expect("get name from chain_id").parse().unwrap();
			let chain_id = ChainId::new(chain_name, header.height().revision_number);
			let tm_chain_id = tendermint::chain::Id::from_str(chain_id.as_str()).unwrap();
			if tm_chain_id != header.signed_header.header.chain_id {
				return Err(Ics02Error::header_verification_failure("chain id mismatch".to_string()))
			}

			let trusted_state = TrustedBlockState {
				chain_id: &tm_chain_id,
				header_time: trusted_consensus_state.timestamp().into_tm_time().unwrap(),
				height: header.trusted_height.revision_height.try_into().map_err(|_| {
					Ics02Error::client_error(
						client_state.client_type().to_owned(),
						Error::invalid_header_height(header.trusted_height).to_string(),
					)
				})?,
				next_validators: &header.trusted_validator_set,
				next_validators_hash: trusted_consensus_state.next_validators_hash,
			};

			let untrusted_state = UntrustedBlockState {
				signed_header: &header.signed_header,
				validators: &header.validator_set,
				// NB: This will skip the
				// VerificationPredicates::next_validators_match check for the
				// untrusted state.
				next_validators: None,
			};

			let options = client_state.as_light_client_options()?;

			let verifier = ProdVerifier::<H>::default();

			let verdict = verifier.verify_validator_sets(&untrusted_state);

			match verdict {
				Verdict::Success => {},
				Verdict::NotEnoughTrust(voting_power_tally) =>
					return Err(Error::not_enough_trusted_vals_signed(format!(
						"voting power tally: {}",
						voting_power_tally
					))
					.into()),
				Verdict::Invalid(detail) => return Err(Error::verification_error(detail).into()),
			}

			let verdict =
				verifier.verify_commit_against_trusted(&untrusted_state, &trusted_state, &options);

			match verdict {
				Verdict::Success => {},
				Verdict::NotEnoughTrust(voting_power_tally) =>
					return Err(Error::not_enough_trusted_vals_signed(format!(
						"voting power tally: {}",
						voting_power_tally
					))
					.into()),
				Verdict::Invalid(detail) => return Err(Error::verification_error(detail).into()),
			}

			let verdict = verifier.verify_commit(&untrusted_state);

			match verdict {
				Verdict::Success => {},
				Verdict::NotEnoughTrust(voting_power_tally) =>
					return Err(Error::not_enough_trusted_vals_signed(format!(
						"voting power tally: {}",
						voting_power_tally
					))
					.into()),
				Verdict::Invalid(detail) => return Err(Error::verification_error(detail).into()),
			}

			if let Ok(expires_at) = trusted_state.header_time + options.trusting_period {
				if expires_at < ctx.host_timestamp().into_tm_time().unwrap() {
					return Err(Ics02Error::header_verification_failure("Expired".to_string()))
				}
			} else {
				return Err(Ics02Error::header_verification_failure(
					"Can't get expire time".to_string(),
				))
			}
		},
		ClientMessage::Misbehaviour(_misbehaviour) => {},
	};

	Ok(())
}
