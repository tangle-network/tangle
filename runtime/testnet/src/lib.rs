// Copyright 2022 Webb Technologies Inc.
//
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
//
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod frontier_evm;
pub mod impls;
pub mod precompiles;
pub mod voter_bags;

use frame_election_provider_support::{
	onchain, BalancingConfig, ElectionDataProvider, SequentialPhragmen, VoteWeight,
};
use frame_support::{
	traits::{Contains, OnFinalize, WithdrawReasons},
	weights::ConstantMultiplier,
};
use pallet_election_provider_multi_phase::SolutionAccuracyOf;
use pallet_grandpa::{
	fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as pallet_session_historical;
pub use pallet_staking::StakerStatus;
use pallet_transaction_payment::{
	CurrencyAdapter, FeeDetails, Multiplier, RuntimeDispatchInfo, TargetedFeeAdjustment,
};
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata, H160, H256, U256};
use sp_runtime::{
	create_runtime_str,
	curve::PiecewiseLinear,
	generic, impl_opaque_keys,
	traits::{
		self, BlakeTwo256, Block as BlockT, Bounded, Convert, ConvertInto, DispatchInfoOf,
		Dispatchable, NumberFor, OpaqueKeys, PostDispatchInfoOf, StaticLookup, UniqueSaturatedInto,
	},
	transaction_validity::{
		TransactionPriority, TransactionSource, TransactionValidity, TransactionValidityError,
	},
	ApplyExtrinsicResult, DispatchResult, FixedPointNumber, FixedU128, Perquintill,
	SaturatedConversion,
};
use sp_staking::{
	currency_to_vote::U128CurrencyToVote,
	offence::{OffenceError, ReportOffence},
	SessionIndex,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use static_assertions::const_assert;
use tangle_primitives::{
	jobs::{JobResult, JobSubmission, JobType, JobWithResult, ValidatorOffenceType},
	roles::ValidatorRewardDistribution,
	traits::jobs::{JobToFee, MPCHandler},
};

#[cfg(any(feature = "std", test))]
pub use frame_system::Call as SystemCall;

// A few exports that help ease life for downstream crates.
pub use dkg_runtime_primitives::crypto::AuthorityId as DKGId;
use dkg_runtime_primitives::{
	MaxAuthorities, MaxKeyLength, MaxProposalLength, MaxReporters, MaxSignatureLength, TypedChainId,
};
pub use frame_support::{
	construct_runtime,
	dispatch::DispatchClass,
	pallet_prelude::Get,
	parameter_types,
	traits::{
		ConstU128, ConstU16, ConstU32, Currency, EitherOfDiverse, EqualPrivilegeOnly, Everything,
		Imbalance, InstanceFilter, KeyOwnerProofSystem, LockIdentifier, OnUnbalanced,
	},
	weights::{
		constants::{
			BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_MILLIS,
		},
		IdentityFee, Weight,
	},
	PalletId, StorageValue,
};
use frame_system::EnsureRoot;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
use sp_runtime::generic::Era;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{MultiAddress, Perbill, Percent, Permill};
use tangle_primitives::AuraId;
pub use tangle_primitives::{
	currency::*, fee::*, time::*, AccountId, AccountIndex, Address, Balance, BlockNumber, Hash,
	Header, Index, Moment, Reputation, Signature, AVERAGE_ON_INITIALIZE_RATIO,
	EPOCH_DURATION_IN_BLOCKS, MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO, SESSION_PERIOD_BLOCKS,
	UNSIGNED_PROPOSAL_EXPIRY,
};

// Frontier
use fp_rpc::TransactionStatus;
use pallet_ethereum::{Call::transact, Transaction as EthereumTransaction};
use pallet_evm::{Account as EVMAccount, FeeCalculator, Runner};

pub type Nonce = u32;

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("tangle-standalone"),
	impl_name: create_runtime_str!("tangle-standalone"),
	authoring_version: 1,
	spec_version: 501, // v0.5.1
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 0,
};

pub const fn deposit(items: u32, bytes: u32) -> Balance {
	// map to 1/10 of what the kusama relay chain charges (v9020)
	(items as Balance * 2_000 * CENT + (bytes as Balance) * 100 * MILLIUNIT) / 10
}

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

pub const MAXIMUM_BLOCK_LENGTH: u32 = 5 * 1024 * 1024;

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const BlockHashCount: BlockNumber = 256;
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
		::with_sensible_defaults(MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO);
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(MAXIMUM_BLOCK_LENGTH, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
			pub im_online: ImOnline,
			pub dkg: DKG,
		}
	}
}

impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = BaseFilter;
	type BlockHashCount = BlockHashCount;
	type BlockLength = BlockLength;
	type Block = Block;
	type BlockWeights = BlockWeights;
	type RuntimeCall = RuntimeCall;
	type DbWeight = RocksDbWeight;
	type RuntimeEvent = RuntimeEvent;
	type Hash = Hash;
	type Nonce = Nonce;
	type Hashing = BlakeTwo256;
	type Lookup = Indices;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type RuntimeOrigin = RuntimeOrigin;
	type PalletInfo = PalletInfo;
	type SS58Prefix = SS58Prefix;
	type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
	type Version = Version;
}

parameter_types! {
	pub const IndexDeposit: Balance = UNIT;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = AccountIndex;
	type Currency = Balances;
	type Deposit = IndexDeposit;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_indices::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = EXISTENTIAL_DEPOSIT;
	pub const TransferFee: u128 = MILLIUNIT;
	pub const CreationFee: u128 = MILLIUNIT;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

pub type NegativeImbalance<T> = <pallet_balances::Pallet<T> as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

impl pallet_balances::Config for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxHolds = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
}

parameter_types! {
	pub const TransactionByteFee: Balance = MILLIUNIT;
	pub const OperationalFeeMultiplier: u8 = 5;
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
	pub MaximumMultiplier: Multiplier = Bounded::max_value();
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = CurrencyAdapter<Balances, impls::DealWithFees<Runtime>>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = TargetedFeeAdjustment<
		Self,
		TargetBlockFullness,
		AdjustmentVariable,
		MinimumMultiplier,
		MaximumMultiplier,
	>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		BlockWeights::get().max_block;
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = ConstU32<512>;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = Preimage;
}

parameter_types! {
	pub const PreimageMaxSize: u32 = 4096 * 1024;
	pub const PreimageBaseDeposit: Balance = UNIT;
	// One cent: $10,000 / MB
	pub const PreimageByteDeposit: Balance = 10 * MILLIUNIT;
}

