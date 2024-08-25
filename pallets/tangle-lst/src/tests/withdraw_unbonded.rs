use super::*;

#[test]
fn withdraw_unbonded_works_against_slashed_no_era_sub_pool() {
	ExtBuilder::default()
		.add_members(vec![(40, 40), (550, 550)])
		.build_and_execute(|| {
			let pool_id = NextPoolId::<Runtime>::get() - 1;

			// reduce the noise a bit.
			let _ = balances_events_since_last_call();

			// Given
			assert_eq!(StakingMock::bonding_duration(), 3);
			assert_ok!(Pools::fully_unbond(RuntimeOrigin::signed(550), pool_id, 550));
			assert_ok!(Pools::fully_unbond(RuntimeOrigin::signed(40), pool_id, 40));
			assert_eq!(Balances::free_balance(default_bonded_account()), 600);

			let mut current_era = 1;
			CurrentEra::set(current_era);

			let mut sub_pools = SubPoolsStorage::<Runtime>::get(pool_id).unwrap();
			let unbond_pool = sub_pools.with_era.get_mut(&3).unwrap();
			// Sanity check
			assert_eq!(*unbond_pool, UnbondPool { points: 550 + 40, balance: 550 + 40 });

			// Simulate a slash to the pool with_era(current_era), decreasing the balance by
			// half
			{
				unbond_pool.balance /= 2; // 295
				SubPoolsStorage::<Runtime>::insert(pool_id, sub_pools);
				// Update the equivalent of the unbonding chunks for the `StakingMock`
				let mut x = UnbondingBalanceMap::get();
				*x.get_mut(&default_bonded_account()).unwrap() /= 5;
				UnbondingBalanceMap::set(&x);
				Balances::make_free_balance_be(
					&default_bonded_account(),
					Balances::free_balance(default_bonded_account()) / 2, // 300
				);
				StakingMock::set_bonded_balance(
					default_bonded_account(),
					StakingMock::active_stake(&default_bonded_account()).unwrap() / 2,
				);
			};

			// Advance the current_era to ensure all `with_era` pools will be merged into
			// `no_era` pool
			current_era += TotalUnbondingPools::<Runtime>::get();
			CurrentEra::set(current_era);

			// Simulate some other call to unbond that would merge `with_era` pools into
			// `no_era`
			let sub_pools =
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap().maybe_merge_pools(current_era);
			SubPoolsStorage::<Runtime>::insert(pool_id, sub_pools);

			assert_eq!(
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
				SubPools {
					no_era: UnbondPool { points: 550 + 40, balance: 275 + 20 },
					with_era: Default::default()
				}
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { creator: 10, pool_id, capacity: 1_000 },
					Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
					Event::Bonded { member: 40, pool_id, bonded: 40 },
					Event::Bonded { member: 550, pool_id, bonded: 550 },
					Event::Unbonded { member: 550, pool_id, points: 550, balance: 550, era: 3 },
					Event::Unbonded { member: 40, pool_id: 0, points: 40, balance: 40, era: 3 },
				]
			);
			assert_eq!(
				balances_events_since_last_call(),
				vec![BEvent::BalanceSet { who: default_bonded_account(), free: 300 }]
			);

			// When
			assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(550), pool_id, 550, 0));

			// Then
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap().no_era,
				UnbondPool { points: 40, balance: 20 }
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![Event::Withdrawn { member: 550, pool_id, balance: 275, points: 550 },]
			);
			assert_eq!(
				balances_events_since_last_call(),
				vec![BEvent::Transfer { from: default_bonded_account(), to: 550, amount: 275 }]
			);

			// When
			assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(40), pool_id, 40, 0));

			// Then
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap().no_era,
				UnbondPool { points: 0, balance: 0 }
			);
			assert!(!UnbondingMembers::<Runtime>::contains_key(pool_id, 40));
			assert_eq!(
				pool_events_since_last_call(),
				vec![Event::Withdrawn { member: 40, pool_id, balance: 20, points: 40 },]
			);
			assert_eq!(
				balances_events_since_last_call(),
				vec![BEvent::Transfer { from: default_bonded_account(), to: 40, amount: 20 }]
			);

			// now, finally, the depositor can take out its share.
			unsafe_set_state(pool_id, PoolState::Destroying);
			assert_ok!(Pools::unbond_deposit(RuntimeOrigin::signed(10), pool_id));

			current_era += 3;
			CurrentEra::set(current_era);

			assert_eq!(UsedPoolAssetIds::<Runtime>::get(DEFAULT_TOKEN_ID), Some(pool_id));

			// token should be destroyed as everyone has withdrawn
			assert!(Fungibles::token_of(staked_collection_id(), pool_id as AssetId).is_none());
			assert_ok!(Pools::withdraw_deposit(RuntimeOrigin::signed(10), pool_id));
			assert!(Fungibles::token_of(staked_collection_id(), pool_id as AssetId).is_none());
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Unbonded {
						member: Pools::deposit_account_id(pool_id),
						pool_id,
						balance: 5,
						points: 5,
						era: 9
					},
					Event::Withdrawn { member: 10, pool_id, balance: 5, points: 5 },
					Event::Destroyed { pool_id }
				]
			);
			assert_eq!(
				balances_events_since_last_call(),
				vec![
					BEvent::Transfer { from: default_bonded_account(), to: 10, amount: 5 },
					BEvent::Transfer { from: default_reward_account(), to: 10, amount: 5 },
					BEvent::Endowed { account: UnclaimedBalanceReceiver::get(), free_balance: 5 },
					BEvent::Transfer {
						from: pool_bonus_account(pool_id),
						to: UnclaimedBalanceReceiver::get(),
						amount: 5
					}
				]
			);

			// pool was dissolved
			assert!(BondedPool::<Runtime>::get(pool_id).is_none());
			assert!(UsedPoolAssetIds::<Runtime>::get(DEFAULT_TOKEN_ID).is_none())
		});
}

