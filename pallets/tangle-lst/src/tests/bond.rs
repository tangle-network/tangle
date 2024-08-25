use super::*;
use crate::Event;
use frame_support::dispatch::RawOrigin;

#[test]
fn join_works() {
	let bonded = |_points| BondedPool::<Runtime> {
		id: 0,
		inner: BondedPoolInner {
			token_id: DEFAULT_TOKEN_ID,
			state: PoolState::Open,
			capacity: 1_000,
			commission: Commission::default(),
			name: Default::default(),
		},
	};
	ExtBuilder::default().with_check(0).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// Given
		Balances::make_free_balance_be(&11, ExistentialDeposit::get() + 2);
		assert!(!UnbondingMembers::<Runtime>::contains_key(pool_id, 11));

		// When
		assert_ok!(Pools::bond(RuntimeOrigin::signed(11), pool_id, 2.into()));

		// Then

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { creator: 10, pool_id, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
				Event::Bonded { member: 11, pool_id, bonded: 2 },
			]
		);

		assert_eq!(Pools::member_points(pool_id, 11), 2);

		assert_eq!(BondedPool::<Runtime>::get(pool_id).unwrap(), bonded(12));

		// Given
		// The bonded balance is slashed in half
		StakingMock::set_bonded_balance(
			Pools::compute_pool_account_id(pool_id, AccountType::Bonded),
			6,
		);

		// And
		Balances::make_free_balance_be(&12, ExistentialDeposit::get() + 12);
		assert!(!UnbondingMembers::<Runtime>::contains_key(pool_id, 12));

		// When
		assert_ok!(Pools::bond(RuntimeOrigin::signed(12), pool_id, 12.into()));

		// Then
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Bonded { member: 12, pool_id, bonded: 12 }]
		);

		assert_eq!(Pools::member_points(pool_id, 12), 24);
		assert_eq!(BondedPool::<Runtime>::get(pool_id).unwrap(), bonded(12 + 24));
	});
}

#[test]
fn join_pool_with_capacity() {
	let start_balance = 1_000 * UNIT;

	ExtBuilder::default().set_capacity(100).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		Balances::make_free_balance_be(&11, start_balance);
		Balances::make_free_balance_be(&12, start_balance);

		// account 10 is already part of the default pool created with 10 unit own stake and
		// capacity of 100
		assert_eq!(Pools::member_points(pool_id, Pools::deposit_account_id(pool_id)), 10);

		// account 11 is not part of the default pool
		assert!(UnbondingMembers::<Runtime>::get(pool_id, 11).is_none());

		// 11 cannot join with amount (91) overflowing available capacity (100 - 10 = 90)
		assert_noop!(
			Pools::bond(RuntimeOrigin::signed(11), pool_id, 91.into()),
			Error::<Runtime>::CapacityExceeded
		);

		// 11 can join with amount (80) not overflowing available capacity (100 - 10 = 90)
		assert_ok!(Pools::bond(RuntimeOrigin::signed(11), pool_id, 80.into()));

		// account 11 is now part of the default pool
		assert_eq!(Pools::member_points(pool_id, 11), 80);

		// account 12 is not part of the default pool
		assert!(UnbondingMembers::<Runtime>::get(pool_id, 12).is_none());

		// account 12 can join with amount (100) not overflowing available capacity (100 - 10 -
		// 80 = 10) by setting `reduce_amount_to_fill` to true
		assert_ok!(Pools::bond(RuntimeOrigin::signed(12), pool_id, BondValue::Fill));

		// account 12 now has points in the pool
		assert_eq!(Pools::member_points(pool_id, 12), 10);

		// only reduced amount was taken from account 12 and added to the pool
		assert_eq!(Balances::free_balance(12), start_balance - 10);
		let pool = BondedPool::<Runtime>::get(pool_id).unwrap();
		assert_eq!(Balances::free_balance(pool.bonded_account()), 100);
	});
}

