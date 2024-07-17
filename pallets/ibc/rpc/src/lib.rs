#![warn(missing_docs)]

//! IBC RPC Implementation.

use ibc::{
	core::{
		ics03_connection::connection::ConnectionEnd,
		ics04_channel::channel::{ChannelEnd, IdentifiedChannelEnd, Order},
		ics24_host::identifier::{ChannelId, ConnectionId, PortId},
	},
	events::IbcEvent as RawIbcEvent,
};
use ibc_primitives::PacketInfo as RawPacketInfo;
use ibc_proto::{
	cosmos::base::{query::v1beta1::PageResponse, v1beta1::Coin},
	ibc::{
		applications::transfer::v1::{QueryDenomTraceResponse, QueryDenomTracesResponse},
		core::{
			channel::v1::{
				PacketState, QueryChannelResponse, QueryChannelsResponse,
				QueryNextSequenceReceiveResponse, QueryPacketAcknowledgementResponse,
				QueryPacketAcknowledgementsResponse, QueryPacketCommitmentResponse,
				QueryPacketCommitmentsResponse, QueryPacketReceiptResponse,
			},
			client::v1::{
				Height, IdentifiedClientState, QueryClientStateResponse,
				QueryConsensusStateResponse,
			},
			connection::v1::{
				IdentifiedConnection, QueryConnectionResponse, QueryConnectionsResponse,
			},
		},
	},
};
use ibc_runtime_api::IbcRuntimeApi;
use jsonrpsee::{
	core::{Error as RpcError, RpcResult},
	proc_macros::rpc,
	tracing::log,
	types::ErrorObject,
};
use pallet_ibc::{
	events::IbcEvent,
	light_clients::{AnyClientState, AnyConsensusState},
};
use parity_scale_codec::Encode;
use sc_chain_spec::Properties;
use sc_client_api::{BlockBackend, ProofProvider};
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::{blake2_256, storage::ChildInfo};
use sp_runtime::{
	generic::{BlockId, SignedBlock},
	traits::{Block as BlockT, Header as HeaderT},
};
use std::{collections::HashMap, fmt::Display, str::FromStr, sync::Arc};
use tendermint_proto::Protobuf;
pub mod events;
use events::filter_map_pallet_event;
use ibc_proto::ibc::core::channel::v1::IdentifiedChannel;
use pallet_ibc::errors::IbcError;

/// Connection handshake proof
#[derive(Clone, Serialize, Deserialize)]
pub struct ConnHandshakeProof {
	/// Protobuf encoded client state
	pub client_state: IdentifiedClientState,
	/// Trie proof for connection state, client state and consensus state
	pub proof: Vec<u8>,
	/// Proof height
	pub height: Height,
}

/// A type that could be a block number or a block hash
#[derive(Clone, Hash, Debug, PartialEq, Eq, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockNumberOrHash<Hash> {
	/// Block hash
	Hash(Hash),
	/// Block number
	Number(u32),
}

/// A type that could be a block number or a block hash
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeightAndTimestamp {
	/// Height
	pub height: Height,
	/// Timestamp nano seconds
	pub timestamp: u64,
}

impl<Hash: std::fmt::Debug> Display for BlockNumberOrHash<Hash> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			BlockNumberOrHash::Hash(hash) => write!(f, "{hash:?}"),
			BlockNumberOrHash::Number(block_num) => write!(f, "{block_num}"),
		}
	}
}

/// Proof for a set of keys
#[derive(Clone, Serialize, Deserialize)]
pub struct Proof {
	/// Trie proof
	pub proof: Vec<u8>,
	/// Height at which proof was recovered
	pub height: ibc_proto::ibc::core::client::v1::Height,
}

/// Packet info
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug, PartialOrd, Ord)]
pub struct PacketInfo {
	/// Minimal height at which packet proof is available
	pub height: Option<u64>,
	/// Packet sequence
	pub sequence: u64,
	/// Source port
	pub source_port: String,
	/// Source channel
	pub source_channel: String,
	/// Destination port
	pub destination_port: String,
	/// Destination channel
	pub destination_channel: String,
	/// Channel order
	pub channel_order: String,
	/// Opaque packet data
	pub data: Vec<u8>,
	/// Timeout height
	pub timeout_height: Height,
	/// Timeout timestamp
	pub timeout_timestamp: u64,
	/// Packet acknowledgement
	pub ack: Option<Vec<u8>>,
}

impl TryFrom<RawPacketInfo> for PacketInfo {
	type Error = ();

	fn try_from(info: RawPacketInfo) -> core::result::Result<Self, ()> {
		log::info!("RawPacketInfo: {:?}", info);
		Ok(Self {
			height: info.height,
			sequence: info.sequence,
			source_port: String::from_utf8(info.source_port).map_err(|_| ())?,
			source_channel: String::from_utf8(info.source_channel).map_err(|_| ())?,
			destination_port: String::from_utf8(info.destination_port).map_err(|_| ())?,
			destination_channel: String::from_utf8(info.destination_channel).map_err(|_| ())?,
			channel_order: info.channel_order.to_string(),
			data: info.data,
			timeout_height: Height {
				revision_number: info.timeout_height.0,
				revision_height: info.timeout_height.1,
			},
			timeout_timestamp: info.timeout_timestamp,
			ack: info.ack,
		})
	}
}

