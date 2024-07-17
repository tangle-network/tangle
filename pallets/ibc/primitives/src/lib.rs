#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::weights::Weight;
use ibc::{
	applications::transfer::{error::Error as Ics20Error, PrefixedCoin, VERSION},
	core::{
		ics04_channel::{
			channel::{ChannelEnd, Order},
			msgs::acknowledgement::Acknowledgement,
			packet::{Packet, Sequence},
		},
		ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
	},
	signer::Signer,
	timestamp::Timestamp,
	Height,
};
use parity_scale_codec::{Decode, Encode};
use scale_info::{prelude::format, TypeInfo};
use sp_runtime::traits::IdentifyAccount;
use sp_std::{prelude::*, str::FromStr};

pub mod runtime_interface;

/// Packet timeout, could be an offset, or absolute value.
#[derive(Debug, PartialEq, Eq, scale_info::TypeInfo, Encode, Decode, Clone)]
pub enum Timeout {
	Offset {
		/// Timestamp at which this packet should timeout in counterparty in seconds
		/// relative to the latest time stamp
		timestamp: Option<u64>,
		/// Block height at which this packet should timeout on counterparty
		/// relative to the latest height
		height: Option<u64>,
	},
	/// Absolute value
	Absolute {
		/// Timestamp at which this packet should timeout on the counterparty in nanoseconds
		timestamp: Option<u64>,
		/// Block height at which this packet should timeout on the counterparty
		height: Option<u64>,
	},
}

pub enum HandlerMessage<AccountId> {
	OpenChannel {
		port_id: PortId,
		channel_end: ChannelEnd,
	},
	CloseChannel {
		channel_id: ChannelId,
		port_id: PortId,
	},
	Transfer {
		channel_id: ChannelId,
		coin: PrefixedCoin,
		timeout: Timeout,
		from: AccountId,
		to: Signer,
		memo: String,
	},
	SendPacket {
		/// packet data
		data: Vec<u8>,
		/// Packet timeout
		timeout: Timeout,
		/// port id as utf8 string bytes
		port_id: PortId,
		/// channel id as utf8 string bytes
		channel_id: ChannelId,
	},
	WriteAck {
		/// Raw acknowledgement bytes
		ack: Vec<u8>,
		/// Packet
		packet: Packet,
	},
}

#[derive(
	parity_scale_codec::Encode,
	Default,
	Debug,
	parity_scale_codec::Decode,
	Clone,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	scale_info::TypeInfo,
)]
pub struct PacketInfo {
	pub height: Option<u64>,
	pub sequence: u64,
	pub source_port: Vec<u8>,
	pub source_channel: Vec<u8>,
	pub destination_port: Vec<u8>,
	pub destination_channel: Vec<u8>,
	pub channel_order: u8,
	pub data: Vec<u8>,
	pub timeout_height: (u64, u64),
	pub timeout_timestamp: u64,
	pub ack: Option<Vec<u8>>,
}

impl From<PacketInfo> for Packet {
	fn from(packet: PacketInfo) -> Self {
		Self {
			sequence: Sequence::from(packet.sequence),
			source_port: PortId::from_str(
				&String::from_utf8(packet.source_port).unwrap_or_default(),
			)
			.unwrap_or_default(),
			source_channel: ChannelId::from_str(
				&String::from_utf8(packet.source_channel).unwrap_or_default(),
			)
			.unwrap_or_default(),
			destination_port: PortId::from_str(
				&String::from_utf8(packet.destination_port).unwrap_or_default(),
			)
			.unwrap_or_default(),
			destination_channel: ChannelId::from_str(
				&String::from_utf8(packet.destination_channel).unwrap_or_default(),
			)
			.unwrap_or_default(),
			data: packet.data,
			timeout_height: ibc::Height::new(packet.timeout_height.0, packet.timeout_height.1),
			timeout_timestamp: Timestamp::from_nanoseconds(packet.timeout_timestamp)
				.unwrap_or_default(),
		}
	}
}

