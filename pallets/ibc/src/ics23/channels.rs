use crate::{format, Config};
use frame_support::storage::{child, child::ChildInfo, ChildTriePrefixIterator};
use ibc::core::{
	ics04_channel::channel::ChannelEnd,
	ics24_host::{
		identifier::{ChannelId, PortId},
		path::ChannelEndsPath,
		Path,
	},
};
use ibc_primitives::apply_prefix;
use sp_core::Get;
use sp_std::{marker::PhantomData, prelude::*, str::FromStr};
use tendermint_proto::Protobuf;

// todo: pruning
/// (port_id, channel_id) => ChannelEnd
/// trie key path: "channelEnds/ports/{port_id}/channels/{channel_id}"
pub struct Channels<T>(PhantomData<T>);

impl<T: Config> Channels<T> {
	pub fn get(port_id: PortId, channel_id: ChannelId) -> Option<Vec<u8>> {
		let channel_path = format!("{}", ChannelEndsPath(port_id, channel_id));
		let channel_key = apply_prefix(T::PalletPrefix::get(), vec![channel_path]);
		child::get(&ChildInfo::new_default(T::PalletPrefix::get()), &channel_key)
	}

	pub fn insert(port_id: PortId, channel_id: ChannelId, channel_end: &ChannelEnd) {
		let channel_path = format!("{}", ChannelEndsPath(port_id, channel_id));
		let channel_key = apply_prefix(T::PalletPrefix::get(), vec![channel_path]);
		child::put(
			&ChildInfo::new_default(T::PalletPrefix::get()),
			&channel_key,
			&channel_end.encode_vec().expect("encode channel end"),
		);
	}

	// WARNING: too expensive to be called from an on-chain context, only here for rpc layer.
	pub fn iter() -> impl Iterator<Item = (Vec<u8>, Vec<u8>, Vec<u8>)> {
		let prefix = "channelEnds/ports/".to_string();
		let key = apply_prefix(T::PalletPrefix::get(), vec![prefix.clone()]);
		ChildTriePrefixIterator::with_prefix(&ChildInfo::new_default(T::PalletPrefix::get()), &key)
			.filter_map(move |(key, value)| {
				let path = format!("{prefix}{}", String::from_utf8(key).ok()?);
				if let Path::ChannelEnds(ChannelEndsPath(port_id, channel_id)) =
					Path::from_str(&path).ok()?
				{
					return Some((
						port_id.as_bytes().to_vec(),
						channel_id.to_string().as_bytes().to_vec(),
						value,
					));
				}
				None
			})
	}
}
