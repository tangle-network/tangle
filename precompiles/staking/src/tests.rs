// Copyright 2022 Webb Technologies Inc.
//
// This file is part of pallet-evm-precompile-staking package, originally developed by Purestake
// Inc. Pallet-evm-precompile-staking package used in Tangle Network in terms of GPLv3.

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

use crate::{
	mock::{
		active_era, mock_pub_key, new_test_ext, start_session, PCall, Precompiles,
		PrecompilesValue, Runtime, TestAccount,
	},
	StakingPrecompile,
};
use pallet_evm::AddressMapping;
use pallet_staking::Nominators;
use precompile_utils::testing::PrecompileTesterExt;
use sp_core::{H160, H256, U256};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn max_validator_count_works() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(5),
				PCall::max_validator_count {},
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(5u32)
	});
}

#[test]
fn current_era_works() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		start_session(3);
		assert_eq!(active_era(), 1);
		precompiles()
			.prepare_test(TestAccount::Alex, H160::from_low_u64_be(5), PCall::current_era {})
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(1u32);
	});
}

#[test]
fn validator_count_works() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		precompiles()
			.prepare_test(TestAccount::Alex, H160::from_low_u64_be(5), PCall::validator_count {})
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(2u32);
	});
}

#[test]
fn max_nominator_count_works() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(5),
				PCall::max_nominator_count {},
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(5u32);
	});
}

#[test]
fn is_validator_works() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(5),
				PCall::is_validator { validator: H160::from(TestAccount::Alex).into() },
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(true);
	});
}

#[test]
fn eras_total_rewards_should_work() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		start_session(4);
		let era_index = active_era();
		crate::mock::Staking::reward_by_ids(vec![(TestAccount::Alex.into(), 50)]);
		crate::mock::Staking::reward_by_ids(vec![(TestAccount::Bobo.into(), 50)]);
		crate::mock::Staking::reward_by_ids(vec![(TestAccount::Dino.into(), 50)]);
		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(5),
				PCall::eras_total_reward_points { era_index },
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(150u32);
	});
}

#[test]
fn nominate_should_work() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		assert_eq!(
			Nominators::<Runtime>::get(sp_core::sr25519::Public::from(TestAccount::Alex)),
			None
		);
		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(5),
				PCall::nominate { targets: vec![H256::from(mock_pub_key(1)).into()] },
			)
			.expect_no_logs()
			.execute_returns(());

		let nominator =
			Nominators::<Runtime>::get(sp_core::sr25519::Public::from(TestAccount::Alex)).unwrap();
		assert_eq!(nominator.targets.len(), 1);
		assert_eq!(nominator.targets[0], mock_pub_key(1));
	});
}

#[test]
fn bond_should_work() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(5),
				PCall::bond {
					value: U256::from(100),
					payee: H256([
						0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
						0, 0, 0, 0, 0, 0, 1,
					]),
				},
			)
			.expect_no_logs()
			.execute_returns(());

		let nominator =
			Nominators::<Runtime>::get(sp_core::sr25519::Public::from(TestAccount::Alex)).unwrap();
		assert_eq!(nominator.targets.len(), 1);
		assert_eq!(nominator.targets[0], mock_pub_key(1));
	});
}
