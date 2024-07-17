#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use core::{fmt::Formatter, str::FromStr, write};
use frame_support::{dispatch::DispatchResult, weights::Weight};
use ibc::{
	core::{
		ics04_channel::{
			channel::{Counterparty, Order},
			error::Error as Ics04Error,
			msgs::acknowledgement::Acknowledgement,
			packet::Packet,
			Version,
		},
		ics24_host::identifier::{ChannelId, ConnectionId, PortId},
		ics26_routing::context::{
			Acknowledgement as GenericAcknowledgement, Module, ModuleCallbackContext,
			ModuleOutputBuilder,
		},
	},
	signer::Signer,
};
use ibc_primitives::{port_id_from_bytes, CallbackWeight, HandlerMessage, IbcHandler, Timeout};
use sp_std::{marker::PhantomData, prelude::*};

pub const MODULE_ID: &str = "PalletIbcPing";
pub const PORT_ID: &str = "ping";
pub const VERSION: &str = "ping-1";

#[derive(
	Clone,
	PartialEq,
	Eq,
	parity_scale_codec::Encode,
	parity_scale_codec::Decode,
	Debug,
	scale_info::TypeInfo,
)]
pub struct SendPingParams {
	pub data: Vec<u8>,
	/// Timeout height offset relative to the client latest height
	pub timeout_height_offset: u64,
	/// Time out timestamp offset relative to client's latest height
	pub timeout_timestamp_offset: u64,
	// Channel counter, for example counter for channel-0 is 0
	pub channel_id: u64,
}

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

// Definition of the pallet logic, to be aggregated at runtime definition through
// `construct_runtime`.
#[frame_support::pallet]
pub mod pallet {
	// Import various types used to declare pallet in scope.
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Our pallet's configuration trait. All our types and constants go in here. If the
	/// pallet is dependent on specific other pallets, then their configuration traits
	/// should be added to our implied traits list.
	///
	/// `frame_system::Config` should always be included.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// ibc subsystem
		type IbcHandler: ibc_primitives::IbcHandler<<Self as frame_system::Config>::AccountId>;
	}

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(3);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn send_ping(origin: OriginFor<T>, params: SendPingParams) -> DispatchResult {
			ensure_root(origin)?;
			Self::send_ping_impl(params).map_err(|e| {
				log::trace!(target: "pallet_ibc_ping", "[send_ping] error: {:?}", e);
				Error::<T>::PacketSendError
			})?;
			Self::deposit_event(Event::<T>::PacketSent);
			Ok(())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A send packet has been registered
		PacketSent,
		/// A channel has been opened
		ChannelOpened { channel_id: Vec<u8>, port_id: Vec<u8> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid params passed
		InvalidParams,
		/// Error opening channel
		ChannelInitError,
		/// Error registering packet
		PacketSendError,
	}
}

impl<T: Config> Pallet<T> {
	pub fn send_ping_impl(params: SendPingParams) -> Result<(), ibc_primitives::Error> {
		let channel_id = ChannelId::new(params.channel_id);
		T::IbcHandler::handle_message(HandlerMessage::SendPacket {
			data: b"ping".to_vec(),
			timeout: Timeout::Offset {
				height: Some(params.timeout_height_offset),
				timestamp: Some(params.timeout_timestamp_offset),
			},
			port_id: port_id_from_bytes(PORT_ID.as_bytes().to_vec())
				.expect("Valid port id expected"),
			channel_id,
		})
	}
}

#[derive(Clone, Eq, PartialEq)]
pub struct IbcModule<T: Config>(PhantomData<T>);

impl<T: Config> Default for IbcModule<T> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

pub struct PingAcknowledgement(Vec<u8>);

impl AsRef<[u8]> for PingAcknowledgement {
	fn as_ref(&self) -> &[u8] {
		self.0.as_slice()
	}
}

impl GenericAcknowledgement for PingAcknowledgement {}

impl<T: Config> core::fmt::Debug for IbcModule<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "pallet-ibc-ping")
	}
}

