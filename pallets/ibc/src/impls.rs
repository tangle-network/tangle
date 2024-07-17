use alloc::{format, str::FromStr};
use core::time::Duration;

use crate::{
	ics23::{
		acknowledgements::Acknowledgements, channels::Channels, client_states::ClientStates,
		connections::Connections, consensus_states::ConsensusStates,
		next_seq_recv::NextSequenceRecv, packet_commitments::PacketCommitment,
		receipts::PacketReceipt,
	},
	light_clients::AnyClientState,
	routing::Context,
	Acks, ChannelsConnection, Config, ConnectionClient, DenomToAssetId, Error, EscrowAddresses,
	IbcAssets, Pallet, PendingRecvPacketSeqs, PendingSendPacketSeqs, RecvPackets, SendPackets,
	MODULE_ID,
};
use frame_support::traits::{fungibles::Inspect, Currency};
use ibc::{
	applications::transfer::{
		msgs::transfer::MsgTransfer, relay::send_transfer::send_transfer, PrefixedCoin,
	},
	core::{
		ics02_client::{
			client_consensus::ConsensusState, client_state::ClientState, context::ClientReader,
		},
		ics03_connection::context::ConnectionReader,
		ics04_channel::{
			channel::ChannelEnd,
			context::{ChannelKeeper, ChannelReader},
			error::Error as Ics04Error,
			msgs::{chan_close_init::MsgChannelCloseInit, chan_open_init::MsgChannelOpenInit},
			packet::{Packet, Sequence},
		},
		ics24_host::{
			identifier::*,
			path::{
				AcksPath, ChannelEndsPath, ClientConsensusStatePath, ClientStatePath,
				CommitmentsPath, ConnectionsPath, ReceiptsPath, SeqRecvsPath,
			},
		},
		ics26_routing::handler::MsgReceipt,
	},
	handler::HandlerOutputBuilder,
	signer::Signer,
	timestamp::Timestamp,
	tx_msg::Msg,
	Height,
};
use ibc_primitives::{
	apply_prefix, channel_id_from_bytes, client_id_from_bytes, connection_id_from_bytes,
	get_channel_escrow_address, port_id_from_bytes, runtime_interface, ConnectionHandshake,
	Error as IbcHandlerError, HandlerMessage, IbcHandler, IdentifiedChannel, IdentifiedClientState,
	IdentifiedConnection, PacketInfo, PacketState, QueryChannelResponse, QueryChannelsResponse,
	QueryClientStateResponse, QueryConnectionResponse, QueryConnectionsResponse,
	QueryConsensusStateResponse, QueryNextSequenceReceiveResponse,
	QueryPacketAcknowledgementResponse, QueryPacketAcknowledgementsResponse,
	QueryPacketCommitmentResponse, QueryPacketCommitmentsResponse, QueryPacketReceiptResponse,
	Timeout,
};
use parity_scale_codec::{Decode, Encode};
use sp_core::crypto::AccountId32;
use sp_runtime::{
	traits::{Get, IdentifyAccount},
	Either, SaturatedConversion,
};
use sp_std::prelude::*;
use tendermint_proto::Protobuf;

const PACKET_CLEANUP_PER_CYCLE: u64 = 1001;

impl<T: Config> Pallet<T>
where
	T: Send + Sync,
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	pub(crate) fn execute_ibc_messages(
		ctx: &mut Context<T>,
		messages: Vec<ibc_proto::google::protobuf::Any>,
	) {
		let (events, logs) =
			messages.into_iter().fold((vec![], vec![]), |(mut events, mut logs), msg| {
				match ibc::core::ics26_routing::handler::deliver(ctx, msg) {
					Ok(MsgReceipt { events: temp_events, log: temp_logs }) => {
						events.extend(temp_events.into_iter().map(Ok));
						logs.extend(temp_logs);
					},
					Err(e) => {
						log::trace!(target: "pallet_ibc", "execution error: {}", e);
						events.push(Err(e));
					},
				}
				(events, logs)
			});

		log::trace!(target: "pallet_ibc", "logs: {:#?}", logs);
		if !events.is_empty() {
			Self::deposit_event(events.into())
		};
	}
}

