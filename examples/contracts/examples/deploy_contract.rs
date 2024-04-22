use serde_json::Value;
use sp_core::{H256, U256};
use tangle_subxt::subxt::OnlineClient;
use tangle_subxt::subxt::{utils::H160, PolkadotConfig};
use tangle_subxt::tangle_testnet_runtime;
use tangle_subxt::tangle_testnet_runtime::api::runtime_types::primitive_types::U256 as WebbU256;

use crate::tangle_testnet_runtime::api as TangleApi;
use std::fs;
use std::str::FromStr;
use tangle_subxt::tangle_testnet_runtime::api::runtime_types::tangle_testnet_runtime::RuntimeCall;

pub fn get_git_root_path() -> std::path::PathBuf {
	let git_root = std::process::Command::new("git")
		.args(["rev-parse", "--show-toplevel"])
		.output()
		.expect("Failed to get git root")
		.stdout;
	let git_root = std::str::from_utf8(&git_root)
		.expect("Failed to parse git root")
		.trim()
		.to_string();
	std::path::PathBuf::from(&git_root)
}

fn get_signing_rules_abi() -> (Value, Value) {
	let abi_path = get_git_root_path().join("forge/artifacts/VotableSigningRules.json");
	let mut data: Value =
		serde_json::from_str(&fs::read_to_string(abi_path.as_path()).unwrap()).unwrap();
	let abi = data["abi"].take();
	let bytecode = data["bytecode"]["object"].take();
	(abi, bytecode)
}

#[tokio::main]
async fn main() -> Result<(), String> {
	let subxt_client = OnlineClient::<PolkadotConfig>::new().await.unwrap();
	let alice = subxt_signer::sr25519::dev::alice();

	// Deploy simple contract.
	let (_abi, bytecode) = get_signing_rules_abi();
	let stripped_bytecode = bytecode.as_str().unwrap().trim_start_matches("0x");
	let decoded = hex::decode(stripped_bytecode).unwrap();
	let create2_call = RuntimeCall::EVM(TangleApi::evm::Call::create2 {
		source: H160::from_str("0x8efcaf2c4ebbf88bf07f3bb44a2869c4c675ad7a").unwrap(),
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

	let _contract_address = match deployed_contract {
		Some(contract) => {
			println!("Contract {:?} deployed at : {:?}", contract.address, result.block_hash());
			contract.address
		},
		None => return Err("Contract failed to deploy".to_string()),
	};

	Ok(())
}
