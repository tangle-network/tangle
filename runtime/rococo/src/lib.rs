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

pub mod impls;
pub mod protocol_substrate_config;
pub mod weights;
pub mod xcm_config;

use codec::Encode;
use dkg_runtime_primitives::{TypedChainId, UnsignedProposal};
use pallet_dkg_proposals::DKGEcdsaToEthereum;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{self, BlakeTwo256, Block as BlockT, StaticLookup},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, SaturatedConversion,
};

use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use frame_support::weights::ConstantMultiplier;
use pallet_session::historical as pallet_session_historical;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_linkable_tree::types::EdgeMetadata;
use pallet_transaction_payment::{CurrencyAdapter, Multiplier, TargetedFeeAdjustment};
use sp_runtime::{FixedPointNumber, Perquintill};
use webb_primitives::{
	linkable_tree::LinkableTreeInspector, runtime::Element, AccountIndex, ChainId, LeafIndex,
};

// A few exports that help ease life for downstream crates.
pub use dkg_runtime_primitives::crypto::AuthorityId as DKGId;
pub use frame_support::{
	construct_runtime,
	dispatch::DispatchClass,
	match_types, parameter_types,
	traits::{
		ConstU128, ConstU32, Currency, EitherOfDiverse, EqualPrivilegeOnly, Everything, IsInVec,
		Randomness,
	},
	weights::{constants::WEIGHT_PER_SECOND, IdentityFee, Weight},
	PalletId, StorageValue,
};
#[cfg(any(feature = "std", test))]
pub use frame_system::Call as SystemCall;
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot,
};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_runtime::generic::Era;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{MultiAddress, Perbill, Percent, Permill};
pub use tangle_primitives::{
	currency::*, fee::*, time::*, AccountId, Address, Balance, BlockNumber, Hash, Header, Index,
	Moment, Reputation, Signature, AVERAGE_ON_INITIALIZE_RATIO, MAXIMUM_BLOCK_WEIGHT,
	NORMAL_DISPATCH_RATIO, SESSION_PERIOD_BLOCKS,
};
use weights::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight};

pub mod nimbus_session_adapter;
pub mod staking;
use nimbus_session_adapter::NimbusId;
// XCM Imports

use xcm::latest::prelude::*;

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
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;
/// Signed payload
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
		Weight::from_ref_time(0u64)
	}
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;
	use sp_runtime::{generic, traits::BlakeTwo256};

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
}

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("tangle-parachain"),
	impl_name: create_runtime_str!("tangle-parachain"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 0,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const Version: RuntimeVersion = VERSION;
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u8 = 42;
}

use nimbus_session_adapter::{AuthorInherentWithNoOpSession, VrfWithNoOpSession};
impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
		pub dkg: DKG,
		pub nimbus: AuthorInherentWithNoOpSession<Runtime>,
		pub vrf: VrfWithNoOpSession,
	}
}

impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = Everything;
	type BlockHashCount = BlockHashCount;
	type BlockLength = RuntimeBlockLength;
	type BlockNumber = BlockNumber;
	type BlockWeights = RuntimeBlockWeights;
	type RuntimeCall = RuntimeCall;
	type DbWeight = RocksDbWeight;
	type RuntimeEvent = RuntimeEvent;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	type Index = Index;
	type Lookup = Indices;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
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
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"eg/trsry");
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 100;
	pub const MaxApprovals: u32 = 100;
	pub const SpendPeriod: BlockNumber = 100;
}

impl pallet_treasury::Config for Runtime {
	type Currency = Balances;
	type ApproveOrigin = frame_system::EnsureRoot<AccountId>;
	type RejectOrigin = frame_system::EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<u128>;
	type ProposalBondMaximum = ();
	type SpendPeriod = SpendPeriod;
	type Burn = ();
	type BurnDestination = ();
	type PalletId = TreasuryPalletId;
	type SpendFunds = ();
	type MaxApprovals = MaxApprovals;
	type WeightInfo = ();
}

