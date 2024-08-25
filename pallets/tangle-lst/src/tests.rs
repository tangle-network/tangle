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
use crate::{mock::Currency, mock::*, Event};
use frame_support::traits::Currency as CurrencyT;
use frame_support::{assert_err, assert_noop, assert_ok, assert_storage_noop};
use pallet_balances::Event as BEvent;
use sp_runtime::{bounded_btree_map, traits::Dispatchable, FixedU128};

mod bonded_pool;
mod create;
mod join;
mod slash;
mod sub_pools;
mod unbond;
mod withdraw_unbonded;

macro_rules! unbonding_pools_with_era {
	($($k:expr => $v:expr),* $(,)?) => {{
		use sp_std::iter::{Iterator, IntoIterator};
		let not_bounded: BTreeMap<_, _> = Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]));
		BoundedBTreeMap::<EraIndex, UnbondPool<T>, TotalUnbondingPools<T>>::try_from(not_bounded).unwrap()
	}};
}

macro_rules! member_unbonding_eras {
	($( $any:tt )*) => {{
		let x: BoundedBTreeMap<EraIndex, Balance, MaxUnbonding> = bounded_btree_map!($( $any )*);
		x
	}};
}

pub const DEFAULT_ROLES: PoolRoles<AccountId> =
	PoolRoles { depositor: 10, root: Some(900), nominator: Some(901), bouncer: Some(902) };

fn deposit_rewards(r: u128) {
	let b = Currency::free_balance(&default_reward_account()).checked_add(r).unwrap();
	Currency::set_balance(&default_reward_account(), b);
}

fn remove_rewards(r: u128) {
	let b = Currency::free_balance(&default_reward_account()).checked_sub(r).unwrap();
	Currency::set_balance(&default_reward_account(), b);
}

fn mint_lst(pool_id: u32, who: &AccountId, amount: u128) {
	if Assets::asset_exists(pool_id) {
		Assets::mint_into(pool_id, who, amount).unwrap();
	} else {
		Balances::make_free_balance_be(&who, 10000);
		Assets::force_create(RuntimeOrigin::root(), pool_id, *who, false, 1_u32.into()).unwrap();
		Assets::mint_into(pool_id, who, amount).unwrap();
	}
}

fn burn_lst(pool_id: u32, who: &AccountId, amount: u128) {
	Assets::burn_from(pool_id, who, amount, Precision::Exact, Fortitude::Force).unwrap();
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
		assert_eq!(
			BondedPool::<Runtime>::get(last_pool).unwrap(),
			BondedPool::<Runtime> {
				id: last_pool,
				inner: BondedPoolInner {
					commission: Commission::default(),
					roles: DEFAULT_ROLES,
					state: PoolState::Open,
				},
			}
		);
		assert_eq!(
			RewardPools::<Runtime>::get(last_pool).unwrap(),
			RewardPool::<Runtime> {
				last_recorded_reward_counter: Zero::zero(),
				last_recorded_total_payouts: 0,
				total_rewards_claimed: 0,
				total_commission_claimed: 0,
				total_commission_pending: 0,
			}
		);

		let bonded_account = Lst::create_bonded_account(last_pool);
		let reward_account = Lst::create_reward_account(last_pool);

		// the bonded_account should be bonded by the depositor's funds.
		assert_eq!(StakingMock::active_stake(&bonded_account).unwrap(), 10);
		assert_eq!(StakingMock::total_stake(&bonded_account).unwrap(), 10);

		// but not nominating yet.
		assert!(Nominations::get().is_none());

		// reward account should have an initial ED in it.
		assert_eq!(
			Currency::free_balance(&reward_account),
			<Balances as CurrencyT<AccountId>>::minimum_balance()
		);
	})
}

// mod claim_payout {
// 	use super::*;

// 	fn del(points: Balance, last_recorded_reward_counter: u128) -> PoolMember<Runtime> {
// 		PoolMember {
// 			pool_id: 1,
// 			points,
// 			last_recorded_reward_counter: last_recorded_reward_counter.into(),
// 			unbonding_eras: Default::default(),
// 		}
// 	}

// 	fn del_float(points: Balance, last_recorded_reward_counter: f64) -> PoolMember<Runtime> {
// 		PoolMember {
// 			pool_id: 1,
// 			points,
// 			last_recorded_reward_counter: RewardCounter::from_float(last_recorded_reward_counter),
// 			unbonding_eras: Default::default(),
// 		}
// 	}

// 	fn rew(
// 		last_recorded_reward_counter: u128,
// 		last_recorded_total_payouts: Balance,
// 		total_rewards_claimed: Balance,
// 	) -> RewardPool<Runtime> {
// 		RewardPool {
// 			last_recorded_reward_counter: last_recorded_reward_counter.into(),
// 			last_recorded_total_payouts,
// 			total_rewards_claimed,
// 			total_commission_claimed: 0,
// 			total_commission_pending: 0,
// 		}
// 	}

// 	#[test]
// 	fn claim_payout_works() {
// 		ExtBuilder::default()
// 			.add_members(vec![(40, 40), (50, 50)])
// 			.build_and_execute(|| {
// 				// Given each member currently has a free balance of
// 				Currency::set_balance(&10, 0);
// 				Currency::set_balance(&40, 0);
// 				Currency::set_balance(&50, 0);
// 				let ed = Currency::minimum_balance();

// 				// and the reward pool has earned 100 in rewards
// 				assert_eq!(Currency::free_balance(&default_reward_account()), ed);
// 				deposit_rewards(100);

// 				let _ = pool_events_since_last_call();

// 				// When
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 10, pool_id: 1, payout: 10 },]
// 				);
// 				// last recorded reward counter at the time of this member's payout is 1
// 				assert_eq!(PoolMembers::<Runtime>::get(10).unwrap(), del(10, 1));
// 				// pool's 'last_recorded_reward_counter' and 'last_recorded_total_payouts' don't
// 				// really change unless if someone bonds/unbonds.
// 				assert_eq!(RewardPools::<Runtime>::get(1).unwrap(), rew(0, 0, 10));
// 				assert_eq!(Currency::free_balance(&10), 10);
// 				assert_eq!(Currency::free_balance(&default_reward_account()), ed + 90);

// 				// When
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(40)));

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 40, pool_id: 1, payout: 40 }]
// 				);
// 				assert_eq!(PoolMembers::<Runtime>::get(40).unwrap(), del(40, 1));
// 				assert_eq!(RewardPools::<Runtime>::get(1).unwrap(), rew(0, 0, 50));
// 				assert_eq!(Currency::free_balance(&40), 40);
// 				assert_eq!(Currency::free_balance(&default_reward_account()), ed + 50);

// 				// When
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(50)));

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 50, pool_id: 1, payout: 50 }]
// 				);
// 				assert_eq!(PoolMembers::<Runtime>::get(50).unwrap(), del(50, 1));
// 				assert_eq!(RewardPools::<Runtime>::get(1).unwrap(), rew(0, 0, 100));
// 				assert_eq!(Currency::free_balance(&50), 50);
// 				assert_eq!(Currency::free_balance(&default_reward_account()), ed);

// 				// Given the reward pool has some new rewards
// 				deposit_rewards(50);

// 				// When
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 10, pool_id: 1, payout: 5 }]
// 				);
// 				assert_eq!(PoolMembers::<Runtime>::get(10).unwrap(), del_float(10, 1.5));
// 				assert_eq!(RewardPools::<Runtime>::get(1).unwrap(), rew(0, 0, 105));
// 				assert_eq!(Currency::free_balance(&10), 10 + 5);
// 				assert_eq!(Currency::free_balance(&default_reward_account()), ed + 45);

// 				// When
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(40)));

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 40, pool_id: 1, payout: 20 }]
// 				);
// 				assert_eq!(PoolMembers::<Runtime>::get(40).unwrap(), del_float(40, 1.5));
// 				assert_eq!(RewardPools::<Runtime>::get(1).unwrap(), rew(0, 0, 125));
// 				assert_eq!(Currency::free_balance(&40), 40 + 20);
// 				assert_eq!(Currency::free_balance(&default_reward_account()), ed + 25);

// 				// Given del 50 hasn't claimed and the reward pools has just earned 50
// 				deposit_rewards(50);
// 				assert_eq!(Currency::free_balance(&default_reward_account()), ed + 75);

// 				// When
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(50)));

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 50, pool_id: 1, payout: 50 }]
// 				);
// 				assert_eq!(PoolMembers::<Runtime>::get(50).unwrap(), del_float(50, 2.0));
// 				assert_eq!(RewardPools::<Runtime>::get(1).unwrap(), rew(0, 0, 175));
// 				assert_eq!(Currency::free_balance(&50), 50 + 50);
// 				assert_eq!(Currency::free_balance(&default_reward_account()), ed + 25);

// 				// When
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 10, pool_id: 1, payout: 5 }]
// 				);
// 				assert_eq!(PoolMembers::<Runtime>::get(10).unwrap(), del(10, 2));
// 				assert_eq!(RewardPools::<Runtime>::get(1).unwrap(), rew(0, 0, 180));
// 				assert_eq!(Currency::free_balance(&10), 15 + 5);
// 				assert_eq!(Currency::free_balance(&default_reward_account()), ed + 20);

// 				// Given del 40 hasn't claimed and the reward pool has just earned 400
// 				deposit_rewards(400);
// 				assert_eq!(Currency::free_balance(&default_reward_account()), ed + 420);

// 				// When
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 10, pool_id: 1, payout: 40 }]
// 				);

// 				// We expect a payout of 40
// 				assert_eq!(PoolMembers::<Runtime>::get(10).unwrap(), del(10, 6));
// 				assert_eq!(RewardPools::<Runtime>::get(1).unwrap(), rew(0, 0, 220));
// 				assert_eq!(Currency::free_balance(&10), 20 + 40);
// 				assert_eq!(Currency::free_balance(&default_reward_account()), ed + 380);

// 				// Given del 40 + del 50 haven't claimed and the reward pool has earned 20
// 				deposit_rewards(20);
// 				assert_eq!(Currency::free_balance(&default_reward_account()), ed + 400);

// 				// When
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 10, pool_id: 1, payout: 2 }]
// 				);
// 				assert_eq!(PoolMembers::<Runtime>::get(10).unwrap(), del_float(10, 6.2));
// 				assert_eq!(RewardPools::<Runtime>::get(1).unwrap(), rew(0, 0, 222));
// 				assert_eq!(Currency::free_balance(&10), 60 + 2);
// 				assert_eq!(Currency::free_balance(&default_reward_account()), ed + 398);

// 				// When
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(40)));

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 40, pool_id: 1, payout: 188 }]
// 				);
// 				assert_eq!(PoolMembers::<Runtime>::get(40).unwrap(), del_float(40, 6.2));
// 				assert_eq!(RewardPools::<Runtime>::get(1).unwrap(), rew(0, 0, 410));
// 				assert_eq!(Currency::free_balance(&40), 60 + 188);
// 				assert_eq!(Currency::free_balance(&default_reward_account()), ed + 210);

// 				// When
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(50)));

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 50, pool_id: 1, payout: 210 }]
// 				);
// 				assert_eq!(PoolMembers::<Runtime>::get(50).unwrap(), del_float(50, 6.2));
// 				assert_eq!(RewardPools::<Runtime>::get(1).unwrap(), rew(0, 0, 620));
// 				assert_eq!(Currency::free_balance(&50), 100 + 210);
// 				assert_eq!(Currency::free_balance(&default_reward_account()), ed);
// 			});
// 	}

// 	#[test]
// 	fn reward_payout_errors_if_a_member_is_fully_unbonding() {
// 		ExtBuilder::default().add_members(vec![(11, 11)]).build_and_execute(|| {
// 			// fully unbond the member.
// 			assert_ok!(fully_unbond_permissioned(11));

// 			assert_noop!(
// 				Lst::claim_payout(RuntimeOrigin::signed(11)),
// 				Error::<Runtime>::FullyUnbonding
// 			);

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 11, pool_id: 1, bonded: 11, joined: true },
// 					Event::Unbonded { member: 11, pool_id: 1, points: 11, balance: 11, era: 3 }
// 				]
// 			);
// 		});
// 	}

// 	#[test]
// 	fn claim_payout_bounds_commission_above_global() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			let (mut member, bonded_pool, mut reward_pool) =
// 				Lst::get_member_with_pools(&10).unwrap();

// 			// top up commission payee account to existential deposit
// 			let _ = Currency::set_balance(&2, 5);

// 			// Set a commission pool 1 to 75%, with a payee set to `2`
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				bonded_pool.id,
// 				Some((Perbill::from_percent(75), 2)),
// 			));

// 			// re-introduce the global maximum to 50% - 25% lower than the current commission of the
// 			// pool.
// 			GlobalMaxCommission::<Runtime>::set(Some(Perbill::from_percent(50)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::PoolCommissionUpdated {
// 						pool_id: 1,
// 						current: Some((Perbill::from_percent(75), 2))
// 					}
// 				]
// 			);

// 			// The pool earns 10 points
// 			deposit_rewards(10);

// 			assert_ok!(Lst::do_reward_payout(
// 				&10,
// 				&mut member,
// 				&mut BondedPool::<Runtime>::get(1).unwrap(),
// 				&mut reward_pool
// 			));

// 			// commission applied is 50%, not 75%. Has been bounded by `GlobalMaxCommission`.
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PaidOut { member: 10, pool_id: 1, payout: 5 },]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn do_reward_payout_works_with_a_pool_of_1() {
// 		let del = |last_recorded_reward_counter| del_float(10, last_recorded_reward_counter);

// 		ExtBuilder::default().build_and_execute(|| {
// 			let (mut member, mut bonded_pool, mut reward_pool) =
// 				Lst::get_member_with_pools(&10).unwrap();
// 			let ed = Currency::minimum_balance();

// 			let payout =
// 				Lst::do_reward_payout(&10, &mut member, &mut bonded_pool, &mut reward_pool)
// 					.unwrap();

// 			// Then
// 			assert_eq!(payout, 0);
// 			assert_eq!(member, del(0.0));
// 			assert_eq!(reward_pool, rew(0, 0, 0));
// 			Lst::put_member_with_pools(
// 				&10,
// 				member.clone(),
// 				bonded_pool.clone(),
// 				reward_pool.clone(),
// 			);

// 			// Given the pool has earned some rewards for the first time
// 			deposit_rewards(5);

// 			// When
// 			let payout =
// 				Lst::do_reward_payout(&10, &mut member, &mut bonded_pool, &mut reward_pool)
// 					.unwrap();

// 			// Then
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 5 }
// 				]
// 			);
// 			assert_eq!(payout, 5);
// 			assert_eq!(reward_pool, rew(0, 0, 5));
// 			assert_eq!(member, del(0.5));
// 			Lst::put_member_with_pools(
// 				&10,
// 				member.clone(),
// 				bonded_pool.clone(),
// 				reward_pool.clone(),
// 			);

// 			// Given the pool has earned rewards again
// 			deposit_rewards(10);

// 			// When
// 			let payout =
// 				Lst::do_reward_payout(&10, &mut member, &mut bonded_pool, &mut reward_pool)
// 					.unwrap();

