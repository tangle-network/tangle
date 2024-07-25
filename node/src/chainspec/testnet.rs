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
		mainnet::{self, DistributionResult},
	},
	testnet_fixtures::{get_bootnodes, get_initial_authorities, get_testnet_root_key},
};
use hex_literal::hex;
use pallet_airdrop_claims::MultiAddress;
use pallet_ibc::pallet::AssetConfig;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_consensus_grandpa::AuthorityId as GrandpaId;
use sc_service::ChainType;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{ecdsa, ed25519, sr25519, Pair, Public, H160, U256};
use sp_runtime::{
	traits::{AccountIdConversion, IdentifyAccount, Verify},
	BoundedVec,
};
use std::{collections::BTreeMap, str::FromStr};
use tangle_primitives::{
	types::{BlockNumber, Signature},
	TESTNET_LOCAL_SS58_PREFIX,
};
use tangle_testnet_runtime::{
	AccountId, BabeConfig, Balance, BalancesConfig, ClaimsConfig, CouncilConfig, EVMChainIdConfig,
	EVMConfig, IbcConfig, ImOnlineConfig, MaxVestingSchedules, Perbill, Precompiles, RoleKeyId,
	RuntimeGenesisConfig, SessionConfig, StakerStatus, StakingConfig, SudoConfig, SystemConfig,
	TreasuryPalletId, VestingConfig, UNIT, WASM_BINARY,
};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

pub const ENDOWMENT: Balance = 10_000_000 * UNIT;

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

/// Generate authority keys for benchmarking.
/// This is used for the `--chain dev` command.
pub fn authority_keys_for_dev(id: u8) -> (AccountId, BabeId, GrandpaId, ImOnlineId, RoleKeyId) {
	(
		AccountPublic::from(sr25519::Public::from_raw([id; 32])).into_account(),
		BabeId::from(sr25519::Public::from_raw([id; 32])),
		GrandpaId::from(ed25519::Public::from_raw([id; 32])),
		ImOnlineId::from(sr25519::Public::from_raw([id; 32])),
		RoleKeyId::from(ecdsa::Public::from_raw([id; 33])),
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

pub fn local_benchmarking_config(chain_id: u64) -> Result<ChainSpec, String> {
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
					authority_keys_for_dev(1),
					authority_keys_for_dev(2),
					authority_keys_for_dev(3),
				],
				// Pre-funded accounts
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Alice"), ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Charlie"), ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Dave"), ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Eve"), ENDOWMENT),
					(authority_keys_for_dev(1).0, ENDOWMENT),
					(authority_keys_for_dev(2).0, ENDOWMENT),
					(authority_keys_for_dev(3).0, ENDOWMENT),
				],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				chain_id,
				Default::default(),
				Default::default(),
				Default::default(),
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

