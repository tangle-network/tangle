//! With Polkadot Bridge Hub bridge configuration.

use crate::{
	xcm_config::{decode_bridge_message, XcmConfig},
	AccountId, Runtime, RuntimeEvent, RuntimeOrigin,
};

use bp_messages::{
	target_chain::{DispatchMessage, MessageDispatch},
	LaneId, MessageNonce,
};
use bp_parachains::SingleParaStoredHeaderDataBuilder;
use bp_runtime::{messages::MessageDispatchResult, ChainId, UnderlyingChainProvider};
use bridge_runtime_common::{
	messages::{
		source::{
			FromThisChainMaximalOutboundPayloadSize, FromThisChainMessageVerifier,
			TargetHeaderChainAdapter,
		},
		target::SourceHeaderChainAdapter,
		BridgedChainWithMessages, MessageBridge, ThisChainWithMessages,
	},
	messages_xcm_extension::{
		SenderAndLane, XcmAsPlainPayload, XcmBlobHauler, XcmBlobHaulerAdapter,
		XcmBlobMessageDispatch,
	},
};
use frame_support::{parameter_types, RuntimeDebug};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidity};
use sp_std::{marker::PhantomData, vec::Vec};
use xcm::prelude::*;
use xcm_builder::HaulBlobExporter;
use xcm_executor::XcmExecutor;

/// Lane that we are using to send and receive messages.
pub const XCM_LANE: LaneId = LaneId([0, 0, 0, 0]);

parameter_types! {
	/// A number of Polkadot mandatory headers that are accepted for free at every
	/// **this chain** block.
	pub const MaxFreePolkadotHeadersPerBlock: u32 = 4;
	/// A number of Polkadot header digests that we keep in the storage.
	pub const PolkadotHeadersToKeep: u32 = 1024;
	/// A name of parachains pallet at Pokadot.
	pub const AtPolkadotParasPalletName: &'static str = bp_polkadot::PARAS_PALLET_NAME;

	/// Chain identifier of Polkadot Bridge Hub.
	pub const BridgeHubPolkadotChainId: ChainId = bp_runtime::BRIDGE_HUB_POLKADOT_CHAIN_ID;
	/// A number of Polkadot Bridge Hub head digests that we keep in the storage.
	pub const BridgeHubPolkadotHeadsToKeep: u32 = 1024;
	/// A maximal size of Polkadot Bridge Hub head digest.
	pub const MaxPolkadotBrdgeHubHeadSize: u32 = bp_polkadot::MAX_NESTED_PARACHAIN_HEAD_DATA_SIZE;

	/// All active outbound lanes.
	pub const ActiveOutboundLanes: &'static [LaneId] = &[XCM_LANE];
	/// Maximal number of unrewarded relayer entries.
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce =
		bp_bridge_hub_polkadot::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	/// Maximal number of unconfirmed messages.
	pub const MaxUnconfirmedMessagesAtInboundLane: MessageNonce =
		bp_bridge_hub_polkadot::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;

	/// Sending chain location and lane used to communicate with Polkadot Bulletin chain.
	pub FromPolkadotBulletinToBridgeHubPolkadotRoute: SenderAndLane = SenderAndLane::new(
		Here.into(),
		XCM_LANE,
	);

	/// XCM message that is never sent to anyone.
	pub NeverSentMessage: Option<Xcm<()>> = None;
}

/// Bridged chain global consensus network.
pub struct BridgedNetwork;

impl sp_runtime::traits::Get<NetworkId> for BridgedNetwork {
	#[cfg(not(feature = "rococo"))]
	fn get() -> NetworkId {
		Polkadot
	}

	#[cfg(feature = "rococo")]
	fn get() -> NetworkId {
		Rococo
	}
}

/// An instance of `pallet_bridge_grandpa` used to bridge with Polkadot.
pub type WithPolkadotBridgeGrandpaInstance = ();
/// An instance of `pallet_bridge_parachains` used to bridge with Polkadot.
pub type WithPolkadotBridgeParachainsInstance = ();
/// An instance of `pallet_bridge_messages` used to bridge with Polkadot Bridge Hub.
pub type WithBridgeHubPolkadotMessagesInstance = ();

impl pallet_bridge_grandpa::Config<WithPolkadotBridgeGrandpaInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = crate::weights::bridge_polkadot_grandpa::WeightInfo<Runtime>;

	type BridgedChain = bp_polkadot::Polkadot;
	type MaxFreeMandatoryHeadersPerBlock = MaxFreePolkadotHeadersPerBlock;
	type HeadersToKeep = PolkadotHeadersToKeep;
}

impl pallet_bridge_parachains::Config<WithPolkadotBridgeParachainsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = crate::weights::bridge_polkadot_parachains::WeightInfo<Runtime>;

	type BridgesGrandpaPalletInstance = WithPolkadotBridgeGrandpaInstance;
	type ParasPalletName = AtPolkadotParasPalletName;
	type ParaStoredHeaderDataBuilder =
		SingleParaStoredHeaderDataBuilder<BridgeHubPolkadotOrRococo>;
	type HeadsToKeep = BridgeHubPolkadotHeadsToKeep;
	type MaxParaHeadDataSize = MaxPolkadotBrdgeHubHeadSize;
}

