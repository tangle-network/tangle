// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
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
use crate::{self as pallet_credits, types::StakeTier};

use frame_election_provider_support::{
	bounds::{ElectionBounds, ElectionBoundsBuilder},
	onchain, SequentialPhragmen,
};
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU128, ConstU32, OneSessionHandler},
	PalletId,
};
use pallet_session::historical as pallet_session_historical;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::{sr25519, ByteArray, ConstU64, RuntimeDebug, H160};
use sp_runtime::{
	testing::UintAuthorityId,
	traits::{ConvertInto, IdentityLookup},
	AccountId32, BuildStorage, DispatchError, Perbill, Percent,
};
use sp_staking::currency_to_vote::U128CurrencyToVote;
use sp_weights::Weight;
use std::cell::RefCell;
use tangle_primitives::{
	services::{Asset, EvmAddressMapping, EvmGasWeightMapping},
	traits::RewardsManager,
	types::{rewards::LockMultiplier, BlockNumber},
};

pub type AccountId = AccountId32;
pub type Balance = u128;
pub type Nonce = u32;
pub type AssetId = u128;

// Define constants for common accounts
pub const ALICE: AccountId = AccountId32::new([1u8; 32]);
pub const BOB: AccountId = AccountId32::new([2u8; 32]);
pub const CHARLIE: AccountId = AccountId32::new([3u8; 32]);
pub const DAVE: AccountId = AccountId32::new([4u8; 32]);
pub const EVE: AccountId = AccountId32::new([5u8; 32]);

#[frame_support::derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = Nonce;
	type RuntimeCall = RuntimeCall;
	type Hash = sp_core::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = ();
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeHoldReason;
	type FreezeIdentifier = [u8; 8];
	type MaxFreezes = ConstU32<50>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 6000 / 2;
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
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
	type CurrencyToVote = U128CurrencyToVote;
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
	type DisablingStrategy = pallet_staking::UpToLimitDisablingStrategy;
}

parameter_types! {
	pub const SlashRecipient: AccountId = AccountId32::new([9u8; 32]);
}

impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u128;
	type AssetId = AssetId;
	type AssetIdParameter = u128;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type AssetDeposit = ConstU128<1>;
	type AssetAccountDeposit = ConstU128<10>;
	type MetadataDepositBase = ConstU128<1>;
	type MetadataDepositPerByte = ConstU128<1>;
	type ApprovalDeposit = ConstU128<1>;
	type StringLimit = ConstU32<50>;
	type Freezer = ();
	type WeightInfo = ();
	type CallbackHandle = ();
	type Extra = ();
	type RemoveItemsLimit = ConstU32<5>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
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

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxAssetsPerService: u32 = 64;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxRpcAddressLength: u32 = 256;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxResourceNameLength: u32 = 16;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const SlashDeferDuration: u32 = 7;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxMasterBlueprintServiceManagerRevisions: u32 = u32::MAX;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MinimumNativeSecurityRequirement: Percent = Percent::from_percent(10);
}

type Block = frame_system::mocking::MockBlock<Runtime>;

thread_local! {
	static DEPOSIT_CALLS: RefCell<Vec<(AccountId, Asset<AssetId>, Balance, Option<LockMultiplier>)>> = RefCell::new(Vec::new());
	static WITHDRAWAL_CALLS: RefCell<Vec<(AccountId, Asset<AssetId>, Balance)>> = RefCell::new(Vec::new());
}

pub struct MockRewardsManager;

impl RewardsManager<AccountId, AssetId, Balance, BlockNumber> for MockRewardsManager {
	type Error = DispatchError;

	fn record_deposit(
		account_id: &AccountId,
		asset: Asset<AssetId>,
		amount: Balance,
		lock_multiplier: Option<LockMultiplier>,
	) -> Result<(), Self::Error> {
		DEPOSIT_CALLS.with(|calls| {
			calls.borrow_mut().push((account_id.clone(), asset, amount, lock_multiplier));
		});
		Ok(())
	}

	fn record_withdrawal(
		account_id: &AccountId,
		asset: Asset<AssetId>,
		amount: Balance,
	) -> Result<(), Self::Error> {
		WITHDRAWAL_CALLS.with(|calls| {
			calls.borrow_mut().push((account_id.clone(), asset, amount));
		});
		Ok(())
	}

	fn record_service_reward(
		_account_id: &AccountId,
		_asset: Asset<AssetId>,
		_amount: Balance,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn get_asset_deposit_cap_remaining(_asset: Asset<AssetId>) -> Result<Balance, Self::Error> {
		Ok(100_000_u32.into())
	}

	fn get_asset_incentive_cap(_asset: Asset<AssetId>) -> Result<Balance, Self::Error> {
		Ok(0_u32.into())
	}
}

impl MockRewardsManager {
	pub fn record_deposit_calls(
	) -> Vec<(AccountId, Asset<AssetId>, Balance, Option<LockMultiplier>)> {
		DEPOSIT_CALLS.with(|calls| calls.borrow().clone())
	}

