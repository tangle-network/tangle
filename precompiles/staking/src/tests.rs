// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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

use crate::mock::{
	active_era, make_all_reward_payment, mock_pub_key, new_test_ext, reward_all_elected,
	start_active_era, start_session, Balances, PCall, Precompiles, PrecompilesValue, Runtime,
	TestAccount,
};
use frame_support::traits::Currency;
use pallet_staking::Nominators;
use precompile_utils::testing::PrecompileTesterExt;
use sp_core::{H160, H256, U256};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

pub fn convert_h160_to_h256(address: H160) -> H256 {
	let mut bytes = [0u8; 32];
	bytes[12..32].copy_from_slice(&address.0);
	H256(bytes)
}

#[test]
fn max_validator_count_works() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
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
			.prepare_test(TestAccount::Alex, H160::from_low_u64_be(1), PCall::current_era {})
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(1u32);
	});
}

#[test]
fn validator_count_works() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		precompiles()
			.prepare_test(TestAccount::Alex, H160::from_low_u64_be(1), PCall::validator_count {})
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(4u32);
	});
}

#[test]
fn max_nominator_count_works() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
				PCall::max_nominator_count {},
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(5u32);
	});
}

#[test]
fn eras_total_rewards_should_work() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		start_session(4);
		let era_index = active_era();
		crate::mock::Staking::reward_by_ids(vec![(TestAccount::Alex.into(), 50)]);
		crate::mock::Staking::reward_by_ids(vec![(TestAccount::Bobo.into(), 50)]);
		crate::mock::Staking::reward_by_ids(vec![(TestAccount::Charlie.into(), 50)]);
		precompiles()
			.prepare_test(
				TestAccount::Alex,
				H160::from_low_u64_be(1),
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
				H160::from_low_u64_be(1),
				PCall::nominate { targets: vec![H256::from(mock_pub_key(1))] },
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
		Balances::make_free_balance_be(&TestAccount::Eve.into(), 1000);

		precompiles()
			.prepare_test(
				TestAccount::Eve,
				H160::from_low_u64_be(1),
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
	});
}

// Payout to stash account should work
#[test]
fn nominator_payout_to_stash_account_should_work() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		let account = sp_core::sr25519::Public::from(TestAccount::Eve);

		assert_eq!(Balances::total_balance(&account), 0);

		// Lets give some tokens to account for bonding and nominating.
		Balances::make_free_balance_be(&TestAccount::Eve.into(), 1_000_000_000u128);

		// Lets Bond token with reward destination to Stash account
		precompiles()
			.prepare_test(
				TestAccount::Eve,
				H160::from_low_u64_be(1),
				PCall::bond {
					value: U256::from(1_000_000_000),
					payee: H256([
						0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
						0, 0, 0, 0, 0, 0, 2,
					]),
				},
			)
			.expect_no_logs()
			.execute_returns(());

		assert_eq!(Nominators::<Runtime>::get(account), None);

		// Now lets nominate [Bobo,Dave,Charlie] for above account
		let bob_address = sp_core::sr25519::Public::from(TestAccount::Bobo);
		let dave_address = sp_core::sr25519::Public::from(TestAccount::Dave);
		let charlie_address = sp_core::sr25519::Public::from(TestAccount::Charlie);

		precompiles()
			.prepare_test(
				TestAccount::Eve,
				H160::from_low_u64_be(1),
				PCall::nominate {
					targets: vec![
						H256::from(bob_address),
						H256::from(dave_address),
						H256::from(charlie_address),
					],
				},
			)
			.expect_no_logs()
			.execute_returns(());
		let nominator = Nominators::<Runtime>::get(account).unwrap();
		assert_eq!(nominator.targets.len(), 3);

		// Nominator will be added in next era (era1).
		// Nominator should have 0 usable balance.
		assert_eq!(Balances::usable_balance(account), 0);

		// Reward payment for era 0.
		// All the old validators should be rewarded.
		reward_all_elected();
		start_active_era(1);
		make_all_reward_payment(0);

		// Reward payment for era 1.
		// Now the nominator should receive some rewards
		reward_all_elected();
		start_active_era(2);
		make_all_reward_payment(1);

		// Stash acount is same as controller account.
		// Therefore should receive some usable balance in same controller account.
		assert_eq!(Balances::usable_balance(account), 10);
	});
}

// Payout to evm account should work
#[test]
fn nominator_payout_to_evm_account_should_work() {
	new_test_ext(vec![1, 2, 3, 4]).execute_with(|| {
		// We will be using this account to receive rewards
		let evm_address = H160::repeat_byte(0x00);
		let reward_address = convert_h160_to_h256(evm_address);
		let mapped_substrate_account = sp_core::sr25519::Public::from(TestAccount::Empty);
		assert_eq!(Balances::total_balance(&mapped_substrate_account), 0);

		let account = sp_core::sr25519::Public::from(TestAccount::Eve);
		assert_eq!(Balances::total_balance(&account), 0);

		// Lets give some tokens to account for bonding and nominating.
		Balances::make_free_balance_be(&TestAccount::Eve.into(), 1_000_000_000u128);

		// Lets Bond token with reward destination to above evm address
		precompiles()
			.prepare_test(
				TestAccount::Eve,
				H160::from_low_u64_be(1),
				PCall::bond { value: U256::from(1_000_000_000), payee: reward_address },
			)
			.expect_no_logs()
			.execute_returns(());

		assert_eq!(Nominators::<Runtime>::get(account), None);

		// Now lets nominate [Bobo,Dave,Charlie] for above account
		let bob_address = sp_core::sr25519::Public::from(TestAccount::Bobo);
		let dave_address = sp_core::sr25519::Public::from(TestAccount::Dave);
		let charlie_address = sp_core::sr25519::Public::from(TestAccount::Charlie);

		precompiles()
			.prepare_test(
				TestAccount::Eve,
				H160::from_low_u64_be(1),
				PCall::nominate {
					targets: vec![
						H256::from(bob_address),
						H256::from(dave_address),
						H256::from(charlie_address),
					],
				},
			)
			.expect_no_logs()
			.execute_returns(());
		let nominator = Nominators::<Runtime>::get(account).unwrap();
		assert_eq!(nominator.targets.len(), 3);

		// Nominator will be added in next era (era1).
		// Nominator should have 0 usable balance.
		assert_eq!(Balances::usable_balance(account), 0);

		// Reward payment for era 0.
		// All the old validators should be rewarded.
		reward_all_elected();
		start_active_era(1);
		make_all_reward_payment(0);

		// Reward payment for era 1.
		// Now the nominator should receive some rewards
		reward_all_elected();
		start_active_era(2);
		make_all_reward_payment(1);

		// Mapped substrate account for above evm address which is used as reward destination.
		assert_eq!(Balances::usable_balance(mapped_substrate_account), 10);
	});
}
