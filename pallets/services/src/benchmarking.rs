use crate::{Call, Config, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use parity_scale_codec::Decode;
use sp_core::{ecdsa, H160};
use sp_runtime::KeyTypeId;
use sp_std::vec;
use tangle_primitives::services::*;

const CGGMP21_BLUEPRINT: H160 = H160([0x21; 20]);

fn zero_key() -> ecdsa::Public {
	ecdsa::Public::from([0; 33])
}

fn mock_account_id<T: Config>(id: u8) -> T::AccountId {
	T::AccountId::decode(&mut &[id; 32][..]).unwrap()
}

fn operator_preferences<T: Config>() -> OperatorPreferences {
	OperatorPreferences {
		key: zero_key(),
		approval: ApprovalPreference::default(),
		price_targets: Default::default(),
	}
}

fn cggmp21_blueprint<T: Config>() -> ServiceBlueprint<T::Constraints> {
	ServiceBlueprint {
		metadata: ServiceMetadata { name: "CGGMP21 TSS".try_into().unwrap(), ..Default::default() },
		jobs: vec![
			JobDefinition {
				metadata: JobMetadata { name: "keygen".try_into().unwrap(), ..Default::default() },
				params: vec![FieldType::Uint8].try_into().unwrap(),
				result: vec![FieldType::Bytes].try_into().unwrap(),
				verifier: JobResultVerifier::Evm(CGGMP21_BLUEPRINT),
			},
			JobDefinition {
				metadata: JobMetadata { name: "sign".try_into().unwrap(), ..Default::default() },
				params: vec![FieldType::Uint64, FieldType::Bytes].try_into().unwrap(),
				result: vec![FieldType::Bytes].try_into().unwrap(),
				verifier: JobResultVerifier::Evm(CGGMP21_BLUEPRINT),
			},
		]
		.try_into()
		.unwrap(),
		registration_hook: ServiceRegistrationHook::Evm(CGGMP21_BLUEPRINT),
		registration_params: vec![].try_into().unwrap(),
		request_hook: ServiceRequestHook::Evm(CGGMP21_BLUEPRINT),
		request_params: vec![].try_into().unwrap(),
		gadget: Default::default(),
	}
}

benchmarks! {

	create_blueprint {
		let alice = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
	}: _(RawOrigin::Signed(alice.clone()), blueprint)

	pre_register {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let bob: T::AccountId =  mock_account_id::<T>(2u8);

	}: _(RawOrigin::Signed(bob.clone()), 0)


	register {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let operator_preference = operator_preferences::<T>();

	}: _(RawOrigin::Signed(bob.clone()), 0, operator_preference, Default::default())


	unregister {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let operator_preference = operator_preferences::<T>();

		let _= Pallet::<T>::register(RawOrigin::Signed(bob.clone()).into(), 0, operator_preference, Default::default());

	}: _(RawOrigin::Signed(bob.clone()), 0)


	update_approval_preference {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let operator_preference = operator_preferences::<T>();

		let _= Pallet::<T>::register(RawOrigin::Signed(bob.clone()).into(), 0, operator_preference, Default::default());

	}: _(RawOrigin::Signed(bob.clone()), 0, ApprovalPreference::Required)


	update_price_targets {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let operator_preference = operator_preferences::<T>();
		let price_targets = Default::default();

		let _= Pallet::<T>::register(RawOrigin::Signed(bob.clone()).into(), 0, operator_preference, Default::default());

	}: _(RawOrigin::Signed(bob.clone()), 0, price_targets)


	request {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let operator_preference = operator_preferences::<T>();
		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(bob.clone()).into(), 0, operator_preference, Default::default());

		let charlie: T::AccountId =  mock_account_id::<T>(3u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(charlie.clone()).into(), 0, operator_preference, Default::default());

		let dave: T::AccountId =  mock_account_id::<T>(4u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(dave.clone()).into(), 0, operator_preference, Default::default());

		let eve: T::AccountId =  mock_account_id::<T>(5u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(eve.clone()).into(), 0, operator_preference, Default::default());

	}: _(
			RawOrigin::Signed(eve.clone()),
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			100u32.into(),
			Default::default()
		)

	approve {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let bob: T::AccountId =  mock_account_id::<T>(2u8);

		let operator_preference = operator_preferences::<T>();
		let _= Pallet::<T>::register(
			RawOrigin::Signed(bob.clone()).into(),
			0,
			operator_preference,
			Default::default()
		);

		let operator_preference = OperatorPreferences { approval: ApprovalPreference::Required, ..operator_preference };

		let charlie: T::AccountId =  mock_account_id::<T>(3u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(charlie.clone()).into(), 0, operator_preference, Default::default());

		let dave: T::AccountId =  mock_account_id::<T>(4u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(dave.clone()).into(), 0, operator_preference, Default::default());

		let eve: T::AccountId =  mock_account_id::<T>(5u8);
		let _= Pallet::<T>::request(
			RawOrigin::Signed(eve.clone()).into(),
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			100u32.into(),
			Default::default()
		);

	}: _(RawOrigin::Signed(charlie.clone()), 0)


	reject {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let operator_preference = operator_preferences::<T>();
		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(bob.clone()).into(),
			0,
			operator_preference,
			Default::default()
		);

		let operator_preference = OperatorPreferences { approval: ApprovalPreference::Required, ..operator_preference };

		let charlie: T::AccountId =  mock_account_id::<T>(3u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(charlie.clone()).into(), 0, operator_preference, Default::default());

		let dave: T::AccountId =  mock_account_id::<T>(4u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(dave.clone()).into(), 0, operator_preference, Default::default());

		let eve: T::AccountId =  mock_account_id::<T>(5u8);
		let _= Pallet::<T>::request(
			RawOrigin::Signed(eve.clone()).into(),
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			100u32.into(),
			Default::default()
		);

	}: _(RawOrigin::Signed(charlie.clone()), 0)


	terminate {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let operator_preference = operator_preferences::<T>();

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(bob.clone()).into(), 0, operator_preference, Default::default());

		let charlie: T::AccountId =  mock_account_id::<T>(3u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(charlie.clone()).into(), 0, operator_preference, Default::default());

		let dave: T::AccountId =  mock_account_id::<T>(4u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(dave.clone()).into(), 0, operator_preference, Default::default());

		let eve: T::AccountId =  mock_account_id::<T>(5u8);
		let _= Pallet::<T>::request(
			RawOrigin::Signed(eve.clone()).into(),
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			100u32.into(),
			Default::default()
		);

	}: _(RawOrigin::Signed(eve.clone()),0)


	call {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);


		let operator_preference = operator_preferences::<T>();

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(bob.clone()).into(), 0, operator_preference, Default::default());

		let charlie: T::AccountId =  mock_account_id::<T>(3u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(charlie.clone()).into(), 0, operator_preference, Default::default());

		let dave: T::AccountId =  mock_account_id::<T>(4u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(dave.clone()).into(), 0, operator_preference, Default::default());

		let eve: T::AccountId =  mock_account_id::<T>(5u8);
		let _= Pallet::<T>::request(
			RawOrigin::Signed(eve.clone()).into(),
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			100u32.into(),
			Default::default()
		);

	}: _(
			RawOrigin::Signed(eve.clone()),
			0,
			0,
			vec![Field::Uint8(2)].try_into().unwrap()
		)


	submit_result {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let operator_preference = operator_preferences::<T>();

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(bob.clone()).into(), 0, operator_preference, Default::default());

		let charlie: T::AccountId =  mock_account_id::<T>(3u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(charlie.clone()).into(), 0, operator_preference, Default::default());

		let dave: T::AccountId =  mock_account_id::<T>(4u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(dave.clone()).into(), 0, operator_preference, Default::default());

		let eve: T::AccountId =  mock_account_id::<T>(5u8);
		let _= Pallet::<T>::request(
			RawOrigin::Signed(eve.clone()).into(),
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			100u32.into(),
			Default::default()
		);

		let _= Pallet::<T>::call(
			RawOrigin::Signed(eve.clone()).into(),
			0,
			0,
			vec![Field::Uint8(2)].try_into().unwrap()
		);

		let keygen_job_call_id = 0;
		let key_type = KeyTypeId(*b"mdkg");
		let dkg = sp_io::crypto::ecdsa_generate(key_type, None);

	}: _(
			RawOrigin::Signed(bob.clone()),
			0,
			keygen_job_call_id,
			vec![Field::Bytes(dkg.0.to_vec().try_into().unwrap())].try_into().unwrap()
		)

}

// Define the module and associated types for the benchmarks
impl_benchmark_test_suite!(
	Pallet,
	crate::mock::new_test_ext(vec![1, 2, 3, 4]),
	crate::mock::Runtime,
);
