// This file is part of Tangle.
// Copyright (C) 2022-2023 Webb Technologies Inc.
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
	construct_runtime, parameter_types,
	traits::{ConstU128, ConstU32, ConstU64, Everything},
};
use frame_system::EnsureSigned;
use sp_core::H256;
use sp_runtime::{traits::IdentityLookup, BuildStorage};
pub type AccountId = u128;
pub type Balance = u128;
pub type BlockNumber = u64;

use tangle_primitives::{jobs::*};

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
	type ExistentialDeposit = ConstU128<1>;
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
		if job.job_type.is_phase_one() {
			job.job_type.clone().get_participants().unwrap().len().try_into().unwrap()
		} else {
			20
		}
	}

	fn verify(
		_job: &JobInfo<AccountId, BlockNumber, Balance>,
		_phase_one_data: Option<PhaseOneResult<AccountId, BlockNumber>>,
		_result: Vec<u8>,
	) -> DispatchResult {
		Ok(())
	}
}

pub struct MockZkSaasPallet;
impl MockZkSaasPallet {
	fn job_to_fee(job: &JobSubmission<AccountId, BlockNumber>) -> Balance {
		if job.job_type.is_phase_one() {
			10
		} else {
			20
		}
	}

	fn verify(
		_job: &JobInfo<AccountId, BlockNumber, Balance>,
		_phase_one_data: Option<PhaseOneResult<AccountId, BlockNumber>>,
		_result: Vec<u8>,
	) -> DispatchResult {
		Ok(())
	}
}

pub struct MockJobToFeeHandler;

impl JobToFee<AccountId, BlockNumber> for MockJobToFeeHandler {
	type Balance = Balance;

	fn job_to_fee(job: &JobSubmission<AccountId, BlockNumber>) -> Balance {
		match job.job_type {
			JobType::DKG(_) => MockDKGPallet::job_to_fee(job),
			JobType::DKGSignature(_) => MockDKGPallet::job_to_fee(job),
			JobType::ZkSaasPhaseOne(_) => MockZkSaasPallet::job_to_fee(job),
			JobType::ZkSaasPhaseTwo(_) => MockZkSaasPallet::job_to_fee(job),
		}
	}
}

pub struct MockRolesHandler;

impl RolesHandler<AccountId> for MockRolesHandler {
	fn is_validator(address: AccountId, _role_type: JobKey) -> bool {
		let validators = [1, 2, 3, 4, 5];
		validators.contains(&address)
	}

	fn slash_validator(_address: AccountId, _offence: ValidatorOffence) -> DispatchResult {
		Ok(())
	}
}

pub struct MockMPCHandler;

impl MPCHandler<AccountId, BlockNumber, Balance> for MockMPCHandler {
	fn verify(
		job: &JobInfo<AccountId, BlockNumber, Balance>,
		phase_one_data: Option<PhaseOneResult<AccountId, BlockNumber>>,
		result: Vec<u8>,
	) -> DispatchResult {
		match job.job_type {
			JobType::DKG(_) => MockDKGPallet::verify(job, phase_one_data, result),
			JobType::DKGSignature(_) => MockDKGPallet::verify(job, phase_one_data, result),
			JobType::ZkSaasPhaseOne(_) => MockZkSaasPallet::verify(job, phase_one_data, result),
			JobType::ZkSaasPhaseTwo(_) => MockZkSaasPallet::verify(job, phase_one_data, result),
		}
	}

	fn verify_validator_report(
		_validator: AccountId,
		_offence: ValidatorOffence,
		_report: Vec<u8>,
	) -> DispatchResult {
		Ok(())
	}

	fn validate_authority_key(_validator: AccountId, _authority_key: Vec<u8>) -> DispatchResult {
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
	type MPCHandler = MockMPCHandler;
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
	pallet_balances::GenesisConfig::<Runtime> { balances: vec![(10, 100), (20, 100)] }
		.assimilate_storage(&mut t)
		.unwrap();

	t.into()
}
