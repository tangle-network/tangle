// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
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
	mainnet_fixtures::{get_bootnodes, get_initial_authorities, get_root_key},
};
use hex_literal::hex;
use pallet_airdrop_claims::MultiAddress;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use parity_scale_codec::alloc::collections::BTreeMap;
use sc_consensus_grandpa::AuthorityId as GrandpaId;
use sc_service::ChainType;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{H160, Pair, Public, sr25519};
use sp_runtime::{
	BoundedVec,
	traits::{AccountIdConversion, IdentifyAccount, Verify},
};
use tangle_primitives::types::{BlockNumber, Signature};
use tangle_runtime::{
	AccountId, Balance, MaxVestingSchedules, Perbill, Precompiles, RoleKeyId, StakerStatus,
	TreasuryPalletId, UNIT, WASM_BINARY,
};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec;

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
) -> tangle_runtime::opaque::SessionKeys {
	tangle_runtime::opaque::SessionKeys { babe, grandpa, im_online, role }
}

pub fn local_mainnet_config(chain_id: u64) -> Result<ChainSpec, String> {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "TNT".into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	properties.insert("ss58Format".into(), tangle_primitives::MAINNET_SS58_PREFIX.into());

	let endowment: Balance = 10_000 * UNIT;

	let mut endowed_accounts = mainnet::get_initial_endowed_accounts().0;
	endowed_accounts.extend(vec![
		(get_account_id_from_seed::<sr25519::Public>("Alice"), endowment),
		(get_account_id_from_seed::<sr25519::Public>("Bob"), endowment),
		(get_account_id_from_seed::<sr25519::Public>("Alice//stash"), endowment),
		(get_account_id_from_seed::<sr25519::Public>("Bob//stash"), endowment),
	]);

	Ok(ChainSpec::builder(WASM_BINARY.expect("WASM not available"), Default::default())
		.with_name("Local Tangle Mainnet")
		.with_id("local-tangle-mainnet")
		.with_chain_type(ChainType::Local)
		.with_properties(properties)
		.with_genesis_config_patch(mainnet_genesis(
			// Initial validators
			vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
			// Endowed accounts
			endowed_accounts,
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
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
			Default::default(),
		))
		.build())
}

pub fn tangle_mainnet_config(chain_id: u64) -> Result<ChainSpec, String> {
	let _boot_nodes = get_bootnodes();
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "tTNT".into());
	properties.insert("tokenDecimals".into(), 18u32.into());
	properties.insert("ss58Format".into(), 42.into());

	Ok(ChainSpec::builder(WASM_BINARY.expect("WASM not available"), Default::default())
		.with_name("Tangle Mainnet")
		.with_id("tangle-mainnet")
		.with_chain_type(ChainType::Live)
		.with_properties(properties)
		.with_genesis_config_patch(mainnet_genesis(
			// Initial validators
			get_initial_authorities(),
			// Endowed accounts
			mainnet::get_initial_endowed_accounts().0,
			// Sudo account
			get_root_key(),
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
		))
		.build())
}

#[allow(clippy::too_many_arguments)]
fn mainnet_genesis(
	initial_authorities: Vec<(AccountId, BabeId, GrandpaId, ImOnlineId, RoleKeyId)>,
	endowed_accounts: Vec<(AccountId, Balance)>,
	root_key: AccountId,
	chain_id: u64,
	genesis_airdrop: DistributionResult,
	genesis_non_airdrop: Vec<(MultiAddress, u128, u64, u64, u128)>,
	genesis_evm_distribution: Vec<(H160, fp_evm::GenesisAccount)>,
) -> serde_json::Value {
	// stakers: all validators and nominators.
	let stakers: Vec<(AccountId, AccountId, Balance, StakerStatus<AccountId>)> =
		initial_authorities
			.iter()
			.map(|x| (x.0.clone(), x.0.clone(), 100 * UNIT, StakerStatus::<AccountId>::Validator))
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
	let revert_bytecode = [0x60, 0x00, 0x60, 0x00, 0xFD];
	let evm_accounts = {
		let mut map = BTreeMap::new();

		for (address, account) in genesis_evm_distribution {
			map.insert(address, account);
		}

		Precompiles::used_addresses_h160().for_each(|address| {
			map.insert(
				address,
				fp_evm::GenesisAccount {
					nonce: Default::default(),
					balance: Default::default(),
					storage: Default::default(),
					code: revert_bytecode.to_vec(),
				},
			);
		});
		map
	};

	let council_members: Vec<AccountId> = vec![
		hex!["483b466832e094f01b1779a7ed07025df319c492dac5160aca89a3be117a7b6d"].into(),
		hex!["86d08e7bbe77bc74e3d88ee22edc53368bc13d619e05b66fe6c4b8e2d5c7015a"].into(),
		hex!["e421301e5aa5dddee51f0d8c73e794df16673e53157c5ea657be742e35b1793f"].into(),
		hex!["4ce3a4da3a7c1ce65f7edeff864dc3dd42e8f47eecc2726d99a0a80124698217"].into(),
		hex!["dcd9b70a0409b7626cba1a4016d8da19f4df5ce9fc5e8d16b789e71bb1161d73"].into(),
	]
	.into_iter()
	.collect();

	serde_json::json!({
		"sudo": { "key": Some(root_key) },
		"balances": {
			"balances": endowed_accounts.to_vec(),
		},
		"session": {
			"keys": initial_authorities
				.iter()
				.map(|x| {
			(
				x.0.clone(),
				x.0.clone(),
				generate_session_keys(x.1.clone(), x.2.clone(), x.3.clone(), x.4.clone()),
			)
				})
				.collect::<Vec<_>>()
		},
		"vesting": {
			"vesting": genesis_non_airdrop
				.iter()
				.map(|(address, _value, begin, end, liquid)| {
				(address.clone().to_account_id_32(), *begin, *end, *liquid)
			})
			.collect::<Vec<_>>(),
		},
		"staking": {
			"validatorCount": initial_authorities.len() as u32,
			"minimumValidatorCount": initial_authorities.len() as u32 - 1,
			"invulnerables": initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
			"slashRewardFraction": Perbill::from_percent(10),
			"stakers" : stakers,
		},
		"council": {
			"members": council_members,
		},
		"babe": {
			"epochConfig": tangle_runtime::BABE_GENESIS_EPOCH_CONFIG,
		},
		"evm" : {
			"accounts": evm_accounts
		},
		"claims": {
			"claims": genesis_airdrop.claims,
			"vesting": vesting_claims,
			"expiry": Some((
				5_265_000u64, // 1 year
				MultiAddress::Native(TreasuryPalletId::get().into_account_truncating()),
			)),
		},
		"evmChainId": { "chainId": chain_id },
	})
}
