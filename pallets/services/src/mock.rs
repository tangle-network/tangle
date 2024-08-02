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
#![allow(clippy::all)]
use super::*;
use crate::{self as pallet_services};
use frame_election_provider_support::{
	bounds::{ElectionBounds, ElectionBoundsBuilder},
	onchain, SequentialPhragmen,
};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU128, ConstU32, ConstU64, Contains, Everything, OneSessionHandler},
};
use mock_evm::MockedEvmRunner;
use pallet_evm::GasWeightMapping;
use pallet_session::historical as pallet_session_historical;
use sp_core::{sr25519, H160, H256};
use sp_keystore::{testing::MemoryKeystore, KeystoreExt, KeystorePtr};
use sp_runtime::{
	app_crypto::ecdsa::Public,
	testing::UintAuthorityId,
	traits::{ConvertInto, IdentityLookup},
	AccountId32, BuildStorage, Perbill,
};

use std::collections::BTreeMap;
use std::sync::Arc;

pub type AccountId = AccountId32;
pub type Balance = u128;
pub type BlockNumber = u64;

impl frame_system::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Block = Block;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type RuntimeTask = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = ();
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
}

parameter_types! {
	pub ElectionBoundsOnChain: ElectionBounds = ElectionBoundsBuilder::default()
		.voters_count(5_000.into()).targets_count(1_250.into()).build();
	pub ElectionBoundsMultiPhase: ElectionBounds = ElectionBoundsBuilder::default()
		.voters_count(10_000.into()).targets_count(1_500.into()).build();
}

impl pallet_session::historical::Config for Runtime {
	type FullIdentification = AccountId;
	type FullIdentificationOf = ConvertInto;
}

pub struct BaseFilter;
impl Contains<RuntimeCall> for BaseFilter {
	fn contains(call: &RuntimeCall) -> bool {
		let is_stake_unbond_call =
			matches!(call, RuntimeCall::Staking(pallet_staking::Call::unbond { .. }));

		if is_stake_unbond_call {
			// no unbond call
			return false;
		}

		// no chill call
		if matches!(call, RuntimeCall::Staking(pallet_staking::Call::chill { .. })) {
			return false;
		}

		// no withdraw_unbonded call
		let is_stake_withdraw_call =
			matches!(call, RuntimeCall::Staking(pallet_staking::Call::withdraw_unbonded { .. }));

		if is_stake_withdraw_call {
			return false;
		}

		true
	}
}

sp_runtime::impl_opaque_keys! {
	pub struct MockSessionKeys {
		pub other: MockSessionHandler,
	}
}

pub struct MockSessionHandler;
impl OneSessionHandler<AccountId> for MockSessionHandler {
	type Key = UintAuthorityId;

