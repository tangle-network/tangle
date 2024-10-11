// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 512.
#![recursion_limit = "512"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod filters;
pub mod frontier_evm;
pub mod impls;
pub mod migrations;
pub mod precompiles;
pub mod tangle_services;
pub mod voter_bags;

use frame_election_provider_support::{
	bounds::{ElectionBounds, ElectionBoundsBuilder},
	onchain, BalancingConfig, ElectionDataProvider, SequentialPhragmen, VoteWeight,
};
use frame_support::derive_impl;
use frame_support::genesis_builder_helper::build_state;
use frame_support::genesis_builder_helper::get_preset;
use frame_support::{
	traits::{
		tokens::{PayFromAccount, UnityAssetBalanceConversion},
		AsEnsureOriginWithArg, Contains, OnFinalize, WithdrawReasons,
	},
	weights::ConstantMultiplier,
};
use frame_system::EnsureSigned;
use frontier_evm::DefaultBaseFeePerGas;
use pallet_election_provider_multi_phase::{GeometricDepositBase, SolutionAccuracyOf};
use pallet_evm::GasWeightMapping;
use pallet_grandpa::{
	fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_services_rpc_runtime_api::BlockNumberOf;
use pallet_session::historical as pallet_session_historical;
pub use pallet_staking::StakerStatus;
#[allow(deprecated)]
use pallet_transaction_payment::{
	CurrencyAdapter, FeeDetails, Multiplier, RuntimeDispatchInfo, TargetedFeeAdjustment,
};
use pallet_tx_pause::RuntimeCallNameOf;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use precompiles::TanglePrecompiles;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata, H160, H256, U256};
use sp_genesis_builder::PresetId;
use sp_runtime::{
	create_runtime_str,
	curve::PiecewiseLinear,
	generic, impl_opaque_keys,
	traits::{
		self, BlakeTwo256, Block as BlockT, Bounded, Convert, ConvertInto, DispatchInfoOf,
		Dispatchable, IdentityLookup, NumberFor, OpaqueKeys, PostDispatchInfoOf, StaticLookup,
		UniqueSaturatedInto,
	},
	transaction_validity::{
		TransactionPriority, TransactionSource, TransactionValidity, TransactionValidityError,
	},
	ApplyExtrinsicResult, FixedPointNumber, FixedU128, Perquintill, RuntimeDebug,
	SaturatedConversion,
};
use sp_std::{prelude::*, vec::Vec};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use static_assertions::const_assert;
// use sygma_traits::{
// 	ChainID, DecimalConverter, DepositNonce, DomainID, ExtractDestinationData, ResourceId,
// 	VerifyingContractAddress,
// };
use tangle_primitives::services::RpcServicesWithBlueprint;

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
#[cfg(any(feature = "std", test))]
pub use frame_system::Call as SystemCall;
use frame_system::EnsureRoot;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
use sp_runtime::generic::Era;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{MultiAddress, Perbill, Percent, Permill};
use sp_staking::currency_to_vote::U128CurrencyToVote;
pub use tangle_primitives::{
	currency::*,
	fee::*,
	time::*,
	types::{
		AccountId, AccountIndex, Address, Balance, BlockNumber, Hash, Header, Index, Moment,
		Signature,
	},
	verifier::{arkworks::ArkworksVerifierGroth16Bn254, circom::CircomVerifierGroth16Bn254},
	BabeId, AVERAGE_ON_INITIALIZE_RATIO, MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO,
};
use tangle_primitives::{
	democracy::{
		COOLOFF_PERIOD, ENACTMENT_PERIOD, FASTTRACK_VOTING_PERIOD, LAUNCH_PERIOD, MAX_PROPOSALS,
		MINIMUM_DEPOSIT, VOTING_PERIOD,
	},
	elections::{
		CANDIDACY_BOND, DESIRED_MEMBERS, DESIRED_RUNNERS_UP, ELECTIONS_PHRAGMEN_PALLET_ID,
		MAX_CANDIDATES, MAX_VOTERS, TERM_DURATION,
	},
	staking::{
		BONDING_DURATION, HISTORY_DEPTH, MAX_NOMINATOR_REWARDED_PER_VALIDATOR, OFFCHAIN_REPEAT,
		SESSIONS_PER_ERA, SLASH_DEFER_DURATION,
	},
	treasury::{
		BURN, DATA_DEPOSIT_PER_BYTE, MAXIMUM_REASON_LENGTH, MAX_APPROVALS, PROPOSAL_BOND,
		PROPOSAL_BOND_MINIMUM, SPEND_PERIOD, TIP_COUNTDOWN, TIP_FINDERS_FEE,
		TIP_REPORT_DEPOSIT_BASE, TREASURY_PALLET_ID,
	},
};
pub use tangle_services::PalletServicesConstraints;

// Precompiles
pub type Precompiles = TanglePrecompiles<Runtime>;

// Frontier
use fp_rpc::TransactionStatus;
use pallet_ethereum::{Call::transact, Transaction as EthereumTransaction};
use pallet_evm::{Account as EVMAccount, FeeCalculator, HashedAddressMapping, Runner};
pub type Nonce = u32;

/// The BABE epoch configuration at genesis.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
	sp_consensus_babe::BabeEpochConfiguration {
		c: PRIMARY_PROBABILITY,
		allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
	};

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("tangle-testnet"),
	impl_name: create_runtime_str!("tangle-testnet"),
	authoring_version: 1,
	spec_version: 1200, // v1.2.0
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 0,
};

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

	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub babe: Babe,
			pub grandpa: Grandpa,
			pub im_online: ImOnline,
		}
	}
}

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = filters::TestnetCallFilter;
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
	type RuntimeTask = RuntimeTask;
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
	type OnTimestampSet = Babe;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = EXISTENTIAL_DEPOSIT / 100_000_000;
	pub const TransferFee: u128 = MILLIUNIT;
	pub const CreationFee: u128 = MILLIUNIT;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
	pub const MaxFreezes: u32 = 50;
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
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = MaxFreezes;
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
	#[allow(deprecated)]
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
	type Consideration = ();
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

parameter_types! {
	// NOTE: Currently it is not possible to change the epoch duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS / 2;
	pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
	pub const ReportLongevity: u64 =
		BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
	pub const MaxAuthorities: u32 = 1000;
	pub const MaxNominators: u32 = 1000;
}

