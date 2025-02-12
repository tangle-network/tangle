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
use crate::{self as pallet_multi_asset_delegation};
use ethabi::Uint;
use frame_election_provider_support::{
	bounds::{ElectionBounds, ElectionBoundsBuilder},
	onchain, SequentialPhragmen,
};
use frame_support::{
	construct_runtime, derive_impl,
	pallet_prelude::{Hooks, Weight},
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU128, ConstU32, OneSessionHandler},
	PalletId,
};
use frame_system::pallet_prelude::BlockNumberFor;
use mock_evm::MockedEvmRunner;
use pallet_evm::GasWeightMapping;
use pallet_session::historical as pallet_session_historical;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde_json::json;
use sp_core::{sr25519, H160};
use sp_keyring::AccountKeyring;
use sp_keystore::{testing::MemoryKeystore, KeystoreExt, KeystorePtr};
use sp_runtime::{
	generic,
	testing::UintAuthorityId,
	traits::{ConvertInto, IdentityLookup, OpaqueKeys},
	AccountId32, BoundToRuntimeAppPublic, BuildStorage, DispatchError, Perbill,
};
use sp_staking::currency_to_vote::U128CurrencyToVote;
use std::cell::RefCell;
use tangle_primitives::{
	services::{EvmAddressMapping, EvmGasWeightMapping, EvmRunner},
	traits::RewardsManager,
	types::rewards::LockMultiplier,
};

use core::ops::Mul;
use std::{collections::BTreeMap, sync::Arc};

pub type AccountId = AccountId32;
pub type Balance = u128;
pub type Nonce = u32;
pub type AssetId = u128;
pub type BlockNumber = BlockNumberFor<Runtime>;

pub use tangle_primitives::services::Asset;

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

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
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

impl BoundToRuntimeAppPublic for MockSessionHandler {
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
	type SessionManager = pallet::RoundChangeSessionManager<Self, MockSessionManager>;
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
	pub const ServicesEVMAddress: H160 = H160([0x11; 20]);
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
	type AssetIdParameter = AssetId;
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

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaxLocks: u32 = 50;
	pub const MinOperatorBondAmount: u64 = 10_000;
	pub const BondDuration: u32 = 10;
	pub PID: PalletId = PalletId(*b"PotStake");

	pub const SlashRecipient: AccountId = mock_pub_key(1);

	#[derive(PartialEq, Eq, Clone, Copy, Debug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub const MaxDelegatorBlueprints : u32 = 50;

	#[derive(PartialEq, Eq, Clone, Copy, Debug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub const MaxOperatorBlueprints : u32 = 50;

	#[derive(PartialEq, Eq, Clone, Copy, Debug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub const MaxWithdrawRequests: u32 = 50;

	#[derive(PartialEq, Eq, Clone, Copy, Debug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub const MaxUnstakeRequests: u32 = 50;

	#[derive(PartialEq, Eq, Clone, Copy, Debug, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub const MaxDelegations: u32 = 50;
}

type DepositCall = (AccountId, Asset<AssetId>, Balance, Option<LockMultiplier>);
type WithdrawalCall = (AccountId, Asset<AssetId>, Balance);

thread_local! {
	static DEPOSIT_CALLS: RefCell<Vec<DepositCall>> = RefCell::new(Vec::new());
	static WITHDRAWAL_CALLS: RefCell<Vec<WithdrawalCall>> = RefCell::new(Vec::new());
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
	pub fn record_deposit_calls() -> Vec<DepositCall> {
		DEPOSIT_CALLS.with(|calls| calls.borrow().clone())
	}

	pub fn record_withdrawal_calls() -> Vec<WithdrawalCall> {
		WITHDRAWAL_CALLS.with(|calls| calls.borrow().clone())
	}

	pub fn clear_all() {
		DEPOSIT_CALLS.with(|calls| calls.borrow_mut().clear());
		WITHDRAWAL_CALLS.with(|calls| calls.borrow_mut().clear());
	}
}

