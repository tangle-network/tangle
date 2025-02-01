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

use super::*;
use crate::mock::*;
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::bounded_vec;
use sp_core::Pair;
use sp_runtime::Percent;
use tangle_primitives::services::*;

mod blueprint;
mod hooks;
mod jobs;
mod registration;
mod service;
mod slashing;

pub const ALICE: u8 = 1;
pub const BOB: u8 = 2;
pub const CHARLIE: u8 = 3;
pub const DAVE: u8 = 4;
pub const EVE: u8 = 5;

pub const KEYGEN_JOB_ID: u8 = 0;
pub const SIGN_JOB_ID: u8 = 1;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MachineKind {
	Large,
	Medium,
	Small,
}

/// All prices are specified in USD/hr (in u64, so 1e6 = 1$)
fn price_targets(kind: MachineKind) -> PriceTargets {
	match kind {
		MachineKind::Large => PriceTargets {
			cpu: 2_000,
			mem: 1_000,
			storage_hdd: 100,
			storage_ssd: 200,
			storage_nvme: 300,
		},
		MachineKind::Medium => PriceTargets {
			cpu: 1_000,
			mem: 500,
			storage_hdd: 50,
			storage_ssd: 100,
			storage_nvme: 150,
		},
		MachineKind::Small => {
			PriceTargets { cpu: 500, mem: 250, storage_hdd: 25, storage_ssd: 50, storage_nvme: 75 }
		},
	}
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
				result: bounded_vec![FieldType::Bytes],
			},
			JobDefinition {
				metadata: JobMetadata { name: "sign".try_into().unwrap(), ..Default::default() },
				params: bounded_vec![FieldType::Uint64, FieldType::Bytes],
				result: bounded_vec![FieldType::Bytes],
			},
		],
		registration_params: bounded_vec![],
		request_params: bounded_vec![],
		gadget: Default::default(),
		supported_membership_models: bounded_vec![
			MembershipModel::Fixed { min_operators: 1 },
			MembershipModel::Dynamic { min_operators: 1, max_operators: None },
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
	bob_exposed_restake_percentage: Percent,
}

/// A Helper function that creates a blueprint and service instance
fn deploy() -> Deployment {
	let alice = mock_pub_key(ALICE);
	let blueprint = cggmp21_blueprint();
	let blueprint_id = Services::next_blueprint_id();
	assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
	assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

	let bob = mock_pub_key(BOB);
	assert_ok!(Services::register(
		RuntimeOrigin::signed(bob.clone()),
		blueprint_id,
		OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
		Default::default(),
		0,
	));

	let eve = mock_pub_key(EVE);
	let service_id = Services::next_instance_id();
	assert_ok!(Services::request(
		RuntimeOrigin::signed(eve.clone()),
		None,
		blueprint_id,
		vec![alice.clone()],
		vec![bob.clone()],
		Default::default(),
		vec![WETH],
		100,
		Asset::Custom(USDC),
		0,
	));

	assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

	let bob_exposed_restake_percentage = Percent::from_percent(10);
	assert_ok!(Services::approve(
		RuntimeOrigin::signed(bob.clone()),
		service_id,
		bob_exposed_restake_percentage,
	));

	assert!(Instances::<Runtime>::contains_key(service_id));

	Deployment { blueprint_id, service_id, bob_exposed_restake_percentage }
}
