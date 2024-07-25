use crate::tangle_testnet_runtime::api::runtime_types::{
	bounded_collections::bounded_vec::BoundedVec,
	tangle_primitives::{jobs, roles},
};

use sp_core::Pair;

use tangle_subxt::{
	subxt::{OnlineClient, PolkadotConfig},
	tangle_testnet_runtime,
};

#[tokio::main]
async fn main() -> Result<(), String> {
	let subxt_client = OnlineClient::<PolkadotConfig>::new().await.unwrap();
	let alice = subxt_signer::sr25519::dev::alice();
	// Dks signing key
	let dkg_key = sp_core::ecdsa::Pair::from_seed(&[3u8; 32]);
	let dkg_pubkey = dkg_key.public();

	let proposal_data = b"phase2";
	let proposal_data_hash = sp_core::hashing::keccak_256(proposal_data);
	let dkg_signature = dkg_key.sign_prehashed(&proposal_data_hash);
	let dkg_phase_two_result = jobs::JobResult::DKGPhaseTwo(jobs::tss::DKGTSSSignatureResult {
		signature_scheme: jobs::tss::DigitalSignatureScheme::EcdsaSecp256k1,
		signature: BoundedVec(dkg_signature.0[..64].to_vec()),
		derivation_path: None,
		data: BoundedVec(proposal_data_hash.to_vec()),
		verifying_key: BoundedVec(dkg_pubkey.0.to_vec()),
		chain_code: None,
		__ignore: Default::default(),
	});

	// Phase 2 Job Id
	let phase2_job_id = 1u64;
	let job_result_tx = tangle_testnet_runtime::api::tx().jobs().submit_job_result(
		roles::RoleType::Tss(roles::tss::ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
		phase2_job_id,
		dkg_phase_two_result,
	);

	let _hash = subxt_client
		.tx()
		.sign_and_submit_then_watch_default(&job_result_tx, &alice)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.unwrap();

	Ok(())
}
