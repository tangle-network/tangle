use crate::{
	ics20_fee::FlatFeeConverter,
	light_clients::{AnyClientState, AnyConsensusState},
	mock::*,
	routing::Context,
	Any, Config, ConsensusHeights, DenomToAssetId, Event, MultiAddress, Pallet,
	PendingRecvPacketSeqs, PendingSendPacketSeqs, Timeout, TransferParams, MODULE_ID,
};
use core::time::Duration;
use frame_support::{
	assert_noop, assert_ok,
	traits::{
		fungibles::{Inspect, Mutate},
		Currency, Hooks, Len,
	},
	weights::Weight,
};
use ibc::{
	applications::transfer::{
		acknowledgement::Acknowledgement as Ics20Acknowledgement, packet::PacketData, Coin,
		PrefixedDenom, VERSION,
	},
	core::{
		ics02_client::{
			client_state::ClientState,
			context::{ClientKeeper, ClientReader},
			height::Height,
			msgs::create_client::{MsgCreateAnyClient, TYPE_URL},
		},
		ics03_connection::{
			connection::{ConnectionEnd, Counterparty, State as ConnState},
			context::{ConnectionKeeper, ConnectionReader},
			msgs::conn_open_init,
			version::Version as ConnVersion,
		},
		ics04_channel::{
			channel::{ChannelEnd, Counterparty as ChanCounterParty, Order, State},
			context::{ChannelKeeper, ChannelReader},
			msgs::{
				acknowledgement::{Acknowledgement, MsgAcknowledgement},
				recv_packet::MsgRecvPacket,
			},
			packet::Packet,
			Version as ChanVersion,
		},
		ics23_commitment::commitment::CommitmentPrefix,
		ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
	},
	mock::{
		client_state::{MockClientState, MockConsensusState},
		header::{MockClientMessage, MockHeader},
	},
	proofs::Proofs,
	signer::Signer,
	tx_msg::Msg,
};
use ibc_primitives::{get_channel_escrow_address, HandlerMessage, IbcHandler};
use sp_core::Pair;
use sp_runtime::{
	traits::{Bounded, IdentifyAccount},
	AccountId32,
};
use std::{
	str::FromStr,
	time::{SystemTime, UNIX_EPOCH},
};
use tendermint_proto::Protobuf;

fn setup_client_and_consensus_state(port_id: PortId) {
	// Set up client state and consensus states
	let mock_client_state =
		MockClientState::new(MockClientMessage::from(MockHeader::new(Height::new(0, 1))));
	let mock_cs_state = MockConsensusState::new(MockHeader::new(Height::new(0, 1)));
	let client_id = ClientId::new(&mock_client_state.client_type(), 0).unwrap();
	let counterparty_client_id = ClientId::new(&mock_client_state.client_type(), 1).unwrap();
	let msg = MsgCreateAnyClient::<Context<Test>>::new(
		AnyClientState::Mock(mock_client_state),
		AnyConsensusState::Mock(mock_cs_state),
		Signer::from_str(MODULE_ID).unwrap(),
	)
	.unwrap()
	.encode_vec()
	.unwrap();
	let mut ctx = Context::<Test>::default();

	let msg = Any { type_url: TYPE_URL.to_string(), value: msg };
	assert_ok!(Ibc::deliver(RuntimeOrigin::signed(AccountId32::new([0; 32])), vec![msg]));

	let connection_id = ConnectionId::new(0);
	let commitment_prefix: CommitmentPrefix =
		<Test as Config>::PalletPrefix::get().to_vec().try_into().unwrap();
	let delay_period = core::time::Duration::from_nanos(0);
	let connection_counterparty =
		Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
	let connection_end = ConnectionEnd::new(
		ConnState::Open,
		client_id,
		connection_counterparty,
		vec![ConnVersion::default()],
		delay_period,
	);

	let counterparty = ChanCounterParty::new(port_id.clone(), Some(ChannelId::new(1)));
	let channel_end = ChannelEnd::new(
		State::Open,
		Order::Unordered,
		counterparty,
		vec![connection_id.clone()],
		ChanVersion::new(VERSION.to_string()),
	);
	let channel_id = ChannelId::new(0);
	ctx.store_connection(connection_id, &connection_end).unwrap();
	ctx.store_channel((port_id.clone(), channel_id), &channel_end).unwrap();
	ctx.store_next_sequence_send((port_id.clone(), channel_id), 1.into()).unwrap();
	ctx.store_next_sequence_recv((port_id, channel_id), 1.into()).unwrap()
}

