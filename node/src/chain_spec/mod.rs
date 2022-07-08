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

use arkworks_setups::{common::setup_params, Curve};
use cumulus_primitives_core::ParaId;
use egg_rococo_runtime::{
	AccountId, AnchorBn254Config, AnchorVerifierBn254Config, AssetRegistryConfig, AuraId, DKGId,
	HasherBn254Config, MerkleTreeBn254Config, MixerBn254Config, MixerVerifierBn254Config,
	Signature, EXISTENTIAL_DEPOSIT, MILLIUNIT, UNIT,
};
use hex_literal::hex;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

pub mod rococo;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<egg_rococo_runtime::GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain
/// in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_from_seed::<AuraId>(seed)
}

/// Generate DKG keys from seed.
///
/// This function's return type must always match the session keys of the chain
/// in tuple format.
pub fn get_dkg_keys_from_seed(seed: &str) -> DKGId {
	get_from_seed::<DKGId>(seed)
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we
/// have just one key).
pub fn dkg_session_keys(keys: AuraId, dkg_keys: DKGId) -> egg_rococo_runtime::SessionKeys {
	egg_rococo_runtime::SessionKeys { aura: keys, dkg: dkg_keys }
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn development_config(id: ParaId) -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "tEGG".into());
	properties.insert("tokenDecimals".into(), 12u32.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Local,
		move || {
			testnet_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
						get_dkg_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
						get_dkg_keys_from_seed("Bob"),
					),
				],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				id,
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("egg-dev"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
}

pub fn local_testnet_config(id: ParaId) -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "tEGG".into());
	properties.insert("tokenDecimals".into(), 12u32.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				// root
				hex!["5ebd99141e19db88cd2c4b778d3cc43e3678d40168aaea56f33d2ea31f67463f"].into(),
				vec![
					(
						//1//account
						hex!["28714d0740d6b321ad67b8e1a4edd0b53376f735bd10e4904a2c49167bcb7841"]
							.into(),
						//1//aura
						hex!["28714d0740d6b321ad67b8e1a4edd0b53376f735bd10e4904a2c49167bcb7841"]
							.unchecked_into(),
						//1//dkg (--scheme Ecdsa)
						hex!["03568538f7152c4ee734ad74983e1d86e2329ec100bb06b1c2af0bada2f72ffa28"]
							.unchecked_into(),
					),
					(
						//1//account
						hex!["f2ca12f1d3e0cba599b9f17f5675a1dd2d5d781d7a97d241312acc39e0b6f112"]
							.into(),
						//1//aura
						hex!["f2ca12f1d3e0cba599b9f17f5675a1dd2d5d781d7a97d241312acc39e0b6f112"]
							.unchecked_into(),
						//1//dkg (--scheme Ecdsa)
						hex!["03e620a6e19d236bdfe40ef95b9601483629d0e0097e9a7cfb97e7c99e63da609d"]
							.unchecked_into(),
					),
				],
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
				id,
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("egg-template-local"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
}

fn testnet_genesis(
	root_key: AccountId,
	invulnerables: Vec<(AccountId, AuraId, DKGId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> egg_rococo_runtime::GenesisConfig {
	let curve_bn254 = Curve::Bn254;

	log::info!("Bn254 x5 w3 params");
	let bn254_x5_3_params = setup_params::<ark_bn254::Fr>(curve_bn254, 5, 3);

	log::info!("Verifier params for mixer");
	let mixer_verifier_bn254_params = {
		let vk_bytes = include_bytes!("../../../verifying_keys/mixer/bn254/verifying_key.bin");
		vk_bytes.to_vec()
	};

	log::info!("Verifier params for anchor");
	let anchor_verifier_bn254_params = {
		let vk_bytes = include_bytes!("../../../verifying_keys/anchor/bn254/2/verifying_key.bin");
		vk_bytes.to_vec()
	};

	egg_rococo_runtime::GenesisConfig {
		system: egg_rococo_runtime::SystemConfig {
			code: egg_rococo_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		sudo: egg_rococo_runtime::SudoConfig { key: Some(root_key) },
		balances: egg_rococo_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, MILLIUNIT * 4_096_000))
				.collect(),
		},
		indices: Default::default(),
		parachain_info: egg_rococo_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: egg_rococo_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: egg_rococo_runtime::SessionConfig {
			keys: invulnerables
				.iter()
				.cloned()
				.map(|(acc, aura, dkg)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						dkg_session_keys(aura, dkg), // session keys
					)
				})
				.collect(),
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		dkg: egg_rococo_runtime::DKGConfig {
			authorities: invulnerables.iter().map(|x| x.2.clone()).collect::<_>(),
			keygen_threshold: 2,
			signature_threshold: 1,
			authority_ids: invulnerables.iter().map(|x| x.0.clone()).collect::<_>(),
		},
		dkg_proposals: Default::default(),
		asset_registry: AssetRegistryConfig {
			asset_names: vec![],
			native_asset_name: b"WEBB".to_vec(),
			native_existential_deposit: egg_rococo_runtime::EXISTENTIAL_DEPOSIT,
		},
		hasher_bn_254: HasherBn254Config {
			parameters: Some(bn254_x5_3_params.to_bytes()),
			phantom: Default::default(),
		},
		mixer_verifier_bn_254: MixerVerifierBn254Config {
			parameters: Some(mixer_verifier_bn254_params),
			phantom: Default::default(),
		},
		anchor_verifier_bn_254: AnchorVerifierBn254Config {
			parameters: Some(anchor_verifier_bn254_params),
			phantom: Default::default(),
		},
		merkle_tree_bn_254: MerkleTreeBn254Config {
			phantom: Default::default(),
			default_hashes: None,
		},
		mixer_bn_254: MixerBn254Config {
			mixers: vec![(0, 10 * UNIT), (0, 100 * UNIT), (0, 1000 * UNIT)],
		},
		anchor_bn_254: AnchorBn254Config {
			anchors: vec![(0, 10 * UNIT, 2), (0, 100 * UNIT, 2), (0, 1000 * UNIT, 2)],
		},
		treasury: Default::default(),
		vesting: Default::default(),
	}
}
