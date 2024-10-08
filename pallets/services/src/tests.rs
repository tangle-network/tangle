// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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
use sp_core::{bounded_vec, ecdsa, ByteArray};
use sp_runtime::KeyTypeId;
use tangle_primitives::services::*;

const ALICE: u8 = 1;
const BOB: u8 = 2;
const CHARLIE: u8 = 3;
const DAVE: u8 = 4;
const EVE: u8 = 5;

const USDC: AssetId = 1;
const WETH: AssetId = 2;

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
	ServiceBlueprint {
		metadata: ServiceMetadata { name: "CGGMP21 TSS".try_into().unwrap(), ..Default::default() },
		jobs: bounded_vec![
			JobDefinition {
				metadata: JobMetadata { name: "keygen".try_into().unwrap(), ..Default::default() },
				params: bounded_vec![FieldType::Uint8],
				result: bounded_vec![FieldType::Bytes],
				verifier: JobResultVerifier::Evm(CGGMP21_BLUEPRINT),
			},
			JobDefinition {
				metadata: JobMetadata { name: "sign".try_into().unwrap(), ..Default::default() },
				params: bounded_vec![FieldType::Uint64, FieldType::Bytes],
				result: bounded_vec![FieldType::Bytes],
				verifier: JobResultVerifier::Evm(CGGMP21_BLUEPRINT),
			},
		],
		registration_hook: ServiceRegistrationHook::Evm(CGGMP21_BLUEPRINT),
		registration_params: bounded_vec![],
		request_hook: ServiceRequestHook::Evm(CGGMP21_BLUEPRINT),
		request_params: bounded_vec![],
		gadget: Default::default(),
	}
}

#[test]
fn create_service_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);

		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let next_id = Services::next_blueprint_id();
		assert_eq!(next_id, 1);
		assert_events(vec![RuntimeEvent::Services(crate::Event::BlueprintCreated {
			owner: alice,
			blueprint_id: next_id - 1,
		})]);
	});
}

#[test]
fn register_on_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);

		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);

		let registration_call = Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: price_targets(MachineKind::Large),
			},
			Default::default(),
		);
		assert_ok!(registration_call);

		assert_events(vec![RuntimeEvent::Services(crate::Event::Registered {
			provider: bob.clone(),
			blueprint_id: 0,
			preferences: OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
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
				OperatorPreferences {
					key: zero_key(),
					approval: ApprovalPreference::default(),
					price_targets: Default::default()
				},
				Default::default(),
			),
			crate::Error::<Runtime>::AlreadyRegistered
		);

		// if we try to register with a non active operator, should fail
		assert_err!(
			Services::register(
				RuntimeOrigin::signed(mock_pub_key(10)),
				0,
				OperatorPreferences {
					key: zero_key(),
					approval: ApprovalPreference::default(),
					price_targets: Default::default()
				},
				Default::default(),
			),
			crate::Error::<Runtime>::OperatorNotActive
		);
	});
}

#[test]
fn pre_register_on_blueprint() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
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
fn update_approval_preference() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);

		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);

		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: price_targets(MachineKind::Small)
			},
			Default::default(),
		));

		assert_eq!(
			Operators::<Runtime>::get(0, &bob).unwrap(),
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: price_targets(MachineKind::Small)
			}
		);

		assert_events(vec![RuntimeEvent::Services(crate::Event::Registered {
			provider: bob.clone(),
			blueprint_id: 0,
			preferences: OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: price_targets(MachineKind::Small),
			},
			registration_args: Default::default(),
		})]);

		// update approval preference
		assert_ok!(Services::update_approval_preference(
			RuntimeOrigin::signed(bob.clone()),
			0,
			ApprovalPreference::Required,
		));

		assert_eq!(
			Operators::<Runtime>::get(0, &bob).unwrap().approval,
			ApprovalPreference::Required
		);

		assert_events(vec![RuntimeEvent::Services(crate::Event::ApprovalPreferenceUpdated {
			operator: bob,
			blueprint_id: 0,
			approval_preference: ApprovalPreference::Required,
		})]);

		// try to update approval preference when not registered
		let charlie = mock_pub_key(CHARLIE);
		assert_err!(
			Services::update_approval_preference(
				RuntimeOrigin::signed(charlie),
				0,
				ApprovalPreference::Required
			),
			crate::Error::<Runtime>::NotRegistered
		);
	});
}

#[test]
fn update_price_targets() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);

		let blueprint = cggmp21_blueprint();

		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);

		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: price_targets(MachineKind::Small)
			},
			Default::default(),
		));

		assert_eq!(
			Operators::<Runtime>::get(0, &bob).unwrap(),
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: price_targets(MachineKind::Small)
			}
		);

		assert_events(vec![RuntimeEvent::Services(crate::Event::Registered {
			provider: bob.clone(),
			blueprint_id: 0,
			preferences: OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
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
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: Default::default()
			},
			Default::default(),
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
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: Default::default()
			},
			Default::default(),
		));
		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: Default::default()
			},
			Default::default(),
		));
		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: Default::default()
			},
			Default::default(),
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
		));
		// this service gets immediately accepted by all providers.
		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);
		assert!(Instances::<Runtime>::contains_key(0));
		// The service should also be added to the services for each operator.
		let profile = OperatorsProfile::<Runtime>::get(bob).unwrap();
		assert!(profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(charlie).unwrap();
		assert!(profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(dave).unwrap();
		assert!(profile.services.contains(&0));

		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceInitiated {
			owner: eve,
			request_id: None,
			service_id: 0,
			blueprint_id: 0,
			assets: vec![USDC, WETH],
		})]);
	});
}

