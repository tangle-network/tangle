use super::*;
use crate::errors::IbcError;
use ibc::{
	core::{
		ics02_client::{
			events as ClientEvents,
			events::{CodeId, NewBlock},
		},
		ics03_connection::events as ConnectionEvents,
		ics04_channel::{events as ChannelEvents, packet::Packet},
		ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
		ics26_routing::{context::ModuleId, error::Error as RoutingError},
	},
	events::{IbcEvent as RawIbcEvent, ModuleEvent},
	timestamp::Timestamp,
	Height,
};

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, scale_info::TypeInfo)]
/// IBC events
// Using Vec<u8> instead of String because Encode and Decode are not implemented for Strings in
// no-std environment.
pub enum IbcEvent {
	/// New block
	NewBlock { revision_height: u64, revision_number: u64 },
	/// Client created
	CreateClient {
		client_id: Vec<u8>,
		client_type: Vec<u8>,
		revision_height: u64,
		revision_number: u64,
		consensus_height: u64,
		consensus_revision_number: u64,
	},
	/// Client updated
	UpdateClient {
		client_id: Vec<u8>,
		client_type: Vec<u8>,
		revision_height: u64,
		revision_number: u64,
		consensus_height: u64,
		consensus_revision_number: u64,
	},
	/// Client upgraded
	UpgradeClient {
		client_id: Vec<u8>,
		client_type: Vec<u8>,
		revision_height: u64,
		revision_number: u64,
		consensus_height: u64,
		consensus_revision_number: u64,
	},
	/// Client misbehaviour
	ClientMisbehaviour {
		client_id: Vec<u8>,
		client_type: Vec<u8>,
		revision_height: u64,
		revision_number: u64,
		consensus_height: u64,
		consensus_revision_number: u64,
	},
	/// Connection open init
	OpenInitConnection {
		revision_height: u64,
		revision_number: u64,
		connection_id: Option<Vec<u8>>,
		client_id: Vec<u8>,
		counterparty_connection_id: Option<Vec<u8>>,
		counterparty_client_id: Vec<u8>,
	},
	/// Connection open confirm
	OpenConfirmConnection {
		revision_height: u64,
		revision_number: u64,
		connection_id: Option<Vec<u8>>,
		client_id: Vec<u8>,
		counterparty_connection_id: Option<Vec<u8>>,
		counterparty_client_id: Vec<u8>,
	},
	/// Connection try open
	OpenTryConnection {
		revision_height: u64,
		revision_number: u64,
		connection_id: Option<Vec<u8>>,
		client_id: Vec<u8>,
		counterparty_connection_id: Option<Vec<u8>>,
		counterparty_client_id: Vec<u8>,
	},
	/// Connection open acknowledge
	OpenAckConnection {
		revision_height: u64,
		revision_number: u64,
		connection_id: Option<Vec<u8>>,
		client_id: Vec<u8>,
		counterparty_connection_id: Option<Vec<u8>>,
		counterparty_client_id: Vec<u8>,
	},
	/// Channel open init
	OpenInitChannel {
		revision_height: u64,
		revision_number: u64,
		port_id: Vec<u8>,
		channel_id: Option<Vec<u8>>,
		connection_id: Vec<u8>,
		counterparty_port_id: Vec<u8>,
		counterparty_channel_id: Option<Vec<u8>>,
	},
	/// Channel open confirm
	OpenConfirmChannel {
		revision_height: u64,
		revision_number: u64,
		port_id: Vec<u8>,
		channel_id: Option<Vec<u8>>,
		connection_id: Vec<u8>,
		counterparty_port_id: Vec<u8>,
		counterparty_channel_id: Option<Vec<u8>>,
	},
	/// Channel try open
	OpenTryChannel {
		revision_height: u64,
		revision_number: u64,
		port_id: Vec<u8>,
		channel_id: Option<Vec<u8>>,
		connection_id: Vec<u8>,
		counterparty_port_id: Vec<u8>,
		counterparty_channel_id: Option<Vec<u8>>,
	},
	/// Open ack channel
	OpenAckChannel {
		revision_height: u64,
		revision_number: u64,
		port_id: Vec<u8>,
		channel_id: Option<Vec<u8>>,
		connection_id: Vec<u8>,
		counterparty_port_id: Vec<u8>,
		counterparty_channel_id: Option<Vec<u8>>,
	},
	/// Channel close init
	CloseInitChannel {
		revision_height: u64,
		revision_number: u64,
		port_id: Vec<u8>,
		channel_id: Vec<u8>,
		connection_id: Vec<u8>,
		counterparty_port_id: Vec<u8>,
		counterparty_channel_id: Option<Vec<u8>>,
	},
	/// Channel close confirm
	CloseConfirmChannel {
		revision_height: u64,
		revision_number: u64,
		channel_id: Option<Vec<u8>>,
		port_id: Vec<u8>,
		connection_id: Vec<u8>,
		counterparty_port_id: Vec<u8>,
		counterparty_channel_id: Option<Vec<u8>>,
	},
	/// Receive packet
	ReceivePacket {
		revision_height: u64,
		revision_number: u64,
		port_id: Vec<u8>,
		channel_id: Vec<u8>,
		dest_port: Vec<u8>,
		dest_channel: Vec<u8>,
		sequence: u64,
	},
	/// Send packet
	SendPacket {
		revision_height: u64,
		revision_number: u64,
		port_id: Vec<u8>,
		channel_id: Vec<u8>,
		dest_port: Vec<u8>,
		dest_channel: Vec<u8>,
		sequence: u64,
	},
	/// Acknowledgement packet
	AcknowledgePacket {
		revision_height: u64,
		revision_number: u64,
		port_id: Vec<u8>,
		channel_id: Vec<u8>,
		sequence: u64,
	},
	/// WriteAcknowledgement
	WriteAcknowledgement {
		revision_height: u64,
		revision_number: u64,
		port_id: Vec<u8>,
		channel_id: Vec<u8>,
		dest_port: Vec<u8>,
		dest_channel: Vec<u8>,
		sequence: u64,
	},
	/// Timeout packet
	TimeoutPacket {
		revision_height: u64,
		revision_number: u64,
		port_id: Vec<u8>,
		channel_id: Vec<u8>,
		sequence: u64,
	},
	/// TimeoutOnClose packet
	TimeoutOnClosePacket {
		revision_height: u64,
		revision_number: u64,
		port_id: Vec<u8>,
		channel_id: Vec<u8>,
		sequence: u64,
	},
	/// Empty
	Empty,
	/// Chain Error
	ChainError,
	/// App module
	AppModule { kind: Vec<u8>, module_id: Vec<u8> },
	/// Push WASM Code
	PushWasmCode { wasm_code_id: CodeId },
}

