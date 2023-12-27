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
use self::mainnet::DistributionResult;

use pallet_airdrop_claims::{MultiAddress, StatementKind};

use tangle_primitives::{Balance, BlockNumber};
use tangle_runtime::EXISTENTIAL_DEPOSIT;

pub mod develop;
pub mod mainnet;
pub mod testnet;

pub fn combine_distributions<T>(distributions: Vec<Vec<T>>) -> Vec<T> {
	let mut combined = Vec::new();
	for distribution in distributions {
		for elt in distribution {
			combined.push(elt);
		}
	}
	combined
}

pub fn get_unique_distribution_results(
	distribution_results: Vec<DistributionResult>,
) -> DistributionResult {
	assert!(!distribution_results.is_empty());
	let vesting_lengths: Vec<BlockNumber> =
		distribution_results.iter().map(|result| result.vesting_length).collect();
	let vesting_cliffs: Vec<BlockNumber> =
		distribution_results.iter().map(|result| result.vesting_cliff).collect();
	assert!(vesting_lengths.windows(2).all(|w| w[0] == w[1]), "Vesting lengths are not equal.");
	assert!(vesting_cliffs.windows(2).all(|w| w[0] == w[1]), "Vesting cliffs are not equal.");

	let combined_claims: Vec<(MultiAddress, Balance, Option<StatementKind>)> =
		distribution_results.iter().flat_map(|result| result.claims.clone()).collect();
	let combined_vesting: Vec<(MultiAddress, Vec<(Balance, Balance, BlockNumber)>)> =
		distribution_results.iter().flat_map(|result| result.vesting.clone()).collect();

	let mut unique_claims = std::collections::HashMap::new();
	for (address, balance, statement) in combined_claims {
		unique_claims
			.entry(address)
			.and_modify(|e: &mut (Balance, Option<StatementKind>)| e.0 += balance)
			.or_insert((balance, statement));
	}
	let unique_claims: Vec<(MultiAddress, Balance, Option<StatementKind>)> = unique_claims
		.into_iter()
		.filter_map(|(address, (balance, statement))| {
			// Skip any claims that are below the existential deposit.
			if balance < EXISTENTIAL_DEPOSIT {
				None
			} else {
				Some((address, balance, statement))
			}
		})
		.collect();

	let mut unique_vesting = std::collections::HashMap::new();
	for (address, schedules) in combined_vesting {
		unique_vesting
			.entry(address)
			.and_modify(|e: &mut Vec<(Balance, Balance, BlockNumber)>| e.extend(schedules.clone()))
			.or_insert(schedules);
	}
	let unique_vesting: Vec<(MultiAddress, Vec<(Balance, Balance, BlockNumber)>)> = unique_vesting
		.into_iter()
		.map(|(address, schedules)| (address, schedules))
		.collect();

	DistributionResult {
		claims: unique_claims,
		vesting: unique_vesting,
		vesting_length: vesting_lengths[0],
		vesting_cliff: vesting_cliffs[0],
	}
}
