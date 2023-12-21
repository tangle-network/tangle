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

use super::*;
use crate::{self as pallet_jobs, mock_evm::address_build};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU128, ConstU32, ConstU64, Everything},
};
use frame_system::EnsureSigned;
use sp_core::H256;
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
pub type Balance = u128;
pub type BlockNumber = u64;

use sp_core::ecdsa;
use sp_io::crypto::ecdsa_generate;
use sp_keystore::{testing::MemoryKeystore, KeystoreExt, KeystorePtr};
use sp_std::sync::Arc;
use tangle_primitives::{
	currency::UNIT,
	jobs::*,
	roles::{RoleTypeMetadata, TssRoleMetadata},
};

/// Key type for DKG keys
pub const KEY_TYPE: sp_application_crypto::KeyTypeId = sp_application_crypto::KeyTypeId(*b"wdkg");

impl frame_system::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId32;
	type Block = Block;
	type Lookup = IdentityLookup<Self::AccountId>;
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
	fn job_to_fee(job: &JobSubmission<AccountId32, BlockNumber>) -> Balance {
		if job.job_type.is_phase_one() {
			job.job_type.clone().get_participants().unwrap().len().try_into().unwrap()
		} else {
			20
		}
	}
}

pub struct MockZkSaasPallet;
impl MockZkSaasPallet {
	fn job_to_fee(job: &JobSubmission<AccountId32, BlockNumber>) -> Balance {
		if job.job_type.is_phase_one() {
			10
		} else {
			20
		}
	}
}

pub struct MockJobToFeeHandler;

impl JobToFee<AccountId32, BlockNumber> for MockJobToFeeHandler {
	type Balance = Balance;

	fn job_to_fee(job: &JobSubmission<AccountId32, BlockNumber>) -> Balance {
		match job.job_type {
			JobType::DKGTSSPhaseOne(_) => MockDKGPallet::job_to_fee(job),
			JobType::DKGTSSPhaseTwo(_) => MockDKGPallet::job_to_fee(job),
			JobType::ZkSaaSPhaseOne(_) => MockZkSaasPallet::job_to_fee(job),
			JobType::ZkSaaSPhaseTwo(_) => MockZkSaasPallet::job_to_fee(job),
		}
	}
}

pub struct MockRolesHandler;

impl RolesHandler<AccountId32> for MockRolesHandler {
	fn is_validator(address: AccountId32, _role_type: JobKey) -> bool {
		let validators = [
			AccountId32::new([1u8; 32]),
			AccountId32::new([2u8; 32]),
			AccountId32::new([3u8; 32]),
			AccountId32::new([4u8; 32]),
			AccountId32::new([5u8; 32]),
		];
		validators.contains(&address)
	}

	fn report_offence(_offence_report: ReportValidatorOffence<AccountId32>) -> DispatchResult {
		Ok(())
	}

	fn get_validator_metadata(address: AccountId32, _job_key: JobKey) -> Option<RoleTypeMetadata> {
		let mock_err_account = AccountId32::new([100u8; 32]);
		if address == mock_err_account {
			None
		} else {
			Some(RoleTypeMetadata::Tss(TssRoleMetadata {
				key_type: DkgKeyType::Ecdsa,
				authority_key: mock_pub_key().to_raw_vec(),
			}))
		}
	}
}

pub struct MockMPCHandler;

impl MPCHandler<AccountId32, BlockNumber, Balance> for MockMPCHandler {
	fn verify(_data: JobWithResult<AccountId32>) -> DispatchResult {
		Ok(())
	}

	fn verify_validator_report(
		_validator: AccountId32,
		_offence: ValidatorOffenceType,
		_signatures: Vec<Vec<u8>>,
	) -> DispatchResult {
		Ok(())
	}

	fn validate_authority_key(_validator: AccountId32, _authority_key: Vec<u8>) -> DispatchResult {
		Ok(())
	}
}

parameter_types! {
	pub const JobsPalletId: PalletId = PalletId(*b"py/jobss");
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ForceOrigin = EnsureSigned<AccountId32>;
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
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Jobs: pallet_jobs,
		EVM: pallet_evm,
		Ethereum: pallet_ethereum,
	}
);

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

pub fn to_account_id32(id: u8) -> AccountId32 {
	AccountId32::new([id; 32])
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

	let pairs = (0..10).map(|i| address_build(i as u8)).collect::<Vec<_>>();

	let initial_balance: u128 = 10 * UNIT;
	let balances: Vec<(AccountId32, u128)> =
		(0..10).map(|i| (pairs[i].account_id.clone(), initial_balance)).collect();

	// We use default for brevity, but you can configure as desired if needed.
	pallet_balances::GenesisConfig::<Runtime> {
		balances: [(to_account_id32(10), 100u128), (to_account_id32(20), 100u128)]
			.iter()
			.cloned()
			.chain(balances.iter().cloned())
			.collect::<Vec<(AccountId32, u128)>>(),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	// set to block 1 to test events
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext.register_extension(KeystoreExt(Arc::new(MemoryKeystore::new()) as KeystorePtr));
	ext
}

fn mock_pub_key() -> ecdsa::Public {
	ecdsa_generate(KEY_TYPE, None)
}
