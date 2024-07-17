use crate::{
	light_clients::{AnyClientMessage, AnyClientState, AnyConsensusState, HostFunctionsManager},
	routing::Context,
	Config, MODULE_ID,
};
use core::{str::FromStr, time::Duration};
use frame_system::pallet_prelude::BlockNumberFor;
use ibc::{
	applications::transfer::VERSION,
	core::{
		ics02_client::{msgs::update_client::MsgUpdateAnyClient, trust_threshold::TrustThreshold},
		ics03_connection::{
			connection::{ConnectionEnd, Counterparty, State as ConnState},
			msgs::{
				conn_open_ack::MsgConnectionOpenAck, conn_open_confirm::MsgConnectionOpenConfirm,
				conn_open_try::MsgConnectionOpenTry,
			},
			version::Version as ConnVersion,
		},
		ics04_channel::{
			channel::{
				ChannelEnd, Counterparty as ChannelCounterParty, Order as ChannelOrder,
				State as ChannelState,
			},
			context::{ChannelKeeper, ChannelReader},
			msgs::{
				acknowledgement::MsgAcknowledgement, chan_close_confirm::MsgChannelCloseConfirm,
				chan_close_init::MsgChannelCloseInit, chan_open_ack::MsgChannelOpenAck,
				chan_open_confirm::MsgChannelOpenConfirm, chan_open_try::MsgChannelOpenTry,
				recv_packet::MsgRecvPacket, timeout::MsgTimeout,
			},
			packet::Packet,
			Version as ChannelVersion,
		},
		ics23_commitment::{commitment::CommitmentPrefix, specs::ProofSpecs},
		ics24_host::{
			identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId},
			path::{
				AcksPath, ChannelEndsPath, ClientConsensusStatePath, ClientStatePath,
				CommitmentsPath, ConnectionsPath, SeqRecvsPath,
			},
		},
	},
	proofs::Proofs,
	signer::Signer,
	timestamp::Timestamp,
	Height,
};
use ibc_proto::{cosmos::ics23::v1::CommitmentProof, ibc::core::commitment::v1::MerkleProof};
use ics07_tendermint::{
	client_state::ClientState as TendermintClientState, consensus_state::ConsensusState,
};
use ics11_beefy::{
	client_state::ClientState as BeefyClientState,
	consensus_state::ConsensusState as BeefyConsensusState,
};
use scale_info::prelude::format;
use sp_consensus_grandpa::{AuthoritySignature, KEY_TYPE};
use sp_core::crypto::{AccountId32, ByteArray};
use sp_runtime::SaturatedConversion;
use sp_std::prelude::*;
use tendermint::{
	account,
	block::{parts::Header as PartSetHeader, signed_header::SignedHeader},
	chain,
	signature::Ed25519Signature,
	validator, vote,
	vote::ValidatorIndex,
	AppHash, Hash,
};
use tendermint_light_client_verifier::types::Validator;
use tendermint_proto::Protobuf;

pub const TENDERMINT_TIMESTAMP: u64 = 1650894363;

#[derive(parity_scale_codec::Encode)]
struct ConsensusProofwithHostConsensusStateProof {
	host_proof: Vec<u8>,
	connection_proof: Vec<u8>,
}

/// Create a mock avl implementation that can be used to mock tendermint's iavl tree.
fn create_avl() -> simple_iavl::avl::AvlTree<Vec<u8>, Vec<u8>> {
	let mut avl_tree = simple_iavl::avl::AvlTree::new();
	// Insert some dummy data in tree
	for i in 0..100u8 {
		let key = vec![i; 32];
		avl_tree.insert(key, vec![0u8; 64]);
	}
	avl_tree
}

/// Creates a tendermint header
fn create_tendermint_header() -> ics07_tendermint::client_message::Header {
	let (.., header) = generate_tendermint_header(2, 2);
	header
}

fn generate_validator(public_key: sp_core::ed25519::Public) -> validator::Info {
	let public_key = tendermint::PublicKey::from_raw_ed25519(public_key.as_slice()).unwrap();
	let info = validator::Info {
		address: {
			let digest = sp_io::hashing::sha2_256(public_key.to_bytes().as_slice())
				[..account::LENGTH]
				.to_vec();
			account::Id::try_from(digest).unwrap()
		},
		pub_key: public_key,
		power: vote::Power::try_from(1u32).unwrap(),
		name: None,
		proposer_priority: validator::ProposerPriority::from(1),
	};
	info
}

