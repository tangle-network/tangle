use super::*;
use frame_support::{assert_err, assert_noop, assert_ok};

#[test]
fn create_works() {
	ExtBuilder::default().build_and_execute(|| {
		// next pool id is 2.
		let next_pool_stash = Lst::create_bonded_account(2);
		let ed = <Balances as CurrencyT<AccountId>>::minimum_balance();

		assert_eq!(TotalValueLocked::<T>::get(), 10);
		assert!(!BondedPools::<Runtime>::contains_key(2));
		assert!(!RewardPools::<Runtime>::contains_key(2));
		assert_err!(StakingMock::active_stake(&next_pool_stash), "balance not found");

		Currency::make_free_balance_be(&11, StakingMock::minimum_nominator_bond() * 10 + ed);
		assert_ok!(Lst::create(
			RuntimeOrigin::signed(11),
			StakingMock::minimum_nominator_bond(),
			123,
			456,
			789,
			Default::default(),
			Default::default()
		));

		assert_eq!(TotalValueLocked::<T>::get(), 10 + StakingMock::minimum_nominator_bond());

		assert_eq!(Currency::free_balance(11), 90);

		let bonded_pool = BondedPool::<Runtime>::get(2).unwrap();

		assert_eq!(bonded_pool.id, 2);

		let inner = bonded_pool.inner;
		assert_eq!(inner.commission, Commission::default());

		let roles = inner.roles;
		assert_eq!(roles.depositor, 11);
		assert_eq!(roles.root, Some(123));
		assert_eq!(roles.nominator, Some(456));
		assert_eq!(roles.bouncer, Some(789));

		assert_eq!(inner.state, PoolState::Open);

		assert_eq!(
			StakingMock::active_stake(&next_pool_stash).unwrap(),
			StakingMock::minimum_nominator_bond()
		);
		assert_eq!(RewardPools::<Runtime>::get(2).unwrap(), RewardPool { ..Default::default() });

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
				Event::Created { depositor: 11, pool_id: 2 },
				Event::Bonded { member: 11, pool_id: 2, bonded: 10, joined: true }
			]
		);
	});
}

#[test]
fn create_errors_correctly() {
	ExtBuilder::default().with_check(0).build_and_execute(|| {
		// Given
		assert_eq!(MinCreateBond::<Runtime>::get(), 2);
		assert_eq!(StakingMock::minimum_nominator_bond(), 10);

		// Then
		assert_noop!(
			Lst::create(
				RuntimeOrigin::signed(11),
				9,
				123,
				456,
				789,
				Default::default(),
				Default::default()
			),
			Error::<Runtime>::MinimumBondNotMet
		);

		// Given
		MinCreateBond::<Runtime>::put(20);

		// Then
		assert_noop!(
			Lst::create(
				RuntimeOrigin::signed(11),
				19,
				123,
				456,
				789,
				Default::default(),
				Default::default()
			),
			Error::<Runtime>::MinimumBondNotMet
		);

		// Given
		BondedPool::<Runtime> {
			id: 2,
			inner: BondedPoolInner {
				commission: Commission::default(),
				roles: DEFAULT_ROLES,
				state: PoolState::Open,
				metadata: PoolMetadata { name: Default::default(), icon: Default::default() },
			},
		}
		.put();
		assert_eq!(MaxPools::<Runtime>::get(), Some(2));
		assert_eq!(BondedPools::<Runtime>::count(), 2);

		// Then
		assert_noop!(
			Lst::create(
				RuntimeOrigin::signed(11),
				20,
				123,
				456,
				789,
				Default::default(),
				Default::default()
			),
			Error::<Runtime>::MaxPools
		);
	});
}

#[test]
fn create_with_pool_id_works() {
	ExtBuilder::default().build_and_execute(|| {
		let ed = <Balances as CurrencyT<AccountId>>::minimum_balance();

		Currency::make_free_balance_be(&11, StakingMock::minimum_nominator_bond() * 10 + ed);
		assert_ok!(Lst::create(
			RuntimeOrigin::signed(11),
			StakingMock::minimum_nominator_bond(),
			123,
			456,
			789,
			Default::default(),
			Default::default()
		));

		assert_eq!(Currency::free_balance(11), 90);
		// delete the initial pool created, then pool_Id `1` will be free

		assert_noop!(
			Lst::create_with_pool_id(
				RuntimeOrigin::signed(12),
				20,
				234,
				654,
				783,
				1,
				Default::default(),
				Default::default()
			),
			Error::<Runtime>::PoolIdInUse
		);

		assert_noop!(
			Lst::create_with_pool_id(
				RuntimeOrigin::signed(12),
				20,
				234,
				654,
				783,
				3,
				Default::default(),
				Default::default()
			),
			Error::<Runtime>::InvalidPoolId
		);

		// start dismantling the pool.
		assert_ok!(Lst::set_state(RuntimeOrigin::signed(902), 1, PoolState::Destroying));
	});
}