/// IBC RPC methods.
#[rpc(client, server)]
pub trait IbcApi<BlockNumber, Hash, AssetId>
where
	Hash: PartialEq + Eq + std::hash::Hash,
	AssetId: parity_scale_codec::Codec,
{
	/// Query packet data
	#[method(name = "ibc_querySendPackets")]
	fn query_send_packets(
		&self,
		channel_id: String,
		port_id: String,
		seqs: Vec<u64>,
	) -> RpcResult<Vec<PacketInfo>>;
	/// Query Recv Packet
	#[method(name = "ibc_queryRecvPackets")]
	fn query_recv_packets(
		&self,
		channel_id: String,
		port_id: String,
		seqs: Vec<u64>,
	) -> RpcResult<Vec<PacketInfo>>;

	/// Query local time and height that a client was updated
	#[method(name = "ibc_clientUpdateTimeAndHeight")]
	fn query_client_update_time_and_height(
		&self,
		client_id: String,
		revision_number: u64,
		revision_height: u64,
	) -> RpcResult<HeightAndTimestamp>;

	/// Generate proof for given key
	#[method(name = "ibc_queryProof")]
	fn query_proof(&self, height: u32, keys: Vec<Vec<u8>>) -> RpcResult<Proof>;

	/// Query latest height
	#[method(name = "ibc_queryLatestHeight")]
	fn query_latest_height(&self) -> RpcResult<BlockNumber>;

	/// Query balance of an address on chain, addr should be a valid hexadecimal or SS58 string,
	/// representing the account id.
	#[method(name = "ibc_queryBalanceWithAddress")]
	fn query_balance_with_address(&self, addr: String, asset_id: AssetId) -> RpcResult<Coin>;

	/// Query a client state
	#[method(name = "ibc_queryClientState")]
	fn query_client_state(
		&self,
		height: u32,
		src_client_id: String,
	) -> RpcResult<QueryClientStateResponse>;

	/// Query client consensus state
	/// If the light client is a beefy light client, the revision height and revision number must be
	/// specified And the `latest_consensus_state` field should be set to false, if not an error
	/// will be returned because the consensus state will not be found
	/// For a beefy light client revision number should be the para id and the revision height the
	/// block height.
	#[method(name = "ibc_queryClientConsensusState")]
	fn query_client_consensus_state(
		&self,
		height: Option<u32>,
		client_id: String,
		revision_height: u64,
		revision_number: u64,
		latest_consensus_state: bool,
	) -> RpcResult<QueryConsensusStateResponse>;

	/// Query upgraded client state
	#[method(name = "ibc_queryUpgradedClient")]
	fn query_upgraded_client(&self, height: u32) -> RpcResult<QueryClientStateResponse>;

	/// Query upgraded consensus state for client
	#[method(name = "ibc_queryUpgradedConnectionState")]
	fn query_upgraded_cons_state(&self, height: u32) -> RpcResult<QueryConsensusStateResponse>;

	/// Query all client states
	#[method(name = "ibc_queryClients")]
	fn query_clients(&self) -> RpcResult<Vec<IdentifiedClientState>>;

	/// Query a connection state
	#[method(name = "ibc_queryConnection")]
	fn query_connection(
		&self,
		height: u32,
		connection_id: String,
	) -> RpcResult<QueryConnectionResponse>;

	/// Query all connection states
	#[method(name = "ibc_queryConnections")]
	fn query_connections(&self) -> RpcResult<QueryConnectionsResponse>;

	/// Query all connection states for associated client
	#[method(name = "ibc_queryConnectionUsingClient")]
	fn query_connection_using_client(
		&self,
		height: u32,
		client_id: String,
	) -> RpcResult<Vec<IdentifiedConnection>>;

	/// Generate proof for connection handshake
	#[method(name = "ibc_generateConnectionHandshakeProof")]
	fn generate_conn_handshake_proof(
		&self,
		height: u32,
		client_id: String,
		conn_id: String,
	) -> RpcResult<ConnHandshakeProof>;

	/// Query a channel state
	#[method(name = "ibc_queryChannel")]
	fn query_channel(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
	) -> RpcResult<QueryChannelResponse>;

	/// Query client state for channel and port id
	#[method(name = "ibc_queryChannelClient")]
	fn query_channel_client(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
	) -> RpcResult<IdentifiedClientState>;

	/// Query all channel states for associated connection
	#[method(name = "ibc_queryConnectionChannels")]
	fn query_connection_channels(
		&self,
		height: u32,
		connection_id: String,
	) -> RpcResult<QueryChannelsResponse>;

	/// Query all channel states
	#[method(name = "ibc_queryChannels")]
	fn query_channels(&self) -> RpcResult<QueryChannelsResponse>;

	/// Query packet commitments
	#[method(name = "ibc_queryPacketCommitments")]
	fn query_packet_commitments(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
	) -> RpcResult<QueryPacketCommitmentsResponse>;

	/// Query packet acknowledgements
	#[method(name = "ibc_queryPacketAcknowledgements")]
	fn query_packet_acknowledgements(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
	) -> RpcResult<QueryPacketAcknowledgementsResponse>;

	/// Given a list of counterparty packet commitments, the querier checks if the packet
	/// has already been received by checking if a receipt exists on this
	/// chain for the packet sequence. All packets that haven't been received yet
	/// are returned in the response
	/// Usage: To use this method correctly, first query all packet commitments on
	/// the sending chain then send the request to this query_unreceived_packets on this
	/// chain. This method should then return the list of packet sequences that
	/// are yet to be received on this chain.
	/// NOTE: WORKS ONLY FOR UNORDERED CHANNELS
	/// CALLER IS RESPONSIBLE FOR PROVIDING A CORRECT LIST OF PACKET COMMITMENT
	/// SEQUENCES FROM COUNTERPARTY.
	#[method(name = "ibc_queryUnreceivedPackets")]
	fn query_unreceived_packets(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
		seqs: Vec<u64>,
	) -> RpcResult<Vec<u64>>;

	/// Given a list of counterparty packet acknowledgements, the querier checks if the ack
	/// has already been received by checking if a packet commitment exists on this
	/// chain for the packet sequence. All acks that haven't been received yet
	/// are returned in the response
	/// Usage: To use this method correctly, first query all packet acks on
	/// the counterparty chain then send the request to this query_unreceived_acknowledgements on
	/// this chain. This method should then return the list of packet acks that
	/// are yet to be received on this chain.
	/// NOTE: WORKS ONLY FOR UNORDERED CHANNELS
	/// CALLER IS RESPONSIBLE FOR PROVIDING A CORRECT LIST OF PACKET ACKNOWLEDGEMENT
	/// SEQUENCES FROM COUNTERPARTY.
	#[method(name = "ibc_queryUnreceivedAcknowledgement")]
	fn query_unreceived_acknowledgements(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
		seqs: Vec<u64>,
	) -> RpcResult<Vec<u64>>;

	/// Query next sequence to be received on channel
	#[method(name = "ibc_queryNextSeqRecv")]
	fn query_next_seq_recv(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
	) -> RpcResult<QueryNextSequenceReceiveResponse>;

	/// Query packet commitment
	#[method(name = "ibc_queryPacketCommitment")]
	fn query_packet_commitment(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
		seq: u64,
	) -> RpcResult<QueryPacketCommitmentResponse>;

	/// Query packet acknowledgement
	#[method(name = "ibc_queryPacketAcknowledgement")]
	fn query_packet_acknowledgement(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
		seq: u64,
	) -> RpcResult<QueryPacketAcknowledgementResponse>;

	/// Query packet receipt
	#[method(name = "ibc_queryPacketReceipt")]
	fn query_packet_receipt(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
		seq: u64,
	) -> RpcResult<QueryPacketReceiptResponse>;

	/// Query the denom trace for an ibc denom from the asset Id
	// In ibc-go this method accepts a string which is the hash of the ibc denom
	// that is because ibc denoms are stored as hashes in ibc-go, but in our implementation here
	#[method(name = "ibc_queryDenomTrace")]
	fn query_denom_trace(&self, asset_id: AssetId) -> RpcResult<QueryDenomTraceResponse>;

	/// Query the denom traces for ibc denoms
	/// key is the asset id from which to start paginating results
	/// The next_key value in the pagination field of the returned result is a scale encoded u128
	/// value
	/// Only one of offset or key should be set, if both are set, key is used instead
	#[method(name = "ibc_queryDenomTraces")]
	fn query_denom_traces(
		&self,
		key: Option<AssetId>,
		offset: Option<u32>,
		limit: Option<u64>,
		count_total: bool,
	) -> RpcResult<QueryDenomTracesResponse>;

	/// Query newly created client in block and extrinsic
	#[method(name = "ibc_queryNewlyCreatedClient")]
	fn query_newly_created_client(
		&self,
		block_hash: Hash,
		ext_hash: Hash,
	) -> RpcResult<IdentifiedClientState>;

	/// Query newly created connection in block and extrinsic
	#[method(name = "ibc_queryNewlyCreatedConnection")]
	fn query_newly_created_connection(
		&self,
		block_hash: Hash,
		ext_hash: Hash,
	) -> RpcResult<IdentifiedConnection>;

	/// Query newly created channel in block and extrinsic
	#[method(name = "ibc_queryNewlyCreatedChannel")]
	fn query_newly_created_channel(
		&self,
		block_hash: Hash,
		ext_hash: Hash,
	) -> RpcResult<IdentifiedChannel>;

	/// Query Ibc Events that were deposited in a series of blocks
	/// Using String keys because HashMap fails to deserialize when key is not a String
	#[method(name = "ibc_queryEvents")]
	fn query_events(
		&self,
		block_numbers: Vec<BlockNumberOrHash<Hash>>,
	) -> RpcResult<HashMap<String, Vec<RawIbcEvent>>>;
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_error(e: impl std::fmt::Display) -> ErrorObject<'static> {
	ErrorObject::owned(
		9876, // no real reason for this value
		"Something wrong",
		Some(format!("{e}")),
	)
}

