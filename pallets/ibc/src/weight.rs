use super::*;
use crate::{light_clients::AnyClientMessage, routing::Context};
use frame_support::weights::constants::WEIGHT_REF_TIME_PER_MILLIS;
use grandpa_client_primitives::{
	justification::GrandpaJustification, standalone::GrandpaStandaloneJustification,
};
use ibc::core::{
	ics02_client::msgs::ClientMsg,
	ics03_connection::{context::ConnectionReader, msgs::ConnectionMsg},
	ics04_channel::msgs::{ChannelMsg, PacketMsg},
	ics24_host::identifier::ClientId,
	ics26_routing::msgs::Ics26Envelope,
};
use ibc_primitives::{client_id_from_bytes, CallbackWeight};
use ics10_grandpa::client_message::{ClientMessage, RelayChainHeader};
use ics10_grandpa_standalone::client_message::{
	ClientMessage as StandaloneClientMessage, StandaloneChainHeader,
};

pub trait WeightInfo {
	fn create_client() -> Weight;
	fn conn_open_init() -> Weight;
	fn update_tendermint_client(i: u32) -> Weight;
	fn conn_try_open_tendermint() -> Weight;
	fn conn_open_ack_tendermint() -> Weight;
	fn conn_open_confirm_tendermint() -> Weight;
	fn channel_open_init() -> Weight;
	fn channel_open_try_tendermint() -> Weight;
	fn channel_open_ack_tendermint() -> Weight;
	fn channel_open_confirm_tendermint() -> Weight;
	fn channel_close_init() -> Weight;
	fn channel_close_confirm_tendermint() -> Weight;
	fn recv_packet_tendermint(i: u32) -> Weight;
	fn ack_packet_tendermint(i: u32, j: u32) -> Weight;
	fn timeout_packet_tendermint(i: u32) -> Weight;
	fn transfer() -> Weight;
	fn on_chan_open_init() -> Weight;
	fn on_chan_open_try() -> Weight;
	fn on_recv_packet() -> Weight;
	fn on_chan_open_ack() -> Weight;
	fn on_chan_open_confirm() -> Weight;
	fn on_chan_close_init() -> Weight;
	fn on_chan_close_confirm() -> Weight;
	fn on_acknowledgement_packet() -> Weight;
	fn on_timeout_packet() -> Weight;
	fn update_grandpa_client(i: u32, j: u32) -> Weight;
	fn packet_cleanup(i: u32) -> Weight;
}

impl WeightInfo for () {
	fn create_client() -> Weight {
		Weight::default()
	}

	fn conn_open_init() -> Weight {
		Weight::default()
	}

	fn update_tendermint_client(i: u32) -> Weight {
		Weight::from_parts(3 * i as u64 * WEIGHT_REF_TIME_PER_MILLIS, 0)
	}

	fn conn_try_open_tendermint() -> Weight {
		Weight::default()
	}

	fn conn_open_ack_tendermint() -> Weight {
		Weight::default()
	}

	fn conn_open_confirm_tendermint() -> Weight {
		Weight::default()
	}

	fn channel_open_init() -> Weight {
		Weight::default()
	}

	fn channel_open_try_tendermint() -> Weight {
		Weight::default()
	}

	fn channel_open_ack_tendermint() -> Weight {
		Weight::default()
	}

	fn channel_open_confirm_tendermint() -> Weight {
		Weight::default()
	}

	fn channel_close_init() -> Weight {
		Weight::default()
	}

	fn channel_close_confirm_tendermint() -> Weight {
		Weight::default()
	}

	fn recv_packet_tendermint(_i: u32) -> Weight {
		Weight::default()
	}

	fn ack_packet_tendermint(_i: u32, _j: u32) -> Weight {
		Weight::default()
	}

	fn timeout_packet_tendermint(_i: u32) -> Weight {
		Weight::default()
	}

	fn transfer() -> Weight {
		Weight::default()
	}

	fn on_chan_open_init() -> Weight {
		Weight::default()
	}

	fn on_chan_open_try() -> Weight {
		Weight::default()
	}

	fn on_recv_packet() -> Weight {
		Weight::default()
	}

	fn on_chan_open_ack() -> Weight {
		Weight::default()
	}

	fn on_chan_open_confirm() -> Weight {
		Weight::default()
	}

	fn on_chan_close_init() -> Weight {
		Weight::default()
	}

	fn on_chan_close_confirm() -> Weight {
		Weight::default()
	}

	fn on_acknowledgement_packet() -> Weight {
		Weight::default()
	}