	fn on_genesis_session<'a, I: 'a>(_: I)
	where
		I: Iterator<Item = (&'a AccountId, Self::Key)>,
		AccountId: 'a,
	{
	}

	fn on_new_session<'a, I: 'a>(_: bool, _: I, _: I)
	where
		I: Iterator<Item = (&'a AccountId, Self::Key)>,
		AccountId: 'a,
	{
	}

	fn on_disabled(_validator_index: u32) {}
}

impl sp_runtime::BoundToRuntimeAppPublic for MockSessionHandler {
	type Public = UintAuthorityId;
}

pub struct MockSessionManager;

impl pallet_session::SessionManager<AccountId> for MockSessionManager {
	fn end_session(_: sp_staking::SessionIndex) {}
	fn start_session(_: sp_staking::SessionIndex) {}
	fn new_session(idx: sp_staking::SessionIndex) -> Option<Vec<AccountId>> {
		if idx == 0 || idx == 1 || idx == 2 {
			Some(vec![mock_pub_key(1), mock_pub_key(2), mock_pub_key(3), mock_pub_key(4)])
		} else {
			None
		}
	}
}

parameter_types! {
	pub const Period: u64 = 1;
	pub const Offset: u64 = 0;
}

impl pallet_session::Config for Runtime {
	type SessionManager = MockSessionManager;
	type Keys = MockSessionKeys;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionHandler = (MockSessionHandler,);
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = AccountId;
	type ValidatorIdOf = pallet_staking::StashOf<Runtime>;
	type WeightInfo = ();
}

pub struct OnChainSeqPhragmen;
impl onchain::Config for OnChainSeqPhragmen {
	type System = Runtime;
	type Solver = SequentialPhragmen<AccountId, Perbill>;
	type DataProvider = Staking;
	type WeightInfo = ();
	type MaxWinners = ConstU32<100>;
	type Bounds = ElectionBoundsOnChain;
}

/// Upper limit on the number of NPOS nominations.
const MAX_QUOTA_NOMINATIONS: u32 = 16;

impl pallet_staking::Config for Runtime {
	type Currency = Balances;
	type CurrencyBalance = <Self as pallet_balances::Config>::Balance;
	type UnixTime = pallet_timestamp::Pallet<Self>;
	type CurrencyToVote = ();
	type RewardRemainder = ();
	type RuntimeEvent = RuntimeEvent;
	type Slash = ();
	type Reward = ();
	type SessionsPerEra = ();
	type SlashDeferDuration = ();
	type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type BondingDuration = ();
	type SessionInterface = ();
	type EraPayout = ();
	type NextNewSession = Session;
	type MaxExposurePageSize = ConstU32<64>;
	type MaxControllersInDeprecationBatch = ConstU32<100>;
	type OffendingValidatorsThreshold = ();
	type ElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type GenesisElectionProvider = Self::ElectionProvider;
	type VoterList = pallet_staking::UseNominatorsAndValidatorsMap<Self>;
	type TargetList = pallet_staking::UseValidatorsMap<Self>;
	type MaxUnlockingChunks = ConstU32<32>;
	type HistoryDepth = ConstU32<84>;
	type EventListeners = ();
	type BenchmarkingConfig = pallet_staking::TestBenchmarkingConfig;
	type NominationsQuota = pallet_staking::FixedNominationsQuota<MAX_QUOTA_NOMINATIONS>;
	type WeightInfo = ();
}

parameter_types! {
	pub const ServicesPalletId: PalletId = PalletId(*b"py/servs");
}

pub struct PalletEVMGasWeightMapping;

impl EvmGasWeightMapping for PalletEVMGasWeightMapping {
	fn gas_to_weight(gas: u64, without_base_weight: bool) -> Weight {
		pallet_evm::FixedGasWeightMapping::<Runtime>::gas_to_weight(gas, without_base_weight)
	}

	fn weight_to_gas(weight: Weight) -> u64 {
		pallet_evm::FixedGasWeightMapping::<Runtime>::weight_to_gas(weight)
	}
}

parameter_types! {
	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxFields: u32 = 256;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxFieldsSize: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxMetadataLength: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxJobsPerService: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxOperatorsPerService: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxPermittedCallers: u32 = 256;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxServicesPerOperator: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxBlueprintsPerOperator: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxServicesPerUser: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxBinariesPerGadget: u32 = 64;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxSourcesPerGadget: u32 = 64;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxGitOwnerLength: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxGitRepoLength: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxGitTagLength: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxBinaryNameLength: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxIpfsHashLength: u32 = 46;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxContainerRegistryLength: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxContainerImageNameLength: u32 = 1024;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxContainerImageTagLength: u32 = 1024;
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Currency = Balances;
	type PalletId = ServicesPalletId;
	type EvmRunner = MockedEvmRunner;
	type EvmGasWeightMapping = PalletEVMGasWeightMapping;
	type MaxFields = MaxFields;
	type MaxFieldsSize = MaxFieldsSize;
	type MaxMetadataLength = MaxMetadataLength;
	type MaxJobsPerService = MaxJobsPerService;
	type MaxOperatorsPerService = MaxOperatorsPerService;
	type MaxPermittedCallers = MaxPermittedCallers;
	type MaxServicesPerOperator = MaxServicesPerOperator;
	type MaxBlueprintsPerOperator = MaxBlueprintsPerOperator;
	type MaxServicesPerUser = MaxServicesPerUser;
	type MaxBinariesPerGadget = MaxBinariesPerGadget;
	type MaxSourcesPerGadget = MaxSourcesPerGadget;
	type MaxGitOwnerLength = MaxGitOwnerLength;
	type MaxGitRepoLength = MaxGitRepoLength;
	type MaxGitTagLength = MaxGitTagLength;
	type MaxBinaryNameLength = MaxBinaryNameLength;
	type MaxIpfsHashLength = MaxIpfsHashLength;
	type MaxContainerRegistryLength = MaxContainerRegistryLength;
	type MaxContainerImageNameLength = MaxContainerImageNameLength;
	type MaxContainerImageTagLength = MaxContainerImageTagLength;
	type Constraints = pallet_services::types::ConstraintsOf<Self>;
	type WeightInfo = ();
}

type Block = frame_system::mocking::MockBlock<Runtime>;

#[allow(non_camel_case_types)]
construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Services: pallet_services,
		EVM: pallet_evm,
		Ethereum: pallet_ethereum,
		Session: pallet_session,
		Staking: pallet_staking,
		Historical: pallet_session_historical,
	}
);

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

pub fn mock_pub_key(id: u8) -> AccountId {
	sr25519::Public::from_raw([id; 32]).into()
}

pub fn mock_authorities(vec: Vec<u8>) -> Vec<AccountId> {
	vec.into_iter().map(|id| mock_pub_key(id)).collect()
}

pub fn new_test_ext(ids: Vec<u8>) -> sp_io::TestExternalities {
	new_test_ext_raw_authorities(mock_authorities(ids))
}