impl<T: Config> Pallet<T>
where
	T: Send + Sync,
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	// IBC Runtime Api helper methods
	/// Get a channel state
	pub fn channel(
		channel_id: Vec<u8>,
		port_id: Vec<u8>,
	) -> Result<QueryChannelResponse, Error<T>> {
		let port_id = port_id_from_bytes(port_id).map_err(|_| Error::<T>::DecodingError)?;
		let channel_id =
			channel_id_from_bytes(channel_id).map_err(|_| Error::<T>::DecodingError)?;
		let channel =
			Channels::<T>::get(port_id.clone(), channel_id).ok_or(Error::<T>::ChannelNotFound)?;
		let channel_path = format!("{}", ChannelEndsPath(port_id, channel_id));
		let key = apply_prefix(T::PalletPrefix::get(), vec![channel_path]);

		Ok(QueryChannelResponse { channel, trie_key: key, height: host_height::<T>() })
	}

	/// Get a connection state
	pub fn connection(connection_id: Vec<u8>) -> Result<QueryConnectionResponse, Error<T>> {
		let connection_id =
			connection_id_from_bytes(connection_id).map_err(|_| Error::<T>::DecodingError)?;
		let connection =
			Connections::<T>::get(&connection_id).ok_or(Error::<T>::ConnectionNotFound)?;

		let connection_path = format!("{}", ConnectionsPath(connection_id));
		let key = apply_prefix(T::PalletPrefix::get(), vec![connection_path]);

		Ok(QueryConnectionResponse { connection, trie_key: key, height: host_height::<T>() })
	}

	/// Get a client state
	pub fn client(client_id: Vec<u8>) -> Result<QueryClientStateResponse, Error<T>> {
		let client_id = client_id_from_bytes(client_id).map_err(|_| Error::<T>::DecodingError)?;
		let client_state =
			ClientStates::<T>::get(&client_id).ok_or(Error::<T>::ClientStateNotFound)?;
		let client_state_path = format!("{}", ClientStatePath(client_id));

		let key = apply_prefix(T::PalletPrefix::get(), vec![client_state_path]);

		Ok(QueryClientStateResponse { client_state, trie_key: key, height: host_height::<T>() })
	}

	/// Get all client states
	/// Returns a Vec of (client_id, client_state)
	pub fn clients() -> Vec<(Vec<u8>, Vec<u8>)> {
		ClientStates::<T>::iter()
			.map(|(client_id, client_state)| (client_id.as_bytes().to_vec(), client_state))
			.collect::<Vec<_>>()
	}

	/// Get a consensus state for client
	pub fn consensus_state(
		client_id: Vec<u8>,
		revision_number: u64,
		revision_height: u64,
		latest_cs: bool,
	) -> Result<QueryConsensusStateResponse, Error<T>> {
		let client_id = client_id_from_bytes(client_id).map_err(|_| Error::<T>::DecodingError)?;
		let height = if latest_cs {
			let client_state =
				ClientStates::<T>::get(&client_id).ok_or(Error::<T>::ClientStateNotFound)?;
			let client_state =
				AnyClientState::decode_vec(&client_state).map_err(|_| Error::<T>::DecodingError)?;
			client_state.latest_height()
		} else {
			Height::new(revision_number, revision_height)
		};
		let consensus_state = ConsensusStates::<T>::get(client_id.clone(), height)
			.ok_or(Error::<T>::ConsensusStateNotFound)?;

		let consensus_path = ClientConsensusStatePath {
			client_id,
			epoch: height.revision_number,
			height: height.revision_height,
		};

		let path = format!("{consensus_path}");
		let key = apply_prefix(T::PalletPrefix::get(), vec![path]);

		Ok(QueryConsensusStateResponse {
			consensus_state,
			trie_key: key,
			height: host_height::<T>(),
		})
	}

	/// Get all connection states for a client
	pub fn connection_using_client(
		client_id: Vec<u8>,
	) -> Result<Vec<IdentifiedConnection>, Error<T>> {
		let connection_ids = ConnectionClient::<T>::get(client_id);
		let connections = connection_ids
			.into_iter()
			.filter_map(|connection_id| {
				let conn_id = connection_id_from_bytes(connection_id.clone()).ok()?;

				Some(IdentifiedConnection {
					connection_end: Connections::<T>::get(&conn_id)?,
					connection_id,
				})
			})
			.collect::<Vec<_>>();

		Ok(connections)
	}

	fn channel_client_id(channel_end: &ChannelEnd) -> Result<ClientId, Error<T>> {
		let ctx = Context::<T>::default();
		let connection_id =
			channel_end.connection_hops.get(0).ok_or(Error::<T>::ConnectionNotFound)?;
		let connection_end =
			ctx.connection_end(connection_id).map_err(|_| Error::<T>::ConnectionNotFound)?;
		Ok(connection_end.client_id().clone())
	}

	/// Get client state for client which this channel is bound to
	pub fn channel_client(
		channel_id: Vec<u8>,
		port_id: Vec<u8>,
	) -> Result<IdentifiedClientState, Error<T>> {
		let channel_id =
			channel_id_from_bytes(channel_id).map_err(|_| Error::<T>::InvalidChannelId)?;
		let port_id = port_id_from_bytes(port_id).map_err(|_| Error::<T>::InvalidPortId)?;
		let ctx = Context::<T>::new();
		let channel_end = ctx
			.channel_end(&(port_id, channel_id))
			.map_err(|_| Error::<T>::ChannelNotFound)?;
		let client_id = Self::channel_client_id(&channel_end)?;
		let client_state =
			ClientStates::<T>::get(&client_id).ok_or(Error::<T>::ClientStateNotFound)?;
		Ok(IdentifiedClientState { client_id: client_id.as_bytes().to_vec(), client_state })
	}

	/// Get all channel states
	pub fn channels() -> Result<QueryChannelsResponse, Error<T>> {
		let channels = Channels::<T>::iter()
			.map(|(port_id, channel_id, channel_end)| {
				Ok(IdentifiedChannel { channel_id, port_id, channel_end })
			})
			.collect::<Result<Vec<_>, Error<T>>>()?;

		Ok(QueryChannelsResponse { channels, height: host_height::<T>() })
	}

	/// Get all connection states
	pub fn connections() -> Result<QueryConnectionsResponse, Error<T>> {
		let connections = Connections::<T>::iter()
			.map(|(connection_id, connection_end)| {
				Ok(IdentifiedConnection { connection_id, connection_end })
			})
			.collect::<Result<Vec<_>, Error<T>>>()?;

		Ok(QueryConnectionsResponse { connections, height: host_height::<T>() })
	}

	/// Get all channels bound to this connection
	pub fn connection_channels(connection_id: Vec<u8>) -> Result<QueryChannelsResponse, Error<T>> {
		let identifiers = ChannelsConnection::<T>::get(connection_id);

		let channels = identifiers
			.into_iter()
			.map(|(port_id_bytes, channel_id_bytes)| {
				let channel_id = channel_id_from_bytes(channel_id_bytes.clone())
					.map_err(|_| Error::<T>::DecodingError)?;
				let port_id = port_id_from_bytes(port_id_bytes.clone())
					.map_err(|_| Error::<T>::DecodingError)?;

				let channel_end =
					Channels::<T>::get(port_id, channel_id).ok_or(Error::<T>::ChannelNotFound)?;
				Ok(IdentifiedChannel {
					channel_id: channel_id_bytes,
					port_id: port_id_bytes,
					channel_end,
				})
			})
			.collect::<Result<Vec<_>, Error<T>>>()?;
		Ok(QueryChannelsResponse { channels, height: host_height::<T>() })
	}

	pub fn packet_commitments(
		channel_id_bytes: Vec<u8>,
		port_id_bytes: Vec<u8>,
	) -> Result<QueryPacketCommitmentsResponse, Error<T>> {
		let channel_id = channel_id_from_bytes(channel_id_bytes.clone())
			.map_err(|_| Error::<T>::DecodingError)?;
		let port_id =
			port_id_from_bytes(port_id_bytes.clone()).map_err(|_| Error::<T>::DecodingError)?;
		let commitments = PacketCommitment::<T>::iter()
			.filter_map(|((p, c, s), commitment)| {
				if p == port_id && c == channel_id {
					let packet_state = PacketState {
						port_id: port_id_bytes.clone(),
						channel_id: channel_id_bytes.clone(),
						sequence: s.into(),
						data: commitment,
					};
					Some(packet_state)
				} else {
					None
				}
			})
			.collect::<Vec<_>>();

		Ok(QueryPacketCommitmentsResponse { commitments, height: host_height::<T>() })
	}

	pub fn packet_acknowledgements(
		channel_id_bytes: Vec<u8>,
		port_id_bytes: Vec<u8>,
	) -> Result<QueryPacketAcknowledgementsResponse, Error<T>> {
		let channel_id = channel_id_from_bytes(channel_id_bytes.clone())
			.map_err(|_| Error::<T>::DecodingError)?;
		let port_id =
			port_id_from_bytes(port_id_bytes.clone()).map_err(|_| Error::<T>::DecodingError)?;
		let acks = Acknowledgements::<T>::iter()
			.filter_map(|((p, c, s), ack)| {
				if p == port_id && c == channel_id {
					let packet_state = PacketState {
						port_id: port_id_bytes.clone(),
						channel_id: channel_id_bytes.clone(),
						sequence: s.into(),
						data: ack,
					};
					Some(packet_state)
				} else {
					None
				}
			})
			.collect::<Vec<_>>();
		Ok(QueryPacketAcknowledgementsResponse { acks, height: host_height::<T>() })
	}

	pub fn unreceived_packets(
		channel_id: Vec<u8>,
		port_id: Vec<u8>,
		seqs: Vec<u64>,
	) -> Result<Vec<u64>, Error<T>> {
		let channel_id =
			channel_id_from_bytes(channel_id).map_err(|_| Error::<T>::DecodingError)?;
		let port_id = port_id_from_bytes(port_id).map_err(|_| Error::<T>::DecodingError)?;
		Ok(seqs
			.into_iter()
			.filter(|s| {
				!PacketReceipt::<T>::contains_key((port_id.clone(), channel_id, (*s).into()))
			})
			.collect())
	}

	pub fn unreceived_acknowledgements(
		channel_id_bytes: Vec<u8>,
		port_id_bytes: Vec<u8>,
		seqs: Vec<u64>,
	) -> Result<Vec<u64>, Error<T>> {
		let channel_id =
			channel_id_from_bytes(channel_id_bytes).map_err(|_| Error::<T>::DecodingError)?;
		let port_id = port_id_from_bytes(port_id_bytes).map_err(|_| Error::<T>::DecodingError)?;
		Ok(seqs
			.into_iter()
			.filter(|s| {
				PacketCommitment::<T>::contains_key((port_id.clone(), channel_id, (*s).into()))
			})
			.collect())
	}

	pub fn next_seq_recv(
		channel_id: Vec<u8>,
		port_id: Vec<u8>,
	) -> Result<QueryNextSequenceReceiveResponse, Error<T>> {
		let port_id = port_id_from_bytes(port_id).map_err(|_| Error::<T>::DecodingError)?;
		let channel_id =
			channel_id_from_bytes(channel_id).map_err(|_| Error::<T>::DecodingError)?;
		let sequence = NextSequenceRecv::<T>::get(port_id.clone(), channel_id)
			.ok_or(Error::<T>::SendPacketError)?;
		let next_seq_recv_path = format!("{}", SeqRecvsPath(port_id, channel_id));
		let key = apply_prefix(T::PalletPrefix::get(), vec![next_seq_recv_path]);

		Ok(QueryNextSequenceReceiveResponse { sequence, trie_key: key, height: host_height::<T>() })
	}

	pub fn packet_commitment(
		channel_id: Vec<u8>,
		port_id: Vec<u8>,
		seq: u64,
	) -> Result<QueryPacketCommitmentResponse, Error<T>> {
		let port_id = port_id_from_bytes(port_id).map_err(|_| Error::<T>::DecodingError)?;
		let channel_id =
			channel_id_from_bytes(channel_id).map_err(|_| Error::<T>::DecodingError)?;
		let commitment = PacketCommitment::<T>::get((port_id.clone(), channel_id, seq.into()))
			.ok_or(Error::<T>::PacketCommitmentNotFound)?;
		let sequence = ibc::core::ics04_channel::packet::Sequence::from(seq);
		let commitment_path = format!("{}", CommitmentsPath { port_id, channel_id, sequence });
		let key = apply_prefix(T::PalletPrefix::get(), vec![commitment_path]);

		Ok(QueryPacketCommitmentResponse { commitment, trie_key: key, height: host_height::<T>() })
	}

	pub fn packet_acknowledgement(
		channel_id: Vec<u8>,
		port_id: Vec<u8>,
		seq: u64,
	) -> Result<QueryPacketAcknowledgementResponse, Error<T>> {
		let port_id = port_id_from_bytes(port_id).map_err(|_| Error::<T>::DecodingError)?;
		let channel_id =
			channel_id_from_bytes(channel_id).map_err(|_| Error::<T>::DecodingError)?;
		let sequence = ibc::core::ics04_channel::packet::Sequence::from(seq);
		let ack = Acknowledgements::<T>::get((port_id.clone(), channel_id, sequence))
			.ok_or(Error::<T>::PacketCommitmentNotFound)?;
		let acks_path = format!("{}", AcksPath { port_id, channel_id, sequence });
		let key = apply_prefix(T::PalletPrefix::get(), vec![acks_path]);

		Ok(QueryPacketAcknowledgementResponse { ack, trie_key: key, height: host_height::<T>() })
	}

	pub fn packet_receipt(
		channel_id: Vec<u8>,
		port_id: Vec<u8>,
		seq: u64,
	) -> Result<QueryPacketReceiptResponse, Error<T>> {
		let port_id = port_id_from_bytes(port_id).map_err(|_| Error::<T>::DecodingError)?;
		let channel_id =
			channel_id_from_bytes(channel_id).map_err(|_| Error::<T>::DecodingError)?;
		let sequence = ibc::core::ics04_channel::packet::Sequence::from(seq);
		let receipt = PacketReceipt::<T>::get((port_id.clone(), channel_id, sequence))
			.ok_or(Error::<T>::PacketReceiptNotFound)?;
		let receipt = String::from_utf8(receipt).map_err(|_| Error::<T>::DecodingError)?;
		let receipt_path = format!("{}", ReceiptsPath { port_id, channel_id, sequence });
		let key = apply_prefix(T::PalletPrefix::get(), vec![receipt_path]);
		let receipt = &receipt == "Ok";
		Ok(QueryPacketReceiptResponse { receipt, trie_key: key, height: host_height::<T>() })
	}

	pub fn connection_handshake(
		client_id: Vec<u8>,
		connection_id: Vec<u8>,
	) -> Result<ConnectionHandshake, Error<T>> {
		let client_id = client_id_from_bytes(client_id).map_err(|_| Error::<T>::DecodingError)?;
		let client_state =
			ClientStates::<T>::get(&client_id).ok_or(Error::<T>::ClientStateNotFound)?;
		let client_state_decoded =
			AnyClientState::decode_vec(&client_state).map_err(|_| Error::<T>::DecodingError)?;
		let height = client_state_decoded.latest_height();
		let connection_id =
			connection_id_from_bytes(connection_id).map_err(|_| Error::<T>::DecodingError)?;
		let prefix = T::PalletPrefix::get();
		let connection_path = format!("{}", ConnectionsPath(connection_id));
		let consensus_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: height.revision_number,
			height: height.revision_height,
		};
		let client_state_path = format!("{}", ClientStatePath(client_id));
		let consensus_path = format!("{consensus_path}");
		let client_state_key = apply_prefix(prefix, vec![client_state_path]);
		let connection_key = apply_prefix(prefix, vec![connection_path]);
		let consensus_key = apply_prefix(prefix, vec![consensus_path]);
		Ok(ConnectionHandshake {
			client_state,
			trie_keys: vec![client_state_key, connection_key, consensus_key],
			height: host_height::<T>(),
		})
	}

	pub fn query_balance_with_address(
		addr: Vec<u8>,
		asset_id: T::AssetId,
	) -> Result<u128, Error<T>> {
		let hex_string = String::from_utf8(addr).map_err(|_| Error::<T>::DecodingError)?;
		let signer = Signer::from_str(&hex_string).map_err(|_| Error::<T>::DecodingError)?;
		let ibc_acc =
			T::AccountIdConversion::try_from(signer).map_err(|_| Error::<T>::DecodingError)?;
		let account_id = ibc_acc.into_account();

		if asset_id == T::NativeAssetId::get() {
			let balance = format!("{:?}", T::NativeCurrency::free_balance(&account_id));
			Ok(balance.parse().unwrap_or_default())
		} else {
			let balance = format!("{:?}", T::Fungibles::balance(asset_id, &account_id));
			Ok(balance.parse().unwrap_or_default())
		}
	}

	pub fn send_packet_key(channel_id: Vec<u8>, port_id: Vec<u8>, seq: u64) -> Vec<u8> {
		let pair = (T::PalletPrefix::get().to_vec(), b"SEND_PACKET", channel_id, port_id, seq);
		pair.encode()
	}

	pub fn recv_packet_key(channel_id: Vec<u8>, port_id: Vec<u8>, seq: u64) -> Vec<u8> {
		let pair = (T::PalletPrefix::get().to_vec(), b"RECV_PACKET", channel_id, port_id, seq);
		pair.encode()
	}

	pub fn ack_key(channel_id: Vec<u8>, port_id: Vec<u8>, seq: u64) -> Vec<u8> {
		let pair = (T::PalletPrefix::get().to_vec(), b"ACK", channel_id, port_id, seq);
		pair.encode()
	}

	pub fn store_raw_acknowledgement(
		key: (PortId, ChannelId, Sequence),
		ack: Vec<u8>,
	) -> Result<(), Error<T>> {
		let channel_id = key.1.to_string().as_bytes().to_vec();
		let port_id = key.0.as_bytes().to_vec();
		let seq = u64::from(key.2);

		let key = Pallet::<T>::ack_key(channel_id, port_id, seq);
		Acks::<T>::insert(key.clone(), ack.clone());
		let ack_str = String::from_utf8_lossy(&ack);
		log::trace!(target: "pallet_ibc", "in channel: [store_raw_acknowledgement] >> writing acknowledgement: {}, key: 0x{}", ack_str, hex::encode(&key));
		Ok(())
	}

	pub(crate) fn packet_cleanup() -> Result<usize, (Error<T>, usize)> {
		let ctx = Context::<T>::default();

		let mut removed_count = 0;
		for (port_id_bytes, channel_id_bytes, _) in Channels::<T>::iter() {
			let channel_id = channel_id_from_bytes(channel_id_bytes.clone())
				.map_err(|_| (Error::<T>::DecodingError, removed_count))?;
			let port_id = port_id_from_bytes(port_id_bytes.clone())
				.map_err(|_| (Error::<T>::DecodingError, removed_count))?;

			let (mut send_seq_set, mut last_removed_send) =
				PendingSendPacketSeqs::<T>::get(&(port_id_bytes.clone(), channel_id_bytes.clone()));

			let last_removed_send_copy = last_removed_send;

			// We first try to remove sequences that were skipped in a previous cycle
			for seq in send_seq_set.clone() {
				if !PacketCommitment::<T>::contains_key((port_id.clone(), channel_id, seq.into())) {
					let key = Pallet::<T>::send_packet_key(
						channel_id_bytes.clone(),
						port_id_bytes.clone(),
						seq,
					);
					SendPackets::<T>::remove(key.clone());
					send_seq_set.remove(&seq);
					last_removed_send = seq;
					removed_count += 1;
				}
			}
			// Try removing at most 1000 sequences in this cycle starting from the last sequence
			// removed
			let next_seq_send =
				ctx.get_next_sequence_send(&(port_id.clone(), channel_id)).map_err(|_| {
					log::trace!(target: "pallet_ibc", "Failed to run packet clean up");
					(Error::<T>::Other, removed_count)
				})?;
			let range = (last_removed_send + 1)
				..(last_removed_send + PACKET_CLEANUP_PER_CYCLE).min(next_seq_send.into());
			for seq in range {
				if !PacketCommitment::<T>::contains_key((port_id.clone(), channel_id, seq.into())) {
					let key = Pallet::<T>::send_packet_key(
						channel_id_bytes.clone(),
						port_id_bytes.clone(),
						seq,
					);
					if SendPackets::<T>::contains_key(key.clone()) {
						SendPackets::<T>::remove(key.clone());
						last_removed_send = seq;
						removed_count += 1;
					}
				} else {
					// Add sequence to pending removal list
					send_seq_set.insert(seq);
				}
			}

			PendingSendPacketSeqs::<T>::insert(
				(port_id_bytes.clone(), channel_id_bytes.clone()),
				(send_seq_set, last_removed_send.max(last_removed_send_copy)),
			);

			let (mut recv_seq_set, mut last_removed_ack) =
				PendingRecvPacketSeqs::<T>::get(&(port_id_bytes.clone(), channel_id_bytes.clone()));
			let last_removed_ack_copy = last_removed_ack;

			// We first try to remove sequences that were skipped in a previous cycle
			for seq in recv_seq_set.clone() {
				if !Acknowledgements::<T>::contains_key((port_id.clone(), channel_id, seq.into())) {
					let key = Pallet::<T>::recv_packet_key(
						channel_id_bytes.clone(),
						port_id_bytes.clone(),
						seq,
					);
					let ack_key =
						Pallet::<T>::ack_key(channel_id_bytes.clone(), port_id_bytes.clone(), seq);
					RecvPackets::<T>::remove(key.clone());
					Acks::<T>::remove(ack_key.clone());
					recv_seq_set.remove(&seq);
					last_removed_ack = seq;
					removed_count += 1;
				}
			}
			// Try removing at most 1000 sequences in this cycle from the last sequence removed
			let next_seq_recv =
				ctx.get_next_sequence_recv(&(port_id.clone(), channel_id)).map_err(|_| {
					log::trace!(target: "pallet_ibc", "Failed to run packet clean up");
					(Error::<T>::Other, removed_count)
				})?;
			let range = (last_removed_ack + 1)
				..(last_removed_ack + PACKET_CLEANUP_PER_CYCLE).min(next_seq_recv.into());
			for seq in range {
				if !Acknowledgements::<T>::contains_key((port_id.clone(), channel_id, seq.into())) {
					let key = Pallet::<T>::recv_packet_key(
						channel_id_bytes.clone(),
						port_id_bytes.clone(),
						seq,
					);

					let ack_key =
						Pallet::<T>::ack_key(channel_id_bytes.clone(), port_id_bytes.clone(), seq);
					if RecvPackets::<T>::contains_key(key.clone()) {
						RecvPackets::<T>::remove(key.clone());
						Acks::<T>::remove(ack_key.clone());
						last_removed_ack = seq;
						removed_count += 1;
					}
				} else {
					// Add sequence to pending removal list
					recv_seq_set.insert(seq);
				}
			}
			PendingRecvPacketSeqs::<T>::insert(
				(port_id_bytes.clone(), channel_id_bytes.clone()),
				(recv_seq_set, last_removed_ack.max(last_removed_ack_copy)),
			);
		}
		Ok(removed_count)
	}

	pub fn get_send_packet_info(
		channel_id: Vec<u8>,
		port_id: Vec<u8>,
		sequences: Vec<u64>,
	) -> Result<Vec<PacketInfo>, Error<T>> {
		let packets = sequences
			.clone()
			.into_iter()
			.filter_map(|seq| {
				let key = Pallet::<T>::send_packet_key(channel_id.clone(), port_id.clone(), seq);
				SendPackets::<T>::get(key).and_then(|v| PacketInfo::decode(&mut &*v).ok())
			})
			.collect();
		log::trace!(target: "pallet_ibc", "get_send_packet_info: {:?}, {:?}", sequences, packets);
		Ok(packets)
	}

	pub fn get_recv_packet_info(
		channel_id: Vec<u8>,
		port_id: Vec<u8>,
		sequences: Vec<u64>,
	) -> Result<Vec<PacketInfo>, Error<T>> {
		let packets = sequences
			.clone()
			.into_iter()
			.filter_map(|seq| {
				let key = Pallet::<T>::recv_packet_key(channel_id.clone(), port_id.clone(), seq);
				let ack_key = Pallet::<T>::ack_key(channel_id.clone(), port_id.clone(), seq);
				let packet_info =
					RecvPackets::<T>::get(key).and_then(|v| PacketInfo::decode(&mut &*v).ok());
				let ack = Acks::<T>::get(ack_key);
				packet_info.map(|mut packet_info| {
					packet_info.ack = ack;
					packet_info
				})
			})
			.collect();
		log::trace!(target: "pallet_ibc", "get_recv_packet_info: {:?}, {:?}", sequences, packets);
		Ok(packets)
	}

	pub fn client_update_time_and_height(
		client_id: Vec<u8>,
		revision_number: u64,
		revision_height: u64,
	) -> Result<(u64, u64), Error<T>> {
		let ctx = Context::<T>::default();
		let client_id = client_id_from_bytes(client_id).map_err(|_| Error::<T>::DecodingError)?;
		let height = Height::new(revision_number, revision_height);
		let update_height = ctx
			.client_update_height(&client_id, height)
			.map_err(|_| Error::<T>::ClientUpdateNotFound)?;
		let update_time = ctx
			.client_update_time(&client_id, height)
			.map_err(|_| Error::<T>::ClientUpdateNotFound)?
			.nanoseconds();
		Ok((update_height.revision_height, update_time))
	}
}

