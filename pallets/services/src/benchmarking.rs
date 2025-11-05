use super::*;
use crate::OriginFor;
use frame_benchmarking::v1::{benchmarks, impl_benchmark_test_suite};
use frame_support::{BoundedVec, assert_ok, traits::Currency};
use frame_system::RawOrigin;
use fp_evm::ExitReason;
use scale_info::prelude::boxed::Box;
use sp_core::{H160, U256, crypto::Pair, ecdsa};
use sp_runtime::{
	KeyTypeId, Percent,
	traits::{SaturatedConversion, Zero},
	Saturating,
};
use sp_std::vec;
use tangle_primitives::services::{
	Asset, AssetSecurityCommitment, AssetSecurityRequirement, BlueprintServiceManager,
	BoundedString, Field, FieldType, JobDefinition, JobMetadata,
	MasterBlueprintServiceManagerRevision, MembershipModel, MembershipModelType,
	OperatorPreferences, PricingModel, ServiceBlueprint, ServiceMetadata,
	EvmAddressMapping, EvmRunner
};

pub type AssetId = u32;
pub type AssetIdOf<T> = <T as Config>::AssetId;

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

fn setup_nominator<T: Config>(
	delegator: T::AccountId,
	bond_amount: BalanceOf<T>,
	operator: T::AccountId,
	assets: Vec<Asset<T::AssetId>>,
	amounts: Vec<BalanceOf<T>>,
) {
	assert_ok!(<T::BenchmarkingHelper as tangle_primitives::traits::MultiAssetDelegationBenchmarkingHelperOperator<
		T::AccountId,
		BalanceOf<T>,
	>>::handle_deposit_and_create_operator_be(operator.clone(), bond_amount));

	for (i, asset) in assets.iter().enumerate() {
		assert_ok!(<T::BenchmarkingHelper as tangle_primitives::traits::MultiAssetDelegationBenchmarkingHelperDelegation<
			T::AccountId,
			BalanceOf<T>,
			T::AssetId,
		>>::process_delegate_be(delegator.clone(), operator.clone(), asset.clone(), amounts[i]));
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

fn ensure_asset_exists<T: Config>(asset: u32) {
	let asset_id: AssetIdOf<T> = asset.into();
	if T::BenchmarkingHelper::asset_exists(asset_id.clone())
	{
		return;
	}

	let owner = asset_admin_account::<T>();
	ensure_native_balance::<T>(&owner);

	let min_balance: BalanceOf<T> = 1_u128.saturated_into();
	let _ = T::BenchmarkingHelper::create(
		asset_id.clone().into(),
		owner.clone(),
		true,
		min_balance,
	);
}

fn ensure_asset_balance<T: Config>(account: &T::AccountId, asset: u32) {
	ensure_asset_exists::<T>(asset);
	let asset_id: AssetIdOf<T> = asset.into();
	let current = T::BenchmarkingHelper::balance(asset_id.clone(), account);
	let current_u128: u128 = current.saturated_into();

	if current_u128 >= CUSTOM_ASSET_BALANCE_TARGET {
		return;
	}

	let delta = CUSTOM_ASSET_BALANCE_TARGET - current_u128;
	if delta == 0 {
		return;
	}

	let delta_balance: BalanceOf<T> =
		delta.saturated_into();
	let owner = asset_admin_account::<T>();
	ensure_native_balance::<T>(&owner);
	let _ = T::BenchmarkingHelper::mint_into(
		asset_id.clone().into(),
		&account.clone(),
		delta_balance,
	);
}

fn ensure_account_ready<T: Config>(account: &T::AccountId) {
	ensure_native_balance::<T>(account);
	ensure_asset_balance::<T>(account, USDC);
	ensure_asset_balance::<T>(account, WETH);
	ensure_asset_balance::<T>(account, WBTC);
}

fn funded_account<T: Config>(id: u8) -> T::AccountId {
	let account = mock_account_id::<T>(id);
	ensure_account_ready::<T>(&account);
	account
}

fn register_operator<T: Config>(blueprint_id: u64, operator: T::AccountId, operator_id: u8) {
	assert_ok!(Pallet::<T>::register(
		RawOrigin::Signed(operator.clone()).into(),
		blueprint_id,
		operator_preferences::<T>(operator_id),
		Default::default(),
		0_u32.into()
	));
}

fn prepare_blueprint_with_operators<T: Config>(operator_ids: &[u8]) -> (T::AccountId, Vec<T::AccountId>) {
	let owner = funded_account::<T>(1u8);
	let blueprint = cggmp21_blueprint::<T>();
	assert_ok!(create_test_blueprint::<T>(RawOrigin::Signed(owner.clone()).into(), blueprint));

	let operators = operator_ids.iter().map(|id| funded_account::<T>(*id)).collect::<Vec<_>>();

	for (idx, operator) in operators.iter().enumerate() {
		setup_nominator::<T>(
			owner.clone(),
			100_u128.saturated_into(),
			operator.clone(),
			vec![Asset::Custom(USDC.into()), Asset::Custom(WETH.into()), Asset::Custom(TNT.into())],
			vec![100_u128.saturated_into(), 100_u128.saturated_into(), 100_u128.saturated_into()],
		);

		register_operator::<T>(0, operator.clone(), idx as u8);
	}

	(owner, operators)
}

fn prepare_service<T: Config>() -> (T::AccountId, [T::AccountId; 3]) {
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
		get_security_commitment::<T>(TNT.into(), 10),
	];

	assert_ok!(Pallet::<T>::approve(
		RawOrigin::Signed(charlie.clone()).into(),
		0,
		security_commitments.clone()
	));
	assert_ok!(Pallet::<T>::approve(
		RawOrigin::Signed(dave.clone()).into(),
		0,
		security_commitments.clone()
	));
	assert_ok!(Pallet::<T>::approve(
		RawOrigin::Signed(bob.clone()).into(),
		0,
		security_commitments.clone()
	));
	(eve, [dave, bob, charlie])
}

fn operator_preferences<T: Config>(seed: u8) -> OperatorPreferences<T::Constraints> {
	OperatorPreferences {
		key: bench_ecdsa_key(seed),
		rpc_address: BoundedString::try_from("https://example.com/rpc".to_owned()).unwrap(),
	}
}

fn cggmp21_blueprint<T: Config>() -> ServiceBlueprint<T::Constraints> {
	let deployer_account = funded_account::<T>(100u8);
	let deployer_address = T::EvmAddressMapping::into_address(deployer_account.clone());
	let deployer_evm_account_id = T::EvmAddressMapping::into_account_id(deployer_address);
	ensure_native_balance::<T>(&deployer_evm_account_id);

	let create_contract = |bytecode: &str, contract_name: &str| -> H160 {
		let mut raw_hex = bytecode.replace("0x", "").replace("\n", "");
		// fix odd length
		if raw_hex.len() % 2 != 0 {
			raw_hex = format!("0{}", raw_hex);
		}
		let code = hex::decode(raw_hex).unwrap();
		eprintln!("Deploying {}", contract_name);

		let gas_limit = 10_000_000_000u64;

		let create_info = T::EvmRunner::create(
			deployer_address,
			code.clone(),
			U256::from(0),
			gas_limit,
			true, // transactional
			false,
		).map_err(|e| {
			eprintln!("Failed to deploy {}", contract_name);
			e.error.into()
		}).unwrap();

		// Verify deployment was successful
		match create_info.exit_reason {
			ExitReason::Succeed(_) => {
				eprintln!("✓ {} deployed successfully to: {:?}", contract_name, create_info.value);
				eprintln!("  Used gas: {:?}", create_info.used_gas);
				eprintln!("  Exit reason: {:?}", create_info.exit_reason);
			},
			ExitReason::Revert(_) => {
				eprintln!("✗ {} deployment reverted", contract_name);
				eprintln!("  Contract address (if created): {:?}", create_info.value);
				eprintln!("  Used gas: {:?}", create_info.used_gas);
				eprintln!("  This usually means the constructor failed or needs arguments");
				panic!("Contract deployment failed: Revert");
			},
			reason => {
				eprintln!("✗ {} deployment failed with reason: {:?}", contract_name, reason);
				eprintln!("  Return value: {:?}", create_info.value);
				eprintln!("  Used gas: {:?}", create_info.used_gas);
				panic!("Contract deployment failed: {:?}", reason);
			}
		}
		
		// Verify contract address is not zero
		if create_info.value == H160::zero() {
			panic!("Contract {} deployed to zero address!", contract_name);
		}

		create_info.value
	};

	let cggmp21_blueprint_addr = create_contract(
		include_str!("./test-artifacts/CGGMP21Blueprint.hex"),
		"CGGMP21Blueprint"
	);
	let mbsm_addr = create_contract(
		include_str!("./test-artifacts/MasterBlueprintServiceManager.hex"),
		"MasterBlueprintServiceManager"
	);

	// Set up master blueprint service manager first
	assert_ok!(Pallet::<T>::update_master_blueprint_service_manager(
		frame_system::RawOrigin::Root.into(),
		mbsm_addr,
	));

	ServiceBlueprint {
		metadata: ServiceMetadata { name: "CGGMP21 TSS".try_into().unwrap(), ..Default::default() },
		manager: BlueprintServiceManager::Evm(cggmp21_blueprint_addr),
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

benchmarks! {

	where_clause {
		where
			<T as crate::module::Config>::AssetId: From<u32>,
	}

	create_blueprint {
		let alice = funded_account::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
	}: _(
		RawOrigin::Signed(alice.clone()),
		blueprint
	)

	pre_register {
		let alice = funded_account::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		assert_ok!(create_test_blueprint::<T>(RawOrigin::Signed(alice.clone()).into(), blueprint));

		let bob = funded_account::<T>(2u8);

	}: _(RawOrigin::Signed(bob.clone()), 0)


	register {
		let alice = funded_account::<T>(1u8);
		let blueprint_id = Pallet::<T>::next_blueprint_id();
		let blueprint = cggmp21_blueprint::<T>();
		assert_ok!(create_test_blueprint::<T>(RawOrigin::Signed(alice.clone()).into(), blueprint));

		let bob = funded_account::<T>(2u8);
		setup_nominator::<T>(
			alice.clone(),
			100_u128.saturated_into(),
			bob.clone(),
			vec![Asset::Custom(USDC.into()), Asset::Custom(WETH.into())],
			vec![100_u128.saturated_into(), 100_u128.saturated_into()],
		);
	}: _(RawOrigin::Signed(bob.clone()), blueprint_id, operator_preferences::<T>(2u8), Default::default(), 0_u32.into())


	// unregister {
	// 	let (_owner, mut operators) = prepare_blueprint_with_operators::<T>(&[2]);
	// 	let bob = operators.pop().expect("Operator exists");

	// }: _(RawOrigin::Signed(bob.clone()), 0)

	// update_rpc_address {
	// 	let (_owner, mut operators) = prepare_blueprint_with_operators::<T>(&[2]);
	// 	let bob = operators.pop().expect("Operator exists");
	// 	let rpc_address = BoundedString::try_from("https://example.com/rpc".to_owned()).unwrap();

	// }: _(RawOrigin::Signed(bob.clone()), 0, rpc_address)


	// request {
	// 	let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2, 3, 4, 5]);
	// 	let eve = operators.pop().expect("Eve exists");
	// 	let dave = operators.pop().expect("Dave exists");
	// 	let charlie = operators.pop().expect("Charlie exists");
	// 	let bob = operators.pop().expect("Bob exists");
	// }: _(
	// 	RawOrigin::Signed(bob.clone()),
	// 	None,
	// 	0,
	// 	vec![alice.clone()],
	// 	vec![bob.clone(), charlie.clone(), dave.clone()],
	// 	Default::default(),
	// 	vec![
	// 		get_security_requirement::<T>(USDC.into(), &[10, 20]),
	// 		get_security_requirement::<T>(WETH.into(), &[10, 20])
	// 	],
	// 	100_u32.into(),
	// 	Asset::Custom(USDC.into()),
	// 	0_u32.into(),
	// 	MembershipModel::Fixed { min_operators: 3 }
	// )

	// approve {
	// 	let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2, 3, 4]);
	// 	let dave = operators.pop().expect("Dave exists");
	// 	let charlie = operators.pop().expect("Charlie exists");
	// 	let bob = operators.pop().expect("Bob exists");

	// 	let eve = funded_account::<T>(5u8);
	// 	assert_ok!(Pallet::<T>::request(
	// 		RawOrigin::Signed(eve.clone()).into(),
	// 		None,
	// 		0,
	// 		vec![alice.clone()],
	// 		vec![bob.clone(), charlie.clone(), dave.clone()],
	// 		Default::default(),
	// 		vec![
	// 			get_security_requirement::<T>(USDC.into(), &[10, 20]),
	// 			get_security_requirement::<T>(WETH.into(), &[10, 20])
	// 		],
	// 		100_u32.into(),
	// 		Asset::Custom(USDC.into()),
	// 		0_u32.into(),
	// 		MembershipModel::Fixed { min_operators: 3 },
	// 	));

	// 	let security_commitments = vec![
	// 		get_security_commitment::<T>(USDC.into(), 10),
	// 		get_security_commitment::<T>(WETH.into(), 10),
	// 		get_security_commitment::<T>(TNT.into(), 10),
	// 	];

	// }: _(RawOrigin::Signed(charlie.clone()), 0, security_commitments)


	// reject {
	// 	let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2, 3, 4]);
	// 	let dave = operators.pop().expect("Dave exists");
	// 	let charlie = operators.pop().expect("Charlie exists");
	// 	let bob = operators.pop().expect("Bob exists");

	// 	let eve = funded_account::<T>(5u8);
	// 	assert_ok!(Pallet::<T>::request(
	// 		RawOrigin::Signed(eve.clone()).into(),
	// 		None,
	// 		0,
	// 		vec![alice.clone()],
	// 		vec![bob.clone(), charlie.clone(), dave.clone()],
	// 		Default::default(),
	// 		vec![
	// 			get_security_requirement::<T>(USDC.into(), &[10, 20]),
	// 			get_security_requirement::<T>(WETH.into(), &[10, 20])
	// 		],
	// 		100_u32.into(),
	// 		Asset::Custom(USDC.into()),
	// 		0_u32.into(),
	// 		MembershipModel::Fixed { min_operators: 3 },
	// 	));

	// }: _(RawOrigin::Signed(charlie.clone()), 0)


	// terminate {
	// 	let (owner, _) = prepare_service::<T>();
	// }: _(RawOrigin::Signed(owner),0)


	// call {
	// 	let (owner, _) = prepare_service::<T>();
	// }: _(RawOrigin::Signed(owner),0,0,vec![Field::Uint8(2)].try_into().unwrap())

	// submit_result {
	// 	let (owner, operators) = prepare_service::<T>();
	// 	assert_ok!(Pallet::<T>::call(
	// 		RawOrigin::Signed(owner.clone()).into(),
	// 		0,
	// 		0,
	// 		vec![Field::Uint8(2)].try_into().unwrap()
	// 	));

	// 	let keygen_job_call_id = 0;
	// 	let key_type = KeyTypeId(*b"mdkg");
	// 	let dkg = sp_io::crypto::ecdsa_generate(key_type, None);
	// }: _(
	// 		RawOrigin::Signed(operators[0].clone()),
	// 		0,
	// 		keygen_job_call_id,
	// 		vec![Field::from(BoundedVec::try_from(dkg.to_raw().to_vec()).unwrap())].try_into().unwrap()
	// 	)

	// heartbeat {
	// 	const HEARTBEAT_INTERVAL_VALUE: u32 = 10;
	// 	let service_id = Pallet::<T>::next_service_request_id();
	// 	let blueprint_id = 0u64;

	// 	let (_owner, operators) = prepare_service::<T>();
	// 	let operator = H160::from_slice(&operators[0].clone().to_vec());

	// 	// Advance blocks to allow heartbeat
	// 	let current_block = frame_system::Pallet::<T>::block_number();
	// 	let heartbeat_block = current_block.saturating_add((HEARTBEAT_INTERVAL_VALUE + 2).into());
	// 	frame_system::Pallet::<T>::set_block_number(heartbeat_block);

	// 	let metrics_data: Vec<u8> = vec![1, 2, 3];

	// 	let mut message = service_id.to_le_bytes().to_vec();
	// 	message.extend_from_slice(&blueprint_id.to_le_bytes());
	// 	message.extend_from_slice(&metrics_data);

	// 	let message_hash = sp_core::hashing::keccak_256(&message);

	// 	let mut seed = [0u8; 32];
	// 	seed.fill(0u8);
	// 	seed[0] = 0u8;
	// 	seed[15] = 0u8.wrapping_mul(7).wrapping_add(3);
	// 	seed[31] = 0u8.wrapping_mul(11).wrapping_add(1);
	// 	let operator_key = ecdsa::Pair::from_seed(&seed);
	// 	let message_hash = sp_core::hashing::keccak_256(&message);
	// 	let signature_bytes = [0u8; 65];
	// 	let signature = ecdsa::Signature::from_raw(signature_bytes);

	// }: _(RawOrigin::Signed(operator.clone()), blueprint_id, service_id, metrics_data, signature)

	// // Slash an operator's stake for a service
	// slash {
	// 	let (owner, operators) = prepare_service::<T>();
	// }: _(RawOrigin::Signed(owner.clone()), operators[0].clone(), 0, Percent::from_percent(50))

	// // Dispute a scheduled slash
	// dispute {
	// 	let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2]);
	// 	let bob = operators.pop().expect("operator exists");

	// 	assert_ok!(Pallet::<T>::request(
	// 		RawOrigin::Signed(alice.clone()).into(),
	// 		None,
	// 		0,
	// 		vec![alice.clone()],
	// 		vec![bob.clone()],
	// 		Default::default(),
	// 		vec![get_security_requirement::<T>(USDC.into(), &[10, 20])],
	// 		100_u32.into(),
	// 		Asset::Custom(USDC.into()),
	// 		0_u32.into(),
	// 		MembershipModel::Fixed { min_operators: 1 }
	// 	));

	// 	assert_ok!(Pallet::<T>::slash(
	// 		RawOrigin::Signed(alice.clone()).into(),
	// 		bob.clone(),
	// 		0,
	// 		Percent::from_percent(50)
	// 	));

	// }: _(RawOrigin::Signed(alice.clone()), 0, 0)

	// // Update master blueprint service manager
	// update_master_blueprint_service_manager {
	// }: _(RawOrigin::Root, H160::zero())

	// // Join a service as an operator
	// join_service {
	// 	let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2]);
	// 	let bob = operators.pop().expect("operator exists");

	// 	assert_ok!(Pallet::<T>::request(
	// 		RawOrigin::Signed(alice.clone()).into(),
	// 		None,
	// 		0,
	// 		vec![alice.clone()],
	// 		vec![bob.clone()],
	// 		Default::default(),
	// 		vec![get_security_requirement::<T>(USDC.into(), &[10, 20])],
	// 		100_u32.into(),
	// 		Asset::Custom(USDC.into()),
	// 		0_u32.into(),
	// 		MembershipModel::Fixed { min_operators: 1 }
	// 	));

	// 	let charlie = register_operator::<T>(0, 3u8);

	// }: _(RawOrigin::Signed(charlie.clone()), 0, vec![get_security_commitment::<T>(USDC.into(), 10)])

	// // Leave a service as an operator
	// leave_service {
	// 	let (alice, operators) = prepare_blueprint_with_operators::<T>(&[2, 3]);
	// 	let mut iter = operators.clone().into_iter();
	// 	let bob = iter.next().expect("bob exists");
	// 	let charlie = iter.next().expect("charlie exists");

	// 	assert_ok!(Pallet::<T>::request(
	// 		RawOrigin::Signed(alice.clone()).into(),
	// 		None,
	// 		0,
	// 		vec![alice.clone()],
	// 		vec![bob.clone(), charlie.clone()],
	// 		Default::default(),
	// 		vec![get_security_requirement::<T>(USDC.into(), &[10, 20])],
	// 		100_u32.into(),
	// 		Asset::Custom(USDC.into()),
	// 		0_u32.into(),
	// 		MembershipModel::Dynamic { min_operators: 1, max_operators: Some(3) }
	// 	));

	// }: _(RawOrigin::Signed(charlie.clone()), 0)

	// // Benchmark payment validation for pay-once services
	// validate_payment_amount_pay_once {
	// 	let alice = funded_account::<T>(1u8);
	// 	setup_master_blueprint_manager::<T>();
	// 	let blueprint = cggmp21_blueprint::<T>();
	// 	assert_ok!(create_test_blueprint::<T>(RawOrigin::Signed(alice.clone()).into(), blueprint));

	// 	let (_, blueprint) = Pallet::<T>::blueprints(0).expect("blueprint exists");
	// 	let amount = 1000_u32.into();
	// }: {
	// 	let _ = Pallet::<T>::validate_payment_amount(&blueprint, amount);
	// }

	// // Benchmark payment processing for subscription services
	// process_subscription_payment {
	// 	let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2]);
	// 	let bob = operators.pop().expect("operator exists");

	// 	assert_ok!(Pallet::<T>::request(
	// 		RawOrigin::Signed(alice.clone()).into(),
	// 		None,
	// 		0,
	// 		vec![alice.clone()],
	// 		vec![bob.clone()],
	// 		Default::default(),
	// 		vec![get_security_requirement::<T>(USDC.into(), &[10, 20])],
	// 		100_u32.into(),
	// 		Asset::Custom(USDC.into()),
	// 		0_u32.into(),
	// 		MembershipModel::Fixed { min_operators: 1 }
	// 	));

	// 	let service_id = 0;
	// 	let job_index = 0;
	// 	let call_id = 0;
	// 	let subscriber = alice.clone();
	// 	let rate_per_interval = 100u32.into();
	// 	let interval = 10u32.into();
	// 	let maybe_end = None;
	// 	let current_block = frame_system::Pallet::<T>::block_number();
	// }: {
	// 	let _ = Pallet::<T>::process_job_subscription_payment(
	// 		service_id,
	// 		job_index,
	// 		call_id,
	// 		&subscriber, // caller (subscriber authorizes their own payment)
	// 		&subscriber, // payer
	// 		rate_per_interval,
	// 		interval,
	// 		maybe_end,
	// 		current_block
	// 	);
	// }

	// // Benchmark event-driven payment processing
	// process_event_driven_payment {
	// 	let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2]);
	// 	let bob = operators.pop().expect("operator exists");

	// 	assert_ok!(Pallet::<T>::request(
	// 		RawOrigin::Signed(alice.clone()).into(),
	// 		None,
	// 		0,
	// 		vec![alice.clone()],
	// 		vec![bob.clone()],
	// 		Default::default(),
	// 		vec![get_security_requirement::<T>(USDC.into(), &[10, 20])],
	// 		100_u32.into(),
	// 		Asset::Custom(USDC.into()),
	// 		0_u32.into(),
	// 		MembershipModel::Fixed { min_operators: 1 }
	// 	));

	// 	let service_id = 0;
	// 	let job_index = 0;
	// 	let call_id = 0;
	// 	let subscriber = alice.clone();
	// 	let reward_per_event = 10u32.into();
	// 	let event_count = 5;
	// }: {
	// 	let _ = Pallet::<T>::process_job_event_driven_payment(
	// 		service_id,
	// 		job_index,
	// 		call_id,
	// 		&subscriber, // caller (subscriber authorizes their own payment)
	// 		&subscriber, // payer
	// 		reward_per_event,
	// 		event_count
	// 	);
	// }

	// // Benchmark subscription payments processing with on_idle
	// process_subscription_payments_on_idle {
	// 	let (alice, mut operators) = prepare_blueprint_with_operators::<T>(&[2]);
	// 	let bob = operators.pop().expect("operator exists");

	// 	// Create multiple service instances to test batch processing
	// 	for i in 0..5 {
	// 		let requester = funded_account::<T>((10 + i) as u8);
	// 		assert_ok!(Pallet::<T>::request(
	// 			RawOrigin::Signed(requester).into(),
	// 			None,
	// 			0,
	// 			vec![alice.clone()],
	// 			vec![bob.clone()],
	// 			Default::default(),
	// 			vec![get_security_requirement::<T>(USDC.into(), &[10, 20])],
	// 			100_u32.into(),
	// 			Asset::Custom(USDC.into()),
	// 			0_u32.into(),
	// 			MembershipModel::Fixed { min_operators: 1 }
	// 		));
	// 	}

	// 	let current_block = 100_u32.into();
	// 	let remaining_weight = frame_support::weights::Weight::from_parts(1_000_000_000, 0);
	// }: {
	// 	let _ = Pallet::<T>::process_subscription_payments_on_idle(current_block, remaining_weight);
	// }
}

// Define the module and associated types for the benchmarks
impl_benchmark_test_suite!(
	Pallet,
	crate::mock::new_test_ext(vec![1, 2, 3, 4]),
	crate::mock::Runtime,
);