// Create a client and initialize a connection
#[test]
fn initialize_connection() {
	new_test_ext().execute_with(|| {
		let mock_client_state =
			MockClientState::new(MockClientMessage::from(MockHeader::default()));
		let mock_cs_state = MockConsensusState::new(MockHeader::default());
		let client_id = ClientId::new(&mock_client_state.client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(&mock_client_state.client_type(), 1).unwrap();
		let msg = MsgCreateAnyClient::<Context<Test>>::new(
			AnyClientState::Mock(mock_client_state),
			AnyConsensusState::Mock(mock_cs_state),
			Signer::from_str(MODULE_ID).unwrap(),
		)
		.unwrap()
		.encode_vec()
		.unwrap();

		let commitment_prefix: CommitmentPrefix =
			<Test as Config>::PalletPrefix::get().to_vec().try_into().unwrap();

		let msg = Any { type_url: TYPE_URL.to_string(), value: msg };

		assert_ok!(Ibc::deliver(RuntimeOrigin::signed(AccountId32::new([0; 32])), vec![msg]));

		let value = conn_open_init::MsgConnectionOpenInit {
			client_id,
			counterparty: Counterparty::new(
				counterparty_client_id,
				Some(ConnectionId::new(1)),
				commitment_prefix,
			),
			version: Some(ConnVersion::default()),
			delay_period: Duration::from_nanos(1000),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		};

		let msg = Any {
			type_url: conn_open_init::TYPE_URL.to_string(),
			value: value.encode_vec().unwrap(),
		};

		assert_ok!(Ibc::deliver(RuntimeOrigin::signed(AccountId32::new([0; 32])), vec![msg]));
	})
}

// try to initialize a connection below the MinimumConnectionDelay
#[test]
fn initialize_connection_with_low_delay() {
	new_test_ext().execute_with(|| {
		let ctx = Context::<Test>::new();

		let mock_client_state =
			MockClientState::new(MockClientMessage::from(MockHeader::default()));
		let mock_cs_state = MockConsensusState::new(MockHeader::default());
		let client_id = ClientId::new(&mock_client_state.client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(&mock_client_state.client_type(), 1).unwrap();
		let msg = MsgCreateAnyClient::<Context<Test>>::new(
			AnyClientState::Mock(mock_client_state),
			AnyConsensusState::Mock(mock_cs_state),
			Signer::from_str(MODULE_ID).unwrap(),
		)
		.unwrap()
		.encode_vec()
		.unwrap();

		let commitment_prefix: CommitmentPrefix =
			<Test as Config>::PalletPrefix::get().to_vec().try_into().unwrap();

		let msg = Any { type_url: TYPE_URL.to_string(), value: msg };

		assert_ok!(Ibc::deliver(RuntimeOrigin::signed(AccountId32::new([0; 32])), vec![msg]));

		let value = conn_open_init::MsgConnectionOpenInit {
			client_id,
			counterparty: Counterparty::new(
				counterparty_client_id,
				Some(ConnectionId::new(1)),
				commitment_prefix,
			),
			version: Some(ConnVersion::default()),
			delay_period: Duration::from_nanos(900),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		};

		let msg = Any {
			type_url: conn_open_init::TYPE_URL.to_string(),
			value: value.encode_vec().unwrap(),
		};

		Ibc::deliver(RuntimeOrigin::signed(AccountId32::new([0; 32])), vec![msg]).unwrap();

		let result = ConnectionReader::connection_end(&ctx, &ConnectionId::new(0));

		assert!(result.is_err())
	})
}

const MILLIS: u128 = 1000000;
#[test]
fn send_transfer() {
	let mut ext = new_test_ext();
	let balance = 100000 * MILLIS;
	ext.execute_with(|| {
		let pair = sp_core::sr25519::Pair::from_seed(b"12345678901234567890123456789012");
		let ss58_address =
			ibc_primitives::runtime_interface::account_id_to_ss58(pair.public().0, 49);
		setup_client_and_consensus_state(PortId::transfer());
		let asset_id =
			<<Test as Config>::IbcDenomToAssetIdConversion as DenomToAssetId<Test>>::from_denom_to_asset_id(
				"TNT",
			)
			.unwrap();
		let _ = <<Test as Config>::NativeCurrency as Currency<
			<Test as frame_system::Config>::AccountId,
		>>::deposit_creating(&AccountId32::new([0; 32]), balance);

		let timeout = Timeout::Offset { timestamp: Some(1000), height: Some(5) };

		Ibc::transfer(
			RuntimeOrigin::signed(AccountId32::new([0; 32])),
			TransferParams {
				to: MultiAddress::Raw(ss58_address.as_bytes().to_vec()),
				source_channel: 0,
				timeout,
			},
			asset_id,
			balance,
			None,
		)
		.unwrap();
	});

	ext.persist_offchain_overlay();

	ext.execute_with(|| {
		let channel_id = ChannelId::new(0);
		let port_id = PortId::transfer();
		let packet_info = Pallet::<Test>::get_send_packet_info(
			channel_id.to_string().as_bytes().to_vec(),
			port_id.as_bytes().to_vec(),
			vec![1],
		)
		.unwrap()
		.get(0)
		.unwrap()
		.clone();
		assert!(!packet_info.data.is_empty());

		let packet_data: PacketData = serde_json::from_slice(packet_info.data.as_slice()).unwrap();
		let send_amount = packet_data.token.amount.as_u256().as_u128();
		let fee = <Test as crate::ics20_fee::Config>::ServiceChargeIn::get() * balance;
		assert_eq!(send_amount, balance - fee);
	})
}

#[test]
fn send_transfer_with_invalid_memo() {
	let mut ext = new_test_ext();
	let balance = 100000 * MILLIS;
	ext.execute_with(|| {
		let pair = sp_core::sr25519::Pair::from_seed(b"12345678901234567890123456789012");
		let ss58_address =
			ibc_primitives::runtime_interface::account_id_to_ss58(pair.public().0, 49);
		setup_client_and_consensus_state(PortId::transfer());
		let asset_id =
			<<Test as Config>::IbcDenomToAssetIdConversion as DenomToAssetId<Test>>::from_denom_to_asset_id(
				"TNT",
			)
			.unwrap();
		let _ = <<Test as Config>::NativeCurrency as Currency<
			<Test as frame_system::Config>::AccountId,
		>>::deposit_creating(&AccountId32::new([0; 32]), balance);

		let timeout = Timeout::Offset { timestamp: Some(1000), height: Some(5) };

		let ctx = Context::<Test>::default();
		let channel_end = ctx
			.channel_end(&(PortId::transfer(), ChannelId::new(0)))
			.expect("expect source_channel unwrap");
		let destination_channel = channel_end.counterparty().channel_id.unwrap();
		Ibc::add_channels_to_feeless_channel_list(
			RuntimeOrigin::root(),
			0,
			destination_channel.sequence(),
		)
		.expect("expect add channels to feeless list");

		let result = Ibc::transfer(
			RuntimeOrigin::signed(AccountId32::new([0; 32])),
			TransferParams {
				to: MultiAddress::Raw(ss58_address.as_bytes().to_vec()),
				source_channel: 0,
				timeout: timeout.clone(),
			},
			asset_id,
			balance,
			Some(RawMemo("{}".to_string())),
		);
		assert_ok!(result);

		let result = Ibc::transfer(
			RuntimeOrigin::signed(AccountId32::new([0; 32])),
			TransferParams {
				to: MultiAddress::Raw(ss58_address.as_bytes().to_vec()),
				source_channel: 0,
				timeout: timeout.clone(),
			},
			asset_id,
			balance,
			Some(RawMemo("invalid memo".to_string())),
		);
		assert_noop!(result, crate::Error::<Test>::InvalidMemo);
	});
}

#[test]
fn send_transfer_no_fee_feeless_channels() {
	let mut ext = new_test_ext();
	let balance = 100000 * MILLIS;
	ext.execute_with(|| {
		let pair = sp_core::sr25519::Pair::from_seed(b"12345678901234567890123456789012");
		let ss58_address =
			ibc_primitives::runtime_interface::account_id_to_ss58(pair.public().0, 49);
		setup_client_and_consensus_state(PortId::transfer());
		let asset_id =
			<<Test as Config>::IbcDenomToAssetIdConversion as DenomToAssetId<Test>>::from_denom_to_asset_id(
				"TNT",
			)
			.unwrap();
		let _ = <<Test as Config>::NativeCurrency as Currency<
			<Test as frame_system::Config>::AccountId,
		>>::deposit_creating(&AccountId32::new([0; 32]), balance);

		let timeout = Timeout::Offset { timestamp: Some(1000), height: Some(5) };

		let ctx = Context::<Test>::default();
		let channel_end = ctx
			.channel_end(&(PortId::transfer(), ChannelId::new(0)))
			.expect("expect source_channel unwrap");
		let destination_channel = channel_end.counterparty().channel_id.unwrap();
		//feeless channels
		Ibc::add_channels_to_feeless_channel_list(
			RuntimeOrigin::root(),
			0,
			destination_channel.sequence(),
		)
		.expect("expect add channels to feeless list");

		Ibc::transfer(
			RuntimeOrigin::signed(AccountId32::new([0; 32])),
			TransferParams {
				to: MultiAddress::Raw(ss58_address.as_bytes().to_vec()),
				source_channel: 0,
				timeout,
			},
			asset_id,
			balance,
			None,
		)
		.unwrap();
	});

	ext.persist_offchain_overlay();

	ext.execute_with(|| {
		let channel_id = ChannelId::new(0);
		let port_id = PortId::transfer();
		let packet_info = Pallet::<Test>::get_send_packet_info(
			channel_id.to_string().as_bytes().to_vec(),
			port_id.as_bytes().to_vec(),
			vec![1],
		)
		.unwrap()
		.get(0)
		.unwrap()
		.clone();
		assert!(!packet_info.data.is_empty());

		let packet_data: PacketData = serde_json::from_slice(packet_info.data.as_slice()).unwrap();
		let send_amount = packet_data.token.amount.as_u256().as_u128();
		assert_eq!(send_amount, balance);
	})
}

#[test]
fn on_deliver_ics20_recv_packet() {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		// Create  a new account
		let pair = sp_core::sr25519::Pair::from_seed(b"12345678901234567890123456789012");
		let ss58_address =
			ibc_primitives::runtime_interface::account_id_to_ss58(pair.public().0, 49);
		frame_system::Pallet::<Test>::set_block_number(1u64);
		let asset_id =
			<<Test as Config>::IbcDenomToAssetIdConversion as DenomToAssetId<Test>>::from_denom_to_asset_id(
				"TNT",
			)
			.unwrap();
		setup_client_and_consensus_state(PortId::transfer());

		let channel_id = ChannelId::new(0);
		let balance = 100000 * MILLIS;

		// We are simulating a transfer back to the source chain

		let denom = "transfer/channel-1/TNT";
		let channel_escrow_address =
			get_channel_escrow_address(&PortId::transfer(), channel_id).unwrap();
		let channel_escrow_address =
			<Test as Config>::AccountIdConversion::try_from(channel_escrow_address)
				.map_err(|_| ())
				.unwrap();
		let channel_escrow_address = channel_escrow_address.into_account();

		// Endow escrow address with tokens
		let _ = <<Test as Config>::NativeCurrency as Currency<
			<Test as frame_system::Config>::AccountId,
		>>::deposit_creating(&channel_escrow_address, balance);

		let prefixed_denom = PrefixedDenom::from_str(denom).unwrap();
		let amt = 1000 * MILLIS;
		println!("Transferred Amount {amt}");
		let coin = Coin {
			denom: prefixed_denom,
			amount: ibc::applications::transfer::Amount::from_str(&format!("{amt:?}")).unwrap(),
		};
		let packet_data = PacketData {
			token: coin,
			sender: Signer::from_str("alice").unwrap(),
			receiver: Signer::from_str(&ss58_address).unwrap(),
			memo: "".to_string(),
		};

		let data = serde_json::to_vec(&packet_data).unwrap();
		let time_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
		let packet = Packet {
			sequence: 1u64.into(),
			source_port: PortId::transfer(),
			source_channel: ChannelId::new(1),
			destination_port: PortId::transfer(),
			destination_channel: ChannelId::new(0),
			data,
			timeout_height: Height::new(2000, 5),
			timeout_timestamp: ibc::timestamp::Timestamp::from_nanoseconds(
				time_now as u64 + 10000000,
			)
			.unwrap(),
		};

		let msg = MsgRecvPacket {
			packet,
			proofs: Proofs::new(
				vec![0u8; 32].try_into().unwrap(),
				None,
				None,
				None,
				Height::new(0, 1),
			)
			.unwrap(),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		};

		let msg = Any { type_url: msg.type_url(), value: msg.encode_vec().unwrap() };

		let account_data = Assets::balance(asset_id, AccountId32::new(pair.public().0));
		// Assert account balance before transfer
		assert_eq!(account_data, 0);
		Ibc::deliver(RuntimeOrigin::signed(AccountId32::new([0; 32])), vec![msg]).unwrap();

		let balance = <<Test as Config>::NativeCurrency as Currency<
			<Test as frame_system::Config>::AccountId,
		>>::free_balance(&AccountId32::new(pair.public().0));

		let pallet_balance =
			<<Test as Config>::NativeCurrency as Currency<
				<Test as frame_system::Config>::AccountId,
			>>::free_balance(&<Test as crate::Config>::FeeAccount::get().into_account());
		let fee = <Test as crate::ics20_fee::Config>::ServiceChargeIn::get() * amt;
		assert_eq!(balance, amt - fee);
		assert_eq!(pallet_balance, fee)
	})
}

