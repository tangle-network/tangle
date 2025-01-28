use super::*;

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
fn pool_withdraw_unbonded_works() {
	ExtBuilder::default().add_members(vec![(20, 10)]).build_and_execute(|| {
		// Given 10 unbond'ed directly against the pool account

		assert_ok!(Lst::unbond(RuntimeOrigin::signed(20), 20, 5));

		assert_eq!(StakingMock::active_stake(&default_bonded_account()), Ok(15));
		assert_eq!(StakingMock::total_stake(&default_bonded_account()), Ok(20));
		assert_eq!(Balances::free_balance(&default_bonded_account()), 20);

		// When
		CurrentEra::set(StakingMock::current_era() + StakingMock::bonding_duration() + 1);
		assert_ok!(Lst::pool_withdraw_unbonded(RuntimeOrigin::signed(10), 1, 0));

		// Then their unbonding balance is no longer locked
		assert_eq!(StakingMock::active_stake(&default_bonded_account()), Ok(15));
		assert_eq!(StakingMock::total_stake(&default_bonded_account()), Ok(15));
		assert_eq!(Balances::free_balance(&default_bonded_account()), 20);
	});
}

use sp_runtime::bounded_btree_map;

#[test]
fn withdraw_unbonded_works_against_slashed_no_era_sub_pool() {
	ExtBuilder::default()
		.add_members(vec![(40, 40), (550, 550)])
		.build_and_execute(|| {
			// reduce the noise a bit.
			let _ = balances_events_since_last_call();

			// Given
			assert_eq!(StakingMock::bonding_duration(), 3);
			assert_ok!(Lst::fully_unbond(RuntimeOrigin::signed(550), 550));
			assert_ok!(Lst::fully_unbond(RuntimeOrigin::signed(40), 40));
			assert_eq!(Currency::free_balance(&default_bonded_account()), 600);

			let mut current_era = 1;
			CurrentEra::set(current_era);

			let mut sub_pools = SubPoolsStorage::<Runtime>::get(1).unwrap();
			let unbond_pool = sub_pools.with_era.get_mut(&3).unwrap();
			// Sanity check
			assert_eq!(*unbond_pool, UnbondPool { points: 550 + 40, balance: 550 + 40 });
			assert_eq!(TotalValueLocked::<Runtime>::get(), 600);

			// Simulate a slash to the pool with_era(current_era), decreasing the balance by
			// half
			{
				unbond_pool.balance /= 2; // 295
				SubPoolsStorage::<Runtime>::insert(1, sub_pools);

				// Adjust the TVL for this non-api usage (direct sub-pool modification)
				TotalValueLocked::<Runtime>::mutate(|x| *x = x.saturating_sub(295));

				// Update the equivalent of the unbonding chunks for the `StakingMock`
				let mut x = UnbondingBalanceMap::get();
				x.get_mut(&default_bonded_account())
					.unwrap()
					.get_mut(current_era as usize)
					.unwrap()
					.1 /= 2;
				UnbondingBalanceMap::set(&x);

				Currency::make_free_balance_be(
					&default_bonded_account(),
					Currency::free_balance(&default_bonded_account()) / 2, // 300
				);
				assert_eq!(StakingMock::active_stake(&default_bonded_account()).unwrap(), 10);
				StakingMock::slash_by(1, 5);
				assert_eq!(StakingMock::active_stake(&default_bonded_account()).unwrap(), 5);
			};

			// Advance the current_era to ensure all `with_era` pools will be merged into
			// `no_era` pool
			current_era += TotalUnbondingPools::<Runtime>::get();
			CurrentEra::set(current_era);

			// Simulate some other call to unbond that would merge `with_era` pools into
			// `no_era`
			let sub_pools =
				SubPoolsStorage::<Runtime>::get(1).unwrap().maybe_merge_pools(current_era);
			SubPoolsStorage::<Runtime>::insert(1, sub_pools);

			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap(),
				SubPools {
					no_era: UnbondPool { points: 550 + 40, balance: 275 + 20 },
					with_era: Default::default()
				}
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { depositor: 10, pool_id: 1 },
					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
					Event::Bonded { member: 40, pool_id: 1, bonded: 40, joined: true },
					Event::Bonded { member: 550, pool_id: 1, bonded: 550, joined: true },
					Event::Unbonded { member: 550, pool_id: 1, points: 550, balance: 550, era: 3 },
					Event::Unbonded { member: 40, pool_id: 1, points: 40, balance: 40, era: 3 },
					Event::PoolSlashed { pool_id: 1, balance: 5 }
				]
			);
			assert_eq!(
				balances_events_since_last_call(),
				vec![BEvent::Burned { who: default_bonded_account(), amount: 300 }]
			);

			// When
			assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(550), 550, 0));

			// Then
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap().no_era,
				UnbondPool { points: 40, balance: 20 }
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Withdrawn { member: 550, pool_id: 1, balance: 275, points: 550 },
					Event::MemberRemoved { pool_id: 1, member: 550 }
				]
			);
			assert_eq!(
				balances_events_since_last_call(),
				vec![BEvent::Transfer { from: default_bonded_account(), to: 550, amount: 275 }]
			);

			// When
			assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(40), 40, 0));

			// Then
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap().no_era,
				UnbondPool { points: 0, balance: 0 }
			);

			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Withdrawn { member: 40, pool_id: 1, balance: 20, points: 40 },
					Event::MemberRemoved { pool_id: 1, member: 40 }
				]
			);
			assert_eq!(
				balances_events_since_last_call(),
				vec![BEvent::Transfer { from: default_bonded_account(), to: 40, amount: 20 }]
			);

			// now, finally, the depositor can take out its share.
			unsafe_set_state(1, PoolState::Destroying);
			assert_ok!(fully_unbond_permissioned(10, 1));

			current_era += 3;
			CurrentEra::set(current_era);

			// when
			assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(10), 10, 0));
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Unbonded { member: 10, pool_id: 1, balance: 5, points: 5, era: 9 },
					Event::Withdrawn { member: 10, pool_id: 1, balance: 5, points: 5 },
					Event::MemberRemoved { pool_id: 1, member: 10 },
					Event::Destroyed { pool_id: 1 }
				]
			);
			assert!(!Metadata::<T>::contains_key(1));
			assert_eq!(
				balances_events_since_last_call(),
				vec![
					BEvent::Transfer { from: default_bonded_account(), to: 10, amount: 5 },
					BEvent::Thawed { who: default_reward_account(), amount: 5 },
					BEvent::Transfer { from: default_reward_account(), to: 10, amount: 5 }
				]
			);
		});
}