impl pallet_multi_asset_delegation::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MinOperatorBondAmount = MinOperatorBondAmount;
	type SlashRecipient = MockSlashRecipient;
	type BondDuration = BondDuration;
	type CurrencyToVote = U128CurrencyToVote;
	type StakingInterface = Staking;
	type ServiceManager = MockServiceManager;
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

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(
	Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, MaxEncodedLen, TypeInfo,
)]
pub enum ProxyType {
	/// All calls can be proxied. This is the trivial/most permissive filter.
	Any = 0,
	/// Only extrinsics related to governance (democracy and collectives).
	Governance = 1,
	/// Allow to veto an announced proxy call.
	CancelProxy = 2,
	/// Allow extrinsic related to Balances.
	Balances = 3,
	/// Allow extrinsic related to Staking.
	Staking = 4,
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl frame_support::traits::InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::Governance => false,
			ProxyType::CancelProxy => false,
			ProxyType::Balances => matches!(c, RuntimeCall::Balances(..)),
			ProxyType::Staking => matches!(c, RuntimeCall::Staking(..)),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ConstU128<1>;
	type ProxyDepositFactor = ConstU128<1>;
	type MaxProxies = ConstU32<32>;
	type WeightInfo = ();
	type MaxPending = ConstU32<32>;
	type CallHasher = sp_runtime::traits::BlakeTwo256;
	type AnnouncementDepositBase = ConstU128<1>;
	type AnnouncementDepositFactor = ConstU128<1>;
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

/// An unchecked extrinsic type to be used in tests.
pub type MockUncheckedExtrinsic = generic::UncheckedExtrinsic<
	AccountId,
	RuntimeCall,
	u32,
	extra::CheckNominatedRestaked<Runtime>,
>;

/// An implementation of `sp_runtime::traits::Block` to be used in tests.
type Block =
	generic::Block<generic::Header<u64, sp_runtime::traits::BlakeTwo256>, MockUncheckedExtrinsic>;

construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Assets: pallet_assets,
		MultiAssetDelegation: pallet_multi_asset_delegation,
		EVM: pallet_evm,
		Ethereum: pallet_ethereum,
		Session: pallet_session,
		Staking: pallet_staking,
		Historical: pallet_session_historical,
		Proxy: pallet_proxy,
		Utility: pallet_utility,
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

pub fn new_test_ext() -> sp_io::TestExternalities {
	let ext = new_test_ext_raw_authorities();
	MockRewardsManager::clear_all();
	ext
}

pub const TNT: AssetId = 0;
pub const USDC: AssetId = 1;
pub const WETH: AssetId = 2;
pub const USDC_ERC20: H160 = H160([0x23; 20]);
pub const VDOT: AssetId = 4;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext_raw_authorities() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
	// We use default for brevity, but you can configure as desired if needed.
	let authorities: Vec<AccountId> = vec![
		AccountKeyring::Alice.into(),
		AccountKeyring::Bob.into(),
		AccountKeyring::Charlie.into(),
	];
	let mut balances: Vec<_> = authorities.iter().map(|i| (i.clone(), 200_000_u128)).collect();

	// Add test accounts with enough balance
	let test_accounts = vec![
		AccountKeyring::Dave.into(),
		AccountKeyring::Eve.into(),
		MultiAssetDelegation::pallet_account(),
	];
	balances.extend(test_accounts.iter().map(|i: &AccountId| (i.clone(), 1_000_000_u128)));

	pallet_balances::GenesisConfig::<Runtime> { balances }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut evm_accounts = BTreeMap::new();

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

	let staking_config = pallet_staking::GenesisConfig::<Runtime> {
		validator_count: 3,
		invulnerables: authorities.clone(),
		..Default::default()
	};

	staking_config.assimilate_storage(&mut t).unwrap();

	// assets_config.assimilate_storage(&mut t).unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.register_extension(KeystoreExt(Arc::new(MemoryKeystore::new()) as KeystorePtr));
	ext.execute_with(|| System::set_block_number(1));
	ext.execute_with(|| {
		System::set_block_number(1);
		Session::on_initialize(1);
		<Staking as Hooks<u64>>::on_initialize(1);

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
		for i in 1..=authorities.len() {
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
