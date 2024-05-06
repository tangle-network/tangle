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

use super::*;
use crate::{self as pallet_services};
use frame_election_provider_support::{
	bounds::{ElectionBounds, ElectionBoundsBuilder},
	onchain, SequentialPhragmen,
};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU128, ConstU32, ConstU64, Contains, Everything},
};
use mock_evm::MockedEvmRunner;
use pallet_session::historical as pallet_session_historical;
use sp_core::{sr25519, H160, H256};
use sp_runtime::{
	app_crypto::ecdsa::Public,
	traits::{ConvertInto, IdentityLookup, OpaqueKeys},
	AccountId32, BuildStorage, DispatchResult, Perbill, Percent,
};
use sp_staking::{
	offence::{OffenceError, ReportOffence},
	SessionIndex,
};
use std::collections::BTreeMap;

use tangle_crypto_primitives::crypto::AuthorityId as RoleKeyId;
use tangle_primitives::{
	jobs::{traits::JobsHandler, *},
	roles::{traits::RolesHandler, ValidatorRewardDistribution},
};

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

type IdentificationTuple = (AccountId, AccountId);
type Offence = pallet_roles::offences::ValidatorOffence<IdentificationTuple>;

parameter_types! {
	pub static Offences: Vec<(Vec<AccountId>, Offence)> = vec![];
	pub ElectionBoundsOnChain: ElectionBounds = ElectionBoundsBuilder::default()
		.voters_count(5_000.into()).targets_count(1_250.into()).build();
	pub ElectionBoundsMultiPhase: ElectionBounds = ElectionBoundsBuilder::default()
		.voters_count(10_000.into()).targets_count(1_500.into()).build();
}

/// A mock offence report handler.
pub struct OffenceHandler;
impl ReportOffence<AccountId, IdentificationTuple, Offence> for OffenceHandler {
	fn report_offence(reporters: Vec<AccountId>, offence: Offence) -> Result<(), OffenceError> {
		Offences::mutate(|l| l.push((reporters, offence)));
		Ok(())
	}

	fn is_known_offence(_offenders: &[IdentificationTuple], _time_slot: &SessionIndex) -> bool {
		false
	}
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
		pub role: pallet_roles::Pallet<Runtime>,
	}
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

// LEGACY: This is a legacy implementation of the RolesHandler trait.
pub struct MockedJobsHandler;

impl JobsHandler<AccountId> for MockedJobsHandler {
	fn get_active_jobs(validator: AccountId) -> Vec<(tangle_primitives::roles::RoleType, JobId)> {
		todo!()
	}

	fn exit_from_known_set(
		validator: AccountId,
		role_type: tangle_primitives::roles::RoleType,
		job_id: JobId,
	) -> DispatchResult {
		todo!()
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
	type SessionHandler = <MockSessionKeys as OpaqueKeys>::KeyTypeIdProviders;
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
	type RolesHandler = Roles;
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
	pub MaxRestake : Percent = Percent::from_percent(50);
	pub Reward : ValidatorRewardDistribution = ValidatorRewardDistribution::try_new(Percent::from_rational(1_u32,2_u32), Percent::from_rational(1_u32,2_u32)).unwrap();
	#[derive(Clone, Debug, Eq, PartialEq, TypeInfo)]
	pub const MaxActiveJobsPerValidator: u32 = 10;
}

impl pallet_roles::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type JobsHandler = MockedJobsHandler;
	type MaxRolesPerAccount = ConstU32<2>;
	type RoleKeyId = RoleKeyId;
	type ValidatorRewardDistribution = Reward;
	type ValidatorSet = Historical;
	type ReportOffences = OffenceHandler;
	type MaxKeyLen = ConstU32<33>;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxValidators = ConstU32<100>;
	type MaxActiveJobsPerValidator = MaxActiveJobsPerValidator;
	type MaxRestake = MaxRestake;
	type RestakerEraPayout = ();
	type MaxRolesPerValidator = ConstU32<15>;
	type WeightInfo = ();
}

parameter_types! {
	pub const ServicesPalletId: PalletId = PalletId(*b"py/servs");
	pub const RuntimeEvmAddress: H160 = H160(hex_literal::hex!("00000000000000000000000000000000000000f1"));
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Currency = Balances;
	type PalletId = ServicesPalletId;
	type RuntimeEvmAddress = RuntimeEvmAddress;
	type EvmRunner = MockedEvmRunner;
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
		Roles: pallet_roles,
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

pub fn mock_role_key_id(id: u8) -> RoleKeyId {
	RoleKeyId::from(Public::from_raw([id; 33]))
}

pub fn mock_authorities(vec: Vec<u8>) -> Vec<(AccountId, RoleKeyId)> {
	vec.into_iter().map(|id| (mock_pub_key(id), mock_role_key_id(id))).collect()
}

pub fn new_test_ext(ids: Vec<u8>) -> sp_io::TestExternalities {
	new_test_ext_raw_authorities(mock_authorities(ids))
}

pub const CGGMP21_REGISTRATION_HOOK: H160 = H160([0x21; 20]);
pub const CGGMP21_REQUEST_HOOK: H160 = H160([0x22; 20]);

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext_raw_authorities(
	authorities: Vec<(AccountId, RoleKeyId)>,
) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
	// We use default for brevity, but you can configure as desired if needed.
	let balances: Vec<_> = authorities.iter().map(|(i, _)| (i.clone(), 20_000_u128)).collect();
	pallet_balances::GenesisConfig::<Runtime> { balances }
		.assimilate_storage(&mut t)
		.unwrap();

	let session_keys: Vec<_> = authorities
		.iter()
		.map(|(id, role_id)| (id.clone(), id.clone(), MockSessionKeys { role: role_id.clone() }))
		.collect();

	pallet_session::GenesisConfig::<Runtime> { keys: session_keys }
		.assimilate_storage(&mut t)
		.unwrap();

	let stakers: Vec<_> = authorities
		.iter()
		.map(|(authority, _)| {
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

	let evm_config =
		pallet_evm::GenesisConfig::<Runtime> { accounts: evm_accounts, ..Default::default() };

	evm_config.assimilate_storage(&mut t).unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
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