// 			// Then
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PaidOut { member: 10, pool_id: 1, payout: 10 }]
// 			);
// 			assert_eq!(payout, 10);
// 			assert_eq!(reward_pool, rew(0, 0, 15));
// 			assert_eq!(member, del(1.5));
// 			Lst::put_member_with_pools(
// 				&10,
// 				member.clone(),
// 				bonded_pool.clone(),
// 				reward_pool.clone(),
// 			);

// 			// Given the pool has earned no new rewards
// 			Currency::set_balance(&default_reward_account(), ed);

// 			// When
// 			let payout =
// 				Lst::do_reward_payout(&10, &mut member, &mut bonded_pool, &mut reward_pool)
// 					.unwrap();

// 			// Then
// 			assert_eq!(pool_events_since_last_call(), vec![]);
// 			assert_eq!(payout, 0);
// 			assert_eq!(reward_pool, rew(0, 0, 15));
// 			assert_eq!(member, del(1.5));
// 			Lst::put_member_with_pools(
// 				&10,
// 				member.clone(),
// 				bonded_pool.clone(),
// 				reward_pool.clone(),
// 			);
// 		});
// 	}

// 	#[test]
// 	fn do_reward_payout_works_with_a_pool_of_3() {
// 		ExtBuilder::default()
// 			.add_members(vec![(40, 40), (50, 50)])
// 			.build_and_execute(|| {
// 				let mut bonded_pool = BondedPool::<Runtime>::get(1).unwrap();
// 				let mut reward_pool = RewardPools::<Runtime>::get(1).unwrap();

// 				let mut del_10 = PoolMembers::<Runtime>::get(10).unwrap();
// 				let mut del_40 = PoolMembers::<Runtime>::get(40).unwrap();
// 				let mut del_50 = PoolMembers::<Runtime>::get(50).unwrap();

// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![
// 						Event::Created { depositor: 10, pool_id: 1 },
// 						Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 						Event::Bonded { member: 40, pool_id: 1, bonded: 40, joined: true },
// 						Event::Bonded { member: 50, pool_id: 1, bonded: 50, joined: true }
// 					]
// 				);

// 				// Given we have a total of 100 points split among the members
// 				assert_eq!(del_50.points + del_40.points + del_10.points, 100);
// 				assert_eq!(bonded_pool.points(), 100);

// 				// and the reward pool has earned 100 in rewards
// 				deposit_rewards(100);

// 				// When
// 				let payout =
// 					Lst::do_reward_payout(&10, &mut del_10, &mut bonded_pool, &mut reward_pool)
// 						.unwrap();

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 10, pool_id: 1, payout: 10 }]
// 				);
// 				assert_eq!(payout, 10);
// 				assert_eq!(del_10, del(10, 1));
// 				assert_eq!(reward_pool, rew(0, 0, 10));
// 				Lst::put_member_with_pools(
// 					&10,
// 					del_10.clone(),
// 					bonded_pool.clone(),
// 					reward_pool.clone(),
// 				);

// 				// When
// 				let payout =
// 					Lst::do_reward_payout(&40, &mut del_40, &mut bonded_pool, &mut reward_pool)
// 						.unwrap();

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 40, pool_id: 1, payout: 40 }]
// 				);
// 				assert_eq!(payout, 40);
// 				assert_eq!(del_40, del(40, 1));
// 				assert_eq!(reward_pool, rew(0, 0, 50));
// 				Lst::put_member_with_pools(
// 					&40,
// 					del_40.clone(),
// 					bonded_pool.clone(),
// 					reward_pool.clone(),
// 				);

// 				// When
// 				let payout =
// 					Lst::do_reward_payout(&50, &mut del_50, &mut bonded_pool, &mut reward_pool)
// 						.unwrap();

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 50, pool_id: 1, payout: 50 }]
// 				);
// 				assert_eq!(payout, 50);
// 				assert_eq!(del_50, del(50, 1));
// 				assert_eq!(reward_pool, rew(0, 0, 100));
// 				Lst::put_member_with_pools(
// 					&50,
// 					del_50.clone(),
// 					bonded_pool.clone(),
// 					reward_pool.clone(),
// 				);

// 				// Given the reward pool has some new rewards
// 				deposit_rewards(50);

// 				// When
// 				let payout =
// 					Lst::do_reward_payout(&10, &mut del_10, &mut bonded_pool, &mut reward_pool)
// 						.unwrap();

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 10, pool_id: 1, payout: 5 }]
// 				);
// 				assert_eq!(payout, 5);
// 				assert_eq!(del_10, del_float(10, 1.5));
// 				assert_eq!(reward_pool, rew(0, 0, 105));
// 				Lst::put_member_with_pools(
// 					&10,
// 					del_10.clone(),
// 					bonded_pool.clone(),
// 					reward_pool.clone(),
// 				);

// 				// When
// 				let payout =
// 					Lst::do_reward_payout(&40, &mut del_40, &mut bonded_pool, &mut reward_pool)
// 						.unwrap();

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 40, pool_id: 1, payout: 20 }]
// 				);
// 				assert_eq!(payout, 20);
// 				assert_eq!(del_40, del_float(40, 1.5));
// 				assert_eq!(reward_pool, rew(0, 0, 125));
// 				Lst::put_member_with_pools(
// 					&40,
// 					del_40.clone(),
// 					bonded_pool.clone(),
// 					reward_pool.clone(),
// 				);

// 				// Given del_50 hasn't claimed and the reward pools has just earned 50
// 				deposit_rewards(50);

// 				// When
// 				let payout =
// 					Lst::do_reward_payout(&50, &mut del_50, &mut bonded_pool, &mut reward_pool)
// 						.unwrap();

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 50, pool_id: 1, payout: 50 }]
// 				);
// 				assert_eq!(payout, 50);
// 				assert_eq!(del_50, del_float(50, 2.0));
// 				assert_eq!(reward_pool, rew(0, 0, 175));
// 				Lst::put_member_with_pools(
// 					&50,
// 					del_50.clone(),
// 					bonded_pool.clone(),
// 					reward_pool.clone(),
// 				);

// 				// When
// 				let payout =
// 					Lst::do_reward_payout(&10, &mut del_10, &mut bonded_pool, &mut reward_pool)
// 						.unwrap();

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 10, pool_id: 1, payout: 5 }]
// 				);
// 				assert_eq!(payout, 5);
// 				assert_eq!(del_10, del_float(10, 2.0));
// 				assert_eq!(reward_pool, rew(0, 0, 180));
// 				Lst::put_member_with_pools(
// 					&10,
// 					del_10.clone(),
// 					bonded_pool.clone(),
// 					reward_pool.clone(),
// 				);

// 				// Given del_40 hasn't claimed and the reward pool has just earned 400
// 				deposit_rewards(400);

// 				// When
// 				let payout =
// 					Lst::do_reward_payout(&10, &mut del_10, &mut bonded_pool, &mut reward_pool)
// 						.unwrap();

// 				// Then
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 10, pool_id: 1, payout: 40 }]
// 				);
// 				assert_eq!(payout, 40);
// 				assert_eq!(del_10, del_float(10, 6.0));
// 				assert_eq!(reward_pool, rew(0, 0, 220));
// 				Lst::put_member_with_pools(
// 					&10,
// 					del_10.clone(),
// 					bonded_pool.clone(),
// 					reward_pool.clone(),
// 				);

// 				// Given del_40 + del_50 haven't claimed and the reward pool has earned 20
// 				deposit_rewards(20);

// 				// When
// 				let payout =
// 					Lst::do_reward_payout(&10, &mut del_10, &mut bonded_pool, &mut reward_pool)
// 						.unwrap();

// 				// Then
// 				assert_eq!(payout, 2);
// 				assert_eq!(del_10, del_float(10, 6.2));
// 				assert_eq!(reward_pool, rew(0, 0, 222));
// 				Lst::put_member_with_pools(
// 					&10,
// 					del_10.clone(),
// 					bonded_pool.clone(),
// 					reward_pool.clone(),
// 				);

// 				// When
// 				let payout =
// 					Lst::do_reward_payout(&40, &mut del_40, &mut bonded_pool, &mut reward_pool)
// 						.unwrap();

// 				// Then
// 				assert_eq!(payout, 188); // 20 (from the 50) + 160 (from the 400) + 8 (from the 20)
// 				assert_eq!(del_40, del_float(40, 6.2));
// 				assert_eq!(reward_pool, rew(0, 0, 410));
// 				Lst::put_member_with_pools(
// 					&40,
// 					del_40.clone(),
// 					bonded_pool.clone(),
// 					reward_pool.clone(),
// 				);

// 				// When
// 				let payout =
// 					Lst::do_reward_payout(&50, &mut del_50, &mut bonded_pool, &mut reward_pool)
// 						.unwrap();

// 				// Then
// 				assert_eq!(payout, 210); // 200 (from the 400) + 10 (from the 20)
// 				assert_eq!(del_50, del_float(50, 6.2));
// 				assert_eq!(reward_pool, rew(0, 0, 620));
// 				Lst::put_member_with_pools(
// 					&50,
// 					del_50.clone(),
// 					bonded_pool.clone(),
// 					reward_pool.clone(),
// 				);
// 			});
// 	}

// 	#[test]
// 	fn rewards_distribution_is_fair_basic() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// reward pool by 10.
// 			deposit_rewards(10);

// 			// 20 joins afterwards.
// 			Currency::set_balance(&20, Currency::minimum_balance() + 10);
// 			assert_ok!(Lst::join(RuntimeOrigin::signed(20), 10, 1));

// 			// reward by another 20
// 			deposit_rewards(20);

// 			// 10 should claim 10 + 10, 20 should claim 20 / 2.
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 20, pool_id: 1, bonded: 10, joined: true },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 20 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 10 },
// 				]
// 			);

// 			// any upcoming rewards are shared equally.
// 			deposit_rewards(20);

// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 10 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 10 },
// 				]
// 			);
// 		});
// 	}

// 	#[test]
// 	fn rewards_distribution_is_fair_basic_with_fractions() {
// 		// basically checks the case where the amount of rewards is less than the pool shares. for
// 		// this, we have to rely on fixed point arithmetic.
// 		ExtBuilder::default().build_and_execute(|| {
// 			deposit_rewards(3);

// 			Currency::set_balance(&20, Currency::minimum_balance() + 10);
// 			assert_ok!(Lst::join(RuntimeOrigin::signed(20), 10, 1));

// 			deposit_rewards(6);

// 			// 10 should claim 3, 20 should claim 3 + 3.
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 20, pool_id: 1, bonded: 10, joined: true },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 3 + 3 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 3 },
// 				]
// 			);

// 			// any upcoming rewards are shared equally.
// 			deposit_rewards(8);

// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 4 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 4 },
// 				]
// 			);

// 			// uneven upcoming rewards are shared equally, rounded down.
// 			deposit_rewards(7);

// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 3 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 3 },
// 				]
// 			);
// 		});
// 	}

// 	#[test]
// 	fn rewards_distribution_is_fair_3() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			let ed = Currency::minimum_balance();

// 			deposit_rewards(30);

// 			Currency::set_balance(&20, ed + 10);
// 			assert_ok!(Lst::join(RuntimeOrigin::signed(20), 10, 1));

// 			deposit_rewards(100);

// 			Currency::set_balance(&30, ed + 10);
// 			assert_ok!(Lst::join(RuntimeOrigin::signed(30), 10, 1));

// 			deposit_rewards(60);

// 			// 10 should claim 10, 20 should claim nothing.
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(30)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 20, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 30, pool_id: 1, bonded: 10, joined: true },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 30 + 100 / 2 + 60 / 3 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 100 / 2 + 60 / 3 },
// 					Event::PaidOut { member: 30, pool_id: 1, payout: 60 / 3 },
// 				]
// 			);

// 			// any upcoming rewards are shared equally.
// 			deposit_rewards(30);

// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(30)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 10 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 10 },
// 					Event::PaidOut { member: 30, pool_id: 1, payout: 10 },
// 				]
// 			);
// 		});
// 	}

// 	#[test]
// 	fn pending_rewards_per_member_works() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			let ed = Currency::minimum_balance();

// 			assert_eq!(Lst::api_pending_rewards(10), Some(0));
// 			deposit_rewards(30);
// 			assert_eq!(Lst::api_pending_rewards(10), Some(30));
// 			assert_eq!(Lst::api_pending_rewards(20), None);

// 			Currency::set_balance(&20, ed + 10);
// 			assert_ok!(Lst::join(RuntimeOrigin::signed(20), 10, 1));

// 			assert_eq!(Lst::api_pending_rewards(10), Some(30));
// 			assert_eq!(Lst::api_pending_rewards(20), Some(0));

// 			deposit_rewards(100);

// 			assert_eq!(Lst::api_pending_rewards(10), Some(30 + 50));
// 			assert_eq!(Lst::api_pending_rewards(20), Some(50));
// 			assert_eq!(Lst::api_pending_rewards(30), None);

// 			Currency::set_balance(&30, ed + 10);
// 			assert_ok!(Lst::join(RuntimeOrigin::signed(30), 10, 1));

// 			assert_eq!(Lst::api_pending_rewards(10), Some(30 + 50));
// 			assert_eq!(Lst::api_pending_rewards(20), Some(50));
// 			assert_eq!(Lst::api_pending_rewards(30), Some(0));

// 			deposit_rewards(60);

// 			assert_eq!(Lst::api_pending_rewards(10), Some(30 + 50 + 20));
// 			assert_eq!(Lst::api_pending_rewards(20), Some(50 + 20));
// 			assert_eq!(Lst::api_pending_rewards(30), Some(20));

// 			// 10 should claim 10, 20 should claim nothing.
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_eq!(Lst::api_pending_rewards(10), Some(0));
// 			assert_eq!(Lst::api_pending_rewards(20), Some(50 + 20));
// 			assert_eq!(Lst::api_pending_rewards(30), Some(20));

// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));
// 			assert_eq!(Lst::api_pending_rewards(10), Some(0));
// 			assert_eq!(Lst::api_pending_rewards(20), Some(0));
// 			assert_eq!(Lst::api_pending_rewards(30), Some(20));

// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(30)));
// 			assert_eq!(Lst::api_pending_rewards(10), Some(0));
// 			assert_eq!(Lst::api_pending_rewards(20), Some(0));
// 			assert_eq!(Lst::api_pending_rewards(30), Some(0));
// 		});
// 	}

// 	#[test]
// 	fn rewards_distribution_is_fair_bond_extra() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			let ed = Currency::minimum_balance();

// 			Currency::set_balance(&20, ed + 20);
// 			assert_ok!(Lst::join(RuntimeOrigin::signed(20), 20, 1));
// 			Currency::set_balance(&30, ed + 20);
// 			assert_ok!(Lst::join(RuntimeOrigin::signed(30), 10, 1));

// 			deposit_rewards(40);

// 			// everyone claims.
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(30)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 20, pool_id: 1, bonded: 20, joined: true },
// 					Event::Bonded { member: 30, pool_id: 1, bonded: 10, joined: true },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 10 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 20 },
// 					Event::PaidOut { member: 30, pool_id: 1, payout: 10 }
// 				]
// 			);

// 			// 30 now bumps itself to be like 20.
// 			assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(30), BondExtra::FreeBalance(10)));

// 			// more rewards come in.
// 			deposit_rewards(100);

// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(30)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Bonded { member: 30, pool_id: 1, bonded: 10, joined: false },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 20 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 40 },
// 					Event::PaidOut { member: 30, pool_id: 1, payout: 40 }
// 				]
// 			);
// 		});
// 	}

