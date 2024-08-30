use super::*;
use crate::{mock::Currency, Event};
use frame_support::traits::Currency as CurrencyT;
use frame_support::{assert_noop, assert_ok};

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

#[test]
fn member_unbond_open() {
	// depositor in pool, pool state open
	//   - member unbond above limit
	//   - member unbonds to 0
	//   - member cannot unbond between within limit and 0
	ExtBuilder::default()
		.min_join_bond(10)
		.add_members(vec![(20, 20)])
		.build_and_execute(|| {
			assert_eq!(TotalValueLocked::<T>::get(), 30);
			// can unbond to above limit
			assert_ok!(Lst::unbond(RuntimeOrigin::signed(20), 20, 1, 5));

			// tvl remains unchanged.
			assert_eq!(TotalValueLocked::<T>::get(), 30);

			// cannot go to below 10:
			assert_noop!(
				Lst::unbond(RuntimeOrigin::signed(20), 20, 1, 10),
				Error::<T>::MinimumBondNotMet
			);
		})
}

#[test]
fn member_kicked() {
	// depositor in pool, pool state blocked
	//   - member cannot be kicked to above limit
	//   - member cannot be kicked between within limit and 0
	//   - member kicked to 0
	ExtBuilder::default()
		.min_join_bond(10)
		.add_members(vec![(20, 20)])
		.build_and_execute(|| {
			unsafe_set_state(1, PoolState::Blocked);
			let kicker = DEFAULT_ROLES.bouncer.unwrap();

			// cannot be kicked to above the limit.
			assert_noop!(
				Lst::unbond(RuntimeOrigin::signed(kicker), 20, 1, 5),
				Error::<T>::PartialUnbondNotAllowedPermissionlessly
			);

			// cannot go to below 10:
			assert_noop!(
				Lst::unbond(RuntimeOrigin::signed(kicker), 20, 1, 15),
				Error::<T>::PartialUnbondNotAllowedPermissionlessly
			);

			// but they themselves can do an unbond
			assert_ok!(Lst::unbond(RuntimeOrigin::signed(20), 20, 1, 2));

			// can be kicked to 0.
			assert_ok!(Lst::unbond(RuntimeOrigin::signed(kicker), 20, 1, 18));
		})
}

#[test]
fn member_unbond_destroying() {
	// depositor in pool, pool state destroying
	//   - member cannot be permissionlessly unbonded to above limit
	//   - member cannot be permissionlessly unbonded between within limit and 0
	//   - member permissionlessly unbonded to 0
	ExtBuilder::default()
		.min_join_bond(10)
		.add_members(vec![(20, 20)])
		.build_and_execute(|| {
			unsafe_set_state(1, PoolState::Destroying);
			let random = 123;

			// cannot be kicked to above the limit.
			assert_noop!(
				Lst::unbond(RuntimeOrigin::signed(random), 20, 1, 5),
				Error::<T>::PartialUnbondNotAllowedPermissionlessly
			);

			// cannot go to below 10:
			assert_noop!(
				Lst::unbond(RuntimeOrigin::signed(random), 20, 1, 15),
				Error::<T>::PartialUnbondNotAllowedPermissionlessly
			);

			// but they themselves can do an unbond
			assert_ok!(Lst::unbond(RuntimeOrigin::signed(20), 20, 1, 2));

			// but can go to 0
			assert_ok!(Lst::unbond(RuntimeOrigin::signed(random), 20, 1, 18));
		})
}

#[test]
fn depositor_unbond_open() {
	// depositor in pool, pool state open
	//   - depositor  unbonds to above limit
	//   - depositor cannot unbond to below limit or 0
	ExtBuilder::default().min_join_bond(10).build_and_execute(|| {
		// give the depositor some extra funds.
		assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), 1, BondExtra::FreeBalance(10)));

		// can unbond to above the limit.
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 5));

		// cannot go to below 10:
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 10),
			Error::<T>::MinimumBondNotMet
		);

		// cannot go to 0 either.
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 15),
			Error::<T>::MinimumBondNotMet
		);
	})
}

