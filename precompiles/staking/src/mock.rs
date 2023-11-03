// Copyright 2022 Webb Technologies Inc.
//
// This file is part of pallet-evm-precompile-staking package, originally developed by Purestake
// Inc. Pallet-evm-precompile-staking package used in Tangle Network in terms of GPLv3.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Test utilities
#![allow(dead_code)]
use super::*;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, OnFinalize, OnInitialize},
	weights::Weight,
	BasicExternalities,
};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use precompile_utils::precompile_set::*;
use sp_staking::currency_to_vote::U128CurrencyToVote;
use std::{sync::Arc, vec};

use pallet_session::historical as pallet_session_historical;
use pallet_staking::EraPayout;
use serde::{Deserialize, Serialize};
use sp_core::{
	self,
	ecdsa::Public,
	sr25519::{self, Public as sr25519Public, Signature},
	ConstU32, Get, H160, H256, U256,
};

use frame_election_provider_support::{onchain, SequentialPhragmen};
use pallet_staking::{
	Config, ConvertCurve, TestBenchmarkingConfig, UseNominatorsAndValidatorsMap, UseValidatorsMap,
};
use sp_io::TestExternalities;
use sp_keystore::{testing::MemoryKeystore, KeystoreExt, KeystorePtr};
use sp_runtime::{
	curve::PiecewiseLinear,
	impl_opaque_keys,
	testing::TestXt,
	traits::{
		self, BlakeTwo256, ConvertInto, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup,
		OpaqueKeys, Verify, Zero,
	},
	AccountId32, BuildStorage, Perbill, Percent,
};

pub use dkg_runtime_primitives::{
	crypto::AuthorityId as DKGId, ConsensusLog, MaxAuthorities, MaxKeyLength, MaxProposalLength,
	MaxReporters, MaxSignatureLength, DKG_ENGINE_ID,
};

impl_opaque_keys! {
	pub struct MockSessionKeys {
		pub dummy: pallet_dkg_metadata::Pallet<Runtime>,
	}
}

pub type Balance = u128;
pub type BlockNumber = u64;
pub type EraIndex = u32;
pub type SessionIndex = u32;

pub const BLOCK_TIME: u64 = 1000;

type Block = frame_system::mocking::MockBlock<Runtime>;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

const PRECOMPILE_ADDRESS_BYTES: [u8; 32] = [
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6,
];

#[derive(
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Clone,
	Encode,
	Decode,
	Debug,
	MaxEncodedLen,
	Serialize,
	Deserialize,
	derive_more::Display,
	scale_info::TypeInfo,
)]
pub enum TestAccount {
	Empty,
	Alex,
	Bobo,
	Dave,
	Charlie,
	Eve,
	PrecompileAddress,
}

impl Default for TestAccount {
	fn default() -> Self {
		Self::Empty
	}
}

// needed for associated type in pallet_evm
impl AddressMapping<AccountId32> for TestAccount {
	fn into_account_id(h160_account: H160) -> AccountId32 {
		match h160_account {
			a if a == H160::repeat_byte(0x01) => TestAccount::Alex.into(),
			a if a == H160::repeat_byte(0x02) => TestAccount::Bobo.into(),
			a if a == H160::repeat_byte(0x03) => TestAccount::Charlie.into(),
			a if a == H160::repeat_byte(0x04) => TestAccount::Dave.into(),
			a if a == H160::repeat_byte(0x05) => TestAccount::Eve.into(),
			a if a == H160::from_low_u64_be(6) => TestAccount::PrecompileAddress.into(),
			_ => TestAccount::Empty.into(),
		}
	}
}

impl AddressMapping<sp_core::sr25519::Public> for TestAccount {
	fn into_account_id(h160_account: H160) -> sp_core::sr25519::Public {
		match h160_account {
			a if a == H160::repeat_byte(0x01) => sr25519Public::from_raw([1u8; 32]),
			a if a == H160::repeat_byte(0x02) => sr25519Public::from_raw([2u8; 32]),
			a if a == H160::repeat_byte(0x03) => sr25519Public::from_raw([3u8; 32]),
			a if a == H160::repeat_byte(0x04) => sr25519Public::from_raw([4u8; 32]),
			a if a == H160::repeat_byte(0x05) => sr25519Public::from_raw([5u8; 32]),
			a if a == H160::from_low_u64_be(6) => sr25519Public::from_raw(PRECOMPILE_ADDRESS_BYTES),
			_ => sr25519Public::from_raw([0u8; 32]),
		}
	}
}