#[test]
fn join_errors_correctly() {
	ExtBuilder::default().with_check(0).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;
		assert_eq!(pool_id, 0);

		assert_noop!(
			Pools::bond(RuntimeOrigin::signed(11), 123, 420.into()),
			Error::<Runtime>::PoolNotFound
		);

		// Force the pools bonded balance to 0, simulating a 100% slash
		StakingMock::set_bonded_balance(
			Pools::compute_pool_account_id(pool_id, AccountType::Bonded),
			0,
		);
		assert_noop!(
			Pools::bond(RuntimeOrigin::signed(11), pool_id, 420.into()),
			Error::<Runtime>::OverflowRisk
		);

		// Given a mocked bonded pool
		let pool_id = 123;
		BondedPool::<Runtime> {
			id: 123,
			inner: BondedPoolInner {
				token_id: DEFAULT_TOKEN_ID,
				state: PoolState::Open,
				capacity: 1_000,
				commission: Commission::default(),
			},
		}
		.put();

		// Force the points:balance ratio to `MaxPointsToBalance` (100/10)
		mint_points(pool_id, 100);
		let max_points_to_balance: u128 =
			<<Runtime as Config>::MaxPointsToBalance as Get<u8>>::get().into();

		StakingMock::set_bonded_balance(
			Pools::compute_pool_account_id(123, AccountType::Bonded),
			max_points_to_balance,
		);
		assert_noop!(
			Pools::bond(RuntimeOrigin::signed(11), 123, 420.into()),
			Error::<Runtime>::OverflowRisk
		);

		StakingMock::set_bonded_balance(
			Pools::compute_pool_account_id(123, AccountType::Bonded),
			Balance::MAX / max_points_to_balance,
		);
		// Balance needs to be gt Balance::MAX / `MaxPointsToBalance`
		assert_noop!(
			Pools::bond(RuntimeOrigin::signed(11), 123, 5.into()),
			sp_runtime::TokenError::FundsUnavailable,
		);

		StakingMock::set_bonded_balance(
			Pools::compute_pool_account_id(1, AccountType::Bonded),
			max_points_to_balance,
		);

		// Cannot join a pool that isn't open
		unsafe_set_state(123, PoolState::Destroying);
		assert_noop!(
			Pools::bond(RuntimeOrigin::signed(11), 123, max_points_to_balance.into()),
			Error::<Runtime>::NotOpen
		);

		// Given
		unsafe_set_state(123, PoolState::Open);
		MinJoinBond::<Runtime>::put(100);

		// Then
		assert_noop!(
			Pools::bond(RuntimeOrigin::signed(11), 123, 99.into()),
			Error::<Runtime>::MinimumBondNotMet
		);
	});
}

#[test]
fn join_max_member_limits_are_respected() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;
		let min_duration = <<Runtime as Config>::MinDuration as Get<EraIndex>>::get();

		// Given
		for i in 1..3 {
			let account = i + 100;
			Balances::make_free_balance_be(&account, 100 + Balances::minimum_balance());

			assert_ok!(Pools::bond(RuntimeOrigin::signed(account), pool_id, 100.into()));
		}

		Balances::make_free_balance_be(&103, 100 + Balances::minimum_balance());

		// Then
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { creator: 10, pool_id, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
				Event::Bonded { member: 101, pool_id, bonded: 100 },
				Event::Bonded { member: 102, pool_id, bonded: 100 }
			]
		);

		// give account (104) NFT allowing to create pools
		mint_pool_token(DEFAULT_TOKEN_ID + 1, 104);

		Balances::make_free_balance_be(&104, 100 + Balances::minimum_balance() * 2);
		assert_ok!(Pools::create(
			RuntimeOrigin::signed(104),
			DEFAULT_TOKEN_ID + 1,
			100,
			1_000,
			min_duration,
			Default::default(),
		));

		// Then
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { creator: 104, pool_id: 1, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(1), pool_id: 1, bonded: 100 }
			]
		);
	});
}

#[test]
fn join_mints_lst() {
	ExtBuilder::default().build_and_execute(|| {
		// Given
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		for i in 1..3 {
			let account = i + 100;
			Balances::make_free_balance_be(&account, 100 + Balances::minimum_balance());

			assert_ok!(Pools::bond(RuntimeOrigin::signed(account), pool_id, 100.into()));

			assert_eq!(Pools::member_points(pool_id, account), 100);
			assert_eq!(Balances::free_balance(account), Balances::minimum_balance());

			// assert event is emitted
			assert_event_deposited!(FungiblesEvent::Minted {
				issuer: RootOrSigned::Signed(<Runtime as Config>::LstCollectionOwner::get()),
				recipient: account,
				collection_id: LST_COLLECTION_ID,
				token_id: pool_id as u128,
				amount: 100,
			});
		}
	});
}

