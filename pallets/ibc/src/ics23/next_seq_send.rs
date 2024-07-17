use crate::{format, Config};
use alloc::vec;
use frame_support::storage::{child, child::ChildInfo};
use ibc::core::ics24_host::{
	identifier::{ChannelId, PortId},
	path::SeqSendsPath,
};
use ibc_primitives::apply_prefix;
use sp_core::Get;
use sp_std::marker::PhantomData;

// todo: pruning
/// (port_id, channel_id) => Sequence
/// trie key path: "nextSequenceSend/ports/{port_id}/channels/{channel_id}"
pub struct NextSequenceSend<T>(PhantomData<T>);

impl<T: Config> NextSequenceSend<T> {
	pub fn get(port_id: PortId, channel_id: ChannelId) -> Option<u64> {
		let next_seq_send_path = format!("{}", SeqSendsPath(port_id, channel_id));
		let next_seq_send_key = apply_prefix(T::PalletPrefix::get(), vec![next_seq_send_path]);
		child::get(&ChildInfo::new_default(T::PalletPrefix::get()), &next_seq_send_key)
	}

	pub fn insert(port_id: PortId, channel_id: ChannelId, seq: u64) {
		let next_seq_send_path = format!("{}", SeqSendsPath(port_id, channel_id));
		let next_seq_send_key = apply_prefix(T::PalletPrefix::get(), vec![next_seq_send_path]);
		child::put(&ChildInfo::new_default(T::PalletPrefix::get()), &next_seq_send_key, &seq)
	}
}
