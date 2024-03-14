use sp_core::Pair;
use subxt::OnlineClient;
use subxt::{self, tx::Signer, utils::AccountId32, PolkadotConfig};
use tangle_runtime::api::runtime_types::pallet_roles::profile::Record;

use crate::tangle_runtime;
use crate::tangle_runtime::api::{
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
	println!("Alice role key: {:?}", alice_role_key.public());

	let bob_role_key = sp_core::ecdsa::Pair::from_string("//Bob", None).unwrap();
	println!("Bob role key: {:?}", bob_role_key.public());

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
	let create_profile_tx = tangle_runtime::api::tx().roles().create_profile(profile, None);

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
		expiry: 100u64,
		ttl: 100u64,
		job_type: jobs::JobType::DKGTSSPhaseOne(jobs::tss::DKGTSSPhaseOneJobType {
			participants: BoundedVec::<AccountId32>(vec![
				alice_account_id.clone(),
				bob_account_id.clone(),
			]),
			threshold: 1u8,
			permitted_caller: None,
			role_type: roles::tss::ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1,
			__ignore: Default::default(),
		}),
		fallback: jobs::FallbackOptions::Destroy,
	};

	let jobs_tx = tangle_runtime::api::tx().jobs().submit_job(dkg_phase_one);
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
		__ignore: Default::default(),
	});

	let job_result_tx = tangle_runtime::api::tx().jobs().submit_job_result(
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
}
