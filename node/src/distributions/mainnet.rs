#![allow(clippy::type_complexity)]
// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
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

#[cfg(not(feature = "testnet"))]
use crate::mainnet_fixtures::{get_initial_authorities, get_root_key};

#[cfg(feature = "testnet")]
use crate::testnet_fixtures::{get_initial_authorities, get_testnet_root_key as get_root_key};

use hex_literal::hex;
use pallet_airdrop_claims::{EthereumAddress, MultiAddress, StatementKind};
use sp_core::H160;
use sp_runtime::{AccountId32, traits::AccountIdConversion};
use std::{collections::BTreeMap, str::FromStr};
use tangle_primitives::types::BlockNumber;
use tangle_runtime::{AccountId, Balance, Perbill, UNIT};

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
			continue;
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
		if key.starts_with("0x") {
			let account_id = H160::from_str(key).expect("should be a valid address");
			let balance = value.as_f64().expect("Invalid balance");

			if balance <= 0.0 {
				continue;
			}

			accounts_map.insert(MultiAddress::EVM(account_id.into()), balance);
		} else {
			let account_id = AccountId::from_str(key).expect("Invalid account ID");
			let balance = value.as_f64().expect("Invalid balance");
			if balance <= 0.0 {
				continue;
			}
			accounts_map.insert(MultiAddress::Native(account_id), balance);
		}
	}
	accounts_map
}

// *** Distribution
// Team : 30% (team account is vested over two years with one year cliff)
// Foundation : 15% (foundation account is vested over two years with one year cliff)
// Investors : 13.64% (investor accounts is vested over two years with one year cliff)
// Treasury : 36.36% (immediate release to treasury pallet account)
// EDG Genesis Airdrop : 1% (5% immediate release)(95% vested over two years, with one month cliff)
// EDG Snapshot Airdrop : 1% (5% immediate release)(95% vested over two years, with one month cliff)
// Leaderboard airdrop : 2% (5% immediate release)(95% vested over two years, with one month cliff)
// Polkadot validator airdrop : 1% (5% immediate release)(95% vested over two years, with one month
// cliff) ***

pub fn get_edgeware_genesis_list() -> Vec<H160> {
	read_contents_to_evm_accounts("node/src/distributions/data/edgeware_genesis_participants.json")
}

pub fn get_bridge_evm_list() -> Vec<H160> {
	read_contents_to_evm_accounts("node/src/distributions/data/evm_bridge_addresses.json")
}

fn get_leaderboard_distribution() -> BTreeMap<AccountId32, f64> {
	// leaderboard data last updated : 27/03/2024
	read_contents_to_substrate_accounts("node/src/distributions/data/leaderboard_addresses.json")
}

fn get_edgeware_snapshot_list() -> BTreeMap<AccountId32, f64> {
	read_contents_to_substrate_accounts(
		"node/src/distributions/data/edgeware_snapshot_distribution.json",
	)
}

fn get_investor_balance_distribution_list() -> BTreeMap<MultiAddress, f64> {
	read_investor_accounts_to_multiaddress(
		"node/src/distributions/data/webb_investor_distributions.json",
	)
}

fn get_team_vesting_only_cliff_accounts() -> BTreeMap<AccountId32, f64> {
	read_contents_to_substrate_accounts(
		"node/src/distributions/data/webb_team_distributions_only_cliff.json",
	)
}

fn get_team_vesting_accounts() -> BTreeMap<AccountId32, f64> {
	read_contents_to_substrate_accounts("node/src/distributions/data/webb_team_distributions.json")
}

pub fn get_polkadot_validator_address_list() -> Vec<AccountId32> {
	read_contents_to_substrate_accounts_list(
		"node/src/distributions/data/polkadot_validator_addresses.json",
	)
}