fn generate_block_header(
	validators: Vec<Validator>,
	chain_id: tendermint::chain::Id,
	last_block_id_hash: Option<tendermint::Hash>,
	height: u64,
) -> tendermint::block::Header {
	let proposer_index = 0;
	let proposer_address = validators[proposer_index].address;

	let valset = validator::Set::without_proposer(validators.clone());
	let next_valset = validator::Set::without_proposer(validators);
	let time = core::time::Duration::from_millis(TENDERMINT_TIMESTAMP.saturating_mul(1000));
	let time = Timestamp::from_nanoseconds(time.as_nanos() as u64)
		.unwrap()
		.into_tm_time()
		.unwrap();

	let last_block_id = last_block_id_hash
		.map(|hash| tendermint::block::Id { hash, part_set_header: Default::default() });

	let header = tendermint::block::Header {
		// block version in Tendermint-go is hardcoded with value 11
		version: tendermint::block::header::Version { block: 11, app: 0 },
		chain_id,
		height: tendermint::block::Height::try_from(height).unwrap(),
		time,
		last_block_id,
		last_commit_hash: None,
		data_hash: None,
		validators_hash: valset.hash_with::<HostFunctionsManager>(),
		next_validators_hash: next_valset.hash_with::<HostFunctionsManager>(),
		consensus_hash: valset.hash_with::<HostFunctionsManager>(), /* TODO: currently not clear
		                                                             * how to produce a valid
		                                                             * hash */
		app_hash: AppHash::from_hex_upper("").unwrap(),
		last_results_hash: None,
		evidence_hash: None,
		proposer_address,
	};
	header
}

pub fn get_vote_sign_bytes(chain_id: chain::Id, vote: &vote::Vote) -> Vec<u8> {
	let signed_vote = vote::SignedVote::from_vote(vote.clone(), chain_id).unwrap();

	signed_vote.sign_bytes()
}

fn generate_vote(
	validator: Validator,
	validator_index: u32,
	header: &tendermint::block::Header,
) -> vote::Vote {
	let block_id = tendermint::block::Id {
		hash: header.hash_with::<HostFunctionsManager>(),
		part_set_header: PartSetHeader::new(1, header.hash_with::<HostFunctionsManager>()).unwrap(),
	};

	let timestamp = header.time;

	let mut vote = vote::Vote {
		vote_type: vote::Type::Precommit,
		height: header.height,
		round: tendermint::block::Round::try_from(1).unwrap(),
		block_id: Some(block_id),
		timestamp: Some(timestamp),
		validator_address: validator.address,
		validator_index: ValidatorIndex::try_from(validator_index).unwrap(),
		signature: tendermint::Signature::new(vec![0_u8; Ed25519Signature::BYTE_SIZE]).unwrap(),
	};

	let sign_bytes = get_vote_sign_bytes(header.chain_id.clone(), &vote);
	let public_key =
		sp_core::ed25519::Public::from_slice(validator.pub_key.to_bytes().as_slice()).unwrap();
	let signature = AuthoritySignature::from(
		sp_io::crypto::ed25519_sign(KEY_TYPE, &public_key, &sign_bytes).unwrap(),
	);
	vote.signature = tendermint::Signature::new(signature.to_vec()).unwrap();

	vote
}

pub fn generate_tendermint_header(
	pre_commits: u32,
	height: u64,
) -> (
	TendermintClientState<HostFunctionsManager>,
	ConsensusState,
	ics07_tendermint::client_message::Header,
) {
	let mut validators = vec![];
	for i in 1..=pre_commits {
		let public_key =
			sp_io::crypto::ed25519_generate(KEY_TYPE, Some(format!("//{}", i).as_bytes().to_vec()));
		let validator = generate_validator(public_key.clone());
		validators.push(validator);
	}
	let next_valset = validator::Set::without_proposer(validators.clone());
	let header = generate_block_header(
		validators.clone(),
		chain::Id::from_str("test-chain").unwrap(),
		None,
		height,
	);

	let spec = simple_iavl::avl::get_proof_spec();
	// The tendermint light client requires two proof specs one for the iavl tree used to
	// in constructing the ibc commitment root and another for the tendermint state tree
	// For the benchmarks we use the same spec for both so we can use one single iavl tree
	// to generate both proofs.
	let proof_specs = ProofSpecs::from(vec![spec.clone(), spec]);

	let tendermint_client_state = TendermintClientState::new(
		ChainId::from_string("test-chain"),
		TrustThreshold::ONE_THIRD,
		Duration::new(65000, 0),
		Duration::new(128000, 0),
		Duration::new(3, 0),
		Height::new(0, 1),
		proof_specs,
		vec!["".to_string()],
	)
	.unwrap();

	let mut commit_sigs = vec![];
	for (i, validator) in validators.iter().enumerate() {
		let vote = generate_vote(validator.clone(), i as u32, &header);
		let commit_sig = tendermint::block::CommitSig::BlockIdFlagCommit {
			validator_address: vote.validator_address,
			timestamp: vote.timestamp.unwrap(),
			signature: vote.signature,
		};

		commit_sigs.push(commit_sig)
	}
	let time = core::time::Duration::from_millis(TENDERMINT_TIMESTAMP.saturating_mul(1000))
		- core::time::Duration::from_secs(10 * 60);
	let tm_consensus_state = ics07_tendermint::consensus_state::ConsensusState {
		timestamp: Timestamp::from_nanoseconds(time.as_nanos() as u64)
			.unwrap()
			.into_tm_time()
			.unwrap(),
		root: sp_core::H256::zero().as_bytes().to_vec().into(),
		next_validators_hash: next_valset.hash_with::<HostFunctionsManager>(),
	};

	let block_id = tendermint::block::Id {
		hash: header.hash_with::<HostFunctionsManager>(),
		part_set_header: PartSetHeader::new(1, header.hash_with::<HostFunctionsManager>()).unwrap(),
	};

	let valset = validator::Set::without_proposer(validators);

	let commit = tendermint::block::Commit {
		height: tendermint::block::Height::try_from(2u32).unwrap(),
		round: tendermint::block::Round::try_from(1u32).unwrap(),
		block_id,
		signatures: commit_sigs,
	};
	let tm_header = ics07_tendermint::client_message::Header {
		signed_header: SignedHeader::new(header, commit).unwrap(),
		validator_set: valset.clone(),
		trusted_height: Height::new(0, 1),
		trusted_validator_set: valset,
	};

	(tendermint_client_state, tm_consensus_state, tm_header)
}

