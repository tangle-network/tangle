use core::ops::Mul;

use crate::{
	mock::*,
	mock_evm::{PCall, PrecompilesValue},
};
use frame_support::assert_ok;
use k256::ecdsa::{SigningKey, VerifyingKey};
use pallet_services::{types::ConstraintsOf, Instances};
use parity_scale_codec::Encode;
use precompile_utils::{prelude::UnboundedBytes, testing::*};
use sp_core::{ecdsa, Pair, H160, U256};
use sp_runtime::{bounded_vec, AccountId32, Percent};
use tangle_primitives::services::{
	Asset, AssetSecurityCommitment, AssetSecurityRequirement, BlueprintServiceManager,
	BoundedString, FieldType, JobDefinition, JobMetadata, MasterBlueprintServiceManagerRevision,
	MembershipModelType, OperatorPreferences, ServiceBlueprint, ServiceMetadata,
};

fn get_security_requirement(a: AssetId, p: &[u8; 2]) -> AssetSecurityRequirement<AssetId> {
	AssetSecurityRequirement {
		asset: Asset::Custom(a),
		min_exposure_percent: Percent::from_percent(p[0]),
		max_exposure_percent: Percent::from_percent(p[1]),
	}
}

fn get_security_commitment(a: AssetId, p: u8) -> AssetSecurityCommitment<AssetId> {
	AssetSecurityCommitment { asset: Asset::Custom(a), exposure_percent: Percent::from_percent(p) }
}

fn test_ecdsa_key() -> [u8; 65] {
	let (ecdsa_key, _) = ecdsa::Pair::generate();
	let secret = SigningKey::from_slice(&ecdsa_key.seed())
		.expect("Should be able to create a secret key from a seed");
	let verifying_key = VerifyingKey::from(secret);
	let public_key = verifying_key.to_encoded_point(false);
	public_key.to_bytes().to_vec().try_into().unwrap()
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
				result: bounded_vec![FieldType::List(Box::new(FieldType::Uint8))],
			},
			JobDefinition {
				metadata: JobMetadata { name: "sign".try_into().unwrap(), ..Default::default() },
				params: bounded_vec![
					FieldType::Uint64,
					FieldType::List(Box::new(FieldType::Uint8))
				],
				result: bounded_vec![FieldType::List(Box::new(FieldType::Uint8))],
			},
		],
		registration_params: bounded_vec![],
		request_params: bounded_vec![],
		sources: Default::default(),
		supported_membership_models: bounded_vec![
			MembershipModelType::Fixed,
			MembershipModelType::Dynamic,
		],
		recommended_resources: Default::default(),
	}
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	check_precompile_implements_solidity_interfaces(&["Services.sol"], PCall::supports_selector)
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
fn test_request_service() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
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

		// Register operator using pallet function
		let bob: AccountId32 = TestAccount::Bob.into();
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: test_ecdsa_key(),
				rpc_address: BoundedString::try_from("https://example.com/rpc".to_string())
					.unwrap()
			},
			Default::default(),
			0,
		));

		// Request service from EVM
		let permitted_callers_data: Vec<AccountId32> = vec![TestAccount::Alex.into()];
		let service_providers_data: Vec<AccountId32> = vec![bob.clone()];
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
					asset_security_requirements: vec![get_security_requirement(WETH, &[10, 20])]
						.into_iter()
						.map(|r| r.encode().into())
						.collect(),
					ttl: U256::from(1000),
					payment_asset_id: U256::from(0),
					payment_token_address: Default::default(),
					amount: U256::from(0),
					min_operators: 1,
					max_operators: u32::MAX,
				},
			)
			.execute_returns(());

		// Approve using pallet function
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			vec![get_security_commitment(WETH, 10), get_security_commitment(TNT, 10)],
		));

		// Ensure the service instance is created
		assert!(Instances::<Runtime>::contains_key(0));
	});
}

