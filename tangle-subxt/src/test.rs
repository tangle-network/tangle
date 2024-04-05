use crate::tangle_testnet_runtime::api::runtime_types::pallet_roles::profile::Record;
use sp_core::Pair;
use subxt::OnlineClient;
use subxt::{self, tx::Signer, utils::AccountId32, PolkadotConfig};

use crate::tangle_testnet_runtime;
use crate::tangle_testnet_runtime::api::{
	self,
	runtime_types::{
		bounded_collections::bounded_vec::BoundedVec,
		tangle_primitives::{jobs, roles},
	},
};

#[tokio::test]
#[ignore = "need to be run manually"]
async fn test_job_submission_event() {
	let subxt_client = OnlineClient::<PolkadotConfig>::new().await.unwrap();
	let alice = subxt_signer::sr25519::dev::alice();
	let bob = subxt_signer::sr25519::dev::bob();
	let alice_account_id =
		<subxt_signer::sr25519::Keypair as Signer<PolkadotConfig>>::account_id(&alice);
	let bob_account_id =
		<subxt_signer::sr25519::Keypair as Signer<PolkadotConfig>>::account_id(&bob);

	let alice_role_key = sp_core::ecdsa::Pair::from_string("//Alice", None).unwrap();
	let bob_role_key = sp_core::ecdsa::Pair::from_string("//Bob", None).unwrap();

	let profile = api::runtime_types::pallet_roles::profile::Profile::Shared(
		api::runtime_types::pallet_roles::profile::SharedRestakeProfile {
			records: BoundedVec(vec![Record {
				amount: None,
				role: roles::RoleType::Tss(
					roles::tss::ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1,
				),
			}]),
			amount: 100 * 1_000_000_000,
		},
	);
	let create_profile_tx = tangle_testnet_runtime::api::tx().roles().create_profile(profile, None);

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

	let jobs_tx = tangle_testnet_runtime::api::tx().jobs().submit_job(dkg_phase_one);
	let _hash = subxt_client
		.tx()
		.sign_and_submit_then_watch_default(&jobs_tx, &alice)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.unwrap();

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

	for i in 1..10 {
		// Submit Phase 2 Job Request.
		let phase_one_id = 0u64;
		let proposal_data = b"proposalBytes";
		let dkg_phase_two = jobs::JobSubmission {
			expiry: 1000u64,
			ttl: 1000u64,
			job_type: jobs::JobType::DKGTSSPhaseTwo(jobs::tss::DKGTSSPhaseTwoJobType {
				role_type: roles::tss::ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1,
				phase_one_id,
				submission: BoundedVec(proposal_data.to_vec()),
				derivation_path: None,
				__ignore: Default::default(),
			}),
			fallback: jobs::FallbackOptions::Destroy,
		};

		let jobs_tx = tangle_testnet_runtime::api::tx().jobs().submit_job(dkg_phase_two);
		let _hash = subxt_client
			.tx()
			.sign_and_submit_then_watch_default(&jobs_tx, &alice)
			.await
			.unwrap()
			.wait_for_finalized_success()
			.await
			.unwrap();

		// Submit Phase 2 Job Result.
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
		let new_job_id = i as u64;
		let job_result_tx = tangle_testnet_runtime::api::tx().jobs().submit_job_result(
			roles::RoleType::Tss(roles::tss::ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
			new_job_id,
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
	}
}
