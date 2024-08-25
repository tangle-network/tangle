use super::*;

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
			let pool_id = NextPoolId::<Runtime>::get() - 1;

			// can unbond to above limit
			assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 5));
			assert_eq!(Pools::member_points(pool_id, 20), 15);
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 20).unwrap().unbonding_points(),
				5
			);

			// cannot go to below 10:
			assert_noop!(
				Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 10),
				Error::<T>::MinimumBondNotMet
			);

			// but can go to 0
			assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 15));
			assert_eq!(Pools::member_points(pool_id, 20), 0);
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 20).unwrap().unbonding_points(),
				20
			);

			// token account should be destroyed when the user has unbonded fully aka no more lst
			// tokens
			assert!(Fungibles::token_account_of((
				staked_collection_id(),
				pool_id as AssetId,
				20
			))
			.is_none());
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
			let pool_id = NextPoolId::<Runtime>::get() - 1;

			let kicker = DEFAULT_MANAGER;

			// cannot be kicked to above the limit.
			assert_noop!(
				Pools::unbond(RuntimeOrigin::signed(kicker), pool_id, 20, 5),
				Error::<T>::PartialUnbondNotAllowedPermissionlessly
			);

			// cannot go to below 10:
			assert_noop!(
				Pools::unbond(RuntimeOrigin::signed(kicker), pool_id, 20, 15),
				Error::<T>::PartialUnbondNotAllowedPermissionlessly
			);

			// but they themselves can do an unbond
			assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 2));
			assert_eq!(Pools::member_points(pool_id, 20), 18);
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 20).unwrap().unbonding_points(),
				2
			);

			// can be kicked to 0.

			assert_ok!(Pools::unbond(RuntimeOrigin::signed(kicker), pool_id, 20, 18));
			assert_eq!(Pools::member_points(pool_id, 20), 0);
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 20).unwrap().unbonding_points(),
				20
			);
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
			let pool_id = NextPoolId::<Runtime>::get() - 1;
			unsafe_set_state(pool_id, PoolState::Destroying);
			let random = 123;

			// cannot be kicked to above the limit.
			assert_noop!(
				Pools::unbond(RuntimeOrigin::signed(random), pool_id, 20, 5),
				Error::<T>::PartialUnbondNotAllowedPermissionlessly
			);

			// cannot go to below 10:
			assert_noop!(
				Pools::unbond(RuntimeOrigin::signed(random), pool_id, 20, 15),
				Error::<T>::PartialUnbondNotAllowedPermissionlessly
			);

			// but they themselves can do an unbond
			assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 2));
			assert_eq!(Pools::member_points(pool_id, 20), 18);
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 20).unwrap().unbonding_points(),
				2
			);

			// but can go to 0
			assert_ok!(Pools::unbond(RuntimeOrigin::signed(random), pool_id, 20, 18));
			assert_eq!(Pools::member_points(pool_id, 20), 0);
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 20).unwrap().unbonding_points(),
				20
			);
		})
}

#[test]
fn test_depositor_unbond_destroying_permissionless() {
	// deposit can be unbonded permissionlessly
	ExtBuilder::default().min_join_bond(10).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// give the depositor some extra funds.
		bond_extra(10, pool_id, 20);

		let balance = Balances::free_balance(10);

		// first 10 is minted to `T::LstCollectionOwner::get()`
		assert_eq!(Pools::member_points(pool_id, 10), 20);

		// set the stage
		unsafe_set_state(pool_id, PoolState::Destroying);

		// first unbond simple points
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 20));
		assert_eq!(UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_points(), 20);

		// withdraw it
		CurrentEra::set(3);
		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(10), pool_id, 10, 0));

		// check
		assert_eq!(Pools::member_points(pool_id, 10), 0);
		assert_eq!(Balances::free_balance(10), balance + 20);

		// now unbond deposit
		// note that the deposit can be unbonded by anyone
		assert_ok!(Pools::unbond_deposit(RuntimeOrigin::signed(123), pool_id));
		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, Pools::deposit_account_id(pool_id))
				.unwrap()
				.unbonding_points(),
			10
		);

		// withdraw it
		CurrentEra::set(6);
		assert_ok!(Pools::withdraw_deposit(RuntimeOrigin::signed(123), pool_id));

		assert_eq!(Pools::member_points(pool_id, 10), 0);

		// since pool is dissolved now, remaining rewards are also transferred to current token
		// holders
		assert_eq!(Balances::free_balance(10), balance + 30 + 5);
	})
}