	fn on_timeout_packet() -> Weight {
		Weight::default()
	}

	fn update_grandpa_client(_i: u32, _j: u32) -> Weight {
		Weight::default()
	}

	fn packet_cleanup(_i: u32) -> Weight {
		Weight::default()
	}
}

pub struct WeightRouter<T: Config>(PhantomData<T>);

impl<T: Config> WeightRouter<T> {
	pub fn get_weight(port_id: &str) -> Option<Box<dyn CallbackWeight>> {
		match port_id {
			#[allow(clippy::box_default)]
			ibc::applications::transfer::PORT_ID_STR => Some(Box::new(ics20::WeightHandler::<T>::default())),
			_ => None,
		}
	}
}

/// Get client id for a port and channel combination
pub fn channel_client<T: Config>(channel_id: &[u8], port_id: &[u8]) -> Result<ClientId, Error<T>> {
	for (connection_id, channels) in ChannelsConnection::<T>::iter() {
		if channels.contains(&(port_id.to_vec(), channel_id.to_vec())) {
			if let Some((client_id, ..)) = ConnectionClient::<T>::iter()
				.find(|(.., connection_ids)| connection_ids.contains(&connection_id))
			{
				return client_id_from_bytes(client_id).map_err(|_| Error::<T>::Other);
			}
		}
	}
	Err(Error::<T>::Other)
}