#[test]
fn depositor_kick() {
	// depositor in pool, pool state blocked
	//   - depositor can never be kicked.
	ExtBuilder::default().min_join_bond(10).build_and_execute(|| {
		// give the depositor some extra funds.
		assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), 1, BondExtra::FreeBalance(10)));

		// set the stage
		unsafe_set_state(1, PoolState::Blocked);
		let kicker = DEFAULT_ROLES.bouncer.unwrap();

		// cannot be kicked to above limit.
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(kicker), 10, 1, 5),
			Error::<T>::PartialUnbondNotAllowedPermissionlessly
		);

		// or below the limit
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(kicker), 10, 1, 15),
			Error::<T>::PartialUnbondNotAllowedPermissionlessly
		);

		// or 0.
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(kicker), 10, 1, 20),
			Error::<T>::DoesNotHavePermission
		);

		// they themselves cannot do it either
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 20),
			Error::<T>::MinimumBondNotMet
		);
	})
}

#[test]
fn depositor_unbond_destroying_permissionless() {
	// depositor can never be permissionlessly unbonded.
	ExtBuilder::default().min_join_bond(10).build_and_execute(|| {
		// give the depositor some extra funds.
		assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), 1, BondExtra::FreeBalance(10)));

		// set the stage
		unsafe_set_state(1, PoolState::Destroying);
		let random = 123;

		// cannot be kicked to above limit.
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(random), 10, 1, 5),
			Error::<T>::PartialUnbondNotAllowedPermissionlessly
		);

		// or below the limit
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(random), 10, 1, 15),
			Error::<T>::PartialUnbondNotAllowedPermissionlessly
		);

		// or 0.
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(random), 10, 1, 20),
			Error::<T>::DoesNotHavePermission
		);

		// they themselves can do it in this case though.
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 20));
	})
}

#[test]
fn depositor_unbond_destroying_not_last_member() {
	// deposit in pool, pool state destroying
	//   - depositor can never leave if there is another member in the pool.
	ExtBuilder::default()
		.min_join_bond(10)
		.add_members(vec![(20, 20)])
		.build_and_execute(|| {
			// give the depositor some extra funds.
			assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), 1, BondExtra::FreeBalance(10)));

			// set the stage
			unsafe_set_state(1, PoolState::Destroying);

			// can go above the limit
			assert_ok!(Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 5));

			// but not below the limit
			assert_noop!(
				Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 10),
				Error::<T>::MinimumBondNotMet
			);

			// and certainly not zero
			assert_noop!(
				Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 15),
				Error::<T>::MinimumBondNotMet
			);
		})
}

#[test]
fn depositor_unbond_destroying_last_member() {
	// deposit in pool, pool state destroying
	//   - depositor can unbond to above limit always.
	//   - depositor cannot unbond to below limit if last.
	//   - depositor can unbond to 0 if last and destroying.
	ExtBuilder::default().min_join_bond(10).build_and_execute(|| {
		// give the depositor some extra funds.
		assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), 1, BondExtra::FreeBalance(10)));

		// set the stage
		unsafe_set_state(1, PoolState::Destroying);

		// can unbond to above the limit.
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 5));

		// still cannot go to below limit
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 10),
			Error::<T>::MinimumBondNotMet
		);

		// can go to 0 too.
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 15));
	})
}

#[test]
fn unbond_of_1_works() {
	ExtBuilder::default().build_and_execute(|| {
		unsafe_set_state(1, PoolState::Destroying);
		assert_ok!(fully_unbond_permissioned(10, 1));

		assert_eq!(
			SubPoolsStorage::<Runtime>::get(1).unwrap().with_era,
			unbonding_pools_with_era! { 3 => UnbondPool::<Runtime> { points: 10, balance: 10 }}
		);

		assert_eq!(
			BondedPool::<Runtime>::get(1).unwrap(),
			BondedPool {
				id: 1,
				inner: BondedPoolInner {
					commission: Commission::default(),
					roles: DEFAULT_ROLES,
					state: PoolState::Destroying,
				}
			}
		);

		assert_eq!(StakingMock::active_stake(&default_bonded_account()).unwrap(), 0);
	});
}

