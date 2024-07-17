use crate::{format, Config};
use alloc::vec;
use frame_support::storage::{child, child::ChildInfo};
use ibc::core::ics24_host::{
	identifier::{ChannelId, PortId},
	path::SeqAcksPath,
};
use ibc_primitives::apply_prefix;
use sp_core::Get;
use sp_std::marker::PhantomData;

// todo: pruning
/// (port_id, channel_id) => Sequence
/// trie key path: "nextSequenceAck/ports/{port_id}/channels/{channel_id}"
pub struct NextSequenceAck<T>(PhantomData<T>);

impl<T: Config> NextSequenceAck<T> {
	pub fn get(port_id: PortId, channel_id: ChannelId) -> Option<u64> {
		let next_seq_ack_path = format!("{}", SeqAcksPath(port_id, channel_id));
		let next_seq_ack_key = apply_prefix(T::PalletPrefix::get(), vec![next_seq_ack_path]);
		child::get(&ChildInfo::new_default(T::PalletPrefix::get()), &next_seq_ack_key)
	}

	pub fn insert(port_id: PortId, channel_id: ChannelId, seq: u64) {
		let next_seq_ack_path = format!("{}", SeqAcksPath(port_id, channel_id));
		let next_seq_ack_key = apply_prefix(T::PalletPrefix::get(), vec![next_seq_ack_path]);
		child::put(&ChildInfo::new_default(T::PalletPrefix::get()), &next_seq_ack_key, &seq)
	}
}