#[test]
fn withdraw_unbonded_works_against_slashed_with_era_sub_pools() {
	ExtBuilder::default()
		.add_members(vec![(40, 40), (550, 550)])
		.build_and_execute(|| {
			let _ = balances_events_since_last_call();

			// Given
			// current bond is 600, we slash it all to 300.
			StakingMock::slash_by(1, 300);
			Currency::make_free_balance_be(&default_bonded_account(), 300);
			assert_eq!(StakingMock::total_stake(&default_bonded_account()), Ok(300));

			assert_ok!(fully_unbond_permissioned(40, 1));
			assert_ok!(fully_unbond_permissioned(550, 1));

			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap().with_era,
				unbonding_pools_with_era! { 3 => UnbondPool { points: 550 / 2 + 40 / 2, balance: 550 / 2 + 40 / 2
				}}
			);

			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { depositor: 10, pool_id: 1 },
					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
					Event::Bonded { member: 40, pool_id: 1, bonded: 40, joined: true },
					Event::Bonded { member: 550, pool_id: 1, bonded: 550, joined: true },
					Event::PoolSlashed { pool_id: 1, balance: 300 },
					Event::Unbonded { member: 40, pool_id: 1, balance: 20, points: 20, era: 3 },
					Event::Unbonded { member: 550, pool_id: 1, balance: 275, points: 275, era: 3 }
				]
			);
			assert_eq!(
				balances_events_since_last_call(),
				vec![BEvent::Burned { who: default_bonded_account(), amount: 300 },]
			);

			CurrentEra::set(StakingMock::bonding_duration());

			// When
			assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(40), 40, 0));

			// Then
			assert_eq!(
				balances_events_since_last_call(),
				vec![BEvent::Transfer { from: default_bonded_account(), to: 40, amount: 20 },]
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Withdrawn { member: 40, pool_id: 1, balance: 20, points: 20 },
					Event::MemberRemoved { pool_id: 1, member: 40 }
				]
			);

			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap().with_era,
				unbonding_pools_with_era! { 3 => UnbondPool { points: 550 / 2, balance: 550 / 2 }}
			);

			// When
			assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(550), 550, 0));

			// Then
			assert_eq!(
				balances_events_since_last_call(),
				vec![BEvent::Transfer { from: default_bonded_account(), to: 550, amount: 275 },]
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Withdrawn { member: 550, pool_id: 1, balance: 275, points: 275 },
					Event::MemberRemoved { pool_id: 1, member: 550 }
				]
			);
			assert!(SubPoolsStorage::<Runtime>::get(1).unwrap().with_era.is_empty());

			// now, finally, the depositor can take out its share.
			unsafe_set_state(1, PoolState::Destroying);
			assert_ok!(fully_unbond_permissioned(10, 1));

			// because everyone else has left, the points
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap().with_era,
				unbonding_pools_with_era! { 6 => UnbondPool { points: 5, balance: 5 }}
			);

			CurrentEra::set(CurrentEra::get() + 3);

			// set metadata to check that it's being removed on dissolve
			assert_ok!(Lst::set_metadata(RuntimeOrigin::signed(900), 1, vec![1, 1]));
			assert!(Metadata::<T>::contains_key(1));

			// when
			assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(10), 10, 0));

			// then
			assert_eq!(Currency::free_balance(&10), 10 + 35);
			assert_eq!(Currency::free_balance(&default_bonded_account()), 0);

			// in this test 10 also gets a fair share of the slash, because the slash was
			// applied to the bonded account.
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Unbonded { member: 10, pool_id: 1, points: 5, balance: 5, era: 6 },
					Event::Withdrawn { member: 10, pool_id: 1, points: 5, balance: 5 },
					Event::MemberRemoved { pool_id: 1, member: 10 },
					Event::Destroyed { pool_id: 1 }
				]
			);
			assert!(!Metadata::<T>::contains_key(1));
			assert_eq!(
				balances_events_since_last_call(),
				vec![
					BEvent::Transfer { from: default_bonded_account(), to: 10, amount: 5 },
					BEvent::Thawed { who: default_reward_account(), amount: 5 },
					BEvent::Transfer { from: default_reward_account(), to: 10, amount: 5 }
				]
			);
		});
}

