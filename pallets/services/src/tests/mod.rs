// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.

use std::collections::BTreeMap;

pub use super::*;
pub use crate::mock::*;
use frame_support::assert_ok;
use sp_core::{Pair, bounded_vec};
use sp_runtime::Percent;
use tangle_primitives::services::*;

mod asset_security;
mod blueprint;
mod hooks;
mod jobs;
mod native_slashing;
mod payments;
mod registration;
mod security;
mod service;
mod slashing;
mod subscription_billing;
mod type_checking;

pub const ALICE: u8 = 1;
pub const BOB: u8 = 2;
pub const CHARLIE: u8 = 3;
pub const DAVE: u8 = 4;
pub const EVE: u8 = 5;

pub const KEYGEN_JOB_ID: u8 = 0;
pub const SIGN_JOB_ID: u8 = 1;

pub fn mint_tokens(
	asset_id: AssetId,
	creator: <Runtime as frame_system::Config>::AccountId,
	recipient: <Runtime as frame_system::Config>::AccountId,
	amount: Balance,
) {
	assert_ok!(Assets::mint(RuntimeOrigin::signed(creator.clone()), asset_id, recipient, amount));
}

// Common test utilities and setup
pub(crate) fn cggmp21_blueprint() -> ServiceBlueprint<ConstraintsOf<Runtime>> {
	#[allow(deprecated)]
	ServiceBlueprint {
		metadata: ServiceMetadata { name: "CGGMP21 TSS".try_into().unwrap(), ..Default::default() },
		manager: BlueprintServiceManager::Evm(CGGMP21_BLUEPRINT),
		master_manager_revision: MasterBlueprintServiceManagerRevision::Latest,
		jobs: bounded_vec![
			JobDefinition {
				metadata: JobMetadata { name: "keygen".try_into().unwrap(), ..Default::default() },
				params: bounded_vec![FieldType::Uint8],
				result: bounded_vec![FieldType::List(Box::new(FieldType::Uint8))],
				pricing_model: PricingModel::PayOnce { amount: 100 },
			},
			JobDefinition {
				metadata: JobMetadata { name: "sign".try_into().unwrap(), ..Default::default() },
				params: bounded_vec![
					FieldType::Uint64,
					FieldType::List(Box::new(FieldType::Uint8))
				],
				result: bounded_vec![FieldType::List(Box::new(FieldType::Uint8))],
				pricing_model: PricingModel::PayOnce { amount: 200 },
			},
		],
		registration_params: bounded_vec![],
		request_params: bounded_vec![],
		sources: Default::default(),
		supported_membership_models: bounded_vec![
			MembershipModelType::Fixed,
			MembershipModelType::Dynamic,
		],
	}
}

pub(crate) fn test_ecdsa_key() -> [u8; 65] {
	let (ecdsa_key, _) = sp_core::ecdsa::Pair::generate();
	let secret = k256::ecdsa::SigningKey::from_slice(&ecdsa_key.seed())
		.expect("Should be able to create a secret key from a seed");
	let verifying_key = k256::ecdsa::VerifyingKey::from(secret);
	let public_key = verifying_key.to_encoded_point(false);
	public_key.to_bytes().to_vec().try_into().unwrap()
}

pub(crate) fn get_security_requirement(
	a: AssetId,
	p: &[u8; 2],
) -> AssetSecurityRequirement<AssetId> {
	AssetSecurityRequirement {
		asset: Asset::Custom(a),
		min_exposure_percent: Percent::from_percent(p[0]),
		max_exposure_percent: Percent::from_percent(p[1]),
	}
}

pub(crate) fn get_security_commitment(a: AssetId, p: u8) -> AssetSecurityCommitment<AssetId> {
	AssetSecurityCommitment { asset: Asset::Custom(a), exposure_percent: Percent::from_percent(p) }
}

struct Deployment {
	blueprint_id: u64,
	service_id: u64,
	#[allow(dead_code)]
	security_commitments: BTreeMap<Asset<AssetId>, AssetSecurityCommitment<AssetId>>,
}

