pub mod context;
pub mod memo;

use crate::{
	routing::Context, ChannelIds, Config, DenomToAssetId, Event, Pallet, SequenceFee, WeightInfo,
};
use alloc::{format, str::FromStr};
use frame_support::weights::Weight;
use frame_system::RawOrigin;
pub use ibc::applications::transfer::{MODULE_ID_STR, PORT_ID_STR};
use ibc::{
	applications::transfer::{
		acknowledgement::{Acknowledgement as Ics20Acknowledgement, ACK_ERR_STR},
		context::{
			on_chan_close_confirm, on_chan_close_init, on_chan_open_ack, on_chan_open_confirm,
			on_chan_open_init, on_chan_open_try, BankKeeper,
		},
		is_receiver_chain_source, is_sender_chain_source,
		packet::PacketData,
		relay::{
			on_ack_packet::process_ack_packet, on_recv_packet::process_recv_packet,
			on_timeout_packet::process_timeout_packet,
		},
		PrefixedCoin, PrefixedDenom, TracePrefix,
	},
	core::{
		ics04_channel::{
			channel::{Counterparty, Order},
			error::Error as Ics04Error,
			msgs::acknowledgement::Acknowledgement,
			packet::Packet,
			Version,
		},
		ics24_host::identifier::{ChannelId, ConnectionId, PortId},
		ics26_routing::context::{Module, ModuleCallbackContext, ModuleOutputBuilder},
	},
	signer::Signer,
};
use ibc_primitives::{CallbackWeight, HandlerMessage, IbcHandler};
use sp_core::crypto::AccountId32;
use sp_runtime::traits::IdentifyAccount;
use sp_std::marker::PhantomData;

pub type Ics20TransferMsg = ibc::applications::transfer::msgs::transfer::MsgTransfer<
	ibc::applications::transfer::Coin<ibc::applications::transfer::PrefixedDenom>,
>;

#[derive(Clone, Eq, Debug, PartialEq)]
pub struct IbcModule<T: Config>(PhantomData<T>);