#[test]
fn withdraw_unbonded_handles_faulty_sub_pool_accounting() {
	ExtBuilder::default().build_and_execute(|| {
		// Given
		assert_eq!(<Balances as CurrencyT<AccountId>>::minimum_balance(), 5);
		assert_eq!(Currency::free_balance(&10), 35);
		assert_eq!(Currency::free_balance(&default_bonded_account()), 10);
		unsafe_set_state(1, PoolState::Destroying);
		assert_ok!(Lst::fully_unbond(RuntimeOrigin::signed(10), 10));

		// Simulate a slash that is not accounted for in the sub pools.
		Currency::make_free_balance_be(&default_bonded_account(), 5);
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(1).unwrap().with_era,
			//------------------------------balance decrease is not account for
			unbonding_pools_with_era! { 3 => UnbondPool { points: 10, balance: 10 } }
		);

		CurrentEra::set(3);

		// When
		assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(10), 10, 0));

		// Then
		assert_eq!(Currency::free_balance(&10), 10 + 35);
		assert_eq!(Currency::free_balance(&default_bonded_account()), 0);
	});
}

#[test]
fn withdraw_unbonded_kick() {
	ExtBuilder::default()
		.add_members(vec![(100, 100), (200, 200)])
		.build_and_execute(|| {
			// Given
			assert_ok!(Lst::fully_unbond(RuntimeOrigin::signed(100), 100));
			assert_ok!(Lst::fully_unbond(RuntimeOrigin::signed(200), 200));
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
			CurrentEra::set(StakingMock::bonding_duration());

			// Cannot kick when pool is open
			assert_noop!(
				Lst::withdraw_unbonded(RuntimeOrigin::signed(902), 100, 0),
				Error::<Runtime>::NotKickerOrDestroying
			);
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { depositor: 10, pool_id: 1 },
					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
					Event::Bonded { member: 100, pool_id: 1, bonded: 100, joined: true },
					Event::Bonded { member: 200, pool_id: 1, bonded: 200, joined: true },
					Event::Unbonded { member: 100, pool_id: 1, points: 100, balance: 100, era: 3 },
					Event::Unbonded { member: 200, pool_id: 1, points: 200, balance: 200, era: 3 }
				]
			);

			// Given
			unsafe_set_state(1, PoolState::Blocked);

			// Cannot kick as a nominator
			assert_noop!(
				Lst::withdraw_unbonded(RuntimeOrigin::signed(901), 100, 0),
				Error::<Runtime>::NotKickerOrDestroying
			);

			// Can kick as root
			assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(900), 100, 0));

			// Can kick as bouncer
			assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(900), 200, 0));

			assert_eq!(Currency::free_balance(&100), 100 + 100);
			assert_eq!(Currency::free_balance(&200), 200 + 200);
			assert_eq!(SubPoolsStorage::<Runtime>::get(1).unwrap(), Default::default());
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Withdrawn { member: 100, pool_id: 1, points: 100, balance: 100 },
					Event::MemberRemoved { pool_id: 1, member: 100 },
					Event::Withdrawn { member: 200, pool_id: 1, points: 200, balance: 200 },
					Event::MemberRemoved { pool_id: 1, member: 200 }
				]
			);
		});
}

