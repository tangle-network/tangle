use std::fs;
use std::sync::Arc;
use std::time::Duration;

use crate::tangle_testnet_runtime::api::runtime_types::pallet_roles::profile::Record;
use crate::tangle_testnet_runtime::api::{
	self as TangleApi,
	runtime_types::{
		bounded_collections::bounded_vec::BoundedVec,
		tangle_primitives::{jobs, roles},
	},
};
use ethers::contract::{abigen, FunctionCall};
use ethers::middleware::Middleware;
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;
use ethers::signers::Signer as EthSigner;
use ethers::types::{Address, Bytes};
use parity_scale_codec::Encode;
use serde_json::Value;
use sp_core::{Pair, H256, U256};
use std::str::FromStr;
use tangle_subxt::subxt::OnlineClient;
use tangle_subxt::subxt::{tx::Signer, utils::AccountId32, utils::H160, PolkadotConfig};
use tangle_subxt::tangle_testnet_runtime;
use tangle_subxt::tangle_testnet_runtime::api::runtime_types::primitive_types::U256 as WebbU256;
use tangle_subxt::tangle_testnet_runtime::api::runtime_types::tangle_testnet_runtime::RuntimeCall;
use tokio::time::sleep;

fn get_signing_rules_abi() -> (Value, Value) {
	let mut data: Value = serde_json::from_str(
		&fs::read_to_string("examples/contracts/artifacts/VotableSigningRules.json").unwrap(),
	)
	.unwrap();
	let abi = data["abi"].take();
	let bytecode = data["bytecode"]["object"].take();
	(abi, bytecode)
}

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

	// Step 2: Deploy the contract.

	let (_abi, bytecode) = get_signing_rules_abi();
	let stripped_bytecode = bytecode.as_str().unwrap().trim_start_matches("0x");
	let decoded = hex::decode(stripped_bytecode).unwrap();
	let create2_call = RuntimeCall::EVM(TangleApi::evm::Call::create2 {
		source: H160::from_str("0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap(),
		init: decoded,
		salt: H256::from([0u8; 32]),
		value: WebbU256([0u64; 4]),
		gas_limit: 10000000u64,
		max_fee_per_gas: WebbU256(U256::from_big_endian(1000000u64.to_be_bytes().as_slice()).0),
		max_priority_fee_per_gas: None,
		nonce: None,
		access_list: vec![],
	});
	let sudo_create2_call = TangleApi::tx().sudo().sudo(create2_call);
	let result = subxt_client
		.tx()
		.sign_and_submit_then_watch_default(&sudo_create2_call, &alice)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.unwrap();

	let deployed_contract = result.find_first::<TangleApi::evm::events::Created>().unwrap();

	let contract_address = match deployed_contract {
		Some(contract) => {
			println!("Contract {:?} deployed at : {:?}", contract.address, result.block_hash());
			contract.address
		},
		None => return Err("Contract failed to deploy".to_string()),
	};

	// Step 3: Submit a DKG phase one job request.
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

	// Step 4: Submit the DKG phase one result.
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

	// Step 5: Intitialize the signing rules contract.
	abigen!(VotableSigningRules, "examples/contracts/artifacts/VotableSigningRules.json");
	let provider = Provider::<Http>::try_from("http://127.0.0.1:9944").unwrap();
	let client = Arc::new(provider.clone());
	let contract = VotableSigningRules::new(Address::from(contract_address), client);
	let phase_1_job_id = 0u64;
	let threshold = 2;
	let use_democracy = false;
	let voters = vec![
		H160::from_str("0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap(),
		H160::from_str("0x5D4ff00Bf77F97E93131a448379f7808D7373026").unwrap(),
		H160::from_str("0xb65EA4E162188d199b14da8bc747F24042c36E2C").unwrap(),
	];
	let expiry = 5000;
	let ttl = 5000;
	let fn_call: FunctionCall<_, _, _> =
		contract.initialize(phase_1_job_id, threshold, use_democracy, voters, expiry, ttl);

	let init_tx_call = RuntimeCall::EVM(TangleApi::evm::Call::call {
		source: H160::from_str("0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap(),
		target: contract_address,
		input: fn_call.calldata().unwrap().to_vec(),
		value: WebbU256([0u64; 4]),
		gas_limit: 10000000u64,
		max_fee_per_gas: WebbU256(U256::from_big_endian(1000000u64.to_be_bytes().as_slice()).0),
		max_priority_fee_per_gas: None,
		nonce: None,
		access_list: vec![],
	});

	let sudo_init_tx_call = TangleApi::tx().sudo().sudo(init_tx_call);
	let result = subxt_client
		.tx()
		.sign_and_submit_then_watch_default(&sudo_init_tx_call, &alice)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.unwrap();

	println!("Contract initialized at : {:?}", result.block_hash());

	// Step 6: Vote on the proposal to submit the phase 2 job.
	let phase_2_job_details: Bytes = b"phase2".into();
	let vote_proposal_fn_call: FunctionCall<_, _, _> =
		contract.vote_proposal(phase_1_job_id, phase_2_job_details.clone());

	let relayer_wallet1 =
		LocalWallet::from_str("99b3c12287537e38c90a9219d4cb074a89a16e9cdb20bf85728ebd97c343e342")
			.unwrap()
			.with_chain_id(3799u32);
	let relayer1_signer_client = SignerMiddleware::new(provider.clone(), relayer_wallet1);
	let _result = relayer1_signer_client
		.send_transaction(vote_proposal_fn_call.clone().tx, None)
		.await
		.unwrap();

	sleep(Duration::from_secs(5)).await;
	println!("Relayer 1 voted on proposal");

	let relayer_wallet2 =
		LocalWallet::from_str("e0a3bc1ac01e8d0653637f5b1a3561bef5281fada1e9258277c67d2ac654c060")
			.unwrap()
			.with_chain_id(3799u32);
	let relayer2_signer_client = SignerMiddleware::new(provider.clone(), relayer_wallet2);
	let _result = relayer2_signer_client
		.send_transaction(vote_proposal_fn_call.clone().tx, None)
		.await
		.unwrap();

	sleep(Duration::from_secs(5)).await;
	println!("Relayer 2 voted on proposal");

	let relayer_wallet3 =
		LocalWallet::from_str("58ba5c1ceadfb2e7e36b267fe464f33e36371de03a5a5469a2e86a99e253a3ae")
			.unwrap()
			.with_chain_id(3799u32);
	let relayer3_signer_client = SignerMiddleware::new(provider.clone(), relayer_wallet3);
	let _result = relayer3_signer_client
		.send_transaction(vote_proposal_fn_call.clone().tx, None)
		.await
		.unwrap();

	sleep(Duration::from_secs(5)).await;
	println!("Relayer 3 voted on proposal");

	Ok(())
}
