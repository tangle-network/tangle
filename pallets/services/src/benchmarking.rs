use super::*;
use crate::OriginFor;
use frame_benchmarking::v1::{benchmarks, impl_benchmark_test_suite};
use frame_support::{BoundedVec, assert_ok, traits::Currency};
use frame_system::RawOrigin;
use scale_info::prelude::boxed::Box;
use scale_info::prelude::format;
use sp_core::{H160, crypto::Pair, ecdsa};
use sp_runtime::{
	KeyTypeId, Percent,
	traits::{SaturatedConversion, Zero},
	Saturating
};
use sp_std::{iter, vec};
use hex;
use tangle_primitives::services::{
	Asset, AssetSecurityCommitment, AssetSecurityRequirement, BlueprintServiceManager,
	BoundedString, Field, FieldType, JobDefinition, JobMetadata,
	MasterBlueprintServiceManagerRevision, MembershipModel, MembershipModelType,
	OperatorPreferences, PricingModel, PricingQuote, ResourcePricing, ServiceBlueprint, ServiceMetadata,
	EvmAddressMapping,
};
use tangle_primitives::{BlueprintId, InstanceId};
use tangle_primitives::traits::RewardRecorder;

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

pub const MBSM: H160 = H160([0x12; 20]);
pub const CGGMP21_BLUEPRINT: H160 = H160([0x21; 20]);

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

fn prepare_blueprint_with_operators<T: Config>(operator_ids: &[u8]) -> (T::AccountId, Vec<T::AccountId>, BlueprintId) {
	let owner = funded_account::<T>(1u8);
	let blueprint = cggmp21_blueprint::<T>();
	let blueprint_id = Pallet::<T>::next_blueprint_id();
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

	(owner, operators, blueprint_id)
}

fn prepare_service<T: Config>() -> (T::AccountId, [T::AccountId; 3], BlueprintId, InstanceId) {
	let (alice, mut operators, blueprint_id) = prepare_blueprint_with_operators::<T>(&[2, 3, 4]);
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

	let service_id = Pallet::<T>::next_instance_id();

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
	(eve, [dave, bob, charlie], blueprint_id, service_id)
}

fn operator_preferences<T: Config>(seed: u8) -> OperatorPreferences<T::Constraints> {
	OperatorPreferences {
		key: bench_ecdsa_key(seed),
		rpc_address: BoundedString::try_from("https://example.com/rpc".to_owned()).unwrap(),
	}
}

