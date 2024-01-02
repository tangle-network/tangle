#![allow(clippy::type_complexity)]
// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.
use std::str::FromStr;
use tangle_primitives::BlockNumber;
use tangle_runtime::UNIT;

use super::testnet::{get_git_root, read_contents, read_contents_to_evm_accounts};
use pallet_airdrop_claims::{EthereumAddress, MultiAddress, StatementKind};
use sp_core::H160;
use sp_runtime::AccountId32;
use std::collections::BTreeMap;
use tangle_testnet_runtime::{AccountId, Balance, ExistentialDeposit};

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

pub fn get_edgeware_genesis_list() -> Vec<H160> {
	read_contents_to_evm_accounts("node/src/distributions/data/edgeware_genesis_participants.json")
}

pub fn get_faucet_evm_list() -> Vec<H160> {
	read_contents_to_evm_accounts("node/src/distributions/data/evm_faucet_addresses.json")
}

pub fn get_bridge_evm_list() -> Vec<H160> {
	read_contents_to_evm_accounts("node/src/distributions/data/evm_bridge_addresses.json")
}

pub fn get_faucet_substrate_list() -> Vec<AccountId32> {
	super::testnet::read_contents_to_substrate_accounts(
		"node/src/distributions/data/substrate_faucet_addresses.json",
	)
}

fn get_edgeware_snapshot_list() -> BTreeMap<AccountId32, f64> {
	read_contents_to_substrate_accounts(
		"node/src/distributions/data/edgeware_snapshot_distribution.json",
	)
}

pub fn get_discord_list() -> Vec<H160> {
	read_contents_to_evm_accounts("node/src/distributions/data/discord_evm_addresses.json")
}

pub const ONE_TOKEN: u128 = UNIT;
pub const TOTAL_SUPPLY: u128 = 100_000_000 * ONE_TOKEN;
pub const ONE_PERCENT_TOTAL_SUPPLY: u128 = TOTAL_SUPPLY / 100;
pub const BLOCK_TIME: u128 = 6;
pub const ONE_MONTH_BLOCKS: u64 = (30 * 24 * 60 * 60 / BLOCK_TIME) as u64;
pub const ONE_YEAR_BLOCKS: u64 = (365 * 24 * 60 * 60 / BLOCK_TIME) as u64;
pub const TWO_YEARS_BLOCKS: u64 = (2 * 365 * 24 * 60 * 60 / BLOCK_TIME) as u64;

pub const ONE_HUNDRED_POINTS: u64 = 100;

#[derive(PartialEq, Eq, Debug)]
pub struct DistributionResult {
	pub claims: Vec<(MultiAddress, Balance, Option<StatementKind>)>,
	pub vesting: Vec<(MultiAddress, Vec<(Balance, Balance, BlockNumber)>)>,
	pub vesting_length: BlockNumber,
	pub vesting_cliff: BlockNumber,
}

fn ninety_nine_percent_endowment(endowment: u128) -> u128 {
	endowment * 99 / 100
}

fn one_percent_endowment(endowment: u128) -> u128 {
	endowment - ninety_nine_percent_endowment(endowment)
}

fn vesting_per_block(endowment: u128, blocks: u64) -> u128 {
	print!("Endowment {:?} Blocks {:?} ", endowment, blocks);
	endowment / blocks as u128
}

pub fn get_edgeware_genesis_balance_distribution() -> DistributionResult {
	let list = get_edgeware_genesis_list();
	let endowment = ONE_PERCENT_TOTAL_SUPPLY / list.len() as u128;
	let edgeware_genesis_list: Vec<(MultiAddress, u128)> = list
		.into_iter()
		.map(|address| (MultiAddress::EVM(EthereumAddress(address.0)), endowment))
		.collect();
	get_distribution_for(
		edgeware_genesis_list,
		Some(StatementKind::Regular),
		ONE_MONTH_BLOCKS,
		TWO_YEARS_BLOCKS,
	)
}