parameter_types! {
	pub const TransactionByteFee: Balance = 10 * MILLIUNIT;
	pub const OperationalFeeMultiplier: u8 = 5;
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = CurrencyAdapter<Balances, crate::impls::DealWithFees<Runtime>>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate =
		TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl pallet_sudo::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
}

parameter_types! {
	pub const MaxAuthorities: u32 = 1_000;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
	pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
	type EventHandler = ();
	type FilterUncle = ();
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = UncleGenerations;
}

parameter_types! {
	pub const Period: u32 = SESSION_PERIOD_BLOCKS;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Keys = SessionKeys;
	type NextSessionRotation = pallet_dkg_metadata::DKGPeriodicSessions<Period, Offset, Runtime>;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type SessionManager = ParachainStaking;
	type ShouldEndSession = pallet_dkg_metadata::DKGPeriodicSessions<Period, Offset, Runtime>;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = IdentityCollator;
	type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

// impl pallet_session::historical::Config for Runtime {
// 	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
// 	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
// }

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const MaxCandidates: u32 = 1000;
	pub const MinCandidates: u32 = 5;
	pub const SessionLength: BlockNumber = 6 * HOURS;
	pub const MaxInvulnerables: u32 = 100;
	pub const ExecutiveBody: BodyId = BodyId::Executive;
}

parameter_types! {
	pub const DecayPercentage: Percent = Percent::from_percent(50);
	pub const UnsignedPriority: u64 = 1 << 20;
	pub const UnsignedInterval: BlockNumber = 3;
}

impl pallet_dkg_metadata::Config for Runtime {
	type DKGId = DKGId;
	type RuntimeEvent = RuntimeEvent;
	type OnAuthoritySetChangeHandler = DKGProposals;
	type OnDKGPublicKeyChangeHandler = ();
	type OffChainAuthId = dkg_runtime_primitives::offchain::crypto::OffchainAuthId;
	type NextSessionRotation = pallet_dkg_metadata::DKGPeriodicSessions<Period, Offset, Runtime>;
	type RefreshDelay = RefreshDelay;
	type KeygenJailSentence = Period;
	type SigningJailSentence = Period;
	type DecayPercentage = DecayPercentage;
	type Reputation = Reputation;
	type UnsignedPriority = UnsignedPriority;
	type UnsignedInterval = UnsignedInterval;
	type AuthorityIdOf = pallet_dkg_metadata::AuthorityIdOf<Self>;
	type ProposalHandler = DKGProposalHandler;
	type WeightInfo = pallet_dkg_metadata::weights::WebbWeight<Runtime>;
}

parameter_types! {
	pub const ChainIdentifier: TypedChainId = TypedChainId::RococoParachain(5);
	pub const ProposalLifetime: BlockNumber = HOURS / 5;
	pub const DKGAccountId: PalletId = PalletId(*b"dw/dkgac");
	pub const RefreshDelay: Permill = Permill::from_percent(90);
	pub const TimeToRestart: BlockNumber = 3;
	 // 1 hr considering block time of 12sec
	 pub const UnsignedProposalExpiry : BlockNumber = 300;
}

impl pallet_dkg_proposal_handler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OffChainAuthId = dkg_runtime_primitives::offchain::crypto::OffchainAuthId;
	type MaxSubmissionsPerBatch = frame_support::traits::ConstU16<100>;
	type UnsignedProposalExpiry = UnsignedProposalExpiry;
	type SignedProposalHandler = ();
	type WeightInfo = pallet_dkg_proposal_handler::weights::WebbWeight<Runtime>;
}