#[test]
fn on_deliver_ics20_recv_packet_incorrect_memo() {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		let incorrect_memo = "Incorrect memo".to_string();
		// Create  a new account
		let pair = sp_core::sr25519::Pair::from_seed(b"12345678901234567890123456789012");
		let reciever = AccountId32::new(pair.public().0);
		let ss58_address =
			ibc_primitives::runtime_interface::account_id_to_ss58(pair.public().0, 49);
		frame_system::Pallet::<Test>::set_block_number(1u64);
		let asset_id =
			<<Test as Config>::IbcDenomToAssetIdConversion as DenomToAssetId<Test>>::from_denom_to_asset_id(
				&"TNT".to_string(),
			)
			.unwrap();
		setup_client_and_consensus_state(PortId::transfer());

		let channel_id = ChannelId::new(0);
		let balance = 100000 * MILLIS;

		// We are simulating a transfer back to the source chain

		let denom = "transfer/channel-1/TNT";
		let channel_escrow_address =
			get_channel_escrow_address(&PortId::transfer(), channel_id).unwrap();
		let channel_escrow_address =
			<Test as Config>::AccountIdConversion::try_from(channel_escrow_address)
				.map_err(|_| ())
				.unwrap();
		let channel_escrow_address = channel_escrow_address.into_account();

		// Endow escrow address with tokens
		let _ = <<Test as Config>::NativeCurrency as Currency<
			<Test as frame_system::Config>::AccountId,
		>>::deposit_creating(&channel_escrow_address, balance);

		let prefixed_denom = PrefixedDenom::from_str(denom).unwrap();
		let amt = 1000 * MILLIS;
		println!("Transferred Amount {}", amt);
		let coin = Coin {
			denom: prefixed_denom,
			amount: ibc::applications::transfer::Amount::from_str(&format!("{:?}", amt)).unwrap(),
		};
		let packet_data = PacketData {
			token: coin,
			sender: Signer::from_str("alice").unwrap(),
			receiver: Signer::from_str(&ss58_address).unwrap(),
			memo: incorrect_memo.clone(),
		};

		let time_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();

		let data = serde_json::to_vec(&packet_data).unwrap();
		let packet = Packet {
			sequence: 1u64.into(),
			source_port: PortId::transfer(),
			source_channel: ChannelId::new(1),
			destination_port: PortId::transfer(),
			destination_channel: ChannelId::new(0),
			data,
			timeout_height: Height::new(2000, 5),
			timeout_timestamp: ibc::timestamp::Timestamp::from_nanoseconds(
				time_now as u64 + 10000000,
			)
			.unwrap(),
		};

		let msg = MsgRecvPacket {
			packet,
			proofs: Proofs::new(
				vec![0u8; 32].try_into().unwrap(),
				None,
				None,
				None,
				Height::new(0, 1),
			)
			.unwrap(),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		};

		let msg = Any { type_url: msg.type_url(), value: msg.encode_vec().unwrap() };

		let account_data = Assets::balance(asset_id, AccountId32::new(pair.public().0));
		// Assert account balance before transfer
		assert_eq!(account_data, 0);
		Ibc::deliver(RuntimeOrigin::signed(AccountId32::new([0; 32])), vec![msg]).unwrap();

		let balance = <<Test as Config>::NativeCurrency as Currency<
			<Test as frame_system::Config>::AccountId,
		>>::free_balance(&reciever);

		let pallet_balance =
			<<Test as Config>::NativeCurrency as Currency<
				<Test as frame_system::Config>::AccountId,
			>>::free_balance(&<Test as crate::Config>::FeeAccount::get().into_account());
		let fee = <Test as crate::ics20_fee::Config>::ServiceChargeIn::get() * amt;
		assert_eq!(balance, amt - fee);
		assert_eq!(pallet_balance, fee);

		// let test_event = TestEvent::PalletIbc($event);
		assert_eq!(
			System::events()
				.iter()
				.filter(|a| {
					if let RuntimeEvent::Ibc(ibc_event) = &a.event {
						if let Event::<Test>::ExecuteMemoStarted { memo, .. } = ibc_event {
							return memo.as_ref().map_or(false, |m| m == &incorrect_memo);
						}
					}
					false
				})
				.count(),
			1
		);

		assert_eq!(
			System::events()
				.iter()
				.filter(|a| {
					if let RuntimeEvent::Ibc(ibc_event) = &a.event {
						if let Event::<Test>::ExecuteMemoIbcTokenTransferFailedWithReason {
							memo,
							reason,
							from,
						} = ibc_event
						{
							return memo == &incorrect_memo && *reason == 0 && from == &reciever;
						}
					}
					false
				})
				.count(),
			1
		);
	})
}

