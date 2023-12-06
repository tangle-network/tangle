use std::str::FromStr;

use fp_evm::GenesisAccount;

use sp_core::{H160, U256};
use sp_runtime::AccountId32;
use std::collections::BTreeMap;
use tangle_testnet_runtime::{AccountId, Balance, ExistentialDeposit};

use super::testnet::{get_git_root, read_contents, read_contents_to_evm_accounts};

/// The contents of the file should be a map of accounts to balances.
fn read_contents_to_substrate_accounts(path_str: &str) -> BTreeMap<AccountId, f64> {
	let mut path = get_git_root();
	path.push(path_str);
	let json = read_contents(&path);
	let json_obj = json.as_object().expect("should be an object");
	let mut accounts_map = BTreeMap::new();
	for (key, value) in json_obj {
		let account_id = AccountId::from_str(key).expect("Invalid account ID");
		let balance = value.as_f64().expect("Invalid balance");
		if balance <= 0.0 {
			continue
		}

		*accounts_map.entry(account_id).or_insert(0.0) += balance;
	}
	accounts_map
}

fn get_edgeware_genesis_list() -> Vec<H160> {
	read_contents_to_evm_accounts(
		"standalone/node/src/distributions/data/edgeware_genesis_participants.json",
	)
}

fn get_edgeware_snapshot_list() -> BTreeMap<AccountId32, f64> {
	read_contents_to_substrate_accounts(
		"standalone/node/src/distributions/data/edgeware_snapshot_distribution.json",
	)
}

fn get_discord_list() -> Vec<H160> {
	read_contents_to_evm_accounts(
		"standalone/node/src/distributions/data/discord_evm_addresses.json",
	)
}

pub fn get_edgeware_genesis_balance_distribution() -> Vec<(H160, GenesisAccount)> {
	const ONE_TOKEN: u128 = 1_000_000_000_000_000_000;
	const ENDOWMENT: u128 = 100 * ONE_TOKEN;
	get_edgeware_genesis_list()
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

pub fn get_leaderboard_balance_distribution() -> Vec<(H160, GenesisAccount)> {
	const ONE_TOKEN: u128 = 1_000_000_000_000_000_000;
	const ENDOWMENT: u128 = ONE_TOKEN;
	get_discord_list()
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

pub fn get_substrate_balance_distribution() -> Vec<(AccountId32, Balance)> {
	const ONE_TOKEN: u128 = 1_000_000_000_000_000_000;
	const ENDOWMENT: u128 = 10_000_000 * ONE_TOKEN;

	let convert_to_u128 = |value: f64| -> u128 { (ENDOWMENT as f64 * value) as u128 };

	let balances: Vec<(AccountId32, Balance)> = get_edgeware_snapshot_list()
		.into_iter()
		.filter(|(_, value)| *value > 0.0)
		.map(|(address, value)| (address, Balance::from(convert_to_u128(value))))
		.filter(|(_, value)| *value > ExistentialDeposit::get())
		.collect();

	balances
}