pub fn get_leaderboard_balance_distribution() -> DistributionResult {
	let discord_list: Vec<(MultiAddress, u64)> = get_discord_list()
		.into_iter()
		.map(|address| (MultiAddress::EVM(EthereumAddress(address.0)), ONE_HUNDRED_POINTS))
		.collect();

	let faucet_evm_list: Vec<(MultiAddress, u64)> = get_faucet_evm_list()
		.into_iter()
		.map(|address| (MultiAddress::EVM(EthereumAddress(address.0)), ONE_HUNDRED_POINTS))
		.collect();

	let faucet_substrate_list: Vec<(MultiAddress, u64)> = get_faucet_substrate_list()
		.into_iter()
		.map(|address| (MultiAddress::Native(address), ONE_HUNDRED_POINTS))
		.collect();

	let bridge_evm_list: Vec<(MultiAddress, u64)> = get_bridge_evm_list()
		.into_iter()
		.map(|address| (MultiAddress::EVM(EthereumAddress(address.0)), ONE_HUNDRED_POINTS))
		.collect();

	let leaderboard_points: Vec<(MultiAddress, u64)> = vec![];
	// Chain all point lists together
	let points_list = discord_list
		.into_iter()
		.chain(leaderboard_points)
		.chain(faucet_evm_list)
		.chain(faucet_substrate_list)
		.chain(bridge_evm_list)
		.collect::<Vec<(MultiAddress, u64)>>();
	// Sum all the points
	let total_points = points_list.iter().map(|(_, points)| points).sum::<u64>();
	let combined_balances: Vec<(MultiAddress, u128)> = points_list
		.into_iter()
		.map(|(address, points)| {
			(address, (points as u128 * ONE_PERCENT_TOTAL_SUPPLY) / total_points as u128)
		})
		.collect();

	get_distribution_for(
		combined_balances,
		Some(StatementKind::Regular),
		ONE_MONTH_BLOCKS,
		TWO_YEARS_BLOCKS,
	)
}