#[test]
fn unbond_of_3_works() {
	ExtBuilder::default()
		.add_members(vec![(40, 40), (550, 550)])
		.build_and_execute(|| {
			let ed = <Balances as CurrencyT<AccountId>>::minimum_balance();
			// Given a slash from 600 -> 500
			StakingMock::slash_by(1, 500);

			// and unclaimed rewards of 600.
			Currency::make_free_balance_be(&default_reward_account(), ed + 600);

			// When
			assert_ok!(fully_unbond_permissioned(40, 1));

			// Then
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap().with_era,
				unbonding_pools_with_era! { 3 => UnbondPool { points: 6, balance: 6 }}
			);
			assert_eq!(
				BondedPool::<Runtime>::get(1).unwrap(),
				BondedPool {
					id: 1,
					inner: BondedPoolInner {
						commission: Commission::default(),
						roles: DEFAULT_ROLES,
						state: PoolState::Open,
					}
				}
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { depositor: 10, pool_id: 1 },
					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
					Event::Bonded { member: 40, pool_id: 1, bonded: 40, joined: true },
					Event::Bonded { member: 550, pool_id: 1, bonded: 550, joined: true },
					Event::PoolSlashed { pool_id: 1, balance: 100 },
					Event::PaidOut { member: 40, pool_id: 1, payout: 40 },
					Event::Unbonded { member: 40, pool_id: 1, balance: 6, points: 6, era: 3 }
				]
			);

			assert_eq!(StakingMock::active_stake(&default_bonded_account()).unwrap(), 94);
			assert_eq!(Currency::free_balance(40), 40 + 40); // We claim rewards when unbonding

			// When
			unsafe_set_state(1, PoolState::Destroying);
			assert_ok!(fully_unbond_permissioned(550, 1));

			// Then
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap().with_era,
				unbonding_pools_with_era! { 3 => UnbondPool { points: 98, balance: 98 }}
			);
			assert_eq!(
				BondedPool::<Runtime>::get(1).unwrap(),
				BondedPool {
					id: 1,
					inner: BondedPoolInner {
						commission: Commission::default(),
						roles: DEFAULT_ROLES,
						state: PoolState::Destroying,
					}
				}
			);
			assert_eq!(StakingMock::active_stake(&default_bonded_account()).unwrap(), 2);
			assert_eq!(Currency::free_balance(550), 550 + 550);
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::PaidOut { member: 550, pool_id: 1, payout: 550 },
					Event::Unbonded { member: 550, pool_id: 1, points: 92, balance: 92, era: 3 }
				]
			);

			// When
			CurrentEra::set(3);
			assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(10), 40, 1, 0));
			assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(10), 550, 1, 0));
			assert_ok!(fully_unbond_permissioned(10, 1));

			// Then
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap().with_era,
				unbonding_pools_with_era! { 6 => UnbondPool { points: 2, balance: 2 }}
			);
			assert_eq!(
				BondedPool::<Runtime>::get(1).unwrap(),
				BondedPool {
					id: 1,
					inner: BondedPoolInner {
						commission: Commission::default(),
						roles: DEFAULT_ROLES,
						state: PoolState::Destroying,
					}
				}
			);
			assert_eq!(StakingMock::active_stake(&default_bonded_account()).unwrap(), 0);

			assert_eq!(Currency::free_balance(550), 550 + 550 + 92);
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Withdrawn { member: 40, pool_id: 1, points: 6, balance: 6 },
					Event::MemberRemoved { pool_id: 1, member: 40 },
					Event::Withdrawn { member: 550, pool_id: 1, points: 92, balance: 92 },
					Event::MemberRemoved { pool_id: 1, member: 550 },
					Event::PaidOut { member: 10, pool_id: 1, payout: 10 },
					Event::Unbonded { member: 10, pool_id: 1, points: 2, balance: 2, era: 6 }
				]
			);
		});
}

