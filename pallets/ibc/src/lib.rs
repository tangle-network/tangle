#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unreachable_patterns)]
#![allow(clippy::type_complexity)]
#![allow(clippy::useless_format)]
#![allow(non_camel_case_types)]
#![deny(
	unused_imports,
	clippy::useless_conversion,
	bad_style,
	bare_trait_objects,
	improper_ctypes,
	non_shorthand_field_patterns,
	no_mangle_generic_items,
	overflowing_literals,
	path_statements,
	patterns_in_fns_without_body,
	unconditional_recursion,
	unused_allocation,
	unused_comparisons,
	unused_parens,
	while_true,
	trivial_casts,
	trivial_numeric_casts,
	unused_extern_crates
)]

//! Pallet IBC
//! Implements the ibc protocol for substrate runtimes.
#[macro_use]
extern crate alloc;

use crate::ics20::ValidateMemo;
use core::fmt::Debug;
use cumulus_primitives_core::ParaId;
use frame_support::weights::Weight;
pub use pallet::*;
use parity_scale_codec::{Decode, Encode};
use scale_info::{
	prelude::{
		format,
		string::{String, ToString},
		vec,
	},
	TypeInfo,
};
use sp_runtime::Either;
use sp_std::{marker::PhantomData, prelude::*, str::FromStr};

mod channel;
mod client;
mod connection;
pub mod errors;
pub mod events;
pub mod ics20;
mod ics23;
pub mod light_clients;
mod port;
pub mod routing;
pub use client::HostConsensusProof;
pub use ibc_primitives::Timeout;
pub use light_client_common;

pub const MODULE_ID: &str = "pallet_ibc";

#[derive(Clone, PartialEq, Eq, Encode, Decode, Debug, TypeInfo)]
pub struct Any {
	pub type_url: String,
	pub value: Vec<u8>,
}

impl From<ibc_proto::google::protobuf::Any> for Any {
	fn from(any: ibc_proto::google::protobuf::Any) -> Self {
		Self { type_url: any.type_url, value: any.value }
	}
}

impl From<Any> for ibc_proto::google::protobuf::Any {
	fn from(any: Any) -> Self {
		Self { type_url: any.type_url, value: any.value }
	}
}

/// Params needed to upgrade clients for all connected chains.
#[derive(Debug, PartialEq, Eq, scale_info::TypeInfo, Encode, Decode, Clone)]
pub struct UpgradeParams {
	/// Protobuf encoded client state
	pub client_state: Vec<u8>,
	/// Protobuf encoded consensus state
	pub consensus_state: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, scale_info::TypeInfo, Encode, Decode, Clone)]
pub enum MultiAddress<AccountId> {
	Id(AccountId),
	Raw(Vec<u8>),
}

#[derive(Debug, PartialEq, Eq, scale_info::TypeInfo, Encode, Decode, Clone)]
pub struct TransferParams<AccountId> {
	/// Account id or valid utf8 string bytes
	pub to: MultiAddress<AccountId>,
	/// Source channel identifier on host chain
	pub source_channel: u64,
	/// Timeout for this packet
	pub timeout: Timeout,
}

#[derive(Debug, PartialEq, Eq, scale_info::TypeInfo, Encode, Decode, Clone)]
pub enum LightClientProtocol {
	Beefy,
	Grandpa,
	GrandpaStandalone,
}
#[cfg(any(test, feature = "runtime-benchmarks"))]
pub(crate) mod benchmarks;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod ics20_fee;
mod impls;
pub mod weight;

pub use weight::WeightInfo;

