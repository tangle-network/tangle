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

use crate::testnet_fixtures::{get_bootnodes, get_initial_authorities, get_testnet_root_key};

use hex_literal::hex;
use pallet_airdrop_claims::{MultiAddress, StatementKind};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_consensus_grandpa::AuthorityId as GrandpaId;
use sc_service::ChainType;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public, H160};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	BoundedVec,
};
use std::collections::BTreeMap;
use tangle_crypto_primitives::crypto::AuthorityId as RoleKeyId;
use tangle_primitives::types::BlockNumber;
use tangle_testnet_runtime::{
	AccountId, BabeConfig, Balance, BalancesConfig, ClaimsConfig, EVMChainIdConfig, EVMConfig,
	ElectionsConfig, ImOnlineConfig, MaxVestingSchedules, Perbill, RuntimeGenesisConfig,
	SessionConfig, Signature, StakerStatus, StakingConfig, SudoConfig, SystemConfig, UNIT,
	WASM_BINARY,
};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

pub const ENDOWMENT: Balance = 10_000_000 * UNIT;
pub const STASH: Balance = ENDOWMENT / 10;

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

/// Generate an babe authority key.
pub fn authority_keys_from_seed(
	stash: &str,
) -> (AccountId, BabeId, GrandpaId, ImOnlineId, RoleKeyId) {
	(
		get_account_id_from_seed::<sr25519::Public>(stash),
		get_from_seed::<BabeId>(stash),
		get_from_seed::<GrandpaId>(stash),
		get_from_seed::<ImOnlineId>(stash),
		get_from_seed::<RoleKeyId>(stash),
	)
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we
/// have just one key).
fn generate_session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
	role: RoleKeyId,
) -> tangle_testnet_runtime::opaque::SessionKeys {
	tangle_testnet_runtime::opaque::SessionKeys { babe, grandpa, im_online, role }
}

pub fn local_testnet_config(chain_id: u64) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "tTNT".into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	properties.insert("ss58Format".into(), 42.into());
	#[allow(deprecated)]
	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				// Initial PoA authorities
				vec![
					authority_keys_from_seed("Alice"),
					authority_keys_from_seed("Bob"),
					authority_keys_from_seed("Charlie"),
					authority_keys_from_seed("Dave"),
					authority_keys_from_seed("Eve"),
				],
				vec![],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Alice"), ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Charlie"), ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Dave"), ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Eve"), ENDOWMENT),
				],
				chain_id,
				Default::default(),
				Default::default(),
				Default::default(),
				true,
			)
		},
		// Bootnodes
		vec![],
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
		wasm_binary,
	))
}

