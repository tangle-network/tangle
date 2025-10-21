use super::*;
use crate::OriginFor;
use frame_benchmarking::v1::{benchmarks, impl_benchmark_test_suite};
use frame_support::{BoundedVec, assert_ok, traits::Currency};
use frame_system::RawOrigin;
use scale_info::prelude::boxed::Box;
use sp_core::{H160, crypto::Pair, ecdsa};
use sp_runtime::{
	KeyTypeId, Percent,
	traits::{SaturatedConversion, StaticLookup, Zero},
};
use sp_std::vec;
use tangle_primitives::services::{
	Asset, AssetSecurityCommitment, AssetSecurityRequirement, BlueprintServiceManager,
	BoundedString, Field, FieldType, JobDefinition, JobMetadata,
	MasterBlueprintServiceManagerRevision, MembershipModel, MembershipModelType,
	OperatorPreferences, PricingModel, ServiceBlueprint, ServiceMetadata,
};

pub type AssetId = u32;
pub type AssetIdOf<T> = <T as Config>::AssetId;
#[allow(dead_code)]
const CGGMP21_BLUEPRINT: H160 = H160([0x21; 20]);
#[allow(dead_code)]
pub const TNT: AssetId = 0;
pub const USDC: AssetId = 1;
pub const WETH: AssetId = 2;
pub const WBTC: AssetId = 3;

const NATIVE_BALANCE_TARGET: u128 = 1_000_000_000_000;
const CUSTOM_ASSET_BALANCE_TARGET: u128 = 1_000_000_000_000;
const ASSET_ADMIN_ID: u8 = 200;

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

fn derive_ecdsa_key(seed: [u8; 32]) -> [u8; 65] {
	let ecdsa_key = sp_core::ecdsa::Pair::from_seed(&seed);
	let secret = k256::ecdsa::SigningKey::from_slice(&ecdsa_key.seed())
		.expect("Should be able to create a secret key from a seed");
	let verifying_key = k256::ecdsa::VerifyingKey::from(secret);
	let public_key = verifying_key.to_encoded_point(false);
	public_key.to_bytes().to_vec().try_into().unwrap()
}

#[allow(dead_code)]
pub(crate) fn test_ecdsa_key() -> [u8; 65] {
	derive_ecdsa_key([1u8; 32])
}

fn bench_ecdsa_key(seed_byte: u8) -> [u8; 65] {
	let mut seed = [0u8; 32];
	seed.fill(seed_byte);
	seed[0] = seed_byte;
	seed[15] = seed_byte.wrapping_mul(7).wrapping_add(3);
	seed[31] = seed_byte.wrapping_mul(11).wrapping_add(1);
	derive_ecdsa_key(seed)
}

fn mock_account_id<T: Config>(id: u8) -> T::AccountId {
	frame_benchmarking::account("account", id as u32, 0)
}

fn asset_admin_account<T: Config>() -> T::AccountId {
	mock_account_id::<T>(ASSET_ADMIN_ID)
}

fn ensure_native_balance<T: Config>(account: &T::AccountId) {
	let target: BalanceOf<T> = NATIVE_BALANCE_TARGET.saturated_into();
	let current = T::Currency::free_balance(account);
	if current < target {
		let needed = target - current;
		if !needed.is_zero() {
			let _ = T::Currency::deposit_creating(account, needed);
		}
	}
}

fn ensure_asset_exists<T>(asset: u32)
where
	T: Config + pallet_assets::Config<pallet_assets::Instance1, AssetId = AssetIdOf<T>>,
{
	let asset_id: AssetIdOf<T> = asset.into();
	if pallet_assets::Pallet::<T, pallet_assets::Instance1>::maybe_total_supply(asset_id.clone()).is_some() {
		return;
	}

	let owner = asset_admin_account::<T>();
	ensure_native_balance::<T>(&owner);

	let owner_lookup = T::Lookup::unlookup(owner.clone());
	let min_balance: <T as pallet_assets::Config<pallet_assets::Instance1>>::Balance = 1u128.saturated_into();
	let _ = pallet_assets::Pallet::<T, pallet_assets::Instance1>::force_create(
		RawOrigin::Root.into(),
		asset_id.clone().into(),
		owner_lookup,
		true,
		min_balance,
	);
}