fn create_and_sign_pricing_quote<T: Config>(
	blueprint_id: BlueprintId,
	ttl: BlockNumberFor<T>,
	total_cost_rate: u128,
	timestamp: u64,
	expiry: u64,
	security_commitments: Vec<AssetSecurityCommitment<T::AssetId>>,
	operator: T::AccountId,
	operator_id: u8,
) -> (PricingQuote<T::Constraints>, ecdsa::Signature) {
	// Convert security commitments from T::AssetId to u128 and create BoundedVec
	let security_commitments_u128: BoundedVec<AssetSecurityCommitment<u128>, <T::Constraints as tangle_primitives::services::Constraints>::MaxOperatorsPerService> = BoundedVec::try_from(
		security_commitments
			.into_iter()
			.map(|commitment| AssetSecurityCommitment {
				asset: match commitment.asset {
					Asset::Custom(id) => Asset::Custom(id.saturated_into::<u128>()),
					Asset::Erc20(addr) => Asset::Erc20(addr),
				},
				exposure_percent: commitment.exposure_percent,
			})
			.collect::<Vec<_>>()
	)
	.unwrap();

	// Create pricing quote
	let quote = PricingQuote {
		blueprint_id,
		ttl_blocks: ttl.saturated_into(),
		total_cost_rate,
		timestamp,
		expiry,
		resources: vec![ResourcePricing {
			kind: BoundedString::try_from("CPU".to_owned()).unwrap(),
			count: 1,
			price_per_unit_rate: total_cost_rate,
		}]
		.try_into()
		.unwrap(),
		security_commitments: security_commitments_u128,
	};

	// Hash the quote
	let message = tangle_primitives::services::pricing::hash_pricing_quote(&quote);

	// Generate the seed using the same algorithm as bench_ecdsa_key
	let mut seed = [0u8; 32];
	seed.fill(operator_id);
	seed[0] = operator_id;
	seed[15] = operator_id.wrapping_mul(7).wrapping_add(3);
	seed[31] = operator_id.wrapping_mul(11).wrapping_add(1);

	// Get the operator's preferences to get their public key (matches what's stored)
	let operator_preferences = crate::Operators::<T>::get(blueprint_id, operator.clone())
		.expect("operator exists");
	let public_key = ecdsa::Public::from_full(&operator_preferences.key)
		.expect("failed to derive public key from operator preferences");

	// Generate key in keystore using the seed (ensures private key is available for signing)
	// Note: ecdsa_generate might produce a different public key, but we use the one from preferences
	// The keystore lookup in ecdsa_sign should work if the seed produces the same key pair
	let key_type = KeyTypeId(*b"mdkg");
	let seed_hex = format!("0x{}", hex::encode(seed));
	let _generated_public_key = sp_io::crypto::ecdsa_generate(key_type, Some(seed_hex.as_bytes().to_vec()));

	// Sign the message - ecdsa_sign will look up the private key in keystore by public key
	// If the generated key doesn't match, this will fail
	let signature = sp_io::crypto::ecdsa_sign(key_type, &public_key, &message)
		.expect("failed to sign pricing quote");

	(quote, signature)
}

