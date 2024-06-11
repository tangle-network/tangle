// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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
	construct_runtime, parameter_types, traits::Everything, weights::Weight, PalletId,
};
use frame_system::EnsureSigned;
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::{precompile_set::*, testing::MockAccount};
use scale_info::TypeInfo;
use sp_core::{H256, U256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage, DispatchResult, Perbill,
};
use tangle_primitives::{
	jobs::{
		traits::{JobToFee, MPCHandler},
		*,
	},
	misbehavior::{MisbehaviorHandler, MisbehaviorSubmission},
	roles::traits::RolesHandler,
};

pub type AccountId = MockAccount;
pub type Balance = u128;
pub type BlockNumber = u64;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		Timestamp: pallet_timestamp,
		Jobs: pallet_jobs,
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const MaximumBlockWeight: Weight = Weight::from_parts(1024, 1);
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type Block = Block;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type RuntimeTask = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}
parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
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

pub type Precompiles<R> =
	PrecompileSetBuilder<R, (PrecompileAt<AddressU64<1>, JobsPrecompile<R>>,)>;

pub type PCall = JobsPrecompileCall<Runtime>;

parameter_types! {
	pub SuicideQuickClearLimit: u32 = 0;
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
	type PrecompilesType = Precompiles<Runtime>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
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

pub struct MockDKGPallet;
impl MockDKGPallet {
	fn job_to_fee(
		job: &JobSubmission<
			AccountId,
			BlockNumber,
			MaxParticipants,
			MaxSubmissionLen,
			MaxAdditionalParamsLen,
		>,
	) -> Balance {
		if job.job_type.is_phase_one() {
			job.job_type.clone().get_participants().unwrap().len().try_into().unwrap()
		} else {
			20
		}
	}
}

pub struct MockZkSaasPallet;
impl MockZkSaasPallet {
	fn job_to_fee(
		job: &JobSubmission<
			AccountId,
			BlockNumber,
			MaxParticipants,
			MaxSubmissionLen,
			MaxAdditionalParamsLen,
		>,
	) -> Balance {
		if job.job_type.is_phase_one() {
			10
		} else {
			20
		}
	}
}

pub struct MockJobToFeeHandler;

impl JobToFee<AccountId, BlockNumber, MaxParticipants, MaxSubmissionLen, MaxAdditionalParamsLen>
	for MockJobToFeeHandler
{
	type Balance = Balance;

	fn job_to_fee(
		job: &JobSubmission<
			AccountId,
			BlockNumber,
			MaxParticipants,
			MaxSubmissionLen,
			MaxAdditionalParamsLen,
		>,
	) -> Balance {
		match job.job_type {
			JobType::DKGTSSPhaseOne(_)
			| JobType::DKGTSSPhaseTwo(_)
			| JobType::DKGTSSPhaseThree(_)
			| JobType::DKGTSSPhaseFour(_) => MockDKGPallet::job_to_fee(job),
			JobType::ZkSaaSPhaseOne(_) | JobType::ZkSaaSPhaseTwo(_) => {
				MockZkSaasPallet::job_to_fee(job)
			},
		}
	}

	fn calculate_result_extension_fee(_result: Vec<u8>, _extension_time: BlockNumber) -> Balance {
		Default::default()
	}
}


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