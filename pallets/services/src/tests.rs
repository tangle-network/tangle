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
use crate::types::ConstraintsOf;

use super::*;
use frame_support::{assert_err, assert_ok};
use mock::*;
use sp_core::U256;
use sp_core::{bounded_vec, ecdsa, ByteArray};
use sp_runtime::{KeyTypeId, Percent};
use tangle_primitives::services::*;
use tangle_primitives::MultiAssetDelegationInfo;

const ALICE: u8 = 1;
const BOB: u8 = 2;
const CHARLIE: u8 = 3;
const DAVE: u8 = 4;
const EVE: u8 = 5;

const KEYGEN_JOB_ID: u8 = 0;
const SIGN_JOB_ID: u8 = 1;

fn zero_key() -> ecdsa::Public {
	ecdsa::Public::try_from([0; 33].as_slice()).unwrap()
}

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

fn cggmp21_blueprint() -> ServiceBlueprint<ConstraintsOf<Runtime>> {
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
	}
}

#[test]
fn update_mbsm() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		assert_eq!(Pallet::<Runtime>::mbsm_latest_revision(), 0);
		assert_eq!(Pallet::<Runtime>::mbsm_address(0).unwrap(), MBSM);

		// Add a new revision
		let new_mbsm = {
			let mut v = MBSM;
			v.randomize();
			v
		};

		assert_ok!(Services::update_master_blueprint_service_manager(
			RuntimeOrigin::root(),
			new_mbsm
		));

		assert_eq!(Pallet::<Runtime>::mbsm_latest_revision(), 1);
		assert_eq!(Pallet::<Runtime>::mbsm_address(1).unwrap(), new_mbsm);
		// Old one should still be there
		assert_eq!(Pallet::<Runtime>::mbsm_address(0).unwrap(), MBSM);
		// Doesn't exist
		assert!(Pallet::<Runtime>::mbsm_address(2).is_err());
	});
}

#[test]
fn update_mbsm_not_root() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);
		assert_err!(
			Services::update_master_blueprint_service_manager(RuntimeOrigin::signed(alice), MBSM),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn create_service_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		let alice = mock_pub_key(ALICE);

		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint,));

		let next_id = Services::next_blueprint_id();
		assert_eq!(next_id, 1);
		assert_events(vec![RuntimeEvent::Services(crate::Event::BlueprintCreated {
			owner: alice,
			blueprint_id: next_id - 1,
		})]);

		let (_, blueprint) = Services::blueprints(next_id - 1).unwrap();

		// The MBSM should be set on the blueprint
		assert_eq!(Pallet::<Runtime>::mbsm_address_of(&blueprint).unwrap(), MBSM);
		// The master manager revision should pinned to a specific revision that is equal to the
		// latest revision of the MBSM.
		assert_eq!(
			blueprint.master_manager_revision,
			MasterBlueprintServiceManagerRevision::Specific(0)
		);
	});
}

#[test]
fn register_on_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);

		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);

		let registration_call = Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				price_targets: price_targets(MachineKind::Large),
			},
			Default::default(),
			0,
		);
		assert_ok!(registration_call);

		assert_events(vec![RuntimeEvent::Services(crate::Event::Registered {
			provider: bob.clone(),
			blueprint_id: 0,
			preferences: OperatorPreferences {
				key: zero_key(),
				price_targets: price_targets(MachineKind::Large),
			},
			registration_args: Default::default(),
		})]);

		// The blueprint should be added to my blueprints in my profile.
		let profile = OperatorsProfile::<Runtime>::get(bob.clone()).unwrap();
		assert!(profile.blueprints.contains(&0));

		// if we try to register again, it should fail.
		assert_err!(
			Services::register(
				RuntimeOrigin::signed(bob),
				0,
				OperatorPreferences { key: zero_key(), price_targets: Default::default() },
				Default::default(),
				0,
			),
			crate::Error::<Runtime>::AlreadyRegistered
		);

		// if we try to register with a non active operator, should fail
		assert_err!(
			Services::register(
				RuntimeOrigin::signed(mock_pub_key(10)),
				0,
				OperatorPreferences { key: zero_key(), price_targets: Default::default() },
				Default::default(),
				0,
			),
			crate::Error::<Runtime>::OperatorNotActive
		);
	});
}

#[test]
fn pre_register_on_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);

		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);

		let pre_registration_call = Services::pre_register(RuntimeOrigin::signed(bob.clone()), 0);
		assert_ok!(pre_registration_call);

		assert_events(vec![RuntimeEvent::Services(crate::Event::PreRegistration {
			operator: bob.clone(),
			blueprint_id: 0,
		})]);
	});
}

#[test]
fn update_price_targets() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);

		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);

		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				price_targets: price_targets(MachineKind::Small)
			},
			Default::default(),
			0,
		));

		assert_eq!(
			Operators::<Runtime>::get(0, &bob).unwrap(),
			OperatorPreferences {
				key: zero_key(),
				price_targets: price_targets(MachineKind::Small)
			}
		);

		assert_events(vec![RuntimeEvent::Services(crate::Event::Registered {
			provider: bob.clone(),
			blueprint_id: 0,
			preferences: OperatorPreferences {
				key: zero_key(),
				price_targets: price_targets(MachineKind::Small),
			},
			registration_args: Default::default(),
		})]);

		// update price targets
		assert_ok!(Services::update_price_targets(
			RuntimeOrigin::signed(bob.clone()),
			0,
			price_targets(MachineKind::Medium),
		));

		assert_eq!(
			Operators::<Runtime>::get(0, &bob).unwrap().price_targets,
			price_targets(MachineKind::Medium)
		);

		assert_events(vec![RuntimeEvent::Services(crate::Event::PriceTargetsUpdated {
			operator: bob,
			blueprint_id: 0,
			price_targets: price_targets(MachineKind::Medium),
		})]);

		// try to update price targets when not registered
		let charlie = mock_pub_key(CHARLIE);
		assert_err!(
			Services::update_price_targets(
				RuntimeOrigin::signed(charlie),
				0,
				price_targets(MachineKind::Medium)
			),
			crate::Error::<Runtime>::NotRegistered
		);
	});
}

#[test]
fn unregister_from_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: zero_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));
		assert_ok!(Services::unregister(RuntimeOrigin::signed(bob.clone()), 0));
		assert!(!Operators::<Runtime>::contains_key(0, &bob));

		// The blueprint should be removed from my blueprints in my profile.
		let profile = OperatorsProfile::<Runtime>::get(bob.clone()).unwrap();
		assert!(!profile.blueprints.contains(&0));

		assert_events(vec![RuntimeEvent::Services(crate::Event::Unregistered {
			operator: bob,
			blueprint_id: 0,
		})]);

		// try to deregister when not registered
		let charlie = mock_pub_key(CHARLIE);
		assert_err!(
			Services::unregister(RuntimeOrigin::signed(charlie), 0),
			crate::Error::<Runtime>::NotRegistered
		);
	});
}

#[test]
fn request_service() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: zero_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));
		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			OperatorPreferences { key: zero_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));
		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			OperatorPreferences { key: zero_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		let eve = mock_pub_key(EVE);
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			Default::default(),
			vec![USDC, WETH],
			100,
			Asset::Custom(USDC),
			0,
		));

		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// Bob approves the request
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			Percent::from_percent(10)
		));

		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceRequestApproved {
			operator: bob.clone(),
			request_id: 0,
			blueprint_id: 0,
			approved: vec![bob.clone()],
			pending_approvals: vec![charlie.clone(), dave.clone()],
		})]);
		// Charlie approves the request
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			Percent::from_percent(20)
		));

		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceRequestApproved {
			operator: charlie.clone(),
			request_id: 0,
			blueprint_id: 0,
			approved: vec![bob.clone(), charlie.clone()],
			pending_approvals: vec![dave.clone()],
		})]);

		// Dave approves the request
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(dave.clone()),
			0,
			Percent::from_percent(30)
		));

		assert_events(vec![
			RuntimeEvent::Services(crate::Event::ServiceRequestApproved {
				operator: dave.clone(),
				request_id: 0,
				blueprint_id: 0,
				approved: vec![bob.clone(), charlie.clone(), dave.clone()],
				pending_approvals: vec![],
			}),
			RuntimeEvent::Services(crate::Event::ServiceInitiated {
				owner: eve,
				request_id: 0,
				service_id: 0,
				blueprint_id: 0,
				assets: vec![USDC, WETH],
			}),
		]);

		// The request is now fully approved
		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		// Now the service should be initiated
		assert!(Instances::<Runtime>::contains_key(0));

		// The service should also be added to the services for each operator.
		let profile = OperatorsProfile::<Runtime>::get(bob).unwrap();
		assert!(profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(charlie).unwrap();
		assert!(profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(dave).unwrap();
		assert!(profile.services.contains(&0));
	});
}

