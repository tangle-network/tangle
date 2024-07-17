use super::*;
use core::time::Duration;
use frame_support::traits::Get;
use ibc_primitives::PacketInfo;
use sp_core::crypto::AccountId32;

use crate::{
	ics23::{
		acknowledgements::Acknowledgements, channels::Channels, next_seq_ack::NextSequenceAck,
		next_seq_recv::NextSequenceRecv, next_seq_send::NextSequenceSend,
		packet_commitments::PacketCommitment, receipts::PacketReceipt,
	},
	impls::host_height,
	routing::Context,
};
use ibc::{
	core::{
		ics02_client::error::Error as ICS02Error,
		ics04_channel::{
			channel::ChannelEnd,
			commitment::{AcknowledgementCommitment, PacketCommitment as PacketCommitmentType},
			context::{ChannelKeeper, ChannelReader},
			error::Error as ICS04Error,
			msgs::acknowledgement::Acknowledgement,
			packet::{Receipt, Sequence},
		},
		ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
	},
	timestamp::Timestamp,
	Height,
};
use tendermint_proto::Protobuf;

impl<T: Config + Sync + Send> ChannelReader for Context<T> {
	/// Returns the ChannelEnd for the given `port_id` and `chan_id`.
	fn channel_end(&self, port_channel_id: &(PortId, ChannelId)) -> Result<ChannelEnd, ICS04Error> {
		log::trace!(target: "pallet_ibc",
			"in channel : [channel_end] >> port_id = {:?}, channel_id = {:?}",
			port_channel_id.0,
			port_channel_id.1
		);
		let data = <Channels<T>>::get(port_channel_id.0.clone(), port_channel_id.1)
			.ok_or_else(ICS04Error::missing_channel)?;
		let channel_end = ChannelEnd::decode_vec(&data).map_err(|_| {
			ICS04Error::channel_not_found(port_channel_id.clone().0, port_channel_id.clone().1)
		})?;
		log::trace!(target: "pallet_ibc", "in channel : [channel_end] >> channel_end = {:?}", channel_end);
		Ok(channel_end)
	}

	fn connection_channels(
		&self,
		conn_id: &ConnectionId,
	) -> Result<Vec<(PortId, ChannelId)>, ICS04Error> {
		if <ChannelsConnection<T>>::contains_key(conn_id.as_bytes()) {
			let port_and_channel_id = <ChannelsConnection<T>>::get(conn_id.as_bytes());

			let mut result = vec![];

			for item in port_and_channel_id {
				let port_id = String::from_utf8(item.0).map_err(|e| {
					ICS04Error::implementation_specific(format!(
						"[connection_channels]: error decoding port_id: {e}"
					))
				})?;
				let port_id = PortId::from_str(port_id.as_str()).map_err(|e| {
					ICS04Error::implementation_specific(format!(
						"[connection_channels]: invalid port id string: {e}"
					))
				})?;

				let channel_id = String::from_utf8(item.1).map_err(|e| {
					ICS04Error::implementation_specific(format!(
						"[connection_channels]: error decoding channel_id: {e}"
					))
				})?;
				let channel_id = ChannelId::from_str(channel_id.as_str()).map_err(|e| {
					ICS04Error::implementation_specific(format!(
						"[connection_channels]: error decoding channel_id: {e}"
					))
				})?;

				result.push((port_id, channel_id));
			}

			log::trace!(target: "pallet_ibc",
				"in channel : [connection_channels] >> Vector<(PortId, ChannelId)> =  {:?}",
				result
			);
			Ok(result)
		} else {
			Err(ICS04Error::connection_not_open(conn_id.clone()))
		}
	}

