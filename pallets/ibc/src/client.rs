use super::*;

use crate::{
	ics23::{client_states::ClientStates, clients::Clients, consensus_states::ConsensusStates},
	impls::host_height,
	light_clients::{AnyClient, AnyClientMessage, AnyClientState, AnyConsensusState},
	routing::Context,
};

use frame_support::traits::Get;
use frame_system::pallet_prelude::BlockNumberFor;
use ibc::{
	core::{
		ics02_client::{
			client_consensus::ConsensusState,
			client_state::{ClientState, ClientType},
			context::{ClientKeeper, ClientReader, ClientTypes},
			error::Error as ICS02Error,
		},
		ics24_host::identifier::ClientId,
	},
	timestamp::Timestamp,
	Height,
};
use light_client_common::ChainType;
use sp_runtime::SaturatedConversion;
use tendermint_proto::Protobuf;

#[derive(Encode, Decode)]
pub struct HostConsensusProof {
	pub header: Vec<u8>,
	pub extrinsic: Vec<u8>,
	pub extrinsic_proof: Vec<Vec<u8>>,
	pub code_id: Option<Vec<u8>>,
}

impl<T: Config + Send + Sync> ClientReader for Context<T>
where
	BlockNumberFor<T>: From<u32>,
{
	fn client_type(&self, client_id: &ClientId) -> Result<ClientType, ICS02Error> {
		log::trace!(target: "pallet_ibc", "in client : [client_type] >> client_id = {:?}", client_id);

		if <Clients<T>>::contains_key(client_id) {
			let data = <Clients<T>>::get(client_id)
				.ok_or_else(|| ICS02Error::client_not_found(client_id.clone()))?;
			let data = String::from_utf8(data).map_err(|e| {
				ICS02Error::implementation_specific(format!(
					"[client_type]: error decoding client type bytes to string {e}"
				))
			})?;
			Ok(data)
		} else {
			log::trace!(target: "pallet_ibc", "in client : [client_type] >> read client_type is None");
			Err(ICS02Error::client_not_found(client_id.clone()))
		}
	}

	fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, ICS02Error> {
		log::trace!(target: "pallet_ibc", "in client : [client_state] >> client_id = {:?}", client_id);
		let data = <ClientStates<T>>::get(client_id)
			.ok_or_else(|| ICS02Error::client_not_found(client_id.clone()))?;
		let state = AnyClientState::decode_vec(&data)
			.map_err(|_| ICS02Error::client_not_found(client_id.clone()))?;
		log::trace!(target: "pallet_ibc", "in client : [client_state] >> any client_state: {:?}", state);
		Ok(state)
	}

	fn consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<AnyConsensusState, ICS02Error> {
		log::trace!(target: "pallet_ibc",
			"in client : [consensus_state] >> client_id = {:?}, height = {:?}",
			client_id,
			height
		);

		let native_height = height;
		let value = <ConsensusStates<T>>::get(client_id.clone(), height)
			.ok_or_else(|| ICS02Error::consensus_state_not_found(client_id.clone(), height))?;

		let any_consensus_state = AnyConsensusState::decode_vec(&value)
			.map_err(|_| ICS02Error::consensus_state_not_found(client_id.clone(), native_height))?;
		log::trace!(target: "pallet_ibc",
			"in client : [consensus_state] >> any consensus state = {:?}",
			any_consensus_state
		);
		Ok(any_consensus_state)
	}

	fn host_client_type(&self) -> String {
		// todo: https://github.com/cosmos/ibc/pull/839
		if cfg!(any(test, feature = "runtime-benchmarks")) {
			"tendermint".to_string()
		} else {
			match <T as Config>::LightClientProtocol::get() {
				LightClientProtocol::Beefy => "beefy".to_string(),
				LightClientProtocol::Grandpa => "grandpa".to_string(),
				LightClientProtocol::GrandpaStandalone => "grandpa_standalone".to_string(),
			}
		}
	}

	fn next_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Option<AnyConsensusState>, ICS02Error> {
		let consensus_heights = ConsensusHeights::<T>::get(client_id.as_bytes().to_vec());
		let cs_state = consensus_heights
			.into_iter()
			.find(|next_height| next_height > &height)
			.and_then(|next_height| self.consensus_state(client_id, next_height).ok());

		Ok(cs_state)
	}

	fn prev_consensus_state(
		&self,
		client_id: &ClientId,
		height: Height,
	) -> Result<Option<AnyConsensusState>, ICS02Error> {
		let consensus_heights = ConsensusHeights::<T>::get(client_id.as_bytes().to_vec());
		let cs_state = consensus_heights
			.into_iter()
			.filter(|prev_height| prev_height < &height)
			.next_back()
			.and_then(|prev_height| self.consensus_state(client_id, prev_height).ok());
		Ok(cs_state)
	}

	fn host_height(&self) -> Height {
		log::trace!(target: "pallet_ibc", "in client: [host_height]");
		let current_height = host_height::<T>();
		let para_id: u32 = T::ChainId::get().into();
		Height::new(para_id.into(), current_height)
	}

	#[allow(clippy::disallowed_methods)]
	fn host_timestamp(&self) -> Timestamp {
		use frame_support::traits::UnixTime;
		let time = T::TimeProvider::now();
		let ts = Timestamp::from_nanoseconds(time.as_nanos().saturated_into::<u64>())
			.map_err(|e| panic!("{e:?}, caused by {time:?} from pallet timestamp_pallet"));
		// Should not panic, if timestamp is invalid after the genesis block then there's a major
		// error in pallet timestamp
		ts.unwrap()
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn host_consensus_state(
		&self,
		_height: Height,
		_proof: Option<Vec<u8>>,
		_client_state: &AnyClientState,
	) -> Result<AnyConsensusState, ICS02Error> {
		let timestamp = Timestamp::from_nanoseconds(1).unwrap();
		let timestamp = timestamp.into_tm_time().unwrap();

		let consensus_state = match <T as Config>::LightClientProtocol::get() {
			crate::LightClientProtocol::Beefy => {
				AnyConsensusState::Beefy(ics11_beefy::consensus_state::ConsensusState {
					timestamp,
					root: vec![].into(),
				})
			},
			crate::LightClientProtocol::Grandpa => {
				AnyConsensusState::Grandpa(ics10_grandpa::consensus_state::ConsensusState {
					timestamp,
					root: vec![].into(),
				})
			},
			crate::LightClientProtocol::GrandpaStandalone => AnyConsensusState::GrandpaStandalone(
				ics10_grandpa_standalone::consensus_state::ConsensusState {
					timestamp,
					root: vec![].into(),
				},
			),
		};
		Ok(consensus_state)
	}

	#[cfg(not(feature = "runtime-benchmarks"))]
	fn host_consensus_state(
		&self,
		height: Height,
		proof: Option<Vec<u8>>,
		client_state: &AnyClientState,
	) -> Result<AnyConsensusState, ICS02Error> {
		log::trace!(target: "pallet_ibc", "in client: [host_consensus_state] height = {:?}", height);
		use parity_scale_codec::Compact;
		use sp_core::H256;
		use sp_runtime::traits::{BlakeTwo256, Header};
		use sp_trie::LayoutV0;
		let proof = proof.ok_or_else(|| {
			ICS02Error::implementation_specific(format!("No host proof supplied"))
		})?;

		// unfortunately we can't access headers on-chain, but we can verify them using
		// frame_system's cache of header hashes
		let height = u32::try_from(height.revision_height).map_err(|_| {
			ICS02Error::implementation_specific(format!(
				"[host_consensus_state]: Can't fit height: {height} in u32"
			))
		})?;
		let header_hash = frame_system::BlockHash::<T>::get(BlockNumberFor::<T>::from(height));
		// we don't even have the hash for this height (anymore?)
		if header_hash == <T as frame_system::Config>::Hash::default() {
			Err(ICS02Error::implementation_specific(format!(
				"[host_consensus_state]: Unknown height {height}"
			)))?
		}

		let connection_proof: HostConsensusProof =
			parity_scale_codec::Decode::decode(&mut &proof[..]).map_err(|e| {
				ICS02Error::implementation_specific(format!(
					"[host_consensus_state]: Failed to decode proof: {e}"
				))
			})?;
		let header =
			frame_system::pallet_prelude::HeaderFor::<T>::decode(&mut &connection_proof.header[..])
				.map_err(|e| {
					ICS02Error::implementation_specific(format!(
						"[host_consensus_state]: Failed to decode header: {e:?}"
					))
				})?;
		if header.hash() != header_hash {
			Err(ICS02Error::implementation_specific(format!(
				"[host_consensus_state]: Incorrect host consensus state for height {height}"
			)))?
		}
		let timestamp = {
			// verify timestamp extrinsic
			let ext = &*connection_proof.extrinsic;
			let proof = &*connection_proof.extrinsic_proof;
			let extrinsic_root = <[u8; 32]>::try_from(header.extrinsics_root().as_ref())
				.expect("header has been verified; qed");
			// Timestamp extrinsic should be the first inherent and hence the first extrinsic
			// https://github.com/paritytech/substrate/blob/d602397a0bbb24b5d627795b797259a44a5e29e9/primitives/trie/src/lib.rs#L99-L101
			let key = parity_scale_codec::Compact(0u32).encode();
			sp_trie::verify_trie_proof::<LayoutV0<BlakeTwo256>, _, _, _>(
				&H256::from(extrinsic_root),
				proof,
				&vec![(key, Some(ext))],
			)
			.map_err(|_err| {
				ICS02Error::implementation_specific(format!("Invalid extrinsic proof"))
			})?;

			let (_, _, timestamp): (u8, u8, Compact<u64>) =
				parity_scale_codec::Decode::decode(&mut &ext[2..]).map_err(|err| {
					ICS02Error::implementation_specific(format!(
						"Failed to decode extrinsic: {err:?}"
					))
				})?;

			let duration = core::time::Duration::from_millis(timestamp.into());
			Timestamp::from_nanoseconds(duration.as_nanos().saturated_into::<u64>())
				.map_err(|e| {
					ICS02Error::implementation_specific(format!(
						"[host_consensus_state]: error decoding timestamp {e:?}"
					))
				})?
				.into_tm_time()
				.ok_or_else(|| {
					ICS02Error::implementation_specific(
						"[host_consensus_state]: Could not convert timestamp into tendermint time"
							.to_string(),
					)
				})?
		};

		// now this header can be trusted
		let consensus_state = match <T as Config>::LightClientProtocol::get() {
			crate::LightClientProtocol::Beefy => {
				let cs_state = ics11_beefy::consensus_state::ConsensusState {
					timestamp,
					root: header.state_root().as_ref().to_vec().into(),
				};
				AnyConsensusState::Beefy(cs_state)
			},
			crate::LightClientProtocol::Grandpa => {
				let cs_state = ics10_grandpa::consensus_state::ConsensusState {
					timestamp,
					root: header.state_root().as_ref().to_vec().into(),
				};
				let cs = AnyConsensusState::Grandpa(cs_state);

				match &client_state {
					AnyClientState::Wasm(_wasm) => {
						log::trace!(target: "pallet_ibc", "in client : [host_consensus_state] >> using wasm code id" );
						AnyConsensusState::wasm(cs).map_err(ICS02Error::encode)?
					},
					_ => {
						if connection_proof.code_id.is_some() {
							log::trace!(target: "pallet_ibc", "in client : [host_consensus_state] >> using wasm code id");
							AnyConsensusState::wasm(cs).map_err(ICS02Error::encode)?
						} else {
							cs
						}
					},
				}
			},
			crate::LightClientProtocol::GrandpaStandalone => {
				let cs_state = ics10_grandpa_standalone::consensus_state::ConsensusState {
					timestamp,
					root: header.state_root().as_ref().to_vec().into(),
				};
				let cs = AnyConsensusState::GrandpaStandalone(cs_state);

				match &client_state {
					AnyClientState::Wasm(_wasm) => {
						log::trace!(target: "pallet_ibc", "in client : [host_consensus_state] >> using wasm code id" );
						AnyConsensusState::wasm(cs).map_err(ICS02Error::encode)?
					},
					_ => {
						if connection_proof.code_id.is_some() {
							log::trace!(target: "pallet_ibc", "in client : [host_consensus_state] >> using wasm code id");
							AnyConsensusState::wasm(cs).map_err(ICS02Error::encode)?
						} else {
							cs
						}
					},
				}
			},
		};
		Ok(consensus_state)
	}

	fn client_counter(&self) -> Result<u64, ICS02Error> {
		let count = ClientCounter::<T>::get();
		log::trace!(target: "pallet_ibc", "in client : [client_counter] >> client_counter: {:?}", count);

		Ok(count as u64)
	}
}

impl<T: Config> ClientTypes for Context<T> {
	type AnyClientMessage = AnyClientMessage;
	type AnyClientState = AnyClientState;
	type AnyConsensusState = AnyConsensusState;
	type ClientDef = AnyClient;
}

impl<T: Config + Send + Sync> ClientKeeper for Context<T> {
	fn store_client_type(
		&mut self,
		client_id: ClientId,
		client_type: ClientType,
	) -> Result<(), ICS02Error> {
		log::trace!(target: "pallet_ibc",
			"in client : [store_client_type] >> client id = {:?}, client_type = {:?}",
			client_id,
			client_type
		);

		let client_type = client_type.as_bytes().to_vec();
		<Clients<T>>::insert(&client_id, client_type);
		Ok(())
	}

	fn store_client_state(
		&mut self,
		client_id: ClientId,
		client_state: AnyClientState,
	) -> Result<(), ICS02Error> {
		log::trace!(target: "pallet_ibc",
			"in client : [store_client_state] >> client_id: {:?}, client_state = {:?}",
			client_id,
			client_state
		);

		let data = client_state.encode_to_vec().map_err(ICS02Error::encode)?;
		// store client states key-value
		<ClientStates<T>>::insert(&client_id, data);

		Ok(())
	}

	fn store_consensus_state(
		&mut self,
		client_id: ClientId,
		height: Height,
		consensus_state: AnyConsensusState,
	) -> Result<(), ICS02Error> {
		log::trace!(target: "pallet_ibc", "in client : [store_consensus_state] >> client_id: {:?}, height = {:?}, consensus_state = {:?}",
			client_id, height, consensus_state);

		let data = consensus_state.encode_to_vec().map_err(ICS02Error::encode)?;
		// todo: pruning
		ConsensusStates::<T>::insert(client_id.clone(), height, data);
		// We do not need this hack for neither beefy nor grandpa clients
		if !client_id.as_str().starts_with("10-grandpa")
			&& !client_id.as_str().starts_with("11-beefy")
		{
			let mut stored_heights = ConsensusHeights::<T>::get(client_id.as_bytes().to_vec());
			if let Err(val) = stored_heights.try_insert(height) {
				let first =
					*stored_heights.iter().next().expect("Cannot fail as a value always exists");
				stored_heights.remove(&first);
				stored_heights
					.try_insert(val)
					.expect("Cannot panic, since bounds cannot be exceeded at this point");
			}
			ConsensusHeights::<T>::insert(client_id.as_bytes().to_vec(), stored_heights);
		}

		Ok(())
	}

	fn increase_client_counter(&mut self) {
		log::trace!(target: "pallet_ibc", "in client : [increase_client_counter]");
		// increment counter
		if let Some(val) = <ClientCounter<T>>::get().checked_add(1) {
			<ClientCounter<T>>::put(val);
		}
	}

	fn store_update_time(
		&mut self,
		client_id: ClientId,
		height: Height,
		timestamp: Timestamp,
	) -> Result<(), ICS02Error> {
		log::trace!(target: "pallet_ibc", "in client: [store_update_time] >> Client Height {:?}, Host Timestamp {:?} ", height, timestamp);
		let height = height.encode_vec().map_err(ICS02Error::encode)?;
		let timestamp = timestamp.nanoseconds();
		let client_id = client_id.as_bytes().to_vec();
		ClientUpdateTime::<T>::insert(client_id, height, timestamp);
		Ok(())
	}

	fn store_update_height(
		&mut self,
		client_id: ClientId,
		height: Height,
		host_height: Height,
	) -> Result<(), ICS02Error> {
		log::trace!(target: "pallet_ibc", "in client: [store_update_height] >> Client Height {:?}, Host Height {:?} ", height, host_height);
		let height = height.encode_vec().map_err(ICS02Error::encode)?;
		let host_height = host_height.encode_vec().map_err(ICS02Error::encode)?;
		let client_id = client_id.as_bytes().to_vec();
		ClientUpdateHeight::<T>::insert(client_id, height, host_height);
		Ok(())
	}

	fn validate_self_client(&self, client_state: &AnyClientState) -> Result<(), ICS02Error> {
		let unpacked = client_state.unpack_recursive();
		let (chain, chain_id, latest_height) = match unpacked {
			AnyClientState::Beefy(client_state) => {
				if client_state.frozen_height.is_some() {
					Err(ICS02Error::implementation_specific(format!("client state is frozen")))?
				}

				(
					ChainType::RelayChain(client_state.relay_chain),
					client_state.para_id,
					client_state.latest_para_height,
				)
			},
			AnyClientState::Grandpa(client_state) => {
				if client_state.frozen_height.is_some() {
					Err(ICS02Error::implementation_specific(format!("client state is frozen")))?
				}

				(
					ChainType::RelayChain(client_state.relay_chain),
					client_state.para_id,
					client_state.latest_para_height,
				)
			},
			AnyClientState::GrandpaStandalone(client_state) => {
				if client_state.frozen_height.is_some() {
					Err(ICS02Error::implementation_specific(format!("client state is frozen")))?
				}

				(
					ChainType::StandaloneChain(client_state.chain),
					client_state.chain_id,
					client_state.latest_height,
				)
			},
			client => Err(ICS02Error::unknown_client_type(format!("{}", client.client_type())))?,
		};

		if chain != T::ChainType::get() {
			log::warn!(target : "pallet_ibc", "configured relay {} differs from provided in data {}", T::ChainType::get(), chain);
			Err(ICS02Error::implementation_specific(format!("relay chain mis-match")))?
		}

		let self_chain_id: u32 = T::ChainId::get().into();
		if chain_id != self_chain_id {
			log::warn!(target : "pallet_ibc", "configured para {} differs from provided in data {}", self_chain_id, chain_id);
			Err(ICS02Error::implementation_specific(format!("para-id mis-match")))?
		}

		let block_number: u32 = <frame_system::Pallet<T>>::block_number().into().as_u32();

		// this really shouldn't be possible
		if latest_height >= block_number {
			Err(ICS02Error::implementation_specific(format!(
				"client has latest_height {latest_height} greater than or equal to chain height {block_number}"
			)))?
		}

		Ok(())
	}
}
