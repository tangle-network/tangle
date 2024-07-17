//! Relayer events.
use ibc::{core::ics04_channel::packet::Packet, events::IbcEvent as RawIbcEvent};
use ibc_runtime_api::IbcRuntimeApi;
use pallet_ibc::events::IbcEvent;
use sc_client_api::HeaderBackend;
use sp_api::{ApiRef, ProvideRuntimeApi};
use sp_runtime::traits::Block as BlockT;

/// Filter out none relayer events and modify
/// Fetch actual packet and acknowledgements from off chain storage and modify packets
pub fn filter_map_pallet_event<C, Block, AssetId>(
	at: <Block as BlockT>::Hash,
	api: &ApiRef<<C as ProvideRuntimeApi<Block>>::Api>,
	ev: IbcEvent,
) -> Option<RawIbcEvent>
where
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: IbcRuntimeApi<Block, AssetId>,
	Block: BlockT,
	AssetId: parity_scale_codec::Codec,
{
	let mut event: RawIbcEvent = ev.try_into().ok()?;
	match &mut event {
		RawIbcEvent::SendPacket(ev) => {
			let channel_id = ev.src_channel_id();
			let port_id = ev.src_port_id();
			let sequence = u64::from(ev.packet.sequence);
			let packets: Vec<ibc_primitives::PacketInfo> = api
				.query_send_packet_info(
					at,
					channel_id.to_string().as_bytes().to_vec(),
					port_id.as_bytes().to_vec(),
					vec![sequence],
				)
				.ok()
				.flatten()?;
			let packet = packets.get(0)?.clone();
			let packet: Packet = packet.into();
			ev.packet = packet;
			Some(event)
		},
		RawIbcEvent::ReceivePacket(ev) => {
			let channel_id = ev.dst_channel_id();
			let port_id = ev.dst_port_id();
			let sequence = u64::from(ev.packet.sequence);
			let packets: Vec<ibc_primitives::PacketInfo> = api
				.query_recv_packet_info(
					at,
					channel_id.to_string().as_bytes().to_vec(),
					port_id.as_bytes().to_vec(),
					vec![sequence],
				)
				.ok()
				.flatten()?;
			let packet = packets.get(0)?.clone();
			let packet: Packet = packet.into();
			ev.packet = packet;
			Some(event)
		},
		RawIbcEvent::WriteAcknowledgement(ev) => {
			let channel_id = ev.dst_channel_id();
			let port_id = ev.dst_port_id();
			let sequence = u64::from(ev.packet.sequence);
			let packets: Vec<ibc_primitives::PacketInfo> = api
				.query_recv_packet_info(
					at,
					channel_id.to_string().as_bytes().to_vec(),
					port_id.as_bytes().to_vec(),
					vec![sequence],
				)
				.ok()
				.flatten()?;
			let packet_info = packets.get(0)?.clone();
			ev.ack = packet_info.ack.clone()?;
			ev.packet = packet_info.into();
			Some(event)
		},
		RawIbcEvent::AcknowledgePacket(ev) => {
			let channel_id = ev.src_channel_id();
			let port_id = ev.src_port_id();
			let sequence = u64::from(ev.packet.sequence);
			let packets: Vec<ibc_primitives::PacketInfo> = api
				.query_send_packet_info(
					at,
					channel_id.to_string().as_bytes().to_vec(),
					port_id.as_bytes().to_vec(),
					vec![sequence],
				)
				.ok()
				.flatten()?;
			let packet = packets.get(0)?.clone();
			let packet: Packet = packet.into();
			ev.packet = packet;
			Some(event)
		},
		RawIbcEvent::TimeoutPacket(ev) => {
			let channel_id = ev.src_channel_id();
			let port_id = ev.src_port_id();
			let sequence = u64::from(ev.packet.sequence);
			let packets: Vec<ibc_primitives::PacketInfo> = api
				.query_send_packet_info(
					at,
					channel_id.to_string().as_bytes().to_vec(),
					port_id.as_bytes().to_vec(),
					vec![sequence],
				)
				.ok()
				.flatten()?;
			let packet = packets.get(0)?.clone();
			let packet: Packet = packet.into();
			ev.packet = packet;
			Some(event)
		},
		RawIbcEvent::TimeoutOnClosePacket(ev) => {
			let channel_id = ev.src_channel_id();
			let port_id = ev.src_port_id();
			let sequence = u64::from(ev.packet.sequence);
			let packets: Vec<ibc_primitives::PacketInfo> = api
				.query_send_packet_info(
					at,
					channel_id.to_string().as_bytes().to_vec(),
					port_id.as_bytes().to_vec(),
					vec![sequence],
				)
				.ok()
				.flatten()?;
			let packet = packets.get(0)?.clone();
			let packet: Packet = packet.into();
			ev.packet = packet;
			Some(event)
		},
		RawIbcEvent::NewBlock(_)
		| RawIbcEvent::AppModule(_)
		| RawIbcEvent::Empty(_)
		| RawIbcEvent::ChainError(_) => None,
		_ => Some(event),
	}
}