pub(crate) fn deliver<T: Config + Send + Sync>(msgs: &[Any]) -> Weight {
	msgs.iter()
		.filter_map(|msg| {
			let type_url = msg.type_url.clone();
			let msg = ibc_proto::google::protobuf::Any { type_url, value: msg.value.clone() };
			let msg: Option<Ics26Envelope<Context<T>>> = msg.try_into().ok();
			msg
		})
		.fold(Weight::default(), |acc, msg| {
			// Add benchmarked weight for that message type
			// Add benchmarked weight for module callback
			let temp = match msg {
				Ics26Envelope::Ics2Msg(msgs) => match msgs {
					ClientMsg::CreateClient(_) => <T as Config>::WeightInfo::create_client(),
					ClientMsg::UpdateClient(msg) => {
						let client_type = msg
							.client_id
							.as_str()
							.rsplit_once('-')
							.map(|(client_type_str, ..)| client_type_str);
						match client_type {
							Some(ty) if ty.contains("tendermint") => match msg.client_message {
								AnyClientMessage::Tendermint(client_message) => match client_message {
									ics07_tendermint::client_message::ClientMessage::Header(header) => {
										<T as Config>::WeightInfo::update_tendermint_client(header.signed_header.commit.signatures.len() as u32)
									}
									ics07_tendermint::client_message::ClientMessage::Misbehaviour(misbehaviour) => {
										<T as Config>::WeightInfo::update_tendermint_client(misbehaviour.header1.signed_header.commit.signatures.len() as u32).
											saturating_add(<T as Config>::WeightInfo::update_tendermint_client(misbehaviour.header2.signed_header.commit.signatures.len() as u32))
									}

								}
								_ => return Weight::MAX,
							}
							Some(ty) if ty.contains("grandpa") => match msg.client_message {
								AnyClientMessage::GrandpaStandalone(client_message) => match client_message {
									StandaloneClientMessage::Header(header) => {
										let justification =
											GrandpaStandaloneJustification::<StandaloneChainHeader>::decode(
												&mut &*header.finality_proof.justification,
											)
											.expect("Justification should be valid");
										<T as Config>::WeightInfo::update_grandpa_client(
											justification.commit.precommits.len() as u32,
											header.finality_proof.unknown_headers.len() as u32,
										)
									},
									StandaloneClientMessage::Misbehaviour(misbehaviour) => {
										let justification_a =
											GrandpaStandaloneJustification::<StandaloneChainHeader>::decode(
												&mut &*misbehaviour.first_finality_proof.justification,
											)
												.expect("Justification should be valid");

										let justification_b =
											GrandpaStandaloneJustification::<StandaloneChainHeader>::decode(
												&mut &*misbehaviour.second_finality_proof.justification,
											)
												.expect("Justification should be valid");

										<T as Config>::WeightInfo::update_grandpa_client(
											justification_a.commit.precommits.len() as u32,
											misbehaviour.first_finality_proof.unknown_headers.len() as u32,
										).saturating_add(<T as Config>::WeightInfo::update_grandpa_client(
											justification_b.commit.precommits.len() as u32,
											misbehaviour.second_finality_proof.unknown_headers.len() as u32,
										))
									},
								},
								AnyClientMessage::Grandpa(client_message) => match client_message {
									ClientMessage::Header(header) => {
										let justification =
											GrandpaJustification::<RelayChainHeader>::decode(
												&mut &*header.finality_proof.justification,
											)
											.expect("Justification should be valid");
										<T as Config>::WeightInfo::update_grandpa_client(
											justification.commit.precommits.len() as u32,
											header.finality_proof.unknown_headers.len() as u32,
										)
									},
									ClientMessage::Misbehaviour(misbehaviour) => {
										let justification_a =
											GrandpaJustification::<RelayChainHeader>::decode(
												&mut &*misbehaviour.first_finality_proof.justification,
											)
												.expect("Justification should be valid");

										let justification_b =
											GrandpaJustification::<RelayChainHeader>::decode(
												&mut &*misbehaviour.second_finality_proof.justification,
											)
												.expect("Justification should be valid");

										<T as Config>::WeightInfo::update_grandpa_client(
											justification_a.commit.precommits.len() as u32,
											misbehaviour.first_finality_proof.unknown_headers.len() as u32,
										).saturating_add(<T as Config>::WeightInfo::update_grandpa_client(
											justification_b.commit.precommits.len() as u32,
											misbehaviour.second_finality_proof.unknown_headers.len() as u32,
										))
									},
								},
								_ => return Weight::MAX,
							},
							_ => Weight::default(),
						}
					},
					ClientMsg::UpgradeClient(_) => Weight::default(),
				},
				Ics26Envelope::Ics3Msg(msgs) => match msgs {
					ConnectionMsg::ConnectionOpenInit(_) =>
						<T as Config>::WeightInfo::conn_open_init(),
					ConnectionMsg::ConnectionOpenTry(msg) => {
						let client_type = msg
							.client_id
							.as_str()
							.rsplit_once('-')
							.map(|(client_type_str, ..)| client_type_str);
						match client_type {
							Some(ty) if ty.contains("tendermint") =>
								<T as Config>::WeightInfo::conn_try_open_tendermint(),
							_ => Weight::default(),
						}
					},
					ConnectionMsg::ConnectionOpenAck(msg) => {
						let connection_id = msg.connection_id;
						let ctx = routing::Context::<T>::new();
						let connection_end = ctx.connection_end(&connection_id).unwrap_or_default();
						let client_type = connection_end
							.client_id()
							.as_str()
							.rsplit_once('-')
							.map(|(client_type_str, ..)| client_type_str);
						match client_type {
							Some(ty) if ty.contains("tendermint") =>
								<T as Config>::WeightInfo::conn_open_ack_tendermint(),
							_ => Weight::default(),
						}
					},
					ConnectionMsg::ConnectionOpenConfirm(msg) => {
						let connection_id = msg.connection_id;
						let ctx = routing::Context::<T>::new();
						let connection_end = ctx.connection_end(&connection_id).unwrap_or_default();
						let client_type = connection_end
							.client_id()
							.as_str()
							.rsplit_once('-')
							.map(|(client_type_str, ..)| client_type_str);
						match client_type {
							Some(ty) if ty.contains("tendermint") =>
								<T as Config>::WeightInfo::conn_open_confirm_tendermint(),
							_ => Weight::default(),
						}
					},
				},
				Ics26Envelope::Ics4ChannelMsg(msgs) => match msgs {
					ChannelMsg::ChannelOpenInit(channel_msg) => {
						let cb = WeightRouter::<T>::get_weight(channel_msg.port_id.as_str())
							.unwrap_or_else(|| Box::new(()));
						let cb_weight = cb.on_chan_open_init();
						let lc_verification_weight =
							match channel_msg.channel.connection_hops.get(0) {
								Some(connection_id) => {
									let ctx = routing::Context::<T>::new();
									let connection_end =
										ctx.connection_end(connection_id).unwrap_or_default();
									let client_type = connection_end
										.client_id()
										.as_str()
										.rsplit_once('-')
										.map(|(client_type_str, ..)| client_type_str);
									match client_type {
										Some(ty) if ty.contains("tendermint") =>
											<T as Config>::WeightInfo::channel_open_init(),
										_ => Weight::default(),
									}
								},
								None => Weight::default(),
							};
						cb_weight.saturating_add(lc_verification_weight)
					},
					ChannelMsg::ChannelOpenTry(channel_msg) => {
						let cb = WeightRouter::<T>::get_weight(channel_msg.port_id.as_str())
							.unwrap_or_else(|| Box::new(()));
						let cb_weight = cb.on_chan_open_try();
						let lc_verification_weight =
							match channel_msg.channel.connection_hops.get(0) {
								Some(connection_id) => {
									let ctx = routing::Context::<T>::new();
									let connection_end =
										ctx.connection_end(connection_id).unwrap_or_default();
									let client_type = connection_end
										.client_id()
										.as_str()
										.rsplit_once('-')
										.map(|(client_type_str, ..)| client_type_str);
									match client_type {
										Some(ty) if ty.contains("tendermint") =>
											<T as Config>::WeightInfo::channel_open_try_tendermint(),
										_ => Weight::default(),
									}
								},
								None => Weight::default(),
							};
						cb_weight.saturating_add(lc_verification_weight)
					},
					ChannelMsg::ChannelOpenAck(channel_msg) => {
						let cb = WeightRouter::<T>::get_weight(channel_msg.port_id.as_str())
							.unwrap_or_else(|| Box::new(()));
						let cb_weight =
							cb.on_chan_open_ack(&channel_msg.port_id, &channel_msg.channel_id);
						let lc_verification_weight = match channel_client::<T>(
							channel_msg.port_id.as_bytes(),
							channel_msg.channel_id.to_string().as_bytes(),
						) {
							Ok(client_id) => {
								let client_type = client_id
									.as_str()
									.rsplit_once('-')
									.map(|(client_type_str, ..)| client_type_str);
								match client_type {
									Some(ty) if ty.contains("tendermint") =>
										<T as Config>::WeightInfo::channel_open_ack_tendermint(),
									_ => Weight::default(),
								}
							},
							Err(_) => Weight::default(),
						};
						cb_weight.saturating_add(lc_verification_weight)
					},
					ChannelMsg::ChannelOpenConfirm(channel_msg) => {
						let cb = WeightRouter::<T>::get_weight(channel_msg.port_id.as_str())
							.unwrap_or_else(|| Box::new(()));
						let cb_weight =
							cb.on_chan_open_confirm(&channel_msg.port_id, &channel_msg.channel_id);
						let lc_verification_weight = match channel_client::<T>(
							channel_msg.port_id.as_bytes(),
							channel_msg.channel_id.to_string().as_bytes(),
						) {
							Ok(client_id) => {
								let client_type = client_id
									.as_str()
									.rsplit_once('-')
									.map(|(client_type_str, ..)| client_type_str);
								match client_type {
									Some(ty) if ty.contains("tendermint") =>
										<T as Config>::WeightInfo::channel_open_confirm_tendermint(),
									_ => Weight::default(),
								}
							},
							Err(_) => Weight::default(),
						};
						cb_weight.saturating_add(lc_verification_weight)
					},
					ChannelMsg::ChannelCloseInit(channel_msg) => {
						let cb = WeightRouter::<T>::get_weight(channel_msg.port_id.as_str())
							.unwrap_or_else(|| Box::new(()));
						let cb_weight =
							cb.on_chan_close_init(&channel_msg.port_id, &channel_msg.channel_id);
						let lc_verification_weight = match channel_client::<T>(
							channel_msg.port_id.as_bytes(),
							channel_msg.channel_id.to_string().as_bytes(),
						) {
							Ok(client_id) => {
								let client_type = client_id
									.as_str()
									.rsplit_once('-')
									.map(|(client_type_str, ..)| client_type_str);
								match client_type {
									Some(ty) if ty.contains("tendermint") =>
										<T as Config>::WeightInfo::channel_close_init(),
									_ => Weight::default(),
								}
							},
							Err(_) => Weight::default(),
						};
						cb_weight.saturating_add(lc_verification_weight)
					},
					ChannelMsg::ChannelCloseConfirm(channel_msg) => {
						let cb = WeightRouter::<T>::get_weight(channel_msg.port_id.as_str())
							.unwrap_or_else(|| Box::new(()));
						let cb_weight =
							cb.on_chan_close_confirm(&channel_msg.port_id, &channel_msg.channel_id);
						let lc_verification_weight = match channel_client::<T>(
							channel_msg.port_id.as_bytes(),
							channel_msg.channel_id.to_string().as_bytes(),
						) {
							Ok(client_id) => {
								let client_type = client_id
									.as_str()
									.rsplit_once('-')
									.map(|(client_type_str, ..)| client_type_str);
								match client_type {
									Some(ty) if ty.contains("tendermint") =>
										<T as Config>::WeightInfo::channel_close_confirm_tendermint(
										),
									_ => Weight::default(),
								}
							},
							Err(_) => Weight::default(),
						};
						cb_weight.saturating_add(lc_verification_weight)
					},
				},
				Ics26Envelope::Ics4PacketMsg(msgs) => match msgs {
					PacketMsg::RecvPacket(packet_msg) => {
						let cb = WeightRouter::<T>::get_weight(
							packet_msg.packet.destination_port.as_str(),
						)
						.unwrap_or_else(|| Box::new(()));
						let cb_weight = cb.on_recv_packet(&packet_msg.packet);
						let lc_verification_weight = match channel_client::<T>(
							packet_msg.packet.destination_port.as_bytes(),
							packet_msg.packet.destination_channel.to_string().as_bytes(),
						) {
							Ok(client_id) => {
								let client_type = client_id
									.as_str()
									.rsplit_once('-')
									.map(|(client_type_str, ..)| client_type_str);
								match client_type {
									Some(ty) if ty.contains("tendermint") =>
										<T as Config>::WeightInfo::recv_packet_tendermint(
											packet_msg.packet.data.len() as u32,
										),
									_ => Weight::default(),
								}
							},
							Err(_) => Weight::default(),
						};
						cb_weight.saturating_add(lc_verification_weight)
					},
					PacketMsg::AckPacket(packet_msg) => {
						let cb = WeightRouter::<T>::get_weight(
							packet_msg.packet.destination_port.as_str(),
						)
						.unwrap_or_else(|| Box::new(()));
						let cb_weight = cb.on_acknowledgement_packet(
							&packet_msg.packet,
							&packet_msg.acknowledgement,
						);
						let lc_verification_weight = match channel_client::<T>(
							packet_msg.packet.destination_port.as_bytes(),
							packet_msg.packet.destination_channel.to_string().as_bytes(),
						) {
							Ok(client_id) => {
								let client_type = client_id
									.as_str()
									.rsplit_once('-')
									.map(|(client_type_str, ..)| client_type_str);
								match client_type {
									Some(ty) if ty.contains("tendermint") =>
										<T as Config>::WeightInfo::ack_packet_tendermint(
											packet_msg.packet.data.len() as u32,
											packet_msg.acknowledgement.into_bytes().len() as u32,
										),
									_ => Weight::default(),
								}
							},
							Err(_) => Weight::default(),
						};
						cb_weight.saturating_add(lc_verification_weight)
					},
					PacketMsg::ToPacket(packet_msg) => {
						let cb = WeightRouter::<T>::get_weight(
							packet_msg.packet.destination_port.as_str(),
						)
						.unwrap_or_else(|| Box::new(()));
						let cb_weight = cb.on_timeout_packet(&packet_msg.packet);
						let lc_verification_weight = match channel_client::<T>(
							packet_msg.packet.destination_port.as_bytes(),
							packet_msg.packet.destination_channel.to_string().as_bytes(),
						) {
							Ok(client_id) => {
								let client_type = client_id
									.as_str()
									.rsplit_once('-')
									.map(|(client_type_str, ..)| client_type_str);
								match client_type {
									Some(ty) if ty.contains("tendermint") =>
										<T as Config>::WeightInfo::timeout_packet_tendermint(
											packet_msg.packet.data.len() as u32,
										),
									_ => Weight::default(),
								}
							},
							Err(_) => Weight::default(),
						};
						cb_weight.saturating_add(lc_verification_weight)
					},
					PacketMsg::ToClosePacket(packet_msg) => {
						let cb = WeightRouter::<T>::get_weight(
							packet_msg.packet.destination_port.as_str(),
						)
						.unwrap_or_else(|| Box::new(()));
						let cb_weight = cb.on_timeout_packet(&packet_msg.packet);
						let lc_verification_weight = match channel_client::<T>(
							packet_msg.packet.destination_port.as_bytes(),
							packet_msg.packet.destination_channel.to_string().as_bytes(),
						) {
							Ok(client_id) => {
								let client_type = client_id
									.as_str()
									.rsplit_once('-')
									.map(|(client_type_str, ..)| client_type_str);
								match client_type {
									Some(ty) if ty.contains("tendermint") =>
										<T as Config>::WeightInfo::timeout_packet_tendermint(
											packet_msg.packet.data.len() as u32,
										),
									_ => Weight::default(),
								}
							},
							Err(_) => Weight::default(),
						};
						cb_weight.saturating_add(lc_verification_weight)
					},
				},
				_ => Weight::default(),
			};
			acc.saturating_add(temp)
		})
}
