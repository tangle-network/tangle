use crate::tangle_testnet_runtime::api::runtime_types::{
	bounded_collections::bounded_vec::BoundedVec,
	tangle_primitives::{jobs, roles},
};
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, Bytes};
use parity_scale_codec::Encode;
use std::str::FromStr;
use std::sync::Arc;
use tangle_subxt::subxt::{tx::Signer, utils::AccountId32, utils::H160, PolkadotConfig};
use tangle_subxt::tangle_testnet_runtime;

#[tokio::main]
async fn main() -> Result<(), String> {
	let alice = subxt_signer::sr25519::dev::alice();
	let bob = subxt_signer::sr25519::dev::bob();
	let alice_account_id =
		<subxt_signer::sr25519::Keypair as Signer<PolkadotConfig>>::account_id(&alice);
	let bob_account_id =
		<subxt_signer::sr25519::Keypair as Signer<PolkadotConfig>>::account_id(&bob);

	// Phase 1 job details
	let dkg_phase_one = jobs::JobSubmission {
		expiry: 1000u64,
		ttl: 1000u64,
		job_type: jobs::JobType::DKGTSSPhaseOne(jobs::tss::DKGTSSPhaseOneJobType {
			participants: BoundedVec::<AccountId32>(vec![
				alice_account_id.clone(),
				bob_account_id.clone(),
			]),
			threshold: 1u8,
			permitted_caller: None,
			role_type: roles::tss::ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1,
			hd_wallet: false,
			__ignore: Default::default(),
		}),
		fallback: jobs::FallbackOptions::Destroy,
	};

	let _jobs_tx = tangle_testnet_runtime::api::tx().jobs().submit_job(dkg_phase_one.clone());

	abigen!(
		VotableSigningRules,
		"/Users/salman/Webb-tools/tangle/forge/artifacts/VotableSigningRules.json"
	);
	let provider = Provider::<Http>::try_from("http://127.0.0.1:9944").unwrap();
	let client = Arc::new(provider.clone());
	// Signing rules contract address which will be used for quering data.
	let contract_address =
		H160::from_str("0x9eec675dde082a7fed3ff3287a10e249e6d6034a").unwrap_or_default();
	let contract = VotableSigningRules::new(Address::from(contract_address), client);
	let phase_1_job_id = [0u8; 32];
	let phase_1_job_details: Bytes = dkg_phase_one.job_type.encode().into();

	let proposal_id = contract
		.calculate_phase_1_proposal_id(phase_1_job_id, phase_1_job_details.clone())
		.call()
		.await
		.unwrap();

	let total_votes = contract.get_proposal_yes_votes_total(proposal_id).call().await.unwrap();
	println!("Total votes : {:?}", total_votes);

	let proposal_status = contract.get_proposal_state(proposal_id).call().await.unwrap();
	println!("Proposal status : {:?}", proposal_status);

	Ok(())
}