pub(crate) fn create_mock_state() -> (TendermintClientState<HostFunctionsManager>, ConsensusState) {
	let (client_state, cs_state, ..) = generate_tendermint_header(2, 2);
	(client_state, cs_state)
}

pub(crate) fn create_mock_beefy_client_state(
) -> (BeefyClientState<HostFunctionsManager>, BeefyConsensusState) {
	let client_state = BeefyClientState {
		chain_id: Default::default(),
		relay_chain: Default::default(),
		mmr_root_hash: Default::default(),
		latest_beefy_height: 1,
		frozen_height: None,
		latest_para_height: 0,
		para_id: 2087,
		authority: Default::default(),
		next_authority_set: Default::default(),
		_phantom: Default::default(),
	};

	let timestamp = ibc::timestamp::Timestamp::from_nanoseconds(1).unwrap();
	let timestamp = timestamp.into_tm_time().unwrap();
	let cs_state = BeefyConsensusState { timestamp, root: vec![].into() };
	(client_state, cs_state)
}

pub(crate) fn create_mock_grandpa_client_state() -> (
	ics10_grandpa::client_state::ClientState<HostFunctionsManager>,
	ics10_grandpa::consensus_state::ConsensusState,
) {
	let client_state = ics10_grandpa::client_state::ClientState {
		relay_chain: Default::default(),
		latest_relay_hash: Default::default(),
		latest_relay_height: 1,
		frozen_height: None,
		latest_para_height: 0,
		para_id: 2087,
		current_set_id: 0,
		current_authorities: vec![],
		_phantom: Default::default(),
	};

	let timestamp = ibc::timestamp::Timestamp::from_nanoseconds(1).unwrap();
	let timestamp = timestamp.into_tm_time().unwrap();
	let cs_state =
		ics10_grandpa::consensus_state::ConsensusState { timestamp, root: vec![].into() };
	(client_state, cs_state)
}

pub(crate) fn create_client_update<T>() -> MsgUpdateAnyClient<Context<T>>
where
	T: Config + Send + Sync,
	BlockNumberFor<T>: From<u32>,
{
	MsgUpdateAnyClient {
		client_id: ClientId::new("07-tendermint", 0).unwrap(),
		client_message: AnyClientMessage::Tendermint(
			ics07_tendermint::client_message::ClientMessage::Header(create_tendermint_header()),
		),
		signer: Signer::from_str(MODULE_ID).unwrap(),
	}
}
// Proof generation process for all tendermint benchmarks
// The process is as follows, we insert the all the required ibc paths and values needed to generate
// the proof in the context of the benchmark in question, then we extract the root from the tree and
// also extract a proof for any key we need, After this we insert the extracted root inside the avl
// tree as the value for the commitment prefix. We then get a proof for the commitment prefix.
// We then extract the new root and use this as the commitment root
// This new root is then set as the ibc commitment root in the light client consensus state.

