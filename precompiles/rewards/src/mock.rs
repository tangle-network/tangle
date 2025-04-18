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
use super::*;
use crate::mock_evm::*;
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU64},
	weights::Weight,
	PalletId,
};
use pallet_evm::GasWeightMapping;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::{
	self,
	sr25519::{Public as sr25519Public, Signature},
	ConstU32, H160,
};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	AccountId32, BuildStorage,
};
use tangle_primitives::{
	services::{EvmAddressMapping, EvmGasWeightMapping},
	ServiceManager,
};

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u64;

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
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
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

impl pallet_multi_asset_delegation::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MinOperatorBondAmount = MinOperatorBondAmount;

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
	type VaultId = AssetId;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxDelegatorBlueprints = MaxDelegatorBlueprints;
	type MaxOperatorBlueprints = MaxOperatorBlueprints;
	type MaxWithdrawRequests = MaxWithdrawRequests;
	type MaxUnstakeRequests = MaxUnstakeRequests;
	type MaxDelegations = MaxDelegations;
	type PalletId = PID;
	type WeightInfo = ();
}

/// Build test externalities, prepopulated with data for testing democracy precompiles
#[derive(Default)]
pub struct ExtBuilder {
	/// Endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

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

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| {
			System::set_block_number(1);
		});
		ext
	}
}
