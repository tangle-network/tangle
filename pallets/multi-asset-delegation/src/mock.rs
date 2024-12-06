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
use crate as pallet_multi_asset_delegation;
use frame_support::{
	derive_impl, parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU16, ConstU32, ConstU64},
	PalletId,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;
pub type Balance = u64;
pub type AssetId = u32;

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DAVE: u64 = 4;
pub const EVE: u64 = 5;

pub const VDOT: AssetId = 1;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Assets: pallet_assets,
		MultiAssetDelegation: pallet_multi_asset_delegation,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU64<1>;
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

pub struct MockServiceManager;

impl tangle_primitives::ServiceManager<u64, Balance> for MockServiceManager {
	fn get_active_blueprints_count(_account: &u64) -> usize {
		// we dont care
		Default::default()
	}

	fn get_active_services_count(_account: &u64) -> usize {
		// we dont care
		Default::default()
	}

	fn can_exit(_account: &u64) -> bool {
		// Mock logic to determine if the given account can exit
		true
	}

	fn get_blueprints_by_operator(_account: &u64) -> Vec<u64> {
		todo!(); // we dont care
	}
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaxLocks: u32 = 50;
	pub const MinOperatorBondAmount: u64 = 10_000;
	pub const BondDuration: u32 = 10;
	pub PID: PalletId = PalletId(*b"PotStake");
	pub const SlashedAmountRecipient : u64 = 0;
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
	pub const NativeAssetId: AssetId = 0;
	pub const BlocksPerMonth: u32 = 30 * 24 * 60 * 12;
}

impl pallet_multi_asset_delegation::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type AssetId = u32;
	type NativeAssetId = NativeAssetId;
	type BlocksPerMonth = BlocksPerMonth;
	type MinOperatorBondAmount = MinOperatorBondAmount;
	type BondDuration = BondDuration;
	type ServiceManager = MockServiceManager;
	type LeaveOperatorsDelay = ConstU32<10>;
	type OperatorBondLessDelay = ConstU32<1>;
	type LeaveDelegatorsDelay = ConstU32<1>;
	type DelegationBondLessDelay = ConstU32<5>;
	type MinDelegateAmount = ConstU64<100>;
	type Fungibles = Assets;
	type VaultId = AssetId;
	type ForceOrigin = frame_system::EnsureRoot<u64>;
	type PalletId = PID;
	type MaxDelegatorBlueprints = MaxDelegatorBlueprints;
	type MaxOperatorBlueprints = MaxOperatorBlueprints;
	type MaxWithdrawRequests = MaxWithdrawRequests;
	type MaxUnstakeRequests = MaxUnstakeRequests;
	type MaxDelegations = MaxDelegations;
	type SlashedAmountRecipient = SlashedAmountRecipient;
	type WeightInfo = ();
}

impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u64;
	type AssetId = AssetId;
	type AssetIdParameter = u32;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<u64>>;
	type ForceOrigin = frame_system::EnsureRoot<u64>;
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

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(ALICE, 100_000),
			(BOB, 200_000),
			(CHARLIE, 300_000),
			(DAVE, 5_000), // Not enough to stake
			(MultiAssetDelegation::pallet_account(), 100), /* give pallet some ED so it can
			                * receive tokens */
			(EVE, 20_000),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