impl From<TestAccount> for H160 {
	fn from(x: TestAccount) -> H160 {
		match x {
			TestAccount::Alex => H160::repeat_byte(0x01),
			TestAccount::Bobo => H160::repeat_byte(0x02),
			TestAccount::Charlie => H160::repeat_byte(0x03),
			TestAccount::Dave => H160::repeat_byte(0x04),
			TestAccount::Eve => H160::repeat_byte(0x05),
			TestAccount::PrecompileAddress => H160::from_low_u64_be(6),
			_ => Default::default(),
		}
	}
}
trait H160Conversion {
	fn to_h160(&self) -> H160;
}

impl H160Conversion for AccountId32 {
	fn to_h160(&self) -> H160 {
		let x = self.encode()[31];
		H160::repeat_byte(x)
	}
}

impl From<TestAccount> for AccountId32 {
	fn from(x: TestAccount) -> Self {
		match x {
			TestAccount::Alex => AccountId32::from([1u8; 32]),
			TestAccount::Bobo => AccountId32::from([2u8; 32]),
			TestAccount::Charlie => AccountId32::from([3u8; 32]),
			TestAccount::Dave => AccountId32::from([4u8; 32]),
			TestAccount::Eve => AccountId32::from([5u8; 32]),
			TestAccount::PrecompileAddress => AccountId32::from(PRECOMPILE_ADDRESS_BYTES),
			_ => AccountId32::from([0u8; 32]),
		}
	}
}

impl From<TestAccount> for sp_core::sr25519::Public {
	fn from(x: TestAccount) -> Self {
		match x {
			TestAccount::Alex => sr25519Public::from_raw([1u8; 32]),
			TestAccount::Bobo => sr25519Public::from_raw([2u8; 32]),
			TestAccount::Charlie => sr25519Public::from_raw([3u8; 32]),
			TestAccount::Dave => sr25519Public::from_raw([4u8; 32]),
			TestAccount::Eve => sr25519Public::from_raw([5u8; 32]),
			TestAccount::PrecompileAddress => sr25519Public::from_raw(PRECOMPILE_ADDRESS_BYTES),
			_ => sr25519Public::from_raw([0u8; 32]),
		}
	}
}

construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		Timestamp: pallet_timestamp,
		DKGMetadata: pallet_dkg_metadata,
		Session: pallet_session,
		Staking: pallet_staking,
		Historical: pallet_session_historical,
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl pallet_dkg_metadata::Config for Runtime {
	type DKGId = DKGId;
	type RuntimeEvent = RuntimeEvent;
	type OnAuthoritySetChangeHandler = ();
	type OnDKGPublicKeyChangeHandler = ();
	type OffChainAuthId = dkg_runtime_primitives::offchain::crypto::OffchainAuthId;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type DKGAuthorityToMerkleLeaf = pallet_dkg_proposals::DKGEcdsaToEthereumAddress;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type KeygenJailSentence = Period;
	type SigningJailSentence = Period;
	type DecayPercentage = DecayPercentage;
	type Reputation = u128;
	type UnsignedInterval = frame_support::traits::ConstU64<0>;
	type UnsignedPriority = frame_support::traits::ConstU64<1000>;
	type AuthorityIdOf = pallet_dkg_metadata::AuthorityIdOf<Self>;
	type ProposalHandler = ();
	type SessionPeriod = Period;
	type MaxKeyLength = MaxKeyLength;
	type MaxSignatureLength = MaxSignatureLength;
	type MaxReporters = MaxReporters;
	type MaxAuthorities = MaxAuthorities;
	type VoteLength = VoteLength;
	type MaxProposalLength = MaxProposalLength;
	type WeightInfo = ();
}

parameter_types! {
	pub const DecayPercentage: Percent = Percent::from_percent(50);
	#[derive(Default, Clone, Encode, Decode, Debug, Eq, PartialEq, scale_info::TypeInfo, Ord, PartialOrd, MaxEncodedLen)]
	pub const VoteLength: u32 = 64;
}

impl pallet_session::Config for Runtime {
	type SessionManager = Staking;
	type Keys = MockSessionKeys;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionHandler = <MockSessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = AccountId;
	type ValidatorIdOf = ConvertInto;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type WeightInfo = ();
}

parameter_types! {
	pub static SessionsPerEra: SessionIndex = 3;
	pub static ExistentialDeposit: Balance = 1;
	pub static SlashDeferDuration: EraIndex = 0;
	pub static Period: BlockNumber = 5;
	pub static Offset: BlockNumber = 0;

}

impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type Block = Block;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Runtime {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 4];
	type MaxLocks = ();
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxHolds = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
}

parameter_types! {
	pub static MaxNominations: u32 = 16;
	pub static HistoryDepth: u32 = 80;
	pub static MaxUnlockingChunks: u32 = 32;
	pub static RewardOnUnbalanceWasCalled: bool = false;
	pub static MaxWinners: u32 = 100;
}