#[test]
fn test_request_service_with_erc20() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
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

		// Register operator using pallet function
		let bob: AccountId32 = TestAccount::Bob.into();
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: test_ecdsa_key(),
				rpc_address: BoundedString::try_from("https://example.com/rpc".to_string())
					.unwrap()
			},
			Default::default(),
			0,
		));

		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, Services::pallet_evm_account())
				.map(|(balance, _)| balance),
			U256::zero(),
		);

		let permitted_callers_data: Vec<AccountId32> = vec![TestAccount::Alex.into()];
		let service_providers_data: Vec<AccountId32> = vec![bob.clone()];
		let request_args_data = vec![0u8];

		let payment_amount = U256::from(5).mul(U256::from(10).pow(6.into())); // 5 USDC

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::request_service {
					blueprint_id: U256::from(0),
					permitted_callers_data: UnboundedBytes::from(permitted_callers_data.encode()),
					service_providers_data: UnboundedBytes::from(service_providers_data.encode()),
					request_args_data: UnboundedBytes::from(request_args_data),
					asset_security_requirements: vec![get_security_requirement(WETH, &[10, 20])]
						.into_iter()
						.map(|r| r.encode().into())
						.collect(),
					ttl: U256::from(1000),
					payment_asset_id: U256::from(0),
					payment_token_address: USDC_ERC20.into(),
					amount: payment_amount,
					min_operators: 1,
					max_operators: u32::MAX,
				},
			)
			.execute_returns(());

		// Services pallet address now should have 5 USDC
		assert_ok!(
			Services::query_erc20_balance_of(USDC_ERC20, Services::pallet_evm_account())
				.map(|(balance, _)| balance),
			payment_amount
		);

		// Approve using pallet function
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			vec![get_security_commitment(WETH, 10), get_security_commitment(TNT, 10)],
		));

		// Ensure the service instance is created
		assert!(Instances::<Runtime>::contains_key(0));
	});
}

#[test]
fn test_request_service_with_asset() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
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

		// Register operator using pallet function
		let bob: AccountId32 = TestAccount::Bob.into();
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: test_ecdsa_key(),
				rpc_address: BoundedString::try_from("https://example.com/rpc".to_string())
					.unwrap()
			},
			Default::default(),
			0,
		));

		assert_eq!(Assets::balance(USDC, Services::pallet_account()), 0);

		let permitted_callers_data: Vec<AccountId32> = vec![TestAccount::Alex.into()];
		let service_providers_data: Vec<AccountId32> = vec![bob.clone()];
		let request_args_data = vec![0u8];

		let payment_amount = U256::from(5).mul(U256::from(10).pow(6.into())); // 5 USDC

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::request_service {
					blueprint_id: U256::from(0),
					permitted_callers_data: UnboundedBytes::from(permitted_callers_data.encode()),
					service_providers_data: UnboundedBytes::from(service_providers_data.encode()),
					request_args_data: UnboundedBytes::from(request_args_data),
					asset_security_requirements: vec![get_security_requirement(WETH, &[10, 20])]
						.into_iter()
						.map(|r| r.encode().into())
						.collect(),
					ttl: U256::from(1000),
					payment_asset_id: U256::from(USDC),
					payment_token_address: Default::default(),
					amount: payment_amount,
					min_operators: 1,
					max_operators: u32::MAX,
				},
			)
			.execute_returns(());

		// Services pallet address now should have 5 USDC
		assert_eq!(Assets::balance(USDC, Services::pallet_account()), payment_amount.as_u128());

		// Approve using pallet function
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			vec![get_security_commitment(WETH, 10), get_security_commitment(TNT, 10)],
		));

		// Ensure the service instance is created
		assert!(Instances::<Runtime>::contains_key(0));
	});
}

#[test]
fn test_terminate_service() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(Services::update_master_blueprint_service_manager(RuntimeOrigin::root(), MBSM));
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

		// Register operator using pallet function
		let bob: AccountId32 = TestAccount::Bob.into();
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			0,
			OperatorPreferences {
				key: test_ecdsa_key(),
				rpc_address: BoundedString::try_from("https://example.com/rpc".to_string())
					.unwrap()
			},
			Default::default(),
			0,
		));

		let permitted_callers_data: Vec<AccountId32> = vec![TestAccount::Alex.into()];
		let service_providers_data: Vec<AccountId32> = vec![bob.clone()];
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
					asset_security_requirements: vec![get_security_requirement(WETH, &[10, 20])]
						.into_iter()
						.map(|r| r.encode().into())
						.collect(),
					ttl: U256::from(1000),
					payment_asset_id: U256::from(0),
					payment_token_address: Default::default(),
					amount: U256::from(0),
					min_operators: 1,
					max_operators: u32::MAX,
				},
			)
			.execute_returns(());

		// Approve using pallet function
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			0,
			vec![get_security_commitment(WETH, 10), get_security_commitment(TNT, 10)],
		));

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
