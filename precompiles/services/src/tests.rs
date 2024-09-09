use crate::mock::*;
use crate::U256;
use frame_support::assert_ok;
use pallet_service_blueprint::{Blueprints, Operators, UserServices};
use precompile_utils::testing::*;
use sp_core::H160;

#[test]
fn test_selector_less_than_four_bytes_reverts() {
	ExtBuilder::default().build().execute_with(|| {
		PrecompilesValue::get()
			.prepare_test(Alice, Precompile1, vec![1u8, 2, 3])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn test_unimplemented_selector_reverts() {
	ExtBuilder::default().build().execute_with(|| {
		PrecompilesValue::get()
			.prepare_test(Alice, Precompile1, vec![1u8, 2, 3, 4])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn test_create_blueprint() {
	ExtBuilder::default().build().execute_with(|| {
		let account = sp_core::sr25519::Public::from(TestAccount::Alex);
		let initial_blueprint_count = Blueprints::<Runtime>::count();

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::create_blueprint {
					blueprint: vec![],
				},
			)
			.execute_returns(());

		assert_eq!(Blueprints::<Runtime>::count(), initial_blueprint_count + 1);
	});
}

#[test]
fn test_register_operator() {
	ExtBuilder::default().build().execute_with(|| {
		let account = sp_core::sr25519::Public::from(TestAccount::Alex);
		let blueprint_id = 0; // Assuming blueprint 0 is created in a previous test

		assert!(Operators::<Runtime>::get(blueprint_id, account).is_none());

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::register {
					blueprint_id: U256::from(blueprint_id),
					preferences: vec![],
					registration_args: vec![],
				},
			)
			.execute_returns(());

		assert!(Operators::<Runtime>::get(blueprint_id, account).is_some());
	});
}

#[test]
fn test_register_operator_already_registered_reverts() {
	ExtBuilder::default().build().execute_with(|| {
		let account = sp_core::sr25519::Public::from(TestAccount::Alex);
		let blueprint_id = 0;

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::register {
					blueprint_id: U256::from(blueprint_id),
					preferences: vec![],
					registration_args: vec![],
				},
			)
			.execute_returns(());

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::register {
					blueprint_id: U256::from(blueprint_id),
					preferences: vec![],
					registration_args: vec![],
				},
			)
			.execute_reverts(|output| output == b"Dispatched call failed with error: Module(ModuleError { index: 1, error: [1, 0, 0, 0], message: Some(\"AlreadyRegistered\") })");
	});
}

#[test]
fn test_request_service() {
	ExtBuilder::default().build().execute_with(|| {
		let account = sp_core::sr25519::Public::from(TestAccount::Alex);
		let blueprint_id = 0;

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::request_service {
					blueprint_id: U256::from(blueprint_id),
					permitted_callers: vec![],
					service_providers: vec![account.into()],
					ttl: U256::from(10),
					request_args: vec![],
				},
			)
			.execute_returns(());

		assert!(UserServices::<Runtime>::get(account).contains(&blueprint_id));
	});
}

#[test]
fn test_unregister_operator() {
	ExtBuilder::default().build().execute_with(|| {
		let account = sp_core::sr25519::Public::from(TestAccount::Alex);
		let blueprint_id = 0;

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::register {
					blueprint_id: U256::from(blueprint_id),
					preferences: vec![],
					registration_args: vec![],
				},
			)
			.execute_returns(());

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::unregister {
					blueprint_id: U256::from(blueprint_id),
				},
			)
			.execute_returns(());

		assert!(Operators::<Runtime>::get(blueprint_id, account).is_none());
	});
}

#[test]
fn test_terminate_service() {
	ExtBuilder::default().build().execute_with(|| {
		let account = sp_core::sr25519::Public::from(TestAccount::Alex);
		let service_id = 0; // Assuming service 0 was created previously

		PrecompilesValue::get()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::terminate_service {
					service_id: U256::from(service_id),
				},
			)
			.execute_returns(());

		// Verify the service is removed or terminated
		assert!(UserServices::<Runtime>::get(account).is_empty());
	});
}
