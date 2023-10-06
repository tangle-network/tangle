use anyhow::Result;
use webb::evm::{
	contract::protocol_solidity,
	ethers::{
		abi::{AbiDecode, AbiEncode},
		types::{Address, Bytes, H256},
	},
};

#[derive(Debug, Clone, serde::Serialize)]
struct RefreshProposal {
	voter_merkle_root: H256,
	average_session_length_in_millisecs: u64,
	voter_count: u32,
	nonce: u32,
	public_key: Bytes,
}

impl RefreshProposal {
	pub fn decode(bytes: &[u8]) -> Result<Self> {
		let mut voter_merkle_root_bytes = [0u8; 32];
		let mut session_length_bytes = [0u8; 8];
		let mut voter_count_bytes = [0u8; 4];
		let mut nonce_bytes = [0u8; 4];
		let mut pub_key_bytes = [0u8; 64];
		voter_merkle_root_bytes.copy_from_slice(&bytes[0..32]);
		session_length_bytes.copy_from_slice(&bytes[32..40]);
		voter_count_bytes.copy_from_slice(&bytes[40..44]);
		nonce_bytes.copy_from_slice(&bytes[44..48]);
		pub_key_bytes.copy_from_slice(&bytes[48..]);
		let voter_merkle_root = voter_merkle_root_bytes;
		let average_session_length_in_millisecs = u64::from_be_bytes(session_length_bytes);
		let voter_count = u32::from_be_bytes(voter_count_bytes);
		let nonce = u32::from_be_bytes(nonce_bytes);
		let pub_key = pub_key_bytes.to_vec();
		Ok(Self {
			voter_merkle_root: H256::from_slice(&voter_merkle_root),
			average_session_length_in_millisecs,
			voter_count,
			nonce,
			public_key: pub_key.into(),
		})
	}
}

#[derive(Debug, clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Debug, clap::Subcommand)]
pub enum Commands {
	/// Vote for a candidate
	Vote {
		/// The index of the leaf node in the Merkle tree
		#[arg(long)]
		leaf_index: u32,
		/// The path to the leaf node in the Merkle tree
		/// Can be specified multiple times
		#[arg(long = "node", action = clap::ArgAction::Append, required = true)]
		sibling_path_nodes: Vec<H256>,
		/// The address of the candidate
		#[arg(long)]
		proposed_governor: Address,
	},
	/// Decode DKG Refresh Proposal
	Decode {
		/// The DKG Refresh Proposal
		refresh_proposal: Bytes,
	},
	/// Convert a public key to an EVM address
	PublicKeyToAddress {
		/// The public key
		public_key: Bytes,
	},
	/// Cast votes to change the governor
	Cast {
		#[arg(long, action = clap::ArgAction::Append, required = true)]
		vote: Vec<Bytes>,
		#[arg(long, action = clap::ArgAction::Append, required = true)]
		signature: Vec<Bytes>,
	},
}

fn main() -> Result<()> {
	let args: Args = clap::Parser::parse();
	match args.command {
		Commands::Vote { leaf_index, sibling_path_nodes, proposed_governor } => {
			let vote = protocol_solidity::signature_bridge::Vote {
				leaf_index,
				sibling_path_nodes: sibling_path_nodes.into_iter().map(Into::into).collect(),
				proposed_governor,
			};

			println!("{}", serde_json::to_string_pretty(&vote)?);
			// encode the vote as a ABI encoded bytes.
			let encoded = vote.encode();
			// print the encoded bytes as a hex string.
			println!("Please Sign the following using your `wdkg` ECDSA key:");
			println!("Encoded Vote: {}", hex::encode(encoded));
		},
		Commands::Decode { refresh_proposal } => {
			let prop = RefreshProposal::decode(&refresh_proposal)?;
			println!("Decoded Refresh Proposal:");
			println!("{}", serde_json::to_string_pretty(&prop)?);
		},
		Commands::PublicKeyToAddress { public_key } => {
			// To convert a public key to an EVM address, we need to hash the public key using
			// Keccak-256 and then take the last 20 bytes of the hash.
			let hash = webb::evm::ethers::utils::keccak256(&public_key);
			let address = Address::from_slice(&hash[12..]);
			println!("Address: {:?}", address);
		},
		Commands::Cast { vote, signature: sigs } => {
			let votes = vote
				.into_iter()
				.map(|v| protocol_solidity::signature_bridge::Vote::decode(&v).map_err(Into::into))
				.collect::<Result<Vec<_>>>()?;
			let call =
				protocol_solidity::signature_bridge::VoteInFavorForceSetGovernorWithSigCall {
					votes,
					sigs,
				};
			println!("{}", serde_json::to_string_pretty(&call)?);
			println!("Please use `cast send` to send this tx to the signature bridge:");
			let call_bytes = call.encode_hex();
			println!("Call: {call_bytes}");
		},
	}
	Ok(())
}