impl pallet_preimage::Config for Runtime {
	type BaseDeposit = PreimageBaseDeposit;
	type ByteDeposit = PreimageByteDeposit;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl pallet_sudo::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = MaxAuthorities;
	type AllowMultipleBlocksPerSlot = frame_support::traits::ConstBool<false>;
}

parameter_types! {
	pub const ReportLongevity: u64 =
		BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * Period::get();
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxSetIdSessionEntries = frame_support::traits::ConstU64<0>;
	type MaxAuthorities = MaxAuthorities;
	type EquivocationReportSystem = ();
	type KeyOwnerProof = sp_core::Void;
	type WeightInfo = ();
}

parameter_types! {
	pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
	type EventHandler = Staking;
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
}

parameter_types! {
	pub const Period: BlockNumber = SESSION_PERIOD_BLOCKS;
	pub const Offset: BlockNumber = 0;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_staking::StashOf<Self>;
	type ShouldEndSession = pallet_dkg_metadata::DKGPeriodicSessions<Period, Offset, Runtime>;
	type NextSessionRotation = pallet_dkg_metadata::DKGPeriodicSessions<Period, Offset, Runtime>;
	type SessionManager = Staking;
	type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = opaque::SessionKeys;
	type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

impl pallet_session::historical::Config for Runtime {
	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

pallet_staking_reward_curve::build! {
	const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
		min_inflation: 0_040_000,
		max_inflation: 0_050_000,
		// 60% of total issuance at a yearly inflation rate of 5%
		ideal_stake: 0_600_000,
		falloff: 0_050_000,
		max_piece_count: 40,
		test_precision: 0_005_000,
	);
}

parameter_types! {
	pub const SessionsPerEra: sp_staking::SessionIndex = 1;
	pub const BondingDuration: sp_staking::EraIndex = 24 * 28;
	pub const SlashDeferDuration: sp_staking::EraIndex = 24 * 7; // 1/4 the bonding duration.
	pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
	pub const MaxNominatorRewardedPerValidator: u32 = 256;
	pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
	pub OffchainRepeat: BlockNumber = 5;
	pub const HistoryDepth: u32 = 80;
}

pub struct StakingBenchmarkingConfig;
impl pallet_staking::BenchmarkingConfig for StakingBenchmarkingConfig {
	type MaxNominators = ConstU32<1000>;
	type MaxValidators = ConstU32<1000>;
}

impl pallet_staking::Config for Runtime {
	type MaxNominations = MaxNominations;
	type Currency = Balances;
	type CurrencyBalance = Balance;
	type AdminOrigin = EnsureRoot<AccountId>;
	type UnixTime = Timestamp;
	type CurrencyToVote = U128CurrencyToVote;
	type RewardRemainder = Treasury;
	type RuntimeEvent = RuntimeEvent;
	type Slash = Treasury; // send the slashed funds to the treasury.
	type Reward = (); // rewards are minted from the void
	type SessionsPerEra = SessionsPerEra;
	type BondingDuration = BondingDuration;
	type SlashDeferDuration = SlashDeferDuration;
	type SessionInterface = Self;
	type TargetList = pallet_staking::UseValidatorsMap<Runtime>;
	type EraPayout = pallet_staking::ConvertCurve<RewardCurve>;
	type NextNewSession = Session;
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
	type ElectionProvider = ElectionProviderMultiPhase;
	type GenesisElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type VoterList = BagsList;
	type MaxUnlockingChunks = ConstU32<32>;
	type HistoryDepth = HistoryDepth;
	type EventListeners = NominationPools;
	type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
	type BenchmarkingConfig = StakingBenchmarkingConfig;
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	pub const VotingPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * 24 * 60 * MINUTES;
	pub const MinimumDeposit: Balance = 100 * UNIT;
	pub const EnactmentPeriod: BlockNumber = 30 * 24 * 60 * MINUTES;
	pub const CooloffPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	pub const MaxProposals: u32 = 100;
}

type EnsureRootOrHalfCouncil = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
>;

impl pallet_democracy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type SubmitOrigin = frame_system::EnsureSigned<AccountId>;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type VoteLockingPeriod = EnactmentPeriod; // Same as EnactmentPeriod
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
	/// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
	type InstantOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
	type InstantAllowed = frame_support::traits::ConstBool<true>;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cool-off period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
	type CooloffPeriod = CooloffPeriod;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = ConstU32<100>;
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
	type MaxProposals = MaxProposals;
	type Preimages = Preimage;
	type MaxDeposits = ConstU32<100>;
	type MaxBlacklisted = ConstU32<100>;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
	pub MaxProposalWeight: Weight = Perbill::from_percent(50) * BlockWeights::get().max_block;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type SetMembersOrigin = EnsureRoot<AccountId>;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type MaxProposalWeight = MaxProposalWeight;
}

parameter_types! {
	// phase durations. 1/4 of the last session for each.
	pub const SignedPhase: u64 = EPOCH_DURATION_IN_BLOCKS / 4;
	pub const UnsignedPhase: u64 = EPOCH_DURATION_IN_BLOCKS / 4;

	// signed config
	pub const SignedRewardBase: Balance = UNIT;
	pub const SignedDepositBase: Balance = UNIT;
	pub const SignedDepositByte: Balance = CENT;

	pub BetterUnsignedThreshold: Perbill = Perbill::from_rational(1u32, 10_000);

	// miner configs
	pub const MultiPhaseUnsignedPriority: TransactionPriority = StakingUnsignedPriority::get() - 1u64;
	pub MinerMaxWeight: Weight = BlockWeights::get()
		.get(DispatchClass::Normal)
		.max_extrinsic.expect("Normal extrinsics have a weight limit configured; qed")
		.saturating_sub(BlockExecutionWeight::get());
	// Solution can occupy 90% of normal block size
	pub MinerMaxLength: u32 = Perbill::from_rational(9u32, 10) *
		*BlockLength::get()
		.max
		.get(DispatchClass::Normal);
}

frame_election_provider_support::generate_solution_type!(
	#[compact]
	pub struct NposSolution16::<
		VoterIndex = u32,
		TargetIndex = u16,
		Accuracy = sp_runtime::PerU16,
		MaxVoters = MaxElectingVoters,
	>(16)
);

parameter_types! {
	pub MaxNominations: u32 = <NposSolution16 as frame_election_provider_support::NposSolution>::LIMIT as u32;
	pub MaxElectingVoters: u32 = 40_000;
	pub MaxElectableTargets: u16 = 10_000;
	// OnChain values are lower.
	pub MaxOnChainElectingVoters: u32 = 5000;
	pub MaxOnChainElectableTargets: u16 = 1250;
	// The maximum winners that can be elected by the Election pallet which is equivalent to the
	// maximum active validators the staking pallet can have.
	pub MaxActiveValidators: u32 = 1000;
}