// Creates a MsgConnectionOpenTry from a tendermint chain submitted to a substrate chain
pub(crate) fn create_conn_open_try<T>() -> (ConsensusState, MsgConnectionOpenTry<Context<T>>)
where
	T: Config + Send + Sync,
	BlockNumberFor<T>: From<u32>,
{
	let client_id = ClientId::new("07-tendermint", 0).unwrap();
	let counterparty_client_id = ClientId::new("10-grandpa", 1).unwrap();
	let commitment_prefix: CommitmentPrefix = "ibc/".as_bytes().to_vec().try_into().unwrap();
	let chain_a_counterparty = Counterparty::new(
		counterparty_client_id.clone(),
		Some(ConnectionId::new(1)),
		commitment_prefix.clone(),
	);
	let delay_period = core::time::Duration::from_secs(1000);
	let chain_b_connection_counterparty =
		Counterparty::new(client_id.clone(), None, commitment_prefix);
	let mut avl_tree = create_avl();
	let connection_end = ConnectionEnd::new(
		ConnState::Init,
		counterparty_client_id.clone(),
		chain_b_connection_counterparty,
		vec![ConnVersion::default()],
		delay_period,
	);

	let (client_state, cs_state) = create_mock_grandpa_client_state();
	let id: u32 = T::ChainId::get().into();
	let consensus_path = format!(
		"{}",
		ClientConsensusStatePath {
			client_id: counterparty_client_id.clone(),
			epoch: id as u64,
			height: 1
		}
	)
	.as_bytes()
	.to_vec();

	let client_path = format!("{}", ClientStatePath(counterparty_client_id)).as_bytes().to_vec();
	let path = format!("{}", ConnectionsPath(ConnectionId::new(1))).as_bytes().to_vec();
	avl_tree.insert(path.clone(), connection_end.encode_vec().unwrap());
	avl_tree
		.insert(consensus_path.clone(), AnyConsensusState::Grandpa(cs_state).encode_vec().unwrap());
	avl_tree.insert(
		client_path.clone(),
		AnyClientState::Grandpa(client_state.clone()).encode_vec().unwrap(),
	);
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof = avl_tree.get_proof(&*path).unwrap();
	let consensus_proof = avl_tree.get_proof(&*consensus_path).unwrap();
	let client_proof = avl_tree.get_proof(&*client_path).unwrap();
	avl_tree.insert("ibc/".as_bytes().to_vec(), root);
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof_0 = avl_tree.get_proof("ibc/".as_bytes()).unwrap();
	let mut buf = Vec::new();
	prost::Message::encode(&proof, &mut buf).unwrap();
	let proof: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	buf.clear();
	prost::Message::encode(&proof_0, &mut buf).unwrap();
	let proof_0: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	buf.clear();
	prost::Message::encode(&consensus_proof, &mut buf).unwrap();
	let consensus_proof: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	buf.clear();
	prost::Message::encode(&client_proof, &mut buf).unwrap();
	let client_proof: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	let merkle_proof = MerkleProof { proofs: vec![proof, proof_0.clone()] };
	buf.clear();
	prost::Message::encode(&merkle_proof, &mut buf).unwrap();
	let consensus_proof = MerkleProof { proofs: vec![consensus_proof, proof_0.clone()] };
	let client_proof = MerkleProof { proofs: vec![client_proof, proof_0] };
	let mut consensus_buf = Vec::new();
	let mut client_buf = Vec::new();
	prost::Message::encode(&consensus_proof, &mut consensus_buf).unwrap();
	prost::Message::encode(&client_proof, &mut client_buf).unwrap();
	let header = create_tendermint_header();
	let cs_state = ConsensusState {
		timestamp: header.signed_header.header.time,
		root: root.into(),
		next_validators_hash: header.signed_header.header.next_validators_hash,
	};
	let para_id: u32 = T::ChainId::get().into();
	(
		cs_state,
		MsgConnectionOpenTry {
			client_id,
			client_state: Some(AnyClientState::Grandpa(client_state)),
			counterparty: chain_a_counterparty,
			counterparty_versions: vec![ConnVersion::default()],
			proofs: Proofs::new(
				buf.try_into().unwrap(),
				Some(client_buf.try_into().unwrap()),
				Some(
					ibc::proofs::ConsensusProof::new(
						consensus_buf.try_into().unwrap(),
						Height::new(para_id.into(), 1),
					)
					.unwrap(),
				),
				None,
				Height::new(0, 2),
			)
			.unwrap(),
			delay_period,
			host_consensus_state_proof: vec![],
			signer: Signer::from_str(MODULE_ID).unwrap(),
		},
	)
}

pub(crate) fn create_conn_open_ack<T>() -> (ConsensusState, MsgConnectionOpenAck<Context<T>>)
where
	T: Config + Send + Sync,
	BlockNumberFor<T>: From<u32>,
{
	let client_id = ClientId::new("07-tendermint", 0).unwrap();
	let counterparty_client_id = ClientId::new("10-grandpa", 1).unwrap();
	let commitment_prefix: CommitmentPrefix = "ibc/".as_bytes().to_vec().try_into().unwrap();
	let delay_period = core::time::Duration::from_secs(1000);
	let chain_b_connection_counterparty =
		Counterparty::new(client_id, Some(ConnectionId::new(0)), commitment_prefix);
	let mut avl_tree = create_avl();
	let connection_end = ConnectionEnd::new(
		ConnState::TryOpen,
		counterparty_client_id.clone(),
		chain_b_connection_counterparty,
		vec![ConnVersion::default()],
		delay_period,
	);

	let (client_state, cs_state) = create_mock_grandpa_client_state();
	let para_id: u32 = T::ChainId::get().into();
	let consensus_path = format!(
		"{}",
		ClientConsensusStatePath {
			client_id: counterparty_client_id.clone(),
			epoch: para_id.into(),
			height: 1
		}
	)
	.as_bytes()
	.to_vec();

	let client_path = format!("{}", ClientStatePath(counterparty_client_id)).as_bytes().to_vec();
	let path = format!("{}", ConnectionsPath(ConnectionId::new(1))).as_bytes().to_vec();
	avl_tree.insert(path.clone(), connection_end.encode_vec().unwrap());
	avl_tree
		.insert(consensus_path.clone(), AnyConsensusState::Grandpa(cs_state).encode_vec().unwrap());
	avl_tree.insert(
		client_path.clone(),
		AnyClientState::Grandpa(client_state.clone()).encode_vec().unwrap(),
	);
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof = avl_tree.get_proof(&*path).unwrap();
	let consensus_proof = avl_tree.get_proof(&*consensus_path).unwrap();
	let client_proof = avl_tree.get_proof(&*client_path).unwrap();
	avl_tree.insert("ibc/".as_bytes().to_vec(), root);
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof_0 = avl_tree.get_proof("ibc/".as_bytes()).unwrap();
	let mut buf = Vec::new();
	prost::Message::encode(&proof, &mut buf).unwrap();
	let proof: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	buf.clear();
	prost::Message::encode(&proof_0, &mut buf).unwrap();
	let proof_0: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	buf.clear();
	prost::Message::encode(&consensus_proof, &mut buf).unwrap();
	let consensus_proof: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	buf.clear();
	prost::Message::encode(&client_proof, &mut buf).unwrap();
	let client_proof: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	let merkle_proof = MerkleProof { proofs: vec![proof, proof_0.clone()] };
	buf.clear();
	prost::Message::encode(&merkle_proof, &mut buf).unwrap();
	let consensus_proof = MerkleProof { proofs: vec![consensus_proof, proof_0.clone()] };
	let client_proof = MerkleProof { proofs: vec![client_proof, proof_0] };
	let mut consensus_buf = Vec::new();
	let mut client_buf = Vec::new();
	prost::Message::encode(&consensus_proof, &mut consensus_buf).unwrap();
	prost::Message::encode(&client_proof, &mut client_buf).unwrap();
	let header = create_tendermint_header();
	let cs_state = ConsensusState {
		timestamp: header.signed_header.header.time,
		root: root.into(),
		next_validators_hash: header.signed_header.header.next_validators_hash,
	};
	let para_id: u32 = T::ChainId::get().into();
	(
		cs_state,
		MsgConnectionOpenAck {
			connection_id: ConnectionId::new(0),
			counterparty_connection_id: ConnectionId::new(1),
			client_state: Some(AnyClientState::Grandpa(client_state)),
			proofs: Proofs::new(
				buf.try_into().unwrap(),
				Some(client_buf.try_into().unwrap()),
				Some(
					ibc::proofs::ConsensusProof::new(
						consensus_buf.try_into().unwrap(),
						Height::new(u64::from(para_id), 1),
					)
					.unwrap(),
				),
				None,
				Height::new(0, 2),
			)
			.unwrap(),
			version: ConnVersion::default(),
			signer: Signer::from_str(MODULE_ID).unwrap(),
			host_consensus_state_proof: Default::default(),
		},
	)
}