#[test]
fn on_deliver_ics20_recv_packet_with_flat_fee() {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		// Create  a new account
		let pair = sp_core::sr25519::Pair::from_seed(b"12345678901234567890123456789012");
		let ss58_address =
			ibc_primitives::runtime_interface::account_id_to_ss58(pair.public().0, 49);
		frame_system::Pallet::<Test>::set_block_number(1u64);
		let asset_id =
			<<Test as Config>::IbcDenomToAssetIdConversion as DenomToAssetId<Test>>::from_denom_to_asset_id(
				"TNTFLATFEE",
			)
			.unwrap();
		setup_client_and_consensus_state(PortId::transfer());

		let channel_id = ChannelId::new(0);
		let balance = 100000 * MILLIS;

		// We are simulating a transfer back to the source chain

		let denom = "transfer/channel-1/TNTFLATFEE";
		let channel_escrow_address =
			get_channel_escrow_address(&PortId::transfer(), channel_id).unwrap();
		let channel_escrow_address =
			<Test as Config>::AccountIdConversion::try_from(channel_escrow_address)
				.map_err(|_| ())
				.unwrap();
		let channel_escrow_address = channel_escrow_address.into_account();

		// Endow escrow address with tokens
		<<Test as Config>::Fungibles as Mutate<
			<Test as frame_system::Config>::AccountId,
		>>::mint_into(asset_id, &channel_escrow_address, balance)
		.unwrap();

		let prefixed_denom = PrefixedDenom::from_str(denom).unwrap();
		let amt = 1000 * MILLIS;
		println!("Transferred Amount {amt}");
		let coin = Coin {
			denom: prefixed_denom,
			amount: ibc::applications::transfer::Amount::from_str(&format!("{amt:?}")).unwrap(),
		};
		let packet_data = PacketData {
			token: coin,
			sender: Signer::from_str("alice").unwrap(),
			receiver: Signer::from_str(&ss58_address).unwrap(),
			memo: "".to_string(),
		};

		let time_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
		let data = serde_json::to_vec(&packet_data).unwrap();
		let packet = Packet {
			sequence: 1u64.into(),
			source_port: PortId::transfer(),
			source_channel: ChannelId::new(1),
			destination_port: PortId::transfer(),
			destination_channel: ChannelId::new(0),
			data,
			timeout_height: Height::new(2000, 5),
			timeout_timestamp: ibc::timestamp::Timestamp::from_nanoseconds(
				time_now as u64 + 10000000,
			)
			.unwrap(),
		};

		let msg = MsgRecvPacket {
			packet,
			proofs: Proofs::new(
				vec![0u8; 32].try_into().unwrap(),
				None,
				None,
				None,
				Height::new(0, 1),
			)
			.unwrap(),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		};

		let msg = Any { type_url: msg.type_url(), value: msg.encode_vec().unwrap() };

		let account_data = Assets::balance(asset_id, AccountId32::new(pair.public().0));
		// Assert account balance before transfer
		assert_eq!(account_data, 0);
		Ibc::deliver(RuntimeOrigin::signed(AccountId32::new([0; 32])), vec![msg]).unwrap();

		let balance =
			<Assets as Inspect<AccountId>>::balance(asset_id, &AccountId32::new(pair.public().0));
		let pallet_balance = <Assets as Inspect<AccountId>>::balance(
			asset_id,
			&<Test as crate::Config>::FeeAccount::get().into_account(),
		);
		let fee_asset_id = <Test as crate::Config>::FlatFeeAssetId::get();
		let fee =
			<Test as crate::Config>::FlatFeeConverter::get_flat_fee(asset_id, fee_asset_id, amt)
				.unwrap_or_default();
		assert_eq!(balance, amt - fee);
		assert_eq!(pallet_balance, fee)
	})
}