#[test]
fn unbond_merges_older_pools() {
	ExtBuilder::default().with_check(1).build_and_execute(|| {
		// Given
		assert_eq!(StakingMock::bonding_duration(), 3);
		SubPoolsStorage::<Runtime>::insert(
			1,
			SubPools {
				no_era: Default::default(),
				with_era: unbonding_pools_with_era! {
					3 => UnbondPool { balance: 10, points: 100 },
					1 + 3 => UnbondPool { balance: 20, points: 20 },
					2 + 3 => UnbondPool { balance: 101, points: 101}
				},
			},
		);
		unsafe_set_state(1, PoolState::Destroying);

		// When
		let current_era = 1 + TotalUnbondingPools::<Runtime>::get();
		CurrentEra::set(current_era);

		assert_ok!(fully_unbond_permissioned(10, 1));

		// Then
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(1).unwrap(),
			SubPools {
				no_era: UnbondPool { balance: 10 + 20, points: 100 + 20 },
				with_era: unbonding_pools_with_era! {
					2 + 3 => UnbondPool { balance: 101, points: 101},
					current_era + 3 => UnbondPool { balance: 10, points: 10 },
				},
			},
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
				Event::Unbonded { member: 10, pool_id: 1, points: 10, balance: 10, era: 9 }
			]
		);
	});
}

#[test]
fn unbond_kick_works() {
	// Kick: the pool is blocked and the caller is either the root or bouncer.
	ExtBuilder::default()
		.add_members(vec![(100, 100), (200, 200)])
		.build_and_execute(|| {
			// Given
			unsafe_set_state(1, PoolState::Blocked);
			let bonded_pool = BondedPool::<Runtime>::get(1).unwrap();
			assert_eq!(bonded_pool.roles.root.unwrap(), 900);
			assert_eq!(bonded_pool.roles.nominator.unwrap(), 901);
			assert_eq!(bonded_pool.roles.bouncer.unwrap(), 902);

			// When the nominator tries to kick, then its a noop
			assert_noop!(
				Lst::fully_unbond(RuntimeOrigin::signed(901), 100, 1),
				Error::<Runtime>::NotKickerOrDestroying
			);

			// When the root kicks then its ok
			// Account with ID 100 is kicked.
			assert_ok!(Lst::fully_unbond(RuntimeOrigin::signed(900), 100, 1));

			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { depositor: 10, pool_id: 1 },
					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
					Event::Bonded { member: 100, pool_id: 1, bonded: 100, joined: true },
					Event::Bonded { member: 200, pool_id: 1, bonded: 200, joined: true },
					Event::Unbonded { member: 100, pool_id: 1, points: 100, balance: 100, era: 3 },
				]
			);

			// When the bouncer kicks then its ok
			// Account with ID 200 is kicked.
			assert_ok!(Lst::fully_unbond(RuntimeOrigin::signed(902), 200, 1));

			assert_eq!(
				pool_events_since_last_call(),
				vec![Event::Unbonded {
					member: 200,
					pool_id: 1,
					points: 200,
					balance: 200,
					era: 3
				}]
			);

			assert_eq!(
				BondedPool::<Runtime>::get(1).unwrap(),
				BondedPool {
					id: 1,
					inner: BondedPoolInner {
						commission: Commission::default(),
						roles: DEFAULT_ROLES,
						state: PoolState::Blocked,
					}
				}
			);
			assert_eq!(StakingMock::active_stake(&default_bonded_account()).unwrap(), 10);
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap(),
				SubPools {
					no_era: Default::default(),
					with_era: unbonding_pools_with_era! {
						3 => UnbondPool { points: 100 + 200, balance: 100 + 200 }
					},
				}
			);
			assert_eq!(
				*UnbondingBalanceMap::get().get(&default_bonded_account()).unwrap(),
				vec![(3, 100), (3, 200)],
			);
		});
}