#[test]
fn withdraw_unbonded_works_against_slashed_with_era_sub_pools() {
	ExtBuilder::default()
		.add_members(vec![(40, 40), (550, 550)])
		.build_and_execute(|| {
			let _ = balances_events_since_last_call();
			let balance = Balances::free_balance(10);
			let pool_id = NextPoolId::<Runtime>::get() - 1;
			let depositor = 10;

			// Given
			// current bond is 600, we slash it all to 300.
			StakingMock::set_bonded_balance(default_bonded_account(), 300);
			Balances::make_free_balance_be(&default_bonded_account(), 300);
			assert_eq!(StakingMock::total_stake(&default_bonded_account()), Ok(300));

			// set bonus account to 1_000
			Balances::make_free_balance_be(&pool_bonus_account(pool_id), 1_000);

			assert_ok!(fully_unbond_permissioned(pool_id, 40));
			assert_ok!(fully_unbond_permissioned(pool_id, 550));

			assert_eq!(
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap().with_era,
				unbonding_pools_with_era! { 3 => UnbondPool { points: 550 / 2 + 40 / 2, balance: 550 / 2 + 40 / 2
				}}
			);

			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { creator: depositor, pool_id, capacity: 1_000 },
					Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
					Event::Bonded { member: 40, pool_id, bonded: 40 },
					Event::Bonded { member: 550, pool_id, bonded: 550 },
					Event::Unbonded { member: 40, pool_id, balance: 20, points: 20, era: 3 },
					Event::Unbonded { member: 550, pool_id, balance: 275, points: 275, era: 3 }
				]
			);
			assert_eq!(
				balances_events_since_last_call(),
				vec![
					BEvent::BalanceSet { who: default_bonded_account(), free: 300 },
					BEvent::BalanceSet { who: pool_bonus_account(pool_id), free: 1_000 }
				]
			);

			CurrentEra::set(StakingMock::bonding_duration());

			// When
			assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(40), pool_id, 40, 0));

			// Then
			assert_eq!(
				balances_events_since_last_call(),
				vec![BEvent::Transfer { from: default_bonded_account(), to: 40, amount: 20 },]
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![Event::Withdrawn { member: 40, pool_id, balance: 20, points: 20 },]
			);

			assert_eq!(
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap().with_era,
				unbonding_pools_with_era! { 3 => UnbondPool { points: 550 / 2, balance: 550 / 2 }}
			);

			// When
			assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(550), pool_id, 550, 0));

			// Then
			assert_eq!(
				balances_events_since_last_call(),
				vec![BEvent::Transfer { from: default_bonded_account(), to: 550, amount: 275 },]
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![Event::Withdrawn { member: 550, pool_id, balance: 275, points: 275 },]
			);
			assert!(SubPoolsStorage::<Runtime>::get(pool_id).unwrap().with_era.is_empty());

			// now, finally, the depositor can take out its share.
			unsafe_set_state(pool_id, PoolState::Destroying);
			assert_ok!(Pools::unbond_deposit(RuntimeOrigin::signed(10), pool_id));

			// because everyone else has left, the points
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap().with_era,
				unbonding_pools_with_era! { 6 => UnbondPool { points: 5, balance: 5 }}
			);

			CurrentEra::set(CurrentEra::get() + 3);

			// when
			assert_eq!(Balances::free_balance(depositor), balance);
			assert_ok!(Pools::withdraw_deposit(RuntimeOrigin::signed(depositor), pool_id,));

			// then
			assert_eq!(Balances::free_balance(depositor), 10 + balance);
			assert_eq!(Balances::free_balance(default_bonded_account()), 0);

			// in this test depositor also gets a fair share of the slash, because the slash was
			// applied to the bonded account.
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Unbonded {
						member: Pools::deposit_account_id(pool_id),
						pool_id,
						points: 5,
						balance: 5,
						era: 6
					},
					Event::Withdrawn { member: depositor, pool_id, points: 5, balance: 5 },
					Event::Destroyed { pool_id }
				]
			);
			assert_eq!(
				balances_events_since_last_call(),
				vec![
					BEvent::Transfer { from: default_bonded_account(), to: depositor, amount: 5 },
					BEvent::Transfer { from: default_reward_account(), to: depositor, amount: 5 },
					BEvent::Endowed {
						account: UnclaimedBalanceReceiver::get(),
						free_balance: 1_000
					},
					BEvent::Transfer {
						from: pool_bonus_account(pool_id),
						to: UnclaimedBalanceReceiver::get(),
						amount: 1_000
					}
				]
			);
			assert_eq!(Balances::free_balance(UnclaimedBalanceReceiver::get()), 1_000);
		});
}

