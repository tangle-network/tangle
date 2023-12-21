use std::{
	fs::File,
	io::Read,
	path::{Path, PathBuf},
	str::FromStr,
};

use fp_evm::GenesisAccount;
use serde_json::Value;
use sp_core::{crypto::Ss58Codec, H160, U256};
use sp_runtime::AccountId32;
use tangle_testnet_runtime::{AccountId, Balance};

pub fn get_git_root() -> PathBuf {
	let git_root = std::process::Command::new("git")
		.args(["rev-parse", "--show-toplevel"])
		.output()
		.expect("Failed to get git root")
		.stdout;
	let git_root_str = String::from_utf8(git_root).expect("Invalid UTF-8 sequence");
	PathBuf::from(git_root_str.trim())
}

pub fn read_contents(path: &Path) -> Value {
	let mut file = File::open(path).expect("file should open read only");
	let mut contents = String::new();
	file.read_to_string(&mut contents).expect("file should be readable");
	let json: serde_json::Value =
		serde_json::from_str(&contents).expect("file should be proper JSON");
	json
}

pub fn read_contents_to_evm_accounts(path_str: &str) -> Vec<H160> {
	let mut path = get_git_root();
	path.push(path_str);
	println!("Path {:?}", path_str);
	let json = read_contents(&path);
	let mut accounts = Vec::new();
	for address in json.as_array().expect("should be an object") {
		accounts.push(
			H160::from_str(address.as_str().expect("should be a string"))
				.expect("should be a valid address"),
		);
	}
	accounts
}

fn read_contents_to_substrate_accounts(path_str: &str) -> Vec<AccountId> {
	let mut path = get_git_root();
	path.push(path_str);
	println!("Path {:?}", path_str);
	let json = read_contents(&path);
	let mut accounts = Vec::new();
	for address in json.as_array().expect("should be an object") {
		accounts.push(
			AccountId::from_ss58check(address.as_str().expect("should be a string"))
				.expect("should be a valid address"),
		);
	}
	accounts
}

pub fn get_edgeware_genesis_list() -> Vec<H160> {
	read_contents_to_evm_accounts("node/src/distributions/data/edgeware_genesis_participants.json")
}

pub fn get_edgeware_snapshot_list() -> Vec<AccountId> {
	read_contents_to_substrate_accounts(
		"node/src/distributions/data/edgeware_snapshot_participants.json",
	)
}

pub fn get_discord_list() -> Vec<H160> {
	read_contents_to_evm_accounts("node/src/distributions/data/discord_evm_addresses.json")
}

pub fn get_evm_balance_distribution() -> Vec<(H160, GenesisAccount)> {
	const ONE_TOKEN: u128 = 1_000_000_000_000_000_000;
	const ENDOWMENT: u128 = 100 * ONE_TOKEN;
	get_edgeware_genesis_list()
		.into_iter()
		.chain(get_discord_list())
		.map(|address| {
			(
				address,
				GenesisAccount {
					balance: U256::from(ENDOWMENT),
					code: Default::default(),
					nonce: Default::default(),
					storage: Default::default(),
				},
			)
		})
		.collect()
}

pub fn get_substrate_balance_distribution() -> Vec<(AccountId32, Balance)> {
	const ONE_TOKEN: u128 = 1_000_000_000_000_000_000;
	const ENDOWMENT: u128 = 100 * ONE_TOKEN;
	get_edgeware_snapshot_list()
		.into_iter()
		.map(|address| (address, Balance::from(ENDOWMENT)))
		.collect()
}