impl pallet_dkg_proposals::Config for Runtime {
	type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type DKGAuthorityToMerkleLeaf = DKGEcdsaToEthereum;
	type DKGId = DKGId;
	type ChainIdentifier = ChainIdentifier;
	type RuntimeEvent = RuntimeEvent;
	type NextSessionRotation = pallet_dkg_metadata::DKGPeriodicSessions<Period, Offset, Runtime>;
	type Proposal = Vec<u8>;
	type ProposalLifetime = ProposalLifetime;
	type ProposalHandler = DKGProposalHandler;
	type Period = Period;
	type WeightInfo = pallet_dkg_proposals::WebbWeight<Runtime>;
}

parameter_types! {
	pub const BasicDeposit: Balance = deposit(1, 258);
	pub const FieldDeposit: Balance = deposit(0, 66);
	pub const SubAccountDeposit: Balance = deposit(1, 53);
	pub const MaxSubAccounts: u32 = 100;
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
	type Slashed = Treasury;
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
	pub Prefix: &'static [u8] = b"Pay TNTs to the Tangle account:";
}

impl pallet_ecdsa_claims::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type VestingSchedule = Vesting;
	type Prefix = Prefix;
	type ForceOrigin = EnsureRoot<Self::AccountId>;
	type MoveClaimOrigin = EnsureRoot<Self::AccountId>;
	type WeightInfo = pallet_ecdsa_claims::TestWeightInfo;
}

parameter_types! {
	pub const MinVestedTransfer: Balance = DOLLAR;
}