// 	#[test]
// 	fn rewards_distribution_is_fair_unbond() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			let ed = Currency::minimum_balance();

// 			Currency::set_balance(&20, ed + 20);
// 			assert_ok!(Lst::join(RuntimeOrigin::signed(20), 20, 1));

// 			deposit_rewards(30);

// 			// everyone claims.
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 20, pool_id: 1, bonded: 20, joined: true },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 10 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 20 }
// 				]
// 			);

// 			// 20 unbonds to be equal to 10 (10 points each).
// 			assert_ok!(Lst::unbond(RuntimeOrigin::signed(20), 20, 10));

// 			// more rewards come in.
// 			deposit_rewards(100);

// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Unbonded { member: 20, pool_id: 1, balance: 10, points: 10, era: 3 },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 50 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 50 },
// 				]
// 			);
// 		});
// 	}

// 	#[test]
// 	fn unclaimed_reward_is_safe() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			let ed = Currency::minimum_balance();

// 			Currency::set_balance(&20, ed + 20);
// 			assert_ok!(Lst::join(RuntimeOrigin::signed(20), 20, 1));
// 			Currency::set_balance(&30, ed + 20);
// 			assert_ok!(Lst::join(RuntimeOrigin::signed(30), 10, 1));

// 			// 10 gets 10, 20 gets 20, 30 gets 10
// 			deposit_rewards(40);

// 			// some claim.
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 20, pool_id: 1, bonded: 20, joined: true },
// 					Event::Bonded { member: 30, pool_id: 1, bonded: 10, joined: true },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 10 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 20 }
// 				]
// 			);

// 			// 10 gets 20, 20 gets 40, 30 gets 20
// 			deposit_rewards(80);

// 			// some claim.
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 20 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 40 }
// 				]
// 			);

// 			// 10 gets 20, 20 gets 40, 30 gets 20
// 			deposit_rewards(80);

// 			// some claim.
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 20 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 40 }
// 				]
// 			);

// 			// now 30 claims all at once
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(30)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PaidOut { member: 30, pool_id: 1, payout: 10 + 20 + 20 }]
// 			);
// 		});
// 	}

// 	#[test]
// 	fn bond_extra_and_delayed_claim() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			let ed = Currency::minimum_balance();

// 			Currency::set_balance(&20, ed + 200);
// 			assert_ok!(Lst::join(RuntimeOrigin::signed(20), 20, 1));

// 			// 10 gets 10, 20 gets 20, 30 gets 10
// 			deposit_rewards(30);

// 			// some claim.
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 20, pool_id: 1, bonded: 20, joined: true },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 10 }
// 				]
// 			);

// 			// 20 has not claimed yet, more reward comes
// 			deposit_rewards(60);

// 			// and 20 bonds more -- they should not have more share of this reward.
// 			assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(20), BondExtra::FreeBalance(10)));

// 			// everyone claim.
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					// 20 + 40, which means the extra amount they bonded did not impact us.
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 60 },
// 					Event::Bonded { member: 20, pool_id: 1, bonded: 10, joined: false },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 20 }
// 				]
// 			);

// 			// but in the next round of rewards, the extra10 they bonded has an impact.
// 			deposit_rewards(60);

// 			// everyone claim.
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 15 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 45 }
// 				]
// 			);
// 		});
// 	}

// 	#[test]
// 	fn create_sets_recorded_data() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			MaxPools::<Runtime>::set(None);
// 			// pool 10 has already been created.
// 			let (member_10, _, reward_pool_10) = Lst::get_member_with_pools(&10).unwrap();

// 			assert_eq!(reward_pool_10.last_recorded_total_payouts, 0);
// 			assert_eq!(reward_pool_10.total_rewards_claimed, 0);
// 			assert_eq!(reward_pool_10.last_recorded_reward_counter, 0.into());

// 			assert_eq!(member_10.last_recorded_reward_counter, 0.into());

// 			// transfer some reward to pool 1.
// 			deposit_rewards(60);

// 			// create pool 2
// 			Currency::set_balance(&20, 100);
// 			assert_ok!(Lst::create(RuntimeOrigin::signed(20), 10, 20, 20, 20));

// 			// has no impact -- initial
// 			let (member_20, _, reward_pool_20) = Lst::get_member_with_pools(&20).unwrap();

// 			assert_eq!(reward_pool_20.last_recorded_total_payouts, 0);
// 			assert_eq!(reward_pool_20.total_rewards_claimed, 0);
// 			assert_eq!(reward_pool_20.last_recorded_reward_counter, 0.into());

// 			assert_eq!(member_20.last_recorded_reward_counter, 0.into());

// 			// pre-fund the reward account of pool id 3 with some funds.
// 			Currency::set_balance(&Lst::create_reward_account(3), 10);

// 			// create pool 3
// 			Currency::set_balance(&30, 100);
// 			assert_ok!(Lst::create(RuntimeOrigin::signed(30), 10, 30, 30, 30));

// 			// reward counter is still the same.
// 			let (member_30, _, reward_pool_30) = Lst::get_member_with_pools(&30).unwrap();
// 			assert_eq!(
// 				Currency::free_balance(&Lst::create_reward_account(3)),
// 				10 + Currency::minimum_balance()
// 			);

// 			assert_eq!(reward_pool_30.last_recorded_total_payouts, 0);
// 			assert_eq!(reward_pool_30.total_rewards_claimed, 0);
// 			assert_eq!(reward_pool_30.last_recorded_reward_counter, 0.into());

// 			assert_eq!(member_30.last_recorded_reward_counter, 0.into());

// 			// and 30 can claim the reward now.
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(30)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::Created { depositor: 20, pool_id: 2 },
// 					Event::Bonded { member: 20, pool_id: 2, bonded: 10, joined: true },
// 					Event::Created { depositor: 30, pool_id: 3 },
// 					Event::Bonded { member: 30, pool_id: 3, bonded: 10, joined: true },
// 					Event::PaidOut { member: 30, pool_id: 3, payout: 10 }
// 				]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn join_updates_recorded_data() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			MaxPoolMembers::<Runtime>::set(None);
// 			MaxPoolMembersPerPool::<Runtime>::set(None);
// 			let join = |x, y| {
// 				Currency::set_balance(&x, y + Currency::minimum_balance());
// 				assert_ok!(Lst::join(RuntimeOrigin::signed(x), y, 1));
// 			};

// 			{
// 				let (member_10, _, reward_pool_10) = Lst::get_member_with_pools(&10).unwrap();

// 				assert_eq!(reward_pool_10.last_recorded_total_payouts, 0);
// 				assert_eq!(reward_pool_10.total_rewards_claimed, 0);
// 				assert_eq!(reward_pool_10.last_recorded_reward_counter, 0.into());

// 				assert_eq!(member_10.last_recorded_reward_counter, 0.into());
// 			}

// 			// someone joins without any rewards being issued.
// 			{
// 				join(20, 10);
// 				let (member, _, reward_pool) = Lst::get_member_with_pools(&20).unwrap();
// 				// reward counter is 0 both before..
// 				assert_eq!(member.last_recorded_reward_counter, 0.into());
// 				assert_eq!(reward_pool.last_recorded_total_payouts, 0);
// 				assert_eq!(reward_pool.last_recorded_reward_counter, 0.into());
// 			}

// 			// transfer some reward to pool 1.
// 			deposit_rewards(60);

// 			{
// 				join(30, 10);
// 				let (member, _, reward_pool) = Lst::get_member_with_pools(&30).unwrap();
// 				assert_eq!(reward_pool.last_recorded_total_payouts, 60);
// 				// explanation: we have a total of 20 points so far (excluding the 10 that just got
// 				// bonded), and 60 unclaimed rewards. each share is then roughly worth of 3 units of
// 				// rewards, thus reward counter is 3. member's reward counter is the same
// 				assert_eq!(member.last_recorded_reward_counter, 3.into());
// 				assert_eq!(reward_pool.last_recorded_reward_counter, 3.into());
// 			}

// 			// someone else joins
// 			{
// 				join(40, 10);
// 				let (member, _, reward_pool) = Lst::get_member_with_pools(&40).unwrap();
// 				// reward counter does not change since no rewards have came in.
// 				assert_eq!(member.last_recorded_reward_counter, 3.into());
// 				assert_eq!(reward_pool.last_recorded_reward_counter, 3.into());
// 				assert_eq!(reward_pool.last_recorded_total_payouts, 60);
// 			}

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 20, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 30, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 40, pool_id: 1, bonded: 10, joined: true }
// 				]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn bond_extra_updates_recorded_data() {
// 		ExtBuilder::default().add_members(vec![(20, 20)]).build_and_execute(|| {
// 			MaxPoolMembers::<Runtime>::set(None);
// 			MaxPoolMembersPerPool::<Runtime>::set(None);

// 			// initial state of pool 1.
// 			{
// 				let (member_10, _, reward_pool_10) = Lst::get_member_with_pools(&10).unwrap();

// 				assert_eq!(reward_pool_10.last_recorded_total_payouts, 0);
// 				assert_eq!(reward_pool_10.total_rewards_claimed, 0);
// 				assert_eq!(reward_pool_10.last_recorded_reward_counter, 0.into());

// 				assert_eq!(member_10.last_recorded_reward_counter, 0.into());
// 			}

// 			Currency::set_balance(&10, 100);
// 			Currency::set_balance(&20, 100);

// 			// 10 bonds extra without any rewards.
// 			{
// 				assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), BondExtra::FreeBalance(10)));
// 				let (member, _, reward_pool) = Lst::get_member_with_pools(&10).unwrap();
// 				assert_eq!(member.last_recorded_reward_counter, 0.into());
// 				assert_eq!(reward_pool.last_recorded_total_payouts, 0);
// 				assert_eq!(reward_pool.last_recorded_reward_counter, 0.into());
// 			}

// 			// 10 bonds extra again with some rewards. This reward should be split equally between
// 			// 10 and 20, as they both have equal points now.
// 			deposit_rewards(30);

// 			{
// 				assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), BondExtra::FreeBalance(10)));
// 				let (member, _, reward_pool) = Lst::get_member_with_pools(&10).unwrap();
// 				// explanation: before bond_extra takes place, there is 40 points and 30 balance in
// 				// the system, RewardCounter is therefore 7.5
// 				assert_eq!(member.last_recorded_reward_counter, RewardCounter::from_float(0.75));
// 				assert_eq!(
// 					reward_pool.last_recorded_reward_counter,
// 					RewardCounter::from_float(0.75)
// 				);
// 				assert_eq!(reward_pool.last_recorded_total_payouts, 30);
// 			}

// 			// 20 bonds extra again, without further rewards.
// 			{
// 				assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(20), BondExtra::FreeBalance(10)));
// 				let (member, _, reward_pool) = Lst::get_member_with_pools(&20).unwrap();
// 				assert_eq!(member.last_recorded_reward_counter, RewardCounter::from_float(0.75));
// 				assert_eq!(
// 					reward_pool.last_recorded_reward_counter,
// 					RewardCounter::from_float(0.75)
// 				);
// 				assert_eq!(reward_pool.last_recorded_total_payouts, 30);
// 			}

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 20, pool_id: 1, bonded: 20, joined: true },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: false },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 15 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: false },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 15 },
// 					Event::Bonded { member: 20, pool_id: 1, bonded: 10, joined: false }
// 				]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn bond_extra_pending_rewards_works() {
// 		ExtBuilder::default().add_members(vec![(20, 20)]).build_and_execute(|| {
// 			MaxPoolMembers::<Runtime>::set(None);
// 			MaxPoolMembersPerPool::<Runtime>::set(None);

// 			// pool receives some rewards.
// 			deposit_rewards(30);
// 			System::reset_events();

// 			// 10 cashes it out, and bonds it.
// 			{
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 				let (member, _, reward_pool) = Lst::get_member_with_pools(&10).unwrap();
// 				// there is 30 points and 30 reward points in the system RC is 1.
// 				assert_eq!(member.last_recorded_reward_counter, 1.into());
// 				assert_eq!(reward_pool.total_rewards_claimed, 10);
// 				// these two are not updated -- only updated when the points change.
// 				assert_eq!(reward_pool.last_recorded_total_payouts, 0);
// 				assert_eq!(reward_pool.last_recorded_reward_counter, 0.into());

// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 10, pool_id: 1, payout: 10 }]
// 				);
// 			}

// 			// 20 re-bonds it.
// 			{
// 				assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(20), BondExtra::Rewards));
// 				let (member, _, reward_pool) = Lst::get_member_with_pools(&10).unwrap();
// 				assert_eq!(member.last_recorded_reward_counter, 1.into());
// 				assert_eq!(reward_pool.total_rewards_claimed, 30);
// 				// since points change, these two are updated.
// 				assert_eq!(reward_pool.last_recorded_total_payouts, 30);
// 				assert_eq!(reward_pool.last_recorded_reward_counter, 1.into());

// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![
// 						Event::PaidOut { member: 20, pool_id: 1, payout: 20 },
// 						Event::Bonded { member: 20, pool_id: 1, bonded: 20, joined: false }
// 					]
// 				);
// 			}
// 		})
// 	}

// 	#[test]
// 	fn unbond_updates_recorded_data() {
// 		ExtBuilder::default()
// 			.add_members(vec![(20, 20), (30, 20)])
// 			.build_and_execute(|| {
// 				MaxPoolMembers::<Runtime>::set(None);
// 				MaxPoolMembersPerPool::<Runtime>::set(None);

// 				// initial state of pool 1.
// 				{
// 					let (member, _, reward_pool) = Lst::get_member_with_pools(&10).unwrap();

// 					assert_eq!(reward_pool.last_recorded_total_payouts, 0);
// 					assert_eq!(reward_pool.total_rewards_claimed, 0);
// 					assert_eq!(reward_pool.last_recorded_reward_counter, 0.into());

// 					assert_eq!(member.last_recorded_reward_counter, 0.into());
// 				}

// 				// 20 unbonds without any rewards.
// 				{
// 					assert_ok!(Lst::unbond(RuntimeOrigin::signed(20), 20, 10));
// 					let (member, _, reward_pool) = Lst::get_member_with_pools(&20).unwrap();
// 					assert_eq!(member.last_recorded_reward_counter, 0.into());
// 					assert_eq!(reward_pool.last_recorded_total_payouts, 0);
// 					assert_eq!(reward_pool.last_recorded_reward_counter, 0.into());
// 				}

// 				// some rewards come in.
// 				deposit_rewards(30);

// 				// and 30 also unbonds half.
// 				{
// 					assert_ok!(Lst::unbond(RuntimeOrigin::signed(30), 30, 10));
// 					let (member, _, reward_pool) = Lst::get_member_with_pools(&30).unwrap();
// 					// 30 reward in the system, and 40 points before this unbond to collect it,
// 					// RewardCounter is 3/4.
// 					assert_eq!(
// 						member.last_recorded_reward_counter,
// 						RewardCounter::from_float(0.75)
// 					);
// 					assert_eq!(reward_pool.last_recorded_total_payouts, 30);
// 					assert_eq!(
// 						reward_pool.last_recorded_reward_counter,
// 						RewardCounter::from_float(0.75)
// 					);
// 				}