#[test]
fn on_ack_transfer_with_custom_success_result() {
	let mut ext = new_test_ext();
	let _ = env_logger::try_init();
	ext.execute_with(|| {
		// Create  a new account
		let pair = sp_core::sr25519::Pair::from_seed(b"12345678901234567890123456789012");
		frame_system::Pallet::<Test>::set_block_number(1u64);
		let asset_id =
			<<Test as Config>::IbcDenomToAssetIdConversion as DenomToAssetId<Test>>::from_denom_to_asset_id(
				"TNTFLATFEE",
			)
			.unwrap();
		setup_client_and_consensus_state(PortId::transfer());

		let channel_id = ChannelId::new(0);
		let balance = 100000 * MILLIS;

		// We are simulating a transfer back to the source chain
		let acc = AccountId32::new(pair.public().0);

		// Endow escrow address with tokens
		<<Test as Config>::Fungibles as Mutate<
			<Test as frame_system::Config>::AccountId,
		>>::mint_into(asset_id, &acc, balance)
		.unwrap();

		let init_balance = <Assets as Inspect<AccountId>>::balance(asset_id, &acc);
		let amt = 1000 * MILLIS;
		println!("Transferred Amount {amt}");

		assert_ok!(Ibc::transfer(
			RuntimeOrigin::signed(acc.clone()),
			TransferParams {
				to: MultiAddress::Raw(vec![42; 10]),
				source_channel: channel_id.sequence(),
				timeout: Timeout::Offset { timestamp: None, height: Some(1) },
			},
			asset_id,
			amt,
			None,
		));

		let packet_info = Ibc::get_send_packet_info(
			channel_id.to_string().as_bytes().to_vec(),
			PortId::transfer().as_bytes().to_vec(),
			vec![1],
		)
		.unwrap()
		.get(0)
		.unwrap()
		.clone();
		let packet = Packet::from(packet_info);

		let msg = MsgAcknowledgement {
			packet,
			acknowledgement: Acknowledgement::from_bytes(
				Ics20Acknowledgement::Result(r#"{"contract_result":1}"#.to_string())
					.to_string()
					.into_bytes(),
			),
			proofs: Proofs::new(
				vec![0u8; 32].try_into().unwrap(),
				None,
				None,
				None,
				Height::new(0, 1),
			)
			.unwrap(),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		};
		let msg = Any { type_url: msg.type_url(), value: msg.encode_vec().unwrap() };
		Ibc::deliver(RuntimeOrigin::signed(AccountId32::new([0; 32])), vec![msg]).unwrap();

		let balance = <Assets as Inspect<AccountId>>::balance(asset_id, &acc);
		assert_eq!(balance, init_balance - amt);
	})
}

#[test]
fn on_deliver_ics20_recv_packet_transfered_amount_less_then_flat_fee() {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		// Create  a new account
		let pair = sp_core::sr25519::Pair::from_seed(b"12345678901234567890123456789012");
		let ss58_address =
			ibc_primitives::runtime_interface::account_id_to_ss58(pair.public().0, 49);
		frame_system::Pallet::<Test>::set_block_number(1u64);
		let asset_id =
			<<Test as Config>::IbcDenomToAssetIdConversion as DenomToAssetId<Test>>::from_denom_to_asset_id(
				"TNTFLATFEE",
			)
			.unwrap();
		setup_client_and_consensus_state(PortId::transfer());

		let channel_id = ChannelId::new(0);
		let balance = 100000 * MILLIS;

		// We are simulating a transfer back to the source chain

		let denom = "transfer/channel-1/TNTFLATFEE";
		let channel_escrow_address =
			get_channel_escrow_address(&PortId::transfer(), channel_id).unwrap();
		let channel_escrow_address =
			<Test as Config>::AccountIdConversion::try_from(channel_escrow_address)
				.map_err(|_| ())
				.unwrap();
		let channel_escrow_address = channel_escrow_address.into_account();

		// Endow escrow address with tokens
		<<Test as Config>::Fungibles as Mutate<
			<Test as frame_system::Config>::AccountId,
		>>::mint_into(asset_id, &channel_escrow_address, balance)
		.unwrap();

		let prefixed_denom = PrefixedDenom::from_str(denom).unwrap();
		let usdt_fee_amount = 1000 * MILLIS;
		let fee_asset_id = <Test as crate::Config>::FlatFeeAssetId::get();
		let amt = <Test as crate::Config>::FlatFeeConverter::get_flat_fee(
			asset_id,
			fee_asset_id,
			usdt_fee_amount,
		)
		.unwrap_or_default();
		println!("Transferred Amount {amt}");
		let coin = Coin {
			denom: prefixed_denom,
			amount: ibc::applications::transfer::Amount::from_str(&format!("{amt:?}")).unwrap(),
		};
		let packet_data = PacketData {
			token: coin,
			sender: Signer::from_str("alice").unwrap(),
			receiver: Signer::from_str(&ss58_address).unwrap(),
			memo: "".to_string(),
		};

		let time_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
		let data = serde_json::to_vec(&packet_data).unwrap();
		let packet = Packet {
			sequence: 1u64.into(),
			source_port: PortId::transfer(),
			source_channel: ChannelId::new(1),
			destination_port: PortId::transfer(),
			destination_channel: ChannelId::new(0),
			data,
			timeout_height: Height::new(2000, 5),
			timeout_timestamp: ibc::timestamp::Timestamp::from_nanoseconds(
				time_now as u64 + 10000000,
			)
			.unwrap(),
		};

		let msg = MsgRecvPacket {
			packet,
			proofs: Proofs::new(
				vec![0u8; 32].try_into().unwrap(),
				None,
				None,
				None,
				Height::new(0, 1),
			)
			.unwrap(),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		};

		let msg = Any { type_url: msg.type_url(), value: msg.encode_vec().unwrap() };

		let account_data = Assets::balance(asset_id, AccountId32::new(pair.public().0));
		// Assert account balance before transfer
		assert_eq!(account_data, 0);
		Ibc::deliver(RuntimeOrigin::signed(AccountId32::new([0; 32])), vec![msg]).unwrap();

		let balance =
			<Assets as Inspect<AccountId>>::balance(asset_id, &AccountId32::new(pair.public().0));
		let pallet_balance = <Assets as Inspect<AccountId>>::balance(
			asset_id,
			&<Test as crate::Config>::FeeAccount::get().into_account(),
		);
		let fee = <Test as crate::Config>::FlatFeeConverter::get_flat_fee(
			asset_id,
			fee_asset_id,
			usdt_fee_amount,
		)
		.unwrap_or_default();
		let actual_fee = fee.min(amt);
		assert_eq!(balance, 0);
		assert_eq!(balance, amt - actual_fee);
		assert_eq!(pallet_balance, actual_fee);
	})
}

