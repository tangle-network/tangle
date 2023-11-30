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

use std::{collections::BTreeMap, marker::PhantomData};

use crate::{
	distributions::{combine_distributions, develop, mainnet, testnet},
	testnet_fixtures::{
		get_standalone_bootnodes, get_standalone_initial_authorities, get_testnet_root_key,
	},
};
use dkg_runtime_primitives::ResourceId;
use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_consensus_grandpa::AuthorityId as GrandpaId;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public, H160};
use sp_runtime::traits::{IdentifyAccount, Verify};
use tangle_mainnet_runtime::{
	AccountId, Balance, BalancesConfig, DKGConfig, DKGId, DKGProposalsConfig, EVMChainIdConfig,
	EVMConfig, ElectionsConfig, Eth2ClientConfig, ImOnlineConfig, MaxNominations, Perbill,
	RuntimeGenesisConfig, SessionConfig, Signature, StakerStatus, StakingConfig, SudoConfig,
	SystemConfig, UNIT, WASM_BINARY,
};
use webb_consensus_types::network_config::{Network, NetworkConfig};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Hermes (Evm, 5001)
const CHAIN_ID_HERMES: [u8; 6] = hex_literal::hex!("010000001389");
/// Athena (Evm, 5002)
const CHAIN_ID_ATHENA: [u8; 6] = hex_literal::hex!("01000000138a");
/// Demeter (Evm, 5003)
const CHAIN_ID_DEMETER: [u8; 6] = hex_literal::hex!("01000000138b");

const RESOURCE_ID_HERMES_ATHENA: ResourceId = ResourceId(hex_literal::hex!(
	"0000000000000000e69a847cd5bc0c9480ada0b339d7f0a8cac2b6670000138a"
));
const RESOURCE_ID_ATHENA_HERMES: ResourceId = ResourceId(hex_literal::hex!(
	"000000000000d30c8839c1145609e564b986f667b273ddcb8496010000001389"
));

/// The default value for keygen threshold
const DEFAULT_DKG_KEYGEN_THRESHOLD: u16 = 5;

/// The default value for signature threshold
const DEFAULT_DKG_SIGNATURE_THRESHOLD: u16 = 3;

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
) -> (AccountId, AccountId, AuraId, GrandpaId, ImOnlineId, DKGId) {
	(
		get_account_id_from_seed::<sr25519::Public>(controller),
		get_account_id_from_seed::<sr25519::Public>(stash),
		get_from_seed::<AuraId>(controller),
		get_from_seed::<GrandpaId>(controller),
		get_from_seed::<ImOnlineId>(stash),
		get_from_seed::<DKGId>(controller),
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
	dkg: DKGId,
) -> tangle_mainnet_runtime::opaque::SessionKeys {
	tangle_mainnet_runtime::opaque::SessionKeys { grandpa, aura, dkg, im_online }
}

pub fn local_testnet_config(chain_id: u64) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "tTNT".into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	properties.insert("ss58Format".into(), 42.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
					authority_keys_from_seed("Alice", "Alice//stash"),
					authority_keys_from_seed("Bob", "Bob//stash"),
					authority_keys_from_seed("Charlie", "Charlie//stash"),
					authority_keys_from_seed("Dave", "Dave//stash"),
					authority_keys_from_seed("Eve", "Eve//stash"),
				],
				vec![],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
				],
				// Initial Chain Ids
				vec![CHAIN_ID_HERMES, CHAIN_ID_ATHENA, CHAIN_ID_DEMETER],
				// Initial resource Ids
				vec![
					(RESOURCE_ID_HERMES_ATHENA, Default::default()),
					(RESOURCE_ID_ATHENA_HERMES, Default::default()),
				],
				// Initial proposers
				vec![
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
				],
				chain_id,
				DEFAULT_DKG_KEYGEN_THRESHOLD,
				DEFAULT_DKG_SIGNATURE_THRESHOLD,
				combine_distributions(vec![
					develop::get_evm_balance_distribution(),
					testnet::get_evm_balance_distribution(),
				]),
				testnet::get_substrate_balance_distribution(),
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
	))
}

pub fn tangle_mainnet_config(chain_id: u64) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "tangle wasm not available".to_string())?;
	let boot_nodes = get_standalone_bootnodes();
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "tTNT".into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	properties.insert("ss58Format".into(), 42.into());

	Ok(ChainSpec::from_genesis(
		"Tangle Mainnet",
		"tangle-mainnet",
		ChainType::Live,
		move || {
			mainnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
					authority_keys_from_seed("Alice", "Alice//stash"),
					authority_keys_from_seed("Bob", "Bob//stash"),
					authority_keys_from_seed("Charlie", "Charlie//stash"),
					authority_keys_from_seed("Dave", "Dave//stash"),
					authority_keys_from_seed("Eve", "Eve//stash"),
				],
				// Sudo account
				get_testnet_root_key(),
				// Pre-funded accounts
				vec![
					get_testnet_root_key(),
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				chain_id,
				combine_distributions(vec![
					mainnet::get_edgeware_genesis_balance_distribution(),
					mainnet::get_leaderboard_balance_distribution(),
				]),
				mainnet::get_substrate_balance_distribution(),
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
	))
}