#[test]
fn request_service_with_no_assets() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: zero_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));
		let eve = mock_pub_key(EVE);
		assert_err!(
			Services::request(
				RuntimeOrigin::signed(eve.clone()),
				0,
				vec![alice.clone()],
				vec![bob.clone()],
				Default::default(),
				vec![], // no assets
				100,
				Asset::Custom(USDC),
				0,
			),
			Error::<Runtime>::NoAssetsProvided
		);
	});
}

#[test]
fn request_service_with_payment_asset() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: zero_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			vec![],
			vec![bob.clone()],
			Default::default(),
			vec![TNT, USDC, WETH],
			100,
			Asset::Custom(USDC),
			5 * 10u128.pow(6), // 5 USDC
		));

		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// The Pallet account now has 5 USDC
		assert_eq!(Assets::balance(USDC, Services::account_id()), 5 * 10u128.pow(6));

		// Bob approves the request
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			Percent::from_percent(10)
		));

		// The request is now fully approved
		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		// Now the service should be initiated
		assert!(Instances::<Runtime>::contains_key(0));
	});
}

#[test]
fn request_service_with_payment_token() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: zero_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			vec![],
			vec![bob.clone()],
			Default::default(),
			vec![TNT, USDC, WETH],
			100,
			Asset::Erc20(USDC_ERC20),
			5 * 10u128.pow(6), // 5 USDC
		));

		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// The Pallet address now has 5 USDC
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, Services::address()).map(|(b, _)| b),
			U256::from(5 * 10u128.pow(6))
		);

		// Bob approves the request
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			Percent::from_percent(10)
		));

		// The request is now fully approved
		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		// Now the service should be initiated
		assert!(Instances::<Runtime>::contains_key(0));
	});
}

#[test]
fn job_calls() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: zero_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));
		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			OperatorPreferences { key: zero_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));
		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			OperatorPreferences { key: zero_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		let eve = mock_pub_key(EVE);
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			Default::default(),
			vec![WETH],
			100,
			Asset::Custom(USDC),
			0,
		));

		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			Percent::from_percent(10)
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			Percent::from_percent(10)
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(dave.clone()),
			0,
			Percent::from_percent(10)
		));
		assert!(Instances::<Runtime>::contains_key(0));
		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceInitiated {
			owner: eve.clone(),
			request_id: 0,
			service_id: 0,
			blueprint_id: 0,
			assets: vec![WETH],
		})]);

		// now we can call the jobs
		let job_call_id = 0;
		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			0,
			0,
			bounded_vec![Field::Uint8(2)],
		));

		assert!(JobCalls::<Runtime>::contains_key(0, job_call_id));
		assert_events(vec![RuntimeEvent::Services(crate::Event::JobCalled {
			caller: eve,
			service_id: 0,
			job: 0,
			call_id: job_call_id,
			args: vec![Field::Uint8(2)],
		})]);
	});
}

