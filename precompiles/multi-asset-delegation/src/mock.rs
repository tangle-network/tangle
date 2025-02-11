// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// This file is part of pallet-evm-precompile-multi-asset-delegation package.
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
use std::{collections::BTreeMap, sync::Arc};

use super::*;
use crate::mock_evm::*;
use core::ops::Mul;
use ethabi::Uint;
use frame_election_provider_support::onchain;
use frame_election_provider_support::SequentialPhragmen;
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU64},
	weights::Weight,
	PalletId,
};
use pallet_evm::GasWeightMapping;
use pallet_multi_asset_delegation::mock::ElectionBoundsOnChain;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sp_core::{
	self,
	sr25519::{Public as sr25519Public, Signature},
	ConstU32, H160,
};
use sp_keystore::{testing::MemoryKeystore, KeystoreExt, KeystorePtr};
use sp_runtime::curve::PiecewiseLinear;
use sp_runtime::DispatchError;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	AccountId32, BuildStorage, Perbill,
};
use sp_staking::{ConvertCurve, EraIndex, StakingInterface};
use tangle_primitives::services::EvmRunner;
use tangle_primitives::services::{EvmAddressMapping, EvmGasWeightMapping};
use tangle_primitives::traits::{RewardsManager, ServiceManager};

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u64;
pub type BlockNumber = u64;

type Block = frame_system::mocking::MockBlock<Runtime>;
type AssetId = u128;

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

impl From<TestAccount> for Address {
	fn from(x: TestAccount) -> Address {
		let h160: H160 = x.into();
		Address::from(h160)
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
	pub enum Runtime
	{
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		Ethereum: pallet_ethereum,
		Session: pallet_session,
		Staking: pallet_staking,
		Historical: pallet_session_historical,
		Timestamp: pallet_timestamp,
		Assets: pallet_assets,
		MultiAssetDelegation: pallet_multi_asset_delegation,
	}
);

parameter_types! {
	pub const SS58Prefix: u8 = 42;
	pub static ExistentialDeposit: Balance = 1;
}

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
	type RuntimeFreezeReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
}

impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u64;
	type AssetId = AssetId;
	type AssetIdParameter = u128;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type AssetDeposit = ConstU64<1>;
	type AssetAccountDeposit = ConstU64<10>;
	type MetadataDepositBase = ConstU64<1>;
	type MetadataDepositPerByte = ConstU64<1>;
	type ApprovalDeposit = ConstU64<1>;
	type StringLimit = ConstU32<50>;
	type Freezer = ();
	type WeightInfo = ();
	type CallbackHandle = ();
	type Extra = ();
	type RemoveItemsLimit = ConstU32<5>;
}

pub struct MockServiceManager;

impl ServiceManager<AccountId, Balance> for MockServiceManager {
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
		// we don't care
		Default::default()
	}

	fn has_active_services(_operator: &AccountId) -> bool {
		false
	}
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
		account_id.using_encoded(|b| {
			let mut addr = [0u8; 20];
			addr.copy_from_slice(&b[0..20]);
			H160(addr)
		})
	}
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaxLocks: u32 = 50;
	pub const MinOperatorBondAmount: u64 = 10_000;
	pub const BondDuration: u32 = 10;
	pub PID: PalletId = PalletId(*b"PotStake");

	#[derive(PartialEq, Eq, Clone, Copy, Debug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub const MaxDelegatorBlueprints : u32 = 50;

	#[derive(PartialEq, Eq, Clone, Copy, Debug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub const MaxOperatorBlueprints : u32 = 50;

	#[derive(PartialEq, Eq, Clone, Copy, Debug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub const MaxWithdrawRequests: u32 = 5;

	#[derive(PartialEq, Eq, Clone, Copy, Debug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub const MaxUnstakeRequests: u32 = 5;

	#[derive(PartialEq, Eq, Clone, Copy, Debug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub const MaxDelegations: u32 = 50;
}

pub struct MockRewardsManager;

impl RewardsManager<AccountId, AssetId, Balance, BlockNumber> for MockRewardsManager {
	type Error = DispatchError;

