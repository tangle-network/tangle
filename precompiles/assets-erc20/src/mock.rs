// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// This file is part of pallet-evm-precompileset-assets-erc20 package, originally developed by
// Purestake Inc. pallet-evm-precompileset-assets-erc20 package used in Tangle Network in terms of
// GPLv3.

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

//! Testing utilities.

use super::*;

use frame_support::{
	construct_runtime, parameter_types, traits::AsEnsureOriginWithArg, weights::Weight,
};

use frame_support::derive_impl;
use frame_system::{EnsureNever, EnsureRoot};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::{
	mock_account,
	precompile_set::*,
	testing::{AddressInPrefixedSet, MockAccount},
};
use sp_runtime::{traits::ConstU32, BuildStorage};

pub type AccountId = MockAccount;
pub type AssetId = u128;
pub type Balance = u128;
pub type Block = frame_system::mocking::MockBlockU32<Runtime>;

/// The foreign asset precompile address prefix. Addresses that match against this prefix will
/// be routed to Erc20AssetsPrecompileSet being marked as foreign
pub const ASSET_PRECOMPILE_ADDRESS_PREFIX: u32 = 0xffffffff;

parameter_types! {
	pub ForeignAssetPrefix: &'static [u8] = &[0xff, 0xff, 0xff, 0xff];
}

mock_account!(ForeignAssetId(AssetId), |value: ForeignAssetId| {
	AddressInPrefixedSet(ASSET_PRECOMPILE_ADDRESS_PREFIX, value.0).into()
});

// Implement the trait, where we convert AccountId to AssetID
impl AddressToAssetId<AssetId> for Runtime {
	/// The way to convert an account to assetId is by ensuring that the prefix is 0XFFFFFFFF
	/// and by taking the lowest 128 bits as the assetId
	fn address_to_asset_id(address: H160) -> Option<AssetId> {
		let address: MockAccount = address.into();
		if address.has_prefix_u32(ASSET_PRECOMPILE_ADDRESS_PREFIX) {
			Some(address.without_prefix())
		} else {
			None
		}
	}

	fn asset_id_to_address(asset_id: AssetId) -> H160 {
		ForeignAssetId(asset_id).into()
	}
}

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
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
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
	type RuntimeHoldReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeFreezeReason = ();
}

pub type Precompiles<R> = PrecompileSetBuilder<
	R,
	(
		PrecompileSetStartingWith<
			ForeignAssetPrefix,
			Erc20AssetsPrecompileSet<R, pallet_assets::Instance1>,
		>,
	),
>;

pub type ForeignPCall = Erc20AssetsPrecompileSetCall<Runtime, pallet_assets::Instance1>;

const MAX_POV_SIZE: u64 = 5 * 1024 * 1024;
/// Block Storage Limit in bytes. Set to 40KB.
const BLOCK_STORAGE_LIMIT: u64 = 40 * 1024;

parameter_types! {
	pub BlockGasLimit: U256 = U256::from(u64::MAX);
	pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
	pub WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub GasLimitPovSizeRatio: u64 = {
		let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
		block_gas_limit.saturating_div(MAX_POV_SIZE)
	};
	pub GasLimitStorageGrowthRatio: u64 = {
		let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
		block_gas_limit.saturating_div(BLOCK_STORAGE_LIMIT)
	};
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<AccountId>;
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
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type SuicideQuickClearLimit = ConstU32<0>;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Runtime>;
}

type ForeignAssetInstance = pallet_assets::Instance1;

// Required for runtime benchmarks
pallet_assets::runtime_benchmarks_enabled! {
	pub struct BenchmarkHelper;
	impl<AssetIdParameter> pallet_assets::BenchmarkHelper<AssetIdParameter> for BenchmarkHelper
	where
		AssetIdParameter: From<u128>,
	{
		fn create_asset_id_parameter(id: u32) -> AssetIdParameter {
			(id as u128).into()
		}
	}
}

// These parameters dont matter much as this will only be called by root with the forced arguments
// No deposit is substracted with those methods
parameter_types! {
	pub const AssetDeposit: Balance = 0;
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 0;
	pub const MetadataDepositPerByte: Balance = 0;
	pub const AssetAccountDeposit: Balance = 0;
}

impl pallet_assets::Config<ForeignAssetInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type AssetAccountDeposit = AssetAccountDeposit;
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	type RemoveItemsLimit = ConstU32<0>;
	type AssetIdParameter = AssetId;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureNever<AccountId>>;
	type CallbackHandle = ();
	pallet_assets::runtime_benchmarks_enabled! {
		type BenchmarkHelper = BenchmarkHelper;
	}
}

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Balances: pallet_balances,
		Assets: pallet_assets::<Instance1>,
		Evm: pallet_evm,
		Timestamp: pallet_timestamp,
	}
);

#[derive(Default)]
pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
			.assimilate_storage(&mut t)
			.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
