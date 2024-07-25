use crate::tangle_testnet_runtime::api::{
	self as TangleApi,
	runtime_types::{
		bounded_collections::bounded_vec::BoundedVec, pallet_roles::profile::Record,
		tangle_primitives::roles,
	},
};
use tangle_subxt::{
	subxt::{OnlineClient, PolkadotConfig},
	tangle_testnet_runtime,
};

#[tokio::main]
async fn main() -> Result<(), String> {
	let subxt_client = OnlineClient::<PolkadotConfig>::new().await.unwrap();
	let alice = subxt_signer::sr25519::dev::alice();
	let bob = subxt_signer::sr25519::dev::bob();

	// Create a profile for the validators.
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

	Ok(())
}
