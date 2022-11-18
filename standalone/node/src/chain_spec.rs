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

use crate::testnet_fixtures::{
	get_arana_bootnodes, get_arana_initial_authorities, get_testnet_root_key,
};
use arkworks_setups::{common::setup_params, Curve};
use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_network_common::config::MultiaddrWithPeerId;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use tangle_runtime::{
	AccountId, AssetRegistryConfig, Balance, BalancesConfig, ClaimsConfig, DKGConfig, DKGId,
	DKGProposalsConfig, ElectionsConfig, GenesisConfig, HasherBn254Config, ImOnlineConfig,
	MaxNominations, MerkleTreeBn254Config, MixerBn254Config, MixerVerifierBn254Config, Perbill,
	SessionConfig, Signature, StakerStatus, StakingConfig, SudoConfig, SystemConfig,
	VAnchorBn254Config, VAnchorVerifierConfig, UNIT, WASM_BINARY,
};
pub type ResourceId = [u8; 32];

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
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
) -> tangle_runtime::opaque::SessionKeys {
	tangle_runtime::opaque::SessionKeys { grandpa, aura, dkg, im_online }
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
					authority_keys_from_seed("Alice", "Alice//stash"),
					authority_keys_from_seed("Bob", "Bob//stash"),
					authority_keys_from_seed("Charlie", "Charlie//stash"),
				],
				vec![],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				// Initial Chain Ids
				vec![
					hex_literal::hex!("010000001389"), // Hermis (Evm, 5001)
					hex_literal::hex!("01000000138a"), // Athena (Evm, 5002)
					hex_literal::hex!("01000000138b"), // Demeter (Evm, 5003)
				],
				// Initial resource Ids
				vec![
					// Resource ID for Chain Hermis => Athena
					(
						hex_literal::hex!(
							"000000000000e69a847cd5bc0c9480ada0b339d7f0a8cac2b66701000000138a"
						),
						Default::default(),
					),
					// Resource ID for Chain Athena => Hermis
					(
						hex_literal::hex!(
							"000000000000d30c8839c1145609e564b986f667b273ddcb8496100000001389"
						),
						Default::default(),
					),
				],
				// Initial proposers
				vec![
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
				],
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
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

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
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				// Initial Chain Ids
				vec![
					hex_literal::hex!("010000001389"), // Hermis (Evm, 5001)
					hex_literal::hex!("01000000138a"), // Athena (Evm, 5002)
				],
				// Initial resource Ids
				vec![
					// Resource ID for Chain Hermis => Athena
					(
						hex_literal::hex!(
							"0000000000000000e69a847cd5bc0c9480ada0b339d7f0a8cac2b6670000138a"
						),
						Default::default(),
					),
					// Resource ID for Chain Athena => Hermis
					(
						hex_literal::hex!(
							"0000000000000000d30c8839c1145609e564b986f667b273ddcb849600001389"
						),
						Default::default(),
					),
				],
				// Initial proposers
				vec![
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
				],
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
		None,
		// Extensions
		None,
	))
}

pub fn arana_live_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Arana wasm not available".to_string())?;
	let boot_nodes = get_arana_bootnodes();

	Ok(ChainSpec::from_genesis(
		"Arana",
		"arana",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				get_arana_initial_authorities(),
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
				],
				vec![],
				vec![],
				get_arana_initial_authorities().iter().map(|a| a.0.clone()).collect(),
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
		None,
		// Extensions
		None,
	))
}

pub fn arana_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Arana wasm not available".to_string())?;
	let boot_nodes = get_arana_bootnodes();

	Ok(ChainSpec::from_genesis(
		"Arana Alpha",
		"arana-alpha",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				get_arana_initial_authorities(),
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
				],
				vec![],
				vec![],
				get_arana_initial_authorities().iter().map(|a| a.0.clone()).collect(),
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
		None,
		// Extensions
		None,
	))
}

// same as arana_testnet but without bootnodes so that we can spinup same network locally
pub fn arana_local_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Arana wasm not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		"Arana Local",
		"arana-local",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				get_arana_initial_authorities(),
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
				],
				vec![],
				vec![],
				get_arana_initial_authorities().iter().map(|a| a.0.clone()).collect(),
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
		None,
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
	_enable_println: bool,
) -> GenesisConfig {
	let curve_bn254 = Curve::Bn254;

	log::info!("Bn254 x5 w3 params");
	let bn254_x5_3_params = setup_params::<ark_bn254::Fr>(curve_bn254, 5, 3);

	log::info!("Verifier params for mixer");
	let mixer_verifier_bn254_params = {
		let vk_bytes = include_bytes!("../../../verifying_keys/mixer/bn254/verifying_key.bin");
		vk_bytes.to_vec()
	};

	log::info!("Verifier params for vanchor");
	let vanchor_verifier_bn254_params = {
		let vk_bytes =
			include_bytes!("../../../verifying_keys/vanchor/bn254/x5/2-2-2/verifying_key.bin");
		vk_bytes.to_vec()
	};
	// TODO: Add proper verifying keys for 16-2
	let vanchor_verifier_16x2_bn254_params = {
		let vk_bytes =
			include_bytes!("../../../verifying_keys/vanchor/bn254/x5/2-2-2/verifying_key.bin");
		vk_bytes.to_vec()
	};

	const ENDOWMENT: Balance = 10_000_000 * UNIT;
	const STASH: Balance = ENDOWMENT / 1000;

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
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	let num_endowed_accounts = endowed_accounts.len();
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		claims: ClaimsConfig { claims: vec![], vesting: vec![], expiry: None },
		sudo: SudoConfig { key: Some(root_key) },
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, ENDOWMENT)).collect(),
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
			keygen_threshold: 5,
			signature_threshold: 3,
			authority_ids: initial_authorities.iter().map(|(x, ..)| x.clone()).collect::<_>(),
		},
		dkg_proposals: DKGProposalsConfig { initial_chain_ids, initial_r_ids, initial_proposers },
		asset_registry: AssetRegistryConfig {
			asset_names: vec![],
			native_asset_name: b"WEBB".to_vec(),
			native_existential_deposit: tangle_runtime::EXISTENTIAL_DEPOSIT,
		},
		hasher_bn_254: HasherBn254Config {
			parameters: Some(bn254_x5_3_params.to_bytes()),
			phantom: Default::default(),
		},
		mixer_verifier_bn_254: MixerVerifierBn254Config {
			parameters: Some(mixer_verifier_bn254_params),
			phantom: Default::default(),
		},
		merkle_tree_bn_254: MerkleTreeBn254Config {
			phantom: Default::default(),
			default_hashes: None,
		},
		mixer_bn_254: MixerBn254Config {
			mixers: vec![(0, 10 * UNIT), (0, 100 * UNIT), (0, 1000 * UNIT)],
		},
		v_anchor_verifier: VAnchorVerifierConfig {
			parameters: Some(vec![
				(2, 2, vanchor_verifier_bn254_params),
				(2, 16, vanchor_verifier_16x2_bn254_params),
			]),
			phantom: Default::default(),
		},
		v_anchor_bn_254: VAnchorBn254Config {
			max_deposit_amount: 1_000_000 * UNIT,
			min_withdraw_amount: 0,
			vanchors: vec![(0, 2)],
			phantom: Default::default(),
		},
		im_online: ImOnlineConfig { keys: vec![] },
	}
}
