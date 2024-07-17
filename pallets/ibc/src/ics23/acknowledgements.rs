use crate::{format, Config};
use frame_support::storage::{child, child::ChildInfo, ChildTriePrefixIterator};
use ibc::core::{
	ics04_channel::{commitment::AcknowledgementCommitment, packet::Sequence},
	ics24_host::{
		identifier::{ChannelId, PortId},
		path::AcksPath,
		Path,
	},
};
use ibc_primitives::apply_prefix;
use sp_core::Get;
use sp_std::{marker::PhantomData, prelude::*, str::FromStr};

/// (port_id, channel_id, sequence) => hash
/// trie key path: "acks/ports/{port_id}/channels/{channel_id}/sequences/{sequence}"
pub struct Acknowledgements<T>(PhantomData<T>);

impl<T: Config> Acknowledgements<T> {
	pub fn insert(
		(port_id, channel_id, sequence): (PortId, ChannelId, Sequence),
		ack: AcknowledgementCommitment,
	) {
		let ack_path = AcksPath { port_id, channel_id, sequence };
		let ack_path = format!("{ack_path}");
		let ack_key = apply_prefix(T::PalletPrefix::get(), vec![ack_path]);
		child::put(&ChildInfo::new_default(T::PalletPrefix::get()), &ack_key, &ack.into_vec())
	}

	pub fn get((port_id, channel_id, sequence): (PortId, ChannelId, Sequence)) -> Option<Vec<u8>> {
		let ack_path = AcksPath { port_id, channel_id, sequence };
		let ack_path = format!("{ack_path}");
		let ack_key = apply_prefix(T::PalletPrefix::get(), vec![ack_path]);
		child::get(&ChildInfo::new_default(T::PalletPrefix::get()), &ack_key)
	}

	pub fn remove((port_id, channel_id, sequence): (PortId, ChannelId, Sequence)) {
		let ack_path = AcksPath { port_id, channel_id, sequence };
		let ack_path = format!("{ack_path}");
		let ack_key = apply_prefix(T::PalletPrefix::get(), vec![ack_path]);
		child::kill(&ChildInfo::new_default(T::PalletPrefix::get()), &ack_key)
	}

	pub fn contains_key((port_id, channel_id, sequence): (PortId, ChannelId, Sequence)) -> bool {
		let ack_path = AcksPath { port_id, channel_id, sequence };
		let ack_path = format!("{ack_path}");
		let ack_key = apply_prefix(T::PalletPrefix::get(), vec![ack_path]);
		child::exists(&ChildInfo::new_default(T::PalletPrefix::get()), &ack_key)
	}

	// WARNING: too expensive to be called from an on-chain context, only here for rpc layer.
	pub fn iter() -> impl Iterator<Item = ((PortId, ChannelId, Sequence), Vec<u8>)> {
		let prefix = "acks/ports/".to_string();
		let prefix_key = apply_prefix(T::PalletPrefix::get(), vec![prefix.clone()]);
		ChildTriePrefixIterator::with_prefix(
			&ChildInfo::new_default(T::PalletPrefix::get()),
			&prefix_key,
		)
		.filter_map(move |(remaining_key, value)| {
			let path = format!("{prefix}{}", String::from_utf8(remaining_key).ok()?);
			if let Path::Acks(AcksPath { port_id, channel_id, sequence }) =
				Path::from_str(&path).ok()?
			{
				return Some(((port_id, channel_id, sequence), value));
			}
			None
		})
	}
}