#[test]
fn depositor_unbond_destroying_last_member() {
	// deposit in pool, pool state destroying
	//   - depositor can unbond to above limit always.
	//   - depositor cannot unbond to below limit if last.
	//   - depositor can unbond to 0 if last and destroying.
	ExtBuilder::default().min_join_bond(10).ed(1).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// give the depositor some extra funds.
		bond_extra(10, pool_id, 20);

		assert_eq!(Pools::member_points(pool_id, 10), 20);

		// set the stage
		unsafe_set_state(pool_id, PoolState::Destroying);

		// can unbond to above the limit.
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 5));
		assert_eq!(Pools::member_points(pool_id, 10), 15);
		assert_eq!(UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_points(), 5);

		// still cannot go to below limit
		assert_noop!(
			Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 10),
			Error::<T>::MinimumBondNotMet
		);

		// can go to 0 too.
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 15));
		assert_eq!(Pools::member_points(pool_id, 10), 0);
		assert_eq!(UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_points(), 20);
	})
}

#[test]
fn unbond_of_1_works() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		bond_extra(10, pool_id, 10);

		unsafe_set_state(pool_id, PoolState::Destroying);

		assert_ok!(fully_unbond_permissioned(pool_id, 10));

		assert_eq!(
			SubPoolsStorage::<Runtime>::get(pool_id).unwrap().with_era,
			unbonding_pools_with_era! { 3 => UnbondPool::<Runtime> { points: 10, balance: 10 }}
		);

		let pool = BondedPool::<Runtime>::get(pool_id).unwrap();
		assert_eq!(
			pool,
			BondedPool {
				id: pool_id,
				inner: BondedPoolInner {
					token_id: DEFAULT_TOKEN_ID,
					state: PoolState::Destroying,
					capacity: 1_000,
					commission: Commission::default(),
				}
			}
		);

		// should contain deposit points
		assert_eq!(pool.points(), 10);
		assert_eq!(StakingMock::active_stake(&default_bonded_account()).unwrap(), 10);
	});
}

#[test]
fn unbond_of_3_works() {
	ExtBuilder::default()
		.add_members(vec![(40, 40), (550, 550)])
		.build_and_execute(|| {
			let ed = Balances::minimum_balance();
			let pool_id = NextPoolId::<Runtime>::get() - 1;

			// Given a slash from 600 -> 100
			StakingMock::set_bonded_balance(default_bonded_account(), 100);
			// and unclaimed rewards of 600.
			Balances::make_free_balance_be(&default_reward_account(), ed + 600);

			// When
			// bond_extra(10, pool_id, 10);
			assert_ok!(fully_unbond_permissioned(pool_id, 40));

			// Then
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap().with_era,
				unbonding_pools_with_era! { 3 => UnbondPool { points: 6, balance: 6 }}
			);
			let pool = BondedPool::<Runtime>::get(pool_id).unwrap();
			assert_eq!(
				pool,
				BondedPool {
					id: pool_id,
					inner: BondedPoolInner {
						token_id: DEFAULT_TOKEN_ID,
						state: PoolState::Open,
						capacity: 1_000,
						commission: Commission::default(),
					}
				}
			);
			assert_eq!(pool.points(), 560);
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { creator: 10, pool_id, capacity: 1_000 },
					Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
					Event::Bonded { member: 40, pool_id, bonded: 40 },
					Event::Bonded { member: 550, pool_id, bonded: 550 },
					Event::Unbonded { member: 40, pool_id, points: 6, balance: 6, era: 3 }
				]
			);

			assert_eq!(StakingMock::active_stake(&default_bonded_account()).unwrap(), 94);
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 40).unwrap().unbonding_eras,
				member_unbonding_eras!(3 => 6)
			);
			assert_eq!(Balances::free_balance(40), 40);

			// When
			unsafe_set_state(pool_id, PoolState::Destroying);
			assert_ok!(fully_unbond_permissioned(pool_id, 550));

			// Then
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap().with_era,
				unbonding_pools_with_era! { 3 => UnbondPool { points: 98, balance: 98 }}
			);
			assert_eq!(
				BondedPool::<Runtime>::get(pool_id).unwrap(),
				BondedPool {
					id: pool_id,
					inner: BondedPoolInner {
						token_id: DEFAULT_TOKEN_ID,
						state: PoolState::Destroying,
						capacity: 1_000,
						commission: Commission::default(),
					}
				}
			);
			assert_eq!(StakingMock::active_stake(&default_bonded_account()).unwrap(), 2);
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 550).unwrap().unbonding_eras,
				member_unbonding_eras!(3 => 92)
			);
			assert_eq!(Balances::free_balance(550), 550);
			assert_eq!(
				pool_events_since_last_call(),
				vec![Event::Unbonded { member: 550, pool_id, points: 92, balance: 92, era: 3 }]
			);

			// When
			CurrentEra::set(3);
			assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(10), pool_id, 40, 0));
			assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(10), pool_id, 550, 0));

			// Then
			let pool = BondedPool::<Runtime>::get(pool_id).unwrap();
			assert_eq!(
				pool,
				BondedPool {
					id: pool_id,
					inner: BondedPoolInner {
						token_id: DEFAULT_TOKEN_ID,
						state: PoolState::Destroying,
						capacity: 1_000,
					}
				}
			);
			assert_eq!(pool.points(), 10); // depositor is still in the pool
			assert_eq!(StakingMock::active_stake(&default_bonded_account()).unwrap(), 2);

			assert_eq!(Balances::free_balance(550), 550 + 92);
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Withdrawn { member: 40, pool_id, points: 6, balance: 6 },
					Event::Withdrawn { member: 550, pool_id, points: 92, balance: 92 },
				]
			);
		});
}