/// Used for testing purposes
///
/// DO NOT USE IN MAINNET
pub fn get_local_balance_distribution() -> DistributionResult {
	let list = vec![
		// Test account with a simple menmonic
		// Mnemonic: "test test test test test test test test test test test junk"
		// Path: m/44'/60'/0'/0/0
		// Private Key: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
		H160::from_str("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
			.expect("internal H160 is valid; qed"),
		// Test account with a simple menmonic
		// Mnemonic: "test test test test test test test test test test test junk"
		// Path: m/44'/60'/0'/0/1
		// Private Key: 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
		H160::from_str("70997970C51812dc3A010C7d01b50e0d17dc79C8")
			.expect("internal H160 is valid; qed"),
		// H160 address of Alice dev account
		// Derived from SS58 (42 prefix) address
		// SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
		// hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
		// Using the full hex key, truncating to the first 20 bytes (the first 40 hex
		// chars)
		H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
			.expect("internal H160 is valid; qed"),
	];
	let endowment = ONE_PERCENT_TOTAL_SUPPLY / list.len() as u128;
	let local_list: Vec<(MultiAddress, u128)> = list
		.into_iter()
		.map(|address| (MultiAddress::EVM(EthereumAddress(address.0)), endowment))
		.collect();
	get_distribution_for(
		local_list,
		Some(StatementKind::Regular),
		ONE_MONTH_BLOCKS,
		TWO_YEARS_BLOCKS,
	)
}

pub fn get_substrate_balance_distribution() -> DistributionResult {
	let arr = get_edgeware_snapshot_list()
		.into_iter()
		.filter(|(_, value)| *value > 0.0)
		.map(|(address, value)| {
			(MultiAddress::Native(address), (value * ONE_PERCENT_TOTAL_SUPPLY as f64) as u128)
		})
		.collect();

	get_distribution_for(arr, Some(StatementKind::Regular), ONE_MONTH_BLOCKS, TWO_YEARS_BLOCKS)
}

pub fn get_investor_balance_distribution() -> Vec<(MultiAddress, u128, u64, u64, u128)> {
	// TODO : Read from actual investor file
	let investor_accounts: Vec<(MultiAddress, u128)> = vec![];
	compute_balance_distribution_with_cliff_and_vesting(investor_accounts)
}

pub fn get_team_balance_distribution() -> Vec<(MultiAddress, u128, u64, u64, u128)> {
	// TODO : Read from actual team file
	let team_accounts: Vec<(MultiAddress, u128)> = vec![];
	compute_balance_distribution_with_cliff_and_vesting(team_accounts)
}

pub fn compute_balance_distribution_with_cliff_and_vesting(
	investor_accounts: Vec<(MultiAddress, u128)>,
) -> Vec<(MultiAddress, u128, u64, u64, u128)> {
	investor_accounts
		.into_iter()
		.map(|(address, value)| {
			(
				address,
				value,
				ONE_YEAR_BLOCKS,
				TWO_YEARS_BLOCKS - ONE_YEAR_BLOCKS,
				one_percent_endowment(value),
			)
		})
		.collect()
}

pub fn get_distribution_for(
	arr: Vec<(MultiAddress, u128)>,
	statement_kind: Option<StatementKind>,
	vesting_cliff: u64,
	total_vesting_schedule: u64,
) -> DistributionResult {
	let mut claims = vec![];
	let mut vesting = vec![];
	arr.into_iter().filter(|(_, value)| *value > 0).for_each(|(address, value)| {
		let claimable_amount = one_percent_endowment(value);
		let vested_amount = value - claimable_amount;
		let cliff_fraction = vesting_cliff as f64 / total_vesting_schedule as f64;
		let remaining_fraction = 1.0 - cliff_fraction;

		if claimable_amount <= ExistentialDeposit::get() {
			log::warn!(
				"One percent endowment for account {:?} is not above the existential deposit",
				address.clone()
			);
		}

		claims.push((address.clone(), claimable_amount, statement_kind));
		let amount_on_cliff = (vested_amount as f64 * cliff_fraction) as u128;
		let amount_after_cliff = (vested_amount as f64 * remaining_fraction) as u128;
		let amount_unlocked_per_block_after_cliff =
			vesting_per_block(amount_after_cliff, total_vesting_schedule - vesting_cliff);
		vesting.push((
			address,
			vec![
				(amount_on_cliff, amount_on_cliff, vesting_cliff),
				(amount_after_cliff, amount_unlocked_per_block_after_cliff, vesting_cliff),
			],
		));
	});

	DistributionResult { claims, vesting, vesting_length: total_vesting_schedule, vesting_cliff }
}

#[test]
fn test_compute_investor_balance_distribution() {
	let alice = MultiAddress::Native(AccountId32::new([0; 32]));
	let bob = MultiAddress::Native(AccountId32::new([1; 32]));

	let amount_per_investor = 100;

	// let compute the expected output
	// the expected output is that
	// 1% is immedately release
	// 1 year cliff (vesting starts after year 1)
	// Vesting finishes 1 year after cliff
	let alice_expected_response: (MultiAddress, u128, u64, u64, u128) = (
		alice.clone(),
		amount_per_investor,
		tangle_primitives::time::DAYS * 365, // begins at one year after block 0
		tangle_primitives::time::DAYS * 365, // num of blocks from beging till fully vested
		1,                                   // 1% of 100
	);
	let bob_expected_response: (MultiAddress, u128, u64, u64, u128) = (
		bob.clone(),
		amount_per_investor,
		tangle_primitives::time::DAYS * 365, // begins at one year after block 0
		tangle_primitives::time::DAYS * 365, // num of blocks from beging till fully vested
		1,                                   // 1% of 100
	);

	assert_eq!(
		compute_balance_distribution_with_cliff_and_vesting(vec![
			(alice, amount_per_investor),
			(bob, amount_per_investor),
		]),
		vec![alice_expected_response, bob_expected_response]
	);
}

#[test]
fn test_get_distribution_for() {
	let alice = MultiAddress::Native(AccountId32::new([0; 32]));
	let bob = MultiAddress::Native(AccountId32::new([1; 32]));

	let amount_per_investor = 100;

	// let compute the expected output
	// the expected output is that
	// 1% is immedately claimable
	// 1 month cliff (vesting starts after 1 month) (use 1 for easier calculation)
	// at 1 month cliff, release 1/24th rewards
	// Vesting finishes after 2 years (use 24 for easier calculation)
	// 1/24th claimable at every month
	let expected_distibution_result = DistributionResult {
		claims: vec![
			(alice.clone(), 1, Some(StatementKind::Regular)),
			(bob.clone(), 1, Some(StatementKind::Regular)),
		],
		vesting: vec![
			(alice.clone(), vec![(4, 4, 1), (94, 4, 1)]),
			(bob.clone(), vec![(4, 4, 1), (94, 4, 1)]),
		],
		vesting_length: 24,
		vesting_cliff: 1,
	};

	assert_eq!(
		get_distribution_for(
			vec![(alice, amount_per_investor), (bob, amount_per_investor),],
			Some(StatementKind::Regular),
			1,
			24,
		),
		expected_distibution_result
	);
}