use crate::ics20_fee::FlatFeeConverter;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use core::fmt::Display;

	use core::time::Duration;

	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		storage::child,
		traits::{
			fungibles::{Inspect, Mutate},
			tokens::{AssetId, Balance},
			ReservableCurrency, UnixTime,
		},
	};
	use frame_system::pallet_prelude::*;
	pub use ibc::signer::Signer;
	use sp_core::{crypto::ByteArray, storage::ChildInfo};

	#[cfg(feature = "testing")]
	use crate::ics23::{
		next_seq_ack::NextSequenceAck, next_seq_recv::NextSequenceRecv,
		next_seq_send::NextSequenceSend,
	};
	use crate::{
		ics20::{HandleMemo, SubstrateMultihopXcmHandler},
		light_clients::AnyConsensusState,
		routing::{Context, ModuleRouter},
	};
	use ibc::{
		applications::transfer::{
			context::BankKeeper, is_sender_chain_source, msgs::transfer::MsgTransfer, Amount,
			PrefixedCoin, PrefixedDenom,
		},
		bigint::U256,
		core::{
			ics02_client::context::{ClientKeeper, ClientReader},
			ics04_channel::context::ChannelReader,
			ics24_host::identifier::{ChannelId, ClientId, PortId},
		},
		timestamp::Timestamp,
		Height,
	};
	use ibc_primitives::{client_id_from_bytes, get_channel_escrow_address, IbcHandler};
	use light_clients::AnyClientState;
	use sp_runtime::{
		traits::{IdentifyAccount, Saturating, Zero},
		AccountId32, BoundedBTreeSet, Perbill,
	};
	#[cfg(feature = "std")]
	use sp_runtime::{Deserialize, Serialize};
	use sp_std::collections::btree_set::BTreeSet;
	use tendermint_proto::Protobuf;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + core::fmt::Debug {
		type TimeProvider: UnixTime;
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Currency type of the runtime
		type NativeCurrency: ReservableCurrency<
			<Self as frame_system::Config>::AccountId,
			Balance = Self::Balance,
		>;
		/// Runtime balance type
		type Balance: Balance + From<u128>;
		/// AssetId type
		type AssetId: AssetId + MaybeSerializeDeserialize + Display;
		/// The native asset id, this will use the `NativeCurrency` for all operations.
		#[pallet::constant]
		type NativeAssetId: Get<Self::AssetId>;
		/// Convert ibc denom to asset id and vice versa
		type IbcDenomToAssetIdConversion: DenomToAssetId<Self>;

		/// Prefix for events stored in the Off-chain DB via Indexing API, child trie and connection
		#[pallet::constant]
		type PalletPrefix: Get<&'static [u8]>;

		/// Light client protocol this chain is operating
		#[pallet::constant]
		type LightClientProtocol: Get<LightClientProtocol>;
		/// Account Id Conversion from SS58 string or hex string
		type AccountIdConversion: TryFrom<Signer>
			+ IdentifyAccount<AccountId = <Self as frame_system::Config>::AccountId>
			+ Clone;
		/// Set of traits needed to handle fungible assets
		type Fungibles: Mutate<
				<Self as frame_system::Config>::AccountId,
				Balance = Self::Balance,
				AssetId = Self::AssetId,
			> + Mutate<
				<Self as frame_system::Config>::AccountId,
				Balance = Self::Balance,
				AssetId = Self::AssetId,
			> + Inspect<
				<Self as frame_system::Config>::AccountId,
				Balance = Self::Balance,
				AssetId = Self::AssetId,
			>;
		/// Expected block time in milliseconds
		#[pallet::constant]
		type ExpectedBlockTime: Get<u64>;
		/// Port and Module resolution
		type Router: ModuleRouter;
		/// Minimum connection delay period in seconds for ibc connections that can be created or
		/// accepted. Ensure that this is non-zero in production as it's a critical vulnerability.
		#[pallet::constant]
		type MinimumConnectionDelay: Get<u64>;
		/// ChainId (or ParaId) of the runtime
		type ChainId: Get<ParaId>;
		/// Chain (Standalone or Relay) this runtime is attached to
		type ChainType: Get<light_client_common::ChainType>;
		/// benchmarking weight info
		type WeightInfo: WeightInfo;
		/// Origin allowed to unfreeze light clients
		type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Origin allowed to freeze light clients
		type FreezeOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Amount to be reserved for client and connection creation
		#[pallet::constant]
		type SpamProtectionDeposit: Get<Self::Balance>;
		type IbcAccountId: Into<AccountId32>;
		type TransferOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::IbcAccountId>;
		type RelayerOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		/// Handle Ics20 Memo
		type HandleMemo: HandleMemo<Self> + Default;
		/// Memo Message types supported by the runtime
		type MemoMessage: parity_scale_codec::Codec
			+ FromStr
			+ ToString
			+ Debug
			+ scale_info::TypeInfo
			+ Clone
			+ Eq
			+ TryFrom<crate::ics20::MemoData>
			+ TryInto<crate::ics20::MemoData>
			+ ValidateMemo;

		type SubstrateMultihopXcmHandler: SubstrateMultihopXcmHandler<AccountId = Self::AccountId>;

		type IsSendEnabled: Get<bool>;
		type IsReceiveEnabled: Get<bool>;
		type FeeAccount: Get<Self::AccountIdConversion>;
		/// Cleanup packets period (in blocks)
		#[pallet::constant]
		type CleanUpPacketsPeriod: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		/// `ServiceChargeOut` represents the service charge rate applied to assets that will be
		/// sent via IBC.
		///
		/// The charge is applied before assets are transffered from the sender side, during
		/// transfer extrinsic (before to burn or send assets to escrow account) before the packet
		/// send via IBC Inter-Blockchain Communication (IBC) protocol.
		///
		/// For example, if the service charge rate for incoming assets is 0.04%, `ServiceChargeIn`
		/// will be configured in rutime as
		/// parameter_types! { pub IbcIcs20ServiceChargeOut: Perbill = Perbill::from_rational(4_u32,
		/// 1000_u32 ) };
		type ServiceChargeOut: Get<Perbill>;

		type FlatFeeConverter: FlatFeeConverter<
			AssetId = <Self as crate::Config>::AssetId,
			Balance = <Self as crate::Config>::Balance,
		>;

		//Asset Id for fee that will charged. for example  USDT
		type FlatFeeAssetId: Get<Self::AssetId>;
		//Asset amount that will be charged. for example 10 (USDT)
		type FlatFeeAmount: Get<Self::Balance>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// client_id , Height => Height
	pub type ClientUpdateHeight<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		OptionQuery,
	>;

	#[pallet::storage]
	pub type ServiceChargeOut<T: Config> = StorageValue<_, Perbill, OptionQuery>;

	#[pallet::storage]
	/// client_id , Height => Timestamp
	pub type ClientUpdateTime<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, Vec<u8>, Blake2_128Concat, Vec<u8>, u64, OptionQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	pub type ChannelCounter<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	pub type PacketCounter<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// connection_identifier => Vec<(port_id, channel_id)>
	pub type ChannelsConnection<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<(Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// storage map. key is tuple of (source_channel.sequence(), destination_channel.sequence()) and
	/// value () that means that this group of channels is feeless
	pub type FeeLessChannelIds<T: Config> =
		StorageMap<_, Blake2_128Concat, (u64, u64), (), ValueQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// storage map where key is transfer sequence number and value calculated fee for that sequence
	/// number
	pub type SequenceFee<T: Config> = StorageMap<_, Blake2_128Concat, u64, u128, ValueQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// counter for clients
	pub type ClientCounter<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// counter for clients
	pub type ConnectionCounter<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// counter for acknowledgments
	pub type AcknowledgementCounter<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// counter for packet receipts
	pub type PacketReceiptCounter<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// client_id => Vec<Connection_id>
	pub type ConnectionClient<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<Vec<u8>>, ValueQuery>;

	#[pallet::storage]
	/// Map of asset id to ibc denom pairs (T::AssetId, Vec<u8>)
	/// ibc denoms represented as utf8 string bytes
	pub type IbcAssetIds<T: Config> =
		CountedStorageMap<_, Twox64Concat, T::AssetId, Vec<u8>, OptionQuery>;

	#[pallet::storage]
	/// Map of asset id to ibc denom pairs (Vec<u8>, T::AssetId)
	/// ibc denoms represented as utf8 string bytes
	pub type IbcDenoms<T: Config> =
		CountedStorageMap<_, Twox64Concat, Vec<u8>, T::AssetId, OptionQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// ChannelIds open from this module
	pub type ChannelIds<T: Config> = StorageValue<_, Vec<Vec<u8>>, ValueQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// Active Escrow addresses
	pub type EscrowAddresses<T: Config> =
		StorageValue<_, BTreeSet<<T as frame_system::Config>::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// Consensus heights
	/// Stored as a tuple of (revision_number, revision_height)
	pub type ConsensusHeights<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		BoundedBTreeSet<Height, frame_support::traits::ConstU32<256>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// SendPackets info
	pub type SendPackets<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, OptionQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// RecvPackets info
	pub type RecvPackets<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, OptionQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// Acks info
	pub type Acks<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, OptionQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// Pending send packet sequences. Used in `packet_cleanup` procedure.
	pub type PendingSendPacketSeqs<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, Vec<u8>), (BTreeSet<u64>, u64), ValueQuery>;

	#[pallet::storage]
	#[allow(clippy::disallowed_types)]
	/// Pending recv packet sequences. Used in `packet_cleanup` procedure.
	pub type PendingRecvPacketSeqs<T: Config> =
		StorageMap<_, Blake2_128Concat, (Vec<u8>, Vec<u8>), (BTreeSet<u64>, u64), ValueQuery>;

	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct AssetConfig<AssetId> {
		pub id: AssetId,
		pub denom: Vec<u8>,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// This should contain the native currency's asset_id and denom.
		pub assets: Vec<AssetConfig<T::AssetId>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { assets: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			for AssetConfig { id, denom } in &self.assets {
				IbcDenoms::<T>::insert(denom.clone(), id);
				IbcAssetIds::<T>::insert(id, denom);
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Events emitted by the ibc subsystem
		Events {
			events: Vec<Result<events::IbcEvent, errors::IbcError>>,
		},
		/// An Ibc token transfer has been started
		TokenTransferInitiated {
			from: Vec<u8>,
			to: Vec<u8>,
			ibc_denom: Vec<u8>,
			local_asset_id: Option<T::AssetId>,
			amount: T::Balance,
			is_sender_source: bool,
			source_channel: Vec<u8>,
			destination_channel: Vec<u8>,
		},
		/// A channel has been opened
		ChannelOpened {
			channel_id: Vec<u8>,
			port_id: Vec<u8>,
		},
		/// Pallet params updated
		ParamsUpdated {
			send_enabled: bool,
			receive_enabled: bool,
		},
		/// An outgoing Ibc token transfer has been completed and burnt
		TokenTransferCompleted {
			from: Signer,
			to: Signer,
			ibc_denom: Vec<u8>,
			local_asset_id: Option<T::AssetId>,
			amount: T::Balance,
			is_sender_source: bool,
			source_channel: Vec<u8>,
			destination_channel: Vec<u8>,
		},
		/// Ibc tokens have been received and minted
		TokenReceived {
			from: Signer,
			to: Signer,
			ibc_denom: Vec<u8>,
			local_asset_id: Option<T::AssetId>,
			amount: T::Balance,
			is_receiver_source: bool,
			source_channel: Vec<u8>,
			destination_channel: Vec<u8>,
		},
		/// Ibc transfer failed, received an acknowledgement error, tokens have been refunded
		TokenTransferFailed {
			from: Signer,
			to: Signer,
			ibc_denom: Vec<u8>,
			local_asset_id: Option<T::AssetId>,
			amount: T::Balance,
			is_sender_source: bool,
			source_channel: Vec<u8>,
			destination_channel: Vec<u8>,
		},
		/// Happens when token transfer timeouts, tokens have been refunded. expected
		/// `TokenTransferFailed` does not happen in this case.
		TokenTransferTimeout {
			from: Signer,
			to: Signer,
			ibc_denom: Vec<u8>,
			local_asset_id: Option<T::AssetId>,
			amount: T::Balance,
			is_sender_source: bool,
			source_channel: Vec<u8>,
			destination_channel: Vec<u8>,
		},
		/// On recv packet was not processed successfully processes
		OnRecvPacketError {
			msg: Vec<u8>,
		},
		/// Client upgrade path has been set
		ClientUpgradeSet,
		/// Client has been frozen
		ClientFrozen {
			client_id: Vec<u8>,
			height: u64,
			revision_number: u64,
		},
		/// Asset Admin Account Updated
		AssetAdminUpdated {
			admin_account: <T as frame_system::Config>::AccountId,
		},

		FeeLessChannelIdsAdded {
			source_channel: u64,
			destination_channel: u64,
		},
		FeeLessChannelIdsRemoved {
			source_channel: u64,
			destination_channel: u64,
		},
		ChargingFeeOnTransferInitiated {
			sequence: u64,
			from: Vec<u8>,
			to: Vec<u8>,
			ibc_denom: Vec<u8>,
			local_asset_id: Option<T::AssetId>,
			amount: T::Balance,
			is_flat_fee: bool,
			source_channel: Vec<u8>,
			destination_channel: Vec<u8>,
		},
		ChargingFeeConfirmed {
			sequence: u64,
		},
		ChargingFeeTimeout {
			sequence: u64,
		},
		ChargingFeeFailedAcknowledgement {
			sequence: u64,
		},
		ChildStateUpdated,
		ClientStateSubstituted {
			client_id: String,
			height: Height,
		},
		ExecuteMemoStarted {
			account_id: T::AccountId,
			memo: Option<String>,
		},
		ExecuteMemoIbcTokenTransferSuccess {
			from: T::AccountId,
			to: Vec<u8>,
			asset_id: T::AssetId,
			amount: T::Balance,
			channel: u64,
			next_memo: Option<T::MemoMessage>,
		},
		ExecuteMemoIbcTokenTransferFailedWithReason {
			from: T::AccountId,
			memo: String,
			reason: u8,
		},
		ExecuteMemoIbcTokenTransferFailed {
			from: T::AccountId,
			to: Vec<u8>,
			asset_id: T::AssetId,
			amount: T::Balance,
			channel: u64,
			next_memo: Option<T::MemoMessage>,
		},
		ExecuteMemoXcmSuccess {
			from: T::AccountId,
			to: T::AccountId,
			amount: u128,
			asset_id: T::AssetId,
			para_id: Option<u32>,
		},
		ExecuteMemoXcmFailed {
			from: T::AccountId,
			to: T::AccountId,
			amount: u128,
			asset_id: T::AssetId,
			para_id: Option<u32>,
		},
	}

	/// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error processing ibc messages
		ProcessingError,
		/// Error decoding some type
		DecodingError,
		/// Error encoding some type
		EncodingError,
		/// Error generating trie proof
		ProofGenerationError,
		/// Client consensus state not found for height
		ConsensusStateNotFound,
		/// Channel not found
		ChannelNotFound,
		/// Client state not found
		ClientStateNotFound,
		/// Connection not found
		ConnectionNotFound,
		/// Packet commitment wasn't found
		PacketCommitmentNotFound,
		/// Packet receipt wasn't found
		PacketReceiptNotFound,
		/// Packet Acknowledgment wasn't found
		PacketAcknowledgmentNotFound,
		/// Error constructing packet
		SendPacketError,
		/// Invalid channel id
		InvalidChannelId,
		/// Invalid port id
		InvalidPortId,
		/// Other forms of errors
		Other,
		/// Invalid route
		InvalidRoute,
		/// Invalid message for extrinsic
		InvalidMessageType,
		/// The interchain token transfer was not successfully initiated
		TransferInternals,
		TransferSerde,
		TransferOther,
		TransferProtocol,
		TransferSend,
		/// Error Decoding utf8 bytes
		Utf8Error,
		/// Invalid asset id
		InvalidAssetId,
		/// Invalid Ibc denom
		PrefixedDenomParse,
		/// Invalid amount
		InvalidAmount,
		/// Invalid timestamp
		InvalidTimestamp,
		/// Unable to get client revision number
		FailedToGetRevisionNumber,
		/// Invalid params passed
		InvalidParams,
		/// Error opening channel
		ChannelInitError,
		/// Latest height and timestamp for a client not found
		TimestampAndHeightNotFound,
		/// Failed to derive channel escrow address
		ChannelEscrowAddress,
		/// Error writing acknowledgement to storage
		WriteAckError,
		/// Client update time and height not found
		ClientUpdateNotFound,
		/// Error Freezing client
		ClientFreezeFailed,
		/// Access denied
		AccessDenied,
		RateLimiter,
		/// Fee errors
		FailedSendFeeToAccount,
		/// Failed to derive origin sender address.
		OriginAddress,
		/// The memo hasn't passed the validation. Potential reasons:
		/// - The memo is too long.
		/// - The memo is in invalid format
		/// - The memo contains unsupported middlewares
		InvalidMemo,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
	where
		T: Send + Sync,
		AccountId32: From<<T as frame_system::Config>::AccountId>,
	{
		fn on_idle(n: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
			if n % T::CleanUpPacketsPeriod::get() != BlockNumberFor::<T>::zero() {
				return remaining_weight;
			}
			log::trace!(target: "pallet_ibc", "Cleaning up packets");
			let removed_packets_count = Pallet::<T>::packet_cleanup()
				.map_err(|(e, n)| {
					log::warn!(target: "pallet_ibc", "Error cleaning up packets: {:?}", e);
					n
				})
				.unwrap_or_else(|n| n) as u32;
			remaining_weight.saturating_sub(T::WeightInfo::packet_cleanup(removed_packets_count))
		}

		fn offchain_worker(_n: BlockNumberFor<T>) {}
	}

	// Dispatch able functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsic", which are often compared to transactions.
	// Dispatch able functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: Send + Sync,
		AccountId32: From<<T as frame_system::Config>::AccountId>,
	{
		#[pallet::call_index(0)]
		#[pallet::weight(crate::weight::deliver::< T > (messages))]
		#[frame_support::transactional]
		pub fn deliver(origin: OriginFor<T>, messages: Vec<Any>) -> DispatchResult {
			use ibc::core::{
				ics02_client::msgs::create_client, ics03_connection::msgs::conn_open_init,
			};
			let sender = T::RelayerOrigin::ensure_origin(origin)?;

			// reserve a fixed deposit for every client and connection created
			// so people don't spam our chain with useless clients.
			let mut ctx = routing::Context::<T>::new();
			let mut reserve_count = 0u128;
			let messages = messages
				.into_iter()
				.map(|message| {
					if matches!(
						message.type_url.as_str(),
						create_client::TYPE_URL
							| conn_open_init::TYPE_URL
							| ibc::core::ics04_channel::msgs::chan_open_init::TYPE_URL
					) {
						reserve_count += 1;
					}

					ibc_proto::google::protobuf::Any {
						type_url: message.type_url,
						value: message.value,
					}
				})
				.collect::<Vec<_>>();
			let reserve_amt = T::SpamProtectionDeposit::get().saturating_mul(reserve_count.into());

			if reserve_amt >= T::SpamProtectionDeposit::get() {
				<T::NativeCurrency as ReservableCurrency<
					<T as frame_system::Config>::AccountId,
				>>::reserve(&sender, reserve_amt)?;
			}
			Self::execute_ibc_messages(&mut ctx, messages);

			Ok(())
		}

		#[pallet::call_index(1)]
		#[frame_support::transactional]
		#[pallet::weight(<T as Config>::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			params: TransferParams<<T as frame_system::Config>::AccountId>,
			asset_id: T::AssetId,
			amount: T::Balance,
			memo: Option<T::MemoMessage>,
		) -> DispatchResult {
			let account_id_32 = T::TransferOrigin::ensure_origin(origin)?.into();
			let denom = T::IbcDenomToAssetIdConversion::from_asset_id_to_denom(asset_id)
				.ok_or(Error::<T>::InvalidAssetId)?;
			let from = {
				let mut hex_string = hex::encode(account_id_32.to_raw_vec());
				hex_string.insert_str(0, "0x");
				hex_string
			};

			let to = match params.to {
				MultiAddress::Id(id) => {
					// we convert id to hex string instead of ss58 because destination chain could
					// have a different ss58 prefix from source chain
					let account_id_32: AccountId32 = id.into();
					let mut hex_string = hex::encode(account_id_32.to_raw_vec());
					hex_string.insert_str(0, "0x");
					hex_string
				},
				MultiAddress::Raw(bytes) => {
					String::from_utf8(bytes).map_err(|_| Error::<T>::Utf8Error)?
				},
			};
			let denom =
				PrefixedDenom::from_str(&denom).map_err(|_| Error::<T>::PrefixedDenomParse)?;
			let ibc_amount =
				Amount::from_str(&format!("{amount:?}")).map_err(|_| Error::<T>::InvalidAmount)?;
			let mut coin = PrefixedCoin { denom, amount: ibc_amount };
			let source_channel = ChannelId::new(params.source_channel);
			let source_port = PortId::transfer();
			let (latest_height, _) =
				Pallet::<T>::latest_height_and_timestamp(&source_port, &source_channel)
					.map_err(|_| Error::<T>::TimestampAndHeightNotFound)?;

			let (timeout_height, timeout_timestamp) = match params.timeout {
				Timeout::Offset { timestamp, height } => {
					let latest_timestamp = T::TimeProvider::now();
					let timestamp = timestamp
						.map(|offset| {
							Timestamp::from_nanoseconds(
								(latest_timestamp + Duration::from_secs(offset)).as_nanos() as u64,
							)
						})
						.transpose()
						.map_err(|_| Error::<T>::InvalidTimestamp)?
						.unwrap_or_default();
					let height = height.map(|offset| latest_height.add(offset)).unwrap_or_default();
					(height, timestamp)
				},
				Timeout::Absolute { timestamp, height } => {
					let timestamp = timestamp
						.map(Timestamp::from_nanoseconds)
						.transpose()
						.map_err(|_| Error::<T>::InvalidTimestamp)?
						.unwrap_or_default();
					let height = height
						.map(|revision_height| {
							Height::new(latest_height.revision_number, revision_height)
						})
						.unwrap_or_default();
					(height, timestamp)
				},
			};

			if timeout_height.is_zero() && timeout_timestamp.nanoseconds() == 0 {
				return Err(Error::<T>::InvalidTimestamp.into());
			}

			let mut ctx = Context::<T>::default();
			let channel_end = ctx
				.channel_end(&(PortId::transfer(), source_channel))
				.map_err(|_| Error::<T>::ChannelNotFound)?;

			let destination_channel =
				channel_end.counterparty().channel_id.ok_or(Error::<T>::ChannelNotFound)?;

			let is_feeless_channel_ids = FeeLessChannelIds::<T>::contains_key((
				source_channel.sequence(),
				destination_channel.sequence(),
			));

			if !is_feeless_channel_ids {
				let percent = ServiceChargeOut::<T>::get().unwrap_or(T::ServiceChargeOut::get());
				// Now we proceed to send the service fee from the receiver's account to the pallet
				// FeeAccount
				let fee_account = T::FeeAccount::get();

				let mut fee_coin = coin.clone();
				let asset_id =
					<T as crate::Config>::IbcDenomToAssetIdConversion::from_denom_to_asset_id(
						&fee_coin.denom.to_string(),
					);
				let amt = coin.amount.as_u256().low_u128();
				let mut is_flat_fee = false;
				let mut fee = match asset_id {
					Ok(a) => {
						let fee_asset_id = T::FlatFeeAssetId::get();
						let fee_asset_amount = T::FlatFeeAmount::get();
						is_flat_fee = true;

						T::FlatFeeConverter::get_flat_fee(a, fee_asset_id, fee_asset_amount)
							.unwrap_or_else(|| {
								// We have ensured that token amounts larger than the max value
								// for a u128 are rejected in the ics20 on_recv_packet callback
								// so we can multiply safely. Percent does Non-Overflowing
								// multiplication so this is infallible
								is_flat_fee = false;
								percent * amt
							})
					},
					Err(_) => percent * amt,
				};

				fee = fee.min(amt);
				fee_coin.amount = U256::from(fee).into();

				let signer_from = Signer::from_str(&from).map_err(|_| Error::<T>::Utf8Error)?;
				let account_id_from = <T as Config>::AccountIdConversion::try_from(signer_from)
					.map_err(|_| Error::<T>::OriginAddress)?;

				ctx.send_coins(&account_id_from, &fee_account, &fee_coin).map_err(|e| {
					log::debug!(target: "pallet_ibc", "[transfer]: error: {:?}", &e);
					Error::<T>::FailedSendFeeToAccount
				})?;

				// We modify the packet data to remove the fee so any other middleware has access to
				// the correct amount deposited in the receiver's account
				coin.amount = (coin.amount.as_u256() - U256::from(fee)).into();
				//found sequence that will used in Pallet::<T>::send_transfer function.
				let sequence = ctx
					.get_next_sequence_send(&(source_port.clone(), source_channel))
					.map_err(|_| Error::<T>::ChannelNotFound)?;
				//use this sequence as a key in storage map where sequence is key and fee is value
				let sequence: u64 = sequence.into();
				//we need this data in storage map because on_timeout_packet and
				// on_acknowledgement_packet use this data to refund fee in case of falure or clean
				// un in case of on_acknowledgement_packet success.
				SequenceFee::<T>::insert(sequence, fee);
				Self::deposit_event(Event::<T>::ChargingFeeOnTransferInitiated {
					sequence,
					from: from.clone().into(),
					to: to.clone().into(),
					amount: fee.into(),
					is_flat_fee,
					local_asset_id: T::IbcDenomToAssetIdConversion::from_denom_to_asset_id(
						&coin.denom.to_string(),
					)
					.ok(),
					ibc_denom: coin.denom.to_string().as_bytes().to_vec(),
					source_channel: source_channel.to_string().as_bytes().to_vec(),
					destination_channel: destination_channel.to_string().as_bytes().to_vec(),
				});
			};

			memo.as_ref()
				.map(|memo| {
					memo.validate().map_err(|e| {
						log::debug!(target: "pallet_ibc", "[transfer]: memo validation error: {}", e);
						Error::<T>::InvalidMemo
					})
				})
				.transpose()?;

			let msg = MsgTransfer {
				source_port,
				source_channel,
				token: coin.clone(),
				sender: Signer::from_str(&from).map_err(|_| Error::<T>::Utf8Error)?,
				receiver: Signer::from_str(&to).map_err(|_| Error::<T>::Utf8Error)?,
				timeout_height,
				timeout_timestamp,
				memo: memo.map(|memo| memo.to_string()).unwrap_or_default(),
			};

			let is_sender_source = is_sender_chain_source(
				msg.source_port.clone(),
				msg.source_channel,
				&msg.token.denom,
			);

			if is_sender_source {
				// Store escrow address
				let escrow_address =
					get_channel_escrow_address(&msg.source_port, msg.source_channel)
						.map_err(|_| Error::<T>::ChannelEscrowAddress)?;
				let account_id = T::AccountIdConversion::try_from(escrow_address)
					.map_err(|_| Error::<T>::ChannelEscrowAddress)?
					.into_account();
				let _ = EscrowAddresses::<T>::try_mutate::<_, &'static str, _>(|addresses| {
					if !addresses.contains(&account_id) {
						addresses.insert(account_id);
						Ok(())
					} else {
						Err("Address already exists")
					}
				});
			}

			Pallet::<T>::send_transfer(msg).map_err(|e| {
				log::warn!(target: "pallet_ibc", "[transfer]: error: {:?}", &e);
				use ibc_primitives::Error::*;
				match e {
					SendPacketError { .. } => Error::<T>::TransferSend,
					SendTransferError { .. } => Error::<T>::TransferSend,

					ReceivePacketError { .. } => Error::<T>::TransferProtocol,
					WriteAcknowledgementError { .. } => Error::<T>::TransferProtocol,
					AcknowledgementError { .. } => Error::<T>::TransferProtocol,
					TimeoutError { .. } => Error::<T>::TransferProtocol,

					TimestampOrHeightNotFound { .. } => Error::<T>::TransferInternals,
					ChannelOrPortError { .. } => Error::<T>::TransferInternals,
					ClientStateError { .. } => Error::<T>::TransferInternals,
					ConnectionIdError { .. } => Error::<T>::TransferInternals,
					ClientIdError { .. } => Error::<T>::TransferInternals,
					BindPortError { .. } => Error::<T>::TransferInternals,
					ChannelInitError { .. } => Error::<T>::TransferInternals,
					ChannelCloseError { .. } => Error::<T>::TransferInternals,

					DecodingError { .. } => Error::<T>::TransferSerde,
					ErrorDecodingPrefix => Error::<T>::TransferSerde,

					Other { .. } => Error::<T>::TransferOther,
				}
			})?;

			Self::deposit_event(Event::<T>::TokenTransferInitiated {
				from: from.as_bytes().to_vec(),
				to: to.as_bytes().to_vec(),
				amount,
				local_asset_id: T::IbcDenomToAssetIdConversion::from_denom_to_asset_id(
					&coin.denom.to_string(),
				)
				.ok(),
				ibc_denom: coin.denom.to_string().as_bytes().to_vec(),
				is_sender_source,
				source_channel: source_channel.to_string().as_bytes().to_vec(),
				destination_channel: destination_channel.to_string().as_bytes().to_vec(),
			});
			Ok(())
		}

		/// We write the consensus & client state under these predefined paths so that
		/// we can produce state proofs of the values to connected chains
		/// in order to execute client upgrades.
		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn upgrade_client(origin: OriginFor<T>, params: UpgradeParams) -> DispatchResult {
			<T as Config>::AdminOrigin::ensure_origin(origin)?;
			const CLIENT_STATE_UPGRADE_PATH: &[u8] = b"client-state-upgrade-path";
			const CONSENSUS_STATE_UPGRADE_PATH: &[u8] = b"consensus-state-upgrade-path";

			sp_io::storage::set(CLIENT_STATE_UPGRADE_PATH, &params.client_state);
			sp_io::storage::set(CONSENSUS_STATE_UPGRADE_PATH, &params.consensus_state);

			Self::deposit_event(Event::<T>::ClientUpgradeSet);

			Ok(())
		}

		/// Freeze a client at a specific height
		#[pallet::call_index(4)]
		#[pallet::weight(0)]
		pub fn freeze_client(
			origin: OriginFor<T>,
			client_id: Vec<u8>,
			height: u64,
		) -> DispatchResult {
			use ibc::core::ics02_client::client_state::ClientState;
			<T as Config>::FreezeOrigin::ensure_origin(origin)?;
			let client_id =
				client_id_from_bytes(client_id).map_err(|_| Error::<T>::DecodingError)?;
			let mut ctx = routing::Context::<T>::default();
			let client_state =
				ctx.client_state(&client_id).map_err(|_| Error::<T>::ClientStateNotFound)?;
			let frozen_state = match client_state {
				AnyClientState::GrandpaStandalone(grandpa) => {
					let latest_height = grandpa.latest_height();
					AnyClientState::wrap(
						&grandpa
							.with_frozen_height(Height::new(latest_height.revision_number, height))
							.map_err(|_| Error::<T>::ClientFreezeFailed)?,
					)
				},
				AnyClientState::Grandpa(grandpa) => {
					let latest_height = grandpa.latest_height();
					AnyClientState::wrap(
						&grandpa
							.with_frozen_height(Height::new(latest_height.revision_number, height))
							.map_err(|_| Error::<T>::ClientFreezeFailed)?,
					)
				},
				AnyClientState::Beefy(beefy) => {
					let latest_height = beefy.latest_height();
					AnyClientState::wrap(
						&beefy
							.with_frozen_height(Height::new(latest_height.revision_number, height))
							.map_err(|_| Error::<T>::ClientFreezeFailed)?,
					)
				},
				AnyClientState::Tendermint(tm) => {
					let latest_height = tm.latest_height();
					AnyClientState::wrap(
						&tm.with_frozen_height(Height::new(latest_height.revision_number, height))
							.map_err(|_| Error::<T>::ClientFreezeFailed)?,
					)
				},
				AnyClientState::Wasm(_) => return Err(Error::<T>::ClientFreezeFailed.into()),
				#[cfg(test)]
				AnyClientState::Mock(mut ms) => {
					ms.frozen_height =
						Some(Height::new(ms.latest_height().revision_number, height));
					AnyClientState::wrap(&ms)
				},
			}
			.ok_or(Error::<T>::ClientFreezeFailed)?;
			let revision_number = frozen_state.latest_height().revision_number;
			ctx.store_client_state(client_id.clone(), frozen_state)
				.map_err(|_| Error::<T>::ClientFreezeFailed)?;

			Self::deposit_event(Event::<T>::ClientFrozen {
				client_id: client_id.as_bytes().to_vec(),
				height,
				revision_number,
			});

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(0)]
		#[frame_support::transactional]
		/// Increase all IBC counters by 1. Used only in testing to ensure that
		/// relayer uses proper proper values for source/sink chains.
		pub fn increase_counters(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			#[cfg(not(feature = "testing"))]
			{
				Err(Error::<T>::AccessDenied.into())
			}
			#[cfg(feature = "testing")]
			{
				ChannelCounter::<T>::set(ChannelCounter::<T>::get() + 1);
				PacketCounter::<T>::set(PacketCounter::<T>::get() + 1);
				ClientCounter::<T>::set(ClientCounter::<T>::get() + 1);
				ConnectionCounter::<T>::set(ConnectionCounter::<T>::get() + 1);
				AcknowledgementCounter::<T>::set(AcknowledgementCounter::<T>::get() + 1);
				PacketReceiptCounter::<T>::set(PacketReceiptCounter::<T>::get() + 1);
				let port_id = PortId::transfer();
				let channel_id = ChannelId::new(ChannelCounter::<T>::get() as _);
				NextSequenceAck::<T>::insert(
					port_id.clone(),
					channel_id,
					NextSequenceAck::<T>::get(port_id.clone(), channel_id).unwrap_or_default() + 1,
				);
				NextSequenceRecv::<T>::insert(
					port_id.clone(),
					channel_id,
					NextSequenceRecv::<T>::get(port_id.clone(), channel_id).unwrap_or_default() + 1,
				);
				NextSequenceSend::<T>::insert(
					port_id.clone(),
					channel_id,
					NextSequenceSend::<T>::get(port_id.clone(), channel_id).unwrap_or_default() + 1,
				);
				Ok(())
			}
		}

		#[pallet::call_index(6)]
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

		#[pallet::call_index(7)]
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

		#[pallet::call_index(8)]
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn set_child_storage(
			origin: OriginFor<T>,
			key: Vec<u8>,
			value: Vec<u8>,
		) -> DispatchResult {
			<T as Config>::AdminOrigin::ensure_origin(origin)?;

			let concat_key = [T::PalletPrefix::get(), &key].concat();
			child::put(&ChildInfo::new_default(T::PalletPrefix::get()), &concat_key, &value);
			Self::deposit_event(Event::<T>::ChildStateUpdated);

			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(0)]
		#[frame_support::transactional]
		pub fn substitute_client_state(
			origin: OriginFor<T>,
			client_id: String,
			height: Height,
			client_state_bytes: Vec<u8>,
			consensus_state_bytes: Vec<u8>,
		) -> DispatchResult {
			<T as Config>::AdminOrigin::ensure_origin(origin)?;

			let client_id = ClientId::from_str(&client_id).map_err(|_| Error::<T>::Other)?;
			let client_state = AnyClientState::decode_vec(&client_state_bytes[..])
				.map_err(|_| Error::<T>::Other)?;
			let consensus_state = AnyConsensusState::decode_vec(&consensus_state_bytes[..])
				.map_err(|_| Error::<T>::Other)?;

			let mut ctx = Context::<T>::new();
			ctx.store_client_state(client_id.clone(), client_state)
				.map_err(|_| Error::<T>::Other)?;
			ctx.store_consensus_state(client_id.clone(), height, consensus_state)
				.map_err(|_| Error::<T>::Other)?;
			ctx.store_update_time(client_id.clone(), height, ctx.host_timestamp())
				.map_err(|_| Error::<T>::Other)?;
			ctx.store_update_height(client_id.clone(), height, ctx.host_height())
				.map_err(|_| Error::<T>::Other)?;

			Self::deposit_event(Event::<T>::ClientStateSubstituted {
				client_id: client_id.to_string(),
				height,
			});

			Ok(())
		}
	}
}

/// Result of the `DenomToAssetId::ibc_assets` function.
pub struct IbcAssets<AssetId> {
	/// List of IBC denoms.
	pub denoms: Vec<Vec<u8>>,
	/// Total number of IBC assets on the chain.
	pub total_count: u64,
	/// The next `AssetId` after the last item in the list.
	pub next_id: Option<AssetId>,
}

pub trait DenomToAssetId<T: Config> {
	type Error: Debug;

	/// Get the equivalent asset id for this ibc denom
	/// **Note**
	/// This function should create and register an asset with a valid metadata
	/// if an asset does not exist for this denom
	fn from_denom_to_asset_id(denom: &str) -> Result<T::AssetId, Self::Error>;

	/// Return full denom for given asset id
	fn from_asset_id_to_denom(id: T::AssetId) -> Option<String>;

	/// Returns `IbcAssets` containing a list of assets bound by `limit`.
	/// `start_key` is either an `AssetId` or an offset to start from.
	fn ibc_assets(start_key: Option<Either<T::AssetId, u32>>, limit: u64) -> IbcAssets<T::AssetId>;
}
