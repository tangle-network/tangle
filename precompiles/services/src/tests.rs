use core::ops::Mul;

use crate::{
	mock::*,
	mock_evm::{PCall, PrecompilesValue},
};
use frame_support::assert_ok;
use k256::ecdsa::{SigningKey, VerifyingKey};
use pallet_services::{types::ConstraintsOf, Instances, Operators, OperatorsProfile};
use parity_scale_codec::Encode;
use precompile_utils::{prelude::UnboundedBytes, testing::*};
use sp_core::{ecdsa, Pair, H160, U256};
use sp_runtime::{bounded_vec, AccountId32};
use tangle_primitives::services::{
	BlueprintServiceManager, FieldType, JobDefinition, JobMetadata,
	MasterBlueprintServiceManagerRevision, OperatorPreferences, PriceTargets, ServiceBlueprint,
	ServiceMetadata,
};

fn test_ecdsa_key() -> [u8; 65] {
	let (ecdsa_key, _) = ecdsa::Pair::generate();
	let secret = SigningKey::from_slice(&ecdsa_key.seed())
		.expect("Should be able to create a secret key from a seed");
	let verifying_key = VerifyingKey::from(secret);
	let public_key = verifying_key.to_encoded_point(false);
	public_key.to_bytes().to_vec().try_into().unwrap()
}

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
fn test_create_blueprint() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
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
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
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
			key: test_ecdsa_key(),
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
					registration_args: UnboundedBytes::from(Vec::new()),
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
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
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
			key: test_ecdsa_key(),
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
					ttl: U256::from(1000),
					payment_asset_id: U256::from(0),
					payment_token_address: Default::default(),
					amount: U256::from(0),
				},
			)
			.execute_returns(());

		// Approve the service request by the operator(s)
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Bob,
				H160::from_low_u64_be(1),
				PCall::approve { request_id: U256::from(0), restaking_percent: 10 },
			)
			.execute_returns(());

		// Ensure the service instance is created
		assert!(Instances::<Runtime>::contains_key(0));
	});
}

#[test]
fn test_request_service_with_erc20() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
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
			key: test_ecdsa_key(),
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

		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, Services::address())
				.map(|(balance, _)| balance),
			U256::zero(),
		);
		// Finally, request the service
		let permitted_callers_data: Vec<AccountId32> = vec![TestAccount::Alex.into()];
		let service_providers_data: Vec<AccountId32> = vec![TestAccount::Bob.into()];
		let request_args_data = vec![0u8];

		let payment_amount = U256::from(5).mul(U256::from(10).pow(6.into())); // 5 USDC

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::request_service {
					blueprint_id: U256::from(0), // Use the first blueprint
					permitted_callers_data: UnboundedBytes::from(permitted_callers_data.encode()),
					service_providers_data: UnboundedBytes::from(service_providers_data.encode()),
					request_args_data: UnboundedBytes::from(request_args_data),
					assets: [TNT, WETH].into_iter().map(Into::into).collect(),
					ttl: U256::from(1000),
					payment_asset_id: U256::from(0),
					payment_token_address: USDC_ERC20.into(),
					amount: payment_amount,
				},
			)
			.execute_returns(());

		// Services pallet address now should have 5 USDC
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, Services::address())
				.map(|(balance, _)| balance),
			payment_amount
		);

		// Approve the service request by the operator(s)
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Bob,
				H160::from_low_u64_be(1),
				PCall::approve { request_id: U256::from(0), restaking_percent: 10 },
			)
			.execute_returns(());

		// Ensure the service instance is created
		assert!(Instances::<Runtime>::contains_key(0));
	});
}

#[test]
fn test_request_service_with_asset() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
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
			key: test_ecdsa_key(),
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

		assert_eq!(Assets::balance(USDC, Services::account_id()), 0);

		// Finally, request the service
		let permitted_callers_data: Vec<AccountId32> = vec![TestAccount::Alex.into()];
		let service_providers_data: Vec<AccountId32> = vec![TestAccount::Bob.into()];
		let request_args_data = vec![0u8];

		let payment_amount = U256::from(5).mul(U256::from(10).pow(6.into())); // 5 USDC

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::request_service {
					blueprint_id: U256::from(0), // Use the first blueprint
					permitted_callers_data: UnboundedBytes::from(permitted_callers_data.encode()),
					service_providers_data: UnboundedBytes::from(service_providers_data.encode()),
					request_args_data: UnboundedBytes::from(request_args_data),
					assets: [TNT, WETH].into_iter().map(Into::into).collect(),
					ttl: U256::from(1000),
					payment_asset_id: U256::from(USDC),
					payment_token_address: Default::default(),
					amount: payment_amount,
				},
			)
			.execute_returns(());

		// Services pallet address now should have 5 USDC
		assert_eq!(Assets::balance(USDC, Services::account_id()), payment_amount.as_u128());

		// Approve the service request by the operator(s)
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Bob,
				H160::from_low_u64_be(1),
				PCall::approve { request_id: U256::from(0), restaking_percent: 10 },
			)
			.execute_returns(());

		// Ensure the service instance is created
		assert!(Instances::<Runtime>::contains_key(0));
	});
}

#[test]
fn test_unregister_operator() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
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
			key: test_ecdsa_key(),
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
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
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
			key: test_ecdsa_key(),
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
					ttl: U256::from(1000),
					payment_asset_id: U256::from(0),
					payment_token_address: Default::default(),
					amount: U256::from(0),
				},
			)
			.execute_returns(());

		// Approve the service request by the operator(s)
		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Bob,
				H160::from_low_u64_be(1),
				PCall::approve { request_id: U256::from(0), restaking_percent: 10 },
			)
			.execute_returns(());

		assert!(Instances::<Runtime>::contains_key(0));

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