fn cggmp21_blueprint<T: Config>() -> ServiceBlueprint<T::Constraints> {
	// Set up master blueprint service manager first
	assert_ok!(Pallet::<T>::update_master_blueprint_service_manager(
		frame_system::RawOrigin::Root.into(),
		MBSM,
	));

	ServiceBlueprint {
		metadata: ServiceMetadata { name: "CGGMP21 TSS".try_into().unwrap(), ..Default::default() },
		manager: BlueprintServiceManager::Evm(CGGMP21_BLUEPRINT),
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


	unregister {
		let (_owner, mut operators, _) = prepare_blueprint_with_operators::<T>(&[2]);
		let bob = operators.pop().expect("Operator exists");

	}: _(RawOrigin::Signed(bob.clone()), 0)

	update_rpc_address {
		let (_owner, mut operators, _) = prepare_blueprint_with_operators::<T>(&[2]);
		let bob = operators.pop().expect("Operator exists");
		let rpc_address = BoundedString::try_from("https://example.com/rpc".to_owned()).unwrap();

	}: _(RawOrigin::Signed(bob.clone()), 0, rpc_address)


	request {
		let (alice, mut operators, _) = prepare_blueprint_with_operators::<T>(&[2, 3, 4, 5]);
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
		let (alice, mut operators, _) = prepare_blueprint_with_operators::<T>(&[2, 3, 4]);
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

	}: _(RawOrigin::Signed(charlie.clone()), 0, security_commitments)


	reject {
		let (alice, mut operators, _) = prepare_blueprint_with_operators::<T>(&[2, 3, 4]);
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
		let (owner, _, _, _) = prepare_service::<T>();
	}: _(RawOrigin::Signed(owner),0)


	call {
		let (owner, _, _, _) = prepare_service::<T>();
	}: _(RawOrigin::Signed(owner),0,0,vec![Field::Uint8(2)].try_into().unwrap())

	request_with_signed_price_quotes {
		let (alice, mut operators, blueprint_id) = prepare_blueprint_with_operators::<T>(&[2, 3, 4]);
		let dave = operators.pop().expect("Dave exists");
		let charlie = operators.pop().expect("Charlie exists");
		let bob = operators.pop().expect("Bob exists");

		let eve = funded_account::<T>(5u8);
		let ttl: BlockNumberFor<T> = 100_u32.into();
		let current_block = frame_system::Pallet::<T>::block_number();
		let timestamp = current_block.saturated_into::<u64>();
		let expiry = timestamp + 1000;

		let security_commitments = vec![
			get_security_commitment::<T>(USDC.into(), 10),
			get_security_commitment::<T>(WETH.into(), 10),
			get_security_commitment::<T>(TNT.into(), 10),
		];

		// Create operators list (will be passed to the extrinsic)
		let operators_list = vec![bob.clone(), charlie.clone(), dave.clone()];
		
		// Create a map to store quotes and signatures by operator
		let mut quotes_and_sigs: sp_std::collections::btree_map::BTreeMap<T::AccountId, (PricingQuote<T::Constraints>, ecdsa::Signature)> = sp_std::collections::btree_map::BTreeMap::new();

		for (idx, operator) in operators_list.iter().enumerate() {
			// Operator IDs match the index in prepare_blueprint_with_operators (0, 1, 2)
			let operator_id = idx as u8;
			let total_cost_rate = 100u128 + (idx as u128 * 10);
			let (quote, signature) = create_and_sign_pricing_quote::<T>(
				blueprint_id,
				ttl,
				total_cost_rate,
				timestamp,
				expiry,
				security_commitments.clone(),
				operator.clone(),
				operator_id,
			);
			quotes_and_sigs.insert(operator.clone(), (quote, signature));
		}

		// Build pricing_quotes and operator_signatures in sorted order (BTreeMap iteration order)
		// The verification code iterates operator_signatures_map (BTreeMap) and uses pricing_quotes[i]
		// So we need pricing_quotes to be in the same order as the BTreeMap iterates (sorted)
		let mut pricing_quotes = Vec::new();
		let mut operator_signatures = Vec::new();
		for (operator, (quote, signature)) in quotes_and_sigs.iter() {
			pricing_quotes.push(quote.clone());
			operator_signatures.push(*signature);
		}
		
		// Also need to ensure operators_list matches the sorted order for the extrinsic call
		// The verification code builds operator_signatures_map from operators.iter().zip(operator_signatures.iter())
		// and then iterates the map in sorted order, using pricing_quotes[i]
		// So we need operators, signatures, and quotes all in the same sorted order
		let sorted_operators: Vec<T::AccountId> = quotes_and_sigs.keys().cloned().collect();

		ensure_account_ready::<T>(&Pallet::<T>::pallet_account());
		let (_, blueprint) = Pallet::<T>::blueprints(blueprint_id).expect("blueprint exists");
		let mbsm_address = Pallet::<T>::mbsm_address_of(&blueprint).expect("MBSM address exists");
		let mbsm_account_id = T::EvmAddressMapping::into_account_id(mbsm_address);
		ensure_account_ready::<T>(&mbsm_account_id);
	}: _(
		RawOrigin::Signed(eve.clone()),
		None,
		blueprint_id,
		vec![alice.clone()],
		sorted_operators,
		Default::default(),
		vec![
			get_security_requirement::<T>(USDC.into(), &[10, 20]),
			get_security_requirement::<T>(WETH.into(), &[10, 20])
		],
		ttl,
		Asset::Custom(USDC.into()),
		MembershipModel::Fixed { min_operators: 3 },
		pricing_quotes,
		operator_signatures,
		security_commitments
	)

	submit_result {
		let (owner, operators, _, _) = prepare_service::<T>();
		assert_ok!(Pallet::<T>::call(
			RawOrigin::Signed(owner.clone()).into(),
			0,
			0,
			vec![Field::Uint8(2)].try_into().unwrap()
		));

		let keygen_job_call_id = 0;
		let key_type = KeyTypeId(*b"mdkg");
		let dkg = sp_io::crypto::ecdsa_generate(key_type, None);
	}: _(
			RawOrigin::Signed(operators[0].clone()),
			0,
			keygen_job_call_id,
			vec![Field::from(BoundedVec::try_from(dkg.to_raw().to_vec()).unwrap())].try_into().unwrap()
		)

	heartbeat {
		const OPERATOR_ID: u8 = 2u8;
		frame_system::Pallet::<T>::set_block_number(2u32.into());

		let (_, operators, blueprint_id, service_id) = prepare_service::<T>();
		let (_, blueprint) = Pallet::<T>::blueprints(blueprint_id).expect("blueprint exists");
		let heartbeat_interval =
			Pallet::<T>::get_heartbeat_interval(&blueprint, blueprint_id, service_id).expect("failed to get heartbeat interval");

		frame_system::Pallet::<T>::set_block_number(frame_system::Pallet::<T>::block_number().saturating_add(heartbeat_interval));

		let metrics_data: Vec<u8> = iter::repeat(1u8).take(T::MaxMetricsDataSize::get() as usize).collect();

		let mut message = service_id.to_le_bytes().to_vec();
		message.extend_from_slice(&blueprint_id.to_le_bytes());
		message.extend_from_slice(&frame_system::Pallet::<T>::block_number().saturated_into::<u64>().to_le_bytes());
		message.extend_from_slice(&metrics_data);
		let message_hash = sp_core::hashing::keccak_256(&message);

		// Get the operator's preferences to get their public key
		let operator_preferences = crate::Operators::<T>::get(blueprint_id, operators[0].clone())
			.expect("operator exists");
		let public_key = ecdsa::Public::from_full(&operator_preferences.key)
			.expect("failed to derive public key from operator preferences");

		// Generate the key in the keystore using the same seed as bench_ecdsa_key
		// This ensures the private key is available for signing
		let key_type = KeyTypeId(*b"mdkg");
		let mut seed = [0u8; 32];
		seed.fill(OPERATOR_ID);
		seed[0] = OPERATOR_ID;
		seed[15] = OPERATOR_ID.wrapping_mul(7).wrapping_add(3);
		seed[31] = OPERATOR_ID.wrapping_mul(11).wrapping_add(1);
		let seed_hex = format!("0x{}", hex::encode(seed));
		let _ = sp_io::crypto::ecdsa_generate(key_type, Some(seed_hex.as_bytes().to_vec()));

		let signature = sp_io::crypto::ecdsa_sign(
			key_type,
			&public_key,
			&message_hash
		).expect("failed to sign the message");
	}: _(RawOrigin::Signed(operators[0].clone()), blueprint_id, service_id, metrics_data, signature)
	
	// Slash an operator's stake for a service
	slash {
		let (owner, operators, _, service_id) = prepare_service::<T>();
		let service = Pallet::<T>::services(service_id).unwrap();
		log::debug!("[SLASH BENCHMARK] service_id: {:?}, blueprint: {:?}", service_id, service.blueprint);
		
		let query_result = Pallet::<T>::query_slashing_origin(&service);
		log::debug!("[SLASH BENCHMARK] query_slashing_origin result: {:?}", query_result);
		
		let slash_origin = match query_result {
			Ok((maybe_origin, weight)) => {
				log::debug!("[SLASH BENCHMARK] query succeeded, maybe_origin: {:?}, weight: {:?}", maybe_origin, weight);
				if let Some(origin) = maybe_origin {
					log::debug!("[SLASH BENCHMARK] slash_origin found: {:?}", origin);
					log::debug!("[SLASH BENCHMARK] calling slash with origin: {:?}, operator: {:?}, service_id: {:?}", origin, operators[0], service_id);
					origin
				} else {
					log::debug!("[SLASH BENCHMARK] ERROR: query_slashing_origin returned None - no slashing origin found");
					panic!("No slashing origin found for service {}", service_id);
				}
			},
			Err(e) => {
				log::debug!("[SLASH BENCHMARK] ERROR: query_slashing_origin failed with error: {:?}", e);
				panic!("query_slashing_origin failed: {:?}", e);
			}
		};
	}: _(RawOrigin::Signed(slash_origin.clone()), operators[0].clone(), 0, Percent::from_percent(50))

	// Dispute a scheduled slash
	dispute {
		let (owner, operators, _, service_id) = prepare_service::<T>();
		let service = Pallet::<T>::services(service_id).unwrap();
		log::debug!("[DISPUTE BENCHMARK] service_id: {:?}, blueprint: {:?}", service_id, service.blueprint);
		
		let slash_query_result = Pallet::<T>::query_slashing_origin(&service);
		log::debug!("[DISPUTE BENCHMARK] query_slashing_origin result: {:?}", slash_query_result);
		
		let slash_origin = match slash_query_result {
			Ok((maybe_origin, weight)) => {
				log::debug!("[DISPUTE BENCHMARK] query_slashing_origin succeeded, maybe_origin: {:?}, weight: {:?}", maybe_origin, weight);
				if let Some(origin) = maybe_origin {
					origin
				} else {
					log::debug!("[DISPUTE BENCHMARK] ERROR: query_slashing_origin returned None");
					panic!("No slashing origin found for service {}", service_id);
				}
			},
			Err(e) => {
				log::debug!("[DISPUTE BENCHMARK] ERROR: query_slashing_origin failed: {:?}", e);
				panic!("query_slashing_origin failed: {:?}", e);
			}
		};
		log::debug!("[DISPUTE BENCHMARK] slash_origin: {:?}", slash_origin);
		
		assert_ok!(Pallet::<T>::slash(RawOrigin::Signed(slash_origin.clone()).into(), operators[0].clone(), 0, Percent::from_percent(50)));
		
		let dispute_query_result = Pallet::<T>::query_dispute_origin(&service);
		log::debug!("[DISPUTE BENCHMARK] query_dispute_origin result: {:?}", dispute_query_result);
		
		let dispute_origin = match dispute_query_result {
			Ok((maybe_origin, weight)) => {
				log::debug!("[DISPUTE BENCHMARK] query_dispute_origin succeeded, maybe_origin: {:?}, weight: {:?}", maybe_origin, weight);
				if let Some(origin) = maybe_origin {
					origin
				} else {
					log::debug!("[DISPUTE BENCHMARK] ERROR: query_dispute_origin returned None");
					panic!("No dispute origin found for service {}", service_id);
				}
			},
			Err(e) => {
				log::debug!("[DISPUTE BENCHMARK] ERROR: query_dispute_origin failed: {:?}", e);
				panic!("query_dispute_origin failed: {:?}", e);
			}
		};
		log::debug!("[DISPUTE BENCHMARK] dispute_origin: {:?}", dispute_origin);
	}: _(RawOrigin::Signed(dispute_origin.clone()), 0, 0)

	// Update master blueprint service manager
	update_master_blueprint_service_manager {
	}: _(RawOrigin::Root, H160::zero())

	// Update default heartbeat threshold
	update_default_heartbeat_threshold {
		let threshold: u8 = 50;
	}: _(RawOrigin::Root, threshold)

	// Update default heartbeat interval
	update_default_heartbeat_interval {
		let interval: BlockNumberFor<T> = 100_u32.into();
	}: _(RawOrigin::Root, interval)

	// Update default heartbeat slashing window
	update_default_heartbeat_slashing_window {
		let window: BlockNumberFor<T> = 1000_u32.into();
	}: _(RawOrigin::Root, window)

	// Join a service as an operator
	join_service {
		let (alice, mut operators, _) = prepare_blueprint_with_operators::<T>(&[2, 3, 4]);
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
			MembershipModel::Dynamic { min_operators: 2, max_operators: None },
		));

		let service_id = Pallet::<T>::next_instance_id();
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
	}: _(RawOrigin::Signed(bob.clone()), service_id, security_commitments)

	// Leave a service as an operator
	leave_service {
		let (alice, mut operators, _) = prepare_blueprint_with_operators::<T>(&[2, 3, 4]);
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
			MembershipModel::Dynamic { min_operators: 2, max_operators: None },
		));

		let service_id = Pallet::<T>::next_instance_id();

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

		assert_ok!(Pallet::<T>::join_service(
			RawOrigin::Signed(bob.clone()).into(),
			service_id,
			security_commitments
		));

	}: _(RawOrigin::Signed(bob.clone()), service_id)

	// Benchmark payment validation for pay-once services
	validate_payment_amount_pay_once {
		let alice = funded_account::<T>(1u8);
		let blueprint = cggmp21_blueprint::<T>();
		assert_ok!(create_test_blueprint::<T>(RawOrigin::Signed(alice.clone()).into(), blueprint));

		let (_, blueprint) = Pallet::<T>::blueprints(0).expect("blueprint exists");
		let amount = 1000_u32.into();
	}: {
		let _ = Pallet::<T>::validate_payment_amount(&blueprint, amount);
	}

	// Benchmark payment processing for subscription services
	process_subscription_payment {
		let (alice, mut operators, _) = prepare_blueprint_with_operators::<T>(&[2]);
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
		let (alice, mut operators, _) = prepare_blueprint_with_operators::<T>(&[2]);
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
		let (alice, mut operators, _) = prepare_blueprint_with_operators::<T>(&[2]);
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

	// Trigger subscription payment manually
	trigger_subscription_payment {
		let (owner, operators, blueprint_id, service_id) = prepare_service::<T>();
		
		// Modify blueprint to have subscription pricing
		let (_, mut blueprint) = Pallet::<T>::blueprints(blueprint_id).expect("blueprint exists");
		let interval: BlockNumberFor<T> = 10u32.into();
		blueprint.jobs[0].pricing_model = PricingModel::Subscription {
			rate_per_interval: 100u128,
			interval: interval.saturated_into(),
			maybe_end: None,
		};
		// Update the blueprint storage
		<Blueprints<T>>::insert(blueprint_id, (owner.clone(), blueprint));

		// Call the job to create subscription billing
		// The billing will be created but may not be saved if payment is not due
		let current_block = frame_system::Pallet::<T>::block_number();
		assert_ok!(Pallet::<T>::call(
			RawOrigin::Signed(owner.clone()).into(),
			service_id,
			0u8,
			vec![Field::Uint8(2)].try_into().unwrap()
		));

		// Ensure billing exists - if it wasn't created by the call, create it manually
		let billing_key = (service_id, 0u8, owner.clone());
		if !<JobSubscriptionBillings<T>>::contains_key(&billing_key) {
			use tangle_primitives::services::JobSubscriptionBilling;
			let billing = JobSubscriptionBilling {
				service_id,
				job_index: 0u8,
				subscriber: owner.clone(),
				last_billed: current_block.saturating_sub(interval), // Set to past so payment is due
				end_block: None,
			};
			<JobSubscriptionBillings<T>>::insert(&billing_key, billing);
			// Update subscription count
			let current_count = <UserSubscriptionCount<T>>::get(&owner);
			<UserSubscriptionCount<T>>::insert(&owner, current_count + 1);
		}

		// Advance blocks so payment is due (interval is 10)
		let target_block = current_block.saturating_add(interval);
		frame_system::Pallet::<T>::set_block_number(target_block);

		ensure_account_ready::<T>(&T::RewardRecorder::account_id());
	}: _(RawOrigin::Signed(owner.clone()), service_id, 0u8)
}

// Define the module and associated types for the benchmarks
impl_benchmark_test_suite!(
	Pallet,
	crate::mock::new_test_ext(vec![1, 2, 3, 4]),
	crate::mock::Runtime,
);