/// The numbers configured here could always be more than the the maximum limits of staking pallet
/// to ensure election snapshot will not run out of memory. For now, we set them to smaller values
/// since the staking is bounded and the weight pipeline takes hours for this single pallet.
pub struct ElectionProviderBenchmarkConfig;
impl pallet_election_provider_multi_phase::BenchmarkingConfig for ElectionProviderBenchmarkConfig {
	const VOTERS: [u32; 2] = [1000, 2000];
	const TARGETS: [u32; 2] = [500, 1000];
	const ACTIVE_VOTERS: [u32; 2] = [500, 800];
	const DESIRED_TARGETS: [u32; 2] = [200, 400];
	const SNAPSHOT_MAXIMUM_VOTERS: u32 = 1000;
	const MINER_MAXIMUM_VOTERS: u32 = 1000;
	const MAXIMUM_TARGETS: u32 = 300;
}

/// Maximum number of iterations for balancing that will be executed in the embedded OCW
/// miner of election provider multi phase.
pub const MINER_MAX_ITERATIONS: u32 = 10;

/// A source of random balance for NposSolver, which is meant to be run by the OCW election miner.
pub struct OffchainRandomBalancing;
impl Get<Option<BalancingConfig>> for OffchainRandomBalancing {
	fn get() -> Option<BalancingConfig> {
		use sp_runtime::traits::TrailingZeroInput;
		let iterations = match MINER_MAX_ITERATIONS {
			0 => 0,
			max => {
				let seed = sp_io::offchain::random_seed();
				let random = <u32>::decode(&mut TrailingZeroInput::new(&seed))
					.expect("input is padded with zeroes; qed") %
					max.saturating_add(1);
				random as usize
			},
		};

		let config = BalancingConfig { iterations, tolerance: 0 };
		Some(config)
	}
}

pub struct OnChainSeqPhragmen;
impl onchain::Config for OnChainSeqPhragmen {
	type System = Runtime;
	type Solver = SequentialPhragmen<
		AccountId,
		pallet_election_provider_multi_phase::SolutionAccuracyOf<Runtime>,
	>;
	type DataProvider = <Runtime as pallet_election_provider_multi_phase::Config>::DataProvider;
	type WeightInfo = frame_election_provider_support::weights::SubstrateWeight<Runtime>;
	type MaxWinners = <Runtime as pallet_election_provider_multi_phase::Config>::MaxWinners;
	type VotersBound = MaxOnChainElectingVoters;
	type TargetsBound = MaxOnChainElectableTargets;
}

impl pallet_election_provider_multi_phase::MinerConfig for Runtime {
	type AccountId = AccountId;
	type MaxLength = MinerMaxLength;
	type MaxWeight = MinerMaxWeight;
	type Solution = NposSolution16;
	type MaxWinners = MaxActiveValidators;
	type MaxVotesPerVoter =
	<<Self as pallet_election_provider_multi_phase::Config>::DataProvider as ElectionDataProvider>::MaxVotesPerVoter;

	// The unsigned submissions have to respect the weight of the submit_unsigned call, thus their
	// weight estimate function is wired to this call's weight.
	fn solution_weight(v: u32, t: u32, a: u32, d: u32) -> Weight {
		<
			<Self as pallet_election_provider_multi_phase::Config>::WeightInfo
			as
			pallet_election_provider_multi_phase::WeightInfo
		>::submit_unsigned(v, t, a, d)
	}
}

impl pallet_election_provider_multi_phase::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EstimateCallFee = TransactionPayment;
	type SignedPhase = SignedPhase;
	type UnsignedPhase = UnsignedPhase;
	type BetterUnsignedThreshold = BetterUnsignedThreshold;
	type BetterSignedThreshold = ();
	type OffchainRepeat = OffchainRepeat;
	type MinerTxPriority = MultiPhaseUnsignedPriority;
	type MinerConfig = Self;
	type SignedMaxSubmissions = ConstU32<10>;
	type SignedRewardBase = SignedRewardBase;
	type SignedDepositBase = SignedDepositBase;
	type SignedDepositByte = SignedDepositByte;
	type SignedMaxRefunds = ConstU32<3>;
	type SignedDepositWeight = ();
	type SignedMaxWeight = MinerMaxWeight;
	type SlashHandler = (); // burn slashes
	type RewardHandler = (); // nothing to do upon rewards
	type DataProvider = Staking;
	type Fallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type GovernanceFallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type Solver = SequentialPhragmen<AccountId, SolutionAccuracyOf<Self>, OffchainRandomBalancing>;
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type MaxElectableTargets = MaxElectableTargets;
	type MaxWinners = MaxActiveValidators;
	type MaxElectingVoters = MaxElectingVoters;
	type BenchmarkingConfig = ElectionProviderBenchmarkConfig;
	type WeightInfo = pallet_election_provider_multi_phase::weights::SubstrateWeight<Self>;
}

parameter_types! {
	pub const BagThresholds: &'static [u64] = &voter_bags::THRESHOLDS;
}

impl pallet_bags_list::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ScoreProvider = Staking;
	type WeightInfo = pallet_bags_list::weights::SubstrateWeight<Runtime>;
	type BagThresholds = BagThresholds;
	type Score = VoteWeight;
}

parameter_types! {
  pub const PostUnbondPoolsWindow: u32 = 4;
  pub const NominationPoolsPalletId: PalletId = PalletId(*b"py/nopls");
  pub const MaxPointsToBalance: u8 = 10;
}

pub struct BalanceToU256;
impl Convert<Balance, sp_core::U256> for BalanceToU256 {
	fn convert(balance: Balance) -> sp_core::U256 {
		sp_core::U256::from(balance)
	}
}
pub struct U256ToBalance;
impl Convert<sp_core::U256, Balance> for U256ToBalance {
	fn convert(n: sp_core::U256) -> Balance {
		n.try_into().unwrap_or(Balance::max_value())
	}
}

impl pallet_nomination_pools::Config for Runtime {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type RewardCounter = FixedU128;
	type BalanceToU256 = BalanceToU256;
	type U256ToBalance = U256ToBalance;
	type Staking = Staking;
	type PostUnbondingPoolsWindow = PostUnbondPoolsWindow;
	type MaxMetadataLen = ConstU32<256>;
	type MaxUnbonding = ConstU32<8>;
	type PalletId = NominationPoolsPalletId;
	type MaxPointsToBalance = MaxPointsToBalance;
}