impl pallet_babe::Config for Runtime {
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
	type EpochChangeTrigger = pallet_babe::ExternalTrigger;
	type DisabledValidators = Session;
	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
	type MaxNominators = MaxNominatorRewardedPerValidator;
	type KeyOwnerProof =
		<Historical as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;
	type EquivocationReportSystem =
		pallet_babe::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxSetIdSessionEntries = frame_support::traits::ConstU64<0>;
	type MaxAuthorities = MaxAuthorities;
	type EquivocationReportSystem = ();
	type KeyOwnerProof = sp_core::Void;
	type MaxNominators = MaxNominatorRewardedPerValidator;
	type WeightInfo = ();
}

impl pallet_authorship::Config for Runtime {
	type EventHandler = Staking;
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
}

use crate::opaque::SessionKeys;

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_staking::StashOf<Self>;
	type ShouldEndSession = Babe;
	type NextSessionRotation = Babe;
	type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = ();
}

impl pallet_session::historical::Config for Runtime {
	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

// Staking reward curve, more details at
// https://docs.rs/pallet-staking-reward-curve/latest/pallet_staking_reward_curve/macro.build.html
// We are aiming for a max inflation of 5%, when 60% of tokens are staked
// In practical sense, our reward rate will fluctuate between 2.5%-5% since the staked token count
// varies
pallet_staking_reward_curve::build! {
	const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
		min_inflation: 0_025_000, // min inflation of 2.5%
		max_inflation: 0_050_000, // max inflation of 5% (acheived only at ideal stake)
		ideal_stake: 0_600_000, // ideal stake (60% of total supply)
		falloff: 0_050_000,
		max_piece_count: 40,
		test_precision: 0_005_000,
	);
}

parameter_types! {
	// Six sessions in an era (24 hours).
	pub const SessionsPerEra: sp_staking::SessionIndex = SESSIONS_PER_ERA / 2;
	// 28 eras for unbonding (28 days).
	pub const BondingDuration: sp_staking::EraIndex = BONDING_DURATION;
	// 27 eras for slash defer duration (27 days).
	pub const SlashDeferDuration: sp_staking::EraIndex = SLASH_DEFER_DURATION;
	pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
	pub const MaxNominatorRewardedPerValidator: u32 = MAX_NOMINATOR_REWARDED_PER_VALIDATOR;
	//pub const OffendingValidatorsThreshold: Perbill = OFFENDING_VALIDATOR_THRESHOLD;
	pub OffchainRepeat: BlockNumber = OFFCHAIN_REPEAT;
	pub const HistoryDepth: u32 = HISTORY_DEPTH;
}

pub struct StakingBenchmarkingConfig;
impl pallet_staking::BenchmarkingConfig for StakingBenchmarkingConfig {
	type MaxNominators = ConstU32<1000>;
	type MaxValidators = ConstU32<1000>;
}

/// Upper limit on the number of NPOS nominations.
const MAX_QUOTA_NOMINATIONS: u32 = 16;

impl pallet_staking::Config for Runtime {
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
	type MaxExposurePageSize = ConstU32<64>;
	type MaxControllersInDeprecationBatch = ConstU32<100>;
	type ElectionProvider = ElectionProviderMultiPhase;
	type GenesisElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type VoterList = BagsList;
	type MaxUnlockingChunks = ConstU32<32>;
	type HistoryDepth = HistoryDepth;
	type EventListeners = NominationPools;
	type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
	type NominationsQuota = pallet_staking::FixedNominationsQuota<MAX_QUOTA_NOMINATIONS>;
	type BenchmarkingConfig = StakingBenchmarkingConfig;
	type DisablingStrategy = pallet_staking::UpToLimitDisablingStrategy;
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = LAUNCH_PERIOD;
	pub const VotingPeriod: BlockNumber = VOTING_PERIOD;
	pub const FastTrackVotingPeriod: BlockNumber = FASTTRACK_VOTING_PERIOD;
	pub const MinimumDeposit: Balance = MINIMUM_DEPOSIT;
	pub const EnactmentPeriod: BlockNumber = ENACTMENT_PERIOD;
	pub const CooloffPeriod: BlockNumber = COOLOFF_PERIOD;
	pub const MaxProposals: u32 = MAX_PROPOSALS;
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
	pub ElectionBoundsOnChain: ElectionBounds = ElectionBoundsBuilder::default()
		.voters_count(5_000.into()).targets_count(1_250.into()).build();
	pub ElectionBoundsMultiPhase: ElectionBounds = ElectionBoundsBuilder::default()
		.voters_count(10_000.into()).targets_count(1_500.into()).build();
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
					.expect("input is padded with zeroes; qed")
					% max.saturating_add(1);
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
	type Bounds = ElectionBoundsOnChain;
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

parameter_types! {
	pub const SignedFixedDeposit: Balance = 1;
	pub const SignedDepositIncreaseFactor: Percent = Percent::from_percent(10);
}

impl pallet_election_provider_multi_phase::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EstimateCallFee = TransactionPayment;
	type SignedPhase = SignedPhase;
	type UnsignedPhase = UnsignedPhase;
	type BetterSignedThreshold = ();
	type OffchainRepeat = OffchainRepeat;
	type MinerTxPriority = MultiPhaseUnsignedPriority;
	type MinerConfig = Self;
	type SignedMaxSubmissions = ConstU32<10>;
	type SignedRewardBase = SignedRewardBase;
	type SignedDepositBase =
		GeometricDepositBase<Balance, SignedFixedDeposit, SignedDepositIncreaseFactor>;
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
	type MaxWinners = MaxActiveValidators;
	type BenchmarkingConfig = ElectionProviderBenchmarkConfig;
	type ElectionBounds = ElectionBoundsMultiPhase;
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
		n.try_into().unwrap_or(Balance::MAX)
	}
}

impl pallet_nomination_pools::Config for Runtime {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type RewardCounter = FixedU128;
	type BalanceToU256 = BalanceToU256;
	type U256ToBalance = U256ToBalance;
	type PostUnbondingPoolsWindow = PostUnbondPoolsWindow;
	type MaxMetadataLen = ConstU32<256>;
	type MaxUnbonding = ConstU32<8>;
	type PalletId = NominationPoolsPalletId;
	type MaxPointsToBalance = MaxPointsToBalance;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type AdminOrigin = EnsureRoot<AccountId>;
	type StakeAdapter = pallet_nomination_pools::adapter::TransferStake<Self, Staking>;
}

