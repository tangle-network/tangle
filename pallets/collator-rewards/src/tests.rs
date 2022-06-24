// This file is part of Webb.

// Copyright (C) 2021 Webb Technologies Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use crate::{mock::*, Event::RewardAmountSet};
use frame_support::{
	assert_noop, assert_ok,
	error::BadOrigin,
	traits::{Currency, Imbalance},
};

#[test]
fn setting_reward_amount_works() {
	new_test_ext().execute_with(|| {
		// Only root should be able to set value
		assert_noop!(
			CollatorRewards::force_set_reward_amount(Origin::signed(1), 100_u32.into()),
			BadOrigin
		);
		// Root should be able to set value
		assert_ok!(CollatorRewards::force_set_reward_amount(Origin::root(), 100_u32.into()));

		assert_eq!(
			last_event(),
			crate::mock::Event::CollatorRewards(RewardAmountSet(100_u32.into()))
		);
	});
}

#[test]
fn reward_payment_from_pallet_account_works() {
	new_test_ext().execute_with(|| {
		// Add some balance to pallet account
		let reward_amount = 100_u32.into();
		let collator_rewards_account = CollatorRewards::account_id();
		Balances::make_free_balance_be(&collator_rewards_account, reward_amount);
		assert_ok!(CollatorRewards::force_set_reward_amount(Origin::root(), reward_amount));

		let withdrawn_reward_amount = CollatorRewards::withdraw_reward().unwrap();
		assert_eq!(withdrawn_reward_amount.peek(), reward_amount);
		// deposit the reward to an account
		let positive_imb = Balances::deposit_creating(&1, withdrawn_reward_amount.peek());
		withdrawn_reward_amount.offset(positive_imb);

		assert_eq!(Balances::total_issuance(), reward_amount);
	});
}

#[test]
fn issuance_happens_when_pallet_account_exhausted() {
	new_test_ext().execute_with(|| {
		// Add some balance to pallet account
		let reward_amount = 100_u32.into();
		let collator_rewards_account = CollatorRewards::account_id();
		Balances::make_free_balance_be(&collator_rewards_account, reward_amount);
		assert_ok!(CollatorRewards::force_set_reward_amount(Origin::root(), reward_amount));

		let withdrawn_reward_amount = CollatorRewards::withdraw_reward().unwrap();
		assert_eq!(withdrawn_reward_amount.peek(), reward_amount);
		let positive_imb = Balances::deposit_creating(&1, withdrawn_reward_amount.peek());
		withdrawn_reward_amount.offset(positive_imb);

		// we have already exhausted the pallet reward amount, then next call should issue new
		// tokens
		let withdrawn_reward_amount = CollatorRewards::withdraw_reward().unwrap();
		assert_eq!(withdrawn_reward_amount.peek(), reward_amount);

		assert_eq!(Balances::total_issuance(), reward_amount * 2);
	});
}