impl pallet_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockNumberToBalance = sp_runtime::traits::ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = ();
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		public: <Signature as traits::Verify>::Signer,
		account: AccountId,
		nonce: Index,
	) -> Option<(RuntimeCall, <UncheckedExtrinsic as traits::Extrinsic>::SignaturePayload)> {
		let tip = 0;
		// take the biggest period possible.
		let period =
			BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;
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

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	pub const VotingPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * 24 * 60 * MINUTES;
	pub const InstantAllowed: bool = true;
	pub const MinimumDeposit: Balance = 100 * UNIT;
	pub const EnactmentPeriod: BlockNumber = 30 * 24 * 60 * MINUTES;
	pub const CooloffPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	// One cent: $10,000 / MB
	pub const PreimageByteDeposit: Balance = CENT;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// To cancel a proposal before it has been passed, the technical committee must
	// be unanimous or Root must agree.
	type CancelProposalOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>,
	>;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to
	// it.
	type CancellationOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
	type CooloffPeriod = CooloffPeriod;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type RuntimeEvent = RuntimeEvent;
	/// A unanimous council can have the next scheduled referendum be a straight
	/// default-carries (NTB) vote.
	type ExternalDefaultOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
	/// A super-majority can have the next scheduled referendum be a straight
	/// majority-carries vote.
	type ExternalMajorityOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
	/// Two thirds of the technical committee can have an
	/// ExternalMajority/ExternalDefault vote be tabled immediately and with a
	/// shorter voting/enactment period.
	type FastTrackOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	type InstantAllowed = InstantAllowed;
	type InstantOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
	type LaunchPeriod = LaunchPeriod;
	type MaxProposals = MaxProposals;
	type MaxVotes = MaxVotes;
	// Same as EnactmentPeriod
	type MinimumDeposit = MinimumDeposit;
	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
	type PalletsOrigin = OriginCaller;
	type PreimageByteDeposit = PreimageByteDeposit;
	type Proposal = RuntimeCall;
	type Scheduler = Scheduler;
	type Slash = Treasury;
	// Any single technical committee member may veto a coming council proposal,
	// however they can only do it once and it lasts only for the cool-off period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
	type VoteLockingPeriod = EnactmentPeriod;
	type VotingPeriod = VotingPeriod;
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type RuntimeEvent = RuntimeEvent;
	type MaxMembers = CouncilMaxMembers;
	type MaxProposals = CouncilMaxProposals;
	type MotionDuration = CouncilMotionDuration;
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

impl pallet_aura_style_filter::Config for Runtime {
	/// Nimbus filter pipeline (final) step 3:
	/// Choose 1 collator from PotentialAuthors as eligible
	/// for each slot in round-robin fashion
	type PotentialAuthors = ParachainStaking;
}

parameter_types! {
	pub LeaveDelayRounds: BlockNumber = SESSION_PERIOD_BLOCKS;
}

/// A convertor from collators id. Since this pallet does not have stash/controller, this is
/// just identity.
pub struct IdentityCollator;
impl<T> sp_runtime::traits::Convert<T, Option<T>> for IdentityCollator {
	fn convert(t: T) -> Option<T> {
		Some(t)
	}
}
impl<T> sp_runtime::traits::Convert<T, T> for IdentityCollator {
	fn convert(t: T) -> T {
		t
	}
}

impl pallet_parachain_staking::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockAuthor = AuthorInherent;
	type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
	/// Minimum round length is 2 minutes (10 * 12 second block times)
	type MinBlocksPerRound = ConstU32<10>;
	/// Rounds before the collator leaving the candidates request can be executed
	type LeaveCandidatesDelay = LeaveDelayRounds;
	/// Rounds before the candidate bond increase/decrease can be executed
	type CandidateBondLessDelay = LeaveDelayRounds;
	/// Rounds before the delegator exit can be executed
	type LeaveDelegatorsDelay = LeaveDelayRounds;
	/// Rounds before the delegator revocation can be executed
	type RevokeDelegationDelay = LeaveDelayRounds;
	/// Rounds before the delegator bond increase/decrease can be executed
	type DelegationBondLessDelay = LeaveDelayRounds;
	/// Rounds before the reward is paid
	type RewardPaymentDelay = ConstU32<2>;
	/// Minimum collators selected per round, default at genesis and minimum forever after
	type MinSelectedCandidates = ConstU32<5>;
	/// Maximum top delegations per candidate
	type MaxTopDelegationsPerCandidate = ConstU32<100>;
	/// Maximum bottom delegations per candidate
	type MaxBottomDelegationsPerCandidate = ConstU32<50>;
	/// Maximum delegations per delegator
	type MaxDelegationsPerDelegator = ConstU32<25>;
	/// Minimum stake on a collator to be considered for block production
	type MinCollatorStk = ConstU128<{ crate::staking::MIN_BOND_TO_BE_CONSIDERED_COLLATOR }>;
	/// Minimum stake the collator runner must bond to register as collator candidate
	type MinCandidateStk = ConstU128<{ crate::staking::NORMAL_COLLATOR_MINIMUM_STAKE }>;
	/// Smallest amount that can be delegated
	type MinDelegation = ConstU128<{ 5 * DOLLAR }>;
	/// Minimum stake required to be reserved to be a delegator
	type MinDelegatorStk = ConstU128<{ 5 * DOLLAR }>;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = IdentityCollator;
	type AccountIdOf = IdentityCollator;
	type MaxInvulnerables = ConstU32<10>;
	type ValidatorRegistration = Session;
	type UpdateOrigin = EnsureRoot<AccountId>;
	type OnCollatorPayout = ();
	type OnNewRound = ();
	type WeightInfo = ();
}

impl pallet_author_inherent::Config for Runtime {
	// We start a new slot each time we see a new relay block.
	type SlotBeacon = cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Self>;
	type AccountLookup = ParachainStaking;
	type WeightInfo = ();
	/// Nimbus filter pipeline step 1:
	/// Filters out NimbusIds not registered as SessionKeys of some AccountId
	type CanAuthor = AuraAuthorFilter;
}

parameter_types! {
	pub const PreimageMaxSize: u32 = 4096 * 1024;
	pub const PreimageBaseDeposit: Balance = UNIT;
}