parameter_types! {
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::MAX;
	/// We prioritize im-online heartbeats over election solution submission.
	pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::MAX / 2;
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
			frame_metadata_hash_extension::CheckMetadataHash::<Runtime>::new(true),
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
	type BlockNumberProvider = System;
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
	pub const CandidacyBond: Balance = CANDIDACY_BOND;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub const VotingBondBase: Balance = deposit(1, 64);
	// additional data per vote is 32 bytes (account id).
	pub const VotingBondFactor: Balance = deposit(0, 32);
	pub const TermDuration: BlockNumber = TERM_DURATION;
	pub const DesiredMembers: u32 = DESIRED_MEMBERS;
	pub const DesiredRunnersUp: u32 = DESIRED_RUNNERS_UP;
	pub const MaxCandidates: u32 = MAX_CANDIDATES;
	pub const MaxVoters: u32 = MAX_VOTERS;
	pub const ElectionsPhragmenPalletId: LockIdentifier = ELECTIONS_PHRAGMEN_PALLET_ID;
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
	pub const ProposalBond: Permill = PROPOSAL_BOND;
	pub const ProposalBondMinimum: Balance = PROPOSAL_BOND_MINIMUM;
	pub const SpendPeriod: BlockNumber = SPEND_PERIOD;
	pub const Burn: Permill = BURN;
	pub const TipCountdown: BlockNumber = TIP_COUNTDOWN;
	pub const TipFindersFee: Percent = TIP_FINDERS_FEE;
	pub const TipReportDepositBase: Balance = TIP_REPORT_DEPOSIT_BASE;
	pub const DataDepositPerByte: Balance = DATA_DEPOSIT_PER_BYTE;
	pub const TreasuryPalletId: PalletId = TREASURY_PALLET_ID;
	pub const MaximumReasonLength: u32 = MAXIMUM_REASON_LENGTH;
	pub const MaxApprovals: u32 = MAX_APPROVALS;
	pub TreasuryAccount: AccountId = Treasury::account_id();
	pub PayoutPeriod: u64 = 10;
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type RejectOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
	>;
	type RuntimeEvent = RuntimeEvent;
	type AssetKind = ();
	type Beneficiary = AccountId;
	type BeneficiaryLookup = IdentityLookup<AccountId>;
	type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
	type BalanceConverter = UnityAssetBalanceConversion;
	type PayoutPeriod = PayoutPeriod;
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
	type OnSlash = Treasury;
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
	type NextSessionRotation = Babe;
	type ValidatorSet = Historical;
	type ReportUnresponsiveness = ();
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
	type MaxKeys = MaxKeys;
	type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
}

/// Calls that cannot be paused by the tx-pause pallet.
pub struct TxPauseWhitelistedCalls;
/// Whitelist `Balances::transfer_keep_alive`, all others are pauseable.
impl Contains<RuntimeCallNameOf<Runtime>> for TxPauseWhitelistedCalls {
	fn contains(full_name: &RuntimeCallNameOf<Runtime>) -> bool {
		matches!(
			(full_name.0.as_slice(), full_name.1.as_slice()),
			(b"Balances", b"transfer_keep_alive")
		)
	}
}

impl pallet_tx_pause::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PauseOrigin = EnsureRoot<AccountId>;
	type UnpauseOrigin = EnsureRoot<AccountId>;
	type WhitelistedCalls = TxPauseWhitelistedCalls;
	type MaxNameLen = ConstU32<256>;
	type WeightInfo = pallet_tx_pause::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const BasicDeposit: Balance = deposit(0, 100);
	pub const ByteDeposit: Balance = deposit(0, 100);
	pub const SubAccountDeposit: Balance = deposit(1, 1);
	pub const MaxSubAccounts: u32 = 100;
	#[derive(Serialize, Deserialize)]
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
	pub const PendingUsernameExpiration: u64 = 7 * DAYS;
}

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = Treasury;
	type ByteDeposit = ByteDeposit;
	type IdentityInformation = pallet_identity::legacy::IdentityInfo<MaxAdditionalFields>;
	type OffchainSignature = Signature;
	type SigningPublicKey = <Signature as traits::Verify>::Signer;
	type UsernameAuthorityOrigin = EnsureRoot<AccountId>;
	type PendingUsernameExpiration = PendingUsernameExpiration;
	type MaxSuffixLength = ConstU32<7>;
	type MaxUsernameLength = ConstU32<32>;
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

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = ConstU32<100>;
	type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub Prefix: &'static [u8] = b"Claim TNTs to the account:";
	pub MaxVestingSchedules: u32 = 16;
}