pub(crate) fn create_conn_open_confirm<T: Config>() -> (ConsensusState, MsgConnectionOpenConfirm) {
	let client_id = ClientId::new("07-tendermint", 0).unwrap();
	let counterparty_client_id = ClientId::new("10-grandpa", 1).unwrap();
	let commitment_prefix: CommitmentPrefix = "ibc/".as_bytes().to_vec().try_into().unwrap();
	let delay_period = core::time::Duration::from_secs(1000);
	let chain_b_connection_counterparty =
		Counterparty::new(client_id, Some(ConnectionId::new(0)), commitment_prefix);
	let mut avl_tree = create_avl();
	let connection_end = ConnectionEnd::new(
		ConnState::Open,
		counterparty_client_id.clone(),
		chain_b_connection_counterparty,
		vec![ConnVersion::default()],
		delay_period,
	);
	let (.., cs_state) = create_mock_beefy_client_state();
	let consensus_path = format!(
		"{}",
		ClientConsensusStatePath {
			client_id: counterparty_client_id,
			epoch: parachain_info::Pallet::<T>::parachain_id().saturated_into::<u32>() as u64,
			height: 1
		}
	)
	.as_bytes()
	.to_vec();

	let path = format!("{}", ConnectionsPath(ConnectionId::new(1))).as_bytes().to_vec();
	avl_tree.insert(path.clone(), connection_end.encode_vec().unwrap());
	avl_tree
		.insert(consensus_path.clone(), AnyConsensusState::Beefy(cs_state).encode_vec().unwrap());
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof = avl_tree.get_proof(&*path).unwrap();
	let consensus_proof = avl_tree.get_proof(&*consensus_path).unwrap();
	avl_tree.insert("ibc/".as_bytes().to_vec(), root);
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof_0 = avl_tree.get_proof("ibc/".as_bytes()).unwrap();
	let mut buf = Vec::new();
	prost::Message::encode(&proof, &mut buf).unwrap();
	let proof: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	buf.clear();
	prost::Message::encode(&proof_0, &mut buf).unwrap();
	let proof_0: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	buf.clear();
	prost::Message::encode(&consensus_proof, &mut buf).unwrap();
	let consensus_proof: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	let merkle_proof = MerkleProof { proofs: vec![proof, proof_0.clone()] };
	buf.clear();
	prost::Message::encode(&merkle_proof, &mut buf).unwrap();
	let consensus_proof = MerkleProof { proofs: vec![consensus_proof, proof_0] };
	let mut consensus_buf = Vec::new();
	prost::Message::encode(&consensus_proof, &mut consensus_buf).unwrap();
	let header = create_tendermint_header();
	let cs_state = ConsensusState {
		timestamp: header.signed_header.header.time,
		root: root.into(),
		next_validators_hash: header.signed_header.header.next_validators_hash,
	};
	(
		cs_state,
		MsgConnectionOpenConfirm {
			connection_id: ConnectionId::new(0),
			proofs: Proofs::new(
				buf.try_into().unwrap(),
				None,
				Some(
					ibc::proofs::ConsensusProof::new(
						consensus_buf.try_into().unwrap(),
						Height::new(
							u32::from(parachain_info::Pallet::<T>::parachain_id()).into(),
							1,
						),
					)
					.unwrap(),
				),
				None,
				Height::new(0, 2),
			)
			.unwrap(),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		},
	)
}

