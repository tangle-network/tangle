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
use crate::mock_evm::MockedEvmRunner;
use crate::{self as pallet_services};
use core::ops::Mul;
use ethabi::Uint;
use frame_election_provider_support::{
	bounds::{ElectionBounds, ElectionBoundsBuilder},
	onchain, SequentialPhragmen,
};
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU128, ConstU32, Hooks, OneSessionHandler},
	PalletId,
};
use frame_system::EnsureRoot;
use pallet_evm::GasWeightMapping;
use pallet_session::historical as pallet_session_historical;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde_json::json;
use sp_core::{sr25519, RuntimeDebug, H160};
use sp_keystore::{testing::MemoryKeystore, KeystoreExt, KeystorePtr};
use sp_runtime::{
	testing::UintAuthorityId,
	traits::{ConvertInto, IdentityLookup},
	AccountId32, BuildStorage, DispatchError, Perbill, Percent,
};
use sp_staking::currency_to_vote::U128CurrencyToVote;
use sp_weights::Weight;
use std::{cell::RefCell, collections::BTreeMap, sync::Arc};
use tangle_primitives::{
	services::{Asset, EvmAddressMapping, EvmGasWeightMapping, EvmRunner},
	traits::{MultiAssetDelegationInfo, RewardsManager, ServiceManager},
	types::{rewards::LockMultiplier, BlockNumber, RoundIndex},
};

pub type AccountId = AccountId32;
pub type Balance = u128;
pub type Nonce = u32;
pub type AssetId = u128;

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
	pub const ServicePalletAccountId: PalletId = PalletId(*b"Services");
	pub const SlashRecipient: AccountId = AccountId32::new([9u8; 32]);
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

pub struct PalletEVMAddressMapping;

impl EvmAddressMapping<AccountId> for PalletEVMAddressMapping {
	fn into_account_id(address: H160) -> AccountId {
		use pallet_evm::AddressMapping;
		<Runtime as pallet_evm::Config>::AddressMapping::into_account_id(address)
	}

	fn into_address(account_id: AccountId) -> H160 {
		H160::from_slice(&AsRef::<[u8; 32]>::as_ref(&account_id)[0..20])
	}
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
	pub const SlashDeferDuration: u32 = 7;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxMasterBlueprintServiceManagerRevisions: u32 = u32::MAX;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const NativeExposureMinimum: Percent = Percent::from_percent(10);
}

impl pallet_services::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Currency = Balances;
	type Fungibles = Assets;
	type PalletId = ServicePalletAccountId;
	type SlashRecipient = SlashRecipient;
	type SlashManager = ();
	type AssetId = AssetId;
	type EvmRunner = MockedEvmRunner;
	type EvmGasWeightMapping = PalletEVMGasWeightMapping;
	type EvmAddressMapping = PalletEVMAddressMapping;
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
	type MaxAssetsPerService = MaxAssetsPerService;
	type MaxMasterBlueprintServiceManagerVersions = MaxMasterBlueprintServiceManagerRevisions;
	type NativeExposureMinimum = NativeExposureMinimum;
	type Constraints = pallet_services::types::ConstraintsOf<Self>;
	type OperatorDelegationManager = MultiAssetDelegation;
	type SlashDeferDuration = SlashDeferDuration;
	type MasterBlueprintServiceManagerUpdateOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
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
	pub const BondDuration: u32 = 28;

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

impl pallet_multi_asset_delegation::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MinOperatorBondAmount = MinOperatorBondAmount;
	type BondDuration = BondDuration;
	type CurrencyToVote = U128CurrencyToVote;
	type StakingInterface = Staking;
	type ServiceManager = Services;
	type LeaveOperatorsDelay = ConstU32<10>;
	type OperatorBondLessDelay = ConstU32<1>;
	type LeaveDelegatorsDelay = ConstU32<1>;
	type DelegationBondLessDelay = ConstU32<5>;
	type MinDelegateAmount = ConstU128<100>;
	type Fungibles = Assets;
	type AssetId = AssetId;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type PalletId = PID;
	type MaxDelegatorBlueprints = MaxDelegatorBlueprints;
	type MaxOperatorBlueprints = MaxOperatorBlueprints;
	type MaxWithdrawRequests = MaxWithdrawRequests;
	type MaxUnstakeRequests = MaxUnstakeRequests;
	type MaxDelegations = MaxDelegations;
	type EvmRunner = MockedEvmRunner;
	type EvmGasWeightMapping = PalletEVMGasWeightMapping;
	type EvmAddressMapping = PalletEVMAddressMapping;
	type RewardsManager = MockRewardsManager;
	type WeightInfo = ();
}

construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Assets: pallet_assets,
		Services: pallet_services,
		EVM: pallet_evm,
		Ethereum: pallet_ethereum,
		Session: pallet_session,
		Staking: pallet_staking,
		Historical: pallet_session_historical,
		MultiAssetDelegation: pallet_multi_asset_delegation,
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

pub fn mock_address(id: u8) -> H160 {
	H160::from_slice(&[id; 20])
}

pub fn account_id_to_address(account_id: AccountId) -> H160 {
	H160::from_slice(&AsRef::<[u8; 32]>::as_ref(&account_id)[0..20])
}

pub fn address_to_account_id(address: H160) -> AccountId {
	use pallet_evm::AddressMapping;
	<Runtime as pallet_evm::Config>::AddressMapping::into_account_id(address)
}

pub fn mock_authorities(vec: Vec<u8>) -> Vec<AccountId> {
	vec.into_iter().map(|id| mock_pub_key(id)).collect()
}

pub fn new_test_ext(ids: Vec<u8>) -> sp_io::TestExternalities {
	new_test_ext_raw_authorities(mock_authorities(ids))
}

pub const MBSM: H160 = H160([0x12; 20]);
pub const CGGMP21_BLUEPRINT: H160 = H160([0x21; 20]);
pub const HOOKS_TEST: H160 = H160([0x22; 20]);
pub const USDC_ERC20: H160 = H160(hex_literal::hex!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"));

pub const TNT: AssetId = 0;
pub const USDC: AssetId = 1;
pub const WETH: AssetId = 2;
pub const WBTC: AssetId = 3;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext_raw_authorities(authorities: Vec<AccountId>) -> sp_io::TestExternalities {
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

	let mut create_contract = |bytecode: &str, address: H160| {
		let mut raw_hex = bytecode.replace("0x", "").replace("\n", "");
		// fix odd length
		if raw_hex.len() % 2 != 0 {
			raw_hex = format!("0{}", raw_hex);
		}
		let code = hex::decode(raw_hex).unwrap();
		evm_accounts.insert(
			address,
			fp_evm::GenesisAccount {
				code,
				storage: Default::default(),
				nonce: Default::default(),
				balance: Default::default(),
			},
		);
	};

	create_contract(include_str!("./test-artifacts/CGGMP21Blueprint.hex"), CGGMP21_BLUEPRINT);
	create_contract(include_str!("./test-artifacts/MasterBlueprintServiceManager.hex"), MBSM);
	create_contract(
		include_str!("./test-artifacts/HookTestBlueprintServiceManager.hex"),
		HOOKS_TEST,
	);
	create_contract(include_str!("./test-artifacts/MockERC20.hex"), USDC_ERC20);

	for i in 1..=authorities.len() {
		evm_accounts.insert(
			mock_address(i as u8),
			fp_evm::GenesisAccount {
				code: vec![],
				storage: Default::default(),
				nonce: Default::default(),
				balance: Uint::from(1_000).mul(Uint::from(10).pow(Uint::from(18))),
			},
		);
	}

	for a in &authorities {
		evm_accounts.insert(
			account_id_to_address(a.clone()),
			fp_evm::GenesisAccount {
				code: vec![],
				storage: Default::default(),
				nonce: Default::default(),
				balance: Uint::from(1_000).mul(Uint::from(10).pow(Uint::from(18))),
			},
		);
	}

	let evm_config =
		pallet_evm::GenesisConfig::<Runtime> { accounts: evm_accounts, ..Default::default() };

	evm_config.assimilate_storage(&mut t).unwrap();

	let assets_config = pallet_assets::GenesisConfig::<Runtime> {
		assets: vec![
			(USDC, authorities[0].clone(), true, 100_000), // 1 cent.
			(WETH, authorities[1].clone(), true, 100),     // 100 wei.
			(WBTC, authorities[2].clone(), true, 100),     // 100 satoshi.
		],
		metadata: vec![
			(USDC, Vec::from(b"USD Coin"), Vec::from(b"USDC"), 6),
			(WETH, Vec::from(b"Wrapped Ether"), Vec::from(b"WETH"), 18),
			(WBTC, Vec::from(b"Wrapped Bitcoin"), Vec::from(b"WBTC"), 18),
		],
		accounts: vec![
			(USDC, authorities[0].clone(), 1_000_000 * 10u128.pow(6)),
			(WETH, authorities[0].clone(), 100 * 10u128.pow(18)),
			(WBTC, authorities[0].clone(), 50 * 10u128.pow(18)),
			//
			(USDC, authorities[1].clone(), 1_000_000 * 10u128.pow(6)),
			(WETH, authorities[1].clone(), 100 * 10u128.pow(18)),
			(WBTC, authorities[1].clone(), 50 * 10u128.pow(18)),
			//
			(USDC, authorities[2].clone(), 1_000_000 * 10u128.pow(6)),
			(WETH, authorities[2].clone(), 100 * 10u128.pow(18)),
			(WBTC, authorities[2].clone(), 50 * 10u128.pow(18)),
		],
		next_asset_id: Some(4),
	};

	assets_config.assimilate_storage(&mut t).unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.register_extension(KeystoreExt(Arc::new(MemoryKeystore::new()) as KeystorePtr));
	ext.execute_with(|| System::set_block_number(1));
	ext.execute_with(|| {
		System::set_block_number(1);
		Session::on_initialize(1);
		<Staking as Hooks<u64>>::on_initialize(1);

		let call = <Runtime as pallet_services::Config>::EvmRunner::call(
			Services::pallet_evm_account(),
			USDC_ERC20,
			serde_json::from_value::<ethabi::Function>(json!({
				"name": "initialize",
				"inputs": [
					{
						"name": "name_",
						"type": "string",
						"internalType": "string"
					},
					{
						"name": "symbol_",
						"type": "string",
						"internalType": "string"
					},
					{
						"name": "decimals_",
						"type": "uint8",
						"internalType": "uint8"
					}
				],
				"outputs": [],
				"stateMutability": "nonpayable"
			}))
			.unwrap()
			.encode_input(&[
				ethabi::Token::String("USD Coin".to_string()),
				ethabi::Token::String("USDC".to_string()),
				ethabi::Token::Uint(6.into()),
			])
			.unwrap(),
			Default::default(),
			300_000,
			true,
			false,
		);

		assert_eq!(call.map(|info| info.exit_reason.is_succeed()).ok(), Some(true));
		// Mint
		for i in 1..=authorities.len() {
			let call = <Runtime as pallet_services::Config>::EvmRunner::call(
				Services::pallet_evm_account(),
				USDC_ERC20,
				serde_json::from_value::<ethabi::Function>(json!({
					"name": "mint",
					"inputs": [
						{
							"internalType": "address",
							"name": "account",
							"type": "address"
						},
						{
							"internalType": "uint256",
							"name": "amount",
							"type": "uint256"
						}
					],
					"outputs": [],
					"stateMutability": "nonpayable"
				}))
				.unwrap()
				.encode_input(&[
					ethabi::Token::Address(mock_address(i as u8)),
					ethabi::Token::Uint(Uint::from(100_000).mul(Uint::from(10).pow(Uint::from(6)))),
				])
				.unwrap(),
				Default::default(),
				300_000,
				true,
				false,
			);

			assert_eq!(call.map(|info| info.exit_reason.is_succeed()).ok(), Some(true));
		}
	});

	ext
}

#[macro_export]
macro_rules! evm_log {
	() => {
		fp_evm::Log { address: H160::zero(), topics: vec![], data: vec![] }
	};

	($contract:expr) => {
		fp_evm::Log { address: $contract, topics: vec![], data: vec![] }
	};

	($contract:expr, $topic:expr) => {
		fp_evm::Log {
			address: $contract,
			topics: vec![sp_core::keccak_256($topic).into()],
			data: vec![],
		}
	};
}

/// Asserts that the EVM logs are as expected.
#[track_caller]
pub fn assert_evm_logs(expected: &[fp_evm::Log]) {
	assert_evm_events_contains(expected.iter().cloned().collect())
}

/// Asserts that the EVM events are as expected.
#[track_caller]
fn assert_evm_events_contains(expected: Vec<fp_evm::Log>) {
	let actual: Vec<fp_evm::Log> = System::events()
		.iter()
		.filter_map(|e| match e.event {
			RuntimeEvent::EVM(pallet_evm::Event::Log { ref log }) => Some(log.clone()),
			_ => None,
		})
		.collect();

	// Check if `expected` is a subset of `actual`
	let mut any_matcher = false;
	for evt in expected {
		if !actual.contains(&evt) {
			panic!("Events don't match\nactual: {actual:?}\nexpected: {evt:?}");
		} else {
			any_matcher = true;
		}
	}

	// At least one event should be present
	if !any_matcher {
		panic!("No events found");
	}
}

// Checks events against the latest. A contiguous set of events must be
// provided. They must include the most recent RuntimeEvent, but do not have to include
// every past RuntimeEvent.
#[track_caller]
pub fn assert_events(mut expected: Vec<RuntimeEvent>) {
	let mut actual: Vec<RuntimeEvent> = System::events()
		.iter()
		.filter_map(|e| match e.event {
			RuntimeEvent::Services(_) => Some(e.event.clone()),
			_ => None,
		})
		.collect();

	expected.reverse();
	for evt in expected {
		let next = actual.pop().expect("RuntimeEvent expected");
		match (&next, &evt) {
			(left_val, right_val) => {
				if !(*left_val == *right_val) {
					panic!("Events don't match\nactual: {actual:#?}\nexpected: {evt:#?}");
				}
			},
		};
	}
}