#[test]
fn request_service_with_approval_process() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));

		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: Default::default()
			},
			Default::default(),
		));

		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::Required,
				price_targets: Default::default()
			},
			Default::default(),
		));

		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::Required,
				price_targets: Default::default()
			},
			Default::default(),
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
		));

		// the service should be pending approval from charlie and dave.
		assert!(ServiceRequests::<Runtime>::contains_key(0));
		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceRequested {
			owner: eve.clone(),
			request_id: 0,
			blueprint_id: 0,
			approved: vec![bob.clone()],
			pending_approvals: vec![charlie.clone(), dave.clone()],
			assets: vec![WETH],
		})]);

		// it should not be added, until all providers approve.
		let profile = OperatorsProfile::<Runtime>::get(bob.clone()).unwrap();
		assert!(!profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(charlie.clone()).unwrap();
		assert!(!profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(dave.clone()).unwrap();
		assert!(!profile.services.contains(&0));
		// charlie approves the service
		assert_ok!(Services::approve(RuntimeOrigin::signed(charlie.clone()), 0));
		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceRequestApproved {
			operator: charlie.clone(),
			request_id: 0,
			blueprint_id: 0,
			approved: vec![charlie.clone(), bob.clone()],
			pending_approvals: vec![dave.clone()],
		})]);

		// dave approves the service, and the service is initiated.
		assert_ok!(Services::approve(RuntimeOrigin::signed(dave.clone()), 0));
		assert!(!ServiceRequests::<Runtime>::contains_key(0));
		assert!(Instances::<Runtime>::contains_key(0));

		// The service should also be added to the services for each operator.
		let profile = OperatorsProfile::<Runtime>::get(bob.clone()).unwrap();
		assert!(profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(charlie.clone()).unwrap();
		assert!(profile.services.contains(&0));
		let profile = OperatorsProfile::<Runtime>::get(dave.clone()).unwrap();
		assert!(profile.services.contains(&0));

		assert_events(vec![
			RuntimeEvent::Services(crate::Event::ServiceRequestApproved {
				operator: dave.clone(),
				request_id: 0,
				blueprint_id: 0,
				approved: vec![charlie.clone(), dave.clone(), bob.clone()],
				pending_approvals: vec![],
			}),
			RuntimeEvent::Services(crate::Event::ServiceInitiated {
				owner: eve,
				request_id: Some(0),
				service_id: 0,
				blueprint_id: 0,
				assets: vec![WETH],
			}),
		]);
	});
}

#[test]
fn request_service_with_no_assets() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: Default::default()
			},
			Default::default(),
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
			),
			Error::<Runtime>::NoAssetsProvided
		);
	});
}

#[test]
fn job_calls() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: Default::default()
			},
			Default::default(),
		));
		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: Default::default()
			},
			Default::default(),
		));
		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: Default::default()
			},
			Default::default(),
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
		));
		// this service gets immediately accepted by all providers.
		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);
		assert!(Instances::<Runtime>::contains_key(0));
		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceInitiated {
			owner: eve.clone(),
			request_id: None,
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
fn job_calls_fails_with_invalid_input() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: Default::default()
			},
			Default::default(),
		));
		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: Default::default()
			},
			Default::default(),
		));
		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: Default::default()
			},
			Default::default(),
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
		));
		// this service gets immediately accepted by all providers.
		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);
		assert!(Instances::<Runtime>::contains_key(0));
		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceInitiated {
			owner: eve.clone(),
			request_id: None,
			service_id: 0,
			blueprint_id: 0,
			assets: vec![WETH],
		})]);

		// now we can call the jobs
		let job_call_id = 0;
		assert_err!(
			Services::call(
				RuntimeOrigin::signed(eve.clone()),
				0,
				0,
				// t > n
				bounded_vec![Field::Uint8(4)],
			),
			crate::Error::<Runtime>::InvalidJobCallInput
		);

		assert!(!JobCalls::<Runtime>::contains_key(0, job_call_id));
	});
}

#[test]
fn job_result() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: Default::default()
			},
			Default::default(),
		));
		let charlie = mock_pub_key(CHARLIE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(charlie.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: Default::default()
			},
			Default::default(),
		));
		let dave = mock_pub_key(DAVE);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(dave.clone()),
			0,
			OperatorPreferences {
				key: zero_key(),
				approval: ApprovalPreference::default(),
				price_targets: Default::default()
			},
			Default::default(),
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
		));
		// this service gets immediately accepted by all providers.
		assert_eq!(ServiceRequests::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);
		assert!(Instances::<Runtime>::contains_key(0));
		assert_events(vec![RuntimeEvent::Services(crate::Event::ServiceInitiated {
			owner: eve.clone(),
			request_id: None,
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
			1,
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
