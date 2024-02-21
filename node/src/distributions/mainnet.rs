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
use super::{
	get_git_root, read_contents, read_contents_to_evm_accounts,
	read_contents_to_substrate_accounts_list,
};
use hex_literal::hex;
use pallet_airdrop_claims::{EthereumAddress, MultiAddress, StatementKind};
use sp_core::H160;
use sp_runtime::{traits::AccountIdConversion, AccountId32};
use std::{collections::BTreeMap, str::FromStr};
use tangle_primitives::types::BlockNumber;
use tangle_runtime::{AccountId, Balance, ExistentialDeposit, Perbill, UNIT};

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

fn read_investor_accounts_to_multiaddress(path_str: &str) -> BTreeMap<MultiAddress, f64> {
	let mut path = get_git_root();
	path.push(path_str);
	let json = read_contents(&path);
	let json_obj = json.as_object().expect("should be an object");
	let mut accounts_map = BTreeMap::new();
	for (key, value) in json_obj {
		// eth address start with `0x`
		let first = key.chars().nth(0).unwrap();
		if first == '0' {
			let account_id = H160::from_str(key).expect("should be a valid address");
			let balance = value.as_f64().expect("Invalid balance");

			if balance <= 0.0 {
				continue
			}

			accounts_map.insert(MultiAddress::EVM(account_id.into()), balance);
		} else {
			let account_id = AccountId::from_str(key).expect("Invalid account ID");
			let balance = value.as_f64().expect("Invalid balance");
			if balance <= 0.0 {
				continue
			}
			accounts_map.insert(MultiAddress::Native(account_id), balance);
		}
	}
	accounts_map
}

// *** Distribution
// Team : 30% (5% immediate) (team account gets 95% that is vested over 2years with 1 year cliff))
// Foundation : 15% (5% immediate) (foundation account gets 95% that is vested over 2years with 1
// year cliff)
// Investors : 16% (5% liquid immediately)(investor accounts gets 95% that is vested
// over 2years with 1 year cliff)
// Treasury : 35% (immediate release to treasury pallet account)
// EDG Genesis Airdrop : 1% (5% immediate release)(95% vested over two years, with one month cliff)
// EDG Snapshot Airdrop : 1% (5% immediate release)(95% vested over two years, with one month cliff)
// Leaderboard airdrop : 2% (5% immediate release)(95% vested over two years, with one month cliff)
// ***

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
	read_contents_to_substrate_accounts_list(
		"node/src/distributions/data/substrate_faucet_addresses.json",
	)
}

fn get_edgeware_snapshot_list() -> BTreeMap<AccountId32, f64> {
	read_contents_to_substrate_accounts(
		"node/src/distributions/data/edgeware_snapshot_distribution.json",
	)
}

