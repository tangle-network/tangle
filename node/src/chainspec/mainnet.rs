// Copyright 2022 Webb Technologies Inc.
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

use std::collections::BTreeMap;

use crate::{
	distributions::{
		combine_distributions, develop, get_unique_distribution_results,
		mainnet::{self, DistributionResult, ONE_TOKEN},
		testnet,
	},
	mainnet_fixtures::{get_root_key, get_standalone_bootnodes},
};
use core::marker::PhantomData;
use pallet_airdrop_claims::MultiAddress;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_consensus_grandpa::AuthorityId as GrandpaId;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public, H160};
use sp_runtime::traits::{IdentifyAccount, Verify};
use tangle_mainnet_runtime::{
	AccountId, Balance, BalancesConfig, ClaimsConfig, EVMChainIdConfig, EVMConfig, ElectionsConfig,
	Eth2ClientConfig, ImOnlineConfig, MaxNominations, Perbill, RuntimeGenesisConfig, SessionConfig,
	Signature, StakerStatus, StakingConfig, SudoConfig, SystemConfig, VestingConfig, UNIT,
	WASM_BINARY,
};
use webb_consensus_types::network_config::{Network, NetworkConfig};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{seed}"), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(
	controller: &str,
	stash: &str,
) -> (AccountId, AccountId, AuraId, GrandpaId, ImOnlineId) {
	(
		get_account_id_from_seed::<sr25519::Public>(controller),
		get_account_id_from_seed::<sr25519::Public>(stash),
		get_from_seed::<AuraId>(controller),
		get_from_seed::<GrandpaId>(controller),
		get_from_seed::<ImOnlineId>(stash),
	)
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we
/// have just one key).
fn dkg_session_keys(
	grandpa: GrandpaId,
	aura: AuraId,
	im_online: ImOnlineId,
) -> tangle_mainnet_runtime::opaque::SessionKeys {
	tangle_mainnet_runtime::opaque::SessionKeys { grandpa, aura, im_online }
}

pub fn tangle_mainnet_config(evm_chain_id: u64) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "tangle wasm not available".to_string())?;
	let boot_nodes = get_standalone_bootnodes();
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "TNT".into());
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
				vec![],
				// Sudo account
				get_root_key(),
				// EVM chain ID
				evm_chain_id,
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
		boot_nodes,
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
	initial_authorities: Vec<(AccountId, AccountId, AuraId, GrandpaId, ImOnlineId)>,
	root_key: AccountId,
	chain_id: u64,
	genesis_airdrop: DistributionResult,
	genesis_non_airdrop: Vec<(MultiAddress, u128, u64, u64, u128)>,
) -> RuntimeGenesisConfig {
	// stakers: all validators and nominators.
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.0.clone(), 1 * ONE_TOKEN, StakerStatus::Validator))
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
				// .filter(|(x, y, _)| {
				// 	match x {
				// 		MultiAddress::EVM(_) => false,
				// 		MultiAddress::Native(_) => true,
				// 	}
				// })
				.map(|(x, y, _, _, _)| (x.clone().to_account_id_32(), y.clone()))
				.collect(),
		},
		vesting: VestingConfig {
			vesting: genesis_non_airdrop
				.iter()
				.map(|(x, _, a, b, c)| {
					(x.clone().to_account_id_32(), a.clone(), b.clone(), c.clone())
				})
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
						dkg_session_keys(x.3.clone(), x.2.clone(), x.4.clone()),
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
		aura: Default::default(),
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
		// ETH2 light client
		eth_2_client: Eth2ClientConfig {
			networks: vec![
				(webb_proposals::TypedChainId::Evm(1), NetworkConfig::new(&Network::Mainnet)),
				(webb_proposals::TypedChainId::Evm(5), NetworkConfig::new(&Network::Goerli)),
			],
			phantom: PhantomData,
		},
		claims: Default::default(),
	}
}