impl From<Packet> for PacketInfo {
	fn from(packet: Packet) -> Self {
		Self {
			sequence: packet.sequence.into(),
			source_port: packet.source_port.to_string().as_bytes().to_vec(),
			source_channel: packet.source_channel.to_string().as_bytes().to_vec(),
			destination_port: packet.destination_port.to_string().as_bytes().to_vec(),
			destination_channel: packet.destination_channel.to_string().as_bytes().to_vec(),
			data: packet.data,
			timeout_height: (
				packet.timeout_height.revision_number,
				packet.timeout_height.revision_height,
			),
			timeout_timestamp: packet.timeout_timestamp.nanoseconds(),
			height: None,
			ack: None,
			channel_order: Default::default(),
		}
	}
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct IdentifiedChannel {
	pub channel_id: Vec<u8>,
	pub port_id: Vec<u8>,
	/// Protobuf encoded `ibc::core::ics04_channel::connection::ChannelEnd`
	pub channel_end: Vec<u8>,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct IdentifiedClientState {
	pub client_id: Vec<u8>,
	/// Protobuf encoded `AnyClientState`
	pub client_state: Vec<u8>,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct IdentifiedConnection {
	pub connection_id: Vec<u8>,
	/// Protobuf encoded `ibc::core::ics03_connection::connection::ConnectionEnd`
	pub connection_end: Vec<u8>,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct QueryClientStateResponse {
	/// Protobuf encoded `AnyClientState`
	pub client_state: Vec<u8>,
	pub height: u64,
	pub trie_key: Vec<u8>,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct QueryClientStatesResponse {
	pub client_states: Vec<Vec<u8>>,
	pub height: u64,
	pub trie_key: Vec<u8>,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct QueryConsensusStateResponse {
	pub consensus_state: Vec<u8>,
	pub height: u64,
	pub trie_key: Vec<u8>,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct QueryConnectionResponse {
	/// Protobuf encoded `ibc::core::ics03_connection::connection::ConnectionEnd`
	pub connection: Vec<u8>,
	pub height: u64,
	pub trie_key: Vec<u8>,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct QueryChannelResponse {
	/// Protobuf encoded `ibc::core::ics04_channel::connection::ChannelEnd`
	pub channel: Vec<u8>,
	pub height: u64,
	pub trie_key: Vec<u8>,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct QueryChannelsResponse {
	pub channels: Vec<IdentifiedChannel>,
	pub height: u64,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct QueryConnectionsResponse {
	pub connections: Vec<IdentifiedConnection>,
	pub height: u64,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct QueryNextSequenceReceiveResponse {
	pub sequence: u64,
	pub height: u64,
	pub trie_key: Vec<u8>,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct QueryPacketCommitmentResponse {
	pub commitment: Vec<u8>,
	pub height: u64,
	pub trie_key: Vec<u8>,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct PacketState {
	pub port_id: Vec<u8>,
	pub channel_id: Vec<u8>,
	pub sequence: u64,
	pub data: Vec<u8>,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct QueryPacketCommitmentsResponse {
	pub commitments: Vec<PacketState>,
	pub height: u64,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct QueryPacketAcknowledgementResponse {
	pub ack: Vec<u8>,
	pub height: u64,
	pub trie_key: Vec<u8>,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct QueryPacketAcknowledgementsResponse {
	pub acks: Vec<PacketState>,
	pub height: u64,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct QueryPacketReceiptResponse {
	pub receipt: bool,
	pub height: u64,
	pub trie_key: Vec<u8>,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct QueryDenomTraceResponse {
	pub denom: Vec<u8>,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct QueryDenomTracesResponse {
	pub denoms: Vec<Vec<u8>>,
	pub next_key: Option<Vec<u8>>,
	pub total: Option<u64>,
}

#[derive(
	Clone,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	PartialEq,
	Eq,
	Ord,
	PartialOrd,
	TypeInfo,
)]
pub struct ConnectionHandshake {
	pub client_state: Vec<u8>,
	pub trie_keys: Vec<Vec<u8>>,
	pub height: u64,
}

#[derive(core::fmt::Debug, Clone, PartialEq, Eq)]
/// Error definition for module
pub enum Error {
	/// Failed to register a new packet
	SendPacketError { msg: Option<String> },
	/// An error involving the connection id
	ConnectionIdError { msg: Option<String> },
	/// An error involving the client id
	ClientIdError { msg: Option<String> },
	/// An error involving channel or port
	ChannelOrPortError { msg: Option<String> },
	/// An error involving Client state
	ClientStateError { msg: Option<String> },
	/// An Error Involving the Timestamp and height
	TimestampOrHeightNotFound { msg: Option<String> },
	/// Failed to register a token transfer packet
	SendTransferError { msg: Option<String> },
	/// Ics20 receive packet processing error
	ReceivePacketError { msg: Option<String> },
	/// Write acknowledgement error
	WriteAcknowledgementError { msg: Option<String> },
	/// Ics20 packet acknowledgement processing error
	AcknowledgementError { msg: Option<String> },
	/// Ics20 packet timeout processing error
	TimeoutError { msg: Option<String> },
	/// Failed to bind port
	BindPortError { msg: Option<String> },
	/// Failed to initialize a new channel
	ChannelInitError { msg: Option<String> },
	/// Failed to close a channel
	ChannelCloseError { msg: Option<String> },
	/// Failed to decode a value
	DecodingError { msg: Option<String> },
	/// Failed to decode commitment prefix
	ErrorDecodingPrefix,
	/// Some other error
	Other { msg: Option<String> },
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, Debug, TypeInfo)]
/// Captures all parameters needed to initialize a channel
pub struct OpenChannelParams {
	/// channel order
	pub order: u8,
	/// connection id as utf8 string bytes
	pub connection_id: Vec<u8>,
	/// counterparty port id as utf8 string bytes
	pub counterparty_port_id: Vec<u8>,
	/// version as utf8 string bytes
	pub version: Vec<u8>,
}

impl TryFrom<&OpenChannelParams> for Order {
	type Error = Error;

	fn try_from(value: &OpenChannelParams) -> Result<Self, Self::Error> {
		match value.order {
			1 => Ok(Order::Unordered),
			2 => Ok(Order::Ordered),
			_ => Err(Error::Other { msg: None }),
		}
	}
}

/// Captures the functions modules can use to interact with the ibc pallet
/// Currently allows modules to register packets and create channels
pub trait IbcHandler<AccountId> {
	/// Get the latest height and latest timestamp for the client paired to the channel and port
	/// combination
	fn latest_height_and_timestamp(
		port_id: &PortId,
		channel_id: &ChannelId,
	) -> Result<(Height, Timestamp), Error>;
	/// Handle a message
	fn handle_message(msg: HandlerMessage<AccountId>) -> Result<(), Error>;
	/// testing related methods
	#[cfg(feature = "runtime-benchmarks")]
	fn create_client() -> Result<ClientId, Error>;
	#[cfg(feature = "runtime-benchmarks")]
	fn create_connection(client_id: ClientId, connection_id: ConnectionId) -> Result<(), Error>;
}

/// Callback Weight
/// This trait must be implemented by module callback handlers to be able to estimate the weight
/// of the callback function.
pub trait CallbackWeight {
	/// Returns the callback weight for the channel open init ibc message
	fn on_chan_open_init(&self) -> Weight;

	/// Returns the callback weight for the channel open try ibc message
	fn on_chan_open_try(&self) -> Weight;

	/// Returns the callback weight for the channel open acknowledgement ibc message
	fn on_chan_open_ack(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight;

	/// Returns the callback weight for the channel open confirm ibc message
	fn on_chan_open_confirm(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight;

	/// Returns the callback weight for the channel close init ibc message
	fn on_chan_close_init(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight;

	/// Returns the callback weight for the channel close confirm ibc message
	fn on_chan_close_confirm(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight;

	/// Returns the callback weight for the receive packet ibc message
	fn on_recv_packet(&self, _packet: &Packet) -> Weight;

	/// Returns the callback weight for the packet acknowledgement ibc message
	fn on_acknowledgement_packet(
		&self,
		_packet: &Packet,
		_acknowledgement: &Acknowledgement,
	) -> Weight;

	/// Returns the callback weight for the packet timeout ibc message
	fn on_timeout_packet(&self, packet: &Packet) -> Weight;
}

impl CallbackWeight for () {
	fn on_chan_open_init(&self) -> Weight {
		Weight::MAX
	}

	fn on_chan_open_try(&self) -> Weight {
		Weight::MAX
	}

	fn on_chan_open_ack(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight {
		Weight::MAX
	}

	fn on_chan_open_confirm(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight {
		Weight::MAX
	}

	fn on_chan_close_init(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight {
		Weight::MAX
	}

	fn on_chan_close_confirm(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight {
		Weight::MAX
	}

	fn on_recv_packet(&self, _packet: &Packet) -> Weight {
		Weight::MAX
	}

	fn on_acknowledgement_packet(
		&self,
		_packet: &Packet,
		_acknowledgement: &Acknowledgement,
	) -> Weight {
		Weight::MAX
	}

	fn on_timeout_packet(&self, _packet: &Packet) -> Weight {
		Weight::MAX
	}
}

/// Get port_id from raw bytes
pub fn port_id_from_bytes(port: Vec<u8>) -> Result<PortId, Error> {
	PortId::from_str(&String::from_utf8(port).map_err(|_| Error::DecodingError { msg: None })?)
		.map_err(|_| Error::DecodingError { msg: None })
}

/// Get channel_id from raw bytes
pub fn channel_id_from_bytes(channel: Vec<u8>) -> Result<ChannelId, Error> {
	ChannelId::from_str(
		&String::from_utf8(channel).map_err(|_| Error::DecodingError { msg: None })?,
	)
	.map_err(|_| Error::DecodingError { msg: None })
}

/// Get connection_id from raw bytes
pub fn connection_id_from_bytes(connection: Vec<u8>) -> Result<ConnectionId, Error> {
	ConnectionId::from_str(
		&String::from_utf8(connection).map_err(|_| Error::DecodingError { msg: None })?,
	)
	.map_err(|_| Error::DecodingError { msg: None })
}

/// Get client_id from raw bytes
pub fn client_id_from_bytes(client_id: Vec<u8>) -> Result<ClientId, Error> {
	ClientId::from_str(
		&String::from_utf8(client_id).map_err(|_| Error::DecodingError { msg: None })?,
	)
	.map_err(|_| Error::DecodingError { msg: None })
}

/// Get trie key by applying the commitment prefix to the path
pub fn apply_prefix(prefix: &[u8], path: Vec<String>) -> Vec<u8> {
	let mut key_path = prefix.to_vec();
	let path = path.iter().flat_map(|val| val.as_bytes()).collect::<Vec<_>>();
	key_path.extend(path);
	key_path
}

pub fn get_channel_escrow_address(
	port_id: &PortId,
	channel_id: ChannelId,
) -> Result<Signer, Ics20Error> {
	let contents = format!("{port_id}/{channel_id}");
	let mut data = VERSION.as_bytes().to_vec();
	data.extend_from_slice(&[0]);
	data.extend_from_slice(contents.as_bytes());

	let hash = sp_io::hashing::sha2_256(&data).to_vec();
	let mut hex_string = hex::encode_upper(hash);
	hex_string.insert_str(0, "0x");
	hex_string.parse::<Signer>().map_err(Ics20Error::signer)
}

// This is needed because Ics20 traits require an implementation of TryFrom<Signer> for AccountId
// associated type
#[derive(Clone)]
pub struct IbcAccount<AccountId>(pub AccountId);

impl<AccountId> IdentifyAccount for IbcAccount<AccountId> {
	type AccountId = AccountId;
	fn into_account(self) -> Self::AccountId {
		self.0
	}
}

impl<AccountId> TryFrom<Signer> for IbcAccount<AccountId>
where
	AccountId: From<[u8; 32]>,
{
	type Error = &'static str;

	/// Convert a signer to an IBC account.
	fn try_from(signer: ibc::signer::Signer) -> Result<Self, Self::Error> {
		let acc_str: &str = signer.as_ref();
		if acc_str.starts_with("0x") {
			match acc_str.strip_prefix("0x") {
				Some(hex_string) => TryInto::<[u8; 32]>::try_into(
					hex::decode(hex_string).map_err(|_| "Error decoding invalid hex string")?,
				)
				.map_err(|_| "Invalid account id hex string")
				.map(|acc| Self(acc.into())),
				_ => Err("Signer does not hold a valid hex string"),
			}
		}
		// Do SS58 decoding instead
		else {
			let bytes = runtime_interface::ss58_to_account_id_32(acc_str)
				.map_err(|_| "Invalid SS58 address")?;
			Ok(Self(bytes.into()))
		}
	}
}
