use crate::{Call, Config, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use parity_scale_codec::Decode;
use scale_info::prelude::boxed::Box;
use sp_core::{H160, Pair};
use sp_runtime::{KeyTypeId, Percent};
use sp_std::{prelude::*, vec};
use tangle_primitives::services::{
	Asset, AssetSecurityCommitment, AssetSecurityRequirement, BlueprintServiceManager,
	BoundedString, Field, FieldType, JobDefinition, JobMetadata,
	MasterBlueprintServiceManagerRevision, MembershipModel, MembershipModelType,
	OperatorPreferences, ServiceBlueprint, ServiceMetadata,
};

pub type AssetId = u32;
const CGGMP21_BLUEPRINT: H160 = H160([0x21; 20]);
pub const TNT: AssetId = 0;
pub const USDC: AssetId = 1;
pub const WETH: AssetId = 2;
pub const WBTC: AssetId = 3;

pub(crate) fn get_security_requirement<T: Config>(
	a: T::AssetId,
	p: &[u8; 2],
) -> AssetSecurityRequirement<<T as Config>::AssetId> {
	AssetSecurityRequirement {
		asset: Asset::Custom(a),
		min_exposure_percent: Percent::from_percent(p[0]),
		max_exposure_percent: Percent::from_percent(p[1]),
	}
}

pub(crate) fn get_security_commitment<T: Config>(
	a: T::AssetId,
	p: u8,
) -> AssetSecurityCommitment<T::AssetId> {
	AssetSecurityCommitment { asset: Asset::Custom(a), exposure_percent: Percent::from_percent(p) }
}

pub(crate) fn test_ecdsa_key() -> [u8; 65] {
	let seed = b"benchmarking_deterministic_key_seed_123";
	let ecdsa_key = sp_core::ecdsa::Pair::from_seed_slice(seed)
		.expect("Should be able to create key from seed");
	let secret = k256::ecdsa::SigningKey::from_slice(&ecdsa_key.seed())
		.expect("Should be able to create a secret key from a seed");
	let verifying_key = k256::ecdsa::VerifyingKey::from(secret);
	let public_key = verifying_key.to_encoded_point(false);
	public_key.to_bytes().to_vec().try_into().unwrap()
}

fn mock_account_id<T: Config>(id: u8) -> T::AccountId {
	T::AccountId::decode(&mut &[id; 32][..]).unwrap()
}

fn operator_preferences<T: Config>() -> OperatorPreferences<T::Constraints> {
	OperatorPreferences {
		key: test_ecdsa_key(),
		rpc_address: BoundedString::try_from("https://example.com/rpc".to_owned()).unwrap(),
	}
}

fn cggmp21_blueprint<T: Config>() -> ServiceBlueprint<<T as Config>::Constraints> {
	#[allow(deprecated)]
	ServiceBlueprint {
		metadata: ServiceMetadata {
			name: "CGGMP21 TSS".to_owned().try_into().unwrap(),
			..Default::default()
		},
		manager: BlueprintServiceManager::Evm(CGGMP21_BLUEPRINT),
		master_manager_revision: MasterBlueprintServiceManagerRevision::Latest,
		jobs: vec![
			JobDefinition {
				metadata: JobMetadata {
					name: "keygen".to_owned().try_into().unwrap(),
					..Default::default()
				},
				params: vec![FieldType::Uint8].try_into().unwrap(),
				result: vec![FieldType::List(Box::new(FieldType::Uint8))].try_into().unwrap(),
			},
			JobDefinition {
				metadata: JobMetadata {
					name: "sign".to_owned().try_into().unwrap(),
					..Default::default()
				},
				params: vec![FieldType::Uint64, FieldType::List(Box::new(FieldType::Uint8))]
					.try_into()
					.unwrap(),
				result: vec![FieldType::List(Box::new(FieldType::Uint8))].try_into().unwrap(),
			},
		]
		.try_into()
		.unwrap(),
		registration_params: Default::default(),
		request_params: Default::default(),
		sources: Default::default(),
		supported_membership_models: vec![MembershipModelType::Fixed, MembershipModelType::Dynamic]
			.try_into()
			.unwrap(),
		recommended_resources: Default::default(),
	}
}

benchmarks! {

	where_clause {
		where
			T::AssetId: From<u32>,
	}

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

	}: _(RawOrigin::Signed(bob.clone()), 0, operator_preference.clone(), Default::default(), 0_u32.into())


	unregister {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let operator_preference = operator_preferences::<T>();

		let _= Pallet::<T>::register(
			RawOrigin::Signed(bob.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

	}: _(RawOrigin::Signed(bob.clone()), 0)

	update_rpc_address {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let operator_preference = operator_preferences::<T>();
		let rpc_address = BoundedString::try_from("https://example.com/rpc".to_owned()).unwrap();

		let _= Pallet::<T>::register(
			RawOrigin::Signed(bob.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

	}: _(RawOrigin::Signed(bob.clone()), 0, rpc_address)


	request {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let operator_preference = operator_preferences::<T>();
		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(bob.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let charlie: T::AccountId =  mock_account_id::<T>(3u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(charlie.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let dave: T::AccountId =  mock_account_id::<T>(4u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(dave.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let eve: T::AccountId =  mock_account_id::<T>(5u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(eve.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

	}: _(
		RawOrigin::Signed(bob.clone()),
		None,
		0,
		vec![alice.clone()],
		vec![bob.clone(), charlie.clone(), dave.clone()],
		Default::default(),
		vec![
			get_security_requirement::<T>(USDC.into(), &[10, 20]),
			get_security_requirement::<T>(WETH.into(), &[10, 20])
		],
		100_u32.into(),
		Asset::Custom(USDC.into()),
		0_u32.into(),
		MembershipModel::Fixed { min_operators: 3 }
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
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let charlie: T::AccountId =  mock_account_id::<T>(3u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(charlie.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let dave: T::AccountId =  mock_account_id::<T>(4u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(dave.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let eve: T::AccountId =  mock_account_id::<T>(5u8);
		let _= Pallet::<T>::request(
			RawOrigin::Signed(eve.clone()).into(),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			Default::default(),
			vec![
				get_security_requirement::<T>(USDC.into(), &[10, 20]),
				get_security_requirement::<T>(WETH.into(), &[10, 20])
			],
			100_u32.into(),
			Asset::Custom(USDC.into()),
			0_u32.into(),
			MembershipModel::Fixed { min_operators: 3 },
		);

	}: _(RawOrigin::Signed(charlie.clone()), 0, vec![
		get_security_commitment::<T>(USDC.into(), 10),
		get_security_commitment::<T>(WETH.into(), 10),
	])


	reject {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let operator_preference = operator_preferences::<T>();
		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(bob.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let charlie: T::AccountId =  mock_account_id::<T>(3u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(charlie.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let dave: T::AccountId =  mock_account_id::<T>(4u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(dave.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let eve: T::AccountId =  mock_account_id::<T>(5u8);
		let _= Pallet::<T>::request(
			RawOrigin::Signed(eve.clone()).into(),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			Default::default(),
			vec![
				get_security_requirement::<T>(USDC.into(), &[10, 20]),
				get_security_requirement::<T>(WETH.into(), &[10, 20])
			],
			100_u32.into(),
			Asset::Custom(USDC.into()),
			0_u32.into(),
			MembershipModel::Fixed { min_operators: 3 },
		);

	}: _(RawOrigin::Signed(charlie.clone()), 0)


	terminate {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let operator_preference = operator_preferences::<T>();

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(bob.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let charlie: T::AccountId =  mock_account_id::<T>(3u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(charlie.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let dave: T::AccountId =  mock_account_id::<T>(4u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(dave.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let eve: T::AccountId =  mock_account_id::<T>(5u8);
		let _= Pallet::<T>::request(
			RawOrigin::Signed(eve.clone()).into(),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			Default::default(),
			vec![
				get_security_requirement::<T>(USDC.into(), &[10, 20]),
				get_security_requirement::<T>(WETH.into(), &[10, 20])
			],
			100_u32.into(),
			Asset::Custom(USDC.into()),
			0_u32.into(),
			MembershipModel::Fixed { min_operators: 3 },
		);

	}: _(RawOrigin::Signed(eve.clone()),0)


	call {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);


		let operator_preference = operator_preferences::<T>();

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(bob.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let charlie: T::AccountId =  mock_account_id::<T>(3u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(charlie.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let dave: T::AccountId =  mock_account_id::<T>(4u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(dave.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let eve: T::AccountId =  mock_account_id::<T>(5u8);
		let _= Pallet::<T>::request(
			RawOrigin::Signed(eve.clone()).into(),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			Default::default(),
			vec![
				get_security_requirement::<T>(USDC.into(), &[10, 20]),
				get_security_requirement::<T>(WETH.into(), &[10, 20])
			],
			100_u32.into(),
			Asset::Custom(USDC.into()),
			0_u32.into(),
			MembershipModel::Fixed { min_operators: 3 },
		);

	}: _(
			RawOrigin::Signed(eve.clone()),
			0,
			0,
			vec![Field::Uint8(2)].try_into().unwrap()
		)

	submit_result {
		use sp_core::ByteArray;

		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let operator_preference = operator_preferences::<T>();

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(bob.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let charlie: T::AccountId =  mock_account_id::<T>(3u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(charlie.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let dave: T::AccountId =  mock_account_id::<T>(4u8);
		let _= Pallet::<T>::register(
			RawOrigin::Signed(dave.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		let eve: T::AccountId =  mock_account_id::<T>(5u8);
		let _= Pallet::<T>::request(
			RawOrigin::Signed(eve.clone()).into(),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone(), dave.clone()],
			Default::default(),
			vec![
				get_security_requirement::<T>(USDC.into(), &[10, 20]),
				get_security_requirement::<T>(WETH.into(), &[10, 20])
			],
			100_u32.into(),
			Asset::Custom(USDC.into()),
			0_u32.into(),
			MembershipModel::Fixed { min_operators: 3 },
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
			vec![Field::from(BoundedVec::try_from(dkg.to_raw_vec()).unwrap())].try_into().unwrap()
		)

	// Slash an operator's stake for a service
	slash {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let operator_preference = operator_preferences::<T>();
		let _= Pallet::<T>::register(
			RawOrigin::Signed(bob.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		// Create a service instance for bob
		let _= Pallet::<T>::request(
			RawOrigin::Signed(alice.clone()).into(),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement::<T>(USDC.into(), &[10, 20])],
			100_u32.into(),
			Asset::Custom(USDC.into()),
			0_u32.into(),
			MembershipModel::Fixed { min_operators: 1 }
		);

	}: _(RawOrigin::Signed(alice.clone()), bob.clone(), 0, Percent::from_percent(50))

	// Dispute a scheduled slash
	dispute {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let operator_preference = operator_preferences::<T>();
		let _= Pallet::<T>::register(
			RawOrigin::Signed(bob.clone()).into(),
			0,
			operator_preference.clone(),
			Default::default(),
			0_u32.into()
		);

		// Create a service instance and slash bob
		let _= Pallet::<T>::request(
			RawOrigin::Signed(alice.clone()).into(),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement::<T>(USDC.into(), &[10, 20])],
			100_u32.into(),
			Asset::Custom(USDC.into()),
			0_u32.into(),
			MembershipModel::Fixed { min_operators: 1 }
		);

		let _= Pallet::<T>::slash(RawOrigin::Signed(alice.clone()).into(), bob.clone(), 0, Percent::from_percent(50));

	}: _(RawOrigin::Signed(alice.clone()), 0, 0)

	// Update master blueprint service manager
	update_master_blueprint_service_manager {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
	}: _(RawOrigin::Root, H160::zero())

	// Join a service as an operator
	join_service {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let operator_preference = operator_preferences::<T>();
		let _= Pallet::<T>::register(RawOrigin::Signed(bob.clone()).into(), 0, operator_preference, Default::default(), 0_u32.into());

		// Create a service instance
		let _= Pallet::<T>::request(
			RawOrigin::Signed(alice.clone()).into(),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement::<T>(USDC.into(), &[10, 20])],
			100_u32.into(),
			Asset::Custom(USDC.into()),
			0_u32.into(),
			MembershipModel::Fixed { min_operators: 1 }
		);

		let charlie: T::AccountId =  mock_account_id::<T>(3u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(charlie.clone()).into(), 0, operator_preference, Default::default(), 0_u32.into());

	}: _(RawOrigin::Signed(charlie.clone()), 0, vec![get_security_commitment::<T>(USDC.into(), 10)])

	// Leave a service as an operator
	leave_service {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let operator_preference = operator_preferences::<T>();
		let _= Pallet::<T>::register(RawOrigin::Signed(bob.clone()).into(), 0, operator_preference, Default::default(), 0_u32.into());

		let charlie: T::AccountId =  mock_account_id::<T>(3u8);
		let _= Pallet::<T>::register(RawOrigin::Signed(charlie.clone()).into(), 0, operator_preference, Default::default(), 0_u32.into());

		// Create a service instance with dynamic membership
		let _= Pallet::<T>::request(
			RawOrigin::Signed(alice.clone()).into(),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone(), charlie.clone()],
			Default::default(),
			vec![get_security_requirement::<T>(USDC.into(), &[10, 20])],
			100_u32.into(),
			Asset::Custom(USDC.into()),
			0_u32.into(),
			MembershipModel::Dynamic { min_operators: 1, max_operators: Some(3) }
		);

	}: _(RawOrigin::Signed(charlie.clone()), 0)

	// Benchmark payment validation for pay-once services
	validate_payment_amount_pay_once {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);
		
		let (_, blueprint) = Pallet::<T>::blueprints(0).unwrap();
		let amount = 1000_u32.into();
	}: {
		let _ = Pallet::<T>::validate_payment_amount(&blueprint, amount);
	}

	// Benchmark payment processing for subscription services
	process_subscription_payment {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let operator_preference = operator_preferences::<T>();
		let _= Pallet::<T>::register(RawOrigin::Signed(bob.clone()).into(), 0, operator_preference, Default::default(), 0_u32.into());

		// Create a service instance
		let _= Pallet::<T>::request(
			RawOrigin::Signed(alice.clone()).into(),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement::<T>(USDC.into(), &[10, 20])],
			100_u32.into(),
			Asset::Custom(USDC.into()),
			0_u32.into(),
			MembershipModel::Fixed { min_operators: 1 }
		);

		let service_id = 0;
		let current_block = 100_u32.into();
	}: {
		let _ = Pallet::<T>::process_service_payment(service_id, current_block);
	}

	// Benchmark event-driven payment processing
	process_event_driven_payment {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let operator_preference = operator_preferences::<T>();
		let _= Pallet::<T>::register(RawOrigin::Signed(bob.clone()).into(), 0, operator_preference, Default::default(), 0_u32.into());

		// Create a service instance
		let _= Pallet::<T>::request(
			RawOrigin::Signed(alice.clone()).into(),
			None,
			0,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement::<T>(USDC.into(), &[10, 20])],
			100_u32.into(),
			Asset::Custom(USDC.into()),
			0_u32.into(),
			MembershipModel::Fixed { min_operators: 1 }
		);

		let service_id = 0;
		let event_count = 5;
	}: {
		let _ = Pallet::<T>::process_event_driven_payment(service_id, event_count);
	}

	// Benchmark subscription payments processing on block
	process_subscription_payments_on_block {
		let alice: T::AccountId = mock_account_id::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		let _= Pallet::<T>::create_blueprint(RawOrigin::Signed(alice.clone()).into(), blueprint);

		let bob: T::AccountId =  mock_account_id::<T>(2u8);
		let operator_preference = operator_preferences::<T>();
		let _= Pallet::<T>::register(RawOrigin::Signed(bob.clone()).into(), 0, operator_preference, Default::default(), 0_u32.into());

		// Create multiple service instances to test batch processing
		for i in 0..5 {
			let requester: T::AccountId = mock_account_id::<T>((10 + i) as u8);
			let _= Pallet::<T>::request(
				RawOrigin::Signed(requester).into(),
				None,
				0,
				vec![alice.clone()],
				vec![bob.clone()],
				Default::default(),
				vec![get_security_requirement::<T>(USDC.into(), &[10, 20])],
				100_u32.into(),
				Asset::Custom(USDC.into()),
				0_u32.into(),
				MembershipModel::Fixed { min_operators: 1 }
			);
		}

		let current_block = 100_u32.into();
	}: {
		let _ = Pallet::<T>::process_subscription_payments_on_block(current_block);
	}
}

// Define the module and associated types for the benchmarks
impl_benchmark_test_suite!(
	Pallet,
	crate::mock::new_test_ext(vec![1, 2, 3, 4]),
	crate::mock::Runtime,
);
