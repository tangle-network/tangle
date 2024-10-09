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
use frame_election_provider_support::{
	bounds::{ElectionBounds, ElectionBoundsBuilder},
	onchain, SequentialPhragmen,
};
use frame_support::derive_impl;
use frame_support::pallet_prelude::Hooks;
use frame_support::pallet_prelude::Weight;
use frame_support::PalletId;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU128, OneSessionHandler},
};
use mock_evm::MockedEvmRunner;
use pallet_evm::GasWeightMapping;
use pallet_services::EvmGasWeightMapping;
use pallet_session::historical as pallet_session_historical;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::Deserialize;
use serde::Serialize;
use sp_core::{self, sr25519, sr25519::Public as sr25519Public, ConstU32, RuntimeDebug, H160};
use sp_keystore::{testing::MemoryKeystore, KeystoreExt, KeystorePtr};
use sp_runtime::{
	testing::UintAuthorityId, traits::ConvertInto, AccountId32, BuildStorage, Perbill,
};
use std::{collections::BTreeMap, sync::Arc};

pub type AccountId = AccountId32;
pub type Balance = u128;

const ALICE: u8 = 1;
const BOB: u8 = 2;
const CHARLIE: u8 = 3;
const DAVE: u8 = 4;
const EVE: u8 = 5;

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type SS58Prefix = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = sp_core::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ();
	type DbWeight = ();
	type BlockLength = ();
	type BlockWeights = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
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
	pub const ServicesPalletId: PalletId = PalletId(*b"py/srvcs");
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

const PRECOMPILE_ADDRESS_BYTES: [u8; 32] = [
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
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
	Bob,
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
			a if a == H160::repeat_byte(0x02) => TestAccount::Bob.into(),
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
			TestAccount::Bob => H160::repeat_byte(0x02),
			TestAccount::Charlie => H160::repeat_byte(0x03),
			TestAccount::Dave => H160::repeat_byte(0x04),
			TestAccount::Eve => H160::repeat_byte(0x05),
			TestAccount::PrecompileAddress => H160::from_low_u64_be(6),
			_ => Default::default(),
		}
	}
}

impl From<TestAccount> for AccountId32 {
	fn from(x: TestAccount) -> Self {
		match x {
			TestAccount::Alex => AccountId32::from([1u8; 32]),
			TestAccount::Bob => AccountId32::from([2u8; 32]),
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
			TestAccount::Bob => sr25519Public::from_raw([2u8; 32]),
			TestAccount::Charlie => sr25519Public::from_raw([3u8; 32]),
			TestAccount::Dave => sr25519Public::from_raw([4u8; 32]),
			TestAccount::Eve => sr25519Public::from_raw([5u8; 32]),
			TestAccount::PrecompileAddress => sr25519Public::from_raw(PRECOMPILE_ADDRESS_BYTES),
			_ => sr25519Public::from_raw([0u8; 32]),
		}
	}
}

pub type AssetId = u32;

pub struct MockDelegationManager;
impl tangle_primitives::traits::MultiAssetDelegationInfo<AccountId, Balance>
	for MockDelegationManager
{
	type AssetId = AssetId;

	fn get_current_round() -> tangle_primitives::types::RoundIndex {
		Default::default()
	}

	fn is_operator(_operator: &AccountId) -> bool {
		// dont care
		true
	}

	fn is_operator_active(operator: &AccountId) -> bool {
		if operator == &mock_pub_key(10) {
			return false;
		}
		true
	}

	fn get_operator_stake(_operator: &AccountId) -> Balance {
		Default::default()
	}

	fn get_total_delegation_by_asset_id(
		_operator: &AccountId,
		_asset_id: &Self::AssetId,
	) -> Balance {
		Default::default()
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

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxAssetsPerService: u32 = 164;
}

impl pallet_services::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Currency = Balances;
	type AssetId = AssetId;
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
	type MaxAssetsPerService = MaxAssetsPerService;
	type Constraints = pallet_services::types::ConstraintsOf<Self>;
	type OperatorDelegationManager = MockDelegationManager;
	type WeightInfo = ();
}

type Block = frame_system::mocking::MockBlock<Runtime>;

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

pub fn mock_pub_key(id: u8) -> AccountId {
	sr25519::Public::from_raw([id; 32]).into()
}

pub fn mock_authorities(vec: Vec<u8>) -> Vec<AccountId> {
	vec.into_iter().map(|id| mock_pub_key(id)).collect()
}

pub const CGGMP21_BLUEPRINT: H160 = H160([0x21; 20]);

/// Build test externalities, prepopulated with data for testing democracy precompiles
#[derive(Default)]
pub(crate) struct ExtBuilder;

impl ExtBuilder {
	/// Build the test externalities for use in tests
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let ids = vec![ALICE, BOB, CHARLIE, DAVE, EVE];
		new_test_ext_raw_authorities(mock_authorities(ids))
	}
}

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

	let cggmp21_blueprint_json: serde_json::Value = serde_json::from_str(include_str!(
		"../../../pallets/services/src/test-artifacts/CGGMP21Blueprint.json"
	))
	.unwrap();
	let cggmp21_blueprint_code = hex::decode(
		cggmp21_blueprint_json["deployedBytecode"]["object"]
			.as_str()
			.unwrap()
			.replace("0x", ""),
	)
	.unwrap();
	evm_accounts.insert(
		CGGMP21_BLUEPRINT,
		fp_evm::GenesisAccount {
			code: cggmp21_blueprint_code,
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