#[test]
fn withdraw_unbonded_handles_faulty_sub_pool_accounting() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		bond_extra(10, pool_id, 10);

		// Given
		let balance = Balances::free_balance(10);
		assert_eq!(Balances::minimum_balance(), 5);
		assert_eq!(Balances::free_balance(10), balance);
		assert_eq!(Balances::free_balance(default_bonded_account()), 20);
		unsafe_set_state(pool_id, PoolState::Destroying);
		assert_ok!(Pools::fully_unbond(RuntimeOrigin::signed(10), pool_id, 10));

		// Simulate a slash that is not accounted for in the sub pools.
		Balances::make_free_balance_be(&default_bonded_account(), 15);
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(pool_id).unwrap().with_era,
			//------------------------------balance decrease is not account for
			unbonding_pools_with_era! { 3 => UnbondPool { points: 10, balance: 10 } }
		);

		CurrentEra::set(3);

		// When
		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(10), pool_id, 10, 0));

		// Then
		assert_eq!(Balances::free_balance(10), 5 + balance); // gets only 5 back
		assert_eq!(Balances::free_balance(default_bonded_account()), 10); // deposit is still bonded
	});
}

#[test]
fn withdraw_unbonded_errors_correctly() {
	ExtBuilder::default().with_check(0).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// Insert the sub-pool
		let sub_pools = SubPools {
			no_era: Default::default(),
			with_era: unbonding_pools_with_era! { 3 => UnbondPool { points: 10, balance: 10  }},
		};
		SubPoolsStorage::<Runtime>::insert(pool_id, sub_pools.clone());

		assert_noop!(
			Pools::withdraw_unbonded(RuntimeOrigin::signed(11), pool_id, 11, 0),
			Error::<Runtime>::PoolMemberNotFound
		);

		let mut member = PoolMember::default();
		UnbondingMembers::<Runtime>::insert(pool_id, 11, member.clone());

		// Simulate calling `unbond`
		member.unbonding_eras = member_unbonding_eras!(3 => 10);
		UnbondingMembers::<Runtime>::insert(pool_id, 11, member.clone());

		// We are still in the bonding duration
		assert_noop!(
			Pools::withdraw_unbonded(RuntimeOrigin::signed(11), pool_id, 11, 0),
			Error::<Runtime>::CannotWithdrawAny
		);

		// If we error the member does not get removed
		assert_eq!(UnbondingMembers::<Runtime>::get(pool_id, 11), Some(member));
		// and the sub pools do not get updated.
		assert_eq!(SubPoolsStorage::<Runtime>::get(pool_id).unwrap(), sub_pools)
	});
}