fn ensure_asset_balance<T>(account: &T::AccountId, asset: u32)
where
	T: Config + pallet_assets::Config<pallet_assets::Instance1, AssetId = AssetIdOf<T>>,
	<T as pallet_assets::Config<pallet_assets::Instance1>>::Balance: SaturatedConversion,
{
	ensure_asset_exists::<T>(asset);
	let asset_id: AssetIdOf<T> = asset.into();
	let current = pallet_assets::Pallet::<T, pallet_assets::Instance1>::balance(asset_id.clone(), account);
	let current_u128: u128 = current.saturated_into();

	if current_u128 >= CUSTOM_ASSET_BALANCE_TARGET {
		return;
	}

	let delta = CUSTOM_ASSET_BALANCE_TARGET - current_u128;
	if delta == 0 {
		return;
	}

	let delta_balance: <T as pallet_assets::Config<pallet_assets::Instance1>>::Balance = delta.saturated_into();
	let owner = asset_admin_account::<T>();
	ensure_native_balance::<T>(&owner);
	let beneficiary = T::Lookup::unlookup(account.clone());
	let _ = pallet_assets::Pallet::<T, pallet_assets::Instance1>::mint(
		RawOrigin::Signed(owner).into(),
		asset_id.into(),
		beneficiary,
		delta_balance,
	);
}

fn ensure_account_ready<T>(account: &T::AccountId)
where
	T: Config + pallet_assets::Config<pallet_assets::Instance1, AssetId = AssetIdOf<T>>,
	<T as pallet_assets::Config<pallet_assets::Instance1>>::Balance: SaturatedConversion,
{
	ensure_native_balance::<T>(account);
	ensure_asset_balance::<T>(account, USDC);
	ensure_asset_balance::<T>(account, WETH);
	ensure_asset_balance::<T>(account, WBTC);
}

fn funded_account<T>(id: u8) -> T::AccountId
where
	T: Config + pallet_assets::Config<pallet_assets::Instance1, AssetId = AssetIdOf<T>>,
	<T as pallet_assets::Config<pallet_assets::Instance1>>::Balance: SaturatedConversion,
{
	let account = mock_account_id::<T>(id);
	ensure_account_ready::<T>(&account);
	account
}

fn register_operator<T>(blueprint_id: u64, id: u8) -> T::AccountId
where
	T: Config + pallet_assets::Config<pallet_assets::Instance1, AssetId = AssetIdOf<T>>,
	<T as pallet_assets::Config<pallet_assets::Instance1>>::Balance: SaturatedConversion,
{
	let operator = funded_account::<T>(id);
	assert_ok!(Pallet::<T>::register(
		RawOrigin::Signed(operator.clone()).into(),
		blueprint_id,
		operator_preferences::<T>(id),
		Default::default(),
		0_u32.into()
	));
	operator
}

fn prepare_blueprint_with_operators<T>(operator_ids: &[u8]) -> (T::AccountId, Vec<T::AccountId>)
where
	T: Config + pallet_assets::Config<pallet_assets::Instance1, AssetId = AssetIdOf<T>>,
	<T as pallet_assets::Config<pallet_assets::Instance1>>::Balance: SaturatedConversion,
{
	let owner = funded_account::<T>(1u8);
	setup_master_blueprint_manager::<T>();
	let blueprint = cggmp21_blueprint::<T>();
	assert_ok!(create_test_blueprint::<T>(RawOrigin::Signed(owner.clone()).into(), blueprint));

	let operators =
		operator_ids.iter().map(|id| register_operator::<T>(0, *id)).collect::<Vec<_>>();

	(owner, operators)
}

fn operator_preferences<T: Config>(seed: u8) -> OperatorPreferences<T::Constraints> {
	OperatorPreferences {
		key: bench_ecdsa_key(seed),
		rpc_address: BoundedString::try_from("https://example.com/rpc".to_owned()).unwrap(),
	}
}

