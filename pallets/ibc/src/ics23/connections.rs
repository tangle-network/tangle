use crate::{format, Config};
use frame_support::storage::{child, child::ChildInfo, ChildTriePrefixIterator};
use ibc::core::{
	ics03_connection::connection::ConnectionEnd,
	ics24_host::{identifier::ConnectionId, path::ConnectionsPath},
};
use ibc_primitives::apply_prefix;
use sp_core::Get;
use sp_std::{marker::PhantomData, prelude::*};
use tendermint_proto::Protobuf;

// todo: pruning
/// connection_id => ConnectionEnd
/// trie key path: "connections/{}"
pub struct Connections<T>(PhantomData<T>);

impl<T: Config> Connections<T> {
	pub fn get(connection_id: &ConnectionId) -> Option<Vec<u8>> {
		let connection_path = format!("{}", ConnectionsPath(connection_id.clone()));
		let connection_key = apply_prefix(T::PalletPrefix::get(), vec![connection_path]);
		child::get(&ChildInfo::new_default(T::PalletPrefix::get()), &connection_key)
	}

	pub fn insert(connection_id: &ConnectionId, connection_end: &ConnectionEnd) {
		let connection_path = format!("{}", ConnectionsPath(connection_id.clone()));
		let connection_key = apply_prefix(T::PalletPrefix::get(), vec![connection_path]);
		child::put(
			&ChildInfo::new_default(T::PalletPrefix::get()),
			&connection_key,
			&connection_end.encode_vec().expect("encode connection end"),
		);
	}

	// WARNING: too expensive to be called from an on-chain context, only here for rpc layer.
	pub fn iter() -> ChildTriePrefixIterator<(Vec<u8>, Vec<u8>)> {
		let prefix_path = "connections/".to_string();
		let key = apply_prefix(T::PalletPrefix::get(), vec![prefix_path]);
		ChildTriePrefixIterator::with_prefix(&ChildInfo::new_default(T::PalletPrefix::get()), &key)
	}
}