impl<T: Config + Send + Sync> Module for IbcModule<T> {
	fn on_chan_open_init(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		_order: Order,
		_connection_hops: &[ConnectionId],
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_counterparty: &Counterparty,
		_version: &Version,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		log::info!("Channel initialized");
		Ok(())
	}

	fn on_chan_open_try(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		order: Order,
		_connection_hops: &[ConnectionId],
		port_id: &PortId,
		_channel_id: &ChannelId,
		counterparty: &Counterparty,
		version: &Version,
		counterparty_version: &Version,
		_relayer: &Signer,
	) -> Result<Version, Ics04Error> {
		if counterparty_version.to_string() != *VERSION || version.to_string() != *VERSION {
			return Err(Ics04Error::no_common_version());
		}

		if order != Order::Ordered {
			return Err(Ics04Error::unknown_order_type(order.to_string()));
		}

		let ping_port = PortId::from_str(PORT_ID).expect("PORT_ID is static and valid; qed");
		if counterparty.port_id() != &ping_port || port_id != &ping_port {
			return Err(Ics04Error::implementation_specific(format!(
				"Invalid counterparty port {:?}",
				counterparty.port_id()
			)));
		}

		Ok(version.clone())
	}

	fn on_chan_open_ack(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty_version: &Version,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		log::info!(
			"Channel acknowledged {:?}, {:?}, {:?}",
			channel_id,
			port_id,
			counterparty_version
		);
		Ok(())
	}

	fn on_chan_open_confirm(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &ChannelId,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		log::info!("Channel open confirmed {:?}, {:?}", channel_id, port_id);
		Ok(())
	}

	fn on_chan_close_init(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &ChannelId,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		log::info!("Channel close started {:?} {:?}", channel_id, port_id);
		Ok(())
	}

	fn on_chan_close_confirm(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &ChannelId,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		log::info!("Channel close confirmed\n ChannelId: {:?}, PortId: {:?}", channel_id, port_id);
		Ok(())
	}

	fn on_recv_packet(
		&self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		packet: &mut Packet,
		_relayer: &Signer,
	) -> Result<Acknowledgement, Ics04Error> {
		let success = "ping-success".as_bytes().to_vec();
		let data = String::from_utf8(packet.data.clone()).ok();
		log::info!("Received Packet Sequence {:?}, Packet Data {:?}", packet.sequence, data);
		let packet = packet.clone();
		T::IbcHandler::handle_message(HandlerMessage::WriteAck { packet, ack: success.clone() })
			.map_err(|e| Ics04Error::implementation_specific(format!("{e:?}")))?;
		Ok(success.into())
	}

	fn on_acknowledgement_packet(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		packet: &mut Packet,
		acknowledgement: &Acknowledgement,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		log::info!("Acknowledged Packet {:?} {:?}", packet, acknowledgement);
		Ok(())
	}

	fn on_timeout_packet(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		packet: &mut Packet,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		log::info!("Timeout Packet {:?}", packet);
		Ok(())
	}
}

pub struct WeightHandler<T: Config>(PhantomData<T>);
impl<T: Config> Default for WeightHandler<T> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<T: Config> CallbackWeight for WeightHandler<T> {
	fn on_chan_open_init(&self) -> Weight {
		Weight::default()
	}

	fn on_chan_open_try(&self) -> Weight {
		Weight::default()
	}

	fn on_chan_open_ack(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight {
		Weight::default()
	}

	fn on_chan_open_confirm(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight {
		Weight::default()
	}

	fn on_chan_close_init(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight {
		Weight::default()
	}

	fn on_chan_close_confirm(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight {
		Weight::default()
	}

	fn on_recv_packet(&self, _packet: &Packet) -> Weight {
		Weight::default()
	}

	fn on_acknowledgement_packet(
		&self,
		_packet: &Packet,
		_acknowledgement: &Acknowledgement,
	) -> Weight {
		Weight::default()
	}

	fn on_timeout_packet(&self, _packet: &Packet) -> Weight {
		Weight::default()
	}
}
