use crate::{format, Config};
use frame_support::storage::{child, child::ChildInfo};
use ibc::{
	core::ics24_host::{identifier::ClientId, path::ClientConsensusStatePath},
	Height,
};
use ibc_primitives::apply_prefix;
use sp_core::Get;
use sp_std::{marker::PhantomData, prelude::*};

/// client_id, height => consensus_state
/// trie key path: "clients/{client_id}/consensusStates/{height}"
/// todo: only store up to 250 (height => consensus_state) per client_id
pub struct ConsensusStates<T>(PhantomData<T>);

impl<T: Config> ConsensusStates<T> {
	pub fn get(client_id: ClientId, height: Height) -> Option<Vec<u8>> {
		let consensus_path = ClientConsensusStatePath {
			client_id,
			epoch: height.revision_number,
			height: height.revision_height,
		};
		let path = format!("{consensus_path}");
		let key = apply_prefix(T::PalletPrefix::get(), vec![path]);
		child::get(&ChildInfo::new_default(T::PalletPrefix::get()), &key)
	}

	pub fn insert(client_id: ClientId, height: Height, consensus_state: Vec<u8>) {
		let consensus_path = ClientConsensusStatePath {
			client_id,
			epoch: height.revision_number,
			height: height.revision_height,
		};
		let path = format!("{consensus_path}");
		let key = apply_prefix(T::PalletPrefix::get(), vec![path]);
		child::put(&ChildInfo::new_default(T::PalletPrefix::get()), &key, &consensus_state)
	}
}