	fn record_deposit(
		_account_id: &AccountId,
		_asset: Asset<AssetId>,
		_amount: Balance,
		_lock_multiplier: Option<LockMultiplier>,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn record_withdrawal(
		_account_id: &AccountId,
		_asset: Asset<AssetId>,
		_amount: Balance,
	) -> Result<(), Self::Error> {
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

impl pallet_multi_asset_delegation::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MinOperatorBondAmount = MinOperatorBondAmount;
	type BondDuration = BondDuration;
	type CurrencyToVote = ();
	type StakingInterface = MockStakingInterface;
	type ServiceManager = MockServiceManager;
	type LeaveOperatorsDelay = ConstU32<10>;
	type EvmRunner = MockedEvmRunner;
	type EvmAddressMapping = PalletEVMAddressMapping;
	type EvmGasWeightMapping = PalletEVMGasWeightMapping;
	type OperatorBondLessDelay = ConstU32<1>;
	type LeaveDelegatorsDelay = ConstU32<1>;
	type DelegationBondLessDelay = ConstU32<5>;
	type MinDelegateAmount = ConstU64<100>;
	type Fungibles = Assets;
	type AssetId = AssetId;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxDelegatorBlueprints = MaxDelegatorBlueprints;
	type MaxOperatorBlueprints = MaxOperatorBlueprints;
	type MaxWithdrawRequests = MaxWithdrawRequests;
	type MaxUnstakeRequests = MaxUnstakeRequests;
	type MaxDelegations = MaxDelegations;
	type PalletId = PID;
	type RewardsManager = MockRewardsManager;
	type WeightInfo = ();
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

sp_runtime::impl_opaque_keys! {
	pub struct MockSessionKeys {
		pub dummy: MockSessionHandler,
	}
}

parameter_types! {
	pub static SessionsPerEra: SessionIndex = 3;
	pub static SlashDeferDuration: EraIndex = 0;
	pub static Period: BlockNumber = 5;
	pub static Offset: BlockNumber = 0;
}

impl pallet_session::Config for Runtime {
	type SessionManager = Staking;
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

pub struct MockReward {}
impl frame_support::traits::OnUnbalanced<pallet_balances::PositiveImbalance<Runtime>>
	for MockReward
{
	fn on_unbalanced(_: pallet_balances::PositiveImbalance<Runtime>) {
		RewardOnUnbalanceWasCalled::set(true);
	}
}

impl pallet_staking::Config for Runtime {
	type Currency = Balances;
	type CurrencyBalance = <Self as pallet_balances::Config>::Balance;
	type UnixTime = pallet_timestamp::Pallet<Self>;
	type CurrencyToVote = ();
	type RewardRemainder = ();
	type RuntimeEvent = RuntimeEvent;
	type Slash = ();
	type Reward = MockReward;
	type SessionsPerEra = SessionsPerEra;
	type SlashDeferDuration = SlashDeferDuration;
	type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type BondingDuration = ();
	type SessionInterface = Self;
	type EraPayout = ConvertCurve<RewardCurve>;
	type MaxExposurePageSize = ConstU32<64>;
	type MaxControllersInDeprecationBatch = ConstU32<100>;
	type NextNewSession = Session;
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

/// Build test externalities, prepopulated with data for testing democracy precompiles
#[derive(Default)]
pub struct ExtBuilder {
	/// Endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

pub fn mock_address(id: u8) -> H160 {
	H160::from_slice(&[id; 20])
}

pub fn account_id_to_address(account_id: AccountId) -> H160 {
	H160::from_slice(&AsRef::<[u8; 32]>::as_ref(&account_id)[0..20])
}

pub const USDC_ERC20: H160 = H160([0x23; 20]);

impl ExtBuilder {
	/// Build the test externalities for use in tests
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self
				.balances
				.iter()
				.chain(
					[
						(TestAccount::Alex.into(), 1_000_000),
						(TestAccount::Bobo.into(), 1_000_000),
						(TestAccount::Charlie.into(), 1_000_000),
						(MultiAssetDelegation::pallet_account(), 100), /* give pallet some ED so
						                                                * it can receive tokens */
					]
					.iter(),
				)
				.cloned()
				.collect(),
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		let mut evm_accounts = BTreeMap::new();

		let accounts = [
			TestAccount::Alex,
			TestAccount::Bobo,
			TestAccount::Charlie,
			TestAccount::Dave,
			TestAccount::Eve,
		];

		for i in 1..=accounts.len() {
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

		for a in &accounts {
			evm_accounts.insert(
				a.clone().into(),
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

		// assets_config.assimilate_storage(&mut t).unwrap();
		let mut ext = sp_io::TestExternalities::new(t);
		ext.register_extension(KeystoreExt(Arc::new(MemoryKeystore::new()) as KeystorePtr));
		ext.execute_with(|| System::set_block_number(1));
		ext.execute_with(|| {
			System::set_block_number(1);

			let call = <Runtime as pallet_multi_asset_delegation::Config>::EvmRunner::call(
				MultiAssetDelegation::pallet_evm_account(),
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
			for i in 1..=accounts.len() {
				let call = <Runtime as pallet_multi_asset_delegation::Config>::EvmRunner::call(
					MultiAssetDelegation::pallet_evm_account(),
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
						ethabi::Token::Uint(
							Uint::from(100_000).mul(Uint::from(10).pow(Uint::from(6))),
						),
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

		ext.execute_with(|| {
			System::set_block_number(1);
		});
		ext
	}
}

pub struct MockStakingInterface;

impl StakingInterface for MockStakingInterface {
	type CurrencyToVote = ();
	type AccountId = AccountId;
	type Balance = Balance;

	fn minimum_nominator_bond() -> Self::Balance {
		unimplemented!()
	}

	fn minimum_validator_bond() -> Self::Balance {
		unimplemented!()
	}

	fn stash_by_ctrl(_controller: &Self::AccountId) -> Result<Self::AccountId, DispatchError> {
		unimplemented!()
	}

	fn bonding_duration() -> sp_staking::EraIndex {
		unimplemented!()
	}

	fn current_era() -> sp_staking::EraIndex {
		unimplemented!()
	}

	fn stake(_who: &Self::AccountId) -> Result<sp_staking::Stake<Self::Balance>, DispatchError> {
		unimplemented!()
	}

	fn bond(
		_who: &Self::AccountId,
		_value: Self::Balance,
		_payee: &Self::AccountId,
	) -> sp_runtime::DispatchResult {
		unimplemented!()
	}

	fn nominate(
		_who: &Self::AccountId,
		_validators: Vec<Self::AccountId>,
	) -> sp_runtime::DispatchResult {
		unimplemented!()
	}

	fn chill(_who: &Self::AccountId) -> sp_runtime::DispatchResult {
		unimplemented!()
	}

	fn bond_extra(_who: &Self::AccountId, _extra: Self::Balance) -> sp_runtime::DispatchResult {
		unimplemented!()
	}

	fn unbond(_stash: &Self::AccountId, _value: Self::Balance) -> sp_runtime::DispatchResult {
		unimplemented!()
	}

	fn update_payee(
		_stash: &Self::AccountId,
		_reward_acc: &Self::AccountId,
	) -> sp_runtime::DispatchResult {
		unimplemented!()
	}

	fn withdraw_unbonded(
		_stash: Self::AccountId,
		_num_slashing_spans: u32,
	) -> Result<bool, DispatchError> {
		unimplemented!()
	}

	fn desired_validator_count() -> u32 {
		unimplemented!()
	}

	fn election_ongoing() -> bool {
		unimplemented!()
	}

	fn force_unstake(_who: Self::AccountId) -> sp_runtime::DispatchResult {
		unimplemented!()
	}

	fn is_exposed_in_era(_who: &Self::AccountId, _era: &sp_staking::EraIndex) -> bool {
		unimplemented!()
	}

	fn status(
		_who: &Self::AccountId,
	) -> Result<sp_staking::StakerStatus<Self::AccountId>, DispatchError> {
		unimplemented!()
	}

	fn is_virtual_staker(_who: &Self::AccountId) -> bool {
		unimplemented!()
	}

	fn slash_reward_fraction() -> sp_runtime::Perbill {
		unimplemented!()
	}
}