impl<T: Config> Default for IbcModule<T> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<T: Config + Send + Sync> Module for IbcModule<T>
where
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	fn on_chan_open_init(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty: &Counterparty,
		version: &Version,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		let mut ctx = Context::<T>::default();
		on_chan_open_init(
			&mut ctx,
			output,
			order,
			connection_hops,
			port_id,
			channel_id,
			counterparty,
			version,
		)
		.map_err(|e| Ics04Error::implementation_specific(e.to_string()))
	}

	fn on_chan_open_try(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty: &Counterparty,
		version: &Version,
		counterparty_version: &Version,
		_relayer: &Signer,
	) -> Result<Version, Ics04Error> {
		let mut ctx = Context::<T>::default();
		on_chan_open_try(
			&mut ctx,
			output,
			order,
			connection_hops,
			port_id,
			channel_id,
			counterparty,
			version,
			counterparty_version,
		)
		.map_err(|e| Ics04Error::implementation_specific(e.to_string()))
	}

	fn on_chan_open_ack(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty_version: &Version,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		let _ = ChannelIds::<T>::try_mutate::<_, (), _>(|channels| {
			channels.push(channel_id.to_string().as_bytes().to_vec());
			Ok(())
		});
		let mut ctx = Context::<T>::default();
		on_chan_open_ack(&mut ctx, output, port_id, channel_id, counterparty_version)
			.map_err(|e| Ics04Error::implementation_specific(e.to_string()))
	}

	fn on_chan_open_confirm(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &ChannelId,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		let _ = ChannelIds::<T>::try_mutate::<_, (), _>(|channels| {
			channels.push(channel_id.to_string().as_bytes().to_vec());
			Ok(())
		});
		let mut ctx = Context::<T>::default();
		on_chan_open_confirm(&mut ctx, output, port_id, channel_id)
			.map_err(|e| Ics04Error::implementation_specific(e.to_string()))
	}

	fn on_chan_close_init(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &ChannelId,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		let _ = ChannelIds::<T>::try_mutate::<_, (), _>(|channels| {
			let rem = channels
				.iter()
				.filter(|chan| chan.as_slice() != channel_id.to_string().as_bytes())
				.cloned()
				.collect();
			*channels = rem;
			Ok(())
		});
		// Remove escrow address for closed channel if it exists
		let _ = Pallet::<T>::remove_channel_escrow_address(port_id, *channel_id);
		let mut ctx = Context::<T>::default();
		on_chan_close_init(&mut ctx, output, port_id, channel_id)
			.map_err(|e| Ics04Error::implementation_specific(e.to_string()))
	}

	fn on_chan_close_confirm(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &ChannelId,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		let _ = ChannelIds::<T>::try_mutate::<_, (), _>(|channels| {
			let rem = channels
				.iter()
				.filter(|chan| chan.as_slice() != channel_id.to_string().as_bytes())
				.cloned()
				.collect();
			*channels = rem;
			Ok(())
		});
		// Remove escrow address for closed channel if it exists
		let _ = Pallet::<T>::remove_channel_escrow_address(port_id, *channel_id);
		let mut ctx = Context::<T>::default();
		on_chan_close_confirm(&mut ctx, output, port_id, channel_id)
			.map_err(|e| Ics04Error::implementation_specific(e.to_string()))
	}

	fn on_recv_packet(
		&self,
		_ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		packet: &mut Packet,
		_relayer: &Signer,
	) -> Result<Acknowledgement, Ics04Error> {
		let mut ctx = Context::<T>::default();

		let result = serde_json::from_slice(packet.data.as_slice())
			.map_err(|e| {
				Ics04Error::implementation_specific(format!("Failed to decode packet data {e:?}"))
			})
			.and_then(|packet_data: PacketData| {
				// We need to reject transaction amounts that are larger than u128 since we expect
				// the balance type of the runtime to be a u128; For a U256 to be converted to a
				// u128 without truncating, the last two words should be zero

				// in order to properly calculate rate limits, we need to adjust the full denom,
				// which wan not included as part of the token in the packet data.

				let mut token = packet_data.token.clone();
				let denom = full_ibc_denom(packet, packet_data.token.clone());
				token.denom = PrefixedDenom::from_str(&denom).map_err(|_| {
					Ics04Error::implementation_specific("Failed to parse token denom".to_string())
				})?;

				let amount = packet_data.token.amount.as_u256();
				u128::try_from(amount)
					.map_err(|e| Ics04Error::implementation_specific(format!("{e:?}")))?;
				process_recv_packet(&mut ctx, output, packet, packet_data.clone())
					.map(|_| packet_data)
					.map_err(|e| {
						log::trace!(target: "pallet_ibc", "[on_recv_packet]: token: {}, error: {:?}", denom, e);
						Ics04Error::implementation_specific(e.to_string())
					})
			});

		let ack = match result {
			Err(err) => {
				log::trace!(target: "pallet_ibc", "Acknowledgement error: {:?}", err);
				let ack = Ics20Acknowledgement::Error(format!("{ACK_ERR_STR}: {err:?}"))
					.to_string()
					.into_bytes();
				Pallet::<T>::handle_message(HandlerMessage::WriteAck {
					packet: packet.clone(),
					ack: ack.clone(),
				})
				.map_err(|e| {
					Ics04Error::implementation_specific(format!("[on_recv_packet] {e:#?}"))
				})?;
				ack
			},
			Ok(packet_data) => {
				let denom = full_ibc_denom(packet, packet_data.token.clone());
				let prefixed_denom = PrefixedDenom::from_str(&denom).map_err(|_| {
					Ics04Error::implementation_specific("Failed to parse token denom".to_string())
				})?;
				Pallet::<T>::deposit_event(Event::<T>::TokenReceived {
					from: packet_data.sender,
					to: packet_data.receiver,
					ibc_denom: denom.as_bytes().to_vec(),
					local_asset_id: T::IbcDenomToAssetIdConversion::from_denom_to_asset_id(&denom)
						.ok(),
					amount: packet_data.token.amount.as_u256().as_u128().into(),
					is_receiver_source: is_receiver_chain_source(
						packet.source_port.clone(),
						packet.source_channel,
						&prefixed_denom,
					),
					source_channel: packet.source_channel.to_string().as_bytes().to_vec(),
					destination_channel: packet.destination_channel.to_string().as_bytes().to_vec(),
				});
				let packet = packet.clone();
				Pallet::<T>::handle_message(HandlerMessage::WriteAck {
					packet,
					ack: Ics20Acknowledgement::success().to_string().into_bytes(),
				})
				.map_err(|e| {
					Ics04Error::implementation_specific(format!("[on_recv_packet] {e:#?}"))
				})?;
				Ics20Acknowledgement::success().to_string().into_bytes()
			},
		};
		Ok(Acknowledgement::from_bytes(ack))
	}

	fn on_acknowledgement_packet(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		packet: &mut Packet,
		acknowledgement: &Acknowledgement,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		let mut ctx = Context::<T>::default();
		let packet_data: PacketData =
			serde_json::from_slice(packet.data.as_slice()).map_err(|e| {
				Ics04Error::implementation_specific(format!("Failed to decode packet data {e:?}"))
			})?;
		let ack = serde_json::from_slice::<Ics20Acknowledgement>(acknowledgement.as_ref())
			.map_err(|e| {
				Ics04Error::implementation_specific(format!(
					"Failed to decode acknowledgement data {e:?}"
				))
			})?;
		let sequence: u64 = packet.sequence.into();
		process_ack_packet(&mut ctx, packet, &packet_data, &ack)
			.map_err(|e| Ics04Error::implementation_specific(e.to_string()))?;
		match ack.into_result() {
			Ok(_) => {
				if SequenceFee::<T>::contains_key(sequence) {
					SequenceFee::<T>::remove(sequence);
					Pallet::<T>::deposit_event(Event::<T>::ChargingFeeConfirmed { sequence });
				}
				Pallet::<T>::deposit_event(Event::<T>::TokenTransferCompleted {
					from: packet_data.sender,
					to: packet_data.receiver,
					ibc_denom: packet_data.token.denom.to_string().as_bytes().to_vec(),
					local_asset_id: T::IbcDenomToAssetIdConversion::from_denom_to_asset_id(
						&packet_data.token.denom.to_string(),
					)
					.ok(),
					amount: packet_data.token.amount.as_u256().as_u128().into(),
					is_sender_source: is_sender_chain_source(
						packet.source_port.clone(),
						packet.source_channel,
						&packet_data.token.denom,
					),
					source_channel: packet.source_channel.to_string().as_bytes().to_vec(),
					destination_channel: packet.destination_channel.to_string().as_bytes().to_vec(),
				})
			},
			Err(e) => {
				log::trace!(
					target: "pallet_ibc::transfer",
					"error: acknowledgement error: {e}",
				);
				Self::refund_fee(packet, &packet_data)?;
				Pallet::<T>::deposit_event(Event::<T>::ChargingFeeFailedAcknowledgement {
					sequence,
				});

				Pallet::<T>::deposit_event(Event::<T>::TokenTransferFailed {
					from: packet_data.sender,
					to: packet_data.receiver,
					ibc_denom: packet_data.token.denom.to_string().as_bytes().to_vec(),
					local_asset_id: T::IbcDenomToAssetIdConversion::from_denom_to_asset_id(
						&packet_data.token.denom.to_string(),
					)
					.ok(),
					amount: packet_data.token.amount.as_u256().as_u128().into(),
					is_sender_source: is_sender_chain_source(
						packet.source_port.clone(),
						packet.source_channel,
						&packet_data.token.denom,
					),
					source_channel: packet.source_channel.to_string().as_bytes().to_vec(),
					destination_channel: packet.destination_channel.to_string().as_bytes().to_vec(),
				})
			},
		}

		Ok(())
	}

	fn on_timeout_packet(
		&mut self,
		_ctx: &dyn ModuleCallbackContext,
		_output: &mut ModuleOutputBuilder,
		packet: &mut Packet,
		_relayer: &Signer,
	) -> Result<(), Ics04Error> {
		let mut ctx = Context::<T>::default();
		let packet_data: PacketData = serde_json::from_slice(packet.data.as_slice())
			.map_err(|e| Ics04Error::app_module(format!("Failed to decode packet data {e:?}")))?;
		process_timeout_packet(&mut ctx, packet, &packet_data)
			.map_err(|e| Ics04Error::app_module(e.to_string()))?;
		let sequence: u64 = packet.sequence.into();
		Self::refund_fee(packet, &packet_data)?;
		Pallet::<T>::deposit_event(Event::<T>::ChargingFeeTimeout { sequence });

		Pallet::<T>::deposit_event(Event::<T>::TokenTransferTimeout {
			from: packet_data.sender,
			to: packet_data.receiver,
			ibc_denom: packet_data.token.denom.to_string().as_bytes().to_vec(),
			local_asset_id: T::IbcDenomToAssetIdConversion::from_denom_to_asset_id(
				&packet_data.token.denom.to_string(),
			)
			.ok(),
			amount: packet_data.token.amount.as_u256().as_u128().into(),
			is_sender_source: is_sender_chain_source(
				packet.source_port.clone(),
				packet.source_channel,
				&packet_data.token.denom,
			),
			source_channel: packet.source_channel.to_string().as_bytes().to_vec(),
			destination_channel: packet.destination_channel.to_string().as_bytes().to_vec(),
		});
		Ok(())
	}
}