// 				// 30 unbonds again, not change this time.
// 				{
// 					assert_ok!(Lst::unbond(RuntimeOrigin::signed(30), 30, 5));
// 					let (member, _, reward_pool) = Lst::get_member_with_pools(&30).unwrap();
// 					assert_eq!(
// 						member.last_recorded_reward_counter,
// 						RewardCounter::from_float(0.75)
// 					);
// 					assert_eq!(reward_pool.last_recorded_total_payouts, 30);
// 					assert_eq!(
// 						reward_pool.last_recorded_reward_counter,
// 						RewardCounter::from_float(0.75)
// 					);
// 				}

// 				// 20 unbonds again, not change this time, just collecting their reward.
// 				{
// 					assert_ok!(Lst::unbond(RuntimeOrigin::signed(20), 20, 5));
// 					let (member, _, reward_pool) = Lst::get_member_with_pools(&20).unwrap();
// 					assert_eq!(
// 						member.last_recorded_reward_counter,
// 						RewardCounter::from_float(0.75)
// 					);
// 					assert_eq!(reward_pool.last_recorded_total_payouts, 30);
// 					assert_eq!(
// 						reward_pool.last_recorded_reward_counter,
// 						RewardCounter::from_float(0.75)
// 					);
// 				}

// 				// trigger 10's reward as well to see all of the payouts.
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![
// 						Event::Created { depositor: 10, pool_id: 1 },
// 						Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 						Event::Bonded { member: 20, pool_id: 1, bonded: 20, joined: true },
// 						Event::Bonded { member: 30, pool_id: 1, bonded: 20, joined: true },
// 						Event::Unbonded { member: 20, pool_id: 1, balance: 10, points: 10, era: 3 },
// 						Event::PaidOut { member: 30, pool_id: 1, payout: 15 },
// 						Event::Unbonded { member: 30, pool_id: 1, balance: 10, points: 10, era: 3 },
// 						Event::Unbonded { member: 30, pool_id: 1, balance: 5, points: 5, era: 3 },
// 						Event::PaidOut { member: 20, pool_id: 1, payout: 7 },
// 						Event::Unbonded { member: 20, pool_id: 1, balance: 5, points: 5, era: 3 },
// 						Event::PaidOut { member: 10, pool_id: 1, payout: 7 }
// 					]
// 				);
// 			})
// 	}

// 	#[test]
// 	fn rewards_are_rounded_down_depositor_collects_them() {
// 		ExtBuilder::default().add_members(vec![(20, 20)]).build_and_execute(|| {
// 			// initial balance of 10.

// 			assert_eq!(Currency::free_balance(&10), 35);
// 			assert_eq!(
// 				Currency::free_balance(&default_reward_account()),
// 				Currency::minimum_balance()
// 			);

// 			// some rewards come in.
// 			deposit_rewards(40);

// 			// everyone claims
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

// 			// some dust (1) remains in the reward account.
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 20, pool_id: 1, bonded: 20, joined: true },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 13 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 26 }
// 				]
// 			);

// 			// start dismantling the pool.
// 			assert_ok!(Lst::set_state(RuntimeOrigin::signed(902), 1, PoolState::Destroying));
// 			assert_ok!(fully_unbond_permissioned(20));

// 			CurrentEra::set(3);
// 			assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(20), 20, 0));
// 			assert_ok!(fully_unbond_permissioned(10));

// 			CurrentEra::set(6);
// 			assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(10), 10, 0));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::StateChanged { pool_id: 1, new_state: PoolState::Destroying },
// 					Event::Unbonded { member: 20, pool_id: 1, balance: 20, points: 20, era: 3 },
// 					Event::Withdrawn { member: 20, pool_id: 1, balance: 20, points: 20 },
// 					Event::MemberRemoved { pool_id: 1, member: 20 },
// 					Event::Unbonded { member: 10, pool_id: 1, balance: 10, points: 10, era: 6 },
// 					Event::Withdrawn { member: 10, pool_id: 1, balance: 10, points: 10 },
// 					Event::MemberRemoved { pool_id: 1, member: 10 },
// 					Event::Destroyed { pool_id: 1 }
// 				]
// 			);

// 			assert!(!Metadata::<T>::contains_key(1));
// 			// original ed + ed put into reward account + reward + bond + dust.
// 			assert_eq!(Currency::free_balance(&10), 35 + 5 + 13 + 10 + 1);
// 		})
// 	}

// 	#[test]
// 	fn claim_payout_large_numbers() {
// 		let unit = 10u128.pow(12); // akin to KSM
// 		ExistentialDeposit::set(unit);
// 		StakingMinBond::set(unit * 1000);

// 		ExtBuilder::default()
// 			.max_members(Some(4))
// 			.max_members_per_pool(Some(4))
// 			.add_members(vec![(20, 1500 * unit), (21, 2500 * unit), (22, 5000 * unit)])
// 			.build_and_execute(|| {
// 				// some rewards come in.
// 				assert_eq!(Currency::free_balance(&default_reward_account()), unit);
// 				deposit_rewards(unit / 1000);

// 				// everyone claims
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(21)));
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(22)));

// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![
// 						Event::Created { depositor: 10, pool_id: 1 },
// 						Event::Bonded {
// 							member: 10,
// 							pool_id: 1,
// 							bonded: 1000000000000000,
// 							joined: true
// 						},
// 						Event::Bonded {
// 							member: 20,
// 							pool_id: 1,
// 							bonded: 1500000000000000,
// 							joined: true
// 						},
// 						Event::Bonded {
// 							member: 21,
// 							pool_id: 1,
// 							bonded: 2500000000000000,
// 							joined: true
// 						},
// 						Event::Bonded {
// 							member: 22,
// 							pool_id: 1,
// 							bonded: 5000000000000000,
// 							joined: true
// 						},
// 						Event::PaidOut { member: 10, pool_id: 1, payout: 100000000 },
// 						Event::PaidOut { member: 20, pool_id: 1, payout: 150000000 },
// 						Event::PaidOut { member: 21, pool_id: 1, payout: 250000000 },
// 						Event::PaidOut { member: 22, pool_id: 1, payout: 500000000 }
// 					]
// 				);
// 			})
// 	}

// 	#[test]
// 	fn claim_payout_other_works() {
// 		ExtBuilder::default().add_members(vec![(20, 20)]).build_and_execute(|| {
// 			Currency::set_balance(&default_reward_account(), 8);
// 			// ... of which only 3 are claimable to make sure the reward account does not die.
// 			let claimable_reward = 8 - ExistentialDeposit::get();
// 			// NOTE: easier to read if we use 3, so let's use the number instead of variable.
// 			assert_eq!(claimable_reward, 3, "test is correct if rewards are divisible by 3");

// 			// given
// 			assert_eq!(Currency::free_balance(&10), 35);

// 			// Permissioned by default
// 			assert_noop!(
// 				Lst::claim_payout_other(RuntimeOrigin::signed(80), 10),
// 				Error::<Runtime>::DoesNotHavePermission
// 			);

// 			assert_ok!(Lst::set_claim_permission(
// 				RuntimeOrigin::signed(10),
// 				ClaimPermission::PermissionlessWithdraw
// 			));
// 			assert_ok!(Lst::claim_payout_other(RuntimeOrigin::signed(80), 10));

// 			// then
// 			assert_eq!(Currency::free_balance(&10), 36);
// 			assert_eq!(Currency::free_balance(&default_reward_account()), 7);
// 		})
// 	}
// }

// mod nominate {
// 	use super::*;

// 	#[test]
// 	fn nominate_works() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// Depositor can't nominate
// 			assert_noop!(
// 				Lst::nominate(RuntimeOrigin::signed(10), 1, vec![21]),
// 				Error::<Runtime>::NotNominator
// 			);

// 			// bouncer can't nominate
// 			assert_noop!(
// 				Lst::nominate(RuntimeOrigin::signed(902), 1, vec![21]),
// 				Error::<Runtime>::NotNominator
// 			);

// 			// Root can nominate
// 			assert_ok!(Lst::nominate(RuntimeOrigin::signed(900), 1, vec![21]));
// 			assert_eq!(Nominations::get().unwrap(), vec![21]);

// 			// Nominator can nominate
// 			assert_ok!(Lst::nominate(RuntimeOrigin::signed(901), 1, vec![31]));
// 			assert_eq!(Nominations::get().unwrap(), vec![31]);

// 			// Can't nominate for a pool that doesn't exist
// 			assert_noop!(
// 				Lst::nominate(RuntimeOrigin::signed(902), 123, vec![21]),
// 				Error::<Runtime>::PoolNotFound
// 			);
// 		});
// 	}
// }

// mod set_state {
// 	use super::*;

// 	#[test]
// 	fn set_state_works() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// Given
// 			assert_ok!(BondedPool::<Runtime>::get(1).unwrap().ok_to_be_open());

// 			// Only the root and bouncer can change the state when the pool is ok to be open.
// 			assert_noop!(
// 				Lst::set_state(RuntimeOrigin::signed(10), 1, PoolState::Blocked),
// 				Error::<Runtime>::CanNotChangeState
// 			);
// 			assert_noop!(
// 				Lst::set_state(RuntimeOrigin::signed(901), 1, PoolState::Blocked),
// 				Error::<Runtime>::CanNotChangeState
// 			);

// 			// Root can change state
// 			assert_ok!(Lst::set_state(RuntimeOrigin::signed(900), 1, PoolState::Blocked));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::StateChanged { pool_id: 1, new_state: PoolState::Blocked }
// 				]
// 			);

// 			assert_eq!(BondedPool::<Runtime>::get(1).unwrap().state, PoolState::Blocked);

// 			// bouncer can change state
// 			assert_ok!(Lst::set_state(RuntimeOrigin::signed(902), 1, PoolState::Destroying));
// 			assert_eq!(BondedPool::<Runtime>::get(1).unwrap().state, PoolState::Destroying);

// 			// If the pool is destroying, then no one can set state
// 			assert_noop!(
// 				Lst::set_state(RuntimeOrigin::signed(900), 1, PoolState::Blocked),
// 				Error::<Runtime>::CanNotChangeState
// 			);
// 			assert_noop!(
// 				Lst::set_state(RuntimeOrigin::signed(902), 1, PoolState::Blocked),
// 				Error::<Runtime>::CanNotChangeState
// 			);

// 			// If the pool is not ok to be open, then anyone can set it to destroying

// 			// Given
// 			unsafe_set_state(1, PoolState::Open);
// 			// slash the pool to the point that `max_points_to_balance` ratio is
// 			// surpassed. Making this pool destroyable by anyone.
// 			StakingMock::slash_by(1, 10);

// 			// When
// 			assert_ok!(Lst::set_state(RuntimeOrigin::signed(11), 1, PoolState::Destroying));
// 			// Then
// 			assert_eq!(BondedPool::<Runtime>::get(1).unwrap().state, PoolState::Destroying);

// 			// Given
// 			Currency::set_balance(&default_bonded_account(), Balance::MAX / 10);
// 			unsafe_set_state(1, PoolState::Open);
// 			// When
// 			assert_ok!(Lst::set_state(RuntimeOrigin::signed(11), 1, PoolState::Destroying));
// 			// Then
// 			assert_eq!(BondedPool::<Runtime>::get(1).unwrap().state, PoolState::Destroying);

// 			// If the pool is not ok to be open, it cannot be permissionlessly set to a state that
// 			// isn't destroying
// 			unsafe_set_state(1, PoolState::Open);
// 			assert_noop!(
// 				Lst::set_state(RuntimeOrigin::signed(11), 1, PoolState::Blocked),
// 				Error::<Runtime>::CanNotChangeState
// 			);

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::StateChanged { pool_id: 1, new_state: PoolState::Destroying },
// 					Event::PoolSlashed { pool_id: 1, balance: 0 },
// 					Event::StateChanged { pool_id: 1, new_state: PoolState::Destroying },
// 					Event::StateChanged { pool_id: 1, new_state: PoolState::Destroying }
// 				]
// 			);
// 		});
// 	}
// }

// mod set_metadata {
// 	use super::*;

// 	#[test]
// 	fn set_metadata_works() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// Root can set metadata
// 			assert_ok!(Lst::set_metadata(RuntimeOrigin::signed(900), 1, vec![1, 1]));
// 			assert_eq!(Metadata::<Runtime>::get(1), vec![1, 1]);

// 			// bouncer can set metadata
// 			assert_ok!(Lst::set_metadata(RuntimeOrigin::signed(902), 1, vec![2, 2]));
// 			assert_eq!(Metadata::<Runtime>::get(1), vec![2, 2]);

// 			// Depositor can't set metadata
// 			assert_noop!(
// 				Lst::set_metadata(RuntimeOrigin::signed(10), 1, vec![3, 3]),
// 				Error::<Runtime>::DoesNotHavePermission
// 			);

// 			// Nominator can't set metadata
// 			assert_noop!(
// 				Lst::set_metadata(RuntimeOrigin::signed(901), 1, vec![3, 3]),
// 				Error::<Runtime>::DoesNotHavePermission
// 			);

// 			// Metadata cannot be longer than `MaxMetadataLen`
// 			assert_noop!(
// 				Lst::set_metadata(RuntimeOrigin::signed(900), 1, vec![1, 1, 1]),
// 				Error::<Runtime>::MetadataExceedsMaxLen
// 			);
// 		});
// 	}
// }

// mod set_configs {
// 	use super::*;

// 	#[test]
// 	fn set_configs_works() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// Setting works
// 			assert_ok!(Lst::set_configs(
// 				RuntimeOrigin::root(),
// 				ConfigOp::Set(1 as Balance),
// 				ConfigOp::Set(2 as Balance),
// 				ConfigOp::Set(3u32),
// 				ConfigOp::Set(4u32),
// 				ConfigOp::Set(5u32),
// 				ConfigOp::Set(Perbill::from_percent(6))
// 			));
// 			assert_eq!(MinJoinBond::<Runtime>::get(), 1);
// 			assert_eq!(MinCreateBond::<Runtime>::get(), 2);
// 			assert_eq!(MaxPools::<Runtime>::get(), Some(3));
// 			assert_eq!(MaxPoolMembers::<Runtime>::get(), Some(4));
// 			assert_eq!(MaxPoolMembersPerPool::<Runtime>::get(), Some(5));
// 			assert_eq!(GlobalMaxCommission::<Runtime>::get(), Some(Perbill::from_percent(6)));

// 			// Noop does nothing
// 			assert_storage_noop!(assert_ok!(Lst::set_configs(
// 				RuntimeOrigin::root(),
// 				ConfigOp::Noop,
// 				ConfigOp::Noop,
// 				ConfigOp::Noop,
// 				ConfigOp::Noop,
// 				ConfigOp::Noop,
// 				ConfigOp::Noop,
// 			)));

// 			// Removing works
// 			assert_ok!(Lst::set_configs(
// 				RuntimeOrigin::root(),
// 				ConfigOp::Remove,
// 				ConfigOp::Remove,
// 				ConfigOp::Remove,
// 				ConfigOp::Remove,
// 				ConfigOp::Remove,
// 				ConfigOp::Remove,
// 			));
// 			assert_eq!(MinJoinBond::<Runtime>::get(), 0);
// 			assert_eq!(MinCreateBond::<Runtime>::get(), 0);
// 			assert_eq!(MaxPools::<Runtime>::get(), None);
// 			assert_eq!(MaxPoolMembers::<Runtime>::get(), None);
// 			assert_eq!(MaxPoolMembersPerPool::<Runtime>::get(), None);
// 			assert_eq!(GlobalMaxCommission::<Runtime>::get(), None);
// 		});
// 	}
// }