#[test]
fn withdraw_unbonded_destroying_permissionless() {
	ExtBuilder::default().add_members(vec![(100, 100)]).build_and_execute(|| {
		// Given
		assert_ok!(Lst::fully_unbond(RuntimeOrigin::signed(100), 100));
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
		CurrentEra::set(StakingMock::bonding_duration());
		assert_eq!(Currency::free_balance(&100), 100);

		// Cannot permissionlessly withdraw
		assert_noop!(
			Lst::fully_unbond(RuntimeOrigin::signed(420), 100),
			Error::<Runtime>::NotKickerOrDestroying
		);

		// Given
		unsafe_set_state(1, PoolState::Destroying);

		// Can permissionlessly withdraw a member that is not the depositor
		assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(420), 100, 0));

		assert_eq!(SubPoolsStorage::<Runtime>::get(1).unwrap(), Default::default(),);
		assert_eq!(Currency::free_balance(&100), 100 + 100);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
				Event::Bonded { member: 100, pool_id: 1, bonded: 100, joined: true },
				Event::Unbonded { member: 100, pool_id: 1, points: 100, balance: 100, era: 3 },
				Event::Withdrawn { member: 100, pool_id: 1, points: 100, balance: 100 },
				Event::MemberRemoved { pool_id: 1, member: 100 }
			]
		);
	});
}

#[test]
fn partial_withdraw_unbonded_depositor() {
	ExtBuilder::default().ed(1).build_and_execute(|| {
		assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), BondExtra::FreeBalance(10)));
		unsafe_set_state(1, PoolState::Destroying);

		// given
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(10), 10, 6));
		CurrentEra::set(1);
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(10), 10, 1));

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
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: false },
				Event::Unbonded { member: 10, pool_id: 1, points: 6, balance: 6, era: 3 },
				Event::Unbonded { member: 10, pool_id: 1, points: 1, balance: 1, era: 4 }
			]
		);

		// when
		CurrentEra::set(2);
		assert_noop!(
			Lst::withdraw_unbonded(RuntimeOrigin::signed(10), 10, 0),
			Error::<Runtime>::CannotWithdrawAny
		);

		// when
		CurrentEra::set(3);
		assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(10), 10, 0));

		// then
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(1).unwrap(),
			SubPools {
				no_era: Default::default(),
				with_era: unbonding_pools_with_era! {
					4 => UnbondPool { points: 1, balance: 1 }
				}
			}
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Withdrawn { member: 10, pool_id: 1, points: 6, balance: 6 }]
		);

		// when
		CurrentEra::set(4);
		assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(10), 10, 0));

		// then
		assert_eq!(SubPoolsStorage::<Runtime>::get(1).unwrap(), Default::default());
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Withdrawn { member: 10, pool_id: 1, points: 1, balance: 1 },]
		);

		// when repeating:
		assert_noop!(
			Lst::withdraw_unbonded(RuntimeOrigin::signed(10), 10, 0),
			Error::<Runtime>::CannotWithdrawAny
		);
	});
}