fn get_investor_balance_distribution_list() -> BTreeMap<MultiAddress, f64> {
	read_investor_accounts_to_multiaddress(
		"node/src/distributions/data/webb_investor_distribution.json",
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

fn ninety_five_percent_endowment(endowment: u128) -> u128 {
	endowment * 95 / 100
}

fn five_percent_endowment(endowment: u128) -> u128 {
	endowment - ninety_five_percent_endowment(endowment)
}

fn vesting_per_block(endowment: u128, blocks: u64) -> u128 {
	print!("Endowment {:?} Blocks {:?} ", endowment, blocks);
	endowment / blocks as u128
}

fn get_team_distribution_share() -> Perbill {
	Perbill::from_rational(30_u32, 100_u32)
}

fn get_investor_distribution_share() -> Perbill {
	Perbill::from_rational(16_u32, 100_u32)
}

fn get_foundation_distribution_share() -> Perbill {
	Perbill::from_rational(15_u32, 100_u32)
}

fn get_treasury_distribution_share() -> Perbill {
	Perbill::from_rational(35_u32, 100_u32)
}

fn get_initial_liquidity_share() -> Perbill {
	Perbill::from_rational(5_u32, 100_u32)
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
	let investor_accounts: Vec<(MultiAddress, u128)> = get_investor_balance_distribution_list()
		.into_iter()
		.map(|(address, balance)| (address, balance as u128))
		.collect();
	compute_balance_distribution_with_cliff_and_vesting_no_endowment(investor_accounts)
}

pub fn get_team_balance_distribution() -> Vec<(MultiAddress, u128, u64, u64, u128)> {
	let team_address: AccountId =
		hex!["8e1c2bdddab9573d8cb094dbffba24a2b2c21b7e71e3f5b604e8607483872443"].into();
	let balance =
		(get_team_distribution_share() - get_initial_liquidity_share()).mul_floor(TOTAL_SUPPLY);
	let team_account = (MultiAddress::Native(team_address), balance as u128);
	compute_balance_distribution_with_cliff_and_vesting(vec![team_account])
}

pub fn get_treasury_balance() -> (AccountId, u128) {
	let pallet_id = tangle_primitives::treasury::TREASURY_PALLET_ID;
	let acc: AccountId = pallet_id.into_account_truncating();

	// any leftover from investors are sent to treasury
	let investors_actual_spend = get_investor_balance_distribution_list()
		.into_values()
		.map(|balance| (balance as u128))
		.sum::<u128>();

	let investors_actual_spend_as_percent =
		Perbill::from_rational(investors_actual_spend, TOTAL_SUPPLY);
	let leftover_from_investors =
		get_investor_distribution_share() - investors_actual_spend_as_percent;
	let leftover_amount = leftover_from_investors * TOTAL_SUPPLY;

	(acc, get_treasury_distribution_share() * TOTAL_SUPPLY + leftover_amount)
}

pub fn get_foundation_balance_distribution() -> Vec<(MultiAddress, u128, u64, u64, u128)> {
	let foundation_address: AccountId =
		hex!["0cdd6ca9c578fabcc65373004944a401866d5c61568ffb22ecd8ef528599f95b"].into();
	let balance = get_foundation_distribution_share().mul_floor(TOTAL_SUPPLY) -
		get_initial_liquidity_share()
			.mul_floor(get_foundation_distribution_share().mul_floor(TOTAL_SUPPLY));
	let foundation_account = (MultiAddress::Native(foundation_address), balance as u128);
	compute_balance_distribution_with_cliff_and_vesting(vec![foundation_account])
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
				five_percent_endowment(value),
			)
		})
		.collect()
}

pub fn compute_balance_distribution_with_cliff_and_vesting_no_endowment(
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
				Default::default(),
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
		let claimable_amount = five_percent_endowment(value);
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
	// 5% is immedately release
	// 1 year cliff (vesting starts after year 1)
	// Vesting finishes 1 year after cliff
	let alice_expected_response: (MultiAddress, u128, u64, u64, u128) = (
		alice.clone(),
		amount_per_investor,
		tangle_primitives::time::DAYS * 365, // begins at one year after block 0
		tangle_primitives::time::DAYS * 365, // num of blocks from beginning till fully vested
		5,                                   // 5% of 100
	);
	let bob_expected_response: (MultiAddress, u128, u64, u64, u128) = (
		bob.clone(),
		amount_per_investor,
		tangle_primitives::time::DAYS * 365, // begins at one year after block 0
		tangle_primitives::time::DAYS * 365, // num of blocks from beging till fully vested
		5,                                   // 5% of 100
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
	// 5% is immedately claimable
	// 1 month cliff (vesting starts after 1 month) (use 1 for easier calculation)
	// at 1 month cliff, release 1/24th rewards
	// Vesting finishes after 2 years (use 24 for easier calculation)
	// 1/24th claimable at every month
	let expected_distibution_result = DistributionResult {
		claims: vec![
			(alice.clone(), 5, Some(StatementKind::Regular)),
			(bob.clone(), 5, Some(StatementKind::Regular)),
		],
		vesting: vec![
			(alice.clone(), vec![(3, 3, 1), (91, 3, 1)]),
			(bob.clone(), vec![(3, 3, 1), (91, 3, 1)]),
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