#[test]
fn unbond_merges_older_pools() {
	ExtBuilder::default().with_check(1).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// Given
		assert_eq!(StakingMock::bonding_duration(), 3);
		SubPoolsStorage::<Runtime>::insert(
			pool_id,
			SubPools {
				no_era: Default::default(),
				with_era: unbonding_pools_with_era! {
					3 => UnbondPool { balance: 10, points: 100 },
					1 + 3 => UnbondPool { balance: 20, points: 20 },
					2 + 3 => UnbondPool { balance: 101, points: 101}
				},
			},
		);

		// bond some extra funds for the depositor, since deposit points are stored in the
		// pallet's account
		bond_extra(10, pool_id, 10);

		unsafe_set_state(pool_id, PoolState::Destroying);

		// When
		let current_era = 1 + TotalUnbondingPools::<Runtime>::get();
		CurrentEra::set(current_era);

		assert_ok!(fully_unbond_permissioned(pool_id, 10));

		// Then
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
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
				Event::Created { creator: 10, pool_id, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
				Event::Bonded { member: 10, pool_id, bonded: 10 },
				Event::Unbonded { member: 10, pool_id, points: 10, balance: 10, era: 9 }
			]
		);
	});
}

#[test]
fn unbond_kick_works() {
	// Kick: the caller is the admin
	ExtBuilder::default()
		.add_members(vec![(100, 100), (200, 200)])
		.build_and_execute(|| {
			let pool_id = NextPoolId::<Runtime>::get() - 1;

			// When the nominator tries to kick, then its a noop
			assert_noop!(
				Pools::fully_unbond(RuntimeOrigin::signed(901), pool_id, 100),
				Error::<Runtime>::NotKickerOrDestroying
			);

			// When the root kicks then its ok
			assert_ok!(Pools::fully_unbond(RuntimeOrigin::signed(DEFAULT_MANAGER), pool_id, 100));

			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { creator: 10, pool_id, capacity: 1_000 },
					Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
					Event::Bonded { member: 100, pool_id, bonded: 100 },
					Event::Bonded { member: 200, pool_id, bonded: 200 },
					Event::Unbonded { member: 100, pool_id, points: 100, balance: 100, era: 3 },
				]
			);

			// When the admin kicks then its ok
			assert_ok!(Pools::fully_unbond(RuntimeOrigin::signed(DEFAULT_MANAGER), pool_id, 200));

			assert_eq!(
				pool_events_since_last_call(),
				vec![Event::Unbonded { member: 200, pool_id, points: 200, balance: 200, era: 3 }]
			);

			let pool = BondedPool::<Runtime>::get(pool_id).unwrap();
			assert_eq!(
				pool,
				BondedPool {
					id: pool_id,
					inner: BondedPoolInner {
						token_id: DEFAULT_TOKEN_ID,
						state: PoolState::Open,
						capacity: 1_000,
						commission: Commission::default(),
					}
				}
			);
			assert_eq!(pool.points(), 10); // Only 10 points because 200 + 100 was unbonded
			assert_eq!(StakingMock::active_stake(&default_bonded_account()).unwrap(), 10);
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
				SubPools {
					no_era: Default::default(),
					with_era: unbonding_pools_with_era! {
						3 => UnbondPool { points: 100 + 200, balance: 100 + 200 }
					},
				}
			);
			assert_eq!(
				*UnbondingBalanceMap::get().get(&default_bonded_account()).unwrap(),
				100 + 200
			);
		});
}