#[test]
fn unbond_permissionless_works() {
	// Scenarios where non-admin accounts can unbond others
	ExtBuilder::default().add_members(vec![(100, 100)]).build_and_execute(|| {
		// Given the pool is blocked
		unsafe_set_state(1, PoolState::Blocked);

		// A permissionless unbond attempt errors
		assert_noop!(
			Lst::fully_unbond(RuntimeOrigin::signed(420), 100, 1),
			Error::<Runtime>::NotKickerOrDestroying
		);

		// permissionless unbond must be full
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(420), 100, 1, 80),
			Error::<Runtime>::PartialUnbondNotAllowedPermissionlessly,
		);

		// Given the pool is destroying
		unsafe_set_state(1, PoolState::Destroying);

		// The depositor cannot be fully unbonded until they are the last member
		assert_noop!(
			Lst::fully_unbond(RuntimeOrigin::signed(10), 10, 1),
			Error::<Runtime>::MinimumBondNotMet,
		);

		// Any account can unbond a member that is not the depositor
		assert_ok!(Lst::fully_unbond(RuntimeOrigin::signed(420), 100, 1));

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
				Event::Bonded { member: 100, pool_id: 1, bonded: 100, joined: true },
				Event::Unbonded { member: 100, pool_id: 1, points: 100, balance: 100, era: 3 }
			]
		);

		// still permissionless unbond must be full
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(420), 100, 1, 80),
			Error::<Runtime>::PartialUnbondNotAllowedPermissionlessly,
		);

		// Given the pool is blocked
		unsafe_set_state(1, PoolState::Blocked);

		// The depositor cannot be unbonded
		assert_noop!(
			Lst::fully_unbond(RuntimeOrigin::signed(420), 10, 1),
			Error::<Runtime>::DoesNotHavePermission
		);

		// Given the pools is destroying
		unsafe_set_state(1, PoolState::Destroying);

		// The depositor cannot be unbonded yet.
		assert_noop!(
			Lst::fully_unbond(RuntimeOrigin::signed(420), 10, 1),
			Error::<Runtime>::DoesNotHavePermission,
		);

		// but when everyone is unbonded it can..
		CurrentEra::set(3);
		assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(10), 100, 1, 0));

		// still permissionless unbond must be full.
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(420), 10, 1, 5),
			Error::<Runtime>::PartialUnbondNotAllowedPermissionlessly,
		);

		// depositor can never be unbonded permissionlessly .
		assert_noop!(
			Lst::fully_unbond(RuntimeOrigin::signed(420), 10, 1),
			Error::<T>::DoesNotHavePermission
		);
		// but depositor itself can do it.
		assert_ok!(Lst::fully_unbond(RuntimeOrigin::signed(10), 10, 1));

		assert_eq!(
			SubPoolsStorage::<Runtime>::get(1).unwrap(),
			SubPools {
				no_era: Default::default(),
				with_era: unbonding_pools_with_era! {
					3 + 3 => UnbondPool { points: 10, balance: 10 }
				}
			}
		);
		assert_eq!(StakingMock::active_stake(&default_bonded_account()).unwrap(), 0);
		assert_eq!(
			*UnbondingBalanceMap::get().get(&default_bonded_account()).unwrap(),
			vec![(6, 10)]
		);
	});
}