impl pallet_bridge_messages::Config<WithBridgeHubPolkadotMessagesInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = crate::weights::bridge_polkadot_messages::WeightInfo<Runtime>;

	type BridgedChainId = BridgeHubPolkadotChainId;
	type ActiveOutboundLanes = ActiveOutboundLanes;
	type MaxUnrewardedRelayerEntriesAtInboundLane = MaxUnrewardedRelayerEntriesAtInboundLane;
	type MaxUnconfirmedMessagesAtInboundLane = MaxUnconfirmedMessagesAtInboundLane;

	type MaximalOutboundPayloadSize =
		FromThisChainMaximalOutboundPayloadSize<WithBridgeHubPolkadotMessageBridge>;
	type OutboundPayload = XcmAsPlainPayload;

	type InboundPayload = XcmAsPlainPayload;
	type InboundRelayer = AccountId;
	type DeliveryPayments = ();

	type TargetHeaderChain = TargetHeaderChainAdapter<WithBridgeHubPolkadotMessageBridge>;
	type LaneMessageVerifier = FromThisChainMessageVerifier<WithBridgeHubPolkadotMessageBridge>;
	type DeliveryConfirmationPayments = ();

	type SourceHeaderChain = SourceHeaderChainAdapter<WithBridgeHubPolkadotMessageBridge>;
	type MessageDispatch = WithXcmWeightDispatcher<
		XcmBlobMessageDispatch<FromBridgeHubPolkadotBlobDispatcher, Self::WeightInfo, ()>,
	>;
	type OnMessagesDelivered = ();
}

/// Message bridge with Polkadot Bridge Hub.
pub struct WithBridgeHubPolkadotMessageBridge;

/// Polkadot Bridge Hub headers provider.
pub type BridgeHubPolkadotHeadersProvider = pallet_bridge_parachains::ParachainHeaders<
	Runtime,
	WithPolkadotBridgeParachainsInstance,
	BridgeHubPolkadotOrRococo,
>;

impl MessageBridge for WithBridgeHubPolkadotMessageBridge {
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bp_polkadot_bulletin::WITH_POLKADOT_BULLETIN_MESSAGES_PALLET_NAME;
	type ThisChain = PolkadotBulletinChain;
	type BridgedChain = BridgeHubPolkadot;
	type BridgedHeaderChain = BridgeHubPolkadotHeadersProvider;
}

/// BridgeHubPolkadot chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct BridgeHubPolkadot;

impl UnderlyingChainProvider for BridgeHubPolkadot {
	type Chain = BridgeHubPolkadotOrRococo;
}

impl BridgedChainWithMessages for BridgeHubPolkadot {}

/// BridgeHubRococo chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct PolkadotBulletinChain;

impl UnderlyingChainProvider for PolkadotBulletinChain {
	type Chain = bp_polkadot_bulletin::PolkadotBulletin;
}

impl ThisChainWithMessages for PolkadotBulletinChain {
	type RuntimeOrigin = RuntimeOrigin;
}

/// Message dispatcher that decodes XCM message and return its actual dispatch weight.
pub struct WithXcmWeightDispatcher<Inner>(PhantomData<Inner>);

impl<Inner> MessageDispatch for WithXcmWeightDispatcher<Inner>
where
	Inner: MessageDispatch<DispatchPayload = XcmAsPlainPayload>,
{
	type DispatchPayload = XcmAsPlainPayload;
	type DispatchLevelResult = Inner::DispatchLevelResult;

	fn is_active() -> bool {
		Inner::is_active()
	}

	fn dispatch_weight(message: &mut DispatchMessage<Self::DispatchPayload>) -> Weight {
		message
			.data
			.payload
			.as_ref()
			.map_err(drop)
			.and_then(|payload| decode_bridge_message(payload).map(|(_, xcm)| xcm).map_err(drop))
			.and_then(|xcm| xcm.try_into().map_err(drop))
			.and_then(|xcm| XcmExecutor::<XcmConfig>::prepare(xcm).map_err(drop))
			.map(|weighed_xcm| weighed_xcm.weight_of())
			.unwrap_or(Weight::zero())
	}

	fn dispatch(
		message: DispatchMessage<Self::DispatchPayload>,
	) -> MessageDispatchResult<Self::DispatchLevelResult> {
		let mut result = Inner::dispatch(message);
		// ensure that unspent is always zero here to avoid inconstency
		result.unspent_weight = Weight::zero();
		result
	}
}

/// Dispatches received XCM messages from the Polkadot Bridge Hub.
pub type FromBridgeHubPolkadotBlobDispatcher = crate::xcm_config::ImmediateXcmDispatcher;

/// Export XCM messages to be relayed to the Polkadot Bridge Hub chain.
pub type ToBridgeHubPolkadotHaulBlobExporter =
	HaulBlobExporter<XcmBlobHaulerAdapter<ToBridgeHubPolkadotXcmBlobHauler>, BridgedNetwork, ()>;

/// Messages pallet adapter to use by XCM blob hauler.
pub struct ToBridgeHubPolkadotXcmBlobHauler;
impl XcmBlobHauler for ToBridgeHubPolkadotXcmBlobHauler {
	type Runtime = Runtime;
	type MessagesInstance = WithBridgeHubPolkadotMessagesInstance;
	type SenderAndLane = FromPolkadotBulletinToBridgeHubPolkadotRoute;

	type ToSourceChainSender = ();
	type CongestedMessage = NeverSentMessage;
	type UncongestedMessage = NeverSentMessage;
}