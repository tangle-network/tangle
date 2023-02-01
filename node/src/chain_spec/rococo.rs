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
use hex_literal::hex;
use sc_network_common::config::MultiaddrWithPeerId;
use sc_service::ChainType;
use sp_core::{crypto::UncheckedInto, sr25519};
use tangle_rococo_runtime::{
	AccountId, AssetRegistryConfig, AuraId, ClaimsConfig, DKGId, HasherBn254Config, ImOnlineConfig,
	ImOnlineId, MerkleTreeBn254Config, MixerBn254Config, MixerVerifierBn254Config,
	ParachainStakingConfig, VAnchorBn254Config, VAnchorVerifierConfig, UNIT,
};

/// Arana alpha bootnodes
pub fn get_rococo_bootnodes() -> Vec<MultiaddrWithPeerId> {
	vec![
		"/ip4/140.82.21.142/tcp/30333/p2p/12D3KooWPa2aP9fASpyzq2zunUYQYRmc47kwER5NDjEac1ugxDGp"
			.parse()
			.unwrap(),
		"/ip4/149.28.81.60/tcp/30333/p2p/12D3KooWDnXf4qokuVvBGUrvGfpCjsxEBjrnF6fdB2h4Y8t9BD2x"
			.parse()
			.unwrap(),
		"/ip4/45.32.66.129/tcp/30333/p2p/12D3KooWLerJ2wzwmS9hSVxh1QkzwMNeTuquYrA1D9urPcssEFHf"
			.parse()
			.unwrap(),
	]
}

pub fn tangle_alpha_config(id: ParaId) -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "TNT".into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Tangle Alpha",
		// ID
		"tangle-alpha",
		ChainType::Development,
		move || {
			rococo_genesis(
				// root
				hex!["a62a5c2e22ebd14273f1e6552ba0ee07937ff3d859f53475296bbcbb8af1752e"].into(),
				// invulnerables
				generate_invulnerables::<[u8; 32]>(&[
					(
						// publickey
						hex!["a62a5c2e22ebd14273f1e6552ba0ee07937ff3d859f53475296bbcbb8af1752e"],
						// DKG key --scheme Ecdsa
						hex!["03fd0f9d6e4ef6eeb0718866a43c04764177f3fc03203e9ff7ed4dd2885cb52943"]
							.unchecked_into(),
					),
					(
						// publickey
						hex!["6850cc5d0369d11f93c820b91f7bfed4f6fc8b3a5f70a80171183129face154b"],
						// DKG key --scheme Ecdsa
						hex!["03ae1a02a91d59ff20ece458640afbbb672b9335f7da4c9f7d699129d431680ae9"]
							.unchecked_into(),
					),
					(
						// publickey
						hex!["1469f5f6719beaa0a7364259e5fb10846a4457f181807a0c00a6a9cdf14a260d"],
						// DKG key --scheme Ecdsa
						hex!["0252abf0dd2ed408700de539fd65dfc2f6d201e76a4c2e19b875d7b3176a468b0f"]
							.unchecked_into(),
					),
				]),
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					hex!["5ebd99141e19db88cd2c4b778d3cc43e3678d40168aaea56f33d2ea31f67463f"].into(),
					hex!["28714d0740d6b321ad67b8e1a4edd0b53376f735bd10e4904a2c49167bcb7841"].into(),
				],
				id,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some("tangle-alpha"),
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

pub fn tangle_rococo_config(id: ParaId) -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "rTNT".into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Tangle Rococo",
		// ID
		"tangle_rococo",
		ChainType::Live,
		move || {
			testnet_genesis(
				// root
				minerva_testnet_fixtures::get_testnet_root_key(),
				// invulnerables
				minerva_testnet_fixtures::get_testnet_initial_authorities(),
				vec![
					// collator accounts
					hex!["66f07ce0432d73995e3c37afb65aed10d72c872400282d87e23c7cbbf7be5a4e"].into(),
					hex!["0cffebaeb8ba50c523ec6a8ed518d534c1e27cd6f692d4d28618e3256a880412"].into(),
					hex!["3c845c875a53061c8efbe6b149966a105f95097b49280256f65fd994686ed341"].into(),
					hex!["a80afbb2600998b2858e011a1a74e9aa92d8b8edc31ec54253c43d7eafef0675"].into(),
					hex!["3874c16c9855de4791f363d5779dab4cd8e71f21b62494288344002e3a031265"].into(),
					// relayer accounts
					hex!["b6806626f5e4490c27a4ccffed4fed513539b6a455b14b32f58878cf7c5c4e68"].into(),
					hex!["22203dbd79c7ef6ce6bd7ec9b1f4d87425b1db0ab827543d3c7ce3f6a0749005"].into(),
					hex!["6a682aa89827a4028c9f1c2612fb1caa63957a892c7b05659b4e4f46b669b10d"].into(),
					hex!["6abe9075d17ca10075c1f8c11169009334f567e12047c80712fdc499cad8e026"].into(),
					hex!["d85cbc2e3242d5264a020cef8d577b4022e08fa3295423604d4cc2d12bfc906f"].into(),
				],
				id,
			)
		},
		// Bootnodes
		get_rococo_bootnodes(),
		// Telemetry
		None,
		// Protocol ID
		Some("tangle_rococo"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo".into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
}

fn rococo_genesis(
	root_key: AccountId,
	invulnerables: Vec<(AccountId, AuraId, DKGId, NimbusId, VrfId, ImOnlineId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> tangle_rococo_runtime::GenesisConfig {
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
		vk_bytes.to_vec().try_into().unwrap()
	};

	// TODO: Add proper verifying keys for 16-2
	let vanchor_verifier_16x2_bn254_params = {
		let vk_bytes =
			include_bytes!("../../../verifying_keys/vanchor/bn254/x5/2-2-2/verifying_key.bin");
		vk_bytes.to_vec().try_into().unwrap()
	};

	tangle_rococo_runtime::GenesisConfig {
		system: tangle_rococo_runtime::SystemConfig {
			code: tangle_rococo_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		claims: ClaimsConfig { claims: vec![], vesting: vec![], expiry: None },
		sudo: tangle_rococo_runtime::SudoConfig { key: Some(root_key) },
		balances: tangle_rococo_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1_000_000_000 * UNIT)).collect(),
		},
		democracy: Default::default(),
		council: Default::default(),
		indices: Default::default(),
		parachain_info: tangle_rococo_runtime::ParachainInfoConfig { parachain_id: id },
		session: tangle_rococo_runtime::SessionConfig {
			keys: invulnerables
				.iter()
				.cloned()
				.map(|(acc, aura, dkg, nimbus, vrf, im_online)| {
					(
						acc.clone(),                                         // account id
						acc,                                                 // validator id
						dkg_session_keys(aura, dkg, nimbus, vrf, im_online), // session keys
					)
				})
				.collect(),
		},
		aura: Default::default(),
		parachain_system: Default::default(),
		dkg: tangle_rococo_runtime::DKGConfig {
			authorities: invulnerables.iter().map(|x| x.2.clone()).collect::<_>(),
			keygen_threshold: 5,
			signature_threshold: 3,
			authority_ids: invulnerables.iter().map(|x| x.0.clone()).collect::<_>(),
		},
		dkg_proposals: Default::default(),
		asset_registry: AssetRegistryConfig {
			asset_names: vec![],
			native_asset_name: b"TNT".to_vec().try_into().unwrap(),
			native_existential_deposit: tangle_rococo_runtime::EXISTENTIAL_DEPOSIT,
		},
		hasher_bn_254: HasherBn254Config {
			parameters: Some(bn254_x5_3_params.to_bytes().try_into().unwrap()),
			phantom: Default::default(),
		},
		mixer_verifier_bn_254: MixerVerifierBn254Config {
			parameters: Some(mixer_verifier_bn254_params.try_into().unwrap()),
			phantom: Default::default(),
		},
		merkle_tree_bn_254: MerkleTreeBn254Config {
			phantom: Default::default(),
			default_hashes: None,
		},
		mixer_bn_254: MixerBn254Config {
			mixers: vec![(0, 10 * UNIT), (0, 100 * UNIT), (0, 1000 * UNIT)],
		},
		v_anchor_bn_254: VAnchorBn254Config {
			max_deposit_amount: 1_000_000 * UNIT,
			min_withdraw_amount: 0,
			vanchors: vec![(0, 2)],
			phantom: Default::default(),
		},
		v_anchor_verifier: VAnchorVerifierConfig {
			parameters: Some(vec![
				(2, 2, vanchor_verifier_bn254_params),
				(2, 16, vanchor_verifier_16x2_bn254_params),
			]),
			phantom: Default::default(),
		},
		treasury: Default::default(),
		vesting: Default::default(),
		parachain_staking: ParachainStakingConfig {
			candidates: invulnerables
				.iter()
				.cloned()
				.map(|(account, _, _, _, _, _)| {
					(account, tangle_rococo_runtime::staking::NORMAL_COLLATOR_MINIMUM_STAKE)
				})
				.collect(),
			delegations: vec![], //delegations
			inflation_config: tangle_rococo_runtime::staking::inflation_config::<
				tangle_rococo_runtime::Runtime,
			>(),
			collator_commission: COLLATOR_COMMISSION,
			parachain_bond_reserve_percent: PARACHAIN_BOND_RESERVE_PERCENT,
			blocks_per_round: BLOCKS_PER_ROUND,
		},
		im_online: ImOnlineConfig { keys: vec![] },
	}
}