parameter_types! {
	pub const DecayPercentage: Percent = Percent::from_percent(50);
	pub const UnsignedPriority: u64 = 1 << 20;
	 pub const UnsignedInterval: BlockNumber = 1;
	 pub const SessionPeriod : BlockNumber = SESSION_PERIOD_BLOCKS;
	 #[derive(Default, Clone, Encode, Decode, Debug, Eq, PartialEq, scale_info::TypeInfo, Ord, PartialOrd, parity_scale_codec::MaxEncodedLen)]
	 pub const VoteLength: u32 = 64;
}

impl pallet_dkg_metadata::Config for Runtime {
	type DKGId = DKGId;
	type RuntimeEvent = RuntimeEvent;
	type OnAuthoritySetChangeHandler = DKGProposals;
	type OnDKGPublicKeyChangeHandler = ();
	type OffChainAuthId = dkg_runtime_primitives::offchain::crypto::OffchainAuthId;
	type NextSessionRotation = pallet_dkg_metadata::DKGPeriodicSessions<Period, Offset, Runtime>;
	type KeygenJailSentence = Period;
	type SigningJailSentence = Period;
	type DecayPercentage = DecayPercentage;
	type Reputation = Reputation;
	type ForceOrigin = EnsureRoot<AccountId>;
	type UnsignedPriority = UnsignedPriority;
	type SessionPeriod = SessionPeriod;
	type UnsignedInterval = UnsignedInterval;
	type AuthorityIdOf = pallet_dkg_metadata::AuthorityIdOf<Self>;
	type ProposalHandler = DKGProposalHandler;
	type MaxKeyLength = MaxKeyLength;
	type MaxSignatureLength = MaxSignatureLength;
	type DKGAuthorityToMerkleLeaf = pallet_dkg_proposals::DKGEcdsaToEthereumAddress;
	type MaxReporters = MaxReporters;
	type MaxAuthorities = MaxAuthorities;
	type VoteLength = VoteLength;
	type MaxProposalLength = MaxProposalLength;
	type WeightInfo = pallet_dkg_metadata::weights::WebbWeight<Runtime>;
}

parameter_types! {
	pub const ChainIdentifier: TypedChainId = TypedChainId::Substrate(1081);
	pub const ProposalLifetime: BlockNumber = HOURS / 5;
	pub const DKGAccountId: PalletId = PalletId(*b"dw/dkgac");
	pub const RefreshDelay: Permill = Permill::from_percent(90);
	pub const TimeToRestart: BlockNumber = 3;
	pub const UnsignedProposalExpiry: BlockNumber = UNSIGNED_PROPOSAL_EXPIRY;
}

impl pallet_dkg_proposal_handler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ForceOrigin = EnsureRoot<AccountId>;
	type OffChainAuthId = dkg_runtime_primitives::offchain::crypto::OffchainAuthId;
	type UnsignedProposalExpiry = UnsignedProposalExpiry;
	type SignedProposalHandler = (BridgeRegistry, DKG);
	type MaxProposalsPerBatch = dkg_runtime_primitives::CustomU32Getter<10>;
	type BatchId = u32;
	type ValidatorSet = Historical;
	type ReportOffences = Offences;
	type WeightInfo = pallet_dkg_proposal_handler::weights::WebbWeight<Runtime>;
}

parameter_types! {
	#[derive(Clone, Encode, Decode, Debug, Eq, PartialEq, scale_info::TypeInfo, Ord, PartialOrd)]
	pub const MaxVotes : u32 = 100;
	#[derive(Clone, Encode, Decode, Debug, Eq, PartialEq, scale_info::TypeInfo, Ord, PartialOrd, Serialize, Deserialize)]
	pub const MaxResources : u32 = 1000;
	#[derive(Clone, Encode, Decode, Debug, Eq, PartialEq, scale_info::TypeInfo, Ord, PartialOrd)]
	pub const MaxProposers : u32 = 1000;
}

impl pallet_dkg_proposals::Config for Runtime {
	type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type DKGAuthorityToMerkleLeaf = pallet_dkg_proposals::DKGEcdsaToEthereumAddress;
	type DKGId = DKGId;
	type ChainIdentifier = ChainIdentifier;
	type RuntimeEvent = RuntimeEvent;
	type NextSessionRotation = pallet_dkg_metadata::DKGPeriodicSessions<Period, Offset, Runtime>;
	type MaxProposalLength = MaxProposalLength;
	type ProposalLifetime = ProposalLifetime;
	type ProposalHandler = DKGProposalHandler;
	type Period = Period;
	type MaxVotes = MaxVotes;
	type MaxResources = MaxResources;
	type MaxProposers = MaxProposers;
	type VotingKeySize = MaxKeyLength;
	type WeightInfo = pallet_dkg_proposals::WebbWeight<Runtime>;
}

impl pallet_bridge_registry::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BridgeIndex = u32;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxResources = MaxResources;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxProposalLength = MaxProposalLength;
	type WeightInfo = ();
}

parameter_types! {
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
	/// We prioritize im-online heartbeats over election solution submission.
	pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		public: <Signature as traits::Verify>::Signer,
		account: AccountId,
		nonce: Nonce,
	) -> Option<(RuntimeCall, <UncheckedExtrinsic as traits::Extrinsic>::SignaturePayload)> {
		let tip = 0;
		// take the biggest period possible.
		let period = BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2);
		let current_block = System::block_number()
			.saturated_into::<u64>()
			// The `System::block_number` is initialized with `n+1`,
			// so the actual block number is `n`.
			.saturating_sub(1);
		let era = Era::mortal(period, current_block);
		let extra = (
			frame_system::CheckNonZeroSender::<Runtime>::new(),
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(era),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				log::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let address = Indices::unlookup(account);
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (address, signature, extra)))
	}
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as traits::Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	RuntimeCall: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = RuntimeCall;
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 100 * UNIT;
	pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
		WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
}

impl pallet_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting::weights::SubstrateWeight<Runtime>;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	// `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
	// highest number of schedules that encodes less than 2^10.
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

impl pallet_offences::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
	type OnOffenceHandler = Staking;
}

parameter_types! {
	pub const CandidacyBond: Balance = 10 * UNIT;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub const VotingBondBase: Balance = deposit(1, 64);
	// additional data per vote is 32 bytes (account id).
	pub const VotingBondFactor: Balance = deposit(0, 32);
	pub const TermDuration: BlockNumber = 7 * DAYS;
	pub const DesiredMembers: u32 = 13;
	pub const DesiredRunnersUp: u32 = 7;
	pub const MaxCandidates: u32 = 10;
	pub const MaxVoters: u32 = 5;
	pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
}