#[test]
fn withdraw_unbonded_kick() {
	ExtBuilder::default()
		.add_members(vec![(100, 100), (200, 200)])
		.build_and_execute(|| {
			let pool_id = NextPoolId::<Runtime>::get() - 1;

			// Given
			assert_ok!(Pools::fully_unbond(RuntimeOrigin::signed(100), pool_id, 100));
			assert_ok!(Pools::fully_unbond(RuntimeOrigin::signed(200), pool_id, 200));
			assert_eq!(
				BondedPool::<Runtime>::get(pool_id).unwrap(),
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
			CurrentEra::set(StakingMock::bonding_duration());

			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { creator: 10, pool_id, capacity: 1_000 },
					Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
					Event::Bonded { member: 100, pool_id, bonded: 100 },
					Event::Bonded { member: 200, pool_id, bonded: 200 },
					Event::Unbonded { member: 100, pool_id, points: 100, balance: 100, era: 3 },
					Event::Unbonded { member: 200, pool_id, points: 200, balance: 200, era: 3 }
				]
			);

			// Cannot kick as a nominator
			assert_noop!(
				Pools::withdraw_unbonded(RuntimeOrigin::signed(901), pool_id, 100, 0),
				Error::<Runtime>::NotKickerOrDestroying
			);

			// Can kick as admin
			assert_ok!(Pools::withdraw_unbonded(
				RuntimeOrigin::signed(DEFAULT_MANAGER),
				pool_id,
				100,
				0
			));

			// Can kick as state toggler
			assert_ok!(Pools::withdraw_unbonded(
				RuntimeOrigin::signed(DEFAULT_MANAGER),
				pool_id,
				200,
				0
			));

			assert_eq!(Balances::free_balance(100), 100 + 100);
			assert_eq!(Balances::free_balance(200), 200 + 200);
			assert!(!UnbondingMembers::<Runtime>::contains_key(pool_id, 100));
			assert!(!UnbondingMembers::<Runtime>::contains_key(pool_id, 200));
			assert_eq!(SubPoolsStorage::<Runtime>::get(pool_id).unwrap(), Default::default());
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Withdrawn { member: 100, pool_id, points: 100, balance: 100 },
					Event::Withdrawn { member: 200, pool_id, points: 200, balance: 200 },
				]
			);
		});
}

#[test]
fn withdraw_unbonded_destroying_permissionless() {
	ExtBuilder::default().add_members(vec![(100, 100)]).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// Given
		assert_ok!(Pools::fully_unbond(RuntimeOrigin::signed(100), pool_id, 100));
		assert_eq!(
			BondedPool::<Runtime>::get(pool_id).unwrap(),
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
		CurrentEra::set(StakingMock::bonding_duration());
		assert_eq!(Balances::free_balance(100), 100);

		// Cannot permissionlessly withdraw
		assert_noop!(
			Pools::fully_unbond(RuntimeOrigin::signed(420), pool_id, 100),
			Error::<Runtime>::NotKickerOrDestroying
		);

		// Given
		unsafe_set_state(pool_id, PoolState::Destroying);

		// Can permissionlesly withdraw a member that is not the depositor
		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(420), pool_id, 100, 0));

		assert_eq!(SubPoolsStorage::<Runtime>::get(pool_id).unwrap(), Default::default(),);
		assert_eq!(Balances::free_balance(100), 100 + 100);
		assert!(!UnbondingMembers::<Runtime>::contains_key(pool_id, 100));
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { creator: 10, pool_id, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
				Event::Bonded { member: 100, pool_id, bonded: 100 },
				Event::Unbonded { member: 100, pool_id, points: 100, balance: 100, era: 3 },
				Event::Withdrawn { member: 100, pool_id, points: 100, balance: 100 },
			]
		);
	});
}