/// An implementation of IBC specific RPC methods.
pub struct IbcRpcHandler<C, B, AssetId> {
	client: Arc<C>,
	/// A copy of the chain properties.
	pub chain_props: Properties,
	_marker: std::marker::PhantomData<(B, AssetId)>,
}

impl<C, B, AssetId> IbcRpcHandler<C, B, AssetId> {
	/// Create new `IbcRpcHandler` with the given reference to the client.
	pub fn new(client: Arc<C>, chain_props: Properties) -> Self {
		Self { client, chain_props, _marker: Default::default() }
	}
}

impl<C, Block, AssetId>
	IbcApiServer<<<Block as BlockT>::Header as HeaderT>::Number, Block::Hash, AssetId>
	for IbcRpcHandler<C, Block, AssetId>
where
	Block: BlockT,
	C: Send
		+ Sync
		+ 'static
		+ ProvideRuntimeApi<Block>
		+ HeaderBackend<Block>
		+ ProofProvider<Block>
		+ BlockBackend<Block>,
	C::Api: IbcRuntimeApi<Block, AssetId>,
	AssetId: parity_scale_codec::Codec + Copy + Send + Sync + 'static,
{
	fn query_send_packets(
		&self,
		channel_id: String,
		port_id: String,
		seqs: Vec<u64>,
	) -> RpcResult<Vec<PacketInfo>> {
		let api = self.client.runtime_api();
		let packets: Vec<ibc_primitives::PacketInfo> = api
			.query_send_packet_info(
				self.client.info().best_hash,
				channel_id.as_bytes().to_vec(),
				port_id.as_bytes().to_vec(),
				seqs,
			)
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Error fetching packets"))?;

		packets
			.into_iter()
			.map(|packet| {
				Ok(PacketInfo {
					sequence: packet.sequence,
					source_port: String::from_utf8(packet.source_port).map_err(|_| {
						runtime_error_into_rpc_error("Failed to decode source port")
					})?,
					source_channel: String::from_utf8(packet.source_channel).map_err(|_| {
						runtime_error_into_rpc_error("Failed to decode source channel")
					})?,
					destination_port: String::from_utf8(packet.destination_port).map_err(|_| {
						runtime_error_into_rpc_error("Failed to decode destination port")
					})?,
					destination_channel: String::from_utf8(packet.destination_channel).map_err(
						|_| runtime_error_into_rpc_error("Failed to decode destination channel"),
					)?,
					data: packet.data,
					timeout_height: Height {
						revision_number: packet.timeout_height.0,
						revision_height: packet.timeout_height.1,
					},
					timeout_timestamp: packet.timeout_timestamp,
					height: packet.height,
					channel_order: {
						Order::from_i32(packet.channel_order as i32)
							.map_err(|_| {
								runtime_error_into_rpc_error(
									"Packet info should have a valid channel order",
								)
							})?
							.to_string()
					},
					ack: packet.ack,
				})
			})
			.collect()
	}

	fn query_recv_packets(
		&self,
		channel_id: String,
		port_id: String,
		seqs: Vec<u64>,
	) -> RpcResult<Vec<PacketInfo>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;
		let packets: Vec<ibc_primitives::PacketInfo> = api
			.query_recv_packet_info(
				at,
				channel_id.as_bytes().to_vec(),
				port_id.as_bytes().to_vec(),
				seqs,
			)
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Error fetching packets"))?;

		packets
			.into_iter()
			.map(|packet| {
				Ok(PacketInfo {
					sequence: packet.sequence,
					source_port: String::from_utf8(packet.source_port).map_err(|_| {
						runtime_error_into_rpc_error("Failed to decode source port")
					})?,
					source_channel: String::from_utf8(packet.source_channel).map_err(|_| {
						runtime_error_into_rpc_error("Failed to decode source channel")
					})?,
					destination_port: String::from_utf8(packet.destination_port).map_err(|_| {
						runtime_error_into_rpc_error("Failed to decode destination port")
					})?,
					destination_channel: String::from_utf8(packet.destination_channel).map_err(
						|_| runtime_error_into_rpc_error("Failed to decode destination channel"),
					)?,
					data: packet.data,
					timeout_height: Height {
						revision_number: packet.timeout_height.0,
						revision_height: packet.timeout_height.1,
					},
					timeout_timestamp: packet.timeout_timestamp,
					height: packet.height,
					channel_order: {
						Order::from_i32(packet.channel_order as i32)
							.map_err(|_| {
								runtime_error_into_rpc_error(
									"Packet info should have a valid channel order",
								)
							})?
							.to_string()
					},
					ack: packet.ack,
				})
			})
			.collect()
	}

	fn query_client_update_time_and_height(
		&self,
		client_id: String,
		revision_number: u64,
		revision_height: u64,
	) -> RpcResult<HeightAndTimestamp> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;
		let para_id = api
			.para_id(at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let (update_height, update_time) = api
			.client_update_time_and_height(
				at,
				client_id.as_bytes().to_vec(),
				revision_number,
				revision_height,
			)
			.ok()
			.flatten()
			.ok_or_else(|| {
				runtime_error_into_rpc_error("Failed to get client update time and height")
			})?;
		Ok(HeightAndTimestamp {
			height: Height { revision_number: para_id.into(), revision_height: update_height },
			timestamp: update_time,
		})
	}

	fn query_proof(&self, height: u32, mut keys: Vec<Vec<u8>>) -> RpcResult<Proof> {
		let api = self.client.runtime_api();
		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let para_id = api
			.para_id(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let child_trie_key = api
			.child_trie_key(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Failed to get child trie key"))?;
		let child_info = ChildInfo::new_default(&child_trie_key);
		let proof = self
			.client
			.read_child_proof(hash_at, &child_info, &mut keys.iter_mut().map(|nodes| &nodes[..]))
			.map_err(runtime_error_into_rpc_error)?
			.iter_nodes()
			.collect::<Vec<_>>()
			.encode();
		Ok(Proof {
			proof,
			height: Height { revision_number: para_id.into(), revision_height: height as u64 },
		})
	}

	fn query_latest_height(&self) -> RpcResult<<<Block as BlockT>::Header as HeaderT>::Number> {
		if let Ok(Some(height)) = self.client.number(self.client.info().best_hash) {
			Ok(height)
		} else {
			Err(runtime_error_into_rpc_error("Could not get latest height"))
		}
	}

	fn query_balance_with_address(&self, addr: String, asset_id: AssetId) -> RpcResult<Coin> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;
		let denom = String::from_utf8(
			api.denom_trace(at, asset_id)
				.map_err(|e| {
					runtime_error_into_rpc_error(format!("failed to get denom trace: {e}"))
				})?
				.ok_or_else(|| runtime_error_into_rpc_error("denom trace not found"))?
				.denom,
		)
		.map_err(|_| runtime_error_into_rpc_error("failed to convert denom to string"))?;

		match api
			.query_balance_with_address(at, addr.as_bytes().to_vec(), asset_id)
			.ok()
			.flatten()
		{
			Some(amt) => Ok(Coin { denom, amount: sp_core::U256::from(amt).as_u128().to_string() }),
			None => Err(runtime_error_into_rpc_error("Error querying balance")),
		}
	}

	fn query_client_state(
		&self,
		height: u32,
		client_id: String,
	) -> RpcResult<QueryClientStateResponse> {
		let api = self.client.runtime_api();

		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let para_id = api
			.para_id(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let result: ibc_primitives::QueryClientStateResponse = api
			.client_state(hash_at, client_id.as_bytes().to_vec())
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("[API] Error querying client state"))?;
		let mut keys = vec![result.trie_key];
		let child_trie_key = api
			.child_trie_key(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Failed to get child trie key"))?;
		let child_info = ChildInfo::new_default(&child_trie_key);
		let proof = self
			.client
			.read_child_proof(hash_at, &child_info, &mut keys.iter_mut().map(|nodes| &nodes[..]))
			.map_err(runtime_error_into_rpc_error)?
			.iter_nodes()
			.collect::<Vec<_>>()
			.encode();
		let client_state = AnyClientState::decode_vec(&result.client_state).map_err(|e| {
			runtime_error_into_rpc_error(format!("Error querying client state: {e:?}"))
		})?;
		Ok(QueryClientStateResponse {
			client_state: Some(client_state.into()),
			proof,
			proof_height: Some(ibc_proto::ibc::core::client::v1::Height {
				revision_number: para_id.into(),
				revision_height: result.height,
			}),
		})
	}

	fn query_client_consensus_state(
		&self,
		height: Option<u32>,
		client_id: String,
		revision_height: u64,
		revision_number: u64,
		latest_cs: bool,
	) -> RpcResult<QueryConsensusStateResponse> {
		let api = self.client.runtime_api();
		let at = if let Some(height) = height {
			BlockId::Number(height.into())
		} else {
			BlockId::Hash(self.client.info().best_hash)
		};
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let para_id = api
			.para_id(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let result: ibc_primitives::QueryConsensusStateResponse = api
			.client_consensus_state(
				hash_at,
				client_id.as_bytes().to_vec(),
				revision_number,
				revision_height,
				latest_cs,
			)
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Error querying client consensus state"))?;
		let consensus_state = AnyConsensusState::decode_vec(&result.consensus_state)
			.map_err(|_| runtime_error_into_rpc_error("Error querying client consensus state"))?;
		let mut keys = vec![result.trie_key];
		let child_trie_key = api
			.child_trie_key(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Failed to get child trie key"))?;
		let child_info = ChildInfo::new_default(&child_trie_key);
		let proof = self
			.client
			.read_child_proof(hash_at, &child_info, &mut keys.iter_mut().map(|nodes| &nodes[..]))
			.map_err(runtime_error_into_rpc_error)?
			.iter_nodes()
			.collect::<Vec<_>>()
			.encode();
		Ok(QueryConsensusStateResponse {
			consensus_state: Some(consensus_state.into()),
			proof,
			proof_height: Some(ibc_proto::ibc::core::client::v1::Height {
				revision_number: para_id.into(),
				revision_height: result.height,
			}),
		})
	}
	// TODO: Unimplemented
	fn query_upgraded_client(&self, _height: u32) -> RpcResult<QueryClientStateResponse> {
		Err(runtime_error_into_rpc_error("Unimplemented"))
	}

	fn query_upgraded_cons_state(&self, _height: u32) -> RpcResult<QueryConsensusStateResponse> {
		Err(runtime_error_into_rpc_error("Unimplemented"))
	}

	fn query_clients(&self) -> RpcResult<Vec<IdentifiedClientState>> {
		let api = self.client.runtime_api();

		let client_states: Option<Vec<(Vec<u8>, Vec<u8>)>> =
			api.clients(self.client.info().best_hash).ok().flatten();
		match client_states {
			Some(client_states) => client_states
				.into_iter()
				.map(|(client_id, client_state)| {
					let client_state = AnyClientState::decode_vec(&client_state).map_err(|_| {
						runtime_error_into_rpc_error("Failed to decode client state")
					})?;
					Ok(IdentifiedClientState {
						client_id: String::from_utf8(client_id).map_err(|_| {
							runtime_error_into_rpc_error("Failed to decode client id")
						})?,
						client_state: Some(client_state.into()),
					})
				})
				.collect(),
			_ => Err(runtime_error_into_rpc_error("Failed to fetch client states")),
		}
	}

	fn query_connection(
		&self,
		height: u32,
		connection_id: String,
	) -> RpcResult<QueryConnectionResponse> {
		let api = self.client.runtime_api();

		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let para_id = api
			.para_id(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let result: ibc_primitives::QueryConnectionResponse = api
			.connection(hash_at, connection_id.as_bytes().to_vec())
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Failed to fetch connection state"))?;
		let connection_end =
			ibc::core::ics03_connection::connection::ConnectionEnd::decode_vec(&result.connection)
				.map_err(|_| runtime_error_into_rpc_error("Failed to decode connection end"))?;
		let mut keys = vec![result.trie_key];
		let child_trie_key = api
			.child_trie_key(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Failed to get child trie key"))?;
		let child_info = ChildInfo::new_default(&child_trie_key);
		let proof = self
			.client
			.read_child_proof(hash_at, &child_info, &mut keys.iter_mut().map(|nodes| &nodes[..]))
			.map_err(runtime_error_into_rpc_error)?
			.iter_nodes()
			.collect::<Vec<_>>()
			.encode();
		Ok(QueryConnectionResponse {
			connection: Some(connection_end.into()),
			proof,
			proof_height: Some(ibc_proto::ibc::core::client::v1::Height {
				revision_number: para_id.into(),
				revision_height: result.height,
			}),
		})
	}

	fn query_connections(&self) -> RpcResult<QueryConnectionsResponse> {
		let api = self.client.runtime_api();

		let at = BlockId::Hash(self.client.info().best_hash);
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let result: ibc_primitives::QueryConnectionsResponse = api
			.connections(hash_at)
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Failed to fetch connections"))?;
		let para_id = api
			.para_id(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let connections = result
			.connections
			.into_iter()
			.map(|identified_connection| {
				let connection_id = String::from_utf8(identified_connection.connection_id)
					.map_err(|_| runtime_error_into_rpc_error("Failed to decode connection id"))?;
				let connection_id = ConnectionId::from_str(&connection_id)
					.map_err(|_| runtime_error_into_rpc_error("Failed to decode connection id"))?;
				let connection_end = ConnectionEnd::decode_vec(
					&identified_connection.connection_end,
				)
				.map_err(|_| runtime_error_into_rpc_error("Failed to decode connection end"))?;
				let identified_connection =
					ibc::core::ics03_connection::connection::IdentifiedConnectionEnd::new(
						connection_id,
						connection_end,
					);
				let identified_connection: IdentifiedConnection = identified_connection.into();
				Ok(identified_connection)
			})
			.collect::<RpcResult<Vec<_>>>()?;
		Ok(QueryConnectionsResponse {
			connections,
			pagination: None,
			height: Some(ibc_proto::ibc::core::client::v1::Height {
				revision_number: para_id.into(),
				revision_height: result.height,
			}),
		})
	}

	fn query_connection_using_client(
		&self,
		height: u32,
		client_id: String,
	) -> RpcResult<Vec<IdentifiedConnection>> {
		let api = self.client.runtime_api();

		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let result: Vec<ibc_primitives::IdentifiedConnection> = api
			.connection_using_client(hash_at, client_id.as_bytes().to_vec())
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Failed to fetch connections"))?;
		result
			.into_iter()
			.map(|ident_conn| {
				let connection_id = String::from_utf8(ident_conn.connection_id)
					.map_err(|_| runtime_error_into_rpc_error("Failed to decode connection id"))?;
				let connection_id = ConnectionId::from_str(&connection_id)
					.map_err(|_| runtime_error_into_rpc_error("Failed to decode connection id"))?;
				let connection_end = ConnectionEnd::decode_vec(&ident_conn.connection_end)
					.map_err(|_| runtime_error_into_rpc_error("Failed to decode connection end"))?;
				let identified_connection =
					ibc::core::ics03_connection::connection::IdentifiedConnectionEnd::new(
						connection_id,
						connection_end,
					);
				let identified_connection: IdentifiedConnection = identified_connection.into();
				Ok(identified_connection)
			})
			.collect::<RpcResult<Vec<_>>>()
	}

	fn generate_conn_handshake_proof(
		&self,
		height: u32,
		client_id: String,
		conn_id: String,
	) -> RpcResult<ConnHandshakeProof> {
		let api = self.client.runtime_api();

		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let para_id = api
			.para_id(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let mut result: ibc_primitives::ConnectionHandshake = api
			.connection_handshake(
				hash_at,
				client_id.as_bytes().to_vec(),
				conn_id.as_bytes().to_vec(),
			)
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Error getting trie inputs"))?;
		let child_trie_key = api
			.child_trie_key(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Failed to get child trie key"))?;
		let child_info = ChildInfo::new_default(&child_trie_key);
		let proof = self
			.client
			.read_child_proof(
				hash_at,
				&child_info,
				&mut result.trie_keys.iter_mut().map(|nodes| &nodes[..]),
			)
			.map_err(runtime_error_into_rpc_error)?
			.iter_nodes()
			.collect::<Vec<_>>()
			.encode();

		let client_state = AnyClientState::decode_vec(&result.client_state)
			.map_err(|_| runtime_error_into_rpc_error("Failed to decode client state"))?;
		Ok(ConnHandshakeProof {
			client_state: IdentifiedClientState {
				client_id,
				client_state: Some(client_state.into()),
			},
			proof,
			height: ibc_proto::ibc::core::client::v1::Height {
				revision_number: para_id.into(),
				revision_height: result.height,
			},
		})
	}

	fn query_channel(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
	) -> RpcResult<QueryChannelResponse> {
		let api = self.client.runtime_api();

		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let para_id = api
			.para_id(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let result: ibc_primitives::QueryChannelResponse = api
			.channel(hash_at, channel_id.as_bytes().to_vec(), port_id.as_bytes().to_vec())
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Failed to fetch channel state"))?;
		let channel = ibc::core::ics04_channel::channel::ChannelEnd::decode_vec(&result.channel)
			.map_err(|_| runtime_error_into_rpc_error("Failed to decode channel state"))?;
		let mut keys = vec![result.trie_key];
		let child_trie_key = api
			.child_trie_key(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Failed to get child trie key"))?;
		let child_info = ChildInfo::new_default(&child_trie_key);
		let proof = self
			.client
			.read_child_proof(hash_at, &child_info, &mut keys.iter_mut().map(|nodes| &nodes[..]))
			.map_err(runtime_error_into_rpc_error)?
			.iter_nodes()
			.collect::<Vec<_>>()
			.encode();
		Ok(QueryChannelResponse {
			channel: Some(channel.into()),
			proof,
			proof_height: Some(ibc_proto::ibc::core::client::v1::Height {
				revision_number: para_id.into(),
				revision_height: result.height,
			}),
		})
	}

	fn query_channel_client(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
	) -> RpcResult<IdentifiedClientState> {
		let api = self.client.runtime_api();

		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let result: ibc_primitives::IdentifiedClientState = api
			.channel_client(hash_at, channel_id.as_bytes().to_vec(), port_id.as_bytes().to_vec())
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Failed to Client state for channel"))?;

		let client_state = AnyClientState::decode_vec(&result.client_state)
			.map_err(|_| runtime_error_into_rpc_error("Failed to decode client state"))?;
		Ok(IdentifiedClientState {
			client_id: String::from_utf8(result.client_id)
				.map_err(|_| runtime_error_into_rpc_error("Failed to decode client id"))?,
			client_state: Some(client_state.into()),
		})
	}

	fn query_connection_channels(
		&self,
		height: u32,
		connection_id: String,
	) -> RpcResult<QueryChannelsResponse> {
		let api = self.client.runtime_api();

		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let para_id = api
			.para_id(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let result: ibc_primitives::QueryChannelsResponse = api
			.connection_channels(hash_at, connection_id.as_bytes().to_vec())
			.ok()
			.flatten()
			.ok_or_else(|| {
				runtime_error_into_rpc_error("Failed to fetch channels state for connection")
			})?;
		let channels = result
			.channels
			.into_iter()
			.map(|temp| {
				let port_id = PortId::from_str(
					&String::from_utf8(temp.port_id)
						.map_err(|_| runtime_error_into_rpc_error("Failed to decode port id"))?,
				)
				.map_err(|_| runtime_error_into_rpc_error("Failed to decode port id"))?;
				let channel_id = ChannelId::from_str(
					&String::from_utf8(temp.channel_id)
						.map_err(|_| runtime_error_into_rpc_error("Failed to decode port id"))?,
				)
				.map_err(|_| runtime_error_into_rpc_error("Failed to decode port id"))?;
				let channel_end = ChannelEnd::decode_vec(&temp.channel_end)
					.map_err(|_| runtime_error_into_rpc_error("Failed to decode port id"))?;
				let identified_channel =
					IdentifiedChannelEnd::new(port_id, channel_id, channel_end);
				let identified_channel: ibc_proto::ibc::core::channel::v1::IdentifiedChannel =
					identified_channel.into();
				Ok(identified_channel)
			})
			.collect::<RpcResult<Vec<_>>>()?;

		Ok(QueryChannelsResponse {
			channels,
			pagination: None,
			height: Some(ibc_proto::ibc::core::client::v1::Height {
				revision_number: para_id.into(),
				revision_height: result.height,
			}),
		})
	}

	fn query_channels(&self) -> RpcResult<QueryChannelsResponse> {
		let api = self.client.runtime_api();
		let at = BlockId::Hash(self.client.info().best_hash);
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let para_id = api
			.para_id(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let result: ibc_primitives::QueryChannelsResponse = api
			.channels(hash_at)
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Failed to fetch channels"))?;
		let channels = result
			.channels
			.into_iter()
			.map(|temp| {
				let port_id = PortId::from_str(
					&String::from_utf8(temp.port_id)
						.map_err(|_| runtime_error_into_rpc_error("Failed to decode port id"))?,
				)
				.map_err(|_| runtime_error_into_rpc_error("Failed to decode port id"))?;
				let channel_id = ChannelId::from_str(
					&String::from_utf8(temp.channel_id)
						.map_err(|_| runtime_error_into_rpc_error("Failed to decode port id"))?,
				)
				.map_err(|_| runtime_error_into_rpc_error("Failed to decode port id"))?;
				let channel_end = ChannelEnd::decode_vec(&temp.channel_end)
					.map_err(|_| runtime_error_into_rpc_error("Failed to decode port id"))?;
				let identified_channel =
					IdentifiedChannelEnd::new(port_id, channel_id, channel_end);
				let identified_channel: ibc_proto::ibc::core::channel::v1::IdentifiedChannel =
					identified_channel.into();
				Ok(identified_channel)
			})
			.collect::<RpcResult<Vec<_>>>()?;

		Ok(QueryChannelsResponse {
			channels,
			pagination: None,
			height: Some(ibc_proto::ibc::core::client::v1::Height {
				revision_number: para_id.into(),
				revision_height: result.height,
			}),
		})
	}

	fn query_packet_commitments(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
	) -> RpcResult<QueryPacketCommitmentsResponse> {
		let api = self.client.runtime_api();

		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let para_id = api
			.para_id(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let result: ibc_primitives::QueryPacketCommitmentsResponse = api
			.packet_commitments(
				hash_at,
				channel_id.as_bytes().to_vec(),
				port_id.as_bytes().to_vec(),
			)
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Failed to fetch commitments"))?;
		let commitments = result
			.commitments
			.into_iter()
			.map(|packet_state| {
				let port_id = String::from_utf8(packet_state.port_id)
					.map_err(|_| runtime_error_into_rpc_error("Failed to decode port id"))?;
				let channel_id = String::from_utf8(packet_state.channel_id)
					.map_err(|_| runtime_error_into_rpc_error("Failed to decode port id"))?;
				Ok(PacketState {
					port_id,
					channel_id,
					sequence: packet_state.sequence,
					data: packet_state.data,
				})
			})
			.collect::<RpcResult<Vec<_>>>()?;
		Ok(QueryPacketCommitmentsResponse {
			commitments,
			pagination: None,
			height: Some(ibc_proto::ibc::core::client::v1::Height {
				revision_number: para_id.into(),
				revision_height: result.height,
			}),
		})
	}

	fn query_packet_acknowledgements(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
	) -> RpcResult<QueryPacketAcknowledgementsResponse> {
		let api = self.client.runtime_api();

		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;

		let para_id = api
			.para_id(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let result: ibc_primitives::QueryPacketAcknowledgementsResponse = api
			.packet_acknowledgements(
				hash_at,
				channel_id.as_bytes().to_vec(),
				port_id.as_bytes().to_vec(),
			)
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Failed to fetch acknowledgements"))?;
		let acknowledgements = result
			.acks
			.into_iter()
			.map(|packet_state| {
				let port_id = String::from_utf8(packet_state.port_id)
					.map_err(|_| runtime_error_into_rpc_error("Failed to decode port id"))?;
				let channel_id = String::from_utf8(packet_state.channel_id)
					.map_err(|_| runtime_error_into_rpc_error("Failed to decode port id"))?;
				Ok(PacketState {
					port_id,
					channel_id,
					sequence: packet_state.sequence,
					data: packet_state.data,
				})
			})
			.collect::<RpcResult<Vec<_>>>()?;
		Ok(QueryPacketAcknowledgementsResponse {
			acknowledgements,
			pagination: None,
			height: Some(ibc_proto::ibc::core::client::v1::Height {
				revision_number: para_id.into(),
				revision_height: result.height,
			}),
		})
	}

	fn query_unreceived_packets(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
		seqs: Vec<u64>,
	) -> RpcResult<Vec<u64>> {
		let api = self.client.runtime_api();
		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;

		api.unreceived_packets(
			hash_at,
			channel_id.as_bytes().to_vec(),
			port_id.as_bytes().to_vec(),
			seqs,
		)
		.ok()
		.flatten()
		.ok_or_else(|| runtime_error_into_rpc_error("Failed to unreceived packet sequences"))
	}

	fn query_unreceived_acknowledgements(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
		seqs: Vec<u64>,
	) -> RpcResult<Vec<u64>> {
		let api = self.client.runtime_api();
		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;

		api.unreceived_acknowledgements(
			hash_at,
			channel_id.as_bytes().to_vec(),
			port_id.as_bytes().to_vec(),
			seqs,
		)
		.ok()
		.flatten()
		.ok_or_else(|| runtime_error_into_rpc_error("Failed to unreceived packet sequences"))
	}

	fn query_next_seq_recv(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
	) -> RpcResult<QueryNextSequenceReceiveResponse> {
		let api = self.client.runtime_api();

		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let para_id = api
			.para_id(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let result: ibc_primitives::QueryNextSequenceReceiveResponse = api
			.next_seq_recv(hash_at, channel_id.as_bytes().to_vec(), port_id.as_bytes().to_vec())
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Error fetching next sequence recv"))?;
		let mut keys = vec![result.trie_key];
		let child_trie_key = api
			.child_trie_key(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Failed to get child trie key"))?;
		let child_info = ChildInfo::new_default(&child_trie_key);
		let proof = self
			.client
			.read_child_proof(hash_at, &child_info, &mut keys.iter_mut().map(|nodes| &nodes[..]))
			.map_err(runtime_error_into_rpc_error)?
			.iter_nodes()
			.collect::<Vec<_>>()
			.encode();
		Ok(QueryNextSequenceReceiveResponse {
			next_sequence_receive: result.sequence,
			proof,
			proof_height: Some(ibc_proto::ibc::core::client::v1::Height {
				revision_number: para_id.into(),
				revision_height: result.height,
			}),
		})
	}

	fn query_packet_commitment(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
		seq: u64,
	) -> RpcResult<QueryPacketCommitmentResponse> {
		let api = self.client.runtime_api();

		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let para_id = api
			.para_id(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let result: ibc_primitives::QueryPacketCommitmentResponse = api
			.packet_commitment(
				hash_at,
				channel_id.as_bytes().to_vec(),
				port_id.as_bytes().to_vec(),
				seq,
			)
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Error fetching packet commitment"))?;
		let mut keys = vec![result.trie_key];
		let child_trie_key = api
			.child_trie_key(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Failed to get child trie key"))?;
		let child_info = ChildInfo::new_default(&child_trie_key);
		let proof = self
			.client
			.read_child_proof(hash_at, &child_info, &mut keys.iter_mut().map(|nodes| &nodes[..]))
			.map_err(runtime_error_into_rpc_error)?
			.iter_nodes()
			.collect::<Vec<_>>()
			.encode();
		Ok(QueryPacketCommitmentResponse {
			commitment: result.commitment,
			proof,
			proof_height: Some(ibc_proto::ibc::core::client::v1::Height {
				revision_number: para_id.into(),
				revision_height: result.height,
			}),
		})
	}

	fn query_packet_acknowledgement(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
		seq: u64,
	) -> RpcResult<QueryPacketAcknowledgementResponse> {
		let api = self.client.runtime_api();

		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let para_id = api
			.para_id(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let result: ibc_primitives::QueryPacketAcknowledgementResponse = api
			.packet_acknowledgement(
				hash_at,
				channel_id.as_bytes().to_vec(),
				port_id.as_bytes().to_vec(),
				seq,
			)
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Error fetching packet acknowledgement"))?;
		let mut keys = vec![result.trie_key];
		let child_trie_key = api
			.child_trie_key(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Failed to get child trie key"))?;
		let child_info = ChildInfo::new_default(&child_trie_key);
		let proof = self
			.client
			.read_child_proof(hash_at, &child_info, &mut keys.iter_mut().map(|nodes| &nodes[..]))
			.map_err(runtime_error_into_rpc_error)?
			.iter_nodes()
			.collect::<Vec<_>>()
			.encode();
		Ok(QueryPacketAcknowledgementResponse {
			acknowledgement: result.ack,
			proof,
			proof_height: Some(ibc_proto::ibc::core::client::v1::Height {
				revision_number: para_id.into(),
				revision_height: result.height,
			}),
		})
	}

	fn query_packet_receipt(
		&self,
		height: u32,
		channel_id: String,
		port_id: String,
		seq: u64,
	) -> RpcResult<QueryPacketReceiptResponse> {
		let api = self.client.runtime_api();

		let at = BlockId::Number(height.into());
		let hash_at = self
			.client
			.block_hash_from_id(&at)
			.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
			.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;
		let para_id = api
			.para_id(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Error getting para id"))?;
		let result: ibc_primitives::QueryPacketReceiptResponse = api
			.packet_receipt(
				hash_at,
				channel_id.as_bytes().to_vec(),
				port_id.as_bytes().to_vec(),
				seq,
			)
			.ok()
			.flatten()
			.ok_or_else(|| runtime_error_into_rpc_error("Error fetching packet receipt"))?;
		let mut keys = vec![result.trie_key];
		let child_trie_key = api
			.child_trie_key(hash_at)
			.map_err(|_| runtime_error_into_rpc_error("Failed to get child trie key"))?;
		let child_info = ChildInfo::new_default(&child_trie_key);
		let proof = self
			.client
			.read_child_proof(hash_at, &child_info, &mut keys.iter_mut().map(|nodes| &nodes[..]))
			.map_err(runtime_error_into_rpc_error)?
			.iter_nodes()
			.collect::<Vec<_>>()
			.encode();
		Ok(QueryPacketReceiptResponse {
			received: result.receipt,
			proof,
			proof_height: Some(ibc_proto::ibc::core::client::v1::Height {
				revision_number: para_id.into(),
				revision_height: result.height,
			}),
		})
	}

	fn query_denom_trace(&self, asset_id: AssetId) -> RpcResult<QueryDenomTraceResponse> {
		let api = self.client.runtime_api();
		let block_hash = self.client.info().best_hash;

		let denom_trace =
			api.denom_trace(block_hash, asset_id).ok().flatten().ok_or_else(|| {
				runtime_error_into_rpc_error(
					"[ibc_rpc]: Could not find a denom trace for asset id provided",
				)
			})?;

		let denom_str = String::from_utf8(denom_trace.denom).map_err(|_| {
			runtime_error_into_rpc_error(
				"[ibc_rpc]: Could not decode ibc denom into a valid string",
			)
		})?;
		let denom_trace = ibc::applications::transfer::PrefixedDenom::from_str(&denom_str)
			.map_err(|_| {
				runtime_error_into_rpc_error(
					"[ibc_rpc]: Could not derive a valid ibc denom from string",
				)
			})?;
		let denom_trace: ibc_proto::ibc::applications::transfer::v1::DenomTrace =
			denom_trace.try_into().map_err(|_| {
				runtime_error_into_rpc_error(
					"[ibc_rpc]: Could not derive a valid ibc denom from string",
				)
			})?;

		Ok(QueryDenomTraceResponse { denom_trace: Some(denom_trace) })
	}

	fn query_denom_traces(
		&self,
		key: Option<AssetId>,
		offset: Option<u32>,
		limit: Option<u64>,
		count_total: bool,
	) -> RpcResult<QueryDenomTracesResponse> {
		let api = self.client.runtime_api();
		let block_hash = self.client.info().best_hash;

		let at = block_hash;
		// Set default limit to 20 items
		let limit = limit.unwrap_or(20);
		let result =
			api.denom_traces(at, key, offset, limit, count_total).ok().ok_or_else(|| {
				runtime_error_into_rpc_error(
					"[ibc_rpc]: Could not find a denom trace for asset id provided",
				)
			})?;

		let denom_traces = result
			.denoms
			.into_iter()
			.map(|denom| {
				let denom_str = String::from_utf8(denom).map_err(|_| {
					runtime_error_into_rpc_error(
						"[ibc_rpc]: Could not decode ibc denom into a valid string",
					)
				})?;
				let denom_trace = ibc::applications::transfer::PrefixedDenom::from_str(&denom_str)
					.map_err(|_| {
						runtime_error_into_rpc_error(
							"[ibc_rpc]: Could not derive a valid ibc denom from string",
						)
					})?;
				let denom_trace: ibc_proto::ibc::applications::transfer::v1::DenomTrace =
					denom_trace.try_into().map_err(|_| {
						runtime_error_into_rpc_error(
							"[ibc_rpc]: Could not derive a valid ibc denom from string",
						)
					})?;
				Ok(denom_trace)
			})
			.collect::<RpcResult<Vec<_>>>()?;

		Ok(QueryDenomTracesResponse {
			denom_traces,
			pagination: result.next_key.map(|key| PageResponse {
				next_key: key.encode(),
				total: result.total.unwrap_or_default(),
			}),
		})
	}

	fn query_newly_created_client(
		&self,
		block_hash: Block::Hash,
		ext_hash: Block::Hash,
	) -> RpcResult<IdentifiedClientState> {
		let (block, event) = self.ibc_event_by_tx_id(block_hash, ext_hash)?;
		let api = self.client.runtime_api();

		match event {
			Ok(IbcEvent::CreateClient { client_id, .. }) => {
				let result: ibc_primitives::QueryClientStateResponse = api
					.client_state(block.block.header().hash(), client_id.clone())
					.ok()
					.flatten()
					.ok_or_else(|| runtime_error_into_rpc_error("client state to exist"))?;

				let client_state = AnyClientState::decode_vec(&result.client_state)
					.map_err(|_| runtime_error_into_rpc_error("client state to be valid"))?;
				Ok(IdentifiedClientState {
					client_id: String::from_utf8(client_id).map_err(|_| {
						runtime_error_into_rpc_error("client id should be valid utf8")
					})?,
					client_state: Some(client_state.into()),
				})
			},
			_ => {
				Err(runtime_error_into_rpc_error("[ibc_rpc]: Could not find client creation event"))
			},
		}
	}

	fn query_newly_created_connection(
		&self,
		block_hash: Block::Hash,
		ext_hash: Block::Hash,
	) -> RpcResult<IdentifiedConnection> {
		let (block, event) = self.ibc_event_by_tx_id(block_hash, ext_hash)?;

		match event {
			Ok(IbcEvent::OpenInitConnection { connection_id, client_id, .. }) => {
				let connection_id =
					connection_id.expect("connection id should exist after its creation");

				let height = (*block.block.header().number()).try_into().map_err(|_| {
					runtime_error_into_rpc_error("block number should be valid u64")
				})?;
				let connections: Vec<IdentifiedConnection> = self.query_connection_using_client(
					height,
					String::from_utf8(client_id).map_err(|_| {
						runtime_error_into_rpc_error("client id should be valid utf8")
					})?,
				)?;
				let connection = connections
					.into_iter()
					.find(|connection| connection.id.as_bytes() == connection_id)
					.ok_or_else(|| {
						runtime_error_into_rpc_error("connection should exist after its creation")
					})?;
				Ok(connection)
			},
			_ => {
				Err(runtime_error_into_rpc_error("[ibc_rpc]: Could not find client creation event"))
			},
		}
	}

	fn query_newly_created_channel(
		&self,
		block_hash: Block::Hash,
		ext_hash: Block::Hash,
	) -> RpcResult<IdentifiedChannel> {
		let (block, event) = self.ibc_event_by_tx_id(block_hash, ext_hash)?;

		match event {
			Ok(IbcEvent::OpenInitChannel { channel_id, port_id, connection_id, .. }) => {
				let channel_id = channel_id.expect("channel should exist after its creation");

				let height = (*block.block.header().number()).try_into().map_err(|_| {
					runtime_error_into_rpc_error("block number should be valid u64")
				})?;
				let channels: QueryChannelsResponse = self.query_connection_channels(
					height,
					String::from_utf8(connection_id).map_err(|_| {
						runtime_error_into_rpc_error("connection id should be valid utf8")
					})?,
				)?;
				let channel = channels
					.channels
					.into_iter()
					.find(|ch| {
						ch.channel_id.as_bytes() == channel_id && ch.port_id.as_bytes() == port_id
					})
					.ok_or_else(|| {
						runtime_error_into_rpc_error("connection should exist after its creation")
					})?;
				Ok(channel)
			},
			_ => {
				Err(runtime_error_into_rpc_error("[ibc_rpc]: Could not find client creation event"))
			},
		}
	}

	fn query_events(
		&self,
		block_numbers: Vec<BlockNumberOrHash<Block::Hash>>,
	) -> RpcResult<HashMap<String, Vec<RawIbcEvent>>> {
		let api = self.client.runtime_api();
		let mut events = HashMap::new();
		for block_number_or_hash in block_numbers {
			let at = match block_number_or_hash {
				BlockNumberOrHash::Hash(block_hash) => BlockId::Hash(block_hash),
				BlockNumberOrHash::Number(block_number) => BlockId::Number(block_number.into()),
			};
			let hash_at = self
				.client
				.block_hash_from_id(&at)
				.map_err(|_| runtime_error_into_rpc_error("Unknown Block"))?
				.ok_or_else(|| runtime_error_into_rpc_error("Unknown Block"))?;

			let temp = api.block_events(hash_at, None).map_err(|_| {
				runtime_error_into_rpc_error("[ibc_rpc]: failed to read block events")
			})?;
			let temp = temp
				.into_iter()
				.filter_map(|event| {
					filter_map_pallet_event::<C, Block, AssetId>(hash_at, &api, event.ok()?)
				})
				.collect();
			events.insert(block_number_or_hash.to_string(), temp);
		}
		Ok(events)
	}
}

impl<C, Block, AssetId> IbcRpcHandler<C, Block, AssetId>
where
	Block: BlockT,
	C: 'static
		+ BlockBackend<Block>
		+ HeaderBackend<Block>
		+ ProofProvider<Block>
		+ ProvideRuntimeApi<Block>
		+ Send
		+ Sync,
	C::Api: IbcRuntimeApi<Block, AssetId>,
	AssetId: parity_scale_codec::Codec + Copy,
{
	fn ibc_event_by_tx_id(
		&self,
		block_hash: <Block as BlockT>::Hash,
		ext_hash: <Block as BlockT>::Hash,
	) -> RpcResult<(SignedBlock<Block>, core::result::Result<IbcEvent, IbcError>)> {
		let api = self.client.runtime_api();
		let block = self.client.block(block_hash).ok().flatten().ok_or_else(|| {
			runtime_error_into_rpc_error("[ibc_rpc]: failed to find block with provided hash")
		})?;
		let extrinsics = block.block.extrinsics();
		let (ext_index, ..) = extrinsics
			.iter()
			.enumerate()
			.find(|(_, ext)| ext_hash.as_ref() == blake2_256(ext.encode().as_slice()).as_ref())
			.ok_or_else(|| {
				runtime_error_into_rpc_error(
					"[ibc_rpc]: failed to find extrinsic with provided hash",
				)
			})?;

		let events = api
			.block_events(block.block.header().hash(), Some(ext_index as _))
			.map_err(|_| runtime_error_into_rpc_error("[ibc_rpc]: failed to read block events"))?;

		// There should be only one ibc event in this list in this case
		let event = events
			.get(0)
			.ok_or_else(|| runtime_error_into_rpc_error("[ibc_rpc]: Could not find any ibc event"))?
			.clone();
		Ok((block, event))
	}
}