// Make sure that there are no more than `MaxMembers` members elected via
// elections-phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl pallet_elections_phragmen::Config for Runtime {
	type CandidacyBond = CandidacyBond;
	type ChangeMembers = Council;
	type Currency = Balances;
	type CurrencyToVote = U128CurrencyToVote;
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type RuntimeEvent = RuntimeEvent;
	type MaxVotesPerVoter = ConstU32<100>;
	// NOTE: this implies that council's genesis members cannot be set directly and
	// must come from this module.
	type InitializeMembers = Council;
	type KickedMember = ();
	type LoserCandidate = ();
	type PalletId = ElectionsPhragmenPalletId;
	type TermDuration = TermDuration;
	type MaxCandidates = MaxCandidates;
	type MaxVoters = MaxVoters;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type WeightInfo = pallet_elections_phragmen::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = UNIT;
	pub const SpendPeriod: BlockNumber = DAYS;
	pub const Burn: Permill = Permill::from_percent(50);
	pub const TipCountdown: BlockNumber = DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = UNIT;
	pub const DataDepositPerByte: Balance = CENT;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const MaximumReasonLength: u32 = 300;
	pub const MaxApprovals: u32 = 100;
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 5>,
	>;
	type RejectOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
	>;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type ProposalBondMaximum = ();
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<u128>;
	type SpendFunds = Bounties;
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type MaxApprovals = MaxApprovals;
}

parameter_types! {
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 5 * UNIT;
	pub const BountyDepositBase: Balance = UNIT;
	pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
	pub const CuratorDepositMin: Balance = UNIT;
	pub const CuratorDepositMax: Balance = 100 * UNIT;
	pub const BountyDepositPayoutDelay: BlockNumber = DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
}

impl pallet_bounties::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type CuratorDepositMultiplier = CuratorDepositMultiplier;
	type CuratorDepositMin = CuratorDepositMin;
	type CuratorDepositMax = CuratorDepositMax;
	type BountyValueMinimum = BountyValueMinimum;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = pallet_bounties::weights::SubstrateWeight<Runtime>;
	type ChildBountyManager = ChildBounties;
}

parameter_types! {
	pub const ChildBountyValueMinimum: Balance = UNIT;
}

impl pallet_child_bounties::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxActiveChildBountyCount = ConstU32<5>;
	type ChildBountyValueMinimum = ChildBountyValueMinimum;
	type WeightInfo = pallet_child_bounties::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const MaxKeys: u32 = 10_000;
	pub const MaxPeerInHeartbeats: u32 = 10_000;
}

impl pallet_im_online::Config for Runtime {
	type AuthorityId = ImOnlineId;
	type RuntimeEvent = RuntimeEvent;
	type NextSessionRotation = pallet_dkg_metadata::DKGPeriodicSessions<Period, Offset, Runtime>;
	type ValidatorSet = Historical;
	type ReportUnresponsiveness = ();
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
	type MaxKeys = MaxKeys;
	type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
}

impl pallet_transaction_pause::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type UpdateOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
}

parameter_types! {
	pub const BasicDeposit: Balance = deposit(1, 258);
	pub const FieldDeposit: Balance = deposit(0, 66);
	pub const SubAccountDeposit: Balance = deposit(1, 53);
	pub const MaxSubAccounts: u32 = 100;
	#[derive(Serialize, Deserialize)]
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = ();
	type ForceOrigin = EnsureRoot<Self::AccountId>;
	type RegistrarOrigin = EnsureRoot<Self::AccountId>;
	type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

// parameter_types! {
// 	pub const StoragePricePerByte: u128 = MILLIUNIT;
// 	pub const Eth2ClientPalletId: PalletId = PalletId(*b"py/eth2c");
// }

// impl pallet_eth2_light_client::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;
// 	type StoragePricePerByte = StoragePricePerByte;
// 	type PalletId = Eth2ClientPalletId;
// 	type Currency = Balances;
// }

parameter_types! {
	pub const JobsPalletId: PalletId = PalletId(*b"py/jobss");
}

pub struct MockJobToFeeHandler;

impl JobToFee<AccountId, BlockNumber> for MockJobToFeeHandler {
	type Balance = Balance;

	fn job_to_fee(job: &JobSubmission<AccountId, BlockNumber>) -> Balance {
		match job.job_type {
			JobType::DKGPhaseOne(_) => Dkg::job_to_fee(job),
			JobType::DKGPhaseTwo(_) => Dkg::job_to_fee(job),
			JobType::ZkSaaSPhaseOne(_) => todo!(), // TODO : Replace with zksaas pallet
			JobType::ZkSaaSPhaseTwo(_) => todo!(), // TODO : Replace with zksaas pallet
		}
	}
}

pub struct MockMPCHandler;

impl MPCHandler<AccountId, BlockNumber, Balance> for MockMPCHandler {
	fn verify(data: JobWithResult<AccountId>) -> DispatchResult {
		match data.result {
			JobResult::DKGPhaseOne(_) => Dkg::verify(data.result),
			JobResult::DKGPhaseTwo(_) => Dkg::verify(data.result),
			JobResult::ZkSaaSPhaseOne(_) => todo!(), // TODO : Replace with zksaas pallet
			JobResult::ZkSaaSPhaseTwo(_) => todo!(), // TODO : Replace with zksaas pallet
		}
	}

	fn verify_validator_report(
		_validator: AccountId,
		_offence: ValidatorOffenceType,
		_signatures: Vec<Vec<u8>>,
	) -> DispatchResult {
		Ok(())
	}

	fn validate_authority_key(_validator: AccountId, _authority_key: Vec<u8>) -> DispatchResult {
		Ok(())
	}
}

impl pallet_jobs::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type Currency = Balances;
	type JobToFee = MockJobToFeeHandler;
	type RolesHandler = Roles;
	type MPCHandler = MockMPCHandler;
	type PalletId = JobsPalletId;
	type WeightInfo = ();
}

type IdTuple = pallet_session::historical::IdentificationTuple<Runtime>;
type Offence = pallet_roles::offences::ValidatorOffence<IdTuple>;
/// A mock offence report handler.
pub struct OffenceHandler;
impl ReportOffence<AccountId, IdTuple, Offence> for OffenceHandler {
	fn report_offence(_reporters: Vec<AccountId>, _offence: Offence) -> Result<(), OffenceError> {
		Ok(())
	}