impl From<RawIbcEvent> for IbcEvent {
	fn from(event: RawIbcEvent) -> Self {
		match event {
			RawIbcEvent::NewBlock(ev) => IbcEvent::NewBlock {
				revision_height: ev.height.revision_height,
				revision_number: ev.height.revision_number,
			},
			RawIbcEvent::CreateClient(ev) => IbcEvent::CreateClient {
				client_id: ev.0.client_id.as_bytes().to_vec(),
				client_type: ev.0.client_type.as_bytes().to_vec(),
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				consensus_height: ev.0.consensus_height.revision_height,
				consensus_revision_number: ev.0.consensus_height.revision_number,
			},
			RawIbcEvent::UpdateClient(ev) => IbcEvent::UpdateClient {
				client_id: ev.client_id().as_bytes().to_vec(),
				client_type: ev.client_type().as_bytes().to_vec(),
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				consensus_height: ev.consensus_height().revision_height,
				consensus_revision_number: ev.consensus_height().revision_number,
			},
			RawIbcEvent::UpgradeClient(ev) => IbcEvent::UpgradeClient {
				client_id: ev.client_id().as_bytes().to_vec(),
				client_type: ev.0.client_type.as_bytes().to_vec(),
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				consensus_height: ev.0.consensus_height.revision_height,
				consensus_revision_number: ev.0.consensus_height.revision_number,
			},
			RawIbcEvent::ClientMisbehaviour(ev) => IbcEvent::ClientMisbehaviour {
				client_id: ev.client_id().as_bytes().to_vec(),
				client_type: ev.0.client_type.as_bytes().to_vec(),
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				consensus_height: ev.0.consensus_height.revision_height,
				consensus_revision_number: ev.0.consensus_height.revision_number,
			},
			RawIbcEvent::OpenInitConnection(ev) => IbcEvent::OpenInitConnection {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				connection_id: ev.connection_id().map(|val| val.as_bytes().to_vec()),
				client_id: ev.attributes().client_id.as_bytes().to_vec(),
				counterparty_connection_id: ev
					.attributes()
					.counterparty_connection_id
					.as_ref()
					.map(|val| val.as_bytes().to_vec()),
				counterparty_client_id: ev.attributes().counterparty_client_id.as_bytes().to_vec(),
			},
			RawIbcEvent::OpenTryConnection(ev) => IbcEvent::OpenTryConnection {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				connection_id: ev.connection_id().map(|val| val.as_bytes().to_vec()),
				client_id: ev.attributes().client_id.as_bytes().to_vec(),
				counterparty_connection_id: ev
					.attributes()
					.counterparty_connection_id
					.as_ref()
					.map(|val| val.as_bytes().to_vec()),
				counterparty_client_id: ev.attributes().counterparty_client_id.as_bytes().to_vec(),
			},
			RawIbcEvent::OpenAckConnection(ev) => IbcEvent::OpenAckConnection {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				connection_id: ev.connection_id().map(|val| val.as_bytes().to_vec()),
				client_id: ev.attributes().client_id.as_bytes().to_vec(),
				counterparty_connection_id: ev
					.attributes()
					.counterparty_connection_id
					.as_ref()
					.map(|val| val.as_bytes().to_vec()),
				counterparty_client_id: ev.attributes().counterparty_client_id.as_bytes().to_vec(),
			},
			RawIbcEvent::OpenConfirmConnection(ev) => IbcEvent::OpenConfirmConnection {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				connection_id: ev.connection_id().map(|val| val.as_bytes().to_vec()),
				client_id: ev.attributes().client_id.as_bytes().to_vec(),
				counterparty_connection_id: ev
					.attributes()
					.counterparty_connection_id
					.as_ref()
					.map(|val| val.as_bytes().to_vec()),
				counterparty_client_id: ev.attributes().counterparty_client_id.as_bytes().to_vec(),
			},
			RawIbcEvent::OpenInitChannel(ev) => IbcEvent::OpenInitChannel {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				channel_id: ev
					.channel_id()
					.map(|channel_id| channel_id.to_string().as_bytes().to_vec()),
				port_id: ev.port_id().as_bytes().to_vec(),
				connection_id: ev.connection_id.as_bytes().to_vec(),
				counterparty_port_id: ev.counterparty_port_id.as_bytes().to_vec(),
				counterparty_channel_id: ev
					.counterparty_channel_id
					.map(|val| val.to_string().as_bytes().to_vec()),
			},
			RawIbcEvent::OpenTryChannel(ev) => IbcEvent::OpenTryChannel {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				channel_id: ev
					.channel_id()
					.map(|channel_id| channel_id.to_string().as_bytes().to_vec()),
				port_id: ev.port_id().as_bytes().to_vec(),
				connection_id: ev.connection_id.as_bytes().to_vec(),
				counterparty_port_id: ev.counterparty_port_id.as_bytes().to_vec(),
				counterparty_channel_id: ev
					.counterparty_channel_id
					.map(|val| val.to_string().as_bytes().to_vec()),
			},
			RawIbcEvent::OpenAckChannel(ev) => IbcEvent::OpenAckChannel {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				channel_id: ev
					.channel_id()
					.map(|channel_id| channel_id.to_string().as_bytes().to_vec()),
				port_id: ev.port_id().as_bytes().to_vec(),
				connection_id: ev.connection_id.as_bytes().to_vec(),
				counterparty_port_id: ev.counterparty_port_id.as_bytes().to_vec(),
				counterparty_channel_id: ev
					.counterparty_channel_id
					.map(|val| val.to_string().as_bytes().to_vec()),
			},
			RawIbcEvent::OpenConfirmChannel(ev) => IbcEvent::OpenConfirmChannel {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				channel_id: ev
					.channel_id()
					.map(|channel_id| channel_id.to_string().as_bytes().to_vec()),
				port_id: ev.port_id().as_bytes().to_vec(),
				connection_id: ev.connection_id.as_bytes().to_vec(),
				counterparty_port_id: ev.counterparty_port_id.as_bytes().to_vec(),
				counterparty_channel_id: ev
					.counterparty_channel_id
					.map(|val| val.to_string().as_bytes().to_vec()),
			},
			RawIbcEvent::CloseInitChannel(ev) => IbcEvent::CloseInitChannel {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				channel_id: ev.channel_id().to_string().as_bytes().to_vec(),
				port_id: ev.port_id().as_bytes().to_vec(),
				connection_id: ev.connection_id.as_bytes().to_vec(),
				counterparty_port_id: ev.counterparty_port_id.as_bytes().to_vec(),
				counterparty_channel_id: ev
					.counterparty_channel_id
					.map(|val| val.to_string().as_bytes().to_vec()),
			},
			RawIbcEvent::CloseConfirmChannel(ev) => IbcEvent::CloseConfirmChannel {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				port_id: ev.port_id.as_bytes().to_vec(),
				channel_id: ev
					.channel_id()
					.map(|channel_id| channel_id.to_string().as_bytes().to_vec()),
				connection_id: ev.connection_id.as_bytes().to_vec(),
				counterparty_port_id: ev.counterparty_port_id.as_bytes().to_vec(),
				counterparty_channel_id: ev
					.counterparty_channel_id
					.map(|val| val.to_string().as_bytes().to_vec()),
			},
			RawIbcEvent::SendPacket(ev) => IbcEvent::SendPacket {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				channel_id: ev.src_channel_id().to_string().as_bytes().to_vec(),
				port_id: ev.src_port_id().as_bytes().to_vec(),
				dest_port: ev.dst_port_id().as_bytes().to_vec(),
				dest_channel: ev.dst_channel_id().to_string().as_bytes().to_vec(),
				sequence: ev.packet.sequence.into(),
			},
			RawIbcEvent::ReceivePacket(ev) => IbcEvent::ReceivePacket {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				channel_id: ev.src_channel_id().to_string().as_bytes().to_vec(),
				port_id: ev.src_port_id().as_bytes().to_vec(),
				dest_port: ev.dst_port_id().as_bytes().to_vec(),
				dest_channel: ev.dst_channel_id().to_string().as_bytes().to_vec(),
				sequence: ev.packet.sequence.into(),
			},
			RawIbcEvent::WriteAcknowledgement(ev) => IbcEvent::WriteAcknowledgement {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				channel_id: ev.src_channel_id().to_string().as_bytes().to_vec(),
				port_id: ev.src_port_id().as_bytes().to_vec(),
				dest_port: ev.dst_port_id().as_bytes().to_vec(),
				dest_channel: ev.dst_channel_id().to_string().as_bytes().to_vec(),
				sequence: ev.packet.sequence.into(),
			},
			RawIbcEvent::AcknowledgePacket(ev) => IbcEvent::AcknowledgePacket {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				channel_id: ev.src_channel_id().to_string().as_bytes().to_vec(),
				port_id: ev.src_port_id().as_bytes().to_vec(),
				sequence: ev.packet.sequence.into(),
			},
			RawIbcEvent::TimeoutPacket(ev) => IbcEvent::TimeoutPacket {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				channel_id: ev.src_channel_id().to_string().as_bytes().to_vec(),
				port_id: ev.src_port_id().as_bytes().to_vec(),
				sequence: ev.packet.sequence.into(),
			},
			RawIbcEvent::TimeoutOnClosePacket(ev) => IbcEvent::TimeoutOnClosePacket {
				revision_height: ev.height().revision_height,
				revision_number: ev.height().revision_number,
				channel_id: ev.src_channel_id().to_string().as_bytes().to_vec(),
				port_id: ev.src_port_id().as_bytes().to_vec(),
				sequence: ev.packet.sequence.into(),
			},
			RawIbcEvent::Empty(_) => IbcEvent::Empty,
			RawIbcEvent::ChainError(_) => IbcEvent::ChainError,
			RawIbcEvent::AppModule(ev) => IbcEvent::AppModule {
				kind: ev.kind.as_bytes().to_vec(),
				module_id: ev.module_name.to_string().as_bytes().to_vec(),
			},
			RawIbcEvent::PushWasmCode(ev) => {
				let wasm_code_id = ev.0;
				IbcEvent::PushWasmCode { wasm_code_id }
			},
		}
	}
}