#[test]
fn on_deliver_ics20_recv_packet_should_not_double_spend() {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		// Create  a new account
		let pair = sp_core::sr25519::Pair::from_seed(b"12345678901234567890123456789012");
		let ss58_address =
			ibc_primitives::runtime_interface::account_id_to_ss58(pair.public().0, 49);
		frame_system::Pallet::<Test>::set_block_number(1u64);
		setup_client_and_consensus_state(PortId::transfer());

		let channel_id = ChannelId::new(0);
		let balance = 100000 * MILLIS;

		// We are simulating a transfer back to the source chain

		let denom = "transfer/channel-1/TNT";
		let channel_escrow_address =
			get_channel_escrow_address(&PortId::transfer(), channel_id).unwrap();
		let channel_escrow_address =
			<Test as Config>::AccountIdConversion::try_from(channel_escrow_address)
				.map_err(|_| ())
				.unwrap();
		let channel_escrow_address = channel_escrow_address.into_account();

		// Endow escrow address with tokens
		let _ = <<Test as Config>::NativeCurrency as Currency<
			<Test as frame_system::Config>::AccountId,
		>>::deposit_creating(&channel_escrow_address, balance);

		let prefixed_denom = PrefixedDenom::from_str(denom).unwrap();
		let amt = MILLIS / 100;
		println!("Transferred Amount {amt}");
		let coin = Coin {
			denom: prefixed_denom,
			amount: ibc::applications::transfer::Amount::from_str(&format!("{amt:?}")).unwrap(),
		};
		let packet_data = PacketData {
			token: coin,
			sender: Signer::from_str("alice").unwrap(),
			receiver: Signer::from_str(&ss58_address).unwrap(),
			memo: "".to_string(),
		};

		let time_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
		let data = serde_json::to_vec(&packet_data).unwrap();
		let packet = Packet {
			sequence: 1u64.into(),
			source_port: PortId::transfer(),
			source_channel: ChannelId::new(1),
			destination_port: PortId::transfer(),
			destination_channel: ChannelId::new(0),
			data,
			timeout_height: Height::new(2000, 5),
			timeout_timestamp: ibc::timestamp::Timestamp::from_nanoseconds(
				time_now as u64 + 10000000,
			)
			.unwrap(),
		};

		let msg = MsgRecvPacket {
			packet,
			proofs: Proofs::new(
				vec![0u8; 32].try_into().unwrap(),
				None,
				None,
				None,
				Height::new(0, 1),
			)
			.unwrap(),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		};

		let msg = Any { type_url: msg.type_url(), value: msg.encode_vec().unwrap() };
		let fee_amt = <Test as crate::ics20_fee::Config>::ServiceChargeIn::get() * amt;

		let account_balance = <<Test as Config>::NativeCurrency as Currency<
			<Test as frame_system::Config>::AccountId,
		>>::free_balance(&AccountId32::new(pair.public().0));
		// Assert account balance before transfer
		assert_eq!(account_balance, 0);
		Ibc::deliver(RuntimeOrigin::signed(AccountId32::new([0; 32])), vec![msg.clone()]).unwrap();

		let account_balance = <<Test as Config>::NativeCurrency as Currency<
			<Test as frame_system::Config>::AccountId,
		>>::free_balance(&AccountId32::new(pair.public().0));
		// Assert account balance after transfer
		assert_eq!(account_balance, amt - fee_amt);

		let balance = <<Test as Config>::NativeCurrency as Currency<
			<Test as frame_system::Config>::AccountId,
		>>::free_balance(&AccountId32::new(pair.public().0));
		let pallet_balance =
			<<Test as Config>::NativeCurrency as Currency<
				<Test as frame_system::Config>::AccountId,
			>>::free_balance(&<Test as crate::Config>::FeeAccount::get().into_account());
		// fee is less than ExistentialDeposit, so it is not deducted
		assert_eq!(balance, amt - fee_amt);
		assert_eq!(pallet_balance, fee_amt);

		// try sending the same packet again
		Ibc::deliver(RuntimeOrigin::signed(AccountId32::new([0; 32])), vec![msg]).unwrap();

		let account_data = <<Test as Config>::NativeCurrency as Currency<
			<Test as frame_system::Config>::AccountId,
		>>::free_balance(&AccountId32::new(pair.public().0));
		// Assert account balance after transfer
		assert_eq!(account_data, amt - fee_amt);

		let balance = <<Test as Config>::NativeCurrency as Currency<
			<Test as frame_system::Config>::AccountId,
		>>::free_balance(&AccountId32::new(pair.public().0));
		let pallet_balance =
			<<Test as Config>::NativeCurrency as Currency<
				<Test as frame_system::Config>::AccountId,
			>>::free_balance(&<Test as crate::Config>::FeeAccount::get().into_account());
		// fee is less than ExistentialDeposit, so it is not deducted
		assert_eq!(balance, amt - fee_amt);
		assert_eq!(pallet_balance, fee_amt);
	})
}