/// A Helper function that creates a blueprint and service instance
fn deploy() -> Deployment {
	let alice = mock_pub_key(ALICE);
	let blueprint = cggmp21_blueprint();
	let blueprint_id = Services::next_blueprint_id();
	assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
	assert_ok!(create_test_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

	let alice = mock_pub_key(ALICE);
	let bob = mock_pub_key(BOB);

	assert_ok!(join_and_register(
		bob.clone(),
		blueprint_id,
		test_ecdsa_key(),
		1000,
		Some("https://example.com/rpc")
	));

	let eve = mock_pub_key(EVE);
	// Give EVE some USDC tokens to pay for the service (need 100 USDC with 6 decimals = 100_000_000
	// units) - USDC is owned by ALICE (authorities[0])
	mint_tokens(USDC, mock_pub_key(ALICE), eve.clone(), 200 * 10u128.pow(6));
	let service_id = Services::next_instance_id();
	assert_ok!(Services::request(
		RuntimeOrigin::signed(eve.clone()),
		None,
		blueprint_id,
		vec![alice.clone()],
		vec![bob.clone()],
		Default::default(),
		vec![
			get_security_requirement(TNT, &[10, 20]), // Include native asset requirement
			get_security_requirement(WETH, &[10, 20])
		],
		100,
		Asset::Custom(USDC),
		100 * 10u128.pow(6), /* Payment amount should match the blueprint's pricing model amount
		                      * (100 USDC with 6 decimals) */
		MembershipModel::Fixed { min_operators: 1 },
	));

	assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

	let security_commitments =
		vec![get_security_commitment(TNT, 10), get_security_commitment(WETH, 10)];
	let security_commitments_map = security_commitments
		.iter()
		.map(|c| (c.asset, c.clone()))
		.collect::<BTreeMap<_, _>>();

	assert_ok!(Services::approve(
		RuntimeOrigin::signed(bob.clone()),
		service_id,
		security_commitments.clone(),
	));

	assert!(Instances::<Runtime>::contains_key(service_id));

	Deployment { blueprint_id, service_id, security_commitments: security_commitments_map }
}

pub fn join_and_register(
	operator: AccountId,
	blueprint_id: BlueprintId,
	key: [u8; 65],
	stake_amount: Balance,
	rpc_address: Option<&str>,
) -> DispatchResult {
	// Join operators with stake
	assert_ok!(MultiAssetDelegation::join_operators(
		RuntimeOrigin::signed(operator.clone()),
		stake_amount
	));

	// Set up delegations for common assets used in tests
	let delegator = mock_pub_key(CHARLIE); // Use Charlie as a delegator

	// Set up native asset delegation
	assert_ok!(MultiAssetDelegation::delegate_nomination(
		RuntimeOrigin::signed(delegator.clone()),
		operator.clone(),
		stake_amount,
		Default::default(),
	));

	// Set up WETH delegation - WETH is owned by BOB (authorities[1])
	let weth_stake = 100_000;
	mint_tokens(WETH, mock_pub_key(BOB), delegator.clone(), weth_stake * 10u128.pow(18));
	assert_ok!(MultiAssetDelegation::deposit(
		RuntimeOrigin::signed(delegator.clone()),
		Asset::Custom(WETH),
		weth_stake,
		None,
		None,
	));
	assert_ok!(MultiAssetDelegation::delegate(
		RuntimeOrigin::signed(delegator.clone()),
		operator.clone(),
		Asset::Custom(WETH),
		weth_stake,
		Default::default(),
	));

	// Set up USDC delegation - USDC is owned by ALICE (authorities[0])
	let usdc_stake = 100_000;
	mint_tokens(USDC, mock_pub_key(ALICE), delegator.clone(), usdc_stake * 10u128.pow(6));
	assert_ok!(MultiAssetDelegation::deposit(
		RuntimeOrigin::signed(delegator.clone()),
		Asset::Custom(USDC),
		usdc_stake,
		None,
		None,
	));
	assert_ok!(MultiAssetDelegation::delegate(
		RuntimeOrigin::signed(delegator.clone()),
		operator.clone(),
		Asset::Custom(USDC),
		usdc_stake,
		Default::default(),
	));

	// Register for blueprint
	let rpc_address = match rpc_address {
		Some(addr) => BoundedString::try_from(addr.to_string()).unwrap(),
		None => BoundedString::default(),
	};

	// Register for blueprint
	assert_ok!(Services::register(
		RuntimeOrigin::signed(operator.clone()),
		blueprint_id,
		OperatorPreferences { key, rpc_address },
		Default::default(),
		0,
	));

	Ok(())
}

#[allow(dead_code)]
pub fn assert_events(mut expected: Vec<RuntimeEvent>) {
	let mut actual: Vec<RuntimeEvent> = System::events()
		.into_iter()
		.map(|e| e.event)
		.filter(|e| matches!(e, RuntimeEvent::Services(_)))
		.collect();
	expected.reverse();

	for evt in expected {
		let next = actual.pop().expect("event expected");
		assert_eq!(next, evt, "Events don't match");
	}
	assert!(actual.is_empty(), "More events than expected");
}

pub fn create_test_blueprint(
	origin: RuntimeOrigin,
	blueprint: ServiceBlueprint<ConstraintsOf<Runtime>>,
) -> Result<(), sp_runtime::DispatchError> {
	Services::create_blueprint(origin, blueprint).map(|_| ()).map_err(|e| e.error)
}

pub fn create_test_blueprint_with_pricing(
	origin: RuntimeOrigin,
	blueprint: ServiceBlueprint<ConstraintsOf<Runtime>>,
	_pricing_model: PricingModel<u32, u128>,
) -> Result<(), sp_runtime::DispatchError> {
	Services::create_blueprint(origin, blueprint).map(|_| ()).map_err(|e| e.error)
}