#[test]
fn partial_withdraw_unbonded_depositor() {
	ExtBuilder::default().ed(1).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		let min_bond = Pools::depositor_min_bond();

		bond_extra(10, pool_id, 10 + min_bond);
		unsafe_set_state(pool_id, PoolState::Destroying);

		// given
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 6));
		CurrentEra::set(1);
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 1));
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
		assert_eq!(Pools::member_points(pool_id, 10), min_bond + 3);
		assert_eq!(UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_points(), 7);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { creator: 10, pool_id, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
				Event::Bonded { member: 10, pool_id, bonded: min_bond + 10 },
				Event::Unbonded { member: 10, pool_id, points: 6, balance: 6, era: 3 },
				Event::Unbonded { member: 10, pool_id, points: 1, balance: 1, era: 4 }
			]
		);

		// when
		CurrentEra::set(2);
		assert_noop!(
			Pools::withdraw_unbonded(RuntimeOrigin::signed(10), pool_id, 10, 0),
			Error::<Runtime>::CannotWithdrawAny
		);

		// when
		CurrentEra::set(3);
		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(10), pool_id, 10, 0));

		// then
		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_eras,
			member_unbonding_eras!(4 => 1)
		);
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
			SubPools {
				no_era: Default::default(),
				with_era: unbonding_pools_with_era! {
					4 => UnbondPool { points: 1, balance: 1 }
				}
			}
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Withdrawn { member: 10, pool_id, points: 6, balance: 6 }]
		);

		// when
		CurrentEra::set(4);
		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(10), pool_id, 10, 0));

		// then
		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_eras,
			member_unbonding_eras!()
		);
		assert_eq!(SubPoolsStorage::<Runtime>::get(pool_id).unwrap(), Default::default());
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Withdrawn { member: 10, pool_id, points: 1, balance: 1 },]
		);

		// when repeating:
		assert_noop!(
			Pools::withdraw_unbonded(RuntimeOrigin::signed(10), pool_id, 10, 0),
			Error::<Runtime>::CannotWithdrawAny
		);
	});
}

#[test]
fn partial_withdraw_unbonded_non_depositor() {
	ExtBuilder::default().add_members(vec![(11, 10)]).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// given
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(11), pool_id, 11, 6));
		CurrentEra::set(1);
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(11), pool_id, 11, 1));
		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 11).unwrap().unbonding_eras,
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
		assert_eq!(Pools::member_points(pool_id, 11), 3);
		assert_eq!(UnbondingMembers::<Runtime>::get(pool_id, 11).unwrap().unbonding_points(), 7);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { creator: 10, pool_id, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
				Event::Bonded { member: 11, pool_id, bonded: 10 },
				Event::Unbonded { member: 11, pool_id, points: 6, balance: 6, era: 3 },
				Event::Unbonded { member: 11, pool_id, points: 1, balance: 1, era: 4 }
			]
		);

		// when
		CurrentEra::set(2);
		assert_noop!(
			Pools::withdraw_unbonded(RuntimeOrigin::signed(11), pool_id, 11, 0),
			Error::<Runtime>::CannotWithdrawAny
		);

		// when
		CurrentEra::set(3);
		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(11), pool_id, 11, 0));

		// then
		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 11).unwrap().unbonding_eras,
			member_unbonding_eras!(4 => 1)
		);
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
			SubPools {
				no_era: Default::default(),
				with_era: unbonding_pools_with_era! {
					4 => UnbondPool { points: 1, balance: 1 }
				}
			}
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Withdrawn { member: 11, pool_id, points: 6, balance: 6 }]
		);

		// when
		CurrentEra::set(4);
		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(11), pool_id, 11, 0));

		// then
		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 11).unwrap().unbonding_eras,
			member_unbonding_eras!()
		);
		assert_eq!(SubPoolsStorage::<Runtime>::get(pool_id).unwrap(), Default::default());
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Withdrawn { member: 11, pool_id, points: 1, balance: 1 }]
		);

		// when repeating:
		assert_noop!(
			Pools::withdraw_unbonded(RuntimeOrigin::signed(11), pool_id, 11, 0),
			Error::<Runtime>::CannotWithdrawAny
		);
	});
}