impl pallet_airdrop_claims::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type VestingSchedule = Vesting;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type AddressMapping = HashedAddressMapping<BlakeTwo256>;
	type Prefix = Prefix;
	type MaxVestingSchedules = MaxVestingSchedules;
	type MoveClaimOrigin = frame_system::EnsureRoot<AccountId>;
	type WeightInfo = ();
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
pub enum ProxyType {
	Any,
	NonTransfer,
	Governance,
	Staking,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => !matches!(
				c,
				RuntimeCall::Balances(..)
					| RuntimeCall::Vesting(pallet_vesting::Call::vested_transfer { .. })
			),
			ProxyType::Governance => matches!(
				c,
				RuntimeCall::Democracy(..)
					| RuntimeCall::Council(..)
					| RuntimeCall::Elections(..)
					| RuntimeCall::Treasury(..)
			),
			ProxyType::Staking => {
				matches!(c, RuntimeCall::Staking(..))
			},
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = ConstU32<32>;
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
	type MaxPending = ConstU32<32>;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

parameter_types! {
	pub const PostUnbondingPoolsWindow: u32 = 2;
	pub const MaxMetadataLen: u32 = 2;
	pub const CheckLevel: u8 = 255;
	pub const LstPalletId: PalletId = PalletId(*b"py/tnlst");
}

impl pallet_tangle_lst::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = Balances;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type RewardCounter = FixedU128;
	type BalanceToU256 = BalanceToU256;
	type U256ToBalance = U256ToBalance;
	type Staking = Staking;
	type PostUnbondingPoolsWindow = ConstU32<4>;
	type PalletId = LstPalletId;
	type MaxMetadataLen = MaxMetadataLen;
	// we use the same number of allowed unlocking chunks as with staking.
	type MaxUnbonding = <Self as pallet_staking::Config>::MaxUnlockingChunks;
	type Fungibles = Assets;
	type AssetId = AssetId;
	type PoolId = AssetId;
	type MaxNameLength = ConstU32<50>;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxPointsToBalance = frame_support::traits::ConstU8<10>;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime {
		System: frame_system = 1,
		Timestamp: pallet_timestamp = 2,

		Sudo: pallet_sudo = 3,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip = 4,

		Assets: pallet_assets = 5,
		Balances: pallet_balances = 6,
		TransactionPayment: pallet_transaction_payment = 7,

		Authorship: pallet_authorship = 8,
		Babe: pallet_babe = 9,
		Grandpa: pallet_grandpa = 10,

		Indices: pallet_indices = 11,
		Democracy: pallet_democracy = 12,
		Council: pallet_collective::<Instance1> = 13,
		Vesting: pallet_vesting = 14,

		Elections: pallet_elections_phragmen = 15,
		ElectionProviderMultiPhase: pallet_election_provider_multi_phase = 16,
		Staking: pallet_staking = 17,
		Session: pallet_session = 18,
		Historical: pallet_session_historical = 19,
		Treasury: pallet_treasury = 20,
		Bounties: pallet_bounties = 21,
		ChildBounties: pallet_child_bounties = 22,
		BagsList: pallet_bags_list = 23,
		NominationPools: pallet_nomination_pools = 24,

		Scheduler: pallet_scheduler = 25,
		Preimage: pallet_preimage = 26,
		Offences: pallet_offences = 27,

		TxPause: pallet_tx_pause = 28,
		ImOnline: pallet_im_online = 29,
		Identity: pallet_identity = 30,
		Utility: pallet_utility = 31,
		Multisig: pallet_multisig = 32,

		Ethereum: pallet_ethereum = 33,
		EVM: pallet_evm = 34,
		EVMChainId: pallet_evm_chain_id = 35,
		DynamicFee: pallet_dynamic_fee = 36,
		BaseFee: pallet_base_fee = 37,
		HotfixSufficients: pallet_hotfix_sufficients = 38,

		Claims: pallet_airdrop_claims = 39,

		// DO NOT USE below indexes
		// Roles: pallet_roles = 40,
		// Jobs: pallet_jobs = 41,
		// Dkg: pallet_dkg = 42,
		// ZkSaaS: pallet_zksaas = 43,

		Proxy: pallet_proxy = 44,
		MultiAssetDelegation: pallet_multi_asset_delegation = 45,
		Services: pallet_services = 51,
		Lst: pallet_tangle_lst = 52,

		// Sygma
		// SygmaAccessSegregator: sygma_access_segregator = 46,
		// SygmaBasicFeeHandler: sygma_basic_feehandler = 47,
		// SygmaFeeHandlerRouter: sygma_fee_handler_router = 48,
		// SygmaPercentageFeeHandler: sygma_percentage_feehandler = 49,
		// SygmaBridge: sygma_bridge = 50,

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
	frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
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
	migrations::MigrateSessionKeys<Runtime>,
>;

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
			RuntimeCall::Ethereum(call) => {
				call.pre_dispatch_self_contained(info, dispatch_info, len)
			},
			_ => None,
		}
	}

	fn apply_self_contained(
		self,
		info: Self::SignedInfo,
	) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
		match self {
			call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) => {
				Some(call.dispatch(RuntimeOrigin::from(
					pallet_ethereum::RawOrigin::EthereumTransaction(info),
				)))
			},
			_ => None,
		}
	}
}

parameter_types! {
	pub const AssetDeposit: Balance = 10 * UNIT;
	pub const AssetAccountDeposit: Balance = DOLLAR;
	pub const ApprovalDeposit: Balance = ExistentialDeposit::get();
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = deposit(1, 68);
	pub const MetadataDepositPerByte: Balance = deposit(0, 1);
}

#[cfg(not(feature = "runtime-benchmarks"))]
pub type AssetId = u128;

#[cfg(feature = "runtime-benchmarks")]
pub type AssetId = u32;

impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type AssetIdParameter = parity_scale_codec::Compact<AssetId>;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = AssetAccountDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type RemoveItemsLimit = ConstU32<1000>;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

parameter_types! {
	pub const MinOperatorBondAmount: Balance = 10_000;
	pub const BondDuration: u32 = 10;
	pub const MinDelegateAmount : Balance = 1000;
	pub PID: PalletId = PalletId(*b"PotStake");
}

impl pallet_multi_asset_delegation::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MinOperatorBondAmount = MinOperatorBondAmount;
	type BondDuration = BondDuration;
	type ServiceManager = Services;
	type LeaveOperatorsDelay = ConstU32<10>;
	type OperatorBondLessDelay = ConstU32<1>;
	type LeaveDelegatorsDelay = ConstU32<1>;
	type DelegationBondLessDelay = ConstU32<5>;
	type MinDelegateAmount = MinDelegateAmount;
	type Fungibles = Assets;
	type AssetId = AssetId;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type PalletId = PID;
	type VaultId = AssetId;
	type WeightInfo = ();
}

// parameter_types! {
// 	pub const SygmaAccessSegregatorPalletIndex: u8 = 90;
// 	pub const SygmaBasicFeeHandlerPalletIndex: u8 = 91;
// 	pub const SygmaFeeHandlerRouterPalletIndex: u8 = 92;
// 	pub const SygmaPercentageFeeHandlerRouterPalletIndex: u8 = 93;
// 	pub const SygmaBridgePalletIndex: u8 = 94;
// }

// pub struct SygmaAdminMembers;
// impl SortedMembers<AccountId> for SygmaAdminMembers {
// 	fn sorted_members() -> Vec<AccountId> {
// 		[SygmaBridgeAdminAccount::get()].to_vec()
// 	}
// }

