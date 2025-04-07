// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

use super::*;
use crate::{
	Event,
	mock::{Currency, *},
};
use frame_support::traits::Currency as CurrencyT;

mod bond_extra;
mod bonded_pool;
mod create;
mod join;
mod slash;
mod sub_pools;
mod update_roles;

pub const DEFAULT_ROLES: PoolRoles<AccountId> =
	PoolRoles { depositor: 10, root: Some(900), nominator: Some(901), bouncer: Some(902) };

fn mint_lst(pool_id: u32, who: &AccountId, amount: u128) {
	if Assets::asset_exists(pool_id) {
		Assets::mint_into(pool_id, who, amount).unwrap();
	} else {
		Balances::make_free_balance_be(who, 10000);
		Assets::force_create(RuntimeOrigin::root(), pool_id, *who, false, 1_u32.into()).unwrap();
		Assets::mint_into(pool_id, who, amount).unwrap();
	}
}

fn burn_lst(pool_id: u32, who: &AccountId, amount: u128) {
	Assets::burn_from(
		pool_id,
		who,
		amount,
		Preservation::Expendable,
		Precision::Exact,
		Fortitude::Force,
	)
	.unwrap();
}

#[test]
fn test_setup_works() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(BondedPools::<Runtime>::count(), 1);
		assert_eq!(RewardPools::<Runtime>::count(), 1);
		assert_eq!(SubPoolsStorage::<Runtime>::count(), 0);
		assert_eq!(UnbondingMembers::<Runtime>::count(), 0);
		assert_eq!(StakingMock::bonding_duration(), 3);
		assert!(Metadata::<T>::contains_key(1));

		// initial member.
		assert_eq!(TotalValueLocked::<T>::get(), 10);

		let last_pool = LastPoolId::<Runtime>::get();
		let bonded_pool = BondedPool::<Runtime>::get(last_pool).unwrap();

		assert_eq!(bonded_pool.id, last_pool);

		let inner = bonded_pool.inner;
		assert_eq!(inner.commission, Commission::default());
		assert_eq!(inner.roles, DEFAULT_ROLES);
		assert_eq!(inner.state, PoolState::Open);

		let bonded_account = Lst::create_bonded_account(last_pool);
		let reward_account = Lst::create_reward_account(last_pool);

		// the bonded_account should be bonded by the depositor's funds.
		assert_eq!(StakingMock::active_stake(&bonded_account).unwrap(), 10);
		assert_eq!(StakingMock::total_stake(&bonded_account).unwrap(), 10);

		// but not nominating yet.
		assert!(Nominations::get().is_none());

		// reward account should have an initial ED in it.
		assert_eq!(
			Currency::free_balance(reward_account),
			<Balances as CurrencyT<AccountId>>::minimum_balance()
		);
	})
}