pub(crate) fn create_chan_open_try() -> (ConsensusState, MsgChannelOpenTry) {
	let port_id = PortId::transfer();
	let counterparty = ChannelCounterParty::new(port_id.clone(), None);
	let channel_end = ChannelEnd::new(
		ChannelState::Init,
		ChannelOrder::Unordered,
		counterparty.clone(),
		vec![ConnectionId::new(1)],
		ChannelVersion::new(VERSION.to_string()),
	);
	let mut avl_tree = create_avl();
	let path = format!("{}", ChannelEndsPath(port_id.clone(), ChannelId::new(0)))
		.as_bytes()
		.to_vec();
	avl_tree.insert(path.clone(), channel_end.encode_vec().unwrap());
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof = avl_tree.get_proof(&*path).unwrap();
	avl_tree.insert("ibc/".as_bytes().to_vec(), root);
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof_0 = avl_tree.get_proof("ibc/".as_bytes()).unwrap();
	let mut buf = Vec::new();
	prost::Message::encode(&proof, &mut buf).unwrap();
	let proof: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	buf.clear();
	prost::Message::encode(&proof_0, &mut buf).unwrap();
	let proof_0: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	let merkle_proof = MerkleProof { proofs: vec![proof, proof_0] };
	buf.clear();
	prost::Message::encode(&merkle_proof, &mut buf).unwrap();
	let header = create_tendermint_header();
	let cs_state = ConsensusState {
		timestamp: header.signed_header.header.time,
		root: root.into(),
		next_validators_hash: header.signed_header.header.next_validators_hash,
	};
	let mut channel_end = ChannelEnd::new(
		ChannelState::Init,
		ChannelOrder::Unordered,
		counterparty,
		vec![ConnectionId::new(0)],
		ChannelVersion::new(VERSION.to_string()),
	);
	channel_end.set_counterparty_channel_id(ChannelId::new(0));
	(
		cs_state,
		MsgChannelOpenTry {
			port_id,
			channel: channel_end,
			counterparty_version: ChannelVersion::new(VERSION.to_string()),
			proofs: Proofs::new(buf.try_into().unwrap(), None, None, None, Height::new(0, 2))
				.unwrap(),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		},
	)
}

pub(crate) fn create_chan_open_ack() -> (ConsensusState, MsgChannelOpenAck) {
	let port_id = PortId::transfer();
	let counterparty = ChannelCounterParty::new(port_id.clone(), Some(ChannelId::new(0)));
	let channel_end = ChannelEnd::new(
		ChannelState::TryOpen,
		ChannelOrder::Unordered,
		counterparty,
		vec![ConnectionId::new(1)],
		ChannelVersion::new(VERSION.to_string()),
	);
	let mut avl_tree = create_avl();
	let path = format!("{}", ChannelEndsPath(port_id.clone(), ChannelId::new(0)))
		.as_bytes()
		.to_vec();
	avl_tree.insert(path.clone(), channel_end.encode_vec().unwrap());
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof = avl_tree.get_proof(&*path).unwrap();
	avl_tree.insert("ibc/".as_bytes().to_vec(), root);
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof_0 = avl_tree.get_proof("ibc/".as_bytes()).unwrap();
	let mut buf = Vec::new();
	prost::Message::encode(&proof, &mut buf).unwrap();
	let proof: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	buf.clear();
	prost::Message::encode(&proof_0, &mut buf).unwrap();
	let proof_0: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	let merkle_proof = MerkleProof { proofs: vec![proof, proof_0] };
	buf.clear();
	prost::Message::encode(&merkle_proof, &mut buf).unwrap();
	let header = create_tendermint_header();
	let cs_state = ConsensusState {
		timestamp: header.signed_header.header.time,
		root: root.into(),
		next_validators_hash: header.signed_header.header.next_validators_hash,
	};

	(
		cs_state,
		MsgChannelOpenAck {
			port_id,
			channel_id: ChannelId::new(0),
			counterparty_channel_id: ChannelId::new(0),
			counterparty_version: ChannelVersion::new(VERSION.to_string()),
			proofs: Proofs::new(buf.try_into().unwrap(), None, None, None, Height::new(0, 2))
				.unwrap(),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		},
	)
}