pub const CGGMP21_REGISTRATION_HOOK: H160 = H160([0x21; 20]);
pub const CGGMP21_REQUEST_HOOK: H160 = H160([0x22; 20]);
pub const CGGMP21_JOB_RESULT_VERIFIER: H160 = H160([0x23; 20]);

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext_raw_authorities(authorities: Vec<(AccountId)>) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
	// We use default for brevity, but you can configure as desired if needed.
	let balances: Vec<_> = authorities.iter().map(|i| (i.clone(), 20_000_u128)).collect();
	pallet_balances::GenesisConfig::<Runtime> { balances }
		.assimilate_storage(&mut t)
		.unwrap();

	let stakers: Vec<_> = authorities
		.iter()
		.map(|authority| {
			(
				authority.clone(),
				authority.clone(),
				10_000_u128,
				pallet_staking::StakerStatus::<AccountId>::Validator,
			)
		})
		.collect();

	let staking_config = pallet_staking::GenesisConfig::<Runtime> {
		stakers,
		validator_count: 4,
		force_era: pallet_staking::Forcing::ForceNew,
		minimum_validator_count: 0,
		max_validator_count: Some(5),
		max_nominator_count: Some(5),
		invulnerables: vec![],
		..Default::default()
	};

	staking_config.assimilate_storage(&mut t).unwrap();

	let mut evm_accounts = BTreeMap::new();

	let cggmp21_registration_hook_json: serde_json::Value = serde_json::from_str(include_str!(
		"../../../forge/out/CGGMP21Hooks.sol/CGGMP21RegistrationHook.json"
	))
	.unwrap();
	let cggmp21_registration_hook_code = hex::decode(
		cggmp21_registration_hook_json["deployedBytecode"]["object"]
			.as_str()
			.unwrap()
			.replace("0x", ""),
	)
	.unwrap();
	evm_accounts.insert(
		CGGMP21_REGISTRATION_HOOK,
		fp_evm::GenesisAccount {
			code: cggmp21_registration_hook_code,
			storage: Default::default(),
			nonce: Default::default(),
			balance: Default::default(),
		},
	);

	let cggmp21_request_hook_json: serde_json::Value = serde_json::from_str(include_str!(
		"../../../forge/out/CGGMP21Hooks.sol/CGGMP21RequestHook.json"
	))
	.unwrap();

	let cggmp21_request_hook_code = hex::decode(
		cggmp21_request_hook_json["deployedBytecode"]["object"]
			.as_str()
			.unwrap()
			.replace("0x", ""),
	)
	.unwrap();

	evm_accounts.insert(
		CGGMP21_REQUEST_HOOK,
		fp_evm::GenesisAccount {
			code: cggmp21_request_hook_code,
			storage: Default::default(),
			nonce: Default::default(),
			balance: Default::default(),
		},
	);

	let cggmp21_job_result_verifier_json: serde_json::Value = serde_json::from_str(include_str!(
		"../../../forge/out/CGGMP21Hooks.sol/CGGMP21JobResultVerifier.json"
	))
	.unwrap();

	let cggmp21_job_result_verifier_code = hex::decode(
		cggmp21_job_result_verifier_json["deployedBytecode"]["object"]
			.as_str()
			.unwrap()
			.replace("0x", ""),
	)
	.unwrap();

	evm_accounts.insert(
		CGGMP21_JOB_RESULT_VERIFIER,
		fp_evm::GenesisAccount {
			code: cggmp21_job_result_verifier_code,
			storage: Default::default(),
			nonce: Default::default(),
			balance: Default::default(),
		},
	);

	let evm_config =
		pallet_evm::GenesisConfig::<Runtime> { accounts: evm_accounts, ..Default::default() };

	evm_config.assimilate_storage(&mut t).unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.register_extension(KeystoreExt(Arc::new(MemoryKeystore::new()) as KeystorePtr));
	ext.execute_with(|| System::set_block_number(1));
	ext.execute_with(|| {
		System::set_block_number(1);
		Session::on_initialize(1);
		<Staking as Hooks<u64>>::on_initialize(1);
	});

	ext
}

// Checks events against the latest. A contiguous set of events must be
// provided. They must include the most recent RuntimeEvent, but do not have to include
// every past RuntimeEvent.
#[track_caller]
pub fn assert_events(mut expected: Vec<RuntimeEvent>) {
	let mut actual: Vec<RuntimeEvent> = System::events().iter().map(|e| e.event.clone()).collect();

	expected.reverse();
	for evt in expected {
		let next = actual.pop().expect("RuntimeEvent expected");
		match (&next, &evt) {
			(left_val, right_val) => {
				if !(*left_val == *right_val) {
					panic!("Events don't match\nactual: {next:#?}\nexpected: {evt:#?}");
				}
			},
		};
	}
}