impl<T: Config> From<Vec<RawIbcEvent>> for Event<T> {
	fn from(events: Vec<RawIbcEvent>) -> Self {
		let events: Vec<Result<IbcEvent, _>> = events.into_iter().map(|ev| Ok(ev.into())).collect();
		Event::Events { events }
	}
}

impl<T: Config> From<Vec<Result<RawIbcEvent, RoutingError>>> for Event<T> {
	fn from(events: Vec<Result<RawIbcEvent, RoutingError>>) -> Self {
		let events: Vec<Result<IbcEvent, IbcError>> = events
			.into_iter()
			.map(|result| result.map(|ev| ev.into()).map_err(|err| err.into()))
			.collect();
		Event::Events { events }
	}
}

const ERROR_STR: &str = "Error converting ibc event";
impl TryFrom<IbcEvent> for RawIbcEvent {
	type Error = &'static str;
	fn try_from(ev: IbcEvent) -> Result<Self, Self::Error> {
		match ev {
			IbcEvent::NewBlock { revision_height, revision_number } => {
				Ok(RawIbcEvent::NewBlock(NewBlock {
					height: Height::new(revision_number, revision_height),
				}))
			},
			IbcEvent::CreateClient {
				client_id,
				client_type,
				revision_height,
				revision_number,
				consensus_height,
				consensus_revision_number,
			} => Ok(RawIbcEvent::CreateClient(ClientEvents::CreateClient(
				ClientEvents::Attributes {
					height: Height::new(revision_number, revision_height),
					client_id: ClientId::from_str(
						&String::from_utf8(client_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					client_type: String::from_utf8(client_type).map_err(|_| ERROR_STR)?,
					consensus_height: Height::new(consensus_revision_number, consensus_height),
				},
			))),
			IbcEvent::UpdateClient {
				client_id,
				client_type,
				revision_height,
				revision_number,
				consensus_height,
				consensus_revision_number,
			} => Ok(RawIbcEvent::UpdateClient(ClientEvents::UpdateClient {
				common: ClientEvents::Attributes {
					height: Height::new(revision_number, revision_height),
					client_id: ClientId::from_str(
						&String::from_utf8(client_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					client_type: String::from_utf8(client_type).map_err(|_| ERROR_STR)?,
					consensus_height: Height::new(consensus_revision_number, consensus_height),
				},
				header: None,
			})),
			IbcEvent::UpgradeClient {
				client_id,
				client_type,
				revision_height,
				revision_number,
				consensus_height,
				consensus_revision_number,
			} => Ok(RawIbcEvent::UpgradeClient(ClientEvents::UpgradeClient(
				ClientEvents::Attributes {
					height: Height::new(revision_number, revision_height),
					client_id: ClientId::from_str(
						&String::from_utf8(client_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					client_type: String::from_utf8(client_type).map_err(|_| ERROR_STR)?,
					consensus_height: Height::new(consensus_revision_number, consensus_height),
				},
			))),
			IbcEvent::ClientMisbehaviour {
				client_id,
				client_type,
				revision_height,
				revision_number,
				consensus_height,
				consensus_revision_number,
			} => Ok(RawIbcEvent::ClientMisbehaviour(ClientEvents::ClientMisbehaviour(
				ClientEvents::Attributes {
					height: Height::new(revision_number, revision_height),
					client_id: ClientId::from_str(
						&String::from_utf8(client_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					client_type: String::from_utf8(client_type).map_err(|_| ERROR_STR)?,
					consensus_height: Height::new(consensus_revision_number, consensus_height),
				},
			))),
			IbcEvent::PushWasmCode { wasm_code_id } => {
				Ok(RawIbcEvent::PushWasmCode(ClientEvents::PushWasmCode(wasm_code_id)))
			},
			IbcEvent::OpenInitConnection {
				revision_height,
				revision_number,
				connection_id,
				client_id,
				counterparty_connection_id,
				counterparty_client_id,
			} => Ok(RawIbcEvent::OpenInitConnection(
				ConnectionEvents::Attributes {
					height: Height::new(revision_number, revision_height),
					connection_id: connection_id.and_then(|connection_id| {
						String::from_utf8(connection_id)
							.ok()
							.and_then(|connection_id| ConnectionId::from_str(&connection_id).ok())
					}),
					client_id: ClientId::from_str(
						&String::from_utf8(client_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					counterparty_connection_id: counterparty_connection_id.and_then(
						|connection_id| {
							String::from_utf8(connection_id).ok().and_then(|connection_id| {
								ConnectionId::from_str(&connection_id).ok()
							})
						},
					),
					counterparty_client_id: ClientId::from_str(
						&String::from_utf8(counterparty_client_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
				}
				.into(),
			)),
			IbcEvent::OpenConfirmConnection {
				revision_height,
				revision_number,
				connection_id,
				client_id,
				counterparty_connection_id,
				counterparty_client_id,
			} => Ok(RawIbcEvent::OpenConfirmConnection(
				ConnectionEvents::Attributes {
					height: Height::new(revision_number, revision_height),
					connection_id: connection_id.and_then(|connection_id| {
						String::from_utf8(connection_id)
							.ok()
							.and_then(|connection_id| ConnectionId::from_str(&connection_id).ok())
					}),
					client_id: ClientId::from_str(
						&String::from_utf8(client_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					counterparty_connection_id: counterparty_connection_id.and_then(
						|connection_id| {
							String::from_utf8(connection_id).ok().and_then(|connection_id| {
								ConnectionId::from_str(&connection_id).ok()
							})
						},
					),
					counterparty_client_id: ClientId::from_str(
						&String::from_utf8(counterparty_client_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
				}
				.into(),
			)),
			IbcEvent::OpenTryConnection {
				revision_height,
				revision_number,
				connection_id,
				client_id,
				counterparty_connection_id,
				counterparty_client_id,
			} => Ok(RawIbcEvent::OpenTryConnection(
				ConnectionEvents::Attributes {
					height: Height::new(revision_number, revision_height),
					connection_id: connection_id.and_then(|connection_id| {
						String::from_utf8(connection_id)
							.ok()
							.and_then(|connection_id| ConnectionId::from_str(&connection_id).ok())
					}),
					client_id: ClientId::from_str(
						&String::from_utf8(client_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					counterparty_connection_id: counterparty_connection_id.and_then(
						|connection_id| {
							String::from_utf8(connection_id).ok().and_then(|connection_id| {
								ConnectionId::from_str(&connection_id).ok()
							})
						},
					),
					counterparty_client_id: ClientId::from_str(
						&String::from_utf8(counterparty_client_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
				}
				.into(),
			)),
			IbcEvent::OpenAckConnection {
				revision_height,
				revision_number,
				connection_id,
				client_id,
				counterparty_connection_id,
				counterparty_client_id,
			} => Ok(RawIbcEvent::OpenAckConnection(
				ConnectionEvents::Attributes {
					height: Height::new(revision_number, revision_height),
					connection_id: connection_id.and_then(|connection_id| {
						String::from_utf8(connection_id)
							.ok()
							.and_then(|connection_id| ConnectionId::from_str(&connection_id).ok())
					}),
					client_id: ClientId::from_str(
						&String::from_utf8(client_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					counterparty_connection_id: counterparty_connection_id.and_then(
						|connection_id| {
							String::from_utf8(connection_id).ok().and_then(|connection_id| {
								ConnectionId::from_str(&connection_id).ok()
							})
						},
					),
					counterparty_client_id: ClientId::from_str(
						&String::from_utf8(counterparty_client_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
				}
				.into(),
			)),
			IbcEvent::OpenInitChannel {
				revision_height,
				revision_number,
				port_id,
				channel_id,
				connection_id,
				counterparty_port_id,
				counterparty_channel_id,
			} => Ok(RawIbcEvent::OpenInitChannel(ChannelEvents::OpenInit {
				height: Height::new(revision_number, revision_height),
				port_id: PortId::from_str(&String::from_utf8(port_id).map_err(|_| ERROR_STR)?)
					.map_err(|_| ERROR_STR)?,
				channel_id: channel_id.and_then(|channel_id| {
					String::from_utf8(channel_id)
						.ok()
						.and_then(|channel_id| ChannelId::from_str(&channel_id).ok())
				}),
				connection_id: ConnectionId::from_str(
					&String::from_utf8(connection_id).map_err(|_| ERROR_STR)?,
				)
				.map_err(|_| ERROR_STR)?,
				counterparty_port_id: PortId::from_str(
					&String::from_utf8(counterparty_port_id).map_err(|_| ERROR_STR)?,
				)
				.map_err(|_| ERROR_STR)?,
				counterparty_channel_id: counterparty_channel_id.and_then(|channel_id| {
					String::from_utf8(channel_id)
						.ok()
						.and_then(|channel_id| ChannelId::from_str(&channel_id).ok())
				}),
			})),
			IbcEvent::OpenConfirmChannel {
				revision_height,
				revision_number,
				port_id,
				channel_id,
				connection_id,
				counterparty_port_id,
				counterparty_channel_id,
			} => Ok(RawIbcEvent::OpenConfirmChannel(ChannelEvents::OpenConfirm {
				height: Height::new(revision_number, revision_height),
				port_id: PortId::from_str(&String::from_utf8(port_id).map_err(|_| ERROR_STR)?)
					.map_err(|_| ERROR_STR)?,
				channel_id: channel_id.and_then(|channel_id| {
					String::from_utf8(channel_id)
						.ok()
						.and_then(|channel_id| ChannelId::from_str(&channel_id).ok())
				}),
				connection_id: ConnectionId::from_str(
					&String::from_utf8(connection_id).map_err(|_| ERROR_STR)?,
				)
				.map_err(|_| ERROR_STR)?,
				counterparty_port_id: PortId::from_str(
					&String::from_utf8(counterparty_port_id).map_err(|_| ERROR_STR)?,
				)
				.map_err(|_| ERROR_STR)?,
				counterparty_channel_id: counterparty_channel_id.and_then(|channel_id| {
					String::from_utf8(channel_id)
						.ok()
						.and_then(|channel_id| ChannelId::from_str(&channel_id).ok())
				}),
			})),
			IbcEvent::OpenTryChannel {
				revision_height,
				revision_number,
				port_id,
				channel_id,
				connection_id,
				counterparty_port_id,
				counterparty_channel_id,
			} => Ok(RawIbcEvent::OpenTryChannel(ChannelEvents::OpenTry {
				height: Height::new(revision_number, revision_height),
				port_id: PortId::from_str(&String::from_utf8(port_id).map_err(|_| ERROR_STR)?)
					.map_err(|_| ERROR_STR)?,
				channel_id: channel_id.and_then(|channel_id| {
					String::from_utf8(channel_id)
						.ok()
						.and_then(|channel_id| ChannelId::from_str(&channel_id).ok())
				}),
				connection_id: ConnectionId::from_str(
					&String::from_utf8(connection_id).map_err(|_| ERROR_STR)?,
				)
				.map_err(|_| ERROR_STR)?,
				counterparty_port_id: PortId::from_str(
					&String::from_utf8(counterparty_port_id).map_err(|_| ERROR_STR)?,
				)
				.map_err(|_| ERROR_STR)?,
				counterparty_channel_id: counterparty_channel_id.and_then(|channel_id| {
					String::from_utf8(channel_id)
						.ok()
						.and_then(|channel_id| ChannelId::from_str(&channel_id).ok())
				}),
			})),
			IbcEvent::OpenAckChannel {
				revision_height,
				revision_number,
				port_id,
				channel_id,
				connection_id,
				counterparty_port_id,
				counterparty_channel_id,
			} => Ok(RawIbcEvent::OpenAckChannel(ChannelEvents::OpenAck {
				height: Height::new(revision_number, revision_height),
				port_id: PortId::from_str(&String::from_utf8(port_id).map_err(|_| ERROR_STR)?)
					.map_err(|_| ERROR_STR)?,
				channel_id: channel_id.and_then(|channel_id| {
					String::from_utf8(channel_id)
						.ok()
						.and_then(|channel_id| ChannelId::from_str(&channel_id).ok())
				}),
				connection_id: ConnectionId::from_str(
					&String::from_utf8(connection_id).map_err(|_| ERROR_STR)?,
				)
				.map_err(|_| ERROR_STR)?,
				counterparty_port_id: PortId::from_str(
					&String::from_utf8(counterparty_port_id).map_err(|_| ERROR_STR)?,
				)
				.map_err(|_| ERROR_STR)?,
				counterparty_channel_id: counterparty_channel_id.and_then(|channel_id| {
					String::from_utf8(channel_id)
						.ok()
						.and_then(|channel_id| ChannelId::from_str(&channel_id).ok())
				}),
			})),
			IbcEvent::CloseInitChannel {
				revision_height,
				revision_number,
				port_id,
				channel_id,
				connection_id,
				counterparty_port_id,
				counterparty_channel_id,
			} => Ok(RawIbcEvent::CloseInitChannel(ChannelEvents::CloseInit {
				height: Height::new(revision_number, revision_height),
				port_id: PortId::from_str(&String::from_utf8(port_id).map_err(|_| ERROR_STR)?)
					.map_err(|_| ERROR_STR)?,
				channel_id: ChannelId::from_str(
					&String::from_utf8(channel_id).map_err(|_| ERROR_STR)?,
				)
				.map_err(|_| ERROR_STR)?,
				connection_id: ConnectionId::from_str(
					&String::from_utf8(connection_id).map_err(|_| ERROR_STR)?,
				)
				.map_err(|_| ERROR_STR)?,
				counterparty_port_id: PortId::from_str(
					&String::from_utf8(counterparty_port_id).map_err(|_| ERROR_STR)?,
				)
				.map_err(|_| ERROR_STR)?,
				counterparty_channel_id: counterparty_channel_id.and_then(|channel_id| {
					String::from_utf8(channel_id)
						.ok()
						.and_then(|channel_id| ChannelId::from_str(&channel_id).ok())
				}),
			})),
			IbcEvent::CloseConfirmChannel {
				revision_height,
				revision_number,
				port_id,
				channel_id,
				connection_id,
				counterparty_port_id,
				counterparty_channel_id,
			} => Ok(RawIbcEvent::CloseConfirmChannel(ChannelEvents::CloseConfirm {
				height: Height::new(revision_number, revision_height),
				port_id: PortId::from_str(&String::from_utf8(port_id).map_err(|_| ERROR_STR)?)
					.map_err(|_| ERROR_STR)?,
				channel_id: channel_id.and_then(|channel_id| {
					String::from_utf8(channel_id)
						.ok()
						.and_then(|channel_id| ChannelId::from_str(&channel_id).ok())
				}),
				connection_id: ConnectionId::from_str(
					&String::from_utf8(connection_id).map_err(|_| ERROR_STR)?,
				)
				.map_err(|_| ERROR_STR)?,
				counterparty_port_id: PortId::from_str(
					&String::from_utf8(counterparty_port_id).map_err(|_| ERROR_STR)?,
				)
				.map_err(|_| ERROR_STR)?,
				counterparty_channel_id: counterparty_channel_id.and_then(|channel_id| {
					String::from_utf8(channel_id)
						.ok()
						.and_then(|channel_id| ChannelId::from_str(&channel_id).ok())
				}),
			})),

			IbcEvent::ReceivePacket {
				revision_height,
				revision_number,
				port_id,
				channel_id,
				dest_port,
				dest_channel,
				sequence,
			} => Ok(RawIbcEvent::ReceivePacket(ChannelEvents::ReceivePacket {
				height: Height::new(revision_number, revision_height),
				packet: Packet {
					sequence: sequence.into(),
					source_port: PortId::from_str(
						&String::from_utf8(port_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					source_channel: ChannelId::from_str(
						&String::from_utf8(channel_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					destination_port: PortId::from_str(
						&String::from_utf8(dest_port).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					destination_channel: ChannelId::from_str(
						&String::from_utf8(dest_channel).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					data: Default::default(),
					timeout_height: Height::default(),
					timeout_timestamp: Timestamp::default(),
				},
			})),
			IbcEvent::SendPacket {
				revision_height,
				revision_number,
				port_id,
				channel_id,
				dest_port,
				dest_channel,
				sequence,
			} => Ok(RawIbcEvent::SendPacket(ChannelEvents::SendPacket {
				height: Height::new(revision_number, revision_height),
				packet: Packet {
					sequence: sequence.into(),
					source_port: PortId::from_str(
						&String::from_utf8(port_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					source_channel: ChannelId::from_str(
						&String::from_utf8(channel_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					destination_port: PortId::from_str(
						&String::from_utf8(dest_port).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					destination_channel: ChannelId::from_str(
						&String::from_utf8(dest_channel).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					data: Default::default(),
					timeout_height: Height::default(),
					timeout_timestamp: Timestamp::default(),
				},
			})),
			IbcEvent::AcknowledgePacket {
				revision_height,
				revision_number,
				port_id,
				channel_id,
				sequence,
			} => Ok(RawIbcEvent::AcknowledgePacket(ChannelEvents::AcknowledgePacket {
				height: Height::new(revision_number, revision_height),
				packet: Packet {
					sequence: sequence.into(),
					source_port: PortId::from_str(
						&String::from_utf8(port_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					source_channel: ChannelId::from_str(
						&String::from_utf8(channel_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					destination_port: Default::default(),
					destination_channel: Default::default(),
					data: Default::default(),
					timeout_height: Height::default(),
					timeout_timestamp: Timestamp::default(),
				},
			})),
			IbcEvent::WriteAcknowledgement {
				revision_height,
				revision_number,
				port_id,
				channel_id,
				dest_port,
				dest_channel,
				sequence,
			} => Ok(RawIbcEvent::WriteAcknowledgement(ChannelEvents::WriteAcknowledgement {
				height: Height::new(revision_number, revision_height),
				packet: Packet {
					sequence: sequence.into(),
					source_port: PortId::from_str(
						&String::from_utf8(port_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					source_channel: ChannelId::from_str(
						&String::from_utf8(channel_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					destination_port: PortId::from_str(
						&String::from_utf8(dest_port).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					destination_channel: ChannelId::from_str(
						&String::from_utf8(dest_channel).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					data: Default::default(),
					timeout_height: Height::default(),
					timeout_timestamp: Timestamp::default(),
				},
				ack: Default::default(),
			})),
			IbcEvent::TimeoutPacket {
				revision_height,
				revision_number,
				port_id,
				channel_id,
				sequence,
			} => Ok(RawIbcEvent::TimeoutPacket(ChannelEvents::TimeoutPacket {
				height: Height::new(revision_number, revision_height),
				packet: Packet {
					sequence: sequence.into(),
					source_port: PortId::from_str(
						&String::from_utf8(port_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					source_channel: ChannelId::from_str(
						&String::from_utf8(channel_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					destination_port: Default::default(),
					destination_channel: Default::default(),
					data: Default::default(),
					timeout_height: Height::default(),
					timeout_timestamp: Timestamp::default(),
				},
			})),
			IbcEvent::TimeoutOnClosePacket {
				revision_height,
				revision_number,
				port_id,
				channel_id,
				sequence,
			} => Ok(RawIbcEvent::TimeoutOnClosePacket(ChannelEvents::TimeoutOnClosePacket {
				height: Height::new(revision_number, revision_height),
				packet: Packet {
					sequence: sequence.into(),
					source_port: PortId::from_str(
						&String::from_utf8(port_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					source_channel: ChannelId::from_str(
						&String::from_utf8(channel_id).map_err(|_| ERROR_STR)?,
					)
					.map_err(|_| ERROR_STR)?,
					destination_port: Default::default(),
					destination_channel: Default::default(),
					data: Default::default(),
					timeout_height: Height::default(),
					timeout_timestamp: Timestamp::default(),
				},
			})),
			IbcEvent::Empty => Ok(RawIbcEvent::Empty("Empty".to_string())),
			IbcEvent::ChainError => Ok(RawIbcEvent::ChainError("Chain Error".to_string())),
			IbcEvent::AppModule { kind, module_id } => Ok(RawIbcEvent::AppModule(ModuleEvent {
				kind: String::from_utf8(kind).map_err(|_| ERROR_STR)?,
				module_name: ModuleId::from_str(
					&String::from_utf8(module_id).map_err(|_| ERROR_STR)?,
				)
				.map_err(|_| ERROR_STR)?,
				attributes: Default::default(),
			})),
		}
	}
}
