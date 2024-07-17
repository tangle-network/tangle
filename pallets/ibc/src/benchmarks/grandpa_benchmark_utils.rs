use crate::light_clients::{AnyClientMessage, HostFunctionsManager};
use alloc::{collections::BTreeMap, format};
use finality_grandpa::{Precommit, SignedPrecommit};
use grandpa_client_primitives::{
	justification::GrandpaJustification, parachain_header_storage_key, Commit, FinalityProof,
	ParachainHeaderProofs,
};
use ibc::{timestamp::Timestamp, Height};
use ics10_grandpa::{
	client_message::{ClientMessage, Header as GrandpaHeader, RelayChainHeader},
	client_state::ClientState,
	consensus_state::ConsensusState,
};
use parity_scale_codec::{Compact, Encode};
use sp_consensus_grandpa::{AuthorityId, AuthoritySignature, KEY_TYPE};
use sp_core::H256;
use sp_runtime::{traits::BlakeTwo256, SaturatedConversion};
use sp_std::prelude::*;
use sp_trie::{generate_trie_proof, LayoutV0, MemoryDB, StorageProof, TrieDBMutBuilder, TrieMut};

pub const GRANDPA_UPDATE_TIMESTAMP: u64 = 1650894363;
/// Builds a grandpa client message that that contains the requested number of precommits
pub fn generate_finality_proof(
	pre_commits: u32,
	unknown_headers: u32,
) -> (ClientState<HostFunctionsManager>, ConsensusState, AnyClientMessage) {
	let para_id = 2000u32;
	let latest_para_height = 1u32;
	let relay_chain_hash = H256::zero();
	let latest_relay_height = 1u32;
	let round = 1;
	let set_id = 1;

	// Build timestamp extrinsic with proof
	let mut para_db = MemoryDB::<BlakeTwo256>::default();

	let mut timestamp_extrinsic =
		(1u8, 0u8, Compact(GRANDPA_UPDATE_TIMESTAMP.saturating_mul(10000))).encode();
	timestamp_extrinsic.insert(0, 0);
	timestamp_extrinsic.insert(0, 0);
	let key = Compact(0u64).encode();
	let extrinsics_root = {
		let mut root = Default::default();
		let mut trie =
			<TrieDBMutBuilder<LayoutV0<BlakeTwo256>>>::new(&mut para_db, &mut root).build();
		trie.insert(&key, &timestamp_extrinsic).unwrap();
		*trie.root()
	};
	let extrinsic_proof = generate_trie_proof::<LayoutV0<BlakeTwo256>, _, _, _>(
		&para_db,
		extrinsics_root,
		vec![&key],
	)
	.unwrap();

	// build a chain of relaychain blocks
	// construct a state root from the parachain header

	let parachain_header = sp_runtime::generic::Header::<_, BlakeTwo256> {
		parent_hash: Default::default(),
		number: latest_para_height + 1,
		state_root: Default::default(),
		extrinsics_root,
		digest: Default::default(),
	};

	let mut para_db = MemoryDB::<BlakeTwo256>::default();
	let key = parachain_header_storage_key(para_id);
	let mut root = Default::default();
	let state_root = {
		let mut trie =
			TrieDBMutBuilder::<LayoutV0<BlakeTwo256>>::new(&mut para_db, &mut root).build();
		trie.insert(key.as_ref(), &parachain_header.encode().encode()).unwrap();
		*trie.root()
	};

	let mut prev_hash = relay_chain_hash;
	let mut headers = vec![];
	for i in 0..unknown_headers {
		let header = RelayChainHeader {
			parent_hash: prev_hash,
			number: latest_relay_height + 1 + i,
			state_root,
			extrinsics_root: Default::default(),
			digest: Default::default(),
		};
		prev_hash = header.hash();
		headers.push(header);
	}
	let header = headers.last().unwrap().clone();
	let header_hash = header.hash();
	let precommit = Precommit { target_hash: header_hash, target_number: header.number };
	let message = finality_grandpa::Message::Precommit(precommit.clone());

	// Build signed precommits
	let mut signed_precommits = vec![];
	let mut authorities = vec![];
	for i in 1..=pre_commits {
		let public_key =
			sp_io::crypto::ed25519_generate(KEY_TYPE, Some(format!("//{}", i).as_bytes().to_vec()));
		authorities.push(AuthorityId::from(public_key.clone()));
		let encoded = sp_consensus_grandpa::localized_payload(round, set_id, &message);
		let signature = AuthoritySignature::from(
			sp_io::crypto::ed25519_sign(KEY_TYPE, &public_key, &encoded).unwrap(),
		);
		let signed_precommit = SignedPrecommit {
			precommit: precommit.clone(),
			signature,
			id: AuthorityId::from(public_key),
		};
		signed_precommits.push(signed_precommit);
	}

	let commit = Commit::<RelayChainHeader> {
		target_hash: header_hash,
		target_number: header.number,
		precommits: signed_precommits,
	};
	let justification =
		GrandpaJustification::<RelayChainHeader> { round, commit, votes_ancestries: vec![] };
	let finality_proof = FinalityProof {
		block: header_hash,
		justification: justification.encode(),
		unknown_headers: headers.clone(),
	};
	let mut parachain_headers = BTreeMap::default();

	let state_proof =
		StorageProof::new(para_db.drain().into_iter().map(|(_, (val, ..))| val.to_vec()))
			.into_nodes()
			.into_iter()
			.collect::<Vec<_>>();
	// For each relay chain header we construct a corresponding ParachainHeaderProofs
	for header in &headers {
		parachain_headers.insert(
			header.hash(),
			ParachainHeaderProofs {
				state_proof: state_proof.clone(),
				extrinsic: timestamp_extrinsic.clone(),
				extrinsic_proof: extrinsic_proof.clone(),
			},
		);
	}

	let grandpa_header = GrandpaHeader {
		finality_proof,
		parachain_headers,
		height: Height::new(para_id.into(), parachain_header.number.into()),
	};
	let client_message = AnyClientMessage::Grandpa(ClientMessage::Header(grandpa_header));

	let client_state = ClientState {
		relay_chain: Default::default(),
		latest_relay_height,
		latest_relay_hash: relay_chain_hash,
		frozen_height: None,
		latest_para_height,
		para_id,
		current_set_id: set_id,
		current_authorities: authorities.into_iter().map(|authority| (authority, 100)).collect(),
		_phantom: Default::default(),
	};

	let time = core::time::Duration::from_millis(GRANDPA_UPDATE_TIMESTAMP.saturating_mul(1000));
	let consensus_state = ConsensusState {
		timestamp: Timestamp::from_nanoseconds(time.as_nanos().saturated_into::<u64>())
			.unwrap()
			.into_tm_time()
			.unwrap(),
		root: H256::zero().as_bytes().to_vec().into(),
	};

	(client_state, consensus_state, client_message)
}