pub const ONE_TOKEN: u128 = UNIT;
pub const TOTAL_SUPPLY: u128 = 100_000_000 * ONE_TOKEN;
pub const ONE_PERCENT_TOTAL_SUPPLY: u128 = TOTAL_SUPPLY / 100;
pub const TWO_PERCENT_TOTAL_SUPPLY: u128 = ONE_PERCENT_TOTAL_SUPPLY * 2;
pub const BLOCK_TIME: u128 = 6;
pub const ONE_MONTH_BLOCKS: u64 = (30 * 24 * 60 * 60 / BLOCK_TIME) as u64;
pub const ONE_YEAR_BLOCKS: u64 = (365 * 24 * 60 * 60 / BLOCK_TIME) as u64;
pub const TWO_YEARS_BLOCKS: u64 = (2 * 365 * 24 * 60 * 60 / BLOCK_TIME) as u64;

pub const ONE_HUNDRED_POINTS: u64 = 100;

#[derive(PartialEq, Eq, Debug, Default)]
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
	//print!("Endowment {:?} Blocks {:?} ", endowment, blocks);
	endowment / blocks as u128
}

fn get_foundation_distribution_share() -> Perbill {
	Perbill::from_rational(15_u32, 100_u32)
}

fn get_treasury_distribution_share() -> Perbill {
	Perbill::from_float(0.3636_f64)
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
	let bridge_evm_list: Vec<(MultiAddress, u64)> = get_bridge_evm_list()
		.into_iter()
		.map(|address| (MultiAddress::EVM(EthereumAddress(address.0)), ONE_HUNDRED_POINTS))
		.collect();

	let leaderboard_points: Vec<(MultiAddress, u64)> = get_leaderboard_distribution()
		.into_iter()
		.map(|(address, points)| (MultiAddress::Native(address), points as u64))
		.collect();

	// Chain all point lists together
	let points_list = bridge_evm_list
		.into_iter()
		.chain(leaderboard_points)
		.collect::<Vec<(MultiAddress, u64)>>();

	// Sum all the points
	let total_points = points_list.iter().map(|(_, points)| points).sum::<u64>();
	let combined_balances: Vec<(MultiAddress, u128)> = points_list
		.into_iter()
		.map(|(address, points)| {
			(address, (points as u128 * TWO_PERCENT_TOTAL_SUPPLY) / total_points as u128)
		})
		.collect();

	get_distribution_for(
		combined_balances,
		Some(StatementKind::Regular),
		ONE_MONTH_BLOCKS,
		TWO_YEARS_BLOCKS,
	)
}