#[test]
fn job_result() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: zero_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));
		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			OperatorPreferences { key: zero_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));
		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			OperatorPreferences { key: zero_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		let eve = mock_pub_key(EVE);
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			Default::default(),
			vec![WETH],
			100,
			Asset::Custom(USDC),
			0,
		));

		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			Percent::from_percent(10)
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			Percent::from_percent(10)
		));

		assert_ok!(Services::approve(
			RuntimeOrigin::signed(dave.clone()),
			0,
			Percent::from_percent(10)
		));
		assert!(Instances::<Runtime>::contains_key(0));
		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceInitiated {
			owner: eve.clone(),
			request_id: 0,
			service_id: 0,
			blueprint_id: 0,
			assets: vec![WETH],
		})]);

		// now we can call the jobs
		let keygen_job_call_id = 0;

		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			0,
			0,
			bounded_vec![Field::Uint8(2)]
		));

		assert!(JobCalls::<Runtime>::contains_key(0, keygen_job_call_id));
		// now we can set the job result
		let key_type = KeyTypeId(*b"mdkg");
		let dkg = sp_io::crypto::ecdsa_generate(key_type, None);
		assert_ok!(Services::submit_result(
			RuntimeOrigin::signed(bob.clone()),
			0,
			keygen_job_call_id,
			bounded_vec![Field::Bytes(dkg.to_raw_vec().try_into().unwrap())],
		));

		// submit signing job
		let _signing_job_call_id = 1;
		let data_hash = sp_core::keccak_256(&[1; 32]);

		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			0,
			SIGN_JOB_ID,
			bounded_vec![
				Field::Uint64(keygen_job_call_id),
				Field::Bytes(data_hash.to_vec().try_into().unwrap())
			],
		));

		// now we can set the job result
		let signature = sp_io::crypto::ecdsa_sign_prehashed(key_type, &dkg, &data_hash).unwrap();
		let mut signature_bytes = signature.to_raw_vec();
		// fix the v value (it should be 27 or 28).
		signature_bytes[64] += 27u8;
		// For some reason, the signature is not being verified.
		// in EVM, ecrecover is used to verify the signature, but it returns
		// 0x000000000000000000000000000000000000000 as the address of the signer.
		// even though the signature is correct, and we have the precomiles in the runtime.
		//
		// assert_ok!(Services::submit_result(
		// 	RuntimeOrigin::signed(bob.clone()),
		// 	0,
		// 	signing_job_call_id,
		// 	bounded_vec![Field::Bytes(signature_bytes.try_into().unwrap())],
		// ));
	});
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
		OperatorPreferences { key: zero_key(), price_targets: Default::default() },
		Default::default(),
		0,
	));

	let eve = mock_pub_key(EVE);
	let service_id = Services::next_instance_id();
	assert_ok!(Services::request(
		RuntimeOrigin::signed(eve.clone()),
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

#[test]
fn unapplied_slash() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let eve = mock_pub_key(EVE);
		let bob = mock_pub_key(BOB);
		// now we can call the jobs
		let job_call_id = Services::next_job_call_id();
		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			service_id,
			KEYGEN_JOB_ID,
			bounded_vec![Field::Uint8(1)],
		));
		// sumbit an invalid result
		let mut dkg = vec![0; 33];
		dkg[32] = 1;
		assert_ok!(Services::submit_result(
			RuntimeOrigin::signed(bob.clone()),
			0,
			job_call_id,
			bounded_vec![Field::Bytes(dkg.try_into().unwrap())],
		));

		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Slash the operator for the invalid result
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		let bob_slash = <Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob);
		let expected_slash_amount =
			(slash_percent * bob_exposed_restake_percentage).mul_floor(bob_slash);

		assert_events(vec![RuntimeEvent::Services(crate::Event::UnappliedSlash {
			era: 0,
			index: 0,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: expected_slash_amount,
		})]);
	});
}

#[test]
fn unapplied_slash_with_invalid_origin() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let eve = mock_pub_key(EVE);
		let bob = mock_pub_key(BOB);
		let slash_percent = Percent::from_percent(50);
		// Try to slash with an invalid origin
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(eve.clone()),
				bob.clone(),
				service_id,
				slash_percent
			),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn slash_account_not_an_operator() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let karen = mock_pub_key(23);

		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();
		let slash_percent = Percent::from_percent(50);
		// Try to slash an operator that is not active in this service
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				karen.clone(),
				service_id,
				slash_percent
			),
			Error::<Runtime>::OffenderNotOperator
		);
	});
}