	fn is_known_offence(_offenders: &[IdTuple], _time_slot: &SessionIndex) -> bool {
		false
	}
}

parameter_types! {
	pub InflationRewardPerSession: Balance = 10_000;
	pub Reward : ValidatorRewardDistribution = ValidatorRewardDistribution::try_new(Percent::from_rational(1_u32,2_u32), Percent::from_rational(1_u32,2_u32)).unwrap();
}

impl pallet_roles::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type JobsHandler = Jobs;
	type MaxRolesPerAccount = ConstU32<2>;
	type MPCHandler = MockMPCHandler;
	type InflationRewardPerSession = InflationRewardPerSession;
	type AuthorityId = AuraId;
	type ValidatorSet = Historical;
	type ReportOffences = OffenceHandler;
	type ValidatorRewardDistribution = Reward;
	type WeightInfo = ();
}

impl pallet_dkg::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UpdateOrigin = EnsureRootOrHalfCouncil;
	type WeightInfo = ();
}

pub struct BaseFilter;
impl Contains<RuntimeCall> for BaseFilter {
	fn contains(call: &RuntimeCall) -> bool {
		let is_core_call = matches!(call, RuntimeCall::System(_) | RuntimeCall::Timestamp(_));
		if is_core_call {
			// always allow core call
			return true
		}

		let is_paused =
			pallet_transaction_pause::PausedTransactionFilter::<Runtime>::contains(call);
		if is_paused {
			// no paused call
			return false
		}
		// Following staking pallet calls will be blocked and will be allowed to execute
		// through role pallet.
		let is_stake_unbound_call =
			matches!(call, RuntimeCall::Staking(pallet_staking::Call::unbond { .. }));

		if is_stake_unbound_call {
			// no unbond call
			return false
		}

		// no chill call
		if matches!(call, RuntimeCall::Staking(pallet_staking::Call::chill { .. })) {
			return false
		}

		// no withdraw_unbonded call
		let is_stake_withdraw_call =
			matches!(call, RuntimeCall::Staking(pallet_staking::Call::withdraw_unbonded { .. }));

		if is_stake_withdraw_call {
			return false
		}

		let democracy_related = matches!(
			call,
			// Filter democracy proposals creation
			RuntimeCall::Democracy(_) |
			// disallow council
			RuntimeCall::Council(_)
		);

		if democracy_related {
			// no democracy call
			return false
		}

		true
	}
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Timestamp: pallet_timestamp,

		Sudo: pallet_sudo,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,

		Balances: pallet_balances,
		TransactionPayment: pallet_transaction_payment,

		Authorship: pallet_authorship,
		Aura: pallet_aura,
		Grandpa: pallet_grandpa,

		// DKG / offchain worker
		DKG: pallet_dkg_metadata,
		DKGProposals: pallet_dkg_proposals,
		DKGProposalHandler: pallet_dkg_proposal_handler,
		BridgeRegistry: pallet_bridge_registry,

		Indices: pallet_indices,
		Democracy: pallet_democracy,
		Council: pallet_collective::<Instance1>,
		Vesting: pallet_vesting,

		Elections: pallet_elections_phragmen,
		ElectionProviderMultiPhase: pallet_election_provider_multi_phase,
		Staking: pallet_staking,
		Session: pallet_session,
		Historical: pallet_session_historical,
		Treasury: pallet_treasury,
		Bounties: pallet_bounties,
		ChildBounties: pallet_child_bounties,
		BagsList: pallet_bags_list,
		NominationPools: pallet_nomination_pools,

		Scheduler: pallet_scheduler,
		Preimage: pallet_preimage,
		Offences: pallet_offences,

		TransactionPause: pallet_transaction_pause,
		ImOnline: pallet_im_online,
		Identity: pallet_identity,
		Utility: pallet_utility,

		Ethereum: pallet_ethereum,
		EVM: pallet_evm,
		EVMChainId: pallet_evm_chain_id,
		DynamicFee: pallet_dynamic_fee,
		BaseFee: pallet_base_fee,
		HotfixSufficients: pallet_hotfix_sufficients,
		//Eth2Client: pallet_eth2_light_client,

		Roles: pallet_roles,
		Jobs: pallet_jobs,
		Dkg: pallet_dkg
	}
);

#[derive(Clone, Serialize, Deserialize)]
pub struct TransactionConverter;

impl fp_rpc::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic {
		UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		)
	}
}

impl fp_rpc::ConvertTransaction<opaque::UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(
		&self,
		transaction: pallet_ethereum::Transaction,
	) -> opaque::UncheckedExtrinsic {
		let extrinsic = UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		);
		let encoded = extrinsic.encode();
		opaque::UncheckedExtrinsic::decode(&mut &encoded[..])
			.expect("Encoded extrinsic is always valid")
	}
}

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	fp_self_contained::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic =
	fp_self_contained::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra, H160>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	OnRuntimeUpgrade,
>;

pub struct OnRuntimeUpgrade;
impl frame_support::traits::OnRuntimeUpgrade for OnRuntimeUpgrade {
	fn on_runtime_upgrade() -> Weight {
		Weight::from_parts(0u64, 0u64)
	}
}

impl fp_self_contained::SelfContainedCall for RuntimeCall {
	type SignedInfo = H160;

	fn is_self_contained(&self) -> bool {
		match self {
			RuntimeCall::Ethereum(call) => call.is_self_contained(),
			_ => false,
		}
	}

	fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) => call.check_self_contained(),
			_ => None,
		}
	}

	fn validate_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<TransactionValidity> {
		match self {
			RuntimeCall::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
			_ => None,
		}
	}

	fn pre_dispatch_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<Result<(), TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) =>
				call.pre_dispatch_self_contained(info, dispatch_info, len),
			_ => None,
		}
	}

	fn apply_self_contained(
		self,
		info: Self::SignedInfo,
	) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
		match self {
			call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) =>
				Some(call.dispatch(RuntimeOrigin::from(
					pallet_ethereum::RawOrigin::EthereumTransaction(info),
				))),
			_ => None,
		}
	}
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}
		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(
			extrinsic: <Block as BlockT>::Extrinsic,
		) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl dkg_runtime_primitives::DKGApi<Block, dkg_runtime_primitives::crypto::AuthorityId, BlockNumber, MaxProposalLength, MaxAuthorities> for Runtime {
		fn authority_set() -> dkg_runtime_primitives::AuthoritySet<dkg_runtime_primitives::crypto::AuthorityId, MaxAuthorities> {
			let authorities = DKG::authorities();
			let authority_set_id = DKG::authority_set_id();

			dkg_runtime_primitives::AuthoritySet {
				authorities,
				id: authority_set_id
			}
		}

		fn queued_authority_set() -> dkg_runtime_primitives::AuthoritySet<dkg_runtime_primitives::crypto::AuthorityId, MaxAuthorities> {
			let queued_authorities = DKG::next_authorities();
			let queued_authority_set_id = DKG::authority_set_id() + 1u64;

			dkg_runtime_primitives::AuthoritySet {
				authorities: queued_authorities,
				id: queued_authority_set_id
			}
		}

		fn signature_threshold() -> u16 {
			DKG::signature_threshold()
		}

		fn keygen_threshold() -> u16 {
			DKG::keygen_threshold()
		}

		fn next_signature_threshold() -> u16 {
			DKG::next_signature_threshold()
		}

		fn next_keygen_threshold() -> u16 {
			DKG::next_keygen_threshold()
		}

		fn should_refresh(block_number: BlockNumber) -> bool {
			DKG::should_refresh(block_number)
		}

		fn next_dkg_pub_key() -> Option<(dkg_runtime_primitives::AuthoritySetId, Vec<u8>)> {
			DKG::next_dkg_public_key().map(|pub_key| (pub_key.0, pub_key.1.into()))
		  }

		  fn next_pub_key_sig() -> Option<Vec<u8>> {
			DKG::next_public_key_signature().map(|pub_key_sig| pub_key_sig.into())
		  }

		fn get_current_session_progress(block_number: BlockNumber) -> Option<Permill> {
			use frame_support::traits::EstimateNextSessionRotation;
			<pallet_session::PeriodicSessions<Period, Offset> as EstimateNextSessionRotation<BlockNumber>>::estimate_current_session_progress(block_number).0
		}

		fn dkg_pub_key() -> (dkg_runtime_primitives::AuthoritySetId, Vec<u8>) {
			(DKG::dkg_public_key().0, DKG::dkg_public_key().1.into())
		  }

		  fn get_best_authorities() -> Vec<(u16, DKGId)> {
			DKG::best_authorities().into()
		  }

		  fn get_next_best_authorities() -> Vec<(u16, DKGId)> {
			DKG::next_best_authorities().into()
		  }

		fn get_unsigned_proposal_batches() -> Vec<pallet_dkg_proposal_handler::StoredUnsignedProposalBatchOf<Runtime>> {
			DKGProposalHandler::get_unsigned_proposal_batches()
		}

		fn get_authority_accounts() -> (Vec<AccountId>, Vec<AccountId>) {
			(DKG::current_authorities_accounts().into(), DKG::next_authorities_accounts().into())
		}

		fn get_reputations(authorities: Vec<DKGId>) -> Vec<(DKGId, Reputation)> {
			authorities.iter().map(|a| (a.clone(), DKG::authority_reputations(a))).collect()
		}

		fn get_keygen_jailed(set: Vec<DKGId>) -> Vec<DKGId> {
			set.iter().filter(|a| pallet_dkg_metadata::JailedKeygenAuthorities::<Runtime>::contains_key(a)).cloned().collect()
		}

		fn get_signing_jailed(set: Vec<DKGId>) -> Vec<DKGId> {
			set.iter().filter(|a| pallet_dkg_metadata::JailedSigningAuthorities::<Runtime>::contains_key(a)).cloned().collect()
		}

		fn refresh_nonce() -> u32 {
			DKG::refresh_nonce()
		}

		fn should_execute_new_keygen() -> (bool, bool) {
			DKG::should_execute_new_keygen()
		}

		fn should_submit_proposer_vote() -> bool {
			DKG::should_submit_proposer_vote()
		}
	}

	impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
		fn chain_id() -> u64 {
			<Runtime as pallet_evm::Config>::ChainId::get()
		}

		fn account_basic(address: H160) -> EVMAccount {
			let (account, _) = pallet_evm::Pallet::<Runtime>::account_basic(&address);
			account
		}

		fn gas_price() -> U256 {
			let (gas_price, _) = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price();
			gas_price
		}

		fn account_code_at(address: H160) -> Vec<u8> {
			pallet_evm::AccountCodes::<Runtime>::get(address)
		}

		fn author() -> H160 {
			<pallet_evm::Pallet<Runtime>>::find_author()
		}

		fn storage_at(address: H160, index: U256) -> H256 {
			let mut tmp = [0u8; 32];
			index.to_big_endian(&mut tmp);
			pallet_evm::AccountStorages::<Runtime>::get(address, H256::from_slice(&tmp[..]))
		}

		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
			use pallet_evm::GasWeightMapping;
			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			let is_transactional = false;
			let validate = true;
			let mut estimated_transaction_len = data.len() +
				// to: 20
				// from: 20
				// value: 32
				// gas_limit: 32
				// nonce: 32
				// 1 byte transaction action variant
				// chain id 8 bytes
				// 65 bytes signature
				210;
			if max_fee_per_gas.is_some() {
				estimated_transaction_len += 32;
			}
			if max_priority_fee_per_gas.is_some() {
				estimated_transaction_len += 32;
			}
			if access_list.is_some() {
				estimated_transaction_len += access_list.encoded_size();
			}

			let gas_limit = gas_limit.min(u64::MAX.into()).low_u64();
			let without_base_extrinsic_weight = true;
			let (weight_limit, proof_size_base_cost) =
				match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					gas_limit,
					without_base_extrinsic_weight
				) {
					weight_limit if weight_limit.proof_size() > 0 => {
						(Some(weight_limit), Some(estimated_transaction_len as u64))
					}
					_ => (None, None),
				};
			let evm_config = config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config());
			<Runtime as pallet_evm::Config>::Runner::call(
				from,
				to,
				data,
				value,
				gas_limit.unique_saturated_into(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.unwrap_or_default(),
				is_transactional,
				validate,
				weight_limit,
				proof_size_base_cost,
				evm_config,
			).map_err(|err| err.error.into())
		}

		fn create(
			from: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
			use pallet_evm::GasWeightMapping;
			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			let is_transactional = false;
			let validate = true;
			let mut estimated_transaction_len = data.len() +
				// from: 20
				// value: 32
				// gas_limit: 32
				// nonce: 32
				// 1 byte transaction action variant
				// chain id 8 bytes
				// 65 bytes signature
				190;
			if max_fee_per_gas.is_some() {
				estimated_transaction_len += 32;
			}
			if max_priority_fee_per_gas.is_some() {
				estimated_transaction_len += 32;
			}
			if access_list.is_some() {
				estimated_transaction_len += access_list.encoded_size();
			}

			let gas_limit = gas_limit.min(u64::MAX.into()).low_u64();
			let without_base_extrinsic_weight = true;
			let (weight_limit, proof_size_base_cost) =
				match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					gas_limit,
					without_base_extrinsic_weight
				) {
					weight_limit if weight_limit.proof_size() > 0 => {
						(Some(weight_limit), Some(estimated_transaction_len as u64))
					}
					_ => (None, None),
				};
			let evm_config = config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config());
			<Runtime as pallet_evm::Config>::Runner::create(
				from,
				data,
				value,
				gas_limit.unique_saturated_into(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.unwrap_or_default(),
				is_transactional,
				validate,
				weight_limit,
				proof_size_base_cost,
				evm_config,
			).map_err(|err| err.error.into())
		}

		fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
			pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
		}

		fn current_block() -> Option<pallet_ethereum::Block> {
			pallet_ethereum::CurrentBlock::<Runtime>::get()
		}

		fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
			pallet_ethereum::CurrentReceipts::<Runtime>::get()
		}

		fn current_all() -> (
			Option<pallet_ethereum::Block>,
			Option<Vec<pallet_ethereum::Receipt>>,
			Option<Vec<TransactionStatus>>
		) {
			(
				pallet_ethereum::CurrentBlock::<Runtime>::get(),
				pallet_ethereum::CurrentReceipts::<Runtime>::get(),
				pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
			)
		}

		fn extrinsic_filter(
			xts: Vec<<Block as BlockT>::Extrinsic>,
		) -> Vec<EthereumTransaction> {
			xts.into_iter().filter_map(|xt| match xt.0.function {
				RuntimeCall::Ethereum(transact { transaction }) => Some(transaction),
				_ => None
			}).collect::<Vec<EthereumTransaction>>()
		}

		fn elasticity() -> Option<Permill> {
			Some(pallet_base_fee::Elasticity::<Runtime>::get())
		}

		fn gas_limit_multiplier_support() {}

		fn pending_block(
			xts: Vec<<Block as BlockT>::Extrinsic>,
		) -> (Option<pallet_ethereum::Block>, Option<Vec<TransactionStatus>>) {
			for ext in xts.into_iter() {
				let _ = Executive::apply_extrinsic(ext);
			}

			Ethereum::on_finalize(System::block_number() + 1);

			(
				pallet_ethereum::CurrentBlock::<Runtime>::get(),
				pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
			)
		}
	}

	impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
		fn convert_transaction(transaction: EthereumTransaction) -> <Block as BlockT>::Extrinsic {
			UncheckedExtrinsic::new_unsigned(
				pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
			)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		Balance,
	> for Runtime {
		fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> fg_primitives::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Grandpa::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			use parity_scale_codec::Encode;

			Historical::prove((fg_primitives::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(fg_primitives::OpaqueKeyOwnershipProof::new)
		}
	}

	impl rpc_primitives_debug::DebugRuntimeApi<Block> for Runtime {
		fn trace_transaction(
			extrinsics: Vec<<Block as BlockT>::Extrinsic>,
			traced_transaction: &EthereumTransaction,
		) -> Result<
			(),
			sp_runtime::DispatchError,
		> {
			#[cfg(feature = "evm-tracing")]
			{
				use evm_tracer::tracer::EvmTracer;
				// Apply the a subset of extrinsics: all the substrate-specific or ethereum
				// transactions that preceded the requested transaction.
				for ext in extrinsics.into_iter() {
					let _ = match &ext.0.function {
						RuntimeCall::Ethereum(transact { transaction }) => {
							if transaction == traced_transaction {
								EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
								return Ok(());
							} else {
								Executive::apply_extrinsic(ext)
							}
						}
						_ => Executive::apply_extrinsic(ext),
					};
				}
				Ok(())
			}
			#[cfg(not(feature = "evm-tracing"))]
			Err(sp_runtime::DispatchError::Other(
				"Missing `evm-tracing` compile time feature flag.",
			))
		}

		fn trace_block(
			extrinsics: Vec<<Block as BlockT>::Extrinsic>,
			known_transactions: Vec<H256>,
		) -> Result<
			(),
			sp_runtime::DispatchError,
		> {
			#[cfg(feature = "evm-tracing")]
			{
				use evm_tracer::tracer::EvmTracer;

				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;

				// Apply all extrinsics. Ethereum extrinsics are traced.
				for ext in extrinsics.into_iter() {
					match &ext.0.function {
						RuntimeCall::Ethereum(transact { transaction }) => {
							if known_transactions.contains(&transaction.hash()) {
								// Each known extrinsic is a new call stack.
								EvmTracer::emit_new();
								EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
							} else {
								let _ = Executive::apply_extrinsic(ext);
							}
						}
						_ => {
							let _ = Executive::apply_extrinsic(ext);
						}
					};
				}

				Ok(())
			}
			#[cfg(not(feature = "evm-tracing"))]
			Err(sp_runtime::DispatchError::Other(
				"Missing `evm-tracing` compile time feature flag.",
			))
		}
	}

	impl rpc_primitives_txpool::TxPoolRuntimeApi<Block> for Runtime {
		fn extrinsic_filter(
			xts_ready: Vec<<Block as BlockT>::Extrinsic>,
			xts_future: Vec<<Block as BlockT>::Extrinsic>,
		) -> rpc_primitives_txpool::TxPoolResponse {
			rpc_primitives_txpool::TxPoolResponse {
				ready: xts_ready
					.into_iter()
					.filter_map(|xt| match xt.0.function {
						RuntimeCall::Ethereum(transact { transaction }) => Some(transaction),
						_ => None,
					})
					.collect(),
				future: xts_future
					.into_iter()
					.filter_map(|xt| match xt.0.function {
						RuntimeCall::Ethereum(transact { transaction }) => Some(transaction),
						_ => None,
					})
					.collect(),
			}
		}
	}
	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark, baseline, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;

			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			let mut list = Vec::<BenchmarkList>::new();

			list_benchmark!(list, extra, pallet_balances, Balances);
			list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
			list_benchmark!(list, extra, pallet_timestamp, Timestamp);
			list_benchmark!(list, extra, pallet_dkg_proposal_handler, DKGProposalHandler);

			let storage_info = AllPalletsWithSystem::storage_info();

			return (list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, pallet_dkg_proposal_handler, DKGProposalHandler);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}
