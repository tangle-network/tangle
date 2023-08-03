use std::{fs::File, io::Read, str::FromStr};

use fp_evm::GenesisAccount;
use sp_core::{crypto::Ss58Codec, H160, U256};
use sp_runtime::AccountId32;
use tangle_runtime::{AccountId, Balance};

/// Read in the list of Ethereum addresses from  `edgeware_participation.json`
/// and return the list.
fn get_edgeware_participation_list() -> Vec<H160> {
	// Print the current directory
	println!("Current directory: {}", std::env::current_dir().unwrap().display());
	let mut file = File::open("./src/distributions/data/edgeware_participants.json")
		.expect("file should open read only");
	let mut contents = String::new();
	file.read_to_string(&mut contents).expect("file should be readable");
	let json: serde_json::Value =
		serde_json::from_str(&contents).expect("file should be proper JSON");
	let mut addresses = Vec::new();
	for address in json.as_array().expect("should be an array") {
		addresses.push(
			H160::from_str(address.as_str().expect("should be a string"))
				.expect("should be a valid address"),
		);
	}
	addresses
}

/// Read in the list of Kabocha public keys, convert them to `AccountId`,
/// and return the list.
fn get_kabocha_participation_list() -> Vec<AccountId> {
	let mut file = File::open("./src/distributions/data/kabocha_participants.json")
		.expect("file should open read only");
	let mut contents = String::new();
	file.read_to_string(&mut contents).expect("file should be readable");
	let json: serde_json::Value =
		serde_json::from_str(&contents).expect("file should be proper JSON");
	let mut accounts = Vec::new();
	for address in json.as_array().expect("should be an array") {
		accounts.push(
			AccountId::from_ss58check(address.as_str().expect("should be a string"))
				.expect("should be a valid address"),
		);
	}
	accounts
}

/// Read in the list of Ethereum addresses from  `edgeware_genesis.json`
/// and return the list, giving each address a balance.
pub fn get_evm_balance_distribution() -> Vec<(H160, GenesisAccount)> {
	const ONE_TOKEN: u128 = 1_000_000_000_000_000_000;
	const ENDOWMENT: u128 = 100 * ONE_TOKEN;
	get_edgeware_participation_list()
		.into_iter()
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

/// Read in the list of Kabocha public keys from `kabocha_genesis.json`
/// and return the list, giving each address a balance.
pub fn get_substrate_balance_distribution() -> Vec<(AccountId32, Balance)> {
	const ONE_TOKEN: u128 = 1_000_000_000_000_000_000;
	const ENDOWMENT: u128 = 100 * ONE_TOKEN;
	get_kabocha_participation_list()
		.into_iter()
		.map(|address| (address.into(), Balance::from(ENDOWMENT)))
		.collect()
}