fn cggmp21_blueprint<T: Config>() -> ServiceBlueprint<T::Constraints> {
	ServiceBlueprint {
		metadata: ServiceMetadata { name: "CGGMP21 TSS".try_into().unwrap(), ..Default::default() },
		manager: BlueprintServiceManager::Evm(H160::from_slice(&[0u8; 20])),
		master_manager_revision: MasterBlueprintServiceManagerRevision::Latest,
		jobs: vec![
			JobDefinition {
				metadata: JobMetadata { name: "keygen".try_into().unwrap(), ..Default::default() },
				params: vec![FieldType::Uint8].try_into().unwrap(),
				result: vec![FieldType::List(Box::new(FieldType::Uint8))].try_into().unwrap(),
				pricing_model: PricingModel::PayOnce { amount: 100u128 },
			},
			JobDefinition {
				metadata: JobMetadata { name: "sign".try_into().unwrap(), ..Default::default() },
				params: vec![FieldType::Uint64, FieldType::List(Box::new(FieldType::Uint8))]
					.try_into()
					.unwrap(),
				result: vec![FieldType::List(Box::new(FieldType::Uint8))].try_into().unwrap(),
				pricing_model: PricingModel::PayOnce { amount: 50u128 },
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
	}
}

fn create_test_blueprint<T: Config>(
	origin: OriginFor<T>,
	blueprint: ServiceBlueprint<T::Constraints>,
) -> Result<(), sp_runtime::DispatchError> {
	Pallet::<T>::create_blueprint(origin, blueprint)
		.map(|_| ())
		.map_err(|e| e.error)
}

fn setup_master_blueprint_manager<T: Config>() {
	// Set up master blueprint service manager first
	Pallet::<T>::update_master_blueprint_service_manager(
		frame_system::RawOrigin::Root.into(),
		H160::from_slice(&[0u8; 20]),
	)
	.unwrap();
}

benchmarks! {

	where_clause {
		where
			<T as crate::module::Config>::AssetId: From<u32>,
			T: pallet_assets::Config<pallet_assets::Instance1, AssetId = <T as crate::module::Config>::AssetId>,
			<T as pallet_assets::Config<pallet_assets::Instance1>>::Balance: SaturatedConversion,
	}

	create_blueprint {
		let alice = funded_account::<T>(1u8);
		setup_master_blueprint_manager::<T>();
		let blueprint = cggmp21_blueprint::<T>();
	}: _(
		RawOrigin::Signed(alice.clone()),
		blueprint
	)

	pre_register {
		let alice = funded_account::<T>(1u8);
		setup_master_blueprint_manager::<T>();
		let blueprint = cggmp21_blueprint::<T>();
		assert_ok!(create_test_blueprint::<T>(RawOrigin::Signed(alice.clone()).into(), blueprint));

		let bob = funded_account::<T>(2u8);

	}: _(RawOrigin::Signed(bob.clone()), 0)


	register {
		let alice = funded_account::<T>(1u8);
		setup_master_blueprint_manager::<T>();
		let blueprint = cggmp21_blueprint::<T>();
		assert_ok!(create_test_blueprint::<T>(RawOrigin::Signed(alice.clone()).into(), blueprint));

		let bob = funded_account::<T>(2u8);

	}: _(RawOrigin::Signed(bob.clone()), 0, operator_preferences::<T>(2u8), Default::default(), 0_u32.into())


	unregister {
		let (_owner, mut operators) = prepare_blueprint_with_operators::<T>(&[2]);
		let bob = operators.pop().expect("Operator exists");

	}: _(RawOrigin::Signed(bob.clone()), 0)

	update_rpc_address {
		let (_owner, mut operators) = prepare_blueprint_with_operators::<T>(&[2]);
		let bob = operators.pop().expect("Operator exists");
		let rpc_address = BoundedString::try_from("https://example.com/rpc".to_owned()).unwrap();

	}: _(RawOrigin::Signed(bob.clone()), 0, rpc_address)


	request {
		let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2, 3, 4, 5]);
		let eve = operators.pop().expect("Eve exists");
		let dave = operators.pop().expect("Dave exists");
		let charlie = operators.pop().expect("Charlie exists");
		let bob = operators.pop().expect("Bob exists");

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
		let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2, 3, 4]);
		let dave = operators.pop().expect("Dave exists");
		let charlie = operators.pop().expect("Charlie exists");
		let bob = operators.pop().expect("Bob exists");

		let eve = funded_account::<T>(5u8);
		assert_ok!(Pallet::<T>::request(
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
		));

		let security_commitments = vec![
			get_security_commitment::<T>(USDC.into(), 10),
			get_security_commitment::<T>(WETH.into(), 10),
		];

	}: _(RawOrigin::Signed(charlie.clone()), 0, security_commitments)


	reject {
		let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2, 3, 4]);
		let dave = operators.pop().expect("Dave exists");
		let charlie = operators.pop().expect("Charlie exists");
		let bob = operators.pop().expect("Bob exists");

		let eve = funded_account::<T>(5u8);
		assert_ok!(Pallet::<T>::request(
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
		));

	}: _(RawOrigin::Signed(charlie.clone()), 0)


	terminate {
		let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2, 3, 4]);
		let dave = operators.pop().expect("Dave exists");
		let charlie = operators.pop().expect("Charlie exists");
		let bob = operators.pop().expect("Bob exists");

		let eve = funded_account::<T>(5u8);
		assert_ok!(Pallet::<T>::request(
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
		));

	}: _(RawOrigin::Signed(eve.clone()),0)


	call {
		let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2, 3, 4]);
		let dave = operators.pop().expect("Dave exists");
		let charlie = operators.pop().expect("Charlie exists");
		let bob = operators.pop().expect("Bob exists");

		let eve = funded_account::<T>(5u8);
		assert_ok!(Pallet::<T>::request(
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
		));

	}: _(
			RawOrigin::Signed(eve.clone()),
			0,
			0,
			vec![Field::Uint8(2)].try_into().unwrap()
		)

	submit_result {
		let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2, 3, 4]);
		let dave = operators.pop().expect("Dave exists");
		let charlie = operators.pop().expect("Charlie exists");
		let bob = operators.pop().expect("Bob exists");

		let eve = funded_account::<T>(5u8);
		assert_ok!(Pallet::<T>::request(
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
		));

		assert_ok!(Pallet::<T>::call(
			RawOrigin::Signed(eve.clone()).into(),
			0,
			0,
			vec![Field::Uint8(2)].try_into().unwrap()
		));

		let keygen_job_call_id = 0;
		let key_type = KeyTypeId(*b"mdkg");
		let dkg = sp_io::crypto::ecdsa_generate(key_type, None);

	}: _(
			RawOrigin::Signed(bob.clone()),
			0,
			keygen_job_call_id,
			vec![Field::from(BoundedVec::try_from(dkg.to_raw().to_vec()).unwrap())].try_into().unwrap()
		)

	heartbeat {
		const HEARTBEAT_INTERVAL_VALUE: u32 = 10;
		const DUMMY_OPERATOR_ADDRESS_BYTES: [u8; 20] = [1u8; 20];

		let creator = funded_account::<T>(0u8);
		let operator_account = funded_account::<T>(1u8);
		let service_requester = funded_account::<T>(2u8);

		let blueprint_id = 0u64;
		let service_id = Pallet::<T>::next_service_request_id();

		setup_master_blueprint_manager::<T>();
		let blueprint = cggmp21_blueprint::<T>();
		assert_ok!(create_test_blueprint::<T>(RawOrigin::Signed(creator.clone()).into(), blueprint));

		let operator_key = ecdsa::Pair::from_seed(&[1u8; 32]);
		let operator_address = H160(DUMMY_OPERATOR_ADDRESS_BYTES);
		let op_preferences = operator_preferences::<T>(1u8);
		let registration_args = Vec::<Field<T::Constraints, T::AccountId>>::new();

		assert_ok!(Pallet::<T>::register(
			RawOrigin::Signed(operator_account.clone()).into(),
			blueprint_id,
			op_preferences,
			registration_args,
			0u32.into()
		));

		frame_system::Pallet::<T>::set_block_number(1u32.into());

		assert_ok!(Pallet::<T>::request(
			RawOrigin::Signed(service_requester.clone()).into(),
			None,
			blueprint_id,
			vec![operator_account.clone()].try_into().unwrap(),
			vec![operator_account.clone()].try_into().unwrap(),
			Default::default(),
			Default::default(),
			100u32.into(),
			Asset::Custom(AssetIdOf::<T>::from(USDC)),
			0u32.into(),
			MembershipModel::Fixed { min_operators: 1u32.into() }
		));

		frame_system::Pallet::<T>::set_block_number(2u32.into());

		frame_system::Pallet::<T>::set_block_number((HEARTBEAT_INTERVAL_VALUE + 2).into());

		let metrics_data: Vec<u8> = vec![1,2,3];

		let mut message = service_id.to_le_bytes().to_vec();
		message.extend_from_slice(&blueprint_id.to_le_bytes());
		message.extend_from_slice(&metrics_data);

		let message_hash = sp_core::hashing::keccak_256(&message);

		let signature_bytes = [0u8; 65];
		let signature = ecdsa::Signature::from_raw(signature_bytes);


	}: _(RawOrigin::Signed(operator_account.clone()), blueprint_id, service_id, metrics_data, signature)

	// Slash an operator's stake for a service
	slash {
		let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2]);
		let bob = operators.pop().expect("operator exists");

		assert_ok!(Pallet::<T>::request(
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
		));

	}: _(RawOrigin::Signed(alice.clone()), bob.clone(), 0, Percent::from_percent(50))

	// Dispute a scheduled slash
	dispute {
		let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2]);
		let bob = operators.pop().expect("operator exists");

		assert_ok!(Pallet::<T>::request(
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
		));

		assert_ok!(Pallet::<T>::slash(
			RawOrigin::Signed(alice.clone()).into(),
			bob.clone(),
			0,
			Percent::from_percent(50)
		));

	}: _(RawOrigin::Signed(alice.clone()), 0, 0)

	// Update master blueprint service manager
	update_master_blueprint_service_manager {
	}: _(RawOrigin::Root, H160::zero())

	// Join a service as an operator
	join_service {
		let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2]);
		let bob = operators.pop().expect("operator exists");

		assert_ok!(Pallet::<T>::request(
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
		));

		let charlie = register_operator::<T>(0, 3u8);

	}: _(RawOrigin::Signed(charlie.clone()), 0, vec![get_security_commitment::<T>(USDC.into(), 10)])

	// Leave a service as an operator
	leave_service {
		let (alice, operators) = prepare_blueprint_with_operators::<T>(&[2, 3]);
		let mut iter = operators.clone().into_iter();
		let bob = iter.next().expect("bob exists");
		let charlie = iter.next().expect("charlie exists");

		assert_ok!(Pallet::<T>::request(
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
		));

	}: _(RawOrigin::Signed(charlie.clone()), 0)

	// Benchmark payment validation for pay-once services
	validate_payment_amount_pay_once {
		let alice = funded_account::<T>(1u8);
		setup_master_blueprint_manager::<T>();
		let blueprint = cggmp21_blueprint::<T>();
		assert_ok!(create_test_blueprint::<T>(RawOrigin::Signed(alice.clone()).into(), blueprint));

		let (_, blueprint) = Pallet::<T>::blueprints(0).expect("blueprint exists");
		let amount = 1000_u32.into();
	}: {
		let _ = Pallet::<T>::validate_payment_amount(&blueprint, amount);
	}

	// Benchmark payment processing for subscription services
	process_subscription_payment {
		let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2]);
		let bob = operators.pop().expect("operator exists");

		assert_ok!(Pallet::<T>::request(
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
		));

		let service_id = 0;
		let job_index = 0;
		let call_id = 0;
		let subscriber = alice.clone();
		let rate_per_interval = 100u32.into();
		let interval = 10u32.into();
		let maybe_end = None;
		let current_block = frame_system::Pallet::<T>::block_number();
	}: {
		let _ = Pallet::<T>::process_job_subscription_payment(
			service_id,
			job_index,
			call_id,
			&subscriber, // caller (subscriber authorizes their own payment)
			&subscriber, // payer
			rate_per_interval,
			interval,
			maybe_end,
			current_block
		);
	}

	// Benchmark event-driven payment processing
	process_event_driven_payment {
		let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2]);
		let bob = operators.pop().expect("operator exists");

		assert_ok!(Pallet::<T>::request(
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
		));

		let service_id = 0;
		let job_index = 0;
		let call_id = 0;
		let subscriber = alice.clone();
		let reward_per_event = 10u32.into();
		let event_count = 5;
	}: {
		let _ = Pallet::<T>::process_job_event_driven_payment(
			service_id,
			job_index,
			call_id,
			&subscriber, // caller (subscriber authorizes their own payment)
			&subscriber, // payer
			reward_per_event,
			event_count
		);
	}

	// Benchmark subscription payments processing with on_idle
	process_subscription_payments_on_idle {
		let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2]);
		let bob = operators.pop().expect("operator exists");

		// Create multiple service instances to test batch processing
		for i in 0..5 {
			let requester = funded_account::<T>((10 + i) as u8);
			assert_ok!(Pallet::<T>::request(
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
			));
		}

		let current_block = 100_u32.into();
		let remaining_weight = frame_support::weights::Weight::from_parts(1_000_000_000, 0);
	}: {
		let _ = Pallet::<T>::process_subscription_payments_on_idle(current_block, remaining_weight);
	}
}

// Define the module and associated types for the benchmarks
impl_benchmark_test_suite!(
	Pallet,
	crate::mock::new_test_ext(vec![1, 2, 3, 4]),
	crate::mock::Runtime,
);