// impl sygma_bridge::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;
// 	type TransferReserveAccounts = BridgeAccounts;
// 	type FeeReserveAccount = SygmaBridgeFeeAccount;
// 	type EIP712ChainID = EIP712ChainID;
// 	type DestVerifyingContractAddress = DestVerifyingContractAddress;
// 	type FeeHandler = SygmaFeeHandlerRouter;
// 	type AssetTransactor = (CurrencyTransactor, FungiblesTransactor);
// 	type ResourcePairs = ResourcePairs;
// 	type IsReserve = ReserveChecker;
// 	type ExtractDestData = DestinationDataParser;
// 	type PalletId = SygmaBridgePalletId;
// 	type PalletIndex = SygmaBridgePalletIndex;
// 	type DecimalConverter = SygmaDecimalConverter<AssetDecimalPairs>;
// 	type WeightInfo = sygma_bridge::weights::SygmaWeightInfo<Runtime>;
// }

// pub type LocationToAccountId = (
// 	// The parent (Relay-chain) origin converts to the parent `AccountId`.
// 	ParentIsPreset<sp_core::crypto::AccountId32>,
// 	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
// 	SiblingParachainConvertsVia<Sibling, sp_core::crypto::AccountId32>,
// 	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
// 	AccountId32Aliases<RelayNetwork, sp_core::crypto::AccountId32>,
// );

// #[allow(deprecated)]
// pub type CurrencyTransactor = XcmCurrencyAdapter<
// 	// Use this currency:
// 	Balances,
// 	// Use this currency when it is a fungible asset matching the given location or name:
// 	IsConcrete<NativeLocation>,
// 	// Convert an XCM Location into a local account id:
// 	LocationToAccountId,
// 	// Our chain's account ID type (we can't get away without mentioning it explicitly):
// 	AccountId32,
// 	// We don't track any teleports of `Balances`.
// 	(),
// >;

// /// Means for transacting assets besides the native currency on this chain.
// pub type FungiblesTransactor = FungiblesAdapter<
// 	// Use this fungibles implementation:
// 	Assets,
// 	// Use this currency when it is a fungible asset matching the given location or name:
// 	SimpleForeignAssetConverter,
// 	// Convert an XCM Location into a local account id:
// 	LocationToAccountId,
// 	// Our chain's account ID type (we can't get away without mentioning it explicitly):
// 	AccountId32,
// 	// Disable teleport.
// 	NoChecking,
// 	// The account to use for tracking teleports.
// 	CheckingAccount,
// >;

// pub struct SimpleForeignAssetConverter(PhantomData<()>);
// impl MatchesFungibles<AssetId, Balance> for SimpleForeignAssetConverter {
// 	fn matches_fungibles(a: &Asset) -> result::Result<(AssetId, Balance), ExecutionError> {
// 		match (&a.fun, &a.id) {
// 			(Fungible(ref amount), AssetId(ref id)) => {
// 				if id == &SygUSDLocation::get() {
// 					Ok((SygUSDAssetId::get(), *amount))
// 				} else if id == &PHALocation::get() {
// 					Ok((PHAAssetId::get(), *amount))
// 				} else {
// 					Err(ExecutionError::AssetNotHandled)
// 				}
// 			},
// 			_ => Err(ExecutionError::AssetNotHandled),
// 		}
// 	}
// }

// impl sygma_access_segregator::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;
// 	type BridgeCommitteeOrigin = EnsureSignedBy<SygmaAdminMembers, AccountId>;
// 	type PalletIndex = SygmaAccessSegregatorPalletIndex;
// 	type Extrinsics = RegisteredExtrinsics;
// 	type WeightInfo = sygma_access_segregator::weights::SygmaWeightInfo<Runtime>;
// }

// impl sygma_basic_feehandler::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;
// 	type PalletIndex = SygmaBasicFeeHandlerPalletIndex;
// 	type WeightInfo = sygma_basic_feehandler::weights::SygmaWeightInfo<Runtime>;
// }

// impl sygma_fee_handler_router::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;
// 	type BasicFeeHandler = SygmaBasicFeeHandler;
// 	type DynamicFeeHandler = ();

// 	type PercentageFeeHandler = SygmaPercentageFeeHandler;
// 	type PalletIndex = SygmaFeeHandlerRouterPalletIndex;
// 	type WeightInfo = sygma_fee_handler_router::weights::SygmaWeightInfo<Runtime>;
// }

// impl sygma_percentage_feehandler::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;
// 	type PalletIndex = SygmaPercentageFeeHandlerRouterPalletIndex;
// 	type WeightInfo = sygma_percentage_feehandler::weights::SygmaWeightInfo<Runtime>;
// }

// parameter_types! {
// 	// tTNT: native asset is always a reserved asset
// 	pub NativeLocation: Location = Location::here();
// 	pub NativeSygmaResourceId: [u8; 32] = hex_literal::hex!("0000000000000000000000000000000000000000000000000000000000002000");

// 	// SygUSD: a non-reserved asset
// 	pub SygUSDLocation: Location = Location::new(
// 		1,
// 		[
// 			Parachain(1000),
// 			slice_to_generalkey(b"sygma"),
// 			slice_to_generalkey(b"sygusd"),
// 		],
// 	);
// 	// SygUSDAssetId is the substrate assetID of SygUSD
// 	pub SygUSDAssetId: AssetId = 2000;
// 	// SygUSDResourceId is the resourceID that mapping with the foreign asset SygUSD
// 	pub SygUSDResourceId: ResourceId = hex_literal::hex!("0000000000000000000000000000000000000000000000000000000000001100");

// 	// PHA: a reserved asset
// 	pub PHALocation: Location = Location::new(
// 		1,
// 		[
// 			Parachain(2004),
// 			slice_to_generalkey(b"sygma"),
// 			slice_to_generalkey(b"pha"),
// 		],
// 	);
// 	// PHAAssetId is the substrate assetID of PHA
// 	pub PHAAssetId: AssetId = 2001;
// 	// PHAResourceId is the resourceID that mapping with the foreign asset PHA
// 	pub PHAResourceId: ResourceId = hex_literal::hex!("0000000000000000000000000000000000000000000000000000000000001000");
// }