#[test]
fn should_fetch_recv_packet_with_acknowledgement() {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(1u64);
		let channel_id = ChannelId::new(0);
		let port_id = PortId::transfer();

		let mut ctx = Context::<Test>::default();

		let channel_end = ChannelEnd { state: State::Open, ..Default::default() };
		ctx.store_channel((port_id.clone(), channel_id), &channel_end).unwrap();
		let packet = Packet {
			sequence: 1u64.into(),
			source_port: port_id.clone(),
			source_channel: channel_id,
			destination_port: port_id.clone(),
			destination_channel: channel_id,
			data: "hello".as_bytes().to_vec(),
			timeout_height: Default::default(),
			timeout_timestamp: Default::default(),
		};

		ctx.store_recv_packet((port_id, channel_id, packet.sequence), packet.clone())
			.unwrap();
		let ack = "success".as_bytes().to_vec();
		Pallet::<Test>::handle_message(HandlerMessage::WriteAck { packet, ack }).unwrap();
	});

	ext.persist_offchain_overlay();

	ext.execute_with(|| {
		let ack = "success".as_bytes().to_vec();
		let channel_id = ChannelId::new(0);
		let port_id = PortId::transfer();
		let packet_info = Pallet::<Test>::get_recv_packet_info(
			channel_id.to_string().as_bytes().to_vec(),
			port_id.as_bytes().to_vec(),
			vec![1],
		)
		.unwrap()
		.get(0)
		.unwrap()
		.clone();
		assert_eq!(packet_info.ack, Some(ack))
	})
}