impl<T: Config> Pallet<T> {
	pub fn remove_channel_escrow_address(
		port_id: &PortId,
		channel_id: ChannelId,
	) -> Result<(), Ics04Error> {
		let escrow_address = get_channel_escrow_address(port_id, channel_id).map_err(|_| {
			Ics04Error::implementation_specific(
				"Failed to derive channel escrow address for removal".to_string(),
			)
		})?;
		let account_id = T::AccountIdConversion::try_from(escrow_address)
			.map_err(|_| {
				Ics04Error::implementation_specific(
					"Failed to derive channel escrow address for removal".to_string(),
				)
			})?
			.into_account();
		let _ = EscrowAddresses::<T>::try_mutate::<_, &'static str, _>(|addresses| {
			addresses.remove(&account_id);
			Ok(())
		});
		Ok(())
	}

	/// Returns true if address provided is an escrow address
	pub fn is_escrow_address(address: <T as frame_system::Config>::AccountId) -> bool {
		let set = EscrowAddresses::<T>::get();
		set.contains(&address)
	}
}

impl<T: Config> Pallet<T> {
	pub fn get_denom_trace(
		asset_id: T::AssetId,
	) -> Option<ibc_primitives::QueryDenomTraceResponse> {
		T::IbcDenomToAssetIdConversion::from_asset_id_to_denom(asset_id).map(|denom| {
			ibc_primitives::QueryDenomTraceResponse { denom: denom.as_bytes().to_vec() }
		})
	}

