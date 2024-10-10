// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! Testing utilities.

use super::*;

use frame_support::derive_impl;
use frame_support::{construct_runtime, parameter_types, weights::Weight};
use pallet_evm::AddressMapping;
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::testing::{Bob, CryptoAlith, CryptoBaltathar, Precompile1};
use precompile_utils::{precompile_set::*, testing::MockAccount};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::{ConstU32, U256};
use sp_core::{Decode, Encode, MaxEncodedLen, H160};
use sp_runtime::BuildStorage;
use sp_std::ops::Deref;

/// Wrapper around MockAccount to implement AddressMapping
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Debug,
	Serialize,
	Deserialize,
	derive_more::Display,
	Encode,
	Decode,
	MaxEncodedLen,
	TypeInfo,
)]
pub struct WrappedMockAccount(pub MockAccount);

impl Deref for WrappedMockAccount {
	type Target = MockAccount;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl AddressMapping<WrappedMockAccount> for WrappedMockAccount {
	fn into_account_id(address: H160) -> WrappedMockAccount {
		WrappedMockAccount(address.into())
	}
}

impl From<AccountId32> for WrappedMockAccount {
	fn from(account_id: AccountId32) -> Self {
		let account_id_bytes: [u8; 32] = account_id.into();
		let evm_address = H160::from_slice(&account_id_bytes[0..20]);
		WrappedMockAccount(MockAccount(evm_address))
	}
}

impl From<CryptoAlith> for WrappedMockAccount {
	fn from(account: CryptoAlith) -> Self {
		WrappedMockAccount(MockAccount(account.into()))
	}
}

impl From<Bob> for WrappedMockAccount {
	fn from(account: Bob) -> Self {
		WrappedMockAccount(MockAccount(account.into()))
	}
}

impl From<Precompile1> for WrappedMockAccount {
	fn from(account: Precompile1) -> Self {
		WrappedMockAccount(MockAccount(account.into()))
	}
}

impl From<CryptoBaltathar> for WrappedMockAccount {
	fn from(account: CryptoBaltathar) -> Self {
		WrappedMockAccount(MockAccount(account.into()))
	}
}

pub type AccountId = WrappedMockAccount;
pub type Balance = u128;
pub type Block = frame_system::mocking::MockBlockU32<Runtime>;

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
	(PrecompileAt<AddressU64<1>, Erc20BalancesPrecompile<R, NativeErc20Metadata>>,),
>;

pub type PCall = Erc20BalancesPrecompileCall<Runtime, NativeErc20Metadata, ()>;

const MAX_POV_SIZE: u64 = 5 * 1024 * 1024;
/// Block storage limit in bytes. Set to 40 KB.
const BLOCK_STORAGE_LIMIT: u64 = 40 * 1024;

parameter_types! {
	pub BlockGasLimit: U256 = U256::from(u64::MAX);
	pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
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

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime	{
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		Timestamp: pallet_timestamp,
	}
);

/// ERC20 metadata for the native token.
pub struct NativeErc20Metadata;

impl Erc20Metadata for NativeErc20Metadata {
	/// Returns the name of the token.
	fn name() -> &'static str {
		"Mock token"
	}

	/// Returns the symbol of the token.
	fn symbol() -> &'static str {
		"MOCK"
	}

	/// Returns the decimals places of the token.
	fn decimals() -> u8 {
		18
	}

	/// Must return `true` only if it represents the main native currency of
	/// the network. It must be the currency used in `pallet_evm`.
	fn is_native_currency() -> bool {
		true
	}
}

pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder { balances: vec![] }
	}
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

pub(crate) fn events() -> Vec<RuntimeEvent> {
	System::events().into_iter().map(|r| r.event).collect::<Vec<_>>()
}