#[test]
fn should_cleanup_offchain_packets_correctly() {
	let mut ext = new_test_ext();
	let channel_id = ChannelId::new(0);
	let port_id = PortId::transfer();
	let cleanup_period = CleanUpPacketsPeriod::get();

	ext.execute_with(|| {
		// Add some packets offchain
		let channel_end = ChannelEnd::default();
		let mut ctx = Context::<Test>::default();
		ctx.store_channel((port_id.clone(), channel_id), &channel_end).unwrap();
		ctx.store_next_sequence_send((port_id.clone(), channel_id), 11.into()).unwrap();
		ctx.store_next_sequence_recv((port_id.clone(), channel_id), 11.into()).unwrap();
		// Store packets and commitments
		for i in 1..=10u64 {
			let packet = Packet {
				sequence: i.into(),
				source_port: port_id.clone(),
				source_channel: channel_id,
				destination_port: port_id.clone(),
				destination_channel: channel_id,
				data: "hello".as_bytes().to_vec(),
				timeout_height: Default::default(),
				timeout_timestamp: Default::default(),
			};
			ctx.store_send_packet((port_id.clone(), channel_id, i.into()), packet).unwrap();

			// Store commitment for even numbers
			if i % 2 == 0 {
				ctx.store_packet_commitment(
					(port_id.clone(), channel_id, i.into()),
					"commitment".as_bytes().to_vec().into(),
				)
				.unwrap();
			}
		}

		// Store packet acknowledgements
		for i in 1..=10u64 {
			let packet = Packet {
				sequence: i.into(),
				source_port: port_id.clone(),
				source_channel: channel_id,
				destination_port: port_id.clone(),
				destination_channel: channel_id,
				data: "hello".as_bytes().to_vec(),
				timeout_height: Default::default(),
				timeout_timestamp: Default::default(),
			};
			ctx.store_recv_packet((port_id.clone(), channel_id, i.into()), packet).unwrap();
			// Store ack for odd numbers
			if i % 2 != 0 {
				ctx.store_packet_acknowledgement(
					(port_id.clone(), channel_id, i.into()),
					"commitment".as_bytes().to_vec().into(),
				)
				.unwrap();
				Pallet::<Test>::store_raw_acknowledgement(
					(port_id.clone(), channel_id, i.into()),
					"acknowledgement".as_bytes().to_vec(),
				)
				.unwrap();
			}
		}
	});

	ext.execute_with(|| {
		// should not cleanup withing the period
		for i in 1..cleanup_period {
			Pallet::<Test>::on_idle(i, Weight::max_value());
		}
	});

	ext.execute_with(|| {
		let channel_id_bytes = channel_id.to_string().as_bytes().to_vec();
		let port_id_bytes = port_id.as_bytes().to_vec();

		let (send_seq_set, _) =
			PendingSendPacketSeqs::<Test>::get(&(port_id_bytes.clone(), channel_id_bytes.clone()));

		let (recv_seq_set, _) =
			PendingRecvPacketSeqs::<Test>::get(&(port_id_bytes, channel_id_bytes));

		assert_eq!(send_seq_set, Default::default());

		assert_eq!(recv_seq_set, Default::default());
	});

	ext.execute_with(|| {
		Pallet::<Test>::on_idle(cleanup_period, Weight::max_value());
	});

	ext.execute_with(|| {
		let channel_id_bytes = channel_id.to_string().as_bytes().to_vec();
		let port_id_bytes = port_id.as_bytes().to_vec();

		let (send_seq_set, last_removed_send) =
			PendingSendPacketSeqs::<Test>::get(&(port_id_bytes.clone(), channel_id_bytes.clone()));

		let (recv_seq_set, last_removed_ack) =
			PendingRecvPacketSeqs::<Test>::get(&(port_id_bytes, channel_id_bytes));

		assert_eq!(send_seq_set, vec![2, 4, 6, 8, 10].into_iter().collect());

		assert_eq!(recv_seq_set, vec![1, 3, 5, 7, 9].into_iter().collect());

		assert_eq!(last_removed_send, 9);
		assert_eq!(last_removed_ack, 10);
		// Now let's prepare to remove pending sequences
		let mut ctx = Context::<Test>::default();
		for i in send_seq_set {
			ctx.delete_packet_commitment((port_id.clone(), channel_id, i.into())).unwrap();
		}

		for i in recv_seq_set {
			ctx.delete_packet_acknowledgement((port_id.clone(), channel_id, i.into()))
				.unwrap();
		}
	});

	ext.execute_with(|| {
		Pallet::<Test>::on_idle(cleanup_period, Weight::max_value());
	});

	ext.execute_with(|| {
		let channel_id_bytes = channel_id.to_string().as_bytes().to_vec();
		let port_id_bytes = port_id.as_bytes().to_vec();

		let (send_seq_set, last_removed_send) =
			PendingSendPacketSeqs::<Test>::get(&(port_id_bytes.clone(), channel_id_bytes.clone()));

		let (recv_seq_set, last_removed_ack) =
			PendingRecvPacketSeqs::<Test>::get(&(port_id_bytes, channel_id_bytes));

		println!("{send_seq_set:?}");

		assert!(send_seq_set.is_empty());

		assert!(recv_seq_set.is_empty());

		assert_eq!(last_removed_send, 10);
		assert_eq!(last_removed_ack, 10);
	});
}

#[test]
fn test_next_and_previous_consensus_state_for_beefy_and_grandpa_clients() {
	new_test_ext().execute_with(|| {
		let client_id_beefy = ClientId::from_str("11-beefy-0").unwrap();
		let mut ctx = Context::<Test>::default();
		let mock_cs_state = MockConsensusState::new(MockHeader::default());
		ctx.store_consensus_state(
			client_id_beefy.clone(),
			Height::new(0, 10),
			AnyConsensusState::Mock(mock_cs_state.clone()),
		)
		.unwrap();
		// Should return None for Beefy and grandpa clients since we do not cache recent consensus
		// heights for beefy and grandpa
		assert!(ctx
			.prev_consensus_state(&client_id_beefy, Height::new(0, 15))
			.unwrap()
			.is_none());
		assert!(ctx.next_consensus_state(&client_id_beefy, Height::new(0, 5)).unwrap().is_none());

		let client_id_grandpa = ClientId::from_str("10-grandpa-0").unwrap();
		ctx.store_consensus_state(
			client_id_grandpa.clone(),
			Height::new(0, 10),
			AnyConsensusState::Mock(mock_cs_state),
		)
		.unwrap();
		assert!(ctx
			.prev_consensus_state(&client_id_grandpa, Height::new(0, 15))
			.unwrap()
			.is_none());
		assert!(ctx
			.next_consensus_state(&client_id_grandpa, Height::new(0, 5))
			.unwrap()
			.is_none());
	})
}

#[test]
fn test_next_and_previous_consensus_state_for_other_client_types() {
	new_test_ext().execute_with(|| {
		let client_id = ClientId::from_str("07-tendermint-0").unwrap();
		let mut ctx = Context::<Test>::default();
		let mock_cs_state = MockConsensusState::new(MockHeader::default());
		// lets store 512 consensus states, only the 256 most recent should remain after the loop
		for i in 1..=512u64 {
			ctx.store_consensus_state(
				client_id.clone(),
				Height::new(0, i),
				AnyConsensusState::Mock(mock_cs_state.clone()),
			)
			.unwrap();
		}

		let stored_heights = ConsensusHeights::<Test>::get(client_id.as_bytes().to_vec());

		assert_eq!(stored_heights.len(), 256);
		assert_eq!(stored_heights.iter().rev().next(), Some(&Height::new(0, 512)));

		// Should return None for Beefy and grandpa clients since we do not cache recent consensus
		// heights for beefy and grandpa
		assert!(ctx.prev_consensus_state(&client_id, Height::new(0, 300)).unwrap().is_some());
		assert!(ctx.next_consensus_state(&client_id, Height::new(0, 400)).unwrap().is_some());
	})
}
