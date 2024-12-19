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
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{ConstU32, ConstU64, SortedMembers},
	weights::Weight,
	BoundedVec,
};
use pallet_evm::{EnsureAddressNever, EnsureAddressOrigin, SubstrateBlockHashMapping};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::{H160, U256};
use sp_runtime::{
	traits::{IdentifyAccount, IdentityLookup},
	AccountId32, BuildStorage,
};
use std::fmt;

pub const ALICE: H160 = H160::repeat_byte(0x01);
pub const BOB: H160 = H160::repeat_byte(0x02);

pub type AccountId = TestAccount;
pub type Balance = u64;
pub type AssetId = u32;
pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
pub type Block = frame_system::mocking::MockBlock<Runtime>;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub const ExistentialDeposit: u64 = 1;
	pub const MinimumPeriod: u64 = 5;
	pub const MaxFeedValues: u32 = 1000;
	pub const MaxHasDispatchedSize: u32 = 100;
	pub const MaxFreezes: u32 = 50;
	pub BlockGasLimit: U256 = U256::from(u64::MAX);
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub GasLimitPovSizeRatio: u64 = {
		let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
		block_gas_limit.saturating_div(5 * 1024 * 1024)
	};
	pub const SuicideQuickClearLimit: u32 = 0;
	pub const RootOperatorAccountId: AccountId = TestAccount::Alice;
}

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
	TypeInfo,
	Copy,
	Serialize,
	Deserialize,
)]
pub enum TestAccount {
	Empty,
	Alice,
	Bob,
	Charlie,
	Dave,
}

impl fmt::Display for TestAccount {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			TestAccount::Empty => write!(f, "Empty"),
			TestAccount::Alice => write!(f, "Alice"),
			TestAccount::Bob => write!(f, "Bob"),
			TestAccount::Charlie => write!(f, "Charlie"),
			TestAccount::Dave => write!(f, "Dave"),
		}
	}
}

impl Default for TestAccount {
	fn default() -> Self {
		Self::Empty
	}
}

impl IdentifyAccount for TestAccount {
	type AccountId = AccountId;

	fn into_account(self) -> Self::AccountId {
		self
	}
}

impl AddressMapping<AccountId32> for TestAccount {
	fn into_account_id(h160_account: H160) -> AccountId32 {
		match h160_account {
			a if a == ALICE => AccountId32::new([0u8; 32]),
			a if a == BOB => AccountId32::new([1u8; 32]),
			_ => AccountId32::new([0u8; 32]),
		}
	}
}

impl pallet_evm::AddressMapping<TestAccount> for TestAccount {
	fn into_account_id(h160_account: H160) -> TestAccount {
		match h160_account {
			a if a == ALICE => TestAccount::Alice,
			a if a == BOB => TestAccount::Bob,
			_ => TestAccount::Empty,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct MockMembers;
impl SortedMembers<AccountId> for MockMembers {
	fn sorted_members() -> Vec<AccountId> {
		vec![TestAccount::Alice]
	}
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type AccountData = pallet_balances::AccountData<Balance>;
}

impl pallet_balances::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type FreezeIdentifier = ();
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type MaxFreezes = MaxFreezes;
	type RuntimeFreezeReason = RuntimeFreezeReason;
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

pub struct EnsureAddressAlways;
impl<OuterOrigin> EnsureAddressOrigin<OuterOrigin> for EnsureAddressAlways {
	type Success = ();

	fn try_address_origin(
		_address: &H160,
		_origin: OuterOrigin,
	) -> Result<Self::Success, OuterOrigin> {
		Ok(())
	}

	fn ensure_address_origin(
		_address: &H160,
		_origin: OuterOrigin,
	) -> Result<Self::Success, sp_runtime::traits::BadOrigin> {
		Ok(())
	}
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressAlways;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = TestAccount;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = SubstrateBlockHashMapping<Self>;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type OnChargeTransaction = ();
	type OnCreate = ();
	type FindAuthor = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type SuicideQuickClearLimit = SuicideQuickClearLimit;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Runtime>;
}

impl pallet_oracle::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Time = Timestamp;
	type Members = MockMembers;
	type MaxFeedValues = MaxFeedValues;
	type WeightInfo = ();
	type OnNewData = ();
	type CombineData = pallet_oracle::DefaultCombineData<Runtime, ConstU32<1>, ConstU64<600>>;
	type OracleKey = u32;
	type OracleValue = u64;
	type RootOperatorAccountId = RootOperatorAccountId;
	type MaxHasDispatchedSize = MaxHasDispatchedSize;
}

construct_runtime!(
	pub enum Runtime
	where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system,
		Balances: pallet_balances,
		Timestamp: pallet_timestamp,
		EVM: pallet_evm,
		Oracle: pallet_oracle,
	}
);

#[derive(Default)]
pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![(TestAccount::Alice, 1_000_000_000_000)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