// fn bridge_accounts_generator() -> BTreeMap<XcmAssetId, AccountId32> {
// 	let mut account_map: BTreeMap<XcmAssetId, AccountId32> = BTreeMap::new();
// 	account_map.insert(NativeLocation::get().into(), BridgeAccountNative::get());
// 	account_map.insert(SygUSDLocation::get().into(), BridgeAccountOtherToken::get());
// 	account_map.insert(PHALocation::get().into(), BridgeAccountOtherToken::get());
// 	account_map
// }

// const DEST_VERIFYING_CONTRACT_ADDRESS: &str = "6CdE2Cd82a4F8B74693Ff5e194c19CA08c2d1c68";
// parameter_types! {
// 	// RegisteredExtrinsics here registers all valid (pallet index, extrinsic_name) paris
// 	// make sure to update this when adding new access control extrinsic
// 	pub RegisteredExtrinsics: Vec<(u8, Vec<u8>)> = [
// 		(SygmaAccessSegregatorPalletIndex::get(), b"grant_access".to_vec()),
// 		(SygmaBasicFeeHandlerPalletIndex::get(), b"set_fee".to_vec()),
// 		(SygmaBridgePalletIndex::get(), b"set_mpc_address".to_vec()),
// 		(SygmaBridgePalletIndex::get(), b"pause_bridge".to_vec()),
// 		(SygmaBridgePalletIndex::get(), b"unpause_bridge".to_vec()),
// 		(SygmaBridgePalletIndex::get(), b"register_domain".to_vec()),
// 		(SygmaBridgePalletIndex::get(), b"unregister_domain".to_vec()),
// 		(SygmaBridgePalletIndex::get(), b"retry".to_vec()),
// 		(SygmaFeeHandlerRouterPalletIndex::get(), b"set_fee_handler".to_vec()),
// 		(SygmaPercentageFeeHandlerRouterPalletIndex::get(), b"set_fee_rate".to_vec()),
// 	].to_vec();

// 	pub const SygmaBridgePalletId: PalletId = PalletId(*b"sygma/01");

// 	// Tangle testnet super admin: 5D2hZnw8Z7kg5LpQiEBb6HPG4V51wYXuKhE7sVhXiUPWj8D1
// 	pub SygmaBridgeAdminAccountKey: [u8; 32] = hex_literal::hex!("2ab4c35efb6ab82377c2325467103cf46742d288ae1f8917f1d5960f4a1e9065");
// 	pub SygmaBridgeAdminAccount: AccountId = SygmaBridgeAdminAccountKey::get().into();

// 	// SygmaBridgeFeeAccount is a substrate account and used for bridging fee collection
// 	// SygmaBridgeFeeAccount address: 5ELLU7ibt5ZrNEYRwohtaRBDBa3TzcWwwPELBPSWWd2mbgv3
// 	pub SygmaBridgeFeeAccount: AccountId32 = AccountId32::new([100u8; 32]);

// 	// BridgeAccountNative: 5EYCAe5jLbHcAAMKvLFSXgCTbPrLgBJusvPwfKcaKzuf5X5e
// 	pub BridgeAccountNative: AccountId32 = SygmaBridgePalletId::get().into_account_truncating();
// 	// BridgeAccountOtherToken  5EYCAe5jLbHcAAMKvLFiGhk3htXY8jQncbLTDGJQnpnPMAVp
// 	pub BridgeAccountOtherToken: AccountId32 = SygmaBridgePalletId::get().into_sub_account_truncating(1u32);
// 	// BridgeAccounts is a list of accounts for holding transferred asset collection
// 	pub BridgeAccounts: BTreeMap<XcmAssetId, AccountId32> = bridge_accounts_generator();

// 	// EIP712ChainID is the chainID that pallet is assigned with, used in EIP712 typed data domain
// 	// For local testing with ./scripts/sygma-setup/execute_proposal_test.js, please change it to 5
// 	pub EIP712ChainID: ChainID = U256::from(3799);

// 	// DestVerifyingContractAddress is a H160 address that is used in proposal signature verification, specifically EIP712 typed data
// 	// When relayers signing, this address will be included in the EIP712Domain
// 	// As long as the relayer and pallet configured with the same address, EIP712Domain should be recognized properly.
// 	pub DestVerifyingContractAddress: VerifyingContractAddress = primitive_types::H160::from_slice(hex::decode(DEST_VERIFYING_CONTRACT_ADDRESS).ok().unwrap().as_slice());

// 	pub CheckingAccount: AccountId32 = AccountId32::new([102u8; 32]);

// 	pub RelayNetwork: NetworkId = NetworkId::Polkadot;
// 	// ResourcePairs is where all supported assets and their associated resourceID are binding
// 	pub ResourcePairs: Vec<(XcmAssetId, ResourceId)> = vec![
// 		(NativeLocation::get().into(), NativeSygmaResourceId::get()),
// 		(SygUSDLocation::get().into(), SygUSDResourceId::get()),
// 		(PHALocation::get().into(), PHAResourceId::get()),
// 	];

// 	pub AssetDecimalPairs: Vec<(XcmAssetId, u8)> = vec![(NativeLocation::get().into(), 18u8), (SygUSDLocation::get().into(), 6u8), (PHALocation::get().into(), 12u8)];
// }

// pub struct ReserveChecker;
// impl ContainsPair<Asset, Location> for ReserveChecker {
// 	fn contains(asset: &Asset, origin: &Location) -> bool {
// 		if let Some(ref id) = ConcrateSygmaAsset::origin(asset) {
// 			if id == origin {
// 				return true;
// 			}
// 		}
// 		false
// 	}
// }

// pub struct ConcrateSygmaAsset;
// impl ConcrateSygmaAsset {
// 	pub fn id(asset: &Asset) -> Option<Location> {
// 		match (&asset.id, &asset.fun) {
// 			(AssetId(ref id), Fungible(_)) => Some(id.clone()),
// 			_ => None,
// 		}
// 	}

