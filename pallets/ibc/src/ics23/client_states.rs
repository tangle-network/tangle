use crate::{format, Config};
use frame_support::storage::{child, child::ChildInfo, ChildTriePrefixIterator};
use ibc::core::ics24_host::{identifier::ClientId, path::ClientStatePath, Path};
use ibc_primitives::apply_prefix;
use sp_core::Get;
use sp_std::{marker::PhantomData, prelude::*, str::FromStr};

/// client_id => client_states
/// trie key path: "clients/{client_id}/clientState"
pub struct ClientStates<T>(PhantomData<T>);

impl<T: Config> ClientStates<T> {
	pub fn get(client_id: &ClientId) -> Option<Vec<u8>> {
		let client_state_path = format!("{}", ClientStatePath(client_id.clone()));
		let client_state_key = apply_prefix(T::PalletPrefix::get(), vec![client_state_path]);
		child::get(&ChildInfo::new_default(T::PalletPrefix::get()), &client_state_key)
	}

	pub fn insert(client_id: &ClientId, client_state: Vec<u8>) {
		let client_state_path = format!("{}", ClientStatePath(client_id.clone()));
		let client_state_key = apply_prefix(T::PalletPrefix::get(), vec![client_state_path]);
		child::put(
			&ChildInfo::new_default(T::PalletPrefix::get()),
			&client_state_key,
			&client_state,
		);
	}

	pub fn _contains_key(client_id: &ClientId) -> bool {
		let client_state_path = format!("{}", ClientStatePath(client_id.clone()));
		let client_state_key = apply_prefix(T::PalletPrefix::get(), vec![client_state_path]);
		child::exists(&ChildInfo::new_default(T::PalletPrefix::get()), &client_state_key)
	}

	// WARNING: too expensive to be called from an on-chain context, only here for rpc layer.
	// client_id => client_state
	pub fn iter() -> impl Iterator<Item = (ClientId, Vec<u8>)> {
		let prefix_path = "clients/".to_string();
		let key = apply_prefix(T::PalletPrefix::get(), vec![prefix_path.clone()]);
		ChildTriePrefixIterator::with_prefix(&ChildInfo::new_default(T::PalletPrefix::get()), &key)
			.filter_map(move |(remaining_key, value)| {
				let path = format!("{prefix_path}{}", String::from_utf8(remaining_key).ok()?);
				if let Path::ClientState(ClientStatePath(client_id)) = Path::from_str(&path).ok()? {
					return Some((client_id, value));
				}
				None
			})
	}
}