impl pallet_preimage::Config for Runtime {
	type BaseDeposit = PreimageBaseDeposit;
	type ByteDeposit = PreimageByteDeposit;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type MaxSize = PreimageMaxSize;
	type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		RuntimeBlockWeights::get().max_block;
	// Retry a scheduled item every 10 blocks (1 minute) until the preimage exists.
	pub const NoPreimagePostponement: Option<u32> = Some(10);
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MaxScheduledPerBlock = ConstU32<50>;
	type MaximumWeight = MaximumSchedulerWeight;
	type NoPreimagePostponement = NoPreimagePostponement;
	type RuntimeOrigin = RuntimeOrigin;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type PalletsOrigin = OriginCaller;
	type PreimageProvider = Preimage;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as sp_runtime::traits::Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	RuntimeCall: From<C>,
{
	type OverarchingCall = RuntimeCall;
	type Extrinsic = UncheckedExtrinsic;
}

impl pallet_transaction_pause::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type UpdateOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
}

// parameter_types! {
// 	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
// 	pub const MaxKeys: u32 = 10_000;
// 	pub const MaxPeerInHeartbeats: u32 = 10_000;
// 	pub const MaxPeerDataEncodingSize: u32 = 1_000;
// }

// impl pallet_im_online::Config for Runtime {
// 	type AuthorityId = ImOnlineId;
// 	type RuntimeEvent = RuntimeEvent;
// 	type NextSessionRotation = pallet_dkg_metadata::DKGPeriodicSessions<Period, Offset, Runtime>;
// 	type ValidatorSet = Historical;
// 	type ReportUnresponsiveness = ();
// 	type UnsignedPriority = ImOnlineUnsignedPriority;
// 	type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
// 	type MaxKeys = MaxKeys;
// 	type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
// 	type MaxPeerDataEncodingSize = MaxPeerDataEncodingSize;
// }

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		// System support stuff
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned} = 1,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 2,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 3,

		// DKG / offchain worker - the order and position of these pallet should not change
		DKG: pallet_dkg_metadata::{Pallet, Storage, Call, Event<T>, Config<T>} = 10,
		DKGProposals: pallet_dkg_proposals = 11,
		DKGProposalHandler: pallet_dkg_proposal_handler = 12,

		// Monetary stuff
		Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>} = 20,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 21,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 22,
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 23,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 24,
		Indices: pallet_indices::{Pallet, Call, Storage, Config<T>, Event<T>} = 25,
		// Claims. Usable initially.
		Claims: pallet_ecdsa_claims::{Pallet, Call, Storage, Event<T>, Config<T>, ValidateUnsigned} = 26,

		ParachainStaking: pallet_parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>} = 27,
		// Collator support.
		AuthorInherent: pallet_author_inherent::{Pallet, Call, Storage, Inherent} = 28,
		AuraAuthorFilter: pallet_aura_style_filter::{Pallet, Storage} = 29,
		// Collator support. the order of these 4 are important and shall not change.
		Authorship: pallet_authorship::{Pallet, Call, Storage} = 30,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 32,
		Aura: pallet_aura::{Pallet, Storage, Config<T>} = 33,
		//AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config} = 34,
		//Historical: pallet_session_historical::{Pallet} = 35,

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 40,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin} = 41,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 42,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 43,

		// Asset helpers
		AssetRegistry: pallet_asset_registry::{Pallet, Call, Storage, Event<T>, Config<T>} = 50,
		Currencies: orml_currencies::{Pallet, Call} = 51,
		Tokens: orml_tokens::{Pallet, Storage, Call, Event<T>} = 52,
		TokenWrapper: pallet_token_wrapper::{Pallet, Storage, Call, Event<T>} = 53,

		// Privacy pallets
		HasherBn254: pallet_hasher::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 60,
		MixerVerifierBn254: pallet_verifier::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 61,
		MerkleTreeBn254: pallet_mt::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 63,
		LinkableTreeBn254: pallet_linkable_tree::<Instance1>::{Pallet, Call, Storage, Event<T>} = 64,
		MixerBn254: pallet_mixer::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 65,
		VAnchorBn254: pallet_vanchor::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 66,
		VAnchorHandlerBn254: pallet_vanchor_handler::<Instance1>::{Pallet, Call, Storage, Event<T>} = 67,
		KeyStorage: pallet_key_storage::<Instance1>::{Pallet, Call, Storage, Event<T>} = 68,
		VAnchorVerifier: pallet_vanchor_verifier::{Pallet, Call, Storage, Event<T>, Config<T>} = 69,

		// Bridge
		SignatureBridge: pallet_signature_bridge::<Instance1>::{Pallet, Call, Storage, Event<T>} = 70,
		TokenWrapperHandler: pallet_token_wrapper_handler::{Pallet, Storage, Call, Event<T>} = 71,

		// Substrate utility pallets
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 80,
		Utility: pallet_utility::{Pallet, Call, Event} = 81,
		Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>, Config<T>} = 82,
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>} = 83,
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 84,
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 85,
		Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 86,
		TransactionPause: pallet_transaction_pause::{Pallet, Call, Storage, Event<T>} = 87,
		//ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, Config<T>, ValidateUnsigned} = 88,
	}
);

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

	impl dkg_runtime_primitives::DKGApi<Block, dkg_runtime_primitives::crypto::AuthorityId, BlockNumber> for Runtime {
		fn authority_set() -> dkg_runtime_primitives::AuthoritySet<dkg_runtime_primitives::crypto::AuthorityId> {
			let authorities = DKG::authorities();
			let authority_set_id = DKG::authority_set_id();

			dkg_runtime_primitives::AuthoritySet {
				authorities,
				id: authority_set_id
			}
		}

		fn queued_authority_set() -> dkg_runtime_primitives::AuthoritySet<dkg_runtime_primitives::crypto::AuthorityId> {
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
			DKG::next_dkg_public_key()
		}

		fn next_pub_key_sig() -> Option<Vec<u8>> {
			DKG::next_public_key_signature()
		}

		fn dkg_pub_key() -> (dkg_runtime_primitives::AuthoritySetId, Vec<u8>) {
			DKG::dkg_public_key()
		}

		fn get_best_authorities() -> Vec<(u16, DKGId)> {
			DKG::best_authorities()
		}

		fn get_next_best_authorities() -> Vec<(u16, DKGId)> {
			DKG::next_best_authorities()
		}

		fn get_current_session_progress(block_number: BlockNumber) -> Option<Permill> {
			use frame_support::traits::EstimateNextSessionRotation;
			<pallet_dkg_metadata::DKGPeriodicSessions<Period, Offset, Runtime> as EstimateNextSessionRotation<BlockNumber>>::estimate_current_session_progress(block_number).0
		}

		fn get_unsigned_proposals() -> Vec<UnsignedProposal> {
			DKGProposalHandler::get_unsigned_proposals()
		}

		fn get_max_extrinsic_delay(block_number: BlockNumber) -> BlockNumber {
			DKG::max_extrinsic_delay(block_number)
		}

		fn get_authority_accounts() -> (Vec<AccountId>, Vec<AccountId>) {
			(DKG::current_authorities_accounts(), DKG::next_authorities_accounts())
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

		fn should_execute_emergency_keygen() -> bool {
			DKG::should_execute_emergency_keygen()
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
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
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

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl pallet_linkable_tree_rpc_runtime_api::LinkableTreeApi<Block, ChainId, Element, LeafIndex> for Runtime {
		fn get_neighbor_roots(tree_id: u32) -> Vec<Element> {
			LinkableTreeBn254::get_neighbor_roots(tree_id).ok().unwrap_or_default()
		}

		fn get_neighbor_edges(tree_id: u32) -> Vec<EdgeMetadata<ChainId, Element, LeafIndex>> {
			LinkableTreeBn254::get_neighbor_edges(tree_id).ok().unwrap_or_default()
		}
	}

	impl pallet_mt_rpc_runtime_api::MerkleTreeApi<Block, Element> for Runtime {
		fn get_leaf(tree_id: u32, index: u32) -> Option<Element> {
			let v = MerkleTreeBn254::leaves(tree_id, index);
			if v == Element::default() {
				None
			} else {
				Some(v)
			}
		}
	}

	impl nimbus_primitives::NimbusApi<Block> for Runtime {
        fn can_author(author: NimbusId, relay_parent: u32, parent_header: &<Block as BlockT>::Header) -> bool {
			use pallet_session::ShouldEndSession;
            let next_block_number = parent_header.number + 1;
            let slot = relay_parent;
            // Because the staking solution calculates the next staking set at the beginning
            // of the first block in the new round, the only way to accurately predict the
            // authors is to compute the selection during prediction.
            // NOTE: This logic must manually be kept in sync with the nimbus filter pipeline
            if pallet_dkg_metadata::DKGPeriodicSessions::<Period, Offset, Runtime>::should_end_session(next_block_number)
            {
                // lookup account from nimbusId
                // mirrors logic in `pallet_author_inherent`
                use nimbus_primitives::AccountLookup;
                let account = match pallet_parachain_staking::Pallet::<Self>::lookup_account(&author) {
                    Some(account) => account,
                    // Authors whose account lookups fail will not be eligible
                    None => {
                        return false;
                    }
                };
                // manually check aura eligibility (in the new round)
                // mirrors logic in `aura_style_filter`
                let truncated_half_slot = (slot >> 1) as usize;
                let active: Vec<AccountId> = pallet_parachain_staking::Pallet::<Self>::compute_top_candidates();
                account == active[truncated_half_slot % active.len()]
            } else {
                // We're not changing rounds, `PotentialAuthors` is not changing, just use can_author
                <AuthorInherent as nimbus_primitives::CanAuthor<_>>::can_author(&author, &relay_parent)
            }
        }
    }

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
			use orml_benchmarking::list_benchmark as list_orml_benchmark;
			use frame_support::traits::StorageInfoTrait;

			use frame_system_benchmarking::Pallet as SystemBench;

			let mut list = Vec::<BenchmarkList>::new();

			list_benchmark!(list, extra, pallet_balances, Balances);
			list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
			list_benchmark!(list, extra, pallet_timestamp, Timestamp);
			list_benchmark!(list, extra, pallet_dkg_proposal_handler, DKGProposalHandler);
			list_benchmark!(list, extra, pallet_signature_bridge, SignatureBridge);
			list_benchmark!(list, extra, pallet_hasher, HasherBn254);
			list_benchmark!(list, extra, pallet_mt, MerkleTreeBn254);
			list_benchmark!(list, extra, pallet_asset_registry, AssetRegistry);
			list_benchmark!(list, extra, pallet_mixer, MixerBn254);
			list_orml_benchmark!(list, extra, orml_tokens, benchmarking::orml_tokens);
			list_orml_benchmark!(list, extra, orml_currencies, benchmarking::orml_currencies);

			let storage_info = AllPalletsWithSystem::storage_info();

			return (list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};
			use orml_benchmarking::{add_benchmark as add_orml_benchmark};
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
			add_benchmark!(params, batches, pallet_signature_bridge, SignatureBridge);
			add_benchmark!(params, batches, pallet_hasher, HasherBn254);
			add_benchmark!(params, batches, pallet_mt, MerkleTreeBn254);
			add_benchmark!(params, batches, pallet_asset_registry, AssetRegistry);
			add_benchmark!(params, batches, pallet_mixer, MixerBn254);
			add_orml_benchmark!(params, batches, orml_tokens, benchmarking::orml_tokens);
			add_orml_benchmark!(params, batches, orml_currencies, benchmarking::orml_currencies);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data =
			cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
				relay_chain_slot,
				sp_std::time::Duration::from_secs(6),
			)
			.create_inherent_data()
			.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = pallet_author_inherent::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
