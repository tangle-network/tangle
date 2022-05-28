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
use egg_runtime::{
	AccountId, AnchorBn254Config, AnchorVerifierBn254Config, AssetRegistryConfig, AuraId, DKGId,
	HasherBn254Config, MerkleTreeBn254Config, MixerBn254Config, MixerVerifierBn254Config,
	Signature, EXISTENTIAL_DEPOSIT, MILLIUNIT, UNIT,
};
use hex_literal::hex;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<egg_runtime::GenesisConfig, Extensions>;

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
pub fn dkg_session_keys(keys: AuraId, dkg_keys: DKGId) -> egg_runtime::SessionKeys {
	egg_runtime::SessionKeys { aura: keys, dkg: dkg_keys }
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
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

	ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Local,
		move || {
			testnet_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![(
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_collator_keys_from_seed("Alice"),
					get_dkg_keys_from_seed("Alice"),
				)],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				id,
			)
		},
		vec![],
		None,
		None,
		None,
		None,
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

	ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![(
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_collator_keys_from_seed("Alice"),
					get_dkg_keys_from_seed("Alice"),
				)],
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
		Vec::new(),
		None,
		None,
		None,
		None,
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
}

pub fn latest_egg_testnet_config(id: ParaId) -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "tEGG".into());
	properties.insert("tokenDecimals".into(), 12u32.into());

	ChainSpec::from_genesis(
		// Name
		"Egg Testnet",
		// ID
		"egg_testnet",
		ChainType::Live,
		move || {
			testnet_genesis(
				// root
				hex!["64fb440cf326ff4d47c2d98107f3533dbbdd8b3d3d2bafb27d141cb435d2f37d"].into(),
				vec![
					(
						//1//account
						hex!["e44c85670c9a5cea588c2533613d130d8b3a81dc9ea1d47a54f289206c86d676"]
							.into(),
						//1//aura
						hex!["40d082fea20451b92da758234b0fd3e26f2f7e87a7e65b83f3c47f23bce29153"]
							.unchecked_into(),
						//1//dkg (--scheme Ecdsa)
						hex!["03ffb24275fd6ed90ac86e3b7a18a3ea8e96ff0aac63f301df1e3145c0d388368a"]
							.unchecked_into(),
					),
					(
						//2//account
						hex!["f2d1c2eb434926d1b6e8e894f3e2021edc88c33afe6266e423ff3da2a93dca5e"]
							.into(),
						//2//aura
						hex!["880ecc41a19a9afe55fd029c568ff138d3e74773eae482117077f672f30ac241"]
							.unchecked_into(),
						//2//dkg (--scheme Ecdsa)
						hex!["02e1e595807c8bd71e4d124fe1a20e2fa68f53452410fed0320961933ff97296f3"]
							.unchecked_into(),
					),
					(
						//3//account
						hex!["a68bb44b8d70f12d392b0e2a9f91608c4f136c9aba144beb1ad558e9f6a51a6d"]
							.into(),
						//3//aura
						hex!["c6469a84e9325507b881c52e0f1f833bed005e3514ff23b675a1008017ee4709"]
							.unchecked_into(),
						//3//dkg (--scheme Ecdsa)
						hex!["03ade18f4f4b5096cb2daf3cddba6fd53d0d701478c397b2ead1c96409c2b6bf1b"]
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
					hex!["64fb440cf326ff4d47c2d98107f3533dbbdd8b3d3d2bafb27d141cb435d2f37d"].into(),
					hex!["e44c85670c9a5cea588c2533613d130d8b3a81dc9ea1d47a54f289206c86d676"].into(),
					hex!["f2d1c2eb434926d1b6e8e894f3e2021edc88c33afe6266e423ff3da2a93dca5e"].into(),
					hex!["a68bb44b8d70f12d392b0e2a9f91608c4f136c9aba144beb1ad558e9f6a51a6d"].into(),
				],
				id,
			)
		},
		vec![
			"/dns/testnet.webb.tools/tcp/30333/p2p/12D3KooWRazFqUMAGSaTYfj9C7WGhxPzmgu422ZHRDS5J41y6b7o".parse().unwrap(),
			"/dns/testnet1.webb.tools/tcp/30333/p2p/12D3KooWE7TRKmNotiqXh38muaymNrT6iMduB1yCM9F9mFwdNcG3".parse().unwrap(),
			"/dns/testnet2.webb.tools/tcp/30333/p2p/12D3KooWGDWxDj62vEwuJbtUXqytqDVYfVsgNw1RyNuVpXUD2Yg7".parse().unwrap(),
		],
		None,
		None,
		None,
		None,
		Extensions {
			relay_chain: "rococo".into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
}

fn testnet_genesis(
	root_key: AccountId,
	invulnerables: Vec<(AccountId, AuraId, DKGId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> egg_runtime::GenesisConfig {
	let curve_bn254 = Curve::Bn254;

	log::info!("Bn254 x5 w3 params");
	let bn254_x5_3_params = setup_params::<ark_bn254::Fr>(curve_bn254, 5, 3);

	log::info!("Verifier params for mixer");
	let mixer_verifier_bn254_params = {
		let vk_bytes = include_bytes!("../../verifying_keys/mixer/bn254/verifying_key.bin");
		vk_bytes.to_vec()
	};

	log::info!("Verifier params for anchor");
	let anchor_verifier_bn254_params = {
		let vk_bytes = include_bytes!("../../verifying_keys/anchor/bn254/2/verifying_key.bin");
		vk_bytes.to_vec()
	};

	egg_runtime::GenesisConfig {
		system: egg_runtime::SystemConfig {
			code: egg_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		sudo: egg_runtime::SudoConfig { key: Some(root_key) },
		balances: egg_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, MILLIUNIT * 4096_000)).collect(),
		},
		indices: Default::default(),
		parachain_info: egg_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: egg_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: egg_runtime::SessionConfig {
			keys: invulnerables
				.iter()
				.cloned()
				.map(|(acc, aura, dkg)| {
					(
						acc.clone(),                 // account id
						acc.clone(),                 // validator id
						dkg_session_keys(aura, dkg), // session keys
					)
				})
				.collect(),
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		dkg: egg_runtime::DKGConfig {
			authorities: invulnerables.iter().map(|x| x.2.clone()).collect::<_>(),
			threshold: Default::default(),
			authority_ids: invulnerables.iter().map(|x| x.0.clone()).collect::<_>(),
		},
		dkg_proposals: Default::default(),
		asset_registry: AssetRegistryConfig {
			asset_names: vec![],
			native_asset_name: b"WEBB".to_vec(),
			native_existential_deposit: egg_runtime::EXISTENTIAL_DEPOSIT,
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
	}
}
