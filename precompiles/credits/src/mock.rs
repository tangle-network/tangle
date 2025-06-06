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
use super::*;
use core::ops::Mul;
use ethabi::Uint;
use frame_election_provider_support::{
	bounds::{ElectionBounds, ElectionBoundsBuilder},
	onchain, SequentialPhragmen,
};
use frame_support::{
	construct_runtime, derive_impl,
	pallet_prelude::Hooks,
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU128, OneSessionHandler},
};
use pallet_session::historical as pallet_session_historical;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::{self, sr25519, sr25519::Public as sr25519Public, ConstU32, RuntimeDebug, H160};
use sp_keystore::{testing::MemoryKeystore, KeystoreExt, KeystorePtr};
use sp_runtime::{
	testing::UintAuthorityId,
	traits::{ConstU64, ConvertInto},
	AccountId32, BuildStorage, Perbill,
};
use std::{collections::BTreeMap, sync::Arc};
use tangle_primitives::{
	rewards::{AssetType, UserDepositWithLocks},
	services::Asset,
};

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

impl From<AccountId32> for TestAccount {
	fn from(x: AccountId32) -> Self {
		let bytes: [u8; 32] = x.into();
		match bytes {
			a if a == [1u8; 32] => TestAccount::Alex,
			a if a == [2u8; 32] => TestAccount::Bob,
			a if a == [3u8; 32] => TestAccount::Charlie,
			a if a == [4u8; 32] => TestAccount::Dave,
			a if a == [5u8; 32] => TestAccount::Eve,
			a if a == PRECOMPILE_ADDRESS_BYTES => TestAccount::PrecompileAddress,
			_ => TestAccount::Empty,
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

pub type AssetId = u128;

pub struct MockDelegationManager;
impl
	tangle_primitives::traits::MultiAssetDelegationInfo<AccountId, Balance, u64, AssetId, AssetType>
	for MockDelegationManager
{
	fn get_current_round() -> tangle_primitives::types::RoundIndex {
		Default::default()
	}

	fn is_operator(_operator: &AccountId) -> bool {
		// don't care
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

	fn get_total_delegation_by_asset(_operator: &AccountId, _asset_id: &Asset<AssetId>) -> Balance {
		Default::default()
	}

	fn get_delegators_for_operator(
		_operator: &AccountId,
	) -> Vec<(AccountId, Balance, Asset<AssetId>)> {
		Default::default()
	}

	fn get_user_deposit_with_locks(
		_who: &AccountId,
		_asset: Asset<AssetId>,
	) -> Option<UserDepositWithLocks<Balance, u64>> {
		None
	}

	fn get_user_deposit_by_asset_type(_who: &AccountId, _asset_type: AssetType) -> Option<Balance> {
		None
	}
}

parameter_types! {
	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const MaxStakeTiers: u32 = 10;

	#[derive(Default, Copy, Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub const CreditBurnRecipient: Option<AccountId> = None;
}

impl pallet_credits::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type AssetId = AssetId;
	type MultiAssetDelegationInfo = MockDelegationManager;
	type BurnConversionRate = ConstU128<1000>;
	type ClaimWindowBlocks = ConstU64<1000>;
	type CreditBurnRecipient = CreditBurnRecipient;
	type MaxOffchainAccountIdLength = ConstU32<100>;
	type MaxStakeTiers = MaxStakeTiers;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type WeightInfo = ();
}

type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Assets: pallet_assets,
		Credits: pallet_credits,
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

pub const MBSM: H160 = H160([0x12; 20]);
pub const CGGMP21_BLUEPRINT: H160 = H160([0x21; 20]);
pub const USDC_ERC20: H160 = H160([0x23; 20]);

#[allow(dead_code)]
pub const TNT: AssetId = 0;
pub const USDC: AssetId = 1;
pub const WETH: AssetId = 2;
pub const WBTC: AssetId = 3;

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
	let balances: Vec<_> = authorities.iter().map(|i| (i.clone(), 20_000_000_u128)).collect();
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

	create_contract(
		include_str!("../../../pallets/services/src/test-artifacts/CGGMP21Blueprint.hex"),
		CGGMP21_BLUEPRINT,
	);
	create_contract(
		include_str!(
			"../../../pallets/services/src/test-artifacts/MasterBlueprintServiceManager.hex"
		),
		MBSM,
	);

	create_contract(
		include_str!("../../../pallets/services/src/test-artifacts/MockERC20.hex"),
		USDC_ERC20,
	);

	// Add some initial balance to the authorities in the EVM pallet
	for a in authorities.iter().cloned() {
		evm_accounts.insert(
			TestAccount::from(a).into(),
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
	});

	ext
}