#[test]
fn dispute() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let bob = mock_pub_key(BOB);
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Slash the operator for the invalid result
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		let era = 0;
		let slash_index = 0;

		// Dispute the slash
		let dispute_origin =
			Services::query_dispute_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::dispute(
			RuntimeOrigin::signed(dispute_origin.clone()),
			era,
			slash_index
		));

		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		let bob_slash = <Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob);
		let expected_slash_amount =
			(slash_percent * bob_exposed_restake_percentage).mul_floor(bob_slash);

		assert_events(vec![RuntimeEvent::Services(crate::Event::SlashDiscarded {
			era: 0,
			index: 0,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: expected_slash_amount,
		})]);
	});
}

#[test]
fn dispute_with_unauthorized_origin() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let eve = mock_pub_key(EVE);
		let bob = mock_pub_key(BOB);
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Slash the operator for the invalid result
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		let era = 0;
		let slash_index = 0;

		// Try to dispute with an invalid origin
		assert_err!(
			Services::dispute(RuntimeOrigin::signed(eve.clone()), era, slash_index),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn dispute_an_already_applied_slash() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let eve = mock_pub_key(EVE);
		let bob = mock_pub_key(BOB);
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Slash the operator for the invalid result
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		let era = 0;
		let slash_index = 0;
		// Simulate a slash happening
		UnappliedSlashes::<Runtime>::remove(era, slash_index);

		// Try to dispute an already applied slash
		assert_err!(
			Services::dispute(RuntimeOrigin::signed(eve.clone()), era, slash_index),
			Error::<Runtime>::UnappliedSlashNotFound
		);
	});
}

#[test]
fn hooks() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));

		let alice = mock_pub_key(ALICE);
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let blueprint = ServiceBlueprint {
			metadata: ServiceMetadata {
				name: "Hooks Tests".try_into().unwrap(),
				..Default::default()
			},
			manager: BlueprintServiceManager::Evm(HOOKS_TEST),
			master_manager_revision: MasterBlueprintServiceManagerRevision::Latest,
			jobs: bounded_vec![JobDefinition {
				metadata: JobMetadata { name: "foo".try_into().unwrap(), ..Default::default() },
				params: bounded_vec![],
				result: bounded_vec![],
			},],
			registration_params: bounded_vec![],
			request_params: bounded_vec![],
			gadget: Default::default(),
		};

		// OnBlueprintCreated hook should be called
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnBlueprintCreated()")]);

		// OnRegister hook should be called
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: zero_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnRegister()")]);

		// OnUnregister hook should be called
		assert_ok!(Services::unregister(RuntimeOrigin::signed(bob.clone()), 0));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnUnregister()")]);

		// Register again to continue testing
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences { key: zero_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		// OnUpdatePriceTargets hook should be called
		assert_ok!(Services::update_price_targets(
			RuntimeOrigin::signed(bob.clone()),
			0,
			price_targets(MachineKind::Medium),
		));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnUpdatePriceTargets()")]);

		// OnRequest hook should be called
		assert_ok!(Services::request(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![USDC, WETH],
			100,
			Asset::Custom(USDC),
			0,
		));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnRequest()")]);

		// OnReject hook should be called
		assert_ok!(Services::reject(RuntimeOrigin::signed(bob.clone()), 0));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnReject()")]);

		// OnApprove hook should be called
		// OnServiceInitialized is also called
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			Percent::from_percent(10)
		));
		assert_evm_logs(&[
			evm_log!(HOOKS_TEST, b"OnApprove()"),
			evm_log!(HOOKS_TEST, b"OnServiceInitialized()"),
		]);

		// OnJobCall hook should be called
		assert_ok!(Services::call(RuntimeOrigin::signed(charlie.clone()), 0, 0, bounded_vec![],));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnJobCall()")]);

		// OnJobResult hook should be called
		assert_ok!(Services::submit_result(
			RuntimeOrigin::signed(bob.clone()),
			0,
			0,
			bounded_vec![],
		));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnJobResult()")]);
		// OnServiceTermination hook should be called
		assert_ok!(Services::terminate(RuntimeOrigin::signed(charlie.clone()), 0));
		assert_evm_logs(&[evm_log!(HOOKS_TEST, b"OnServiceTermination()")]);
	});
}
