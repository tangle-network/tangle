// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
//
// This file is part of pallet-evm-precompile-proxy package, originally developed by Purestake
// Inc. Pallet-evm-precompile-proxy package used in Tangle Network in terms of GPLv3.

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
use crate::{ProxyPrecompile, ProxyPrecompileCall};
use frame_support::{
	construct_runtime, derive_impl, parameter_types, traits::InstanceFilter, weights::Weight,
};
use pallet_evm::{EnsureAddressNever, EnsureAddressOrigin, SubstrateBlockHashMapping};
use precompile_utils::{
	precompile_set::{
		AddressU64, CallableByContract, CallableByPrecompile, OnlyFrom, PrecompileAt,
		PrecompileSetBuilder, RevertPrecompile, SubcallWithMaxNesting,
	},
	testing::MockAccount,
};
use scale_info::TypeInfo;
use sp_core::{H160, U256};
use sp_runtime::{
	codec::{Decode, Encode, MaxEncodedLen},
	traits::BlakeTwo256,
	BuildStorage,
};
pub type AccountId = MockAccount;
pub type Balance = u128;

type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		Timestamp: pallet_timestamp,
		Proxy: pallet_proxy,
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
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

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}
impl pallet_balances::Config for Runtime {
	type MaxReserves = ();
	type ReserveIdentifier = ();
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

pub type Precompiles<R> = PrecompileSetBuilder<
	R,
	(
		PrecompileAt<
			AddressU64<1>,
			ProxyPrecompile<R>,
			(
				SubcallWithMaxNesting<1>,
				CallableByContract<crate::OnlyIsProxyAndProxy<R>>,
				// Batch is the only precompile allowed to call Proxy.
				CallableByPrecompile<OnlyFrom<AddressU64<2>>>,
			),
		>,
		RevertPrecompile<AddressU64<2>>,
	),
>;

pub type PCall = ProxyPrecompileCall<Runtime>;

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

const MAX_POV_SIZE: u64 = 5 * 1024 * 1024;

parameter_types! {
	pub BlockGasLimit: U256 = U256::from(u64::MAX);
	pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub GasLimitPovSizeRatio: u64 = {
		let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
		block_gas_limit.saturating_div(MAX_POV_SIZE)
	};
}

parameter_types! {
	pub SuicideQuickClearLimit: u32 = 0;
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressAlways;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = AccountId;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = Precompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
	type SuicideQuickClearLimit = SuicideQuickClearLimit;
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}
impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

#[repr(u8)]
#[derive(
	Debug,
	Default,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Decode,
	MaxEncodedLen,
	Encode,
	Clone,
	Copy,
	TypeInfo,
)]
pub enum ProxyType {
	#[default]
	Any,
	Something = 1,
	Nothing = 2,
}

impl crate::EvmProxyCallFilter for ProxyType {
	fn is_evm_proxy_call_allowed(
		&self,
		_call: &crate::EvmSubCall,
		_recipient_has_code: bool,
		_gas: u64,
	) -> precompile_utils::EvmResult<bool> {
		Ok(match self {
			Self::Any => true,
			Self::Something => true,
			Self::Nothing => false,
		})
	}
}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, _: &RuntimeCall) -> bool {
		true
	}

	fn is_superset(&self, o: &Self) -> bool {
		(*self as u8) > (*o as u8)
	}
}

parameter_types! {
	pub const ProxyDepositBase: u64 = 100;
	pub const ProxyDepositFactor: u64 = 1;
	pub const MaxProxies: u32 = 5;
	pub const MaxPending: u32 = 5;
}
impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = ();
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = ();
	type AnnouncementDepositFactor = ();
}

/// Build test externalities, prepopulated with data for testing democracy precompiles
#[derive(Default)]
pub(crate) struct ExtBuilder {
	/// Endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl ExtBuilder {
	/// Fund some accounts before starting the test
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	/// Build the test externalities for use in tests
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
			.assimilate_storage(&mut t)
			.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| {
			System::set_block_number(1);
		});
		ext
	}
}

#[allow(dead_code)]
pub(crate) fn events() -> Vec<RuntimeEvent> {
	System::events().into_iter().map(|r| r.event).collect::<Vec<_>>()
}