pub fn local_testnet_config(chain_id: u64) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "tTNT".into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	properties.insert("ss58Format".into(), TESTNET_LOCAL_SS58_PREFIX.into());
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
				// Pre-funded accounts
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Alice"), ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Charlie"), ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Dave"), ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Eve"), ENDOWMENT),
				],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				chain_id,
				Default::default(),
				Default::default(),
				Default::default(),
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
				// Initial validators
				get_initial_authorities(),
				// Endowed accounts
				mainnet::get_initial_endowed_accounts().0,
				// Sudo account
				get_testnet_root_key(),
				// EVM chain ID
				chain_id,
				// Genesis airdrop distribution (pallet-claims)
				get_unique_distribution_results(vec![
					mainnet::get_edgeware_genesis_balance_distribution(),
					mainnet::get_leaderboard_balance_distribution(),
					mainnet::get_edgeware_snapshot_distribution(),
					mainnet::get_polkadot_validator_distribution(),
				]),
				// Genesis investor / team distribution (pallet-balances + pallet-vesting)
				combine_distributions(vec![
					mainnet::get_team_balance_distribution(),
					mainnet::get_team_direct_vesting_distribution(),
					mainnet::get_investor_balance_distribution(),
					mainnet::get_foundation_balance_distribution(),
				]),
				// endowed evm accounts
				vec![],
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
	endowed_accounts: Vec<(AccountId, Balance)>,
	root_key: AccountId,
	chain_id: u64,
	genesis_airdrop: DistributionResult,
	genesis_non_airdrop: Vec<(MultiAddress, u128, u64, u64, u128)>,
	genesis_evm_distribution: Vec<(H160, fp_evm::GenesisAccount)>,
) -> RuntimeGenesisConfig {
	// stakers: all validators and nominators.
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.0.clone(), 100 * UNIT, StakerStatus::Validator))
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

	// As precompiles are implemented inside the Runtime, they don't have a bytecode, and
	// their account code is empty by default. However in Solidity calling a function of a
	// contract often automatically adds a check that the contract bytecode is non-empty.
	// For that reason a dummy code (0x60006000fd) can be inserted at the precompile address
	// to pass that check.
	let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];

	RuntimeGenesisConfig {
		system: SystemConfig { ..Default::default() },
		sudo: SudoConfig { key: Some(root_key) },
		assets: Default::default(),
		balances: BalancesConfig { balances: endowed_accounts.to_vec() },
		vesting: VestingConfig {
			vesting: genesis_non_airdrop
				.iter()
				.map(|(address, _value, begin, end, liquid)| {
					(address.clone().to_account_id_32(), *begin, *end, *liquid)
				})
				.collect(),
		},
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
		council: CouncilConfig {
			members: vec![
				hex!["483b466832e094f01b1779a7ed07025df319c492dac5160aca89a3be117a7b6d"].into(),
				hex!["86d08e7bbe77bc74e3d88ee22edc53368bc13d619e05b66fe6c4b8e2d5c7015a"].into(),
				hex!["e421301e5aa5dddee51f0d8c73e794df16673e53157c5ea657be742e35b1793f"].into(),
				hex!["4ce3a4da3a7c1ce65f7edeff864dc3dd42e8f47eecc2726d99a0a80124698217"].into(),
				hex!["dcd9b70a0409b7626cba1a4016d8da19f4df5ce9fc5e8d16b789e71bb1161d73"].into(),
			],
			..Default::default()
		},
		elections: Default::default(),
		treasury: Default::default(),
		babe: BabeConfig {
			epoch_config: Some(tangle_runtime::BABE_GENESIS_EPOCH_CONFIG),
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

				Precompiles::used_addresses().for_each(|address| {
					map.insert(
						address,
						fp_evm::GenesisAccount {
							nonce: Default::default(),
							balance: Default::default(),
							storage: Default::default(),
							code: revert_bytecode.clone(),
						},
					);
				});

				map.insert(
					// H160 address of Alice dev account
					// Derived from SS58 (42 prefix) address
					// SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
					// hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
					// Using the full hex key, truncating to the first 20 bytes (the first 40 hex
					// chars)
					H160::from_str("8efcaf2c4ebbf88bf07f3bb44a2869c4c675ad7a")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);

				map.insert(
					// Relayer 1
					H160::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);

				map.insert(
					// Relayer 2
					H160::from_str("5D4ff00Bf77F97E93131a448379f7808D7373026")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);

				map.insert(
					// Relayer 3
					H160::from_str("b65EA4E162188d199b14da8bc747F24042c36E2C")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map
			},
			..Default::default()
		},
		ethereum: Default::default(),
		dynamic_fee: Default::default(),
		base_fee: Default::default(),
		claims: ClaimsConfig {
			claims: genesis_airdrop.claims,
			vesting: vesting_claims,
			expiry: Some((
				3_265_000u64,
				MultiAddress::Native(TreasuryPalletId::get().into_account_truncating()),
			)),
		},
		ibc: IbcConfig { assets: vec![AssetConfig { id: 1, denom: b"tTNT".to_vec() }] },
	}
}