impl pallet_session::historical::Config for Runtime {
	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

pallet_staking_reward_curve::build! {
	const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
		min_inflation: 0_025_000,
		max_inflation: 0_100_000,
		ideal_stake: 0_500_000,
		falloff: 0_050_000,
		max_piece_count: 40,
		test_precision: 0_005_000,
	);
}
parameter_types! {
	pub const BondingDuration: EraIndex = 3;
	pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
	pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

pub type Precompiles<R> =
	PrecompileSetBuilder<R, (PrecompileAt<AddressU64<5>, StakingPrecompile<R>>,)>;

pub type PCall = StakingPrecompileCall<Runtime>;

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = TestAccount;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = Precompiles<Runtime>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Runtime>;
}

const MAX_POV_SIZE: u64 = 5 * 1024 * 1024;

parameter_types! {
	pub BlockGasLimit: U256 = U256::from(u64::MAX);
	pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub GasLimitPovSizeRatio: u64 = {
		let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
		block_gas_limit.saturating_div(MAX_POV_SIZE)
	};

}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}

pub struct StakingBenchmarkingConfig;
impl pallet_staking::BenchmarkingConfig for StakingBenchmarkingConfig {
	type MaxValidators = ConstU32<1000>;
	type MaxNominators = ConstU32<1000>;
}

pub struct OnChainSeqPhragmen;
impl onchain::Config for OnChainSeqPhragmen {
	type System = Runtime;
	type Solver = SequentialPhragmen<AccountId, Perbill>;
	type DataProvider = Staking;
	type WeightInfo = ();
	type MaxWinners = ConstU32<100>;
	type VotersBound = ConstU32<{ u32::MAX }>;
	type TargetsBound = ConstU32<{ u32::MAX }>;
}

impl pallet_staking::Config for Runtime {
	type MaxNominations = MaxNominations;
	type Currency = Balances;
	type CurrencyBalance = <Self as pallet_balances::Config>::Balance;
	type UnixTime = Timestamp;
	type CurrencyToVote = U128CurrencyToVote;
	type RewardRemainder = ();
	type RuntimeEvent = RuntimeEvent;
	type Slash = ();
	type Reward = ();
	type SessionsPerEra = SessionsPerEra;
	type SlashDeferDuration = SlashDeferDuration;
	type AdminOrigin = frame_system::EnsureRoot<AccountId>;
	type BondingDuration = BondingDuration;
	type SessionInterface = Self;
	type EraPayout = ConvertCurve<RewardCurve>;
	type NextNewSession = Session;
	type MaxNominatorRewardedPerValidator = ConstU32<64>;
	type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
	type ElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type GenesisElectionProvider = Self::ElectionProvider;
	type VoterList = UseNominatorsAndValidatorsMap<Runtime>;
	type TargetList = UseValidatorsMap<Self>;
	type MaxUnlockingChunks = MaxUnlockingChunks;
	type HistoryDepth = HistoryDepth;
	type BenchmarkingConfig = TestBenchmarkingConfig;
	type EventListeners = ();
	type WeightInfo = ();
}

type Extrinsic = TestXt<RuntimeCall, ()>;

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	type OverarchingCall = RuntimeCall;
	type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		_public: <Signature as traits::Verify>::Signer,
		_account: AccountId,
		nonce: u64,
	) -> Option<(RuntimeCall, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
		Some((call, (nonce, ())))
	}
}

// Note, that we can't use `UintAuthorityId` here. Reason is that the implementation
// of `to_public_key()` assumes, that a public key is 32 bytes long. This is true for
// ed25519 and sr25519 but *not* for ecdsa. An ecdsa public key is 33 bytes.
pub fn mock_dkg_id(id: u8) -> DKGId {
	DKGId::from(Public::from_raw([id; 33]))
}

pub fn mock_pub_key(id: u8) -> AccountId {
	sr25519::Public::from_raw([id; 32])
}

pub fn mock_authorities(vec: Vec<u8>) -> Vec<(AccountId, DKGId)> {
	vec.into_iter().map(|id| (mock_pub_key(id), mock_dkg_id(id))).collect()
}

pub fn new_test_ext(ids: Vec<u8>) -> TestExternalities {
	new_test_ext_raw_authorities(mock_authorities(ids))
}

