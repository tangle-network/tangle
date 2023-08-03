use std::{fs::File, io::Read, str::FromStr};

use fp_evm::GenesisAccount;
use sp_core::{H160, U256};

/// Read in the list of Ethereum addresses from  `edgeware_participation.json`
/// and return the list.
fn get_participation_list() -> Vec<H160> {
	let mut file =
		File::open("./data/edgeware_participation.json").expect("file should open read only");
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

/// Read in the list of Ethereum addresses from  `edgeware_genesis.json`
/// and return the list, giving each address a balance.
pub fn get_distribution() -> Vec<(H160, GenesisAccount)> {
	const ONE_TOKEN: u128 = 1_000_000_000_000_000_000;
	const ENDOWMENT: u128 = 100 * ONE_TOKEN;
	get_participation_list()
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