#[test]
fn partial_withdraw_unbonded_non_depositor() {
	ExtBuilder::default().add_members(vec![(11, 10)]).build_and_execute(|| {
		// given
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(11), 11, 6));
		CurrentEra::set(1);
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(11), 11, 1));
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
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
				Event::Bonded { member: 11, pool_id: 1, bonded: 10, joined: true },
				Event::Unbonded { member: 11, pool_id: 1, points: 6, balance: 6, era: 3 },
				Event::Unbonded { member: 11, pool_id: 1, points: 1, balance: 1, era: 4 }
			]
		);

		// when
		CurrentEra::set(2);
		assert_noop!(
			Lst::withdraw_unbonded(RuntimeOrigin::signed(11), 11, 0),
			Error::<Runtime>::CannotWithdrawAny
		);

		// when
		CurrentEra::set(3);
		assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(11), 11, 0));

		// then
		assert_eq!(
			SubPoolsStorage::<Runtime>::get(1).unwrap(),
			SubPools {
				no_era: Default::default(),
				with_era: unbonding_pools_with_era! {
					4 => UnbondPool { points: 1, balance: 1 }
				}
			}
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Withdrawn { member: 11, pool_id: 1, points: 6, balance: 6 }]
		);

		// when
		CurrentEra::set(4);
		assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(11), 11, 0));

		// then
		assert_eq!(SubPoolsStorage::<Runtime>::get(1).unwrap(), Default::default());
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Withdrawn { member: 11, pool_id: 1, points: 1, balance: 1 }]
		);

		// when repeating:
		assert_noop!(
			Lst::withdraw_unbonded(RuntimeOrigin::signed(11), 11, 0),
			Error::<Runtime>::CannotWithdrawAny
		);
	});
}

#[test]
fn full_multi_step_withdrawing_non_depositor() {
	ExtBuilder::default().add_members(vec![(100, 100)]).build_and_execute(|| {
		assert_eq!(TotalValueLocked::<T>::get(), 110);
		// given
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(100), 100, 75));

		// tvl unchanged.
		assert_eq!(TotalValueLocked::<T>::get(), 110);

		// progress one era and unbond the leftover.
		CurrentEra::set(1);
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(100), 100, 25));

		assert_noop!(
			Lst::withdraw_unbonded(RuntimeOrigin::signed(100), 100, 0),
			Error::<Runtime>::CannotWithdrawAny
		);
		// tvl unchanged.
		assert_eq!(TotalValueLocked::<T>::get(), 110);

		// now the 75 should be free.
		CurrentEra::set(3);
		assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(100), 100, 0));
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
				Event::Bonded { member: 100, pool_id: 1, bonded: 100, joined: true },
				Event::Unbonded { member: 100, pool_id: 1, points: 75, balance: 75, era: 3 },
				Event::Unbonded { member: 100, pool_id: 1, points: 25, balance: 25, era: 4 },
				Event::Withdrawn { member: 100, pool_id: 1, points: 75, balance: 75 },
			]
		);
		// tvl updated
		assert_eq!(TotalValueLocked::<T>::get(), 35);

		// the 25 should be free now, and the member removed.
		CurrentEra::set(4);
		assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(100), 100, 0));
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Withdrawn { member: 100, pool_id: 1, points: 25, balance: 25 },
				Event::MemberRemoved { pool_id: 1, member: 100 }
			]
		);
	})
}