pub fn new_test_ext_raw_authorities(authorities: Vec<(AccountId, DKGId)>) -> TestExternalities {
	let mut t = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.expect("Frame system builds valid default genesis config");

	let balances: Vec<_> = authorities.iter().map(|i| (i.0, 10_000_000_000)).collect();

	pallet_balances::GenesisConfig::<Runtime> { balances }
		.assimilate_storage(&mut t)
		.unwrap();

	let session_keys: Vec<_> = authorities
		.iter()
		.enumerate()
		.map(|(_, id)| (id.0, id.0, MockSessionKeys { dummy: id.1.clone() }))
		.collect();

	BasicExternalities::execute_with_storage(&mut t, || {
		for (ref id, ..) in &session_keys {
			frame_system::Pallet::<Runtime>::inc_providers(id);
		}
	});

	pallet_session::GenesisConfig::<Runtime> { keys: session_keys }
		.assimilate_storage(&mut t)
		.unwrap();

	// controllers are same as stash
	let stakers: Vec<_> = authorities
		.iter()
		.map(|authority| {
			(
				authority.0,
				authority.0,
				10_000_u128,
				pallet_staking::StakerStatus::<sp_core::sr25519::Public>::Validator,
			)
		})
		.collect();

	let staking_config = pallet_staking::GenesisConfig::<Runtime> {
		stakers,
		validator_count: 2,
		force_era: pallet_staking::Forcing::ForceNew,
		minimum_validator_count: 0,
		max_validator_count: Some(5),
		max_nominator_count: Some(5),
		invulnerables: vec![],
		..Default::default()
	};

	staking_config.assimilate_storage(&mut t).unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	// set to block 1 to test events
	ext.execute_with(|| System::set_block_number(1));
	ext.register_extension(KeystoreExt(Arc::new(MemoryKeystore::new()) as KeystorePtr));
	ext
}

/// Used to run to the specified block number
pub(crate) fn run_to_block(n: BlockNumber) {
	while System::block_number() < n {
		println!("System nlock number : {}", System::block_number());
		System::on_finalize(System::block_number());
		Session::on_finalize(System::block_number());
		Balances::on_finalize(System::block_number());
		Staking::on_finalize(System::block_number());
		DKGMetadata::on_finalize(System::block_number());

		System::set_block_number(System::block_number() + 1);
		Session::on_initialize(System::block_number());
		Staking::on_initialize(System::block_number());
		DKGMetadata::on_initialize(System::block_number());

		let current_era = Staking::current_era().unwrap_or(0);
		println!("current_era : {}", current_era);
	}
}

/// Progresses from the current block number (whatever that may be) to the `P * session_index + 1`.
pub(crate) fn start_session(session_index: SessionIndex) {
	let end: u64 = if Offset::get().is_zero() {
		(session_index as u64) * Period::get()
	} else {
		Offset::get() + (session_index.saturating_sub(1) as u64) * Period::get()
	};
	println!("RUNTOBLOCK: {}", end);
	run_to_block(end);
	// session must have progressed properly.
	assert_eq!(
		Session::current_index(),
		session_index,
		"current session index = {}, expected = {}",
		Session::current_index(),
		session_index,
	);
}

/// Go one session forward.
pub(crate) fn advance_session() {
	let current_index = Session::current_index();
	start_session(current_index + 1);
}

/// Progress until the given era.
pub(crate) fn start_active_era(era_index: EraIndex) {
	start_session(era_index * <SessionsPerEra as Get<u32>>::get());
	assert_eq!(active_era(), era_index);
	// One way or another, current_era must have changed before the active era, so they must match
	// at this point.
	assert_eq!(current_era(), active_era());
}

pub(crate) fn current_total_payout_for_duration(duration: u64) -> Balance {
	let (payout, _rest) = <Runtime as Config>::EraPayout::era_payout(
		Staking::eras_total_stake(active_era()),
		Balances::total_issuance(),
		duration,
	);
	assert!(payout > 0);
	payout
}

pub(crate) fn maximum_payout_for_duration(duration: u64) -> Balance {
	let (payout, rest) = <Runtime as Config>::EraPayout::era_payout(
		Staking::eras_total_stake(active_era()),
		Balances::total_issuance(),
		duration,
	);
	payout + rest
}

pub(crate) fn active_era() -> EraIndex {
	Staking::active_era().unwrap().index
}

/// Time it takes to finish a session.
///
/// Note, if you see `time_per_session() - BLOCK_TIME`, it is fine. This is because we set the
/// timestamp after on_initialize, so the timestamp is always one block old.
pub(crate) fn time_per_session() -> u64 {
	Period::get() * BLOCK_TIME
}

/// Time it takes to finish an era.
///
/// Note, if you see `time_per_era() - BLOCK_TIME`, it is fine. This is because we set the
/// timestamp after on_initialize, so the timestamp is always one block old.
pub(crate) fn time_per_era() -> u64 {
	time_per_session() * SessionsPerEra::get() as u64
}

/// Time that will be calculated for the reward per era.
pub(crate) fn reward_time_per_era() -> u64 {
	time_per_era() - BLOCK_TIME
}

pub(crate) fn events() -> Vec<RuntimeEvent> {
	System::events().into_iter().map(|r| r.event).collect::<Vec<_>>()
}

pub(crate) fn current_era() -> EraIndex {
	Staking::current_era().unwrap()
}

pub(crate) fn staking_events() -> Vec<pallet_staking::Event<Runtime>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let RuntimeEvent::Staking(inner) = e { Some(inner) } else { None })
		.collect()
}