	pub fn get_denom_traces(
		key: Option<Either<T::AssetId, u32>>,
		limit: u64,
		count_total: bool,
	) -> ibc_primitives::QueryDenomTracesResponse {
		let IbcAssets { denoms, total_count, next_id } =
			T::IbcDenomToAssetIdConversion::ibc_assets(key, limit);
		ibc_primitives::QueryDenomTracesResponse {
			denoms,
			total: count_total.then_some(total_count),
			next_key: next_id.map(|key| key.encode()),
		}
	}
}

impl<T: Config + Send + Sync> IbcHandler<<T as frame_system::Config>::AccountId> for Pallet<T>
where
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	fn latest_height_and_timestamp(
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<(Height, Timestamp), IbcHandlerError> {
		Pallet::<T>::latest_height_and_timestamp(port_id, channel_id)
	}

	fn handle_message(
		msg: HandlerMessage<<T as frame_system::Config>::AccountId>,
	) -> Result<(), IbcHandlerError> {
		match msg {
			HandlerMessage::OpenChannel { port_id, channel_end } => {
				Pallet::<T>::open_channel(port_id, channel_end)
			},
			HandlerMessage::CloseChannel { channel_id, port_id } => {
				Pallet::<T>::close_channel(port_id, channel_id)
			},
			HandlerMessage::Transfer { timeout, to, from, channel_id, coin, memo } => {
				let msg = Pallet::<T>::to_msg_transfer(coin, from, to, timeout, channel_id, memo)?;
				Pallet::<T>::send_transfer(msg)
			},
			HandlerMessage::SendPacket { data, timeout, port_id, channel_id } => {
				Pallet::<T>::send_packet(data, timeout, port_id, channel_id)
			},
			HandlerMessage::WriteAck { ack, packet } => {
				Pallet::<T>::write_acknowledgement(packet, ack)
			},
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn create_client() -> Result<ClientId, IbcHandlerError> {
		use crate::{
			benchmarks::tendermint_benchmark_utils::create_mock_state,
			light_clients::AnyConsensusState,
		};
		use ibc::core::ics02_client::msgs::create_client::{MsgCreateAnyClient, TYPE_URL};

		let (mock_client_state, mock_cs_state) = create_mock_state();
		let client_id = ClientId::new(&mock_client_state.client_type(), 0).unwrap();
		let msg = MsgCreateAnyClient::<Context<T>>::new(
			AnyClientState::Tendermint(mock_client_state),
			AnyConsensusState::Tendermint(mock_cs_state),
			Signer::from_str("pallet_ibc").unwrap(),
		)
		.unwrap()
		.encode_vec()
		.unwrap();
		let msg = ibc_proto::google::protobuf::Any { type_url: TYPE_URL.to_string(), value: msg };
		let mut ctx = Context::<T>::new();
		ibc::core::ics26_routing::handler::deliver(&mut ctx, msg).unwrap();
		Ok(client_id)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn create_connection(
		client_id: ClientId,
		connection_id: ConnectionId,
	) -> Result<(), IbcHandlerError> {
		use ibc::core::ics03_connection::{
			connection::{ConnectionEnd, Counterparty, State},
			context::ConnectionKeeper,
			version::Version,
		};
		let delay_period = core::time::Duration::from_nanos(1000);
		let counter_party = Counterparty::new(
			client_id.clone(),
			Some(ConnectionId::new(1)),
			<T as Config>::PalletPrefix::get().to_vec().try_into().unwrap(),
		);
		let connection_end = ConnectionEnd::new(
			State::Open,
			client_id.clone(),
			counter_party,
			vec![Version::default()],
			delay_period,
		);
		let mut ctx = crate::routing::Context::<T>::new();
		ctx.store_connection(connection_id.clone(), &connection_end).unwrap();
		ctx.store_connection_to_client(connection_id, &client_id).unwrap();
		Ok(())
	}
}

impl<T: Config + Send + Sync> Pallet<T>
where
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	fn send_packet(
		data: Vec<u8>,
		timeout: Timeout,
		source_port: PortId,
		source_channel: ChannelId,
	) -> Result<(), IbcHandlerError> {
		let mut ctx = Context::<T>::new();
		let source_channel_end =
			ctx.channel_end(&(source_port.clone(), source_channel)).map_err(|_| {
				IbcHandlerError::ChannelOrPortError {
					msg: Some(format!(
						"Failed to fetch Channel end for channel {source_channel} from storage"
					)),
				}
			})?;

		let sequence =
			ctx.get_next_sequence_send(&(source_port.clone(), source_channel))
				.map_err(|_| IbcHandlerError::SendPacketError {
					msg: Some(format!("Failed to get next_sequence_send for {source_channel}")),
				})?;

		let destination_port = source_channel_end.counterparty().port_id().clone();
		let destination_channel = *source_channel_end.counterparty().channel_id().ok_or(
			IbcHandlerError::ChannelOrPortError {
				msg: Some(
					"Failed to find counterparty channel_id in source channel end".to_string(),
				),
			},
		)?;
		let (latest_height, latest_timestamp) =
			Pallet::<T>::latest_height_and_timestamp(&source_port, &source_channel).map_err(
				|_| IbcHandlerError::TimestampOrHeightNotFound {
					msg: Some("Failed to retreive client height and timestamp".to_string()),
				},
			)?;

		let (timeout_height, timeout_timestamp) = match timeout {
			Timeout::Absolute { timestamp, height } => {
				let timeout_timestamp =
					Timestamp::from_nanoseconds(timestamp.ok_or_else(|| {
						IbcHandlerError::TimeoutError {
							msg: Some("Timeout timestamp is missing".to_string()),
						}
					})?)
					.map_err(|_| IbcHandlerError::TimeoutError {
						msg: Some("Failed to covert timestamp from nanosecods".to_string()),
					})?;
				let timeout_height = Height::new(
					latest_height.revision_number,
					height.ok_or_else(|| IbcHandlerError::TimeoutError {
						msg: Some("Timeout height is missing".to_string()),
					})?,
				);
				(timeout_height, timeout_timestamp)
			},
			Timeout::Offset { timestamp, height } => {
				let timeout_timestamp = (latest_timestamp
					+ Duration::from_nanos(timestamp.ok_or_else(|| {
						IbcHandlerError::TimeoutError {
							msg: Some("Timeout timestamp is missing".to_string()),
						}
					})?))
				.map_err(|_| IbcHandlerError::TimeoutError {
					msg: Some("Failed to covert timestamp from nanosecods".to_string()),
				})?;
				let timeout_height =
					latest_height.add(height.ok_or_else(|| IbcHandlerError::TimeoutError {
						msg: Some("Timeout height is missing".to_string()),
					})?);
				(timeout_height, timeout_timestamp)
			},
		};
		let packet = Packet {
			sequence,
			source_port,
			source_channel,
			destination_port,
			destination_channel,
			data,
			timeout_height,
			timeout_timestamp,
		};

		let send_packet_result =
			ibc::core::ics04_channel::handler::send_packet::send_packet(&ctx, packet)
				.map_err(|e| IbcHandlerError::SendPacketError { msg: Some(e.to_string()) })?;
		ctx.store_packet_result(send_packet_result.result)
			.map_err(|e| IbcHandlerError::SendPacketError { msg: Some(e.to_string()) })?;
		Self::deposit_event(send_packet_result.events.into());
		Ok(())
	}

	fn open_channel(port_id: PortId, channel_end: ChannelEnd) -> Result<(), IbcHandlerError> {
		let mut ctx = crate::routing::Context::<T>::new();
		// Signer does not matter in this case
		let msg = MsgChannelOpenInit {
			port_id,
			channel: channel_end,
			signer: Signer::from_str(MODULE_ID)
				.map_err(|_| IbcHandlerError::ChannelInitError { msg: None })?,
		};

		let msg = ibc_proto::google::protobuf::Any {
			type_url: msg.type_url(),
			value: msg
				.encode_vec()
				.map_err(|e| IbcHandlerError::Other { msg: Some(e.to_string()) })?,
		};
		let res = ibc::core::ics26_routing::handler::deliver::<_>(&mut ctx, msg)
			.map_err(|e| IbcHandlerError::ChannelInitError { msg: Some(e.to_string()) })?;
		Self::deposit_event(res.events.into());
		Ok(())
	}

	fn write_acknowledgement(packet: Packet, ack: Vec<u8>) -> Result<(), IbcHandlerError> {
		let mut ctx = Context::<T>::default();
		let error = |action, err| {
			let msg = Some(format!("Failed to {action} acknowledgement{err:?}"));
			IbcHandlerError::AcknowledgementError { msg }
		};
		let result =
			ibc::core::ics04_channel::handler::write_acknowledgement::process(&ctx, packet, ack)
				.map_err(|e| error("validate", e))?;
		ctx.store_packet_result(result.result).map_err(|e| error("store", e))?;
		Self::deposit_event(result.events.into());
		Ok(())
	}

	fn to_msg_transfer(
		coin: PrefixedCoin,
		from: <T as frame_system::Config>::AccountId,
		to: Signer,
		timeout: Timeout,
		channel_id: ChannelId,
		memo: String,
	) -> Result<MsgTransfer<PrefixedCoin>, IbcHandlerError> {
		let account_id_32: AccountId32 = from.into();
		let from = runtime_interface::account_id_to_ss58(
			account_id_32.into(),
			<T as frame_system::Config>::SS58Prefix::get(),
		);
		let (latest_height, latest_timestamp) =
			Pallet::<T>::latest_height_and_timestamp(&PortId::transfer(), &channel_id).map_err(
				|_| IbcHandlerError::TimestampOrHeightNotFound {
					msg: Some("Failed to retreive client height and timestamp".to_string()),
				},
			)?;

		let (timeout_height, timeout_timestamp) = match timeout {
			Timeout::Absolute { timestamp, height } => {
				let timeout_timestamp =
					Timestamp::from_nanoseconds(timestamp.ok_or_else(|| {
						IbcHandlerError::TimeoutError {
							msg: Some("Timeout timestamp is missing".to_string()),
						}
					})?)
					.map_err(|_| IbcHandlerError::TimeoutError {
						msg: Some("Failed to covert timestamp from nanosecods".to_string()),
					})?;
				let timeout_height = Height::new(
					latest_height.revision_number,
					height.ok_or_else(|| IbcHandlerError::TimeoutError {
						msg: Some("Timeout height is missing".to_string()),
					})?,
				);
				(timeout_height, timeout_timestamp)
			},
			Timeout::Offset { timestamp, height } => {
				let timeout_timestamp = (latest_timestamp
					+ Duration::from_nanos(timestamp.ok_or_else(|| {
						IbcHandlerError::TimeoutError {
							msg: Some("Timeout timestamp is missing".to_string()),
						}
					})?))
				.map_err(|_| IbcHandlerError::TimeoutError {
					msg: Some("Failed to covert timestamp from nanosecods".to_string()),
				})?;
				let timeout_height =
					latest_height.add(height.ok_or_else(|| IbcHandlerError::TimeoutError {
						msg: Some("Timeout height is missing".to_string()),
					})?);
				(timeout_height, timeout_timestamp)
			},
		};
		let msg = MsgTransfer {
			source_port: PortId::transfer(),
			source_channel: channel_id,
			token: coin,
			sender: Signer::from_str(&from).map_err(|_| IbcHandlerError::SendTransferError {
				msg: Some("Failed to deriver signer from String".to_string()),
			})?,
			receiver: to,
			timeout_height,
			timeout_timestamp,
			memo,
		};
		Ok(msg)
	}

	pub(crate) fn send_transfer(msg: MsgTransfer<PrefixedCoin>) -> Result<(), IbcHandlerError> {
		let mut ctx = Context::<T>::default();
		let mut handler_output = HandlerOutputBuilder::default();
		send_transfer::<_, _>(&mut ctx, &mut handler_output, msg)
			.map_err(|e| IbcHandlerError::SendTransferError { msg: Some(e.to_string()) })?;
		let result = handler_output.with_result(());
		Self::deposit_event(result.events.into());
		Ok(())
	}

	fn close_channel(port_id: PortId, channel_id: ChannelId) -> Result<(), IbcHandlerError> {
		let mut ctx = crate::routing::Context::<T>::new();
		// Signer does not matter in this case
		let msg = MsgChannelCloseInit {
			port_id,
			channel_id,
			signer: Signer::from_str(MODULE_ID)
				.map_err(|_| IbcHandlerError::ChannelInitError { msg: None })?,
		};
		let msg = ibc_proto::google::protobuf::Any {
			type_url: msg.type_url(),
			value: msg
				.encode_vec()
				.map_err(|e| IbcHandlerError::Other { msg: Some(e.to_string()) })?,
		};
		let res = ibc::core::ics26_routing::handler::deliver::<_>(&mut ctx, msg)
			.map_err(|e| IbcHandlerError::ChannelCloseError { msg: Some(e.to_string()) })?;
		Self::deposit_event(res.events.into());
		Ok(())
	}

	fn latest_height_and_timestamp(
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<(Height, Timestamp), IbcHandlerError> {
		let ctx = Context::<T>::new();
		let source_channel_end =
			ctx.channel_end(&(port_id.clone(), *channel_id)).map_err(|_| {
				IbcHandlerError::ChannelOrPortError {
					msg: Some(format!(
						"Failed to fetch Channel end for channel {channel_id} from storage"
					)),
				}
			})?;
		let client_id = Self::channel_client_id(&source_channel_end).map_err(|_| {
			IbcHandlerError::ClientIdError {
				msg: Some(format!("Could not find client id for {port_id:?}/{channel_id:?}")),
			}
		})?;

		let client_state =
			ctx.client_state(&client_id).map_err(|_| IbcHandlerError::ClientStateError {
				msg: Some(format!("CLient state not found for {client_id:?}")),
			})?;
		let consensus_state = ctx
			.consensus_state(&client_id, client_state.latest_height())
			.map_err(|_| IbcHandlerError::Other {
				msg: Some(format!(
					"Consensus state not found for {:?} at {:?}",
					client_id,
					client_state.latest_height()
				)),
			})?;
		Ok((client_state.latest_height(), consensus_state.timestamp()))
	}
}

pub fn host_height<T: Config>() -> u64 {
	let block_number: u32 =
		<frame_system::Pallet<T>>::block_number().into().saturated_into::<u32>();
	block_number.into()
}