#[test]
fn full_multi_step_withdrawing_non_depositor() {
	ExtBuilder::default().add_members(vec![(100, 100)]).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// given
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(100), pool_id, 100, 75));
		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 100).unwrap().unbonding_eras,
			member_unbonding_eras!(3 => 75)
		);

		// progress one era and unbond the leftover.
		CurrentEra::set(1);
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(100), pool_id, 100, 25));
		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 100).unwrap().unbonding_eras,
			member_unbonding_eras!(3 => 75, 4 => 25)
		);

		assert_noop!(
			Pools::withdraw_unbonded(RuntimeOrigin::signed(100), pool_id, 100, 0),
			Error::<Runtime>::CannotWithdrawAny
		);

		// now the 75 should be free.
		CurrentEra::set(3);
		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(100), pool_id, 100, 0));
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { creator: 10, pool_id, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
				Event::Bonded { member: 100, pool_id, bonded: 100 },
				Event::Unbonded { member: 100, pool_id, points: 75, balance: 75, era: 3 },
				Event::Unbonded { member: 100, pool_id, points: 25, balance: 25, era: 4 },
				Event::Withdrawn { member: 100, pool_id, points: 75, balance: 75 },
			]
		);
		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 100).unwrap().unbonding_eras,
			member_unbonding_eras!(4 => 25)
		);

		// the 25 should be free now, and the member removed.
		CurrentEra::set(4);
		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(100), pool_id, 100, 0));
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Withdrawn { member: 100, pool_id, points: 25, balance: 25 },]
		);
	})
}