	pub fn record_withdrawal_calls() -> Vec<(AccountId, Asset<AssetId>, Balance)> {
		WITHDRAWAL_CALLS.with(|calls| calls.borrow().clone())
	}

	pub fn clear_all() {
		DEPOSIT_CALLS.with(|calls| calls.borrow_mut().clear());
		WITHDRAWAL_CALLS.with(|calls| calls.borrow_mut().clear());
	}
}

parameter_types! {
	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MinOperatorBondAmount: Balance = 1_000;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxDelegatorBlueprints: u32 = 10;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxOperatorBlueprints: u32 = 10;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxWithdrawRequests: u32 = 10;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxUnstakeRequests: u32 = 10;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxDelegations: u32 = 10;
	pub const PID: PalletId = PalletId(*b"tngl/mad");
}

pub struct MockServiceManager;

impl tangle_primitives::traits::ServiceManager<AccountId, Balance> for MockServiceManager {
	fn get_active_blueprints_count(_account: &AccountId) -> usize {
		// we don't care
		Default::default()
	}

	fn get_active_services_count(_account: &AccountId) -> usize {
		// we don't care
		Default::default()
	}

	fn can_exit(_account: &AccountId) -> bool {
		// Mock logic to determine if the given account can exit
		true
	}

	fn get_blueprints_by_operator(_account: &AccountId) -> Vec<u64> {
		unimplemented!(); // we don't care
	}

	fn has_active_services(_operator: &AccountId) -> bool {
		false
	}
}

pub struct MockGasWeightMapping;

impl EvmGasWeightMapping for MockGasWeightMapping {
	fn gas_to_weight(_gas: u64, _without_base_weight: bool) -> Weight {
		0.into()
	}

	fn weight_to_gas(_weight: Weight) -> u64 {
		0
	}
}

pub struct MockAddressMapping;

impl EvmAddressMapping<AccountId> for MockAddressMapping {
	fn into_account_id(address: H160) -> AccountId {
		AccountId::from_slice(&address[0..20]).expect("Invalid address")
	}

	fn into_address(account_id: AccountId) -> H160 {
		H160::from_slice(&account_id.as_slice()[0..20])
	}
}

impl pallet_multi_asset_delegation::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type SlashRecipient = SlashRecipient;
	type MinOperatorBondAmount = MinOperatorBondAmount;
	type CurrencyToVote = U128CurrencyToVote;
	type StakingInterface = Staking;
	type ServiceManager = MockServiceManager;
	type LeaveOperatorsDelay = ConstU32<10>;
	type OperatorBondLessDelay = ConstU32<1>;
	type LeaveDelegatorsDelay = ConstU32<1>;
	type DelegationBondLessDelay = ConstU32<5>;
	type MinDelegateAmount = ConstU128<1>;
	type Fungibles = Assets;
	type AssetId = AssetId;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type PalletId = PID;
	type MaxDelegatorBlueprints = MaxDelegatorBlueprints;
	type MaxOperatorBlueprints = MaxOperatorBlueprints;
	type MaxWithdrawRequests = MaxWithdrawRequests;
	type MaxUnstakeRequests = MaxUnstakeRequests;
	type MaxDelegations = MaxDelegations;
	type EvmRunner = ();
	type EvmGasWeightMapping = MockGasWeightMapping;
	type EvmAddressMapping = MockAddressMapping;
	type RewardsManager = MockRewardsManager;
	type WeightInfo = ();
}

parameter_types! {
	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxStakeTiers: u32 = 10;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const CreditBurnRecipient: Option<AccountId> = None;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxRatePerBlock: Balance = tangle_primitives::credits::MAX_RATE_PER_BLOCK;
}

impl pallet_credits::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type AssetId = AssetId;
	type MultiAssetDelegationInfo = MultiAssetDelegation;
	type BurnConversionRate = ConstU128<1000>;
	type ClaimWindowBlocks = ConstU64<1000>;
	type CreditBurnRecipient = CreditBurnRecipient;
	type MaxOffchainAccountIdLength = ConstU32<100>;
	type MaxStakeTiers = MaxStakeTiers;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxRatePerBlock = MaxRatePerBlock;
	type WeightInfo = ();
}

construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Assets: pallet_assets,
		Session: pallet_session,
		Staking: pallet_staking,
		Historical: pallet_session_historical,
		MultiAssetDelegation: pallet_multi_asset_delegation,
		Credits: pallet_credits,
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

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext_raw_authorities(_authorities: Vec<AccountId>) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

	// Add GenesisConfig for Credits pallet
	pallet_credits::GenesisConfig::<Runtime> {
		stake_tiers: vec![
			StakeTier { threshold: 100_u128, rate_per_block: 1_u128 },
			StakeTier { threshold: 1000_u128, rate_per_block: 5_u128 },
			StakeTier { threshold: 10_000_u128, rate_per_block: 15_u128 },
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
