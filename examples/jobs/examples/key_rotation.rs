use crate::tangle_testnet_runtime::api::{
	self as TangleApi,
	runtime_types::{
		bounded_collections::bounded_vec::BoundedVec,
		pallet_roles::profile::Record,
		tangle_primitives::{jobs, roles},
	},
};

use sp_core::{ByteArray, Pair};
use tangle_subxt::{
	subxt::{tx::Signer, utils::AccountId32, OnlineClient, PolkadotConfig},
	tangle_testnet_runtime,
};

#[tokio::main]
async fn main() -> Result<(), String> {
	let subxt_client = OnlineClient::<PolkadotConfig>::new().await.unwrap();
	let alice = subxt_signer::sr25519::dev::alice();
	let bob = subxt_signer::sr25519::dev::bob();
	let alice_account_id =
		<subxt_signer::sr25519::Keypair as Signer<PolkadotConfig>>::account_id(&alice);
	let bob_account_id =
		<subxt_signer::sr25519::Keypair as Signer<PolkadotConfig>>::account_id(&bob);
	let alice_role_key = sp_core::ecdsa::Pair::from_string("//Alice", None).unwrap();
	let bob_role_key = sp_core::ecdsa::Pair::from_string("//Bob", None).unwrap();

	// Step 1: Create a profile for the validators.

	let profile = TangleApi::runtime_types::pallet_roles::profile::Profile::Shared(
		TangleApi::runtime_types::pallet_roles::profile::SharedRestakeProfile {
			records: BoundedVec(vec![Record {
				amount: None,
				role: roles::RoleType::Tss(
					roles::tss::ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1,
				),
			}]),
			amount: 100 * 1_000_000_000,
		},
	);
	let create_profile_tx = TangleApi::tx().roles().create_profile(profile, None);

	let _hash = subxt_client
		.tx()
		.sign_and_submit_then_watch_default(&create_profile_tx, &alice)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.unwrap();

	let _hash = subxt_client
		.tx()
		.sign_and_submit_then_watch_default(&create_profile_tx, &bob)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.unwrap();

	// Step 2: Submit a DKG phase one job request.
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
	let jobs_tx = tangle_testnet_runtime::api::tx().jobs().submit_job(dkg_phase_one.clone());
	let _hash = subxt_client
		.tx()
		.sign_and_submit_then_watch_default(&jobs_tx, &alice)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.unwrap();

	// Step 3: Submit the DKG phase one result.
	let dkg_key = sp_core::ecdsa::Pair::from_seed(&[3u8; 32]);
	let dkg_pubkey = dkg_key.public();

	let dkg_pubkey_hash = sp_core::hashing::keccak_256(dkg_pubkey.as_ref());
	let alice_signature = alice_role_key.sign_prehashed(&dkg_pubkey_hash);
	let bob_signature = bob_role_key.sign_prehashed(&dkg_pubkey_hash);
	let dkg_phase_one_result = jobs::JobResult::DKGPhaseOne(jobs::tss::DKGTSSKeySubmissionResult {
		key: BoundedVec(dkg_pubkey.0.to_vec()),
		participants: BoundedVec(vec![
			BoundedVec(alice_account_id.0.to_vec()),
			BoundedVec(bob_account_id.0.to_vec()),
		]),
		signature_scheme: jobs::tss::DigitalSignatureScheme::EcdsaSecp256k1,
		signatures: BoundedVec(vec![
			BoundedVec(alice_signature.0.to_vec()),
			BoundedVec(bob_signature.0.to_vec()),
		]),
		threshold: 1,
		chain_code: None,
		__ignore: Default::default(),
	});
	let job_result_tx = tangle_testnet_runtime::api::tx().jobs().submit_job_result(
		roles::RoleType::Tss(roles::tss::ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
		0,
		dkg_phase_one_result,
	);
	let _hash = subxt_client
		.tx()
		.sign_and_submit_then_watch_default(&job_result_tx, &alice)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.unwrap();

	// In order to rotate we need to submita Phase 1 job request
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
	let jobs_tx = tangle_testnet_runtime::api::tx().jobs().submit_job(dkg_phase_one.clone());
	let _hash = subxt_client
		.tx()
		.sign_and_submit_then_watch_default(&jobs_tx, &alice)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.unwrap();

	let new_dkg_key = sp_core::ecdsa::Pair::from_seed(&[4u8; 32]);
	let new_dkg_pubkey = new_dkg_key.public();
	let new_dkg_pubkey_hash = sp_core::hashing::keccak_256(new_dkg_pubkey.as_ref());
	let alice_signature = alice_role_key.sign_prehashed(&new_dkg_pubkey_hash);
	let bob_signature = bob_role_key.sign_prehashed(&new_dkg_pubkey_hash);

	// Sign new dkg key by current dkg key.
	let msg_hash = sp_core::hashing::keccak_256(new_dkg_pubkey.to_raw_vec().as_slice());
	let dkg_signature = dkg_key.sign_prehashed(&msg_hash);

	let dkg_phase_one_result = jobs::JobResult::DKGPhaseOne(jobs::tss::DKGTSSKeySubmissionResult {
		key: BoundedVec(new_dkg_pubkey.0.to_vec()),
		participants: BoundedVec(vec![
			BoundedVec(alice_account_id.0.to_vec()),
			BoundedVec(bob_account_id.0.to_vec()),
		]),
		signature_scheme: jobs::tss::DigitalSignatureScheme::EcdsaSecp256k1,
		signatures: BoundedVec(vec![
			BoundedVec(alice_signature.0.to_vec()),
			BoundedVec(bob_signature.0.to_vec()),
		]),
		threshold: 1,
		chain_code: None,
		__ignore: Default::default(),
	});
	let job_result_tx = tangle_testnet_runtime::api::tx().jobs().submit_job_result(
		roles::RoleType::Tss(roles::tss::ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
		1,
		dkg_phase_one_result,
	);
	let _hash = subxt_client
		.tx()
		.sign_and_submit_then_watch_default(&job_result_tx, &alice)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.unwrap();

	// Step4 Submit Dkg phase 4 job request
	let dkg_phase_four_job = jobs::JobSubmission {
		expiry: 1000u64,
		ttl: 1000u64,
		job_type: jobs::JobType::DKGTSSPhaseFour(jobs::tss::DKGTSSPhaseFourJobType {
			phase_one_id: 0,
			new_phase_one_id: 1,
			role_type: roles::tss::ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1,
		}),
		fallback: jobs::FallbackOptions::Destroy,
	};

	let dkg_phase_four_job_tx =
		tangle_testnet_runtime::api::tx().jobs().submit_job(dkg_phase_four_job.clone());
	let _hash = subxt_client
		.tx()
		.sign_and_submit_then_watch_default(&dkg_phase_four_job_tx, &alice)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.unwrap();

	// Step5 Submit Dkg phase 4 job result
	let dkg_phase4_job_result = jobs::JobResult::DKGPhaseFour(jobs::tss::DKGTSSKeyRotationResult {
		phase_one_id: 0,
		new_phase_one_id: 1,
		new_key: BoundedVec(new_dkg_pubkey.to_raw_vec()),
		key: BoundedVec(dkg_pubkey.to_raw_vec()),
		signature: BoundedVec(dkg_signature.0[..64].to_vec()),
		signature_scheme: jobs::tss::DigitalSignatureScheme::EcdsaSecp256k1,
		derivation_path: None,
		chain_code: None,
		__ignore: Default::default(),
	});

	let dkg_phase4_job_result_tx = tangle_testnet_runtime::api::tx().jobs().submit_job_result(
		roles::RoleType::Tss(roles::tss::ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
		2,
		dkg_phase4_job_result,
	);
	let _hash = subxt_client
		.tx()
		.sign_and_submit_then_watch_default(&dkg_phase4_job_result_tx, &alice)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.unwrap();

	Ok(())
}