#[test]
fn partial_unbond_era_tracking() {
	ExtBuilder::default().ed(1).build_and_execute(|| {
		// to make the depositor capable of withdrawing.
		StakingMinBond::set(1);
		MinCreateBond::<T>::set(1);
		MinJoinBond::<T>::set(1);
		assert_eq!(Lst::depositor_min_bond(), 1);

		// given
		assert!(SubPoolsStorage::<Runtime>::get(1).is_none());
		assert_eq!(CurrentEra::get(), 0);
		assert_eq!(BondingDuration::get(), 3);

		// so the depositor can leave, just keeps the test simpler.
		unsafe_set_state(1, PoolState::Destroying);

		// when: casual unbond
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 1));

		// then
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(1).unwrap(),
			SubPools {
				no_era: Default::default(),
				with_era: unbonding_pools_with_era! {
					3 => UnbondPool { points: 1, balance: 1 }
				}
			}
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
				Event::Unbonded { member: 10, pool_id: 1, points: 1, balance: 1, era: 3 }
			]
		);

		// when: casual further unbond, same era.
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 5));

		// then
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(1).unwrap(),
			SubPools {
				no_era: Default::default(),
				with_era: unbonding_pools_with_era! {
					3 => UnbondPool { points: 6, balance: 6 }
				}
			}
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Unbonded { member: 10, pool_id: 1, points: 5, balance: 5, era: 3 }]
		);

		// when: casual further unbond, next era.
		CurrentEra::set(1);
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 5));

		// then
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(1).unwrap(),
			SubPools {
				no_era: Default::default(),
				with_era: unbonding_pools_with_era! {
					3 => UnbondPool { points: 6, balance: 6 },
					4 => UnbondPool { points: 1, balance: 1 }
				}
			}
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Unbonded { member: 10, pool_id: 1, points: 1, balance: 1, era: 4 }]
		);

		// when: unbonding more than our active: error
		assert_noop!(
			frame_support::storage::with_storage_layer(|| Lst::unbond(
				RuntimeOrigin::signed(10),
				10,
				1,
				5
			)),
			Error::<Runtime>::MinimumBondNotMet
		);
		// instead:
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 3));

		// then
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(1).unwrap(),
			SubPools {
				no_era: Default::default(),
				with_era: unbonding_pools_with_era! {
					3 => UnbondPool { points: 6, balance: 6 },
					4 => UnbondPool { points: 4, balance: 4 }
				}
			}
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Unbonded { member: 10, pool_id: 1, points: 3, balance: 3, era: 4 }]
		);
	});
}

#[test]
fn partial_unbond_max_chunks() {
	ExtBuilder::default().add_members(vec![(20, 20)]).ed(1).build_and_execute(|| {
		MaxUnbonding::set(2);

		// given
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(20), 20, 1, 2));
		CurrentEra::set(1);
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(20), 20, 1, 3));

		// when
		CurrentEra::set(2);
		assert_noop!(
			frame_support::storage::with_storage_layer(|| Lst::unbond(
				RuntimeOrigin::signed(20),
				20,
				1,
				4
			)),
			Error::<Runtime>::MaxUnbondingLimit
		);

		// when
		MaxUnbonding::set(3);
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(20), 20, 1, 3));

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
				Event::Bonded { member: 20, pool_id: 1, bonded: 20, joined: true },
				Event::Unbonded { member: 20, pool_id: 1, points: 2, balance: 2, era: 3 },
				Event::Unbonded { member: 20, pool_id: 1, points: 3, balance: 3, era: 4 },
				Event::Unbonded { member: 20, pool_id: 1, points: 1, balance: 1, era: 5 }
			]
		);
	})
}

// depositor can unbond only up to `MinCreateBond`.
#[test]
fn depositor_permissioned_partial_unbond() {
	ExtBuilder::default().ed(1).build_and_execute(|| {
		// given
		StakingMinBond::set(5);
		assert_eq!(Lst::depositor_min_bond(), 5);

		// can unbond a bit..
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 3));

		// but not less than 2
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 6),
			Error::<Runtime>::MinimumBondNotMet
		);

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
				Event::Unbonded { member: 10, pool_id: 1, points: 3, balance: 3, era: 3 }
			]
		);
	});
}

#[test]
fn depositor_permissioned_partial_unbond_slashed() {
	ExtBuilder::default().ed(1).build_and_execute(|| {
		// given
		assert_eq!(MinCreateBond::<Runtime>::get(), 2);

		// slash the default pool
		StakingMock::slash_by(1, 5);

		// cannot unbond even 7, because the value of shares is now less.
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(10), 10, 1, 7),
			Error::<Runtime>::MinimumBondNotMet
		);
	});
}
