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

use crate::chain_spec::*;
use arkworks_setups::{common::setup_params, Curve};
use cumulus_primitives_core::ParaId;
use egg_rococo_runtime::{
	AccountId, AnchorBn254Config, AnchorVerifierBn254Config, AssetRegistryConfig, AuraId, DKGId,
	HasherBn254Config, MerkleTreeBn254Config, MixerBn254Config, MixerVerifierBn254Config,
	EXISTENTIAL_DEPOSIT, MILLIUNIT, UNIT,
};
use hex_literal::hex;
use sc_service::ChainType;
use sp_core::{crypto::UncheckedInto, sr25519};

pub fn egg_rococo_config(id: ParaId) -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "tEGG".into());
	properties.insert("tokenDecimals".into(), 12u32.into());

	ChainSpec::from_genesis(
		// Name
		"Egg Rococo",
		// ID
		"rococo",
		ChainType::Live,
		move || {
			rococo_genesis(
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

fn rococo_genesis(
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
	}
}