#[test]
fn unbond_permissionless_works() {
	// Scenarios where non-admin accounts can unbond others
	ExtBuilder::default().add_members(vec![(100, 100)]).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// A permissionless unbond attempt errors
		assert_noop!(
			Pools::fully_unbond(RuntimeOrigin::signed(420), pool_id, 100),
			Error::<Runtime>::NotKickerOrDestroying
		);

		// permissionless unbond must be full
		assert_noop!(
			Pools::unbond(RuntimeOrigin::signed(420), pool_id, 100, 80),
			Error::<Runtime>::PartialUnbondNotAllowedPermissionlessly,
		);

		// Given the pool is destroying
		unsafe_set_state(pool_id, PoolState::Destroying);

		// Any account can fully unbond a member
		assert_ok!(Pools::fully_unbond(RuntimeOrigin::signed(420), pool_id, 100));

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { creator: 10, pool_id, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
				Event::Bonded { member: 100, pool_id, bonded: 100 },
				Event::Unbonded { member: 100, pool_id, points: 100, balance: 100, era: 3 }
			]
		);

		// still permissionless unbond must be full
		assert_noop!(
			Pools::unbond(RuntimeOrigin::signed(420), pool_id, 100, 80),
			Error::<Runtime>::PartialUnbondNotAllowedPermissionlessly,
		);

		// Given the pools is destroying
		unsafe_set_state(pool_id, PoolState::Destroying);

		// but when everyone is unbonded it can..
		CurrentEra::set(3);
		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(10), pool_id, 100, 0));

		// still permissionless unbond must be full.
		assert_noop!(
			Pools::unbond(RuntimeOrigin::signed(420), pool_id, 10, 5),
			Error::<Runtime>::PartialUnbondNotAllowedPermissionlessly,
		);

		assert_eq!(get_pool(pool_id).unwrap().points(), 10);
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
			SubPools { no_era: Default::default(), with_era: unbonding_pools_with_era! {} }
		);
		assert_eq!(StakingMock::active_stake(&default_bonded_account()).unwrap(), 10);
	});
}

#[test]
fn partial_unbond_era_tracking() {
	ExtBuilder::default().ed(1).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// to make the depositor capable of withdrawing.
		StakingMinBond::set(1);
		MinCreateBond::<T>::set(1);
		MinJoinBond::<T>::set(1);
		assert_eq!(Pools::depositor_min_bond(), 1);

		// bond some funds for depositor
		bond_extra(10, pool_id, 10);

		// given
		assert_eq!(Pools::member_points(pool_id, 10), 10);
		assert!(!UnbondingMembers::<Runtime>::contains_key(pool_id, 10));
		assert_eq!(pool_id, 0);

		assert_eq!(get_pool(pool_id).unwrap().points(), 20);
		assert!(SubPoolsStorage::<Runtime>::get(pool_id).is_none());
		assert_eq!(CurrentEra::get(), 0);
		assert_eq!(BondingDuration::get(), 3);

		// so the depositor can leave, just keeps the test simpler.
		unsafe_set_state(pool_id, PoolState::Destroying);

		// when: casual unbond
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 1));

		// then
		assert_eq!(Pools::member_points(pool_id, 10), 9);
		assert_eq!(UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_points(), 1);
		assert_eq!(get_pool(pool_id).unwrap().points(), 19);
		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_eras,
			member_unbonding_eras!(3 => 1)
		);
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
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
				Event::Created { creator: 10, pool_id, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
				Event::Bonded { member: 10, pool_id, bonded: 10 },
				Event::Unbonded { member: 10, pool_id, points: 1, balance: 1, era: 3 }
			]
		);

		// when: casual further unbond, same era.
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 5));

		// then
		assert_eq!(Pools::member_points(pool_id, 10), 4);
		assert_eq!(UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_points(), 6);
		assert_eq!(get_pool(pool_id).unwrap().points(), 14);
		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_eras,
			member_unbonding_eras!(3 => 6)
		);
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
			SubPools {
				no_era: Default::default(),
				with_era: unbonding_pools_with_era! {
					3 => UnbondPool { points: 6, balance: 6 }
				}
			}
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Unbonded { member: 10, pool_id, points: 5, balance: 5, era: 3 }]
		);

		// when: casual further unbond, next era.
		CurrentEra::set(1);
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 1));

		// then
		assert_eq!(Pools::member_points(pool_id, 10), 3);
		assert_eq!(UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_points(), 7);
		assert_eq!(get_pool(pool_id).unwrap().points(), 13);
		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_eras,
			member_unbonding_eras!(3 => 6, 4 => 1)
		);
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
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
			vec![Event::Unbonded { member: 10, pool_id, points: 1, balance: 1, era: 4 }]
		);

		// when: unbonding more than our active: error
		assert_noop!(
			frame_support::storage::with_storage_layer(|| Pools::unbond(
				RuntimeOrigin::signed(10),
				pool_id,
				10,
				5
			)),
			Error::<Runtime>::MinimumBondNotMet
		);
		// instead:
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 3));

		// then
		assert_eq!(Pools::member_points(pool_id, 10), 0);
		assert_eq!(UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_points(), 10);
		assert_eq!(get_pool(pool_id).unwrap().points(), 10);
		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_eras,
			member_unbonding_eras!(3 => 6, 4 => 4)
		);
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
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
			vec![Event::Unbonded { member: 10, pool_id, points: 3, balance: 3, era: 4 }]
		);
	});
}

