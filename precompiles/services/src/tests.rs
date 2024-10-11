use crate::mock::*;
use crate::mock_evm::PCall;
use crate::mock_evm::PrecompilesValue;
use pallet_services::types::ConstraintsOf;
use pallet_services::Instances;
use pallet_services::Operators;
use pallet_services::OperatorsProfile;
use parity_scale_codec::Encode;
use precompile_utils::prelude::UnboundedBytes;
use precompile_utils::testing::*;
use sp_core::ecdsa;
use sp_core::{H160, U256};
use sp_runtime::bounded_vec;
use sp_runtime::AccountId32;
use tangle_primitives::services::FieldType;
use tangle_primitives::services::JobDefinition;
use tangle_primitives::services::JobMetadata;
use tangle_primitives::services::JobResultVerifier;
use tangle_primitives::services::PriceTargets;
use tangle_primitives::services::ServiceMetadata;
use tangle_primitives::services::ServiceRegistrationHook;
use tangle_primitives::services::ServiceRequestHook;
use tangle_primitives::services::{ApprovalPreference, OperatorPreferences, ServiceBlueprint};

fn zero_key() -> ecdsa::Public {
	ecdsa::Public::from([0; 33])
}

const WETH: AssetId = 1;

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
fn test_create_blueprint() {
	ExtBuilder.build().execute_with(|| {
		// Create blueprint
		let blueprint_data = cggmp21_blueprint();

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::create_blueprint {
					blueprint_data: UnboundedBytes::from(blueprint_data.encode()),
				},
			)
			.execute_returns(());

		// Ensure the blueprint was created successfully
		assert_eq!(Services::next_blueprint_id(), 1);
	});
}

#[test]
fn test_register_operator() {
	ExtBuilder.build().execute_with(|| {
		// First create the blueprint
		let blueprint_data = cggmp21_blueprint();

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::create_blueprint {
					blueprint_data: UnboundedBytes::from(blueprint_data.encode()),
				},
			)
			.execute_returns(());

		// Now register operator
		let preferences_data = OperatorPreferences {
			key: zero_key(),
			approval: ApprovalPreference::default(),
			price_targets: price_targets(MachineKind::Large),
		}
		.encode();

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Bob,
				H160::from_low_u64_be(1),
				PCall::register_operator {
					blueprint_id: U256::from(0), // We use the first blueprint
					preferences: UnboundedBytes::from(preferences_data),
					registration_args: UnboundedBytes::from(vec![0u8]),
				},
			)
			.execute_returns(());

		// Check that the operator profile exists
		let account: AccountId32 = TestAccount::Bob.into();
		assert!(OperatorsProfile::<Runtime>::get(account).is_ok());
	});
}

#[test]
fn test_request_service() {
	ExtBuilder.build().execute_with(|| {
		// First create the blueprint
		let blueprint_data = cggmp21_blueprint();

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::create_blueprint {
					blueprint_data: UnboundedBytes::from(blueprint_data.encode()),
				},
			)
			.execute_returns(());

		// Now register operator
		let preferences_data = OperatorPreferences {
			key: zero_key(),
			approval: ApprovalPreference::default(),
			price_targets: price_targets(MachineKind::Large),
		}
		.encode();

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Bob,
				H160::from_low_u64_be(1),
				PCall::register_operator {
					blueprint_id: U256::from(0),
					preferences: UnboundedBytes::from(preferences_data),
					registration_args: UnboundedBytes::from(vec![0u8]),
				},
			)
			.execute_returns(());

		// Finally, request the service
		let permitted_callers_data: Vec<AccountId32> = vec![TestAccount::Alex.into()];
		let service_providers_data: Vec<AccountId32> = vec![TestAccount::Bob.into()];
		let request_args_data = vec![0u8];

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::request_service {
					blueprint_id: U256::from(0), // Use the first blueprint
					permitted_callers_data: UnboundedBytes::from(permitted_callers_data.encode()),
					service_providers_data: UnboundedBytes::from(service_providers_data.encode()),
					request_args_data: UnboundedBytes::from(request_args_data),
					assets: [WETH].into_iter().map(Into::into).collect(),
				},
			)
			.execute_returns(());

		// Ensure the service instance is created
		assert!(Instances::<Runtime>::contains_key(0));
	});
}

#[test]
fn test_unregister_operator() {
	ExtBuilder.build().execute_with(|| {
		// First register operator (after blueprint creation)
		let blueprint_data = cggmp21_blueprint();

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::create_blueprint {
					blueprint_data: UnboundedBytes::from(blueprint_data.encode()),
				},
			)
			.execute_returns(());

		let preferences_data = OperatorPreferences {
			key: zero_key(),
			approval: ApprovalPreference::default(),
			price_targets: price_targets(MachineKind::Large),
		}
		.encode();

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Bob,
				H160::from_low_u64_be(1),
				PCall::register_operator {
					blueprint_id: U256::from(0),
					preferences: UnboundedBytes::from(preferences_data),
					registration_args: UnboundedBytes::from(vec![0u8]),
				},
			)
			.execute_returns(());

		// Now unregister operator
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Bob,
				H160::from_low_u64_be(1),
				PCall::unregister_operator { blueprint_id: U256::from(0) },
			)
			.execute_returns(());

		// Ensure the operator is removed
		let bob_account: AccountId32 = TestAccount::Bob.into();
		assert!(!Operators::<Runtime>::contains_key(0, bob_account));
	});
}

#[test]
fn test_terminate_service() {
	ExtBuilder.build().execute_with(|| {
		// First request a service
		let blueprint_data = cggmp21_blueprint();

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::create_blueprint {
					blueprint_data: UnboundedBytes::from(blueprint_data.encode()),
				},
			)
			.execute_returns(());

		let preferences_data = OperatorPreferences {
			key: zero_key(),
			approval: ApprovalPreference::default(),
			price_targets: price_targets(MachineKind::Large),
		}
		.encode();

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Bob,
				H160::from_low_u64_be(1),
				PCall::register_operator {
					blueprint_id: U256::from(0),
					preferences: UnboundedBytes::from(preferences_data),
					registration_args: UnboundedBytes::from(vec![0u8]),
				},
			)
			.execute_returns(());

		let permitted_callers_data: Vec<AccountId32> = vec![TestAccount::Alex.into()];
		let service_providers_data: Vec<AccountId32> = vec![TestAccount::Bob.into()];
		let request_args_data = vec![0u8];

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::request_service {
					blueprint_id: U256::from(0),
					permitted_callers_data: UnboundedBytes::from(permitted_callers_data.encode()),
					service_providers_data: UnboundedBytes::from(service_providers_data.encode()),
					request_args_data: UnboundedBytes::from(request_args_data),
					assets: [WETH].into_iter().map(Into::into).collect(),
				},
			)
			.execute_returns(());

		// Now terminate the service
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::terminate_service { service_id: U256::from(0) },
			)
			.execute_returns(());

		// Ensure the service is removed
		assert!(!Instances::<Runtime>::contains_key(0));
	});
}