// 	pub fn origin(asset: &Asset) -> Option<Location> {
// 		Self::id(asset).and_then(|id| {
// 			match (id.parents, id.first_interior()) {
// 				// Sibling parachain
// 				(1, Some(Parachain(id))) => {
// 					// Assume current parachain id is 1000, for production, always get proper
// 					// parachain info
// 					if *id == 1000 {
// 						Some(Location::new(0, X1(Arc::new([slice_to_generalkey(b"sygma")]))))
// 					} else {
// 						Some(Location::here())
// 					}
// 				},
// 				(1, _) => Some(Location::here()),
// 				// Children parachain
// 				(0, Some(Parachain(id))) => Some(Location::new(0, X1(Arc::new([Parachain(*id)])))),
// 				// Local: (0, Here)
// 				(0, None) => Some(id),
// 				_ => None,
// 			}
// 		})
// 	}
// }

// pub struct DestinationDataParser;
// /// Extract dest to be recipient and Dest DomainID
// /// if dest chain is substrate chain, recipient must be a encoded MultiLocation
// /// if dest chain is non-substrate chain, recipient is [u8; 32]
// impl ExtractDestinationData for crate::DestinationDataParser {
// 	fn extract_dest(dest: &Location) -> Option<(Vec<u8>, DomainID)> {
// 		match (dest.parents, dest.interior.clone()) {
// 			// final dest is on the remote substrate chain
// 			(1, X4(xs)) => {
// 				let [a, b, c, d] = *xs;
// 				match (a, b, c, d) {
// 					(
// 						GeneralKey { length: path_len, data: sygma_path },
// 						GeneralIndex(dest_domain_id),
// 						Parachain(parachain_id),
// 						Junction::AccountId32 { network: None, id: recipient },
// 					) => {
// 						if sygma_path[..path_len as usize] == [0x73, 0x79, 0x67, 0x6d, 0x61] {
// 							return TryInto::<DomainID>::try_into(dest_domain_id).ok().map(
// 								|domain_id| {
// 									let l: Location = Location::new(
// 										1,
// 										Junctions::X2(Arc::new([
// 											Parachain(parachain_id),
// 											Junction::AccountId32 { network: None, id: recipient },
// 										])),
// 									);
// 									(l.encode(), domain_id)
// 								},
// 							);
// 						}
// 						None
// 					},
// 					_ => None,
// 				}
// 			},
// 			(0, X3(xs)) => {
// 				let [a, b, c] = *xs;
// 				match (a, b, c) {
// 					// final dest is on the local substrate chain
// 					(
// 						GeneralKey { length: path_len, data: sygma_path },
// 						GeneralIndex(dest_domain_id),
// 						Junction::AccountId32 { network: None, id: recipient },
// 					) => {
// 						if sygma_path[..path_len as usize] == [0x73, 0x79, 0x67, 0x6d, 0x61] {
// 							return TryInto::<DomainID>::try_into(dest_domain_id).ok().map(
// 								|domain_id| {
// 									let l: Location = Location::new(
// 										0,
// 										Junctions::X1(Arc::new([Junction::AccountId32 {
// 											network: None,
// 											id: recipient,
// 										}])),
// 									);
// 									(l.encode(), domain_id)
// 								},
// 							);
// 						}
// 						None
// 					},
// 					// final dest is on the non-substrate chain such as EVM
// 					(
// 						GeneralKey { length: path_len, data: sygma_path },
// 						GeneralIndex(dest_domain_id),
// 						GeneralKey { length: recipient_len, data: recipient },
// 					) => {
// 						if sygma_path[..path_len as usize] == [0x73, 0x79, 0x67, 0x6d, 0x61] {
// 							return TryInto::<DomainID>::try_into(dest_domain_id).ok().map(
// 								|domain_id| {
// 									(recipient[..recipient_len as usize].to_vec(), domain_id)
// 								},
// 							);
// 						}
// 						None
// 					},
// 					_ => None,
// 				}
// 			},
// 			_ => None,
// 		}
// 	}
// }

// pub struct SygmaDecimalConverter<DecimalPairs>(PhantomData<DecimalPairs>);
// impl<DecimalPairs: Get<Vec<(XcmAssetId, u8)>>> DecimalConverter
// 	for SygmaDecimalConverter<DecimalPairs>
// {
// 	fn convert_to(asset: &Asset) -> Option<u128> {
// 		match (&asset.fun, &asset.id) {
// 			(Fungible(amount), _) => {
// 				for (asset_id, decimal) in DecimalPairs::get().iter() {
// 					if *asset_id == asset.id {
// 						return if *decimal == 18 {
// 							Some(*amount)
// 						} else {
// 							type U112F16 = DecimalFixedU128<U16>;
// 							if *decimal > 18 {
// 								let a =
// 									U112F16::from_num(10u128.saturating_pow(*decimal as u32 - 18));
// 								let b = U112F16::from_num(*amount).checked_div(a);
// 								let r: u128 = b.unwrap_or_else(|| U112F16::from_num(0)).to_num();
// 								if r == 0 {
// 									return None;
// 								}
// 								Some(r)
// 							} else {
// 								// Max is 5192296858534827628530496329220095
// 								// if source asset decimal is 12, the max amount sending to sygma
// 								// relayer is 5192296858534827.628530496329
// 								if *amount > U112F16::MAX {
// 									return None;
// 								}
// 								let a =
// 									U112F16::from_num(10u128.saturating_pow(18 - *decimal as u32));
// 								let b = U112F16::from_num(*amount).saturating_mul(a);
// 								Some(b.to_num())
// 							}
// 						};
// 					}
// 				}
// 				None
// 			},
// 			_ => None,
// 		}
// 	}

// 	fn convert_from(asset: &Asset) -> Option<Asset> {
// 		match (&asset.fun, &asset.id) {
// 			(Fungible(amount), _) => {
// 				for (asset_id, decimal) in DecimalPairs::get().iter() {
// 					if *asset_id == asset.id {
// 						return if *decimal == 18 {
// 							Some((asset.id.clone(), *amount).into())
// 						} else {
// 							type U112F16 = DecimalFixedU128<U16>;
// 							if *decimal > 18 {
// 								// Max is 5192296858534827628530496329220095
// 								// if dest asset decimal is 24, the max amount coming from sygma
// 								// relayer is 5192296858.534827628530496329
// 								if *amount > U112F16::MAX {
// 									return None;
// 								}
// 								let a =
// 									U112F16::from_num(10u128.saturating_pow(*decimal as u32 - 18));
// 								let b = U112F16::from_num(*amount).saturating_mul(a);
// 								let r: u128 = b.to_num();
// 								Some((asset.id.clone(), r).into())
// 							} else {
// 								let a =
// 									U112F16::from_num(10u128.saturating_pow(18 - *decimal as u32));
// 								let b = U112F16::from_num(*amount).checked_div(a);
// 								let r: u128 = b.unwrap_or_else(|| U112F16::from_num(0)).to_num();
// 								if r == 0 {
// 									return None;
// 								}
// 								Some((asset.id.clone(), r).into())
// 							}
// 						};
// 					}
// 				}
// 				None
// 			},
// 			_ => None,
// 		}
// 	}
// }