/// Configure initial storage state for FRAME modules.
#[allow(clippy::too_many_arguments)]
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, AuraId, GrandpaId, ImOnlineId, DKGId)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	initial_chain_ids: Vec<[u8; 6]>,
	initial_r_ids: Vec<(ResourceId, Vec<u8>)>,
	initial_proposers: Vec<AccountId>,
	chain_id: u64,
	dkg_keygen_threshold: u16,
	dkg_signature_threshold: u16,
	genesis_evm_distribution: Vec<(H160, fp_evm::GenesisAccount)>,
	genesis_substrate_distribution: Vec<(AccountId, Balance)>,
	_enable_println: bool,
) -> RuntimeGenesisConfig {
	const ENDOWMENT: Balance = 10_000_000 * UNIT;
	const STASH: Balance = ENDOWMENT / 100;

	// stakers: all validators and nominators.
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	let num_endowed_accounts = endowed_accounts.len();
	RuntimeGenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			..Default::default()
		},
		sudo: SudoConfig { key: Some(root_key) },
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, ENDOWMENT))
				.chain(genesis_substrate_distribution.iter().cloned().map(|(k, v)| (k, v)))
				.collect(),
		},
		vesting: Default::default(),
		indices: Default::default(),
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.1.clone(),
						x.0.clone(),
						dkg_session_keys(x.3.clone(), x.2.clone(), x.4.clone(), x.5.clone()),
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
				.map(|member| (member, STASH))
				.collect(),
		},
		treasury: Default::default(),
		aura: Default::default(),
		grandpa: Default::default(),
		dkg: DKGConfig {
			authorities: initial_authorities.iter().map(|(.., x)| x.clone()).collect::<_>(),
			keygen_threshold: dkg_keygen_threshold,
			signature_threshold: dkg_signature_threshold,
			authority_ids: initial_authorities.iter().map(|(x, ..)| x.clone()).collect::<_>(),
		},
		dkg_proposals: DKGProposalsConfig { initial_chain_ids, initial_r_ids, initial_proposers },
		bridge_registry: Default::default(),
		im_online: ImOnlineConfig { keys: vec![] },
		eth_2_client: Eth2ClientConfig {
			// Vec<(TypedChainId, [u8; 32], ForkVersion, u64)>
			networks: vec![
				(webb_proposals::TypedChainId::Evm(1), NetworkConfig::new(&Network::Mainnet)),
				(webb_proposals::TypedChainId::Evm(5), NetworkConfig::new(&Network::Goerli)),
			],
			phantom: PhantomData,
		},
		nomination_pools: Default::default(),
		transaction_payment: Default::default(),
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
	}
}

/// Configure initial storage state for FRAME modules.
#[allow(clippy::too_many_arguments)]
fn mainnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, AuraId, GrandpaId, ImOnlineId, DKGId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	chain_id: u64,
	_genesis_evm_distribution: Vec<(H160, fp_evm::GenesisAccount)>,
	genesis_substrate_distribution: Vec<(AccountId, Balance)>,
	_enable_println: bool,
) -> RuntimeGenesisConfig {
	const ENDOWMENT: Balance = 100 * UNIT;
	const STASH: Balance = ENDOWMENT / 100;

	// stakers: all validators and nominators.
	let _rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.0.clone(), STASH, StakerStatus::Validator))
		.collect();

	RuntimeGenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			..Default::default()
		},
		sudo: SudoConfig { key: Some(root_key) },
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, ENDOWMENT))
				.chain(genesis_substrate_distribution.iter().cloned().map(|(k, v)| (k, v)))
				.collect(),
		},
		vesting: Default::default(),
		indices: Default::default(),
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.1.clone(),
						x.0.clone(),
						dkg_session_keys(x.3.clone(), x.2.clone(), x.4.clone(), x.5.clone()),
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
			networks: vec![(
				webb_proposals::TypedChainId::Evm(1),
				NetworkConfig::new(&Network::Mainnet),
			)],
			phantom: PhantomData,
		},
		// TODO: Remove these pallets and re-architect for mainnet
		dkg: DKGConfig {
			authorities: initial_authorities.iter().map(|(.., x)| x.clone()).collect::<_>(),
			keygen_threshold: 2,
			signature_threshold: 1,
			authority_ids: initial_authorities.iter().map(|(x, ..)| x.clone()).collect::<_>(),
		},
		dkg_proposals: Default::default(),
		bridge_registry: Default::default(),
	}
}