// mod bond_extra {
// 	use super::*;
// 	use crate::Event;

// 	#[test]
// 	fn bond_extra_from_free_balance_creator() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// 10 is the owner and a member in pool 1, give them some more funds.
// 			Currency::set_balance(&10, 100);

// 			// given
// 			assert_eq!(PoolMembers::<Runtime>::get(10).unwrap().points, 10);
// 			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 10);
// 			assert_eq!(Currency::free_balance(&10), 100);

// 			// when
// 			assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), BondExtra::FreeBalance(10)));

// 			// then
// 			assert_eq!(Currency::free_balance(&10), 90);
// 			assert_eq!(PoolMembers::<Runtime>::get(10).unwrap().points, 20);
// 			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 20);

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: false }
// 				]
// 			);

// 			// when
// 			assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), BondExtra::FreeBalance(20)));

// 			// then
// 			assert_eq!(Currency::free_balance(&10), 70);
// 			assert_eq!(PoolMembers::<Runtime>::get(10).unwrap().points, 40);
// 			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 40);

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::Bonded { member: 10, pool_id: 1, bonded: 20, joined: false }]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn bond_extra_from_rewards_creator() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// put some money in the reward account, all of which will belong to 10 as the only
// 			// member of the pool.
// 			Currency::set_balance(&default_reward_account(), 7);
// 			// ... if which only 2 is claimable to make sure the reward account does not die.
// 			let claimable_reward = 7 - ExistentialDeposit::get();

// 			// given
// 			assert_eq!(PoolMembers::<Runtime>::get(10).unwrap().points, 10);
// 			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 10);
// 			assert_eq!(Currency::free_balance(&10), 35);

// 			// when
// 			assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), BondExtra::Rewards));

// 			// then
// 			assert_eq!(Currency::free_balance(&10), 35);
// 			assert_eq!(PoolMembers::<Runtime>::get(10).unwrap().points, 10 + claimable_reward);
// 			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 10 + claimable_reward);

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: claimable_reward },
// 					Event::Bonded {
// 						member: 10,
// 						pool_id: 1,
// 						bonded: claimable_reward,
// 						joined: false
// 					}
// 				]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn bond_extra_from_rewards_joiner() {
// 		ExtBuilder::default().add_members(vec![(20, 20)]).build_and_execute(|| {
// 			// put some money in the reward account, all of which will belong to 10 as the only
// 			// member of the pool.
// 			Currency::set_balance(&default_reward_account(), 8);
// 			// ... if which only 3 is claimable to make sure the reward account does not die.
// 			let claimable_reward = 8 - ExistentialDeposit::get();
// 			// NOTE: easier to read of we use 3, so let's use the number instead of variable.
// 			assert_eq!(claimable_reward, 3, "test is correct if rewards are divisible by 3");

// 			// given
// 			assert_eq!(PoolMembers::<Runtime>::get(10).unwrap().points, 10);
// 			assert_eq!(PoolMembers::<Runtime>::get(20).unwrap().points, 20);
// 			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 30);

// 			assert_eq!(Currency::free_balance(&10), 35);
// 			assert_eq!(Currency::free_balance(&20), 20);
// 			assert_eq!(TotalValueLocked::<T>::get(), 30);

// 			// when
// 			assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), BondExtra::Rewards));
// 			assert_eq!(Currency::free_balance(&default_reward_account()), 7);

// 			// then
// 			assert_eq!(Currency::free_balance(&10), 35);
// 			assert_eq!(TotalValueLocked::<T>::get(), 31);

// 			// 10's share of the reward is 1/3, since they gave 10/30 of the total shares.
// 			assert_eq!(PoolMembers::<Runtime>::get(10).unwrap().points, 10 + 1);
// 			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 30 + 1);

// 			// when
// 			assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(20), BondExtra::Rewards));

// 			// then
// 			assert_eq!(Currency::free_balance(&20), 20);
// 			assert_eq!(TotalValueLocked::<T>::get(), 33);

// 			// 20's share of the rewards is the other 2/3 of the rewards, since they have 20/30 of
// 			// the shares
// 			assert_eq!(PoolMembers::<Runtime>::get(20).unwrap().points, 20 + 2);
// 			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 30 + 3);

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::Bonded { member: 20, pool_id: 1, bonded: 20, joined: true },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 1, joined: false },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 2 },
// 					Event::Bonded { member: 20, pool_id: 1, bonded: 2, joined: false }
// 				]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn bond_extra_other() {
// 		ExtBuilder::default().add_members(vec![(20, 20)]).build_and_execute(|| {
// 			Currency::set_balance(&default_reward_account(), 8);
// 			// ... of which only 3 are claimable to make sure the reward account does not die.
// 			let claimable_reward = 8 - ExistentialDeposit::get();
// 			// NOTE: easier to read if we use 3, so let's use the number instead of variable.
// 			assert_eq!(claimable_reward, 3, "test is correct if rewards are divisible by 3");

// 			// given
// 			assert_eq!(PoolMembers::<Runtime>::get(10).unwrap().points, 10);
// 			assert_eq!(PoolMembers::<Runtime>::get(20).unwrap().points, 20);
// 			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 30);
// 			assert_eq!(Currency::free_balance(&10), 35);
// 			assert_eq!(Currency::free_balance(&20), 20);

// 			// Permissioned by default
// 			assert_noop!(
// 				Lst::bond_extra_other(RuntimeOrigin::signed(80), 20, BondExtra::Rewards),
// 				Error::<Runtime>::DoesNotHavePermission
// 			);

// 			assert_ok!(Lst::set_claim_permission(
// 				RuntimeOrigin::signed(10),
// 				ClaimPermission::PermissionlessAll
// 			));
// 			assert_ok!(Lst::bond_extra_other(RuntimeOrigin::signed(50), 10, BondExtra::Rewards));
// 			assert_eq!(Currency::free_balance(&default_reward_account()), 7);

// 			// then
// 			assert_eq!(Currency::free_balance(&10), 35);
// 			assert_eq!(PoolMembers::<Runtime>::get(10).unwrap().points, 10 + 1);
// 			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 30 + 1);

// 			// when
// 			assert_noop!(
// 				Lst::bond_extra_other(RuntimeOrigin::signed(40), 40, BondExtra::Rewards),
// 				Error::<Runtime>::PoolMemberNotFound
// 			);

// 			// when
// 			assert_ok!(Lst::bond_extra_other(
// 				RuntimeOrigin::signed(20),
// 				20,
// 				BondExtra::FreeBalance(10)
// 			));

// 			// then
// 			assert_eq!(Currency::free_balance(&20), 12);
// 			assert_eq!(Currency::free_balance(&default_reward_account()), 5);
// 			assert_eq!(PoolMembers::<Runtime>::get(20).unwrap().points, 30);
// 			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 41);
// 		})
// 	}
// }

// mod update_roles {
// 	use super::*;

// 	#[test]
// 	fn update_roles_works() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			assert_eq!(
// 				BondedPools::<Runtime>::get(1).unwrap().roles,
// 				PoolRoles {
// 					depositor: 10,
// 					root: Some(900),
// 					nominator: Some(901),
// 					bouncer: Some(902)
// 				},
// 			);

// 			// non-existent pools
// 			assert_noop!(
// 				Lst::update_roles(
// 					RuntimeOrigin::signed(1),
// 					2,
// 					ConfigOp::Set(5),
// 					ConfigOp::Set(6),
// 					ConfigOp::Set(7)
// 				),
// 				Error::<Runtime>::PoolNotFound,
// 			);

// 			// depositor cannot change roles.
// 			assert_noop!(
// 				Lst::update_roles(
// 					RuntimeOrigin::signed(1),
// 					1,
// 					ConfigOp::Set(5),
// 					ConfigOp::Set(6),
// 					ConfigOp::Set(7)
// 				),
// 				Error::<Runtime>::DoesNotHavePermission,
// 			);

// 			// nominator cannot change roles.
// 			assert_noop!(
// 				Lst::update_roles(
// 					RuntimeOrigin::signed(901),
// 					1,
// 					ConfigOp::Set(5),
// 					ConfigOp::Set(6),
// 					ConfigOp::Set(7)
// 				),
// 				Error::<Runtime>::DoesNotHavePermission,
// 			);
// 			// bouncer
// 			assert_noop!(
// 				Lst::update_roles(
// 					RuntimeOrigin::signed(902),
// 					1,
// 					ConfigOp::Set(5),
// 					ConfigOp::Set(6),
// 					ConfigOp::Set(7)
// 				),
// 				Error::<Runtime>::DoesNotHavePermission,
// 			);

// 			// but root can
// 			assert_ok!(Lst::update_roles(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				ConfigOp::Set(5),
// 				ConfigOp::Set(6),
// 				ConfigOp::Set(7)
// 			));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::RolesUpdated { root: Some(5), bouncer: Some(7), nominator: Some(6) }
// 				]
// 			);
// 			assert_eq!(
// 				BondedPools::<Runtime>::get(1).unwrap().roles,
// 				PoolRoles { depositor: 10, root: Some(5), nominator: Some(6), bouncer: Some(7) },
// 			);

// 			// also root origin can
// 			assert_ok!(Lst::update_roles(
// 				RuntimeOrigin::root(),
// 				1,
// 				ConfigOp::Set(1),
// 				ConfigOp::Set(2),
// 				ConfigOp::Set(3)
// 			));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::RolesUpdated { root: Some(1), bouncer: Some(3), nominator: Some(2) }]
// 			);
// 			assert_eq!(
// 				BondedPools::<Runtime>::get(1).unwrap().roles,
// 				PoolRoles { depositor: 10, root: Some(1), nominator: Some(2), bouncer: Some(3) },
// 			);

// 			// Noop works
// 			assert_ok!(Lst::update_roles(
// 				RuntimeOrigin::root(),
// 				1,
// 				ConfigOp::Set(11),
// 				ConfigOp::Noop,
// 				ConfigOp::Noop
// 			));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::RolesUpdated { root: Some(11), bouncer: Some(3), nominator: Some(2) }]
// 			);

// 			assert_eq!(
// 				BondedPools::<Runtime>::get(1).unwrap().roles,
// 				PoolRoles { depositor: 10, root: Some(11), nominator: Some(2), bouncer: Some(3) },
// 			);

// 			// Remove works
// 			assert_ok!(Lst::update_roles(
// 				RuntimeOrigin::root(),
// 				1,
// 				ConfigOp::Set(69),
// 				ConfigOp::Remove,
// 				ConfigOp::Remove
// 			));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::RolesUpdated { root: Some(69), bouncer: None, nominator: None }]
// 			);

// 			assert_eq!(
// 				BondedPools::<Runtime>::get(1).unwrap().roles,
// 				PoolRoles { depositor: 10, root: Some(69), nominator: None, bouncer: None },
// 			);
// 		})
// 	}
// }

// mod reward_counter_precision {
// 	use super::*;

// 	const DOT: Balance = 10u128.pow(10u32);
// 	const POLKADOT_TOTAL_ISSUANCE_GENESIS: Balance = DOT * 10u128.pow(9u32);

// 	const fn inflation(years: u128) -> u128 {
// 		let mut i = 0;
// 		let mut start = POLKADOT_TOTAL_ISSUANCE_GENESIS;
// 		while i < years {
// 			start = start + start / 10;
// 			i += 1
// 		}
// 		start
// 	}

// 	fn default_pool_reward_counter() -> FixedU128 {
// 		let bonded_pool = BondedPools::<T>::get(1).unwrap();
// 		RewardPools::<Runtime>::get(1)
// 			.unwrap()
// 			.current_reward_counter(1, bonded_pool.points(), bonded_pool.commission.current())
// 			.unwrap()
// 			.0
// 	}

// 	fn pending_rewards(of: AccountId) -> Option<BalanceOf<T>> {
// 		let member = PoolMembers::<T>::get(of).unwrap();
// 		assert_eq!(member.pool_id, 1);
// 		let rc = default_pool_reward_counter();
// 		member.pending_rewards(rc).ok()
// 	}

// 	#[test]
// 	fn smallest_claimable_reward() {
// 		// create a pool that has all of the polkadot issuance in 50 years.
// 		let pool_bond = inflation(50);
// 		ExtBuilder::default().ed(DOT).min_bond(pool_bond).build_and_execute(|| {
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded {
// 						member: 10,
// 						pool_id: 1,
// 						bonded: 1173908528796953165005,
// 						joined: true,
// 					}
// 				]
// 			);

// 			// the smallest reward that this pool can handle is
// 			let expected_smallest_reward = inflation(50) / 10u128.pow(18);

// 			// tad bit less. cannot be paid out.
// 			deposit_rewards(expected_smallest_reward - 1);
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_eq!(pool_events_since_last_call(), vec![]);
// 			// revert it.

// 			remove_rewards(expected_smallest_reward - 1);

// 			// tad bit more. can be claimed.
// 			deposit_rewards(expected_smallest_reward + 1);
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PaidOut { member: 10, pool_id: 1, payout: 1173 }]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn massive_reward_in_small_pool() {
// 		let tiny_bond = 1000 * DOT;
// 		ExtBuilder::default().ed(DOT).min_bond(tiny_bond).build_and_execute(|| {
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10000000000000, joined: true }
// 				]
// 			);

// 			Currency::set_balance(&20, tiny_bond);
// 			assert_ok!(Lst::join(RuntimeOrigin::signed(20), tiny_bond / 2, 1));

// 			// Suddenly, add a shit ton of rewards.
// 			deposit_rewards(inflation(1));