pub fn tangle_testnet_config(chain_id: u64) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "tangle wasm not available".to_string())?;
	let boot_nodes = get_bootnodes();
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "tTNT".into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	properties.insert("ss58Format".into(), 42.into());
	#[allow(deprecated)]
	Ok(ChainSpec::from_genesis(
		"Tangle Testnet",
		"tangle-testnet",
		ChainType::Live,
		move || {
			testnet_genesis(
				// Initial PoA authorities
				get_initial_authorities(),
				// initial nominators
				vec![],
				// Sudo account
				get_testnet_root_key(),
				// Pre-funded accounts
				vec![
					(get_testnet_root_key(), ENDOWMENT * 5), // 50 Million
					(
						hex!["4e85271af1330e5e9384bd3ac5bdc04c0f8ef5a8cc29c1a8ae483d674164745c"]
							.into(),
						ENDOWMENT,
					),
					(
						hex!["587c2ef00ec0a1b98af4c655763acd76ece690fccbb255f01663660bc274960d"]
							.into(),
						ENDOWMENT,
					),
					(
						hex!["a24f729f085de51eebaeaeca97d6d499761b8f6daeca9b99d754a06ef8bcec3f"]
							.into(),
						ENDOWMENT,
					),
					(
						hex!["0a55e5245382700f35d16a5ea6d60a56c36c435bef7204353b8c36871f347857"]
							.into(),
						ENDOWMENT,
					),
					(
						hex!["e0948453e7acbc6ac937e124eb01580191e99f4262d588d4524994deb6134349"]
							.into(),
						ENDOWMENT,
					),
				],
				chain_id,
				Default::default(),
				Default::default(),
				Default::default(),
				true,
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
		wasm_binary,
	))
}

/// Configure initial storage state for FRAME modules.
#[allow(clippy::too_many_arguments)]
fn testnet_genesis(
	initial_authorities: Vec<(AccountId, BabeId, GrandpaId, ImOnlineId, RoleKeyId)>,
	_initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, Balance)>,
	chain_id: u64,
	genesis_evm_distribution: Vec<(H160, fp_evm::GenesisAccount)>,
	genesis_substrate_distribution: Vec<(AccountId, Balance)>,
	claims: Vec<(MultiAddress, Balance)>,
	_enable_println: bool,
) -> RuntimeGenesisConfig {
	// stakers: all validators and nominators.
	let _rng = rand::thread_rng();
	// stakers: all validators and nominators.
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.0.clone(), STASH, StakerStatus::Validator))
		.collect();

	let num_endowed_accounts = endowed_accounts.len();

	let claims_list: Vec<(MultiAddress, Balance, Option<StatementKind>)> = endowed_accounts
		.iter()
		.map(|x| (MultiAddress::Native(x.0.clone()), ENDOWMENT, Some(StatementKind::Regular)))
		.chain(claims.clone().into_iter().map(|(a, b)| (a, b, Some(StatementKind::Regular))))
		.collect();

	let vesting_claims: Vec<(
		MultiAddress,
		BoundedVec<(Balance, Balance, BlockNumber), MaxVestingSchedules>,
	)> = endowed_accounts
		.iter()
		.map(|x| {
			let mut bounded_vec = BoundedVec::new();
			bounded_vec.try_push((ENDOWMENT, ENDOWMENT, 0)).unwrap();
			(MultiAddress::Native(x.0.clone()), bounded_vec)
		})
		.chain(claims.into_iter().map(|(a, b)| {
			let mut bounded_vec = BoundedVec::new();
			bounded_vec.try_push((b, b, 0)).unwrap();
			(a, bounded_vec)
		}))
		.collect();

	RuntimeGenesisConfig {
		system: SystemConfig { ..Default::default() },
		sudo: SudoConfig { key: Some(root_key) },
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts
				.iter()
				.cloned()
				.chain(genesis_substrate_distribution)
				.collect(),
		},
		vesting: Default::default(),
		indices: Default::default(),
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						generate_session_keys(x.1.clone(), x.2.clone(), x.3.clone(), x.4.clone()),
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
		elections: ElectionsConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.map(|member| (member.0, STASH))
				.collect(),
		},
		treasury: Default::default(),
		babe: BabeConfig {
			epoch_config: Some(tangle_testnet_runtime::BABE_GENESIS_EPOCH_CONFIG),
			..Default::default()
		},
		grandpa: Default::default(),
		im_online: ImOnlineConfig { keys: vec![] },
		nomination_pools: Default::default(),
		transaction_payment: Default::default(),
		tx_pause: Default::default(),

		// EVM compatibility
		evm_chain_id: EVMChainIdConfig { chain_id, ..Default::default() },
		evm: EVMConfig {
			accounts: {
				let mut map = BTreeMap::new();
				for (address, account) in genesis_evm_distribution {
					map.insert(address, account);
				}
				map
			},
			..Default::default()
		},
		ethereum: Default::default(),
		dynamic_fee: Default::default(),
		base_fee: Default::default(),
		claims: ClaimsConfig {
			claims: claims_list,
			vesting: vesting_claims,
			expiry: None, // no expiry on testnet
		},
	}
}