#[test]
fn join_slashed_pool_mints_extra_lst() {
	ExtBuilder::default()
		.add_members(vec![(40, 40), (550, 550)])
		.build_and_execute(|| {
			let pool_id = NextPoolId::<Runtime>::get() - 1;

			// Slash the pool by 50% (600 -> 300)
			StakingMock::set_bonded_balance(default_bonded_account(), 300);

			let account = 100;
			let initial_bond = 100;
			Balances::make_free_balance_be(&account, initial_bond + Balances::minimum_balance());

			assert_ok!(Pools::bond(RuntimeOrigin::signed(account), pool_id, initial_bond.into()));

			// should get double the initial bond since pool has been slashed by 50%
			assert_eq!(Pools::member_points(pool_id, account), initial_bond * 2);
			assert_eq!(Balances::free_balance(account), Balances::minimum_balance());

			// assert event is emitted
			assert_event_deposited!(FungiblesEvent::Minted {
				issuer: RootOrSigned::Signed(<Runtime as Config>::LstCollectionOwner::get()),
				recipient: account,
				collection_id: LST_COLLECTION_ID,
				token_id: pool_id as u128,
				amount: initial_bond * 2,
			});

			// sanity check, when the token is unbonded only the initial bond is returned
			assert_ok!(Pools::unbond(RuntimeOrigin::signed(account), pool_id, account, 200));

			// fast forward to unbonding era
			CurrentEra::set(3);
			assert_ok!(Pools::withdraw_unbonded(
				RuntimeOrigin::signed(account),
				pool_id,
				account,
				0
			));

			// back where we started
			assert_eq!(Balances::free_balance(account), initial_bond + Balances::minimum_balance());
		});
}

#[test]
fn join_multiple_pools() {
	ExtBuilder::default().build_and_execute(|| {
		let initial_token_id = DEFAULT_TOKEN_ID + 1;

		Balances::make_free_balance_be(&11, 1_000 * UNIT);

		let mut pools = vec![];

		// create 3 pools by account (10)
		for token_id in initial_token_id..initial_token_id + 4 {
			mint_pool_token(token_id, 10);
			assert_ok!(Pools::create(
				RawOrigin::Signed(10).into(),
				token_id,
				Pools::depositor_min_bond(),
				1_000,
				parameters::nomination_pools::MAX_POOL_DURATION,
				Default::default(),
			));
			pools.push(NextPoolId::<Runtime>::get() - 1);
		}

		for pool_id in pools {
			// join pool by account (11)
			assert_ok!(Pools::bond(
				RawOrigin::Signed(11).into(),
				pool_id,
				Pools::depositor_min_bond().into()
			));

			// ensure account (11) is a not a member of the pool
			let pool_member = UnbondingMembers::<Runtime>::get(pool_id, 11);
			assert!(pool_member.is_none());
			assert_eq!(Pools::member_points(pool_id, 11), Pools::depositor_min_bond());

			// ensure the points and member count are correct
			let pool = get_pool(pool_id).unwrap();
			assert_eq!(pool.points(), Pools::depositor_min_bond() * 2);
		}
	});
}

/// Capacity should work correctly when slashed
#[test]
fn test_join_capacity_slashed() {
	let capacity = 200;

	ExtBuilder::default()
		.set_capacity(capacity)
		.add_members(vec![(40, 40), (50, 50)])
		.build_and_execute(|| {
			let pool_id = 0;
			let bonded_account = default_bonded_account();

			// Simulate a slash to the pool, changing balance from 100 to 50
			let new_amount = 50;
			assert_eq!(StakingMock::active_stake(&bonded_account).unwrap(), 100);
			Balances::make_free_balance_be(&bonded_account, new_amount);
			StakingMock::set_bonded_balance(bonded_account, new_amount);

			// points are now double the balance
			let pool = get_pool(pool_id).unwrap();
			assert_eq!(pool.balance_to_point(1), 2);
			assert_eq!(pool.available_points(), 100);

			// join with 20, granting 40 points
			Balances::make_free_balance_be(&20, 100);
			Pools::bond(RuntimeOrigin::signed(20), pool_id, 20.into()).unwrap();
			let pool = get_pool(pool_id).unwrap();
			assert_eq!(pool.available_points(), 60);
			assert_eq!(Pools::member_points(pool_id, 20), 40);

			// joining with 40 grants 80 points, should exceed capacity
			Balances::make_free_balance_be(&40, 100);
			assert_noop!(
				Pools::bond(RuntimeOrigin::signed(40), pool_id, 40.into()),
				Error::<Runtime>::CapacityExceeded
			);

			// bonding extra causes the same problem
			assert_noop!(
				Pools::bond(RuntimeOrigin::signed(20), pool_id, 40.into()),
				Error::<Runtime>::CapacityExceeded
			);

			// fill the pool
			Balances::make_free_balance_be(&999, 200);
			Pools::bond(RuntimeOrigin::signed(999), pool_id, BondValue::Fill).unwrap();
			assert_eq!(Pools::member_points(pool_id, 999), 60);
			let pool = get_pool(pool_id).unwrap();
			assert_eq!(pool.available_points(), 0);
		})
}