pub(crate) fn create_chan_open_confirm() -> (ConsensusState, MsgChannelOpenConfirm) {
	let port_id = PortId::transfer();
	let counterparty = ChannelCounterParty::new(port_id.clone(), Some(ChannelId::new(0)));
	let channel_end = ChannelEnd::new(
		ChannelState::Open,
		ChannelOrder::Unordered,
		counterparty,
		vec![ConnectionId::new(1)],
		ChannelVersion::new(VERSION.to_string()),
	);
	let mut avl_tree = create_avl();
	let path = format!("{}", ChannelEndsPath(port_id.clone(), ChannelId::new(0)))
		.as_bytes()
		.to_vec();
	avl_tree.insert(path.clone(), channel_end.encode_vec().unwrap());
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof = avl_tree.get_proof(&*path).unwrap();
	avl_tree.insert("ibc/".as_bytes().to_vec(), root);
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof_0 = avl_tree.get_proof("ibc/".as_bytes()).unwrap();
	let mut buf = Vec::new();
	prost::Message::encode(&proof, &mut buf).unwrap();
	let proof: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	buf.clear();
	prost::Message::encode(&proof_0, &mut buf).unwrap();
	let proof_0: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	let merkle_proof = MerkleProof { proofs: vec![proof, proof_0] };
	buf.clear();
	prost::Message::encode(&merkle_proof, &mut buf).unwrap();
	let header = create_tendermint_header();
	let cs_state = ConsensusState {
		timestamp: header.signed_header.header.time,
		root: root.into(),
		next_validators_hash: header.signed_header.header.next_validators_hash,
	};

	(
		cs_state,
		MsgChannelOpenConfirm {
			port_id,
			channel_id: ChannelId::new(0),
			proofs: Proofs::new(buf.try_into().unwrap(), None, None, None, Height::new(0, 2))
				.unwrap(),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		},
	)
}

pub(crate) fn create_chan_close_init() -> MsgChannelCloseInit {
	let port_id = PortId::transfer();
	MsgChannelCloseInit {
		port_id,
		channel_id: ChannelId::new(0),
		signer: Signer::from_str(MODULE_ID).unwrap(),
	}
}

pub(crate) fn create_chan_close_confirm() -> (ConsensusState, MsgChannelCloseConfirm) {
	let port_id = PortId::transfer();
	let counterparty = ChannelCounterParty::new(port_id.clone(), Some(ChannelId::new(0)));
	let channel_end = ChannelEnd::new(
		ChannelState::Closed,
		ChannelOrder::Unordered,
		counterparty,
		vec![ConnectionId::new(1)],
		ChannelVersion::new(VERSION.to_string()),
	);
	let mut avl_tree = create_avl();
	let path = format!("{}", ChannelEndsPath(port_id.clone(), ChannelId::new(0)))
		.as_bytes()
		.to_vec();
	avl_tree.insert(path.clone(), channel_end.encode_vec().unwrap());
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof = avl_tree.get_proof(&*path).unwrap();
	avl_tree.insert("ibc/".as_bytes().to_vec(), root);
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof_0 = avl_tree.get_proof("ibc/".as_bytes()).unwrap();
	let mut buf = Vec::new();
	prost::Message::encode(&proof, &mut buf).unwrap();
	let proof: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	buf.clear();
	prost::Message::encode(&proof_0, &mut buf).unwrap();
	let proof_0: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	let merkle_proof = MerkleProof { proofs: vec![proof, proof_0] };
	buf.clear();
	prost::Message::encode(&merkle_proof, &mut buf).unwrap();
	let header = create_tendermint_header();
	let cs_state = ConsensusState {
		timestamp: header.signed_header.header.time,
		root: root.into(),
		next_validators_hash: header.signed_header.header.next_validators_hash,
	};

	(
		cs_state,
		MsgChannelCloseConfirm {
			port_id,
			channel_id: Default::default(),
			proofs: Proofs::new(buf.try_into().unwrap(), None, None, None, Height::new(0, 2))
				.unwrap(),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		},
	)
}

pub(crate) fn create_recv_packet<T: Config + Send + Sync>(
	data: Vec<u8>,
) -> (ConsensusState, MsgRecvPacket)
where
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	let port_id = PortId::transfer();
	let packet = Packet {
		sequence: 1u64.into(),
		source_port: port_id.clone(),
		source_channel: ChannelId::new(0),
		destination_port: port_id.clone(),
		destination_channel: ChannelId::new(0),
		data,
		timeout_height: Height::new(2087, 5),
		timeout_timestamp: Timestamp::from_nanoseconds(1690894363u64.saturating_mul(1000000000))
			.unwrap(),
	};
	let ctx = Context::<T>::new();
	let commitment =
		ctx.packet_commitment(packet.data.clone(), packet.timeout_height, packet.timeout_timestamp);
	let mut avl_tree = create_avl();
	let path = format!(
		"{}",
		CommitmentsPath { port_id, channel_id: ChannelId::new(0), sequence: 1.into() }
	)
	.as_bytes()
	.to_vec();
	avl_tree.insert(path.clone(), commitment.into_vec());
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof = avl_tree.get_proof(&*path).unwrap();
	avl_tree.insert("ibc/".as_bytes().to_vec(), root);
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof_0 = avl_tree.get_proof("ibc/".as_bytes()).unwrap();
	let mut buf = Vec::new();
	prost::Message::encode(&proof, &mut buf).unwrap();
	let proof: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	buf.clear();
	prost::Message::encode(&proof_0, &mut buf).unwrap();
	let proof_0: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	let merkle_proof = MerkleProof { proofs: vec![proof, proof_0] };
	buf.clear();
	prost::Message::encode(&merkle_proof, &mut buf).unwrap();
	let header = create_tendermint_header();
	let cs_state = ConsensusState {
		timestamp: header.signed_header.header.time,
		root: root.into(),
		next_validators_hash: header.signed_header.header.next_validators_hash,
	};

	(
		cs_state,
		MsgRecvPacket {
			packet,
			proofs: Proofs::new(buf.try_into().unwrap(), None, None, None, Height::new(0, 2))
				.unwrap(),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		},
	)
}