#[test]
fn partial_unbond_max_chunks() {
	ExtBuilder::default().add_members(vec![(20, 20)]).ed(1).build_and_execute(|| {
		MaxUnbonding::set(2);
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// given
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 2));
		CurrentEra::set(1);
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 3));
		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 20).unwrap().unbonding_eras,
			member_unbonding_eras!(3 => 2, 4 => 3)
		);

		// when
		CurrentEra::set(2);
		assert_noop!(
			frame_support::storage::with_storage_layer(|| Pools::unbond(
				RuntimeOrigin::signed(20),
				pool_id,
				20,
				4
			)),
			Error::<Runtime>::MaxUnbondingLimit
		);

		// when
		MaxUnbonding::set(3);
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 1));

		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 20).unwrap().unbonding_eras,
			member_unbonding_eras!(3 => 2, 4 => 3, 5 => 1)
		);

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { creator: 10, pool_id, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
				Event::Bonded { member: 20, pool_id, bonded: 20 },
				Event::Unbonded { member: 20, pool_id, points: 2, balance: 2, era: 3 },
				Event::Unbonded { member: 20, pool_id, points: 3, balance: 3, era: 4 },
				Event::Unbonded { member: 20, pool_id, points: 1, balance: 1, era: 5 }
			]
		);
	})
}

#[test]
fn depositor_permissioned_partial_unbond_slashed() {
	ExtBuilder::default().ed(1).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// given
		assert_eq!(MinCreateBond::<Runtime>::get(), 2);
		assert_eq!(Pools::member_points(pool_id, 10), 0);
		assert!(!UnbondingMembers::<Runtime>::contains_key(pool_id, 10));

		// slash the default pool
		StakingMock::set_bonded_balance(Pools::compute_pool_account_id(1, AccountType::Bonded), 5);

		// cannot unbond even 7, because the value of shares is now less.
		assert_noop!(
			Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 7),
			Error::<Runtime>::MinimumBondNotMet
		);
	});
}

#[test]
fn unbonding_burns_lst() {
	let members = vec![(100, 100), (200, 200), (300, 300)];
	ExtBuilder::default().add_members(members.clone()).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		for (member, balance) in members.iter() {
			assert_ok!(Pools::unbond(
				RuntimeOrigin::signed(*member),
				pool_id,
				*member,
				balance / 2
			));

			assert_eq!(Pools::member_points(pool_id, *member), balance / 2);

			// check events
			assert_event_deposited!(FungiblesEvent::Burned {
				collection_id: LST_COLLECTION_ID,
				token_id: pool_id as u128,
				account_id: *member,
				amount: balance / 2
			});

			// do full unbond
			assert_ok!(Pools::fully_unbond(RuntimeOrigin::signed(*member), pool_id, *member));
			assert_eq!(Pools::member_points(pool_id, *member), 0);

			// check events
			assert_event_deposited!(FungiblesEvent::Burned {
				collection_id: LST_COLLECTION_ID,
				token_id: pool_id as u128,
				account_id: *member,
				amount: balance / 2
			});
		}
	});
}