pub fn get_edgeware_snapshot_distribution() -> DistributionResult {
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

pub fn get_team_direct_vesting_distribution() -> Vec<(MultiAddress, u128, u64, u64, u128)> {
	let direct_team_accounts: Vec<(MultiAddress, u128)> = get_team_vesting_only_cliff_accounts()
		.into_iter()
		.map(|(address, balance)| (MultiAddress::Native(address), balance as u128))
		.collect();

	compute_balance_distribution_with_cliff_and_no_endowment(direct_team_accounts)
}

pub fn get_team_balance_distribution() -> Vec<(MultiAddress, u128, u64, u64, u128)> {
	let team_accounts: Vec<(MultiAddress, u128)> = get_team_vesting_accounts()
		.into_iter()
		.map(|(address, balance)| (MultiAddress::Native(address), balance as u128))
		.collect();
	compute_balance_distribution_with_cliff_and_vesting(team_accounts)
}

pub fn get_initial_endowed_accounts()
-> (Vec<(AccountId, u128)>, Vec<(H160, fp_evm::GenesisAccount)>) {
	let mut endowed_accounts = vec![];

	let pallet_id = tangle_primitives::treasury::TREASURY_PALLET_ID;
	let acc: AccountId = pallet_id.into_account_truncating();
	endowed_accounts.push((acc, get_treasury_distribution_share() * TOTAL_SUPPLY));

	let root_account: AccountId = get_root_key();
	endowed_accounts.push((root_account, 5000 * UNIT)); // root key gets 5000 tokens for transactions

	let initial_authorities = get_initial_authorities();

	#[cfg(not(feature = "testnet"))]
	for (acco, _, _, _, _) in initial_authorities.iter() {
		endowed_accounts.push((acco.clone(), 100 * UNIT));
	}

	#[cfg(feature = "testnet")]
	for (acco, _, _, _) in initial_authorities.iter() {
		endowed_accounts.push((acco.clone(), 100 * UNIT));
	}

	// all team and investor accounts get entire balance
	// this is a requirement for vesting pallet to lockup the balances later
	// see : https://github.com/paritytech/polkadot-sdk/blame/7241a8db7b3496816503c6058dae67f66c666b00/substrate/frame/vesting/src/lib.rs#L241
	for (inv_account, amount) in get_investor_balance_distribution_list() {
		endowed_accounts.push((inv_account.clone().to_account_id_32(), amount as u128))
	}

	for (team_account, amount) in get_team_vesting_only_cliff_accounts() {
		endowed_accounts.push((team_account, amount as u128));
	}

	for (team_account, amount) in get_team_vesting_accounts() {
		endowed_accounts.push((team_account, amount as u128));
	}

	let foundation_address: AccountId =
		hex!["0cdd6ca9c578fabcc65373004944a401866d5c61568ffb22ecd8ef528599f95b"].into();
	let balance = get_foundation_distribution_share().mul_floor(TOTAL_SUPPLY);
	endowed_accounts.push((foundation_address, balance as u128));

	//println!("Endowed accounts {:?}", endowed_accounts);
	(endowed_accounts, Default::default())
}

pub fn get_foundation_balance_distribution() -> Vec<(MultiAddress, u128, u64, u64, u128)> {
	let foundation_address: AccountId =
		hex!["0cdd6ca9c578fabcc65373004944a401866d5c61568ffb22ecd8ef528599f95b"].into();
	let balance = get_foundation_distribution_share().mul_floor(TOTAL_SUPPLY);
	let foundation_account = (MultiAddress::Native(foundation_address), balance as u128);
	compute_balance_distribution_with_cliff_and_vesting(vec![foundation_account])
}

pub fn get_polkadot_validator_distribution() -> DistributionResult {
	let list = get_polkadot_validator_address_list();
	let endowment = ONE_PERCENT_TOTAL_SUPPLY / list.len() as u128;
	let polkadot_validator_dist: Vec<(MultiAddress, u128)> = list
		.into_iter()
		.map(|address| (MultiAddress::Native(address), endowment))
		.collect();
	get_distribution_for(
		polkadot_validator_dist,
		Some(StatementKind::Regular),
		ONE_MONTH_BLOCKS,
		TWO_YEARS_BLOCKS,
	)
}

pub fn compute_balance_distribution_with_cliff_and_vesting(
	investor_accounts: Vec<(MultiAddress, u128)>,
) -> Vec<(MultiAddress, u128, u64, u64, u128)> {
	investor_accounts
		.into_iter()
		.map(|(address, value)| {
			(address, value, ONE_YEAR_BLOCKS, TWO_YEARS_BLOCKS - ONE_YEAR_BLOCKS, 100 * UNIT)
		})
		.collect()
}

pub fn compute_balance_distribution_with_cliff_and_vesting_no_endowment(
	investor_accounts: Vec<(MultiAddress, u128)>,
) -> Vec<(MultiAddress, u128, u64, u64, u128)> {
	investor_accounts
		.into_iter()
		.map(|(address, value)| {
			(address, value, ONE_YEAR_BLOCKS, TWO_YEARS_BLOCKS - ONE_YEAR_BLOCKS, 100 * UNIT)
		})
		.collect()
}

pub fn compute_balance_distribution_with_cliff_and_no_endowment(
	investor_accounts: Vec<(MultiAddress, u128)>,
) -> Vec<(MultiAddress, u128, u64, u64, u128)> {
	investor_accounts
		.into_iter()
		.map(|(address, value)| {
			(
				address,
				value,
				ONE_YEAR_BLOCKS,
				ONE_YEAR_BLOCKS, // immediately vested
				100 * UNIT,
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

		// the entire value is claimable here
		// the claims pallet will lock all the vesting balance so in effect only claimable-amount is
		// usable
		claims.push((address.clone(), value, statement_kind));
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

#[cfg(not(feature = "testnet"))]
#[test]
fn test_compute_investor_balance_distribution() {
	let alice = MultiAddress::Native(AccountId32::new([0; 32]));
	let bob = MultiAddress::Native(AccountId32::new([1; 32]));

	let amount_per_investor = 1000 * UNIT;

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
		100 * UNIT,                          // 100 units
	);
	let bob_expected_response: (MultiAddress, u128, u64, u64, u128) = (
		bob.clone(),
		amount_per_investor,
		tangle_primitives::time::DAYS * 365, // begins at one year after block 0
		tangle_primitives::time::DAYS * 365, // num of blocks from beging till fully vested
		100 * UNIT,                          // 100 units
	);

	assert_eq!(
		compute_balance_distribution_with_cliff_and_vesting(vec![
			(alice, amount_per_investor),
			(bob, amount_per_investor),
		]),
		vec![alice_expected_response, bob_expected_response]
	);
}

#[cfg(not(feature = "testnet"))]
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
			(alice.clone(), 100, Some(StatementKind::Regular)),
			(bob.clone(), 100, Some(StatementKind::Regular)),
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

#[cfg(not(feature = "testnet"))]
#[test]
fn test_distribution_shares() {
	// ============== compute total investor amount and share of distribution ================= //

	let investor_balance_account_distribution = get_investor_balance_distribution();
	let total_investor_amount: u128 = investor_balance_account_distribution
		.clone()
		.into_iter()
		.map(|(_, amount, _, _, _)| amount)
		.sum();

	assert_eq!(total_investor_amount, 13639999999999999916113920); // 13639999 TNT
	assert_eq!(
		Perbill::from_rational(total_investor_amount, TOTAL_SUPPLY),
		Perbill::from_float(0.136399999)
	); // 13.6399999%

	// ============== compute direct vesting team accounts ================= //
	let direct_team_accounts: Vec<(MultiAddress, u128)> = get_team_vesting_only_cliff_accounts()
		.into_iter()
		.map(|(address, balance)| (MultiAddress::Native(address), balance as u128))
		.collect();

	let total_direct_team_amount =
		direct_team_accounts.into_iter().map(|(_address, balance)| balance).sum();

	assert_eq!(total_direct_team_amount, 138150689999999993905152); // 138150 TNT
	assert_eq!(
		Perbill::from_rational(total_direct_team_amount, TOTAL_SUPPLY),
		Perbill::from_float(0.001381506)
	); // 0.1381506%

	// =========== compute treasury total amount ======================== //

	let total_treasury_amount = get_treasury_distribution_share() * TOTAL_SUPPLY;
	assert_eq!(total_treasury_amount, 36360000000000000000000000); // 36360000 TNT
	assert_eq!(
		Perbill::from_rational(total_treasury_amount, TOTAL_SUPPLY),
		Perbill::from_float(0.3636)
	); // 36.36%

	// ============== compute foundation total amount ==================== //

	let foundation_total_amount = get_foundation_distribution_share().mul_floor(TOTAL_SUPPLY);
	assert_eq!(foundation_total_amount, 15000000000000000000000000); // 15000000 TNT
	assert_eq!(
		Perbill::from_rational(foundation_total_amount, TOTAL_SUPPLY),
		Perbill::from_float(0.15)
	); // 15%

	// ============== compute edgeware distribution total amount ====================== //
	let edgeware_genesis_list = get_edgeware_genesis_balance_distribution();

	let total_edgeware_claims_amount: u128 = edgeware_genesis_list
		.claims
		.clone()
		.into_iter()
		.map(|(_, amount, _)| amount)
		.sum();
	let mut total_edgeware_vesting_amount: u128 = Default::default();

	for (_acc, vesting) in edgeware_genesis_list.vesting.clone().into_iter() {
		let cliff_vesting = vesting[0].0;
		let post_cliff_vesting = vesting[1].0;
		total_edgeware_vesting_amount += cliff_vesting + post_cliff_vesting;
	}

	assert_eq!(total_edgeware_claims_amount, 999999999999999999996210); // 50000 TNT
	assert_eq!(total_edgeware_vesting_amount, 949999999999999990376448); // 949999 TNT
	assert_eq!(
		Perbill::from_rational(total_edgeware_claims_amount, TOTAL_SUPPLY),
		Perbill::from_float(0.009999999)
	); // 1%
	assert_eq!(
		Perbill::from_rational(total_edgeware_vesting_amount, TOTAL_SUPPLY),
		Perbill::from_float(0.009499999)
	); // 0.9499999%

	// ============== compute edgeware snapshot distribution total amount ====================== //
	let edgeware_snapshot_list = get_edgeware_snapshot_distribution();

	let total_edgeware_snapshot_claims_amount: u128 = edgeware_snapshot_list
		.claims
		.clone()
		.into_iter()
		.map(|(_, amount, _)| amount)
		.sum();
	let mut total_edgeware_snapshot_vesting_amount: u128 = Default::default();

	for (_acc, vesting) in edgeware_snapshot_list.vesting.clone().into_iter() {
		let cliff_vesting = vesting[0].0;
		let post_cliff_vesting = vesting[1].0;
		total_edgeware_snapshot_vesting_amount += cliff_vesting + post_cliff_vesting;
	}

	assert_eq!(total_edgeware_snapshot_claims_amount, 1000000000000007675499460); // 50000 TNT
	assert_eq!(total_edgeware_snapshot_vesting_amount, 950000000000007330583432); // 949999 TNT
	assert_eq!(
		Perbill::from_rational(total_edgeware_snapshot_claims_amount, TOTAL_SUPPLY),
		Perbill::from_float(0.01)
	); // 1%
	assert_eq!(
		Perbill::from_rational(total_edgeware_snapshot_vesting_amount, TOTAL_SUPPLY),
		Perbill::from_float(0.0095)
	); // 0.95%

	// ============== compute leaderboard distribution total amount ====================== //
	let leaderboard_genesis_list = get_leaderboard_balance_distribution();

	let total_leaderboard_claims_amount: u128 = leaderboard_genesis_list
		.claims
		.clone()
		.into_iter()
		.map(|(_, amount, _)| amount)
		.sum();
	let mut total_leaderboard_vesting_amount: u128 = Default::default();

	for (_acc, vesting) in leaderboard_genesis_list.vesting.clone().into_iter() {
		let cliff_vesting = vesting[0].0;
		let post_cliff_vesting = vesting[1].0;
		total_leaderboard_vesting_amount += cliff_vesting + post_cliff_vesting;
	}

	assert_eq!(total_leaderboard_claims_amount, 1999999999999999999996877); // 100000 TNT
	assert_eq!(total_leaderboard_vesting_amount, 1900000000000000087640984); // 1900000 TNT
	assert_eq!(
		Perbill::from_rational(total_leaderboard_claims_amount, TOTAL_SUPPLY),
		Perbill::from_float(0.019999999)
	); // 1.9999999%
	assert_eq!(
		Perbill::from_rational(total_leaderboard_vesting_amount, TOTAL_SUPPLY),
		Perbill::from_float(0.019)
	); // 1.9%

	// ============== compute polkadot distribution total amount ====================== //
	let polkadot_genesis_list = get_polkadot_validator_distribution();

	let total_polkadot_claims_amount: u128 = polkadot_genesis_list
		.claims
		.clone()
		.into_iter()
		.map(|(_, amount, _)| amount)
		.sum();
	let mut total_polkadot_vesting_amount: u128 = Default::default();

	for (_acc, vesting) in polkadot_genesis_list.vesting.clone().into_iter() {
		let cliff_vesting = vesting[0].0;
		let post_cliff_vesting = vesting[1].0;
		total_polkadot_vesting_amount += cliff_vesting + post_cliff_vesting;
	}

	assert_eq!(total_polkadot_claims_amount, 999999999999999999999204); // 50000 TNT
	assert_eq!(total_polkadot_vesting_amount, 950000000000000090505216); // 949999 TNT
	assert_eq!(
		Perbill::from_rational(total_polkadot_claims_amount, TOTAL_SUPPLY),
		Perbill::from_float(0.009999999)
	); // 0.9999999%
	assert_eq!(
		Perbill::from_rational(total_polkadot_vesting_amount, TOTAL_SUPPLY),
		Perbill::from_float(0.0095)
	); // 0.95%

	// Test total claims
	let total_claims = edgeware_genesis_list.claims.len()
		+ edgeware_snapshot_list.claims.len()
		+ polkadot_genesis_list.claims.len()
		+ leaderboard_genesis_list.claims.len();
	assert_eq!(total_claims, 29452);

	let total_vesting = edgeware_genesis_list.vesting.len()
		+ edgeware_snapshot_list.vesting.len()
		+ polkadot_genesis_list.vesting.len()
		+ leaderboard_genesis_list.vesting.len();
	assert_eq!(total_vesting, 29452);

	let unique_dist = crate::distributions::get_unique_distribution_results(vec![
		get_edgeware_genesis_balance_distribution(),
		get_leaderboard_balance_distribution(),
		get_edgeware_snapshot_distribution(),
		get_polkadot_validator_distribution(),
	]);
	assert_eq!(unique_dist.claims.len(), 13250);
	assert_eq!(unique_dist.vesting.len(), 29140);

	// ================= compute team account distribution ======================= //
	let team_balance_account_distribution = get_team_balance_distribution();
	let total_team_claims_amount: u128 = team_balance_account_distribution
		.into_iter()
		.map(|(_, amount, _, _, _)| amount)
		.sum();

	//println!("Remaining total team amount {:?}", 30000000000000000000000000 -
	// total_team_claims_amount - total_direct_team_amount - 5000 * UNIT);
	assert_eq!(total_team_claims_amount, 29856849309999998760386560); // 29856849 TNT
	assert_eq!(
		Perbill::from_rational(total_team_claims_amount, TOTAL_SUPPLY),
		Perbill::from_float(0.298568493)
	); // 29.8568493%

	// ================= compute intial endowment at genesis ========================= //
	// let total_endowmwnent: u128 =
	// 	get_initial_endowed_accounts().0.into_iter().map(|(_, amount)| amount).sum();
	// assert_eq!(total_endowmwnent - total_treasury_amount, 8900000000000000000000); // 8900 TNT

	let total_genesis_endowment = total_investor_amount
		+ total_direct_team_amount
		+ foundation_total_amount
		+ total_edgeware_claims_amount
		+ total_edgeware_snapshot_claims_amount
		+ total_leaderboard_claims_amount
		+ total_polkadot_claims_amount
		+ total_treasury_amount
		+ 5000 * UNIT
		+ total_team_claims_amount;
	//+ total_endowmwnent;

	assert_eq!(total_genesis_endowment, 100000000000000006345897383); // 100000000 TNT
	assert_eq!(Perbill::from_rational(total_genesis_endowment, TOTAL_SUPPLY), Perbill::one());
	// 100%
}