pub(crate) fn create_ack_packet<T: Config + Send + Sync>(
	data: Vec<u8>,
	ack: Vec<u8>,
) -> (ConsensusState, MsgAcknowledgement)
where
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	let port_id = PortId::transfer();
	let packet = Packet {
		sequence: 1u64.into(),
		source_port: port_id.clone(),
		source_channel: ChannelId::new(0),
		destination_port: port_id.clone(),
		destination_channel: ChannelId::new(0),
		data: data.clone(),
		timeout_height: Height::new(2087, 5),
		timeout_timestamp: Timestamp::from_nanoseconds(1690894363u64.saturating_mul(1000000000))
			.unwrap(),
	};
	let mut ctx = Context::<T>::new();
	let commitment = ctx.packet_commitment(data, packet.timeout_height, packet.timeout_timestamp);
	ctx.store_packet_commitment((port_id.clone(), ChannelId::new(0), 1.into()), commitment)
		.unwrap();
	let ack_commitment = ctx.ack_commitment(ack.clone().into());

	let mut avl_tree = create_avl();
	let path =
		format!("{}", AcksPath { port_id, channel_id: ChannelId::new(0), sequence: 1.into() })
			.as_bytes()
			.to_vec();
	avl_tree.insert(path.clone(), ack_commitment.into_vec());
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof = avl_tree.get_proof(&*path).unwrap();
	avl_tree.insert("ibc/".as_bytes().to_vec(), root);
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof_0 = avl_tree.get_proof("ibc/".as_bytes()).unwrap();
	let mut buf = Vec::new();
	prost::Message::encode(&proof, &mut buf).unwrap();
	let proof: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	buf.clear();
	prost::Message::encode(&proof_0, &mut buf).unwrap();
	let proof_0: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	let merkle_proof = MerkleProof { proofs: vec![proof, proof_0] };
	buf.clear();
	prost::Message::encode(&merkle_proof, &mut buf).unwrap();
	let header = create_tendermint_header();
	let cs_state = ConsensusState {
		timestamp: header.signed_header.header.time,
		root: root.into(),
		next_validators_hash: header.signed_header.header.next_validators_hash,
	};

	(
		cs_state,
		MsgAcknowledgement {
			packet,
			acknowledgement: ack.into(),
			proofs: Proofs::new(buf.try_into().unwrap(), None, None, None, Height::new(0, 2))
				.unwrap(),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		},
	)
}

pub(crate) fn create_timeout_packet<T: Config + Send + Sync>(
	data: Vec<u8>,
) -> (ConsensusState, MsgTimeout)
where
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	let port_id = PortId::transfer();
	let packet = Packet {
		sequence: 1u64.into(),
		source_port: port_id.clone(),
		source_channel: ChannelId::new(0),
		destination_port: port_id.clone(),
		destination_channel: ChannelId::new(0),
		data: data.clone(),
		timeout_height: Height::new(0, 1),
		timeout_timestamp: Timestamp::from_nanoseconds(1620894363u64.saturating_mul(1000000000))
			.unwrap(),
	};
	let mut ctx = Context::<T>::new();
	let commitment = ctx.packet_commitment(data, packet.timeout_height, packet.timeout_timestamp);
	ctx.store_packet_commitment((port_id.clone(), ChannelId::new(0), 1.into()), commitment)
		.unwrap();

	let mut avl_tree = create_avl();
	let path = format!("{}", SeqRecvsPath(port_id, ChannelId::new(0))).as_bytes().to_vec();
	let mut seq_bytes = Vec::new();
	prost::Message::encode(&1u64, &mut seq_bytes).unwrap();
	avl_tree.insert(path.clone(), seq_bytes);
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof = avl_tree.get_proof(&*path).unwrap();
	avl_tree.insert("ibc/".as_bytes().to_vec(), root);
	let root = match *avl_tree.root_hash().unwrap() {
		Hash::Sha256(root) => root.to_vec(),
		Hash::None => panic!("Failed to generate root hash"),
	};
	let proof_0 = avl_tree.get_proof("ibc/".as_bytes()).unwrap();
	let mut buf = Vec::new();
	prost::Message::encode(&proof, &mut buf).unwrap();
	let proof: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	buf.clear();
	prost::Message::encode(&proof_0, &mut buf).unwrap();
	let proof_0: CommitmentProof = prost::Message::decode(buf.as_ref()).unwrap();
	let merkle_proof = MerkleProof { proofs: vec![proof, proof_0] };
	buf.clear();
	prost::Message::encode(&merkle_proof, &mut buf).unwrap();
	let header = create_tendermint_header();
	let cs_state = ConsensusState {
		timestamp: header.signed_header.header.time,
		root: root.into(),
		next_validators_hash: header.signed_header.header.next_validators_hash,
	};

	(
		cs_state,
		MsgTimeout {
			packet,
			next_sequence_recv: Default::default(),
			proofs: Proofs::new(buf.try_into().unwrap(), None, None, None, Height::new(0, 2))
				.unwrap(),
			signer: Signer::from_str(MODULE_ID).unwrap(),
		},
	)
}