#[test]
fn out_of_sync_unbonding_chunks() {
	// the unbonding_eras in pool member are always fixed to the era at which they are unlocked,
	// but the actual unbonding pools get pruned and might get combined in the no_era pool.
	// Pools are only merged when one unbonds, so we unbond a little bit on every era to
	// simulate this.
	ExtBuilder::default()
		.add_members(vec![(20, 100), (30, 100)])
		.build_and_execute(|| {
			System::reset_events();
			let pool_id = NextPoolId::<Runtime>::get() - 1;

			// when
			assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 5));
			assert_ok!(Pools::unbond(RuntimeOrigin::signed(30), pool_id, 30, 5));

			// then member-local unbonding is pretty much in sync with the global pools.
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 20).unwrap().unbonding_eras,
				member_unbonding_eras!(3 => 5)
			);
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 30).unwrap().unbonding_eras,
				member_unbonding_eras!(3 => 5)
			);
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
				SubPools {
					no_era: Default::default(),
					with_era: unbonding_pools_with_era! {
						3 => UnbondPool { points: 10, balance: 10 }
					}
				}
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Unbonded { member: 20, pool_id, points: 5, balance: 5, era: 3 },
					Event::Unbonded { member: 30, pool_id, points: 5, balance: 5, era: 3 },
				]
			);

			// when
			CurrentEra::set(1);
			assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 5));

			// then still member-local unbonding is pretty much in sync with the global pools.
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 20).unwrap().unbonding_eras,
				member_unbonding_eras!(3 => 5, 4 => 5)
			);
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
				SubPools {
					no_era: Default::default(),
					with_era: unbonding_pools_with_era! {
						3 => UnbondPool { points: 10, balance: 10 },
						4 => UnbondPool { points: 5, balance: 5 }
					}
				}
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![Event::Unbonded { member: 20, pool_id, points: 5, balance: 5, era: 4 }]
			);

			// when
			CurrentEra::set(2);
			assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 5));

			// then still member-local unbonding is pretty much in sync with the global pools.
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 20).unwrap().unbonding_eras,
				member_unbonding_eras!(3 => 5, 4 => 5, 5 => 5)
			);
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
				SubPools {
					no_era: Default::default(),
					with_era: unbonding_pools_with_era! {
						3 => UnbondPool { points: 10, balance: 10 },
						4 => UnbondPool { points: 5, balance: 5 },
						5 => UnbondPool { points: 5, balance: 5 }
					}
				}
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![Event::Unbonded { member: 20, pool_id, points: 5, balance: 5, era: 5 }]
			);

			// when
			CurrentEra::set(5);
			assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 5));

			// then
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 20).unwrap().unbonding_eras,
				member_unbonding_eras!(3 => 5, 4 => 5, 5 => 5, 8 => 5)
			);
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
				SubPools {
					// era 3 is merged into no_era.
					no_era: UnbondPool { points: 10, balance: 10 },
					with_era: unbonding_pools_with_era! {
						4 => UnbondPool { points: 5, balance: 5 },
						5 => UnbondPool { points: 5, balance: 5 },
						8 => UnbondPool { points: 5, balance: 5 }
					}
				}
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![Event::Unbonded { member: 20, pool_id, points: 5, balance: 5, era: 8 }]
			);

			// now we start withdrawing unlocked bonds.

			// when
			assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(20), pool_id, 20, 0));
			// then
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 20).unwrap().unbonding_eras,
				member_unbonding_eras!(8 => 5)
			);
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
				SubPools {
					// era 3 is merged into no_era.
					no_era: UnbondPool { points: 5, balance: 5 },
					with_era: unbonding_pools_with_era! {
						8 => UnbondPool { points: 5, balance: 5 }
					}
				}
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![Event::Withdrawn { member: 20, pool_id, points: 15, balance: 15 }]
			);

			// when
			assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(30), pool_id, 30, 0));
			// then
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 30).unwrap().unbonding_eras,
				member_unbonding_eras!()
			);
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(pool_id).unwrap(),
				SubPools {
					// era 3 is merged into no_era.
					no_era: Default::default(),
					with_era: unbonding_pools_with_era! {
						8 => UnbondPool { points: 5, balance: 5 }
					}
				}
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![Event::Withdrawn { member: 30, pool_id, points: 5, balance: 5 }]
			);
		})
}

#[test]
fn full_multi_step_withdrawing_depositor() {
	ExtBuilder::default().ed(1).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// depositor now has 20, they can unbond to 10.
		assert_eq!(Pools::depositor_min_bond(), 10);
		bond_extra(10, pool_id, 20);

		// now they can.
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 7));

		// progress one era and unbond the leftover.
		CurrentEra::set(1);
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 3));

		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_eras,
			member_unbonding_eras!(3 => 7, 4 => 3)
		);

		// they can't unbond to a value below `MinJoinBond` but 0.
		assert_noop!(
			Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 9),
			Error::<Runtime>::MinimumBondNotMet
		);

		// but now they can.
		unsafe_set_state(pool_id, PoolState::Destroying);
		assert_noop!(
			Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 9),
			Error::<Runtime>::MinimumBondNotMet
		);
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 10));

		// now the 7 should be free.
		CurrentEra::set(3);
		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(10), pool_id, 10, 0));

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { creator: 10, pool_id, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
				Event::Bonded { member: 10, pool_id, bonded: 20 },
				Event::Unbonded { member: 10, pool_id, balance: 7, points: 7, era: 3 },
				Event::Unbonded { member: 10, pool_id, balance: 3, points: 3, era: 4 },
				Event::Unbonded { member: 10, pool_id, balance: 10, points: 10, era: 4 },
				Event::Withdrawn { member: 10, pool_id, balance: 7, points: 7 }
			]
		);
		assert_eq!(
			UnbondingMembers::<Runtime>::get(pool_id, 10).unwrap().unbonding_eras,
			member_unbonding_eras!(4 => 13)
		);

		// the 13 should be free now, and the member removed.
		CurrentEra::set(4);
		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(10), pool_id, 10, 0));

		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Withdrawn { member: 10, pool_id, points: 13, balance: 13 },]
		);
	})
}
