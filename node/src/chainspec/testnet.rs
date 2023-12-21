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
	distributions::{combine_distributions, develop, testnet},
	testnet_fixtures::{get_bootnodes, get_initial_authorities, get_testnet_root_key},
};
use core::marker::PhantomData;
use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_consensus_grandpa::AuthorityId as GrandpaId;
use sc_service::ChainType;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public, H160};
use sp_runtime::traits::{IdentifyAccount, Verify};
use tangle_testnet_runtime::{
	AccountId, BabeConfig, Balance, BalancesConfig, EVMChainIdConfig, EVMConfig, ElectionsConfig,
	Eth2ClientConfig, ImOnlineConfig, MaxNominations, Perbill, RuntimeGenesisConfig, SessionConfig,
	Signature, StakerStatus, StakingConfig, SudoConfig, SystemConfig, UNIT, WASM_BINARY,
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

/// Generate an babe authority key.
pub fn authority_keys_from_seed(
	controller: &str,
	stash: &str,
) -> (AccountId, AccountId, BabeId, GrandpaId, ImOnlineId) {
	(
		get_account_id_from_seed::<sr25519::Public>(controller),
		get_account_id_from_seed::<sr25519::Public>(stash),
		get_from_seed::<BabeId>(controller),
		get_from_seed::<GrandpaId>(controller),
		get_from_seed::<ImOnlineId>(stash),
	)
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we
/// have just one key).
fn generate_sesion_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
) -> tangle_testnet_runtime::opaque::SessionKeys {
	tangle_testnet_runtime::opaque::SessionKeys { grandpa, babe, im_online }
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
				chain_id,
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

pub fn tangle_testnet_config(chain_id: u64) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "tangle wasm not available".to_string())?;
	let boot_nodes = get_bootnodes();
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "tTNT".into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	properties.insert("ss58Format".into(), 42.into());

	Ok(ChainSpec::from_genesis(
		"Tangle Standalone Testnet",
		"tangle-testnet",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				get_initial_authorities(),
				// initial nominators
				vec![],
				// Sudo account
				get_testnet_root_key(),
				// Pre-funded accounts
				vec![
					get_testnet_root_key(),
					hex!["4e85271af1330e5e9384bd3ac5bdc04c0f8ef5a8cc29c1a8ae483d674164745c"].into(),
					hex!["804808fb75d16340dc250871138a1a6f1dfa3cab9cc1fbd6f42960f1c39a950d"].into(),
					hex!["587c2ef00ec0a1b98af4c655763acd76ece690fccbb255f01663660bc274960d"].into(),
					hex!["cc195602a63bbdcf2ef4773c86fdbfefe042cb9aa8e3059d02e59a062d9c3138"].into(),
					hex!["a24f729f085de51eebaeaeca97d6d499761b8f6daeca9b99d754a06ef8bcec3f"].into(),
					hex!["368ea402dbd9c9888ae999d6a799cf36e08673ee53c001dfb4529c149fc2c13b"].into(),
					hex!["2c7f3cc085da9175414d1a9d40aa3aa161c8584a9ca62a938684dfbe90ae9d74"].into(),
					hex!["0a55e5245382700f35d16a5ea6d60a56c36c435bef7204353b8c36871f347857"].into(),
					hex!["e0948453e7acbc6ac937e124eb01580191e99f4262d588d4524994deb6134349"].into(),
					hex!["6c73e5ee9f8614e7c9f23fd8f7257d12e061e75fcbeb3b50ed70eb87ba91f500"].into(),
					hex!["541dc9dd9cd9b47ff19c77c3b14fab50ab0774e19abe438719cd09e4f4861166"].into(),
					hex!["607e948bad733780eda6c0bd9b084243276332823ca8481fc20cd01e1a2ef36f"].into(),
					hex!["b2c09cb1b78c3afd2b1ea4316dfb1be9065e070db948477248e4f3e0f1a2d850"].into(),
					hex!["fc156f082d789f94149f8b52b191672fbf202ef1b92b487c3cec9bca2d1fbe72"].into(),
					hex!["0e87759b6eeb6891743900cba17b8b5f31b2fa9c28536d9bcf76468d6e455b23"].into(),
					hex!["48cea44ac6dd245572272dc6d4d33908586fb80886bf3207344388eac279cc25"].into(),
					hex!["fa2c711c82661a761cf200421b9a5ef3257aa977a3a33acad0722d7d6993f03b"].into(),
					hex!["daf7985bfa22b5060a4eb212fbeddb7c47f7c29db5a356ed9500b34d2944eb3d"].into(),
					hex!["4ec0389ae623884a68234fd84d85af833633668aa382007e6515020e8cc29532"].into(),
					hex!["48bb70f924e7362ee55817a6628a79e522a08a31735b0129e47ac435215d6c4e"].into(),
					hex!["d6a033ee1790ef28fffe1b1ffec19b8921690632d073d955b9057e701eced352"].into(),
					hex!["14ecdcc058ee431166402eefb682c276cc16a5d1083409b28076fda4c4d5352f"].into(),
					hex!["400d597fe03f1031a9b4e1983b7c42eeed29ef3f9da6715667d06b367bdb897f"].into(),
					hex!["668cf048845804f31759decbec11cb41bf316b1901d2142a35ad3a8eb7420326"].into(),
				],
				chain_id,
				combine_distributions(vec![
					develop::get_evm_balance_distribution(),
					testnet::get_evm_balance_distribution(),
				]),
				testnet::get_substrate_balance_distribution(),
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
	initial_authorities: Vec<(AccountId, AccountId, BabeId, GrandpaId, ImOnlineId)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	chain_id: u64,
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
						generate_sesion_keys(x.3.clone(), x.2.clone(), x.4.clone()),
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
		babe: BabeConfig {
			epoch_config: Some(tangle_testnet_runtime::BABE_GENESIS_EPOCH_CONFIG),
			..Default::default()
		},
		grandpa: Default::default(),
		im_online: ImOnlineConfig { keys: vec![] },
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
