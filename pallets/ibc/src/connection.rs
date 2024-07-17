use super::*;
use core::time::Duration;

use crate::{ics23::connections::Connections, routing::Context};
use frame_support::traits::Get;
use ibc::{
	core::{
		ics03_connection::{
			connection::ConnectionEnd,
			context::{ConnectionKeeper, ConnectionReader},
			error::Error as ICS03Error,
		},
		ics23_commitment::commitment::CommitmentPrefix,
		ics24_host::identifier::{ClientId, ConnectionId},
	},
	Height,
};
use tendermint_proto::Protobuf;

impl<T: Config + Sync + Send> ConnectionReader for Context<T> {
	fn minimum_delay_period(&self) -> Duration {
		Duration::from_secs(T::MinimumConnectionDelay::get())
	}

	fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ICS03Error> {
		log::trace!(target: "pallet_ibc", "in connection : [connection_end] >> connection_id = {:?}", conn_id);

		let data = <Connections<T>>::get(conn_id)
			.ok_or_else(|| ICS03Error::connection_not_found(conn_id.clone()))?;
		let ret = ConnectionEnd::decode_vec(&data)
			.map_err(|_| ICS03Error::connection_mismatch(conn_id.clone()))?;
		log::trace!(target: "pallet_ibc", "in connection : [connection_end] >>  connection_end = {:?}", ret);
		Ok(ret)
	}

	fn host_oldest_height(&self) -> Height {
		let mut temp = frame_system::BlockHash::<T>::iter().collect::<Vec<_>>();
		temp.sort_by(|(a, ..), (b, ..)| a.cmp(b));
		let (block_number, ..) = temp.get(0).cloned().unwrap_or_default();
		let block_number = format!("{block_number:?}");
		let height = block_number.parse().unwrap_or_default();
		let para_id: u32 = T::ChainId::get().into();
		log::trace!(target: "pallet_ibc",
			"in connection : [host_oldest_height] >> Host oldest height = {:?}",
			Height::new(para_id.into(), height)
		);
		Height::new(para_id.into(), height)
	}

	#[allow(clippy::disallowed_methods)]
	fn commitment_prefix(&self) -> CommitmentPrefix {
		log::trace!(target: "pallet_ibc", "in connection : [commitment_prefix] >> CommitmentPrefix = {:?}", "ibc");
		// If this conversion fails it means the runtime was not configured well
		T::PalletPrefix::get()
			.to_vec()
			.try_into()
			.map_err(|_| panic!("Connection prefix supplied in pallet runtime config is invalid"))
			.unwrap()
	}

	fn connection_counter(&self) -> Result<u64, ICS03Error> {
		let count = ConnectionCounter::<T>::get();
		log::trace!(target: "pallet_ibc", "in connection : [connection_counter] >> Connection_counter = {:?}", count);

		Ok(count as u64)
	}
}

impl<T: Config + Sync + Send> ConnectionKeeper for Context<T> {
	fn store_connection(
		&mut self,
		connection_id: ConnectionId,
		connection_end: &ConnectionEnd,
	) -> Result<(), ICS03Error> {
		log::trace!(target: "pallet_ibc",
			"in connection : [store_connection] >> connection_id: {:?}, connection_end: {:?}",
			connection_id,
			connection_end
		);

		<Connections<T>>::insert(&connection_id, connection_end);

		let temp = ConnectionReader::connection_end(self, &connection_id);
		log::trace!(target: "pallet_ibc", "in connection : [store_connection] >> read store after: {:?}", temp);
		Ok(())
	}

	fn store_connection_to_client(
		&mut self,
		connection_id: ConnectionId,
		client_id: &ClientId,
	) -> Result<(), ICS03Error> {
		log::trace!(target: "pallet_ibc",
			"in connection : [store_connection_to_client] >> connection_id = {:?},\
		 client_id = {:?}",
			connection_id,
			client_id
		);

		ConnectionClient::<T>::try_mutate::<_, _, ICS03Error, _>(
			client_id.as_bytes().to_vec(),
			|val| {
				val.push(connection_id.as_bytes().to_vec());
				Ok(())
			},
		)
	}

	fn increase_connection_counter(&mut self) {
		log::trace!(target: "pallet_ibc", "in connection : [increase_connection_counter]");
		// connections uses a counted storage map
		if let Some(val) = <ConnectionCounter<T>>::get().checked_add(1) {
			<ConnectionCounter<T>>::put(val);
		}
	}
}