// pub fn slice_to_generalkey(key: &[u8]) -> Junction {
// 	let len = key.len();
// 	assert!(len <= 32);
// 	GeneralKey {
// 		length: len as u8,
// 		data: {
// 			let mut data = [0u8; 32];
// 			data[..len].copy_from_slice(key);
// 			data
// 		},
// 	}
// }

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		[frame_benchmarking, BaselineBench::<Runtime>]
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]
		[pallet_timestamp, Timestamp]
		[pallet_services, Services]
		[pallet_tangle_lst_benchmarking, LstBench]
	);
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
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

	// impl sygma_runtime_api::SygmaBridgeApi<Block> for Runtime {
	// 	fn is_proposal_executed(nonce: DepositNonce, domain_id: DomainID) -> bool {
	// 		SygmaBridge::is_proposal_executed(nonce, domain_id)
	// 	}
	// }

	impl pallet_services_rpc_runtime_api::ServicesApi<Block, PalletServicesConstraints, AccountId, AssetId> for Runtime {
		fn query_services_with_blueprints_by_operator(
			operator: AccountId,
		) -> Result<
			Vec<RpcServicesWithBlueprint<PalletServicesConstraints, AccountId, BlockNumberOf<Block>, AssetId>>,
			sp_runtime::DispatchError,
		> {
			Services::services_with_blueprints_by_operator(operator).map_err(Into::into)
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

		fn initialize_pending_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header);
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

	impl sp_consensus_babe::BabeApi<Block> for Runtime {
		fn configuration() -> sp_consensus_babe::BabeConfiguration {
			let epoch_config = Babe::epoch_config().unwrap_or(BABE_GENESIS_EPOCH_CONFIG);
			sp_consensus_babe::BabeConfiguration {
				slot_duration: Babe::slot_duration(),
				epoch_length: EpochDuration::get(),
				c: epoch_config.c,
				authorities: Babe::authorities().to_vec(),
				randomness: Babe::randomness(),
				allowed_slots: epoch_config.allowed_slots,
			}
		}

		fn current_epoch_start() -> sp_consensus_babe::Slot {
			Babe::current_epoch_start()
		}

		fn current_epoch() -> sp_consensus_babe::Epoch {
			Babe::current_epoch()
		}

		fn next_epoch() -> sp_consensus_babe::Epoch {
			Babe::next_epoch()
		}

		fn generate_key_ownership_proof(
			_slot: sp_consensus_babe::Slot,
			authority_id: sp_consensus_babe::AuthorityId,
		) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
			use parity_scale_codec::Encode;

			Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
			key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Babe::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
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
			header: &<Block as BlockT>::Header,
		) -> Result<
			(),
			sp_runtime::DispatchError,
		> {
			#[cfg(feature = "evm-tracing")]
			{
				use evm_tracer::tracer::EvmTracer;

				// Initialize block: calls the "on_initialize" hook on every pallet
				// in AllPalletsWithSystem.
				Executive::initialize_block(header);

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
			header: &<Block as BlockT>::Header,
		) -> Result<
			(),
			sp_runtime::DispatchError,
		> {
			#[cfg(feature = "evm-tracing")]
			{
				use evm_tracer::tracer::EvmTracer;

				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;

				// Initialize block: calls the "on_initialize" hook on every pallet
				// in AllPalletsWithSystem.
				Executive::initialize_block(header);

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

		fn trace_call(
			header: &<Block as BlockT>::Header,
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<(), sp_runtime::DispatchError> {
			#[cfg(feature = "evm-tracing")]
			{
				use evm_tracer::tracer::EvmTracer;

				// Initialize block: calls the "on_initialize" hook on every pallet
				// in AllPalletsWithSystem.
				Executive::initialize_block(header);

				EvmTracer::new().trace(|| {
					let is_transactional = false;
					let validate = true;
					let without_base_extrinsic_weight = true;


					// Estimated encoded transaction size must be based on the heaviest transaction
					// type (EIP1559Transaction) to be compatible with all transaction types.
					let mut estimated_transaction_len = data.len() +
					// pallet ethereum index: 1
					// transact call index: 1
					// Transaction enum variant: 1
					// chain_id 8 bytes
					// nonce: 32
					// max_priority_fee_per_gas: 32
					// max_fee_per_gas: 32
					// gas_limit: 32
					// action: 21 (enum varianrt + call address)
					// value: 32
					// access_list: 1 (empty vec size)
					// 65 bytes signature
					258;

					if access_list.is_some() {
						estimated_transaction_len += access_list.encoded_size();
					}

					let gas_limit = gas_limit.min(u64::MAX.into()).low_u64();

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

					let _ = <Runtime as pallet_evm::Config>::Runner::call(
						from,
						to,
						data,
						value,
						gas_limit,
						max_fee_per_gas,
						max_priority_fee_per_gas,
						nonce,
						access_list.unwrap_or_default(),
						is_transactional,
						validate,
						weight_limit,
						proof_size_base_cost,
						<Runtime as pallet_evm::Config>::config(),
					);
				});
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

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id, |_| None)
		}

		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			vec![]
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use pallet_tangle_lst_benchmarking::Pallet as LstBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmark!(list, extra, pallet_services, Services);
			list_benchmark!(list, extra, pallet_airdrop_claims, Claims);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch};
			use sp_storage::TrackedStorageKey;
			impl frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}
			use pallet_tangle_lst_benchmarking::Pallet as LstBench;

			use frame_support::traits::WhitelistedStorageKeys;
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmark!(params, batches, pallet_services, Services);
			add_benchmark!(params, batches, pallet_airdrop_claims, Claims);

			Ok(batches)
		}
	}
}