// 			// now claim.
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Bonded { member: 20, pool_id: 1, bonded: 5000000000000, joined: true },
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 7333333333333333333 },
// 					Event::PaidOut { member: 20, pool_id: 1, payout: 3666666666666666666 }
// 				]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn reward_counter_calc_wont_fail_in_normal_polkadot_future() {
// 		// create a pool that has roughly half of the polkadot issuance in 10 years.
// 		let pool_bond = inflation(10) / 2;
// 		ExtBuilder::default().ed(DOT).min_bond(pool_bond).build_and_execute(|| {
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded {
// 						member: 10,
// 						pool_id: 1,
// 						bonded: 12_968_712_300_500_000_000,
// 						joined: true,
// 					}
// 				]
// 			);

// 			// in 10 years, the total claimed rewards are large values as well. assuming that a pool
// 			// is earning all of the inflation per year (which is really unrealistic, but worse
// 			// case), that will be:
// 			let pool_total_earnings_10_years = inflation(10) - POLKADOT_TOTAL_ISSUANCE_GENESIS;
// 			deposit_rewards(pool_total_earnings_10_years);

// 			// some whale now joins with the other half ot the total issuance. This will bloat all
// 			// the calculation regarding current reward counter.
// 			Currency::set_balance(&20, pool_bond * 2);
// 			assert_ok!(Lst::join(RuntimeOrigin::signed(20), pool_bond, 1));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::Bonded {
// 					member: 20,
// 					pool_id: 1,
// 					bonded: 12_968_712_300_500_000_000,
// 					joined: true
// 				}]
// 			);

// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PaidOut { member: 10, pool_id: 1, payout: 15937424600999999996 }]
// 			);

// 			// now let a small member join with 10 DOTs.
// 			Currency::set_balance(&30, 20 * DOT);
// 			assert_ok!(Lst::join(RuntimeOrigin::signed(30), 10 * DOT, 1));

// 			// and give a reasonably small reward to the pool.
// 			deposit_rewards(DOT);

// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(30)));
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Bonded { member: 30, pool_id: 1, bonded: 100000000000, joined: true },
// 					// quite small, but working fine.
// 					Event::PaidOut { member: 30, pool_id: 1, payout: 38 }
// 				]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn reward_counter_update_can_fail_if_pool_is_highly_slashed() {
// 		// create a pool that has roughly half of the polkadot issuance in 10 years.
// 		let pool_bond = inflation(10) / 2;
// 		ExtBuilder::default().ed(DOT).min_bond(pool_bond).build_and_execute(|| {
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded {
// 						member: 10,
// 						pool_id: 1,
// 						bonded: 12_968_712_300_500_000_000,
// 						joined: true,
// 					}
// 				]
// 			);

// 			// slash this pool by 99% of that.
// 			StakingMock::slash_by(1, pool_bond * 99 / 100);

// 			// some whale now joins with the other half ot the total issuance. This will trigger an
// 			// overflow. This test is actually a bit too lenient because all the reward counters are
// 			// set to zero. In other tests that we want to assert a scenario won't fail, we should
// 			// also set the reward counters to some large value.
// 			Currency::set_balance(&20, pool_bond * 2);
// 			assert_err!(
// 				Lst::join(RuntimeOrigin::signed(20), pool_bond, 1),
// 				Error::<T>::OverflowRisk
// 			);
// 		})
// 	}

// 	#[test]
// 	fn if_small_member_waits_long_enough_they_will_earn_rewards() {
// 		// create a pool that has a quarter of the current polkadot issuance
// 		ExtBuilder::default()
// 			.ed(DOT)
// 			.min_bond(POLKADOT_TOTAL_ISSUANCE_GENESIS / 4)
// 			.build_and_execute(|| {
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![
// 						Event::Created { depositor: 10, pool_id: 1 },
// 						Event::Bonded {
// 							member: 10,
// 							pool_id: 1,
// 							bonded: 2500000000000000000,
// 							joined: true,
// 						}
// 					]
// 				);

// 				// and have a tiny fish join the pool as well..
// 				Currency::set_balance(&20, 20 * DOT);
// 				assert_ok!(Lst::join(RuntimeOrigin::signed(20), 10 * DOT, 1));

// 				// earn some small rewards
// 				deposit_rewards(DOT / 1000);

// 				// no point in claiming for 20 (nonetheless, it should be harmless)
// 				assert!(pending_rewards(20).unwrap().is_zero());
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![
// 						Event::Bonded {
// 							member: 20,
// 							pool_id: 1,
// 							bonded: 100000000000,
// 							joined: true
// 						},
// 						Event::PaidOut { member: 10, pool_id: 1, payout: 9999997 }
// 					]
// 				);

// 				// earn some small more, still nothing can be claimed for 20, but 10 claims their
// 				// share.
// 				deposit_rewards(DOT / 1000);
// 				assert!(pending_rewards(20).unwrap().is_zero());
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![Event::PaidOut { member: 10, pool_id: 1, payout: 10000000 }]
// 				);

// 				// earn some more rewards, this time 20 can also claim.
// 				deposit_rewards(DOT / 1000);
// 				assert_eq!(pending_rewards(20).unwrap(), 1);
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![
// 						Event::PaidOut { member: 10, pool_id: 1, payout: 10000000 },
// 						Event::PaidOut { member: 20, pool_id: 1, payout: 1 }
// 					]
// 				);
// 			});
// 	}

// 	#[test]
// 	fn zero_reward_claim_does_not_update_reward_counter() {
// 		// create a pool that has a quarter of the current polkadot issuance
// 		ExtBuilder::default()
// 			.ed(DOT)
// 			.min_bond(POLKADOT_TOTAL_ISSUANCE_GENESIS / 4)
// 			.build_and_execute(|| {
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![
// 						Event::Created { depositor: 10, pool_id: 1 },
// 						Event::Bonded {
// 							member: 10,
// 							pool_id: 1,
// 							bonded: 2500000000000000000,
// 							joined: true,
// 						}
// 					]
// 				);

// 				// and have a tiny fish join the pool as well..
// 				Currency::set_balance(&20, 20 * DOT);
// 				assert_ok!(Lst::join(RuntimeOrigin::signed(20), 10 * DOT, 1));

// 				// earn some small rewards
// 				deposit_rewards(DOT / 1000);

// 				// if 20 claims now, their reward counter should stay the same, so that they have a
// 				// chance of claiming this if they let it accumulate. Also see
// 				// `if_small_member_waits_long_enough_they_will_earn_rewards`
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));
// 				assert_eq!(
// 					pool_events_since_last_call(),
// 					vec![
// 						Event::Bonded {
// 							member: 20,
// 							pool_id: 1,
// 							bonded: 100000000000,
// 							joined: true
// 						},
// 						Event::PaidOut { member: 10, pool_id: 1, payout: 9999997 }
// 					]
// 				);

// 				let current_reward_counter = default_pool_reward_counter();
// 				// has been updated, because they actually claimed something.
// 				assert_eq!(
// 					PoolMembers::<T>::get(10).unwrap().last_recorded_reward_counter,
// 					current_reward_counter
// 				);
// 				// has not be updated, even though the claim transaction went through okay.
// 				assert_eq!(
// 					PoolMembers::<T>::get(20).unwrap().last_recorded_reward_counter,
// 					Default::default()
// 				);
// 			});
// 	}
// }

// mod commission {
// 	use super::*;

// 	#[test]
// 	fn set_commission_works() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			let pool_id = 1;
// 			let root = 900;

// 			// Commission can be set by the `root` role.

// 			// When:
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(root),
// 				pool_id,
// 				Some((Perbill::from_percent(50), root))
// 			));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id },
// 					Event::Bonded { member: 10, pool_id, bonded: 10, joined: true },
// 					Event::PoolCommissionUpdated {
// 						pool_id,
// 						current: Some((Perbill::from_percent(50), root))
// 					},
// 				]
// 			);

// 			// Commission can be updated only, while keeping the same payee.

// 			// When:
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(root),
// 				1,
// 				Some((Perbill::from_percent(25), root))
// 			));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PoolCommissionUpdated {
// 					pool_id,
// 					current: Some((Perbill::from_percent(25), root))
// 				},]
// 			);

// 			// Payee can be updated only, while keeping the same commission.

// 			// Given:
// 			let payee = 901;

// 			// When:
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(root),
// 				pool_id,
// 				Some((Perbill::from_percent(25), payee))
// 			));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PoolCommissionUpdated {
// 					pool_id: 1,
// 					current: Some((Perbill::from_percent(25), payee))
// 				},]
// 			);

// 			// Pool earns 80 points and a payout is triggered.

// 			// Given:
// 			deposit_rewards(80);
// 			assert_eq!(
// 				PoolMembers::<Runtime>::get(10).unwrap(),
// 				PoolMember::<Runtime> { pool_id, points: 10, ..Default::default() }
// 			);

// 			// When:
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PaidOut { member: 10, pool_id, payout: 60 }]
// 			);
// 			assert_eq!(RewardPool::<Runtime>::current_balance(pool_id), 20);

// 			// Pending pool commission can be claimed by the root role.

// 			// When:
// 			assert_ok!(Lst::claim_commission(RuntimeOrigin::signed(root), pool_id));

// 			// Then:
// 			assert_eq!(RewardPool::<Runtime>::current_balance(pool_id), 0);
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PoolCommissionClaimed { pool_id: 1, commission: 20 }]
// 			);

// 			// Commission can be removed from the pool completely.

// 			// When:
// 			assert_ok!(Lst::set_commission(RuntimeOrigin::signed(root), pool_id, None));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PoolCommissionUpdated { pool_id, current: None },]
// 			);

// 			// Given a pool now has a reward counter history, additional rewards and payouts can be
// 			// made while maintaining a correct ledger of the reward pool. Pool earns 100 points,
// 			// payout is triggered.
// 			//
// 			// Note that the `total_commission_pending` will not be updated until `update_records`
// 			// is next called, which is not done in this test segment..

// 			// Given:
// 			deposit_rewards(100);

// 			// When:
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PaidOut { member: 10, pool_id, payout: 100 },]
// 			);
// 			assert_eq!(
// 				RewardPools::<Runtime>::get(pool_id).unwrap(),
// 				RewardPool {
// 					last_recorded_reward_counter: FixedU128::from_float(6.0),
// 					last_recorded_total_payouts: 80,
// 					total_rewards_claimed: 160,
// 					total_commission_pending: 0,
// 					total_commission_claimed: 20
// 				}
// 			);

// 			// When set commission is called again, update_records is called and
// 			// `total_commission_pending` is updated, based on the current reward counter and pool
// 			// balance.
// 			//
// 			// Note that commission is now 0%, so it should not come into play with subsequent
// 			// payouts.

// 			// When:
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(root),
// 				1,
// 				Some((Perbill::from_percent(10), root))
// 			));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PoolCommissionUpdated {
// 					pool_id,
// 					current: Some((Perbill::from_percent(10), root))
// 				},]
// 			);
// 			assert_eq!(
// 				RewardPools::<Runtime>::get(pool_id).unwrap(),
// 				RewardPool {
// 					last_recorded_reward_counter: FixedU128::from_float(16.0),
// 					last_recorded_total_payouts: 180,
// 					total_rewards_claimed: 160,
// 					total_commission_pending: 0,
// 					total_commission_claimed: 20
// 				}
// 			);

// 			// Supplying a 0% commission along with a payee results in a `None` current value.

// 			// When:
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(root),
// 				pool_id,
// 				Some((Perbill::from_percent(0), root))
// 			));

// 			// Then:
// 			assert_eq!(
// 				BondedPool::<Runtime>::get(1).unwrap().commission,
// 				Commission {
// 					current: None,
// 					max: None,
// 					change_rate: None,
// 					throttle_from: Some(1),
// 					claim_permission: None,
// 				}
// 			);
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PoolCommissionUpdated {
// 					pool_id,
// 					current: Some((Perbill::from_percent(0), root))
// 				},]
// 			);

// 			// The payee can be updated even when commission has reached maximum commission. Both
// 			// commission and max commission are set to 10% to test this.

// 			// Given:
// 			assert_ok!(Lst::set_commission_max(
// 				RuntimeOrigin::signed(root),
// 				pool_id,
// 				Perbill::from_percent(10)
// 			));
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(root),
// 				pool_id,
// 				Some((Perbill::from_percent(10), root))
// 			));

// 			// When:
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(root),
// 				pool_id,
// 				Some((Perbill::from_percent(10), payee))
// 			));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PoolMaxCommissionUpdated {
// 						pool_id,
// 						max_commission: Perbill::from_percent(10)
// 					},
// 					Event::PoolCommissionUpdated {
// 						pool_id,
// 						current: Some((Perbill::from_percent(10), root))
// 					},
// 					Event::PoolCommissionUpdated {
// 						pool_id,
// 						current: Some((Perbill::from_percent(10), payee))
// 					}
// 				]
// 			);
// 		});
// 	}

// 	#[test]
// 	fn commission_reward_counter_works_one_member() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			let pool_id = 1;
// 			let root = 900;
// 			let member = 10;

// 			// Set the pool commission to 10% to test commission shares. Pool is topped up 40 points
// 			// and `member` immediately claims their pending rewards. Reward pool should still have
// 			// 10% share.

// 			// Given:
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(root),
// 				1,
// 				Some((Perbill::from_percent(10), root)),
// 			));
// 			deposit_rewards(40);

// 			// When:
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 			// Then:
// 			assert_eq!(RewardPool::<Runtime>::current_balance(pool_id), 4);

// 			// Set pool commission to 20% and repeat the same process.

// 			// When:
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(root),
// 				1,
// 				Some((Perbill::from_percent(20), root)),
// 			));

// 			// Then:
// 			assert_eq!(
// 				RewardPools::<Runtime>::get(pool_id).unwrap(),
// 				RewardPool {
// 					last_recorded_reward_counter: FixedU128::from_float(3.6),
// 					last_recorded_total_payouts: 40,
// 					total_rewards_claimed: 36,
// 					total_commission_pending: 4,
// 					total_commission_claimed: 0
// 				}
// 			);

// 			// The current reward counter should yield the correct pending rewards of zero.

// 			// Given:
// 			let (current_reward_counter, _) = RewardPools::<Runtime>::get(pool_id)
// 				.unwrap()
// 				.current_reward_counter(
// 					pool_id,
// 					BondedPools::<Runtime>::get(pool_id).unwrap().points,
// 					Perbill::from_percent(20),
// 				)
// 				.unwrap();

// 			// Then:
// 			assert_eq!(
// 				PoolMembers::<Runtime>::get(member)
// 					.unwrap()
// 					.pending_rewards(current_reward_counter)
// 					.unwrap(),
// 				0
// 			);

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::PoolCommissionUpdated {
// 						pool_id: 1,
// 						current: Some((Perbill::from_percent(10), 900))
// 					},
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 36 },
// 					Event::PoolCommissionUpdated {
// 						pool_id: 1,
// 						current: Some((Perbill::from_percent(20), 900))
// 					}
// 				]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn set_commission_handles_errors() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 				]
// 			);

// 			// Provided pool does not exist.
// 			assert_noop!(
// 				Lst::set_commission(
// 					RuntimeOrigin::signed(900),
// 					9999,
// 					Some((Perbill::from_percent(1), 900)),
// 				),
// 				Error::<Runtime>::PoolNotFound
// 			);

// 			// Sender does not have permission to set commission.
// 			assert_noop!(
// 				Lst::set_commission(
// 					RuntimeOrigin::signed(1),
// 					1,
// 					Some((Perbill::from_percent(5), 900)),
// 				),
// 				Error::<Runtime>::DoesNotHavePermission
// 			);

// 			// Commission increases will be throttled if outside of change_rate allowance.
// 			// Commission is set to 5%.
// 			// Change rate is set to 1% max increase, 2 block delay.

// 			// When:
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				Some((Perbill::from_percent(5), 900)),
// 			));
// 			assert_ok!(Lst::set_commission_change_rate(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				CommissionChangeRate { max_increase: Perbill::from_percent(1), min_delay: 2_u64 }
// 			));
// 			assert_eq!(
// 				BondedPool::<Runtime>::get(1).unwrap().commission,
// 				Commission {
// 					current: Some((Perbill::from_percent(5), 900)),
// 					max: None,
// 					change_rate: Some(CommissionChangeRate {
// 						max_increase: Perbill::from_percent(1),
// 						min_delay: 2_u64
// 					}),
// 					throttle_from: Some(1_u64),
// 					claim_permission: None,
// 				}
// 			);
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PoolCommissionUpdated {
// 						pool_id: 1,
// 						current: Some((Perbill::from_percent(5), 900))
// 					},
// 					Event::PoolCommissionChangeRateUpdated {
// 						pool_id: 1,
// 						change_rate: CommissionChangeRate {
// 							max_increase: Perbill::from_percent(1),
// 							min_delay: 2
// 						}
// 					}
// 				]
// 			);

// 			// Now try to increase commission to 10% (5% increase). This should be throttled.
// 			// Then:
// 			assert_noop!(
// 				Lst::set_commission(
// 					RuntimeOrigin::signed(900),
// 					1,
// 					Some((Perbill::from_percent(10), 900))
// 				),
// 				Error::<Runtime>::CommissionChangeThrottled
// 			);

// 			run_blocks(2);

// 			// Increase commission by 1% and provide an initial payee. This should succeed and set
// 			// the `throttle_from` field.

// 			// When:
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				Some((Perbill::from_percent(6), 900))
// 			));
// 			assert_eq!(
// 				BondedPool::<Runtime>::get(1).unwrap().commission,
// 				Commission {
// 					current: Some((Perbill::from_percent(6), 900)),
// 					max: None,
// 					change_rate: Some(CommissionChangeRate {
// 						max_increase: Perbill::from_percent(1),
// 						min_delay: 2_u64
// 					}),
// 					throttle_from: Some(3_u64),
// 					claim_permission: None,
// 				}
// 			);
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PoolCommissionUpdated {
// 					pool_id: 1,
// 					current: Some((Perbill::from_percent(6), 900))
// 				},]
// 			);

// 			// Attempt to increase the commission an additional 1% (now 7%). This will fail as
// 			// `throttle_from` is now the current block. At least 2 blocks need to pass before we
// 			// can set commission again.

// 			// Then:
// 			assert_noop!(
// 				Lst::set_commission(
// 					RuntimeOrigin::signed(900),
// 					1,
// 					Some((Perbill::from_percent(7), 900))
// 				),
// 				Error::<Runtime>::CommissionChangeThrottled
// 			);

// 			run_blocks(2);

// 			// Can now successfully increase the commission again, to 7%.

// 			// When:
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				Some((Perbill::from_percent(7), 900)),
// 			));
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PoolCommissionUpdated {
// 					pool_id: 1,
// 					current: Some((Perbill::from_percent(7), 900))
// 				},]
// 			);

// 			run_blocks(2);

// 			// Now surpassed the `min_delay` threshold, but the `max_increase` threshold is
// 			// still at play. An attempted commission change now to 8% (+2% increase) should fail.

// 			// Then:
// 			assert_noop!(
// 				Lst::set_commission(
// 					RuntimeOrigin::signed(900),
// 					1,
// 					Some((Perbill::from_percent(9), 900)),
// 				),
// 				Error::<Runtime>::CommissionChangeThrottled
// 			);

// 			// Now set a max commission to the current 5%. This will also update the current
// 			// commission to 5%.

// 			// When:
// 			assert_ok!(Lst::set_commission_max(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				Perbill::from_percent(5)
// 			));
// 			assert_eq!(
// 				BondedPool::<Runtime>::get(1).unwrap().commission,
// 				Commission {
// 					current: Some((Perbill::from_percent(5), 900)),
// 					max: Some(Perbill::from_percent(5)),
// 					change_rate: Some(CommissionChangeRate {
// 						max_increase: Perbill::from_percent(1),
// 						min_delay: 2
// 					}),
// 					throttle_from: Some(7),
// 					claim_permission: None,
// 				}
// 			);
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PoolCommissionUpdated {
// 						pool_id: 1,
// 						current: Some((Perbill::from_percent(5), 900))
// 					},
// 					Event::PoolMaxCommissionUpdated {
// 						pool_id: 1,
// 						max_commission: Perbill::from_percent(5)
// 					}
// 				]
// 			);

// 			// Run 2 blocks into the future so we are eligible to update commission again.
// 			run_blocks(2);

// 			// Now attempt again to increase the commission by 1%, to 6%. This is within the change
// 			// rate allowance, but `max_commission` will now prevent us from going any higher.

// 			// Then:
// 			assert_noop!(
// 				Lst::set_commission(
// 					RuntimeOrigin::signed(900),
// 					1,
// 					Some((Perbill::from_percent(6), 900)),
// 				),
// 				Error::<Runtime>::CommissionExceedsMaximum
// 			);
// 		});
// 	}

// 	#[test]
// 	fn set_commission_max_works_with_error_tests() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// Provided pool does not exist
// 			assert_noop!(
// 				Lst::set_commission_max(RuntimeOrigin::signed(900), 9999, Perbill::from_percent(1)),
// 				Error::<Runtime>::PoolNotFound
// 			);
// 			// Sender does not have permission to set commission
// 			assert_noop!(
// 				Lst::set_commission_max(RuntimeOrigin::signed(1), 1, Perbill::from_percent(5)),
// 				Error::<Runtime>::DoesNotHavePermission
// 			);

// 			// Cannot set max commission above GlobalMaxCommission
// 			assert_noop!(
// 				Lst::set_commission_max(RuntimeOrigin::signed(900), 1, Perbill::from_percent(100)),
// 				Error::<Runtime>::CommissionExceedsGlobalMaximum
// 			);

// 			// Set a max commission commission pool 1 to 80%
// 			assert_ok!(Lst::set_commission_max(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				Perbill::from_percent(80)
// 			));
// 			assert_eq!(
// 				BondedPools::<Runtime>::get(1).unwrap().commission.max,
// 				Some(Perbill::from_percent(80))
// 			);

// 			// We attempt to increase the max commission to 90%, but increasing is
// 			// disallowed due to pool's max commission.
// 			assert_noop!(
// 				Lst::set_commission_max(RuntimeOrigin::signed(900), 1, Perbill::from_percent(90)),
// 				Error::<Runtime>::MaxCommissionRestricted
// 			);

// 			// We will now set a commission to 75% and then amend the max commission
// 			// to 50%. The max commission change should decrease the current
// 			// commission to 50%.
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				Some((Perbill::from_percent(75), 900))
// 			));
// 			assert_ok!(Lst::set_commission_max(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				Perbill::from_percent(50)
// 			));
// 			assert_eq!(
// 				BondedPools::<Runtime>::get(1).unwrap().commission,
// 				Commission {
// 					current: Some((Perbill::from_percent(50), 900)),
// 					max: Some(Perbill::from_percent(50)),
// 					change_rate: None,
// 					throttle_from: Some(1),
// 					claim_permission: None,
// 				}
// 			);

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::PoolMaxCommissionUpdated {
// 						pool_id: 1,
// 						max_commission: Perbill::from_percent(80)
// 					},
// 					Event::PoolCommissionUpdated {
// 						pool_id: 1,
// 						current: Some((Perbill::from_percent(75), 900))
// 					},
// 					Event::PoolCommissionUpdated {
// 						pool_id: 1,
// 						current: Some((Perbill::from_percent(50), 900))
// 					},
// 					Event::PoolMaxCommissionUpdated {
// 						pool_id: 1,
// 						max_commission: Perbill::from_percent(50)
// 					}
// 				]
// 			);
// 		});
// 	}

// 	#[test]
// 	fn set_commission_change_rate_works_with_errors() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// Provided pool does not exist
// 			assert_noop!(
// 				Lst::set_commission_change_rate(
// 					RuntimeOrigin::signed(900),
// 					9999,
// 					CommissionChangeRate {
// 						max_increase: Perbill::from_percent(5),
// 						min_delay: 1000_u64
// 					}
// 				),
// 				Error::<Runtime>::PoolNotFound
// 			);
// 			// Sender does not have permission to set commission
// 			assert_noop!(
// 				Lst::set_commission_change_rate(
// 					RuntimeOrigin::signed(1),
// 					1,
// 					CommissionChangeRate {
// 						max_increase: Perbill::from_percent(5),
// 						min_delay: 1000_u64
// 					}
// 				),
// 				Error::<Runtime>::DoesNotHavePermission
// 			);

// 			// Set a commission change rate for pool 1
// 			assert_ok!(Lst::set_commission_change_rate(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				CommissionChangeRate { max_increase: Perbill::from_percent(5), min_delay: 10_u64 }
// 			));
// 			assert_eq!(
// 				BondedPools::<Runtime>::get(1).unwrap().commission.change_rate,
// 				Some(CommissionChangeRate {
// 					max_increase: Perbill::from_percent(5),
// 					min_delay: 10_u64
// 				})
// 			);
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::PoolCommissionChangeRateUpdated {
// 						pool_id: 1,
// 						change_rate: CommissionChangeRate {
// 							max_increase: Perbill::from_percent(5),
// 							min_delay: 10
// 						}
// 					},
// 				]
// 			);

// 			// We now try to half the min_delay - this will be disallowed. A greater delay between
// 			// commission changes is seen as more restrictive.
// 			assert_noop!(
// 				Lst::set_commission_change_rate(
// 					RuntimeOrigin::signed(900),
// 					1,
// 					CommissionChangeRate {
// 						max_increase: Perbill::from_percent(5),
// 						min_delay: 5_u64
// 					}
// 				),
// 				Error::<Runtime>::CommissionChangeRateNotAllowed
// 			);

// 			// We now try to increase the allowed max_increase - this will fail. A smaller allowed
// 			// commission change is seen as more restrictive.
// 			assert_noop!(
// 				Lst::set_commission_change_rate(
// 					RuntimeOrigin::signed(900),
// 					1,
// 					CommissionChangeRate {
// 						max_increase: Perbill::from_percent(10),
// 						min_delay: 10_u64
// 					}
// 				),
// 				Error::<Runtime>::CommissionChangeRateNotAllowed
// 			);

// 			// Successful more restrictive change of min_delay with the current max_increase
// 			assert_ok!(Lst::set_commission_change_rate(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				CommissionChangeRate { max_increase: Perbill::from_percent(5), min_delay: 20_u64 }
// 			));

// 			// Successful more restrictive change of max_increase with the current min_delay
// 			assert_ok!(Lst::set_commission_change_rate(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				CommissionChangeRate { max_increase: Perbill::from_percent(4), min_delay: 20_u64 }
// 			));

// 			// Successful more restrictive change of both max_increase and min_delay
// 			assert_ok!(Lst::set_commission_change_rate(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				CommissionChangeRate { max_increase: Perbill::from_percent(3), min_delay: 30_u64 }
// 			));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PoolCommissionChangeRateUpdated {
// 						pool_id: 1,
// 						change_rate: CommissionChangeRate {
// 							max_increase: Perbill::from_percent(5),
// 							min_delay: 20
// 						}
// 					},
// 					Event::PoolCommissionChangeRateUpdated {
// 						pool_id: 1,
// 						change_rate: CommissionChangeRate {
// 							max_increase: Perbill::from_percent(4),
// 							min_delay: 20
// 						}
// 					},
// 					Event::PoolCommissionChangeRateUpdated {
// 						pool_id: 1,
// 						change_rate: CommissionChangeRate {
// 							max_increase: Perbill::from_percent(3),
// 							min_delay: 30
// 						}
// 					}
// 				]
// 			);
// 		});
// 	}

// 	#[test]
// 	fn change_rate_does_not_apply_to_decreasing_commission() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// set initial commission of the pool to 10%.
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				Some((Perbill::from_percent(10), 900))
// 			));

// 			// Set a commission change rate for pool 1, 1% every 10 blocks
// 			assert_ok!(Lst::set_commission_change_rate(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				CommissionChangeRate { max_increase: Perbill::from_percent(1), min_delay: 10_u64 }
// 			));
// 			assert_eq!(
// 				BondedPools::<Runtime>::get(1).unwrap().commission.change_rate,
// 				Some(CommissionChangeRate {
// 					max_increase: Perbill::from_percent(1),
// 					min_delay: 10_u64
// 				})
// 			);

// 			// run `min_delay` blocks to allow a commission update.
// 			run_blocks(10_u64);

// 			// Test `max_increase`: attempt to decrease the commission by 5%. Should succeed.
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				Some((Perbill::from_percent(5), 900))
// 			));

// 			// Test `min_delay`: *immediately* attempt to decrease the commission by 2%. Should
// 			// succeed.
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				Some((Perbill::from_percent(3), 900))
// 			));

// 			// Attempt to *increase* the commission by 5%. Should fail.
// 			assert_noop!(
// 				Lst::set_commission(
// 					RuntimeOrigin::signed(900),
// 					1,
// 					Some((Perbill::from_percent(8), 900))
// 				),
// 				Error::<Runtime>::CommissionChangeThrottled
// 			);

// 			// Sanity check: the resulting pool Commission state.
// 			assert_eq!(
// 				BondedPools::<Runtime>::get(1).unwrap().commission,
// 				Commission {
// 					current: Some((Perbill::from_percent(3), 900)),
// 					max: None,
// 					change_rate: Some(CommissionChangeRate {
// 						max_increase: Perbill::from_percent(1),
// 						min_delay: 10_u64
// 					}),
// 					throttle_from: Some(11),
// 					claim_permission: None,
// 				}
// 			);

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::PoolCommissionUpdated {
// 						pool_id: 1,
// 						current: Some((Perbill::from_percent(10), 900))
// 					},
// 					Event::PoolCommissionChangeRateUpdated {
// 						pool_id: 1,
// 						change_rate: CommissionChangeRate {
// 							max_increase: Perbill::from_percent(1),
// 							min_delay: 10
// 						}
// 					},
// 					Event::PoolCommissionUpdated {
// 						pool_id: 1,
// 						current: Some((Perbill::from_percent(5), 900))
// 					},
// 					Event::PoolCommissionUpdated {
// 						pool_id: 1,
// 						current: Some((Perbill::from_percent(3), 900))
// 					}
// 				]
// 			);
// 		});
// 	}

// 	#[test]
// 	fn set_commission_max_to_zero_works() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// 0% max commission test.
// 			// set commission max 0%.
// 			assert_ok!(Lst::set_commission_max(RuntimeOrigin::signed(900), 1, Zero::zero()));

// 			// a max commission of 0% essentially freezes the current commission, even when None.
// 			// All commission update attempts will fail.
// 			assert_noop!(
// 				Lst::set_commission(
// 					RuntimeOrigin::signed(900),
// 					1,
// 					Some((Perbill::from_percent(1), 900))
// 				),
// 				Error::<Runtime>::CommissionExceedsMaximum
// 			);
// 		})
// 	}

// 	#[test]
// 	fn set_commission_change_rate_zero_max_increase_works() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// set commission change rate to 0% per 10 blocks
// 			assert_ok!(Lst::set_commission_change_rate(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				CommissionChangeRate { max_increase: Perbill::from_percent(0), min_delay: 10_u64 }
// 			));

// 			// even though there is a min delay of 10 blocks, a max increase of 0% essentially
// 			// freezes the commission. All commission update attempts will fail.
// 			assert_noop!(
// 				Lst::set_commission(
// 					RuntimeOrigin::signed(900),
// 					1,
// 					Some((Perbill::from_percent(1), 900))
// 				),
// 				Error::<Runtime>::CommissionChangeThrottled
// 			);
// 		})
// 	}

// 	#[test]
// 	fn set_commission_change_rate_zero_min_delay_works() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// set commission change rate to 1% with a 0 block `min_delay`.
// 			assert_ok!(Lst::set_commission_change_rate(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				CommissionChangeRate { max_increase: Perbill::from_percent(1), min_delay: 0_u64 }
// 			));
// 			assert_eq!(
// 				BondedPools::<Runtime>::get(1).unwrap().commission,
// 				Commission {
// 					current: None,
// 					max: None,
// 					change_rate: Some(CommissionChangeRate {
// 						max_increase: Perbill::from_percent(1),
// 						min_delay: 0
// 					}),
// 					throttle_from: Some(1),
// 					claim_permission: None,
// 				}
// 			);

// 			// since there is no min delay, we should be able to immediately set the commission.
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				Some((Perbill::from_percent(1), 900))
// 			));

// 			// sanity check: increasing again to more than +1% will fail.
// 			assert_noop!(
// 				Lst::set_commission(
// 					RuntimeOrigin::signed(900),
// 					1,
// 					Some((Perbill::from_percent(3), 900))
// 				),
// 				Error::<Runtime>::CommissionChangeThrottled
// 			);
// 		})
// 	}

// 	#[test]
// 	fn set_commission_change_rate_zero_value_works() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// Check zero values play nice. 0 `min_delay` and 0% max_increase test.
// 			// set commission change rate to 0% per 0 blocks.
// 			assert_ok!(Lst::set_commission_change_rate(
// 				RuntimeOrigin::signed(900),
// 				1,
// 				CommissionChangeRate { max_increase: Perbill::from_percent(0), min_delay: 0_u64 }
// 			));

// 			// even though there is no min delay, a max increase of 0% essentially freezes the
// 			// commission. All commission update attempts will fail.
// 			assert_noop!(
// 				Lst::set_commission(
// 					RuntimeOrigin::signed(900),
// 					1,
// 					Some((Perbill::from_percent(1), 900))
// 				),
// 				Error::<Runtime>::CommissionChangeThrottled
// 			);

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::PoolCommissionChangeRateUpdated {
// 						pool_id: 1,
// 						change_rate: CommissionChangeRate {
// 							max_increase: Perbill::from_percent(0),
// 							min_delay: 0_u64
// 						}
// 					}
// 				]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn do_reward_payout_with_various_commissions() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// turn off GlobalMaxCommission for this test.
// 			GlobalMaxCommission::<Runtime>::set(None);
// 			let pool_id = 1;

// 			// top up commission payee account to existential deposit
// 			let _ = Currency::set_balance(&2, 5);

// 			// Set a commission pool 1 to 33%, with a payee set to `2`
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				pool_id,
// 				Some((Perbill::from_percent(33), 2)),
// 			));
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::PoolCommissionUpdated {
// 						pool_id: 1,
// 						current: Some((Perbill::from_percent(33), 2))
// 					},
// 				]
// 			);

// 			// The pool earns 10 points
// 			deposit_rewards(10);
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PaidOut { member: 10, pool_id: 1, payout: 7 },]
// 			);

// 			// The pool earns 17 points
// 			deposit_rewards(17);
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PaidOut { member: 10, pool_id: 1, payout: 11 },]
// 			);

// 			// The pool earns 50 points
// 			deposit_rewards(50);
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PaidOut { member: 10, pool_id: 1, payout: 34 },]
// 			);

// 			// The pool earns 10439 points
// 			deposit_rewards(10439);
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PaidOut { member: 10, pool_id: 1, payout: 6994 },]
// 			);

// 			// Set the commission to 100% and ensure the following payout to the pool member will
// 			// not happen.

// 			// When:
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				pool_id,
// 				Some((Perbill::from_percent(100), 2)),
// 			));

// 			// Given:
// 			deposit_rewards(200);
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PoolCommissionUpdated {
// 					pool_id: 1,
// 					current: Some((Perbill::from_percent(100), 2))
// 				},]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn commission_accumulates_on_multiple_rewards() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			let pool_id = 1;

// 			// Given:

// 			// Set initial commission of pool 1 to 10%.
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				pool_id,
// 				Some((Perbill::from_percent(10), 2)),
// 			));
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::PoolCommissionUpdated {
// 						pool_id: 1,
// 						current: Some((Perbill::from_percent(10), 2))
// 					},
// 				]
// 			);

// 			// When:

// 			// The pool earns 100 points
// 			deposit_rewards(100);

// 			// Change commission to 20%
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				pool_id,
// 				Some((Perbill::from_percent(20), 2)),
// 			));
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![Event::PoolCommissionUpdated {
// 					pool_id: 1,
// 					current: Some((Perbill::from_percent(20), 2))
// 				},]
// 			);

// 			// The pool earns 100 points
// 			deposit_rewards(100);

// 			// Then:

// 			// Claim payout:
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 			// Claim commission:
// 			assert_ok!(Lst::claim_commission(RuntimeOrigin::signed(900), pool_id));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 90 + 80 },
// 					Event::PoolCommissionClaimed { pool_id: 1, commission: 30 }
// 				]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn last_recorded_total_payouts_needs_commission() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			let pool_id = 1;

// 			// Given:

// 			// Set initial commission of pool 1 to 10%.
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				pool_id,
// 				Some((Perbill::from_percent(10), 2)),
// 			));
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::PoolCommissionUpdated {
// 						pool_id: 1,
// 						current: Some((Perbill::from_percent(10), 2))
// 					},
// 				]
// 			);

// 			// When:

// 			// The pool earns 100 points
// 			deposit_rewards(100);

// 			// Claim payout:
// 			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));

// 			// Claim commission:
// 			assert_ok!(Lst::claim_commission(RuntimeOrigin::signed(900), pool_id));

// 			// Then:

// 			assert_eq!(
// 				RewardPools::<Runtime>::get(1).unwrap().last_recorded_total_payouts,
// 				90 + 10
// 			);

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 90 },
// 					Event::PoolCommissionClaimed { pool_id: 1, commission: 10 }
// 				]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn do_reward_payout_with_100_percent_commission() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// turn off GlobalMaxCommission for this test.
// 			GlobalMaxCommission::<Runtime>::set(None);

// 			let (mut member, bonded_pool, mut reward_pool) =
// 				Lst::get_member_with_pools(&10).unwrap();

// 			// top up commission payee account to existential deposit
// 			let _ = Currency::set_balance(&2, 5);

// 			// Set a commission pool 1 to 100%, with a payee set to `2`
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				bonded_pool.id,
// 				Some((Perbill::from_percent(100), 2)),
// 			));

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 					Event::PoolCommissionUpdated {
// 						pool_id: 1,
// 						current: Some((Perbill::from_percent(100), 2))
// 					}
// 				]
// 			);

// 			// The pool earns 10 points
// 			deposit_rewards(10);

// 			// execute the payout
// 			assert_ok!(Lst::do_reward_payout(
// 				&10,
// 				&mut member,
// 				&mut BondedPool::<Runtime>::get(1).unwrap(),
// 				&mut reward_pool
// 			));
// 		})
// 	}

// 	#[test]
// 	fn global_max_caps_max_commission_payout() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			// Note: GlobalMaxCommission is set at 90%.

// 			let (mut member, bonded_pool, mut reward_pool) =
// 				Lst::get_member_with_pools(&10).unwrap();

// 			// top up the commission payee account to existential deposit
// 			let _ = Currency::set_balance(&2, 5);

// 			// Set a commission pool 1 to 100% fails.
// 			assert_noop!(
// 				Lst::set_commission(
// 					RuntimeOrigin::signed(900),
// 					bonded_pool.id,
// 					Some((Perbill::from_percent(100), 2)),
// 				),
// 				Error::<Runtime>::CommissionExceedsGlobalMaximum
// 			);
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id: 1 },
// 					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
// 				]
// 			);

// 			// Set pool commission to 90% and then set global max commission to 80%.
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				bonded_pool.id,
// 				Some((Perbill::from_percent(90), 2)),
// 			));
// 			GlobalMaxCommission::<Runtime>::set(Some(Perbill::from_percent(80)));

// 			// The pool earns 10 points
// 			deposit_rewards(10);

// 			// execute the payout
// 			assert_ok!(Lst::do_reward_payout(
// 				&10,
// 				&mut member,
// 				&mut BondedPool::<Runtime>::get(1).unwrap(),
// 				&mut reward_pool
// 			));

// 			// Confirm the commission was only 8 points out of 10 points, and the payout was 2 out
// 			// of 10 points, reflecting the 80% global max commission.
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PoolCommissionUpdated {
// 						pool_id: 1,
// 						current: Some((Perbill::from_percent(90), 2))
// 					},
// 					Event::PaidOut { member: 10, pool_id: 1, payout: 2 },
// 				]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn claim_commission_works() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			/// Deposit rewards into the pool and claim payout. This will set up pending commission
// 			/// to be tested in various scenarios.
// 			fn deposit_rewards_and_claim_payout(caller: AccountId, points: u128) {
// 				deposit_rewards(points);
// 				assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(caller)));
// 			}

// 			let pool_id = 1;

// 			let _ = Currency::set_balance(&900, 5);
// 			assert_ok!(Lst::set_commission(
// 				RuntimeOrigin::signed(900),
// 				pool_id,
// 				Some((Perbill::from_percent(50), 900))
// 			));
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id },
// 					Event::Bonded { member: 10, pool_id, bonded: 10, joined: true },
// 					Event::PoolCommissionUpdated {
// 						pool_id,
// 						current: Some((Perbill::from_percent(50), 900))
// 					},
// 				]
// 			);

// 			// Given:
// 			deposit_rewards_and_claim_payout(10, 100);
// 			assert_eq!(RewardPool::<Runtime>::current_balance(pool_id), 50);

// 			// Pool does not exist
// 			assert_noop!(
// 				Lst::claim_commission(RuntimeOrigin::signed(900), 9999,),
// 				Error::<Runtime>::PoolNotFound
// 			);

// 			// Does not have permission.
// 			assert_noop!(
// 				Lst::claim_commission(RuntimeOrigin::signed(10), pool_id,),
// 				Error::<Runtime>::DoesNotHavePermission
// 			);

// 			// When:
// 			assert_ok!(Lst::claim_commission(RuntimeOrigin::signed(900), pool_id));

// 			// Then:
// 			assert_eq!(RewardPool::<Runtime>::current_balance(pool_id), 0);

// 			// No more pending commission.
// 			assert_noop!(
// 				Lst::claim_commission(RuntimeOrigin::signed(900), pool_id,),
// 				Error::<Runtime>::NoPendingCommission
// 			);

// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PaidOut { member: 10, pool_id, payout: 50 },
// 					Event::PoolCommissionClaimed { pool_id: 1, commission: 50 }
// 				]
// 			);

// 			// The pool commission's claim_permission field is updated to `Permissionless` by the
// 			// root member, which means anyone can now claim commission for the pool.

// 			// Given:
// 			// Some random non-pool member to claim commission.
// 			let non_pool_member = 1001;
// 			let _ = Currency::set_balance(&non_pool_member, 5);

// 			// Set up pending commission.
// 			deposit_rewards_and_claim_payout(10, 100);
// 			assert_ok!(Lst::set_commission_claim_permission(
// 				RuntimeOrigin::signed(900),
// 				pool_id,
// 				Some(CommissionClaimPermission::Permissionless)
// 			));

// 			// When:
// 			assert_ok!(Lst::claim_commission(RuntimeOrigin::signed(non_pool_member), pool_id));

// 			// Then:
// 			assert_eq!(RewardPool::<Runtime>::current_balance(pool_id), 0);
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PaidOut { member: 10, pool_id, payout: 50 },
// 					Event::PoolCommissionClaimPermissionUpdated {
// 						pool_id: 1,
// 						permission: Some(CommissionClaimPermission::Permissionless)
// 					},
// 					Event::PoolCommissionClaimed { pool_id: 1, commission: 50 },
// 				]
// 			);

// 			// The pool commission's claim_permission is updated to an adhoc account by the root
// 			// member, which means now only that account (in addition to the root role) can claim
// 			// commission for the pool.

// 			// Given:
// 			// The account designated to claim commission.
// 			let designated_commission_claimer = 2001;
// 			let _ = Currency::set_balance(&designated_commission_claimer, 5);

// 			// Set up pending commission.
// 			deposit_rewards_and_claim_payout(10, 100);
// 			assert_ok!(Lst::set_commission_claim_permission(
// 				RuntimeOrigin::signed(900),
// 				pool_id,
// 				Some(CommissionClaimPermission::Account(designated_commission_claimer))
// 			));

// 			// When:
// 			// Previous claimer can no longer claim commission.
// 			assert_noop!(
// 				Lst::claim_commission(RuntimeOrigin::signed(1001), pool_id,),
// 				Error::<Runtime>::DoesNotHavePermission
// 			);
// 			// Designated claimer can claim commission.
// 			assert_ok!(Lst::claim_commission(
// 				RuntimeOrigin::signed(designated_commission_claimer),
// 				pool_id
// 			));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PaidOut { member: 10, pool_id, payout: 50 },
// 					Event::PoolCommissionClaimPermissionUpdated {
// 						pool_id: 1,
// 						permission: Some(CommissionClaimPermission::Account(2001))
// 					},
// 					Event::PoolCommissionClaimed { pool_id: 1, commission: 50 },
// 				]
// 			);

// 			// Even with an Account claim permission set, the `root` role of the pool can still
// 			// claim commission.

// 			// Given:
// 			deposit_rewards_and_claim_payout(10, 100);

// 			// When:
// 			assert_ok!(Lst::claim_commission(RuntimeOrigin::signed(900), pool_id));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PaidOut { member: 10, pool_id, payout: 50 },
// 					Event::PoolCommissionClaimed { pool_id: 1, commission: 50 },
// 				]
// 			);

// 			// The root role updates commission's claim_permission back to `None`, which results in
// 			// only the root member being able to claim commission for the pool.

// 			// Given:
// 			deposit_rewards_and_claim_payout(10, 100);

// 			// When:
// 			assert_ok!(Lst::set_commission_claim_permission(
// 				RuntimeOrigin::signed(900),
// 				pool_id,
// 				None
// 			));
// 			// Previous claimer can no longer claim commission.
// 			assert_noop!(
// 				Lst::claim_commission(
// 					RuntimeOrigin::signed(designated_commission_claimer),
// 					pool_id,
// 				),
// 				Error::<Runtime>::DoesNotHavePermission
// 			);
// 			// Root can claim commission.
// 			assert_ok!(Lst::claim_commission(RuntimeOrigin::signed(900), pool_id));

// 			// Then:
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::PaidOut { member: 10, pool_id, payout: 50 },
// 					Event::PoolCommissionClaimPermissionUpdated { pool_id: 1, permission: None },
// 					Event::PoolCommissionClaimed { pool_id: 1, commission: 50 },
// 				]
// 			);
// 		})
// 	}

// 	#[test]
// 	fn set_commission_claim_permission_handles_errors() {
// 		ExtBuilder::default().build_and_execute(|| {
// 			let pool_id = 1;

// 			let _ = Currency::set_balance(&900, 5);
// 			assert_eq!(
// 				pool_events_since_last_call(),
// 				vec![
// 					Event::Created { depositor: 10, pool_id },
// 					Event::Bonded { member: 10, pool_id, bonded: 10, joined: true },
// 				]
// 			);

// 			// Cannot operate on a non-existing pool.
// 			assert_noop!(
// 				Lst::set_commission_claim_permission(
// 					RuntimeOrigin::signed(10),
// 					90,
// 					Some(CommissionClaimPermission::Permissionless)
// 				),
// 				Error::<Runtime>::PoolNotFound
// 			);

// 			// Only the root role can change the commission claim permission.
// 			assert_noop!(
// 				Lst::set_commission_claim_permission(
// 					RuntimeOrigin::signed(10),
// 					pool_id,
// 					Some(CommissionClaimPermission::Permissionless)
// 				),
// 				Error::<Runtime>::DoesNotHavePermission
// 			);
// 		})
// 	}
// }