#[test]
fn test_bond_from_balance_creator() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;
		let capacity = 1_000;

		// 10 is the owner, give them enough funds to fill the pool
		// the pool already has 10 units deposited
		let initial_balance = capacity - 10;
		Balances::make_free_balance_be(&10, initial_balance);

		// given
		assert_eq!(Pools::depositor_min_bond(), 10);
		assert_eq!(Pools::member_points(pool_id, Pools::deposit_account_id(pool_id)), 10);
		assert_eq!(get_pool(pool_id).unwrap().points(), 10);
		assert_eq!(Balances::free_balance(10), initial_balance);

		// when
		bond_extra(10, pool_id, 10);

		// then
		assert_eq!(Balances::free_balance(10), initial_balance - 10);
		assert_eq!(Pools::member_points(pool_id, 10), 10);
		assert_eq!(get_pool(pool_id).unwrap().points(), 20);

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { creator: 10, pool_id: 0, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(pool_id), pool_id: 0, bonded: 10 },
				Event::Bonded { member: 10, pool_id: 0, bonded: 10 }
			]
		);

		// when
		bond_extra(10, pool_id, 20);

		// then
		assert_eq!(Balances::free_balance(10), initial_balance - 30);
		assert_eq!(Pools::member_points(pool_id, 10), 30);
		assert_eq!(get_pool(pool_id).unwrap().points(), 40);

		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Bonded { member: 10, pool_id: 0, bonded: 20 }]
		);

		// cannot bond extra over the capacity
		let minimum_balance = Balances::minimum_balance();
		Balances::make_free_balance_be(&10, capacity + minimum_balance);
		assert_noop!(
			Pools::bond(RuntimeOrigin::signed(10), pool_id, capacity.into()),
			Error::<Runtime>::CapacityExceeded
		);

		// fill the remaining capacity
		let remaining_points = get_pool(pool_id).unwrap().available_points();
		let previous_member_points = Pools::member_points(pool_id, 10);
		Balances::make_free_balance_be(&10, remaining_points + minimum_balance);
		Pools::bond(RuntimeOrigin::signed(10), pool_id, BondValue::Fill).unwrap();

		assert_eq!(Balances::free_balance(10), minimum_balance);
		let pool = get_pool(pool_id).unwrap();
		assert_eq!(pool.capacity, capacity);
		assert_eq!(pool.available_points(), 0);
		assert_eq!(Pools::member_points(pool_id, 10), previous_member_points + remaining_points);

		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Bonded { member: 10, pool_id: 0, bonded: remaining_points }]
		);
	})
}

#[test]
fn test_second_time_bond_mints_lst() {
	ExtBuilder::default().set_capacity(10_000 * UNIT).build_and_execute(|| {
		let pool_id = NextPoolId::<Runtime>::get() - 1;

		for i in 1..3 {
			let account = i + 100;
			Balances::make_free_balance_be(&account, 1100 + Balances::minimum_balance());

			assert_ok!(Pools::bond(RuntimeOrigin::signed(account), pool_id, 1000.into()));

			assert_eq!(Pools::member_points(pool_id, account), 1000);
			assert_eq!(
				FungiblesTrait::balance_of(LST_COLLECTION_ID, pool_id as AssetId, account,),
				1000
			);
			assert_eq!(Balances::free_balance(account), 100 + Balances::minimum_balance());

			// assert event is emitted
			assert_event_deposited!(FungiblesEvent::Minted {
				issuer: RootOrSigned::Signed(<Runtime as Config>::LstCollectionOwner::get()),
				recipient: account,
				collection_id: LST_COLLECTION_ID,
				token_id: pool_id as u128,
				amount: 1000,
			});

			bond_extra(account, pool_id, 100);

			assert_eq!(Pools::member_points(pool_id, account), 1100);
			assert_eq!(Balances::free_balance(account), Balances::minimum_balance());

			// assert event is emitted
			assert_event_deposited!(FungiblesEvent::Minted {
				issuer: RootOrSigned::Signed(<Runtime as Config>::LstCollectionOwner::get()),
				recipient: account,
				collection_id: LST_COLLECTION_ID,
				token_id: pool_id as u128,
				amount: 100,
			});
		}
	})
}