	fn get_next_sequence_send(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, ICS04Error> {
		let seq = <NextSequenceSend<T>>::get(port_channel_id.0.clone(), port_channel_id.1)
			.ok_or_else(|| ICS04Error::missing_next_send_seq(port_channel_id.clone()))?;
		log::trace!(target: "pallet_ibc", "in channel : [get_next_sequence] >> sequence  = {:?}", seq);
		Ok(Sequence::from(seq))
	}

	fn get_next_sequence_recv(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, ICS04Error> {
		let seq = <NextSequenceRecv<T>>::get(port_channel_id.0.clone(), port_channel_id.1)
			.ok_or_else(|| ICS04Error::missing_next_recv_seq(port_channel_id.clone()))?;

		log::trace!(target: "pallet_ibc", "in channel : [get_next_sequence_recv] >> sequence = {:?}", seq);
		Ok(Sequence::from(seq))
	}

	fn get_next_sequence_ack(
		&self,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<Sequence, ICS04Error> {
		let seq = <NextSequenceAck<T>>::get(port_channel_id.0.clone(), port_channel_id.1)
			.ok_or_else(|| ICS04Error::missing_next_ack_seq(port_channel_id.clone()))?;
		log::trace!(target: "pallet_ibc", "in channel : [get_next_sequence_ack] >> sequence = {:?}", seq);
		Ok(Sequence::from(seq))
	}

	fn get_packet_commitment(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<PacketCommitmentType, ICS04Error> {
		if <PacketCommitment<T>>::contains_key((key.0.clone(), key.1, key.2)) {
			let data = <PacketCommitment<T>>::get((key.0.clone(), key.1, key.2))
				.ok_or_else(ICS04Error::missing_packet)?;
			log::trace!(target: "pallet_ibc", "in channel : [get_packet_commitment] >> packet_commitment = {:?}", data);
			Ok(data.into())
		} else {
			log::trace!(target: "pallet_ibc",
				"in channel : [get_packet_commitment] >> read get packet commitment return None"
			);
			Err(ICS04Error::packet_commitment_not_found(key.2))
		}
	}

	fn get_packet_receipt(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<Receipt, ICS04Error> {
		let seq = u64::from(key.2);

		if <PacketReceipt<T>>::contains_key((key.0.clone(), key.1, key.2)) {
			let data = <PacketReceipt<T>>::get((key.0.clone(), key.1, key.2))
				.ok_or_else(|| ICS04Error::packet_receipt_not_found(key.2))?;
			let data = String::from_utf8(data).map_err(|e| {
				ICS04Error::implementation_specific(format!(
					"[get_packet_receipt]: error decoding packet receipt: {e}"
				))
			})?;
			let data = match data.as_ref() {
				"Ok" => Receipt::Ok,
				_ => return Err(ICS04Error::packet_receipt_not_found(seq.into())),
			};
			log::trace!(target: "pallet_ibc", "in channel : [get_packet_receipt] >> packet_receipt = {:?}", data);
			Ok(data)
		} else {
			log::trace!(target: "pallet_ibc", "in channel : [get_packet_receipt] >> read get packet receipt not found");
			Err(ICS04Error::packet_receipt_not_found(key.2))
		}
	}

	fn get_packet_acknowledgement(
		&self,
		key: &(PortId, ChannelId, Sequence),
	) -> Result<AcknowledgementCommitment, ICS04Error> {
		if <Acknowledgements<T>>::contains_key((key.0.clone(), key.1, key.2)) {
			let ack = <Acknowledgements<T>>::get((key.0.clone(), key.1, key.2))
				.ok_or_else(|| ICS04Error::packet_acknowledgement_not_found(key.2))?;
			log::trace!(target: "pallet_ibc",
				"in channel : [get_packet_acknowledgement] >> packet_acknowledgement = {:?}",
				ack
			);
			Ok(ack.into())
		} else {
			log::trace!(target: "pallet_ibc",
				"in channel : [get_packet_acknowledgement] >> get acknowledgement not found"
			);
			Err(ICS04Error::packet_acknowledgement_not_found(key.2))
		}
	}

	/// A hashing function for packet commitments
	fn hash(&self, value: Vec<u8>) -> Vec<u8> {
		sp_io::hashing::sha2_256(&value).to_vec()
	}

	fn client_update_time(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Timestamp, ICS04Error> {
		let encoded_height = height
			.encode_vec()
			.map_err(|e| ICS04Error::ics02_client(ICS02Error::encode(e)))?;
		let client_id_bytes = client_id.as_bytes().to_vec();
		let timestamp =
			ClientUpdateTime::<T>::get(client_id_bytes, encoded_height).ok_or_else(|| {
				ICS04Error::implementation_specific(format!(
					"[client_update_time]:  client update timestamp not found for {client_id} at height: {height}"
				))
			})?;

		log::trace!(target: "pallet_ibc", "in channel: [client_update_time] >> height = {:?}, timestamp = {:?}", height,  timestamp);
		Timestamp::from_nanoseconds(timestamp).map_err(|e| {
			ICS04Error::implementation_specific(format!(
				"[client_update_time]:  error decoding timestamp from nano seconds: {e}"
			))
		})
	}

	fn client_update_height(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Height, ICS04Error> {
		let encoded_height = height
			.encode_vec()
			.map_err(|e| ICS04Error::ics02_client(ICS02Error::encode(e)))?;
		let client_id_bytes = client_id.as_bytes().to_vec();
		let host_height = ClientUpdateHeight::<T>::get(client_id_bytes, encoded_height)
			.ok_or_else(|| {
				ICS04Error::implementation_specific(format!(
					"[client_update_time]:  client update height not found for {client_id} at height: {height}"
				))
			})?;

		log::trace!(target: "pallet_ibc", "in channel: [client_update_height] >> height = {:?}, host height {:?}", height,  host_height);
		Height::decode_vec(&host_height).map_err(|e| {
			ICS04Error::implementation_specific(format!(
				"[client_update_height]: error decoding height: {e}"
			))
		})
	}

	/// Returns a counter on the number of channel ids have been created thus far.
	/// The value of this counter should increase only via method
	/// `ChannelKeeper::increase_channel_counter`.
	fn channel_counter(&self) -> Result<u64, ICS04Error> {
		let count = ChannelCounter::<T>::get();
		log::trace!(target: "pallet_ibc", "in channel: [channel_counter] >> channel_counter = {:?}", count);
		Ok(count.into())
	}

	fn max_expected_time_per_block(&self) -> Duration {
		let expected = T::ExpectedBlockTime::get();
		Duration::from_millis(expected)
	}
}

impl<T: Config + Sync + Send> ChannelKeeper for Context<T>
where
	AccountId32: From<<T as frame_system::Config>::AccountId>,
	Self: ChannelReader,
{
	fn store_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		commitment: PacketCommitmentType,
	) -> Result<(), ICS04Error> {
		log::trace!(target: "pallet_ibc", "in channel : [store_packet_commitment] >> packet_commitment {key:?} = {:#?}", commitment);
		<PacketCommitment<T>>::insert((key.0.clone(), key.1, key.2), commitment);
		if let Some(val) = PacketCounter::<T>::get().checked_add(1) {
			PacketCounter::<T>::put(val);
		}

		Ok(())
	}

	fn store_send_packet(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		packet: ibc::core::ics04_channel::packet::Packet,
	) -> Result<(), ICS04Error> {
		// store packet offchain
		let channel_id = key.1.to_string().as_bytes().to_vec();
		let port_id = key.0.as_bytes().to_vec();
		let seq = u64::from(key.2);
		let channel_end = ChannelReader::channel_end(self, &(key.0, key.1))?;
		let key = Pallet::<T>::send_packet_key(channel_id, port_id, seq);

		let mut packet_info: PacketInfo = packet.into();
		packet_info.height = Some(host_height::<T>());
		packet_info.channel_order = channel_end.ordering as u8;

		SendPackets::<T>::insert(&key, packet_info.encode());
		log::trace!(target: "pallet_ibc", "in channel: [store_send_packet] >> writing packet {:?} {:?}", key, packet_info);
		Ok(())
	}

	fn store_recv_packet(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		packet: ibc::core::ics04_channel::packet::Packet,
	) -> Result<(), ICS04Error> {
		// Store packet offchain
		let channel_id = key.1.to_string().as_bytes().to_vec();
		let port_id = key.0.as_bytes().to_vec();
		let seq = u64::from(key.2);
		let channel_end = ChannelReader::channel_end(self, &(key.0, key.1))?;
		let key = Pallet::<T>::recv_packet_key(channel_id, port_id, seq);
		let mut packet_info: PacketInfo = packet.into();
		packet_info.height = Some(host_height::<T>());
		packet_info.channel_order = channel_end.ordering as u8;
		RecvPackets::<T>::insert(&key, packet_info.encode());
		log::trace!(target: "pallet_ibc", "in channel: [store_recv_packet] >> writing packet {:?} {:?}", key, packet_info);
		Ok(())
	}

	fn delete_packet_commitment(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), ICS04Error> {
		// delete packet commitment
		<PacketCommitment<T>>::remove((key.0.clone(), key.1, key.2));

		if let Some(val) = PacketCounter::<T>::get().checked_sub(1) {
			PacketCounter::<T>::put(val);
		}

		Ok(())
	}

	fn store_packet_receipt(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		receipt: Receipt,
	) -> Result<(), ICS04Error> {
		let receipt = match receipt {
			Receipt::Ok => b"Ok".to_vec(),
		};

		<PacketReceipt<T>>::insert((key.0.clone(), key.1, key.2), receipt);

		if let Some(val) = <PacketReceiptCounter<T>>::get().checked_add(1) {
			<PacketReceiptCounter<T>>::put(val)
		}

		Ok(())
	}

	fn store_raw_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		ack: Acknowledgement,
	) -> Result<(), ICS04Error> {
		Pallet::<T>::store_raw_acknowledgement(key, ack.into_bytes()).map_err(|_| {
			ICS04Error::implementation_specific("Error storing acknowledgement".to_string())
		})
	}

	fn store_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
		ack_commitment: AcknowledgementCommitment,
	) -> Result<(), ICS04Error> {
		// store packet acknowledgement key-value
		<Acknowledgements<T>>::insert((key.0.clone(), key.1, key.2), ack_commitment);

		if let Some(val) = AcknowledgementCounter::<T>::get().checked_add(1) {
			AcknowledgementCounter::<T>::put(val)
		}

		Ok(())
	}

	fn delete_packet_acknowledgement(
		&mut self,
		key: (PortId, ChannelId, Sequence),
	) -> Result<(), ICS04Error> {
		// remove acknowledgements
		<Acknowledgements<T>>::remove((key.0.clone(), key.1, key.2));

		if let Some(val) = AcknowledgementCounter::<T>::get().checked_sub(1) {
			AcknowledgementCounter::<T>::put(val)
		}
		Ok(())
	}

	fn store_connection_channels(
		&mut self,
		conn_id: ConnectionId,
		port_channel_id: &(PortId, ChannelId),
	) -> Result<(), ICS04Error> {
		let conn_id = conn_id.as_bytes().to_vec();

		let port_channel_id = (
			port_channel_id.0.as_bytes().to_vec(),
			port_channel_id.1.to_string().as_bytes().to_vec(),
		);

		if <ChannelsConnection<T>>::contains_key(conn_id.clone()) {
			log::trace!(target: "pallet_ibc", "in channel: [store_connection_channels] >> insert port_channel_id");
			<ChannelsConnection<T>>::try_mutate(conn_id, |val| -> Result<(), &'static str> {
				val.push(port_channel_id);
				Ok(())
			})
			.expect("store connection channels error");
		} else {
			log::trace!(target: "pallet_ibc", "in channel: [store_connection_channels] >> init ChannelsConnection");
			let temp_connection_channels = vec![port_channel_id];
			<ChannelsConnection<T>>::insert(conn_id, temp_connection_channels);
		}

		Ok(())
	}

	/// Stores the given channel_end at a path associated with the port_id and channel_id.
	fn store_channel(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		channel_end: &ChannelEnd,
	) -> Result<(), ICS04Error> {
		// store channels key-value
		<Channels<T>>::insert(port_channel_id.0.clone(), port_channel_id.1, channel_end);

		Ok(())
	}

	fn store_next_sequence_send(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), ICS04Error> {
		let seq = u64::from(seq);

		<NextSequenceSend<T>>::insert(port_channel_id.0.clone(), port_channel_id.1, seq);

		Ok(())
	}

	fn store_next_sequence_recv(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), ICS04Error> {
		let seq = u64::from(seq);

		<NextSequenceRecv<T>>::insert(port_channel_id.0.clone(), port_channel_id.1, seq);

		Ok(())
	}

	fn store_next_sequence_ack(
		&mut self,
		port_channel_id: (PortId, ChannelId),
		seq: Sequence,
	) -> Result<(), ICS04Error> {
		let seq = u64::from(seq);

		<NextSequenceAck<T>>::insert(port_channel_id.0.clone(), port_channel_id.1, seq);

		Ok(())
	}

	/// Called upon channel identifier creation (Init or Try message processing).
	/// Increases the counter which keeps track of how many channels have been created.
	/// Should never fail.
	fn increase_channel_counter(&mut self) {
		log::trace!(target: "pallet_ibc", "in channel: [increase_channel_counter]");
		let _ = ChannelCounter::<T>::try_mutate::<_, (), _>(|val| {
			*val = val.saturating_add(1);
			Ok(())
		});
	}
}
