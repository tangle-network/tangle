// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(clippy::type_complexity)]

use crate::{
	distributions::{
		combine_distributions, get_unique_distribution_results,
		mainnet::{self, DistributionResult, ONE_TOKEN},
	},
	mainnet_fixtures::{get_bootnodes, get_initial_authorities, get_root_key},
};
use core::marker::PhantomData;
use pallet_airdrop_claims::MultiAddress;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_consensus_grandpa::AuthorityId as GrandpaId;
use sc_service::ChainType;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_runtime::{traits::AccountIdConversion, BoundedVec};
use tangle_primitives::BlockNumber;
use tangle_runtime::{
	AccountId, Balance, BalancesConfig, ClaimsConfig, EVMChainIdConfig, Eth2ClientConfig,
	ImOnlineConfig, MaxVestingSchedules, Perbill, RuntimeGenesisConfig, SessionConfig,
	StakerStatus, StakingConfig, SudoConfig, SystemConfig, TreasuryPalletId, VestingConfig,
	WASM_BINARY,
};
use webb_consensus_types::network_config::{Network, NetworkConfig};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we
/// have just one key).
fn generate_session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
) -> tangle_runtime::opaque::SessionKeys {
	tangle_runtime::opaque::SessionKeys { grandpa, babe, im_online }
}

pub fn tangle_mainnet_config(chain_id: u64) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "tangle wasm not available".to_string())?;
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenDecimals".into(), 18u32.into());
	properties.insert("ss58Format".into(), 4006.into());

	Ok(ChainSpec::from_genesis(
		"Tangle Mainnet",
		"tangle-mainnet",
		ChainType::Live,
		move || {
			mainnet_genesis(
				// Wasm binary
				wasm_binary,
				// Initial validators
				get_initial_authorities(),
				// Sudo account
				get_root_key(),
				// EVM chain ID
				chain_id,
				// Genesis airdrop distribution (pallet-claims)
				get_unique_distribution_results(vec![
					mainnet::get_edgeware_genesis_balance_distribution(),
					mainnet::get_leaderboard_balance_distribution(),
					mainnet::get_substrate_balance_distribution(),
				]),
				// Genesis investor / team distribution (pallet-balances + pallet-vesting)
				combine_distributions(vec![
					mainnet::get_team_balance_distribution(),
					mainnet::get_investor_balance_distribution(),
				]),
			)
		},
		// Bootnodes
		get_bootnodes(),
		// Telemetry
		None,
		// Protocol ID
		None,
		// Fork id
		None,
		// Properties
		Some(properties),
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
#[allow(clippy::too_many_arguments)]
fn mainnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, BabeId, GrandpaId, ImOnlineId)>,
	root_key: AccountId,
	chain_id: u64,
	genesis_airdrop: DistributionResult,
	genesis_non_airdrop: Vec<(MultiAddress, u128, u64, u64, u128)>,
) -> RuntimeGenesisConfig {
	// stakers: all validators and nominators.
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.0.clone(), ONE_TOKEN, StakerStatus::Validator))
		.collect();

	let vesting_claims: Vec<(
		MultiAddress,
		BoundedVec<(Balance, Balance, BlockNumber), MaxVestingSchedules>,
	)> = genesis_airdrop
		.vesting
		.into_iter()
		.map(|(x, y)| {
			let mut bounded_vec = BoundedVec::new();
			for (a, b, c) in y {
				bounded_vec.try_push((a, b, c)).unwrap();
			}
			(x, bounded_vec)
		})
		.collect();
	RuntimeGenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			..Default::default()
		},
		sudo: SudoConfig { key: Some(root_key) },
		balances: BalancesConfig {
			balances: genesis_non_airdrop
				.iter()
				.map(|(x, y, _, _, _)| (x.clone().to_account_id_32(), *y))
				.collect(),
		},
		vesting: VestingConfig {
			vesting: genesis_non_airdrop
				.iter()
				.map(|(x, _, a, b, c)| (x.clone().to_account_id_32(), *a, *b, *c))
				.collect(),
		},
		indices: Default::default(),
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.1.clone(),
						x.0.clone(),
						generate_session_keys(x.3.clone(), x.2.clone(), x.4.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32 - 1,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
			..Default::default()
		},
		democracy: Default::default(),
		council: Default::default(),
		elections: Default::default(),
		treasury: Default::default(),
		babe: Default::default(),
		grandpa: Default::default(),
		im_online: ImOnlineConfig { keys: vec![] },
		nomination_pools: Default::default(),
		transaction_payment: Default::default(),
		// EVM compatibility
		evm_chain_id: EVMChainIdConfig { chain_id, ..Default::default() },
		evm: Default::default(),
		ethereum: Default::default(),
		dynamic_fee: Default::default(),
		base_fee: Default::default(),
		eth_2_client: Eth2ClientConfig {
			networks: vec![(
				webb_proposals::TypedChainId::Evm(1),
				NetworkConfig::new(&Network::Mainnet),
			)],
			phantom: PhantomData,
		},
		claims: ClaimsConfig {
			claims: genesis_airdrop.claims,
			vesting: vesting_claims,
			expiry: Some((
				5_256_000u64,
				MultiAddress::Native(TreasuryPalletId::get().into_account_truncating()),
			)),
		},
	}
}
