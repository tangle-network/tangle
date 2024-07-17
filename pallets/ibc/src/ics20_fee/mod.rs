use crate::{routing::Context, DenomToAssetId};
use alloc::format;
use core::{fmt::Debug, marker::PhantomData, str::FromStr};
use ibc::{
	applications::transfer::{
		acknowledgement::Acknowledgement as Ics20Ack, context::BankKeeper,
		is_receiver_chain_source, packet::PacketData, TracePrefix,
	},
	bigint::U256,
	core::{
		ics04_channel::{
			channel::{Counterparty, Order},
			error::Error as Ics04Error,
			msgs::acknowledgement::Acknowledgement,
			packet::Packet,
			Version,
		},
		ics24_host::identifier::{ChannelId, ConnectionId, PortId},
		ics26_routing::context::{Module as IbcModule, ModuleCallbackContext, ModuleOutputBuilder},
	},
	signer::Signer,
};
use ibc_primitives::IbcAccount;
use sp_core::crypto::AccountId32;
use sp_runtime::traits::Get;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, PalletId};
	use frame_system::{ensure_root, pallet_prelude::OriginFor};
	use ibc_primitives::IbcAccount;
	use sp_runtime::{
		traits::{AccountIdConversion, Get},
		Perbill,
	};

	#[pallet::config]
	pub trait Config: frame_system::Config + crate::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
		/// `ServiceChargeIn` represents the service charge rate applied to assets upon receipt via
		/// IBC.
		///
		/// The charge is applied when assets are delivered to the receiving side, on
		/// deliver(before to mint, send assets to destination account) extrinsic using the
		/// Inter-Blockchain Communication (IBC) protocol.
		///
		/// For example, if the service charge rate for incoming assets is 0.04%, `ServiceChargeIn`
		/// will be configured in rutime as
		/// parameter_types! { pub IbcIcs20ServiceChargeIn: Perbill = Perbill::from_rational(4_u32,
		/// 1000_u32 ) };
		type ServiceChargeIn: Get<Perbill>;
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type ServiceChargeIn<T: Config> = StorageValue<_, Perbill, OptionQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// storage map. key is tuple of (source_channel.sequence(), destination_channel.sequence()) and
	/// value () that means that this group of channels is feeless
	pub type FeeLessChannelIds<T: Config> =
		StorageMap<_, Blake2_128Concat, (u64, u64), (), ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		IbcTransferFeeCollected { amount: T::Balance, asset_id: T::AssetId },
		FeeLessChannelIdsAdded { source_channel: u64, destination_channel: u64 },
		FeeLessChannelIdsRemoved { source_channel: u64, destination_channel: u64 },
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn set_charge(origin: OriginFor<T>, charge: Perbill) -> DispatchResult {
			<T as crate::Config>::AdminOrigin::ensure_origin(origin)?;
			ServiceChargeIn::<T>::put(charge);
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn add_channels_to_feeless_channel_list(
			origin: OriginFor<T>,
			source_channel: u64,
			destination_channel: u64,
		) -> DispatchResult {
			ensure_root(origin)?;

			FeeLessChannelIds::<T>::insert((source_channel, destination_channel), ());
			Self::deposit_event(Event::<T>::FeeLessChannelIdsAdded {
				source_channel,
				destination_channel,
			});

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn remove_channels_from_feeless_channel_list(
			origin: OriginFor<T>,
			source_channel: u64,
			destination_channel: u64,
		) -> DispatchResult {
			ensure_root(origin)?;

			FeeLessChannelIds::<T>::remove((source_channel, destination_channel));
			Self::deposit_event(Event::<T>::FeeLessChannelIdsRemoved {
				source_channel,
				destination_channel,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T>
	where
		<T as crate::Config>::AccountIdConversion: From<IbcAccount<T::AccountId>>,
	{
		pub fn account_id() -> <T as crate::Config>::AccountIdConversion {
			IbcAccount(T::PalletId::get().into_account_truncating()).into()
		}
	}
}

pub trait FlatFeeConverter {
	type AssetId;
	type Balance;

	/// Return some value if there is pool or
	/// graph of pools to convert asset id into fee asset id
	fn get_flat_fee(
		asset_id: Self::AssetId,
		fee_asset_id: Self::AssetId,
		fee_asset_amount: Self::Balance,
	) -> Option<u128>;
}

pub struct NonFlatFeeConverter<T>(PhantomData<T>);

impl<T: crate::Config> FlatFeeConverter for NonFlatFeeConverter<T> {
	type AssetId = T::AssetId;
	type Balance = T::Balance;

	fn get_flat_fee(
		_asset_id: Self::AssetId,
		_fee_asset_id: Self::AssetId,
		_fee_asset_amount: Self::Balance,
	) -> Option<u128> {
		None
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ics20ServiceCharge<T: Config, S: IbcModule + Clone + Default + PartialEq + Eq + Debug> {
	inner: S,
	_phantom: core::marker::PhantomData<T>,
}

impl<T: Config + Send + Sync, S: IbcModule + Clone + Default + PartialEq + Eq + Debug> Default
	for Ics20ServiceCharge<T, S>
{
	fn default() -> Self {
		Self { inner: S::default(), _phantom: Default::default() }
	}
}

impl<T: Config + Send + Sync, S: IbcModule + Clone + Default + PartialEq + Eq + Debug> IbcModule
	for Ics20ServiceCharge<T, S>
where
	AccountId32: From<<T as frame_system::Config>::AccountId>,
	<T as crate::Config>::AccountIdConversion: From<IbcAccount<T::AccountId>>,
{
	fn on_chan_open_init(
		&mut self,
		ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty: &Counterparty,
		version: &Version,
		relayer: &Signer,
	) -> Result<(), Ics04Error> {
		self.inner.on_chan_open_init(
			ctx,
			output,
			order,
			connection_hops,
			port_id,
			channel_id,
			counterparty,
			version,
			relayer,
		)
	}

	fn on_chan_open_try(
		&mut self,
		ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		order: Order,
		connection_hops: &[ConnectionId],
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty: &Counterparty,
		version: &Version,
		counterparty_version: &Version,
		relayer: &Signer,
	) -> Result<Version, Ics04Error> {
		self.inner.on_chan_open_try(
			ctx,
			output,
			order,
			connection_hops,
			port_id,
			channel_id,
			counterparty,
			version,
			counterparty_version,
			relayer,
		)
	}

	fn on_chan_open_ack(
		&mut self,
		ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &ChannelId,
		counterparty_version: &Version,
		relayer: &Signer,
	) -> Result<(), Ics04Error> {
		self.inner
			.on_chan_open_ack(ctx, output, port_id, channel_id, counterparty_version, relayer)
	}

	fn on_chan_open_confirm(
		&mut self,
		ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &ChannelId,
		relayer: &Signer,
	) -> Result<(), Ics04Error> {
		self.inner.on_chan_open_confirm(ctx, output, port_id, channel_id, relayer)
	}

	fn on_chan_close_init(
		&mut self,
		ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &ChannelId,
		relayer: &Signer,
	) -> Result<(), Ics04Error> {
		self.inner.on_chan_close_init(ctx, output, port_id, channel_id, relayer)
	}

	fn on_chan_close_confirm(
		&mut self,
		ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		port_id: &PortId,
		channel_id: &ChannelId,
		relayer: &Signer,
	) -> Result<(), Ics04Error> {
		self.inner.on_chan_close_confirm(ctx, output, port_id, channel_id, relayer)
	}

	fn on_recv_packet(
		&self,
		_ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		packet: &mut Packet,
		relayer: &Signer,
	) -> Result<Acknowledgement, Ics04Error> {
		// Module ModuleCallbackContext does not have the ics20 context as part of its trait bounds
		// so we define a new context
		let mut ctx = Context::<T>::default();
		// We want the whole chain of calls to fail only if the ics20 transfer fails, because
		// the other modules are not part of ics-20 standard
		let ack = self.inner.on_recv_packet(&ctx, output, packet, relayer)?;
		let _ = Self::process_fee(&mut ctx, packet, &ack).map_err(|e| {
			log::error!(target: "pallet_ibc", "Error processing fee: {:?}", e);
		});
		Ok(ack)
	}

	fn on_acknowledgement_packet(
		&mut self,
		ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		packet: &mut Packet,
		acknowledgement: &Acknowledgement,
		relayer: &Signer,
	) -> Result<(), Ics04Error> {
		self.inner
			.on_acknowledgement_packet(ctx, output, packet, acknowledgement, relayer)
	}

	fn on_timeout_packet(
		&mut self,
		ctx: &dyn ModuleCallbackContext,
		output: &mut ModuleOutputBuilder,
		packet: &mut Packet,
		relayer: &Signer,
	) -> Result<(), Ics04Error> {
		self.inner.on_timeout_packet(ctx, output, packet, relayer)
	}
}

impl<T: Config + Send + Sync, S: IbcModule + Clone + Default + PartialEq + Eq + Debug>
	Ics20ServiceCharge<T, S>
where
	<T as crate::Config>::AccountIdConversion: From<IbcAccount<T::AccountId>>,
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	fn process_fee(
		ctx: &mut Context<T>,
		packet: &mut Packet,
		ack: &Acknowledgement,
	) -> Result<(), Ics04Error> {
		let mut packet_data = serde_json::from_slice::<PacketData>(packet.data.as_slice())
			.map_err(|e| {
				Ics04Error::implementation_specific(format!("Failed to decode packet data {e:?}"))
			})?;

		let is_feeless_channels = FeeLessChannelIds::<T>::contains_key((
			packet.source_channel.sequence(),
			packet.destination_channel.sequence(),
		));

		if is_feeless_channels {
			return Ok(());
		}

		let percent = ServiceChargeIn::<T>::get().unwrap_or(T::ServiceChargeIn::get());
		let parsed_ack = String::from_utf8(ack.clone().into_bytes())
			.map_err(|e| {
				Ics04Error::implementation_specific(format!(
					"Failed to decode acknowledgement {e:?}"
				))
			})
			.and_then(|x: String| {
				Ics20Ack::from_str(&x).map_err(|e| {
					Ics04Error::implementation_specific(format!(
						"Failed to decode acknowledgement {e:?}"
					))
				})
			})?;
		// Send full amount to receiver using the default ics20 logic
		// We only take the fee charge if the acknowledgement is not an error
		if parsed_ack.is_successful() {
			let mut prefixed_coin = if is_receiver_chain_source(
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
					Ics04Error::implementation_specific("asset does not exist".to_string())
				})?;
			let amount = packet_data.token.amount.as_u256().low_u128();
			let mut fee = {
				let fee_asset_id = T::FlatFeeAssetId::get();
				let fee_asset_amount = T::FlatFeeAmount::get();

				T::FlatFeeConverter::get_flat_fee(asset_id.clone(), fee_asset_id, fee_asset_amount)
					.unwrap_or_else(|| {
						// We have ensured that token amounts larger than the max value for
						// a u128 are rejected in the ics20 on_recv_packet callback so we
						// can multiply safely. Percent does Non-Overflowing multiplication
						// so this is infallible
						percent * amount
					})
			};

			fee = fee.min(amount);

			let receiver =
				<T as crate::Config>::AccountIdConversion::try_from(packet_data.receiver.clone())
					.map_err(|_| {
					Ics04Error::implementation_specific("Failed to receiver account".to_string())
				})?;
			prefixed_coin.amount = fee.into();
			// Now we proceed to send the service fee from the receiver's account to the pallet
			// account
			let fee_account = T::FeeAccount::get();
			ctx.send_coins(&receiver, &fee_account, &prefixed_coin)
				.map_err(|e| Ics04Error::app_module(e.to_string()))?;
			// We modify the packet data to remove the fee so any other middleware has access to the
			// correct amount deposited in the receiver's account
			packet_data.token.amount =
				(packet_data.token.amount.as_u256() - U256::from(fee)).into();
			packet.data = serde_json::to_vec(&packet_data).map_err(|_| {
				Ics04Error::implementation_specific("Failed to receiver account".to_string())
			})?;
			Pallet::<T>::deposit_event(Event::<T>::IbcTransferFeeCollected {
				amount: fee.into(),
				asset_id: asset_id.clone(),
			})
		}
		Ok(())
	}
}
