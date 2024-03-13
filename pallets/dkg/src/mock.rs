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

pub use crate as pallet_dkg;

use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU128, ConstU32, ConstU64, Everything},
};
use frame_system::EnsureSigned;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_keystore::{testing::MemoryKeystore, KeystoreExt, KeystorePtr};
use sp_runtime::{testing::Header, traits::IdentityLookup, BuildStorage};
use std::{sync::Arc, vec};

pub type AccountId = u64;
pub type Balance = u128;

impl frame_system::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Block = Block;
	type Lookup = IdentityLookup<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type RuntimeTask = ();
	type MaxConsumers = ConstU32<16>;
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

frame_support::ord_parameter_types! {
	pub const One: AccountId = 1;
}

parameter_types! {
	#[derive(Clone, Debug, Eq, PartialEq, TypeInfo)]
	pub const MaxParticipants: u32 = 10;
	#[derive(Clone, Debug, Eq, PartialEq, TypeInfo)]
	pub const MaxSubmissionLen: u32 = 32;
	#[derive(Clone, Debug, Eq, PartialEq, TypeInfo)]
	pub const MaxKeyLen: u32 = 256;
	#[derive(Clone, Debug, Eq, PartialEq, TypeInfo)]
	pub const MaxDataLen: u32 = 256;
	#[derive(Clone, Debug, Eq, PartialEq, TypeInfo)]
	pub const MaxSignatureLen: u32 = 256;
	#[derive(Clone, Debug, Eq, PartialEq, TypeInfo)]
	pub const MaxProofLen: u32 = 256;
	#[derive(Clone, Debug, Eq, PartialEq, TypeInfo)]
	pub const MaxAdditionalParamsLen: u32 = 256;
}

impl pallet_dkg::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UpdateOrigin = EnsureSigned<AccountId>;
	type MaxParticipants = MaxParticipants;
	type MaxSubmissionLen = MaxSubmissionLen;
	type MaxKeyLen = MaxKeyLen;
	type MaxDataLen = MaxDataLen;
	type MaxSignatureLen = MaxSignatureLen;
	type MaxProofLen = MaxProofLen;
	type MaxAdditionalParamsLen = MaxAdditionalParamsLen;
	type WeightInfo = ();
}

pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, RuntimeCall, u64, ()>;

construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Balances: pallet_balances,
		DKG: pallet_dkg,
	}
);

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
	// We use default for brevity, but you can configure as desired if needed.
	pallet_balances::GenesisConfig::<Runtime> { balances: vec![(10, 100), (20, 100)] }
		.assimilate_storage(&mut t)
		.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	// set to block 1 to test events
	ext.execute_with(|| System::set_block_number(1));
	ext.register_extension(KeystoreExt(Arc::new(MemoryKeystore::new()) as KeystorePtr));
	ext
}