impl<T> IbcModule<T>
where
	T: Config + Send + Sync,
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	/// Refunds the fee from the FeeAccount to the sender of the packet.
	///
	/// This function is called on `on_timeout_packet` and `on_acknowledgement_packet` in case of
	/// failure when an IBC packet did not get delivered to the destination chain.
	///
	/// # Parameters
	///
	/// - `packet`: The packet that failed to be delivered.
	/// - `packet_data`: The data associated with the packet.
	///
	/// # Returns
	///
	/// Returns `Ok(())` if the fee refund is successful or there is no fee to refund, otherwise
	/// returns `Ics04Error` with a specific error message.
	///
	/// # Errors
	///
	/// This function will return an error if:
	///
	/// - The fee cannot be refunded to the sender's account. ctx.send_coins failed.
	/// - The sender's account cannot be parsed from the packet data.
	fn refund_fee(packet: &Packet, packet_data: &PacketData) -> Result<(), Ics04Error> {
		use ibc::bigint::U256;
		use sp_core::Get;
		let sequence: u64 = packet.sequence.into();
		if !SequenceFee::<T>::contains_key(sequence) {
			return Ok(()) //there is nothing to refund.
		}
		let fee = SequenceFee::<T>::take(sequence);

		let fee_account = T::FeeAccount::get();

		let mut ctx = Context::<T>::default();
		let mut fee_coin = packet_data.token.clone();

		fee_coin.amount = U256::from(fee).into();

		let signer_from = packet_data.sender.clone();
		let refund_to_account_id =
			<T as Config>::AccountIdConversion::try_from(signer_from.clone()).map_err(|_| {
				Ics04Error::implementation_specific(format!(
					"Failed to parse receiver account {signer_from:?}"
				))
			})?;

		ctx.send_coins(&fee_account, &refund_to_account_id, &fee_coin).map_err(|e| {
				log::debug!(target: "pallet_ibc", "[{}]: error when refund the fee : {:?} for sequence {}", &e, fee, sequence);
				Ics04Error::implementation_specific(format!(
					"Failed to refund fee to sender account {signer_from:?}, fee : {fee} , sequence : {sequence} "
				))
			})?;
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
		<T as Config>::WeightInfo::on_chan_open_init()
	}

	fn on_chan_open_try(&self) -> Weight {
		<T as Config>::WeightInfo::on_chan_open_try()
	}

	fn on_chan_open_ack(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight {
		<T as Config>::WeightInfo::on_chan_open_ack()
	}

	fn on_chan_open_confirm(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight {
		<T as Config>::WeightInfo::on_chan_open_confirm()
	}

	fn on_chan_close_init(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight {
		<T as Config>::WeightInfo::on_chan_close_init()
	}

	fn on_chan_close_confirm(&self, _port_id: &PortId, _channel_id: &ChannelId) -> Weight {
		<T as Config>::WeightInfo::on_chan_close_confirm()
	}

	fn on_recv_packet(&self, _packet: &Packet) -> Weight {
		<T as Config>::WeightInfo::on_recv_packet()
	}

	fn on_acknowledgement_packet(
		&self,
		_packet: &Packet,
		_acknowledgement: &Acknowledgement,
	) -> Weight {
		<T as Config>::WeightInfo::on_acknowledgement_packet()
	}

	fn on_timeout_packet(&self, _packet: &Packet) -> Weight {
		<T as Config>::WeightInfo::on_timeout_packet()
	}
}

pub fn full_ibc_denom(packet: &Packet, mut token: PrefixedCoin) -> String {
	if is_receiver_chain_source(packet.source_port.clone(), packet.source_channel, &token.denom) {
		let prefix = TracePrefix::new(packet.source_port.clone(), packet.source_channel);

		token.denom.remove_trace_prefix(&prefix);
		token.denom.to_string()
	} else {
		let prefix = TracePrefix::new(packet.destination_port.clone(), packet.destination_channel);

		token.denom.add_trace_prefix(prefix);
		token.denom.to_string()
	}
}

use ibc::applications::transfer::error::Error as Ics20Error;

pub trait HandleMemo<T: Config> {
	fn execute_memo(&self, packet: &Packet) -> Result<(), Ics20Error>;
}

impl<T: Config> HandleMemo<T> for () {
	fn execute_memo(&self, _packet: &Packet) -> Result<(), Ics20Error> {
		Ok(())
	}
}

pub trait SubstrateMultihopXcmHandler {
	type AccountId;

	fn transfer_xcm(
		from: Self::AccountId,
		to: Self::AccountId,
		para_id: Option<u32>,
		amount: u128,
		currency: u128,
	) -> Option<()>;
}

pub struct SubstrateMultihopXcmHandlerNone<T>(PhantomData<T>);

impl<T: Config> SubstrateMultihopXcmHandler for SubstrateMultihopXcmHandlerNone<T> {
	type AccountId = T::AccountId;
	fn transfer_xcm(
		_: Self::AccountId,
		_: Self::AccountId,
		_: Option<u32>,
		_: u128,
		_: u128,
	) -> Option<()> {
		None
	}
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Forward {
	pub receiver: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub port: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub channel: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub timeout: Option<u64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub retries: Option<u64>,

	/// Determines the destination for message routing based on IBC capabilities.
	///
	/// Since other parachains do not support IBC memo, there are only two options:
	/// sending the message to a specific parachain or sending it to the relay-chain.
	///
	/// If the para ID is `None`, it indicates that the message should be sent to the relay-chain.
	///
	/// # Arguments
	///
	/// * `para_id` - The identifier of the target parachain (if applicable).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub para_id: Option<u32>, //if para id is none, it means send to relay-chain
	#[serde(skip_serializing_if = "Option::is_none")]
	pub substrate: Option<bool>,
	///
	#[serde(skip_serializing_if = "Option::is_none")]
	pub next: Option<Box<MemoData>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MemoData {
	pub forward: Forward,
}

impl TryFrom<MemoData> for String {
	type Error = ();
	fn try_from(value: MemoData) -> Result<Self, Self::Error> {
		serde_json::to_string(&value).map_err(|_| ())
	}
}

impl TryFrom<String> for MemoData {
	type Error = ();
	fn try_from(string: String) -> Result<Self, Self::Error> {
		serde_json::from_str(&string).map_err(|_| ())
	}
}

pub struct MemoIbc {
	pub receiver: String,
	pub port: String,
	pub channel: String,
	pub timeout: u64,
	pub retries: u64,
}

pub struct MemoXcm {
	pub receiver: String,
	pub para_id: Option<u32>, //if para id is none, it means send to relay-chain
}

pub enum MemoType {
	IBC(MemoIbc),
	XCM(MemoXcm),
}

impl Forward {
	pub fn get_memo(&self) -> Result<MemoType, Ics20Error> {
		if self.substrate.unwrap_or_default() {
			let xcm = MemoXcm { receiver: self.receiver.clone(), para_id: self.para_id };
			return Ok(MemoType::XCM(xcm))
		}
		Ok(MemoType::IBC(MemoIbc {
			receiver: self.receiver.clone(),
			port: self
				.port
				.clone()
				.ok_or(Ics20Error::implementation_specific("Failed to get port".to_string()))?,
			channel: self
				.channel
				.clone()
				.ok_or(Ics20Error::implementation_specific("Failed to get channel".to_string()))?,
			timeout: self
				.timeout
				.ok_or(Ics20Error::implementation_specific("Failed to get timeout".to_string()))?,
			retries: self
				.retries
				.ok_or(Ics20Error::implementation_specific("Failed to get retries".to_string()))?,
		}))
	}
}

impl<H: Default, T> Default for IbcMemoHandler<H, T> {
	fn default() -> Self {
		Self { inner: H::default(), _phantom: PhantomData }
	}
}

pub struct IbcMemoHandler<H, T> {
	pub inner: H,
	pub _phantom: PhantomData<T>,
}

use parity_scale_codec::Decode;
impl<T, H: HandleMemo<T>> HandleMemo<T> for IbcMemoHandler<H, T>
where
	T: Config + Send + Sync + pallet_timestamp::Config,
	AccountId32: From<<T as frame_system::Config>::AccountId>,
	u128: From<T::AssetId>,
{
	fn execute_memo(&self, packet: &Packet) -> Result<(), Ics20Error> {
		self.inner.execute_memo(packet)?;

		let packet_data: PacketData =
			serde_json::from_slice(packet.data.as_slice()).map_err(|e| {
				Ics20Error::implementation_specific(format!("Failed to decode packet data {:?}", e))
			})?;
		let receiver = <T as Config>::AccountIdConversion::try_from(packet_data.receiver.clone())
			.map_err(|_| {
				Ics20Error::implementation_specific(format!("Failed to parse receiver account"))
			})?
			.into_account();

		//Handle only memo with IBC forward.
		if packet_data.memo.is_empty() {
			return Ok(())
		}

		crate::Pallet::<T>::deposit_event(Event::<T>::ExecuteMemoStarted {
			account_id: receiver.clone(),
			memo: Some(packet_data.memo.clone()),
		});

		let memo: MemoData = serde_json::from_str(&packet_data.memo).map_err(|_| {
			IbcModule::<T>::emit_memo_execution_failed_event(
				receiver.clone(),
				packet_data.memo.clone(),
				0,
			);
			Ics20Error::implementation_specific(format!(
				"Failed to parse memo : {:?} ",
				packet_data.memo
			))
		})?;

		let prefixed_coin = if is_receiver_chain_source(
			packet.source_port.clone(),
			packet.source_channel,
			&packet_data.token.denom,
		) {
			let prefix = TracePrefix::new(packet.source_port.clone(), packet.source_channel);
			let mut c = packet_data.token.clone();
			c.denom.remove_trace_prefix(&prefix);
			c
		} else {
			let prefix =
				TracePrefix::new(packet.destination_port.clone(), packet.destination_channel);
			let mut c = packet_data.token.clone();
			c.denom.add_trace_prefix(prefix);
			c
		};

		// At this point the asset SHOULD exist
		let asset_id =
			<T as crate::Config>::IbcDenomToAssetIdConversion::from_denom_to_asset_id(
				&prefixed_coin.denom.to_string(),
			)
			.map_err(|_| {
				log::warn!(target: "pallet_ibc", "Asset does not exist for denom: {}", prefixed_coin.denom.to_string());
				IbcModule::<T>::emit_memo_execution_failed_event(receiver.clone(), packet_data.memo.clone(), 1);
				Ics20Error::implementation_specific("asset does not exist".to_string())
			})?;

		let amount = packet_data.token.amount.as_u256().low_u128();

		let memo_forward = memo.forward.get_memo().map_err(|_| {
			IbcModule::<T>::emit_memo_execution_failed_event(
				receiver.clone(),
				packet_data.memo.clone(),
				2,
			);
			Ics20Error::implementation_specific("Failed to get memo".to_string())
		})?;

		let memo_forward = match memo_forward {
			MemoType::IBC(memo_forward) => memo_forward,
			MemoType::XCM(memo_forward) => {
				let s = memo_forward.receiver.strip_prefix("0x").ok_or_else(|| {
					IbcModule::<T>::emit_memo_execution_failed_event(
						receiver.clone(),
						packet_data.memo.clone(),
						11,
					);
					Ics20Error::implementation_specific("failed strip_prefix.".to_string())
				})?;

				let decoded_accout = hex::decode(s).map_err(|_| {
					IbcModule::<T>::emit_memo_execution_failed_event(
						receiver.clone(),
						packet_data.memo.clone(),
						12,
					);
					Ics20Error::implementation_specific("hex::decode".to_string())
				})?;

				let account_to = T::AccountId::decode(&mut &*decoded_accout).map_err(|_| {
					IbcModule::<T>::emit_memo_execution_failed_event(
						receiver.clone(),
						packet_data.memo.clone(),
						13,
					);
					Ics20Error::implementation_specific("T::AccountId::decode".to_string())
				})?;

				T::SubstrateMultihopXcmHandler::transfer_xcm(
					receiver.clone(),
					account_to.clone(),
					memo_forward.para_id,
					amount,
					asset_id.clone().into(),
				)
				.ok_or_else(|| {
					crate::Pallet::<T>::deposit_event(Event::<T>::ExecuteMemoXcmFailed {
						from: receiver.clone(),
						to: account_to.clone(),
						para_id: memo_forward.para_id,
						amount,
						asset_id: asset_id.clone(),
					});
					Ics20Error::implementation_specific(
						"Faield to execute SubstrateMultihopXcmHandler::transfer_xcm.".to_string(),
					)
				})?;

				crate::Pallet::<T>::deposit_event(Event::<T>::ExecuteMemoXcmSuccess {
					from: receiver.clone(),
					to: account_to.clone(),
					para_id: memo_forward.para_id,
					amount,
					asset_id: asset_id.clone(),
				});

				return Ok(())
			},
		};

		let raw_bytes = memo_forward.receiver.into_bytes();

		let transfer_ibc_account_to =
			crate::MultiAddress::<<T as frame_system::Config>::AccountId>::Raw(raw_bytes.clone());

		let origin = RawOrigin::Signed(receiver.clone());
		let channel_id = memo_forward
			.channel
			.split('-')
			.last()
			.ok_or_else(|| {
				IbcModule::<T>::emit_memo_execution_failed_event(
					receiver.clone(),
					packet_data.memo.clone(),
					3,
				);
				Ics20Error::implementation_specific(format!(
					"Failed to extract channel number from channel ID: {:?}",
					memo_forward.channel
				))
			})?
			.parse()
			.map_err(|_| {
				IbcModule::<T>::emit_memo_execution_failed_event(
					receiver.clone(),
					packet_data.memo.clone(),
					4,
				);
				Ics20Error::implementation_specific("Failed to parse channel ID".to_string())
			})?;

		let params = crate::TransferParams::<<T as frame_system::Config>::AccountId> {
			to: transfer_ibc_account_to,
			source_channel: channel_id,
			timeout: ibc_primitives::Timeout::Offset {
				timestamp: Some(memo_forward.timeout),
				height: Some(1000),
			},
		};

		let mut next_memo: Option<T::MemoMessage> = None;
		if let Some(memo) = memo.forward.next {
			let memo_result = <T as crate::Config>::MemoMessage::try_from(*memo).map_err(|_| {
				IbcModule::<T>::emit_memo_execution_failed_event(
					receiver.clone(),
					packet_data.memo.clone(),
					5,
				);
				Ics20Error::implementation_specific("failed to try_from memo".to_string())
			})?;

			next_memo = Some(memo_result);
		}

		crate::Pallet::<T>::transfer(
			origin.into(),
			params,
			asset_id.clone(),
			amount.into(),
			next_memo.clone(),
		)
		.map_err(|_| {
			crate::Pallet::<T>::deposit_event(Event::<T>::ExecuteMemoIbcTokenTransferFailed {
				from: receiver.clone(),
				to: raw_bytes.clone(),
				asset_id: asset_id.clone(),
				amount: amount.into(),
				channel: channel_id,
				next_memo: next_memo.clone(),
			});
			Ics20Error::implementation_specific(
				"Pallet ibc transfer failed to send message".to_string(),
			)
		})?;

		crate::Pallet::<T>::deposit_event(Event::<T>::ExecuteMemoIbcTokenTransferSuccess {
			from: receiver,
			to: raw_bytes,
			asset_id,
			amount: amount.into(),
			channel: channel_id,
			next_memo,
		});
		Ok(())
	}
}

impl<T> IbcModule<T>
where
	T: Config + Send + Sync,
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	//function that takes account and memo and emit event that memo execution failed
	fn emit_memo_execution_failed_event(account: T::AccountId, memo: String, reason: u8) {
		crate::Pallet::<T>::deposit_event(
			Event::<T>::ExecuteMemoIbcTokenTransferFailedWithReason { from: account, memo, reason },
		);
	}
}

pub trait ValidateMemo {
	fn validate(&self) -> Result<(), String>;
}
