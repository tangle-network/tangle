// This file is part of Webb.
// Copyright (C) 2022 Webb Technologies Inc.
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
#![cfg(test)]

use super::*;
use crate as pallet_jobs;
use frame_support::{
	construct_runtime, ord_parameter_types, parameter_types,
	traits::{ConstU128, ConstU32, ConstU64, Everything},
};
use frame_system::{EnsureSigned, EnsureSignedBy};
use sp_core::H256;
use sp_runtime::{traits::IdentityLookup, BuildStorage};
pub type AccountId = u128;
pub const ALICE: AccountId = 1;
pub type Balance = u128;
pub type BlockNumber = u64;

use tangle_primitives::jobs::*;

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
	type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<10>;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = ();
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxHolds = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
}

pub struct MockDKGPallet;
impl MockDKGPallet {
	fn job_to_fee(job: &JobSubmission<AccountId, BlockNumber>) -> Balance {
		Default::default()
	}

	fn verify(
		job: &JobInfo<AccountId, BlockNumber, Balance>,
		phase_one_data: Option<PhaseOneResult<AccountId, BlockNumber>>,
		result: Vec<u8>,
	) -> DispatchResult {
		Ok(())
	}
}

pub struct MockZkSaasPallet;
impl MockZkSaasPallet {
	fn job_to_fee(job: &JobSubmission<AccountId, BlockNumber>) -> Balance {
		Default::default()
	}

	fn verify(
		job: &JobInfo<AccountId, BlockNumber, Balance>,
		phase_one_data: Option<PhaseOneResult<AccountId, BlockNumber>>,
		result: Vec<u8>,
	) -> DispatchResult {
		Ok(())
	}
}

pub struct MockJobToFeeHandler;

impl JobToFee<AccountId, BlockNumber> for MockJobToFeeHandler {
	type Balance = Balance;

	fn job_to_fee(job: &JobSubmission<AccountId, BlockNumber>) -> Balance {
		Default::default()
	}
}

pub struct MockRolesHandler;

impl RolesHandler<AccountId> for MockRolesHandler {
	fn is_validator(address: AccountId, job_key: JobKey) -> bool {
		let validators = vec![1, 2, 3, 4, 5];
		validators.contains(&address)
	}
}

pub struct MockJobResultVerifier;

impl JobResultVerifier<AccountId, BlockNumber, Balance> for MockJobResultVerifier {
	fn verify(
		job: &JobInfo<AccountId, BlockNumber, Balance>,
		phase_one_data: Option<PhaseOneResult<AccountId, BlockNumber>>,
		result: Vec<u8>,
	) -> DispatchResult {
		Ok(())
	}
}

parameter_types! {
	pub const JobsPalletId: PalletId = PalletId(*b"py/jobss");
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ForceOrigin = EnsureSigned<AccountId>;
	type Currency = Balances;
	type JobToFee = MockJobToFeeHandler;
	type RolesHandler = MockRolesHandler;
	type JobResultVerifier = MockJobResultVerifier;
	type PalletId = JobsPalletId;
	type WeightInfo = ();
}

type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Balances: pallet_balances,
		Jobs: pallet_jobs,
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
	pallet_balances::GenesisConfig::<Runtime>::default()
		.assimilate_storage(&mut t)
		.unwrap();
	t.into()
}
