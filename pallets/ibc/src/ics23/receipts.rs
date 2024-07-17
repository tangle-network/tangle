use crate::{format, Config};
use frame_support::storage::{child, child::ChildInfo};
use ibc::core::{
	ics04_channel::packet::Sequence,
	ics24_host::{
		identifier::{ChannelId, PortId},
		path::ReceiptsPath,
	},
};
use ibc_primitives::apply_prefix;
use sp_core::Get;
use sp_std::{marker::PhantomData, prelude::*};

// todo: pruning
/// (port_id, channel_id, sequence) => hash
/// trie key path: "receipts/ports/{port_id}/channels/{channel_id}/sequences/{sequence}"
pub struct PacketReceipt<T>(PhantomData<T>);

impl<T: Config> PacketReceipt<T> {
	pub fn insert(
		(port_id, channel_id, sequence): (PortId, ChannelId, Sequence),
		receipt: Vec<u8>,
	) {
		let receipt_path = ReceiptsPath { port_id, channel_id, sequence };
		let receipt_path = format!("{receipt_path}");
		let receipt_key = apply_prefix(T::PalletPrefix::get(), vec![receipt_path]);
		child::put(&ChildInfo::new_default(T::PalletPrefix::get()), &receipt_key, &receipt)
	}

	pub fn get((port_id, channel_id, sequence): (PortId, ChannelId, Sequence)) -> Option<Vec<u8>> {
		let receipt_path = ReceiptsPath { port_id, channel_id, sequence };
		let receipt_path = format!("{receipt_path}");
		let receipt_key = apply_prefix(T::PalletPrefix::get(), vec![receipt_path]);
		child::get(&ChildInfo::new_default(T::PalletPrefix::get()), &receipt_key)
	}

	// pub fn remove((port_id, channel_id, sequence): (PortId, ChannelId, Sequence)) {
	// 	let receipt_path = ReceiptsPath { port_id, channel_id, sequence };
	// 	let receipt_path = format!("{}", receipt_path);
	// 	let receipt_key = apply_prefix_and_encode(T::PalletPrefix::get(), vec![receipt_path]);
	// 	child::kill(&ChildInfo::new_default(T::PalletPrefix::get()), &receipt_key)
	// }

	pub fn contains_key((port_id, channel_id, sequence): (PortId, ChannelId, Sequence)) -> bool {
		let receipt_path = ReceiptsPath { port_id, channel_id, sequence };
		let receipt_path = format!("{receipt_path}");
		let receipt_key = apply_prefix(T::PalletPrefix::get(), vec![receipt_path]);
		child::exists(&ChildInfo::new_default(T::PalletPrefix::get()), &receipt_key)
	}
}