#[test]
fn out_of_sync_unbonding_chunks() {
	// the unbonding_eras in pool member are always fixed to the era at which they are unlocked,
	// but the actual unbonding pools get pruned and might get combined in the no_era pool.
	// Lst are only merged when one unbonds, so we unbond a little bit on every era to
	// simulate this.
	ExtBuilder::default()
		.add_members(vec![(20, 100), (30, 100)])
		.build_and_execute(|| {
			System::reset_events();

			// when
			assert_ok!(Lst::unbond(RuntimeOrigin::signed(20), 20, 5));
			assert_ok!(Lst::unbond(RuntimeOrigin::signed(30), 30, 5));

			// then member-local unbonding is pretty much in sync with the global pools.
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap(),
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
					Event::Unbonded { member: 20, pool_id: 1, points: 5, balance: 5, era: 3 },
					Event::Unbonded { member: 30, pool_id: 1, points: 5, balance: 5, era: 3 },
				]
			);

			// when
			CurrentEra::set(1);
			assert_ok!(Lst::unbond(RuntimeOrigin::signed(20), 20, 5));

			// then still member-local unbonding is pretty much in sync with the global pools.
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap(),
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
				vec![Event::Unbonded { member: 20, pool_id: 1, points: 5, balance: 5, era: 4 }]
			);

			// when
			CurrentEra::set(2);
			assert_ok!(Lst::unbond(RuntimeOrigin::signed(20), 20, 5));

			// then still member-local unbonding is pretty much in sync with the global pools.
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap(),
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
				vec![Event::Unbonded { member: 20, pool_id: 1, points: 5, balance: 5, era: 5 }]
			);

			// when
			CurrentEra::set(5);
			assert_ok!(Lst::unbond(RuntimeOrigin::signed(20), 20, 5));

			// then
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap(),
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
				vec![Event::Unbonded { member: 20, pool_id: 1, points: 5, balance: 5, era: 8 }]
			);

			// now we start withdrawing unlocked bonds.

			// when
			assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(20), 20, 0));
			// then
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap(),
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
				vec![Event::Withdrawn { member: 20, pool_id: 1, points: 15, balance: 15 }]
			);

			// when
			assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(30), 30, 0));
			// then
			assert_eq!(
				SubPoolsStorage::<Runtime>::get(1).unwrap(),
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
				vec![Event::Withdrawn { member: 30, pool_id: 1, points: 5, balance: 5 }]
			);
		})
}

#[test]
fn full_multi_step_withdrawing_depositor() {
	ExtBuilder::default().ed(1).build_and_execute(|| {
		// depositor now has 20, they can unbond to 10.
		assert_eq!(Lst::depositor_min_bond(), 10);
		assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), BondExtra::FreeBalance(10)));

		// now they can.
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(10), 10, 7));

		// progress one era and unbond the leftover.
		CurrentEra::set(1);
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(10), 10, 3));

		// they can't unbond to a value below 10 other than 0..
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(10), 10, 5),
			Error::<Runtime>::MinimumBondNotMet
		);

		// but not even full, because they pool is not yet destroying.
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(10), 10, 10),
			Error::<Runtime>::MinimumBondNotMet
		);

		// but now they can.
		unsafe_set_state(1, PoolState::Destroying);
		assert_noop!(
			Lst::unbond(RuntimeOrigin::signed(10), 10, 5),
			Error::<Runtime>::MinimumBondNotMet
		);
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(10), 10, 10));

		// now the 7 should be free.
		CurrentEra::set(3);
		assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(10), 10, 0));

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: false },
				Event::Unbonded { member: 10, pool_id: 1, balance: 7, points: 7, era: 3 },
				Event::Unbonded { member: 10, pool_id: 1, balance: 3, points: 3, era: 4 },
				Event::Unbonded { member: 10, pool_id: 1, balance: 10, points: 10, era: 4 },
				Event::Withdrawn { member: 10, pool_id: 1, balance: 7, points: 7 }
			]
		);

		// the 13 should be free now, and the member removed.
		CurrentEra::set(4);
		assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(10), 10, 0));

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Withdrawn { member: 10, pool_id: 1, points: 13, balance: 13 },
				Event::MemberRemoved { pool_id: 1, member: 10 },
				Event::Destroyed { pool_id: 1 },
			]
		);
		assert!(!Metadata::<T>::contains_key(1));
	})
}

#[test]
fn withdraw_unbonded_removes_claim_permissions_on_leave() {
	ExtBuilder::default().add_members(vec![(20, 20)]).build_and_execute(|| {
		// Given
		CurrentEra::set(1);

		assert_ok!(Lst::set_claim_permission(
			RuntimeOrigin::signed(20),
			ClaimPermission::PermissionlessAll
		));
		assert_ok!(Lst::unbond(RuntimeOrigin::signed(20), 20, 20));
		assert_eq!(ClaimPermissions::<Runtime>::get(20), ClaimPermission::PermissionlessAll);

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
				Event::Bonded { member: 20, pool_id: 1, bonded: 20, joined: true },
				Event::Unbonded { member: 20, pool_id: 1, balance: 20, points: 20, era: 4 },
			]
		);

		CurrentEra::set(5);

		// When
		assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(20), 20, 0));

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Withdrawn { member: 20, pool_id: 1, balance: 20, points: 20 },
				Event::MemberRemoved { pool_id: 1, member: 20 }
			]
		);
	});
}
