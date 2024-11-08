use super::*;
use crate::{mock::Currency, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn join_works() {
	ExtBuilder::default().with_check(0).build_and_execute(|| {
		// Given
		Currency::make_free_balance_be(&11, ExistentialDeposit::get() + 2);
		assert_eq!(TotalValueLocked::<T>::get(), 10);

		// When
		assert_ok!(Lst::join(RuntimeOrigin::signed(11), 2, 1));

		// Then
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
				Event::Bonded { member: 11, pool_id: 1, bonded: 2, joined: true },
			]
		);
		assert_eq!(TotalValueLocked::<T>::get(), 12);

		assert_eq!(Assets::balance(1, 11), 2);

		//assert_eq!(BondedPool::<Runtime>::get(1).unwrap(), bonded(12));

		// Given
		// The bonded balance is slashed in half
		StakingMock::slash_by(1, 6);

		// And
		Currency::make_free_balance_be(&12, ExistentialDeposit::get() + 12);

		// When
		assert_ok!(Lst::join(RuntimeOrigin::signed(12), 12, 1));

		// Then
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::PoolSlashed { pool_id: 1, balance: 6 },
				Event::Bonded { member: 12, pool_id: 1, bonded: 12, joined: true }
			]
		);
		assert_eq!(TotalValueLocked::<T>::get(), 18);

		//assert_eq!(BondedPool::<Runtime>::get(1).unwrap(), bonded(12 + 24));
	});
}

#[test]
fn join_errors_correctly() {
	ExtBuilder::default().with_check(0).build_and_execute(|| {
		assert_noop!(
			Lst::join(RuntimeOrigin::signed(11), 420, 123),
			Error::<Runtime>::PoolNotFound
		);

		// Force the pools bonded balance to 0, simulating a 100% slash
		StakingMock::set_bonded_balance(Lst::create_bonded_account(1), 0);
		assert_noop!(Lst::join(RuntimeOrigin::signed(11), 420, 1), Error::<Runtime>::OverflowRisk);

		// Given a mocked bonded pool
		BondedPool::<Runtime> {
			id: 123,
			inner: BondedPoolInner {
				commission: Commission::default(),
				roles: DEFAULT_ROLES,
				state: PoolState::Open,
				metadata: PoolMetadata { name: Default::default(), icon: Default::default() },
			},
		}
		.put();

		// and reward pool
		RewardPools::<Runtime>::insert(123, RewardPool::<Runtime> { ..Default::default() });

		// Force the points:balance ratio to `MaxPointsToBalance` (100/10)
		let max_points_to_balance: u128 =
			<<Runtime as Config>::MaxPointsToBalance as Get<u8>>::get().into();

		StakingMock::set_bonded_balance(Lst::create_bonded_account(123), max_points_to_balance);
		assert_noop!(
			Lst::join(RuntimeOrigin::signed(11), 420, 123),
			Error::<Runtime>::OverflowRisk
		);

		StakingMock::set_bonded_balance(
			Lst::create_bonded_account(123),
			Balance::MAX / max_points_to_balance,
		);

		// // Balance needs to be gt Balance::MAX / `MaxPointsToBalance`
		// assert_noop!(
		// 	Lst::join(RuntimeOrigin::signed(11), 5, 123),
		// 	TokenError::FundsUnavailable,
		// );

		StakingMock::set_bonded_balance(Lst::create_bonded_account(1), max_points_to_balance);

		// Cannot join a pool that isn't open
		unsafe_set_state(123, PoolState::Blocked);
		assert_noop!(
			Lst::join(RuntimeOrigin::signed(11), max_points_to_balance, 123),
			Error::<Runtime>::NotOpen
		);

		unsafe_set_state(123, PoolState::Destroying);
		assert_noop!(
			Lst::join(RuntimeOrigin::signed(11), max_points_to_balance, 123),
			Error::<Runtime>::NotOpen
		);

		// Given
		MinJoinBond::<Runtime>::put(100);

		// Then
		assert_noop!(
			Lst::join(RuntimeOrigin::signed(11), 99, 123),
			Error::<Runtime>::MinimumBondNotMet
		);
	});
}

#[test]
#[cfg_attr(debug_assertions, should_panic(expected = "Defensive failure has been triggered!"))]
fn join_panics_when_reward_pool_not_found() {
	ExtBuilder::default().build_and_execute(|| {
		StakingMock::set_bonded_balance(Lst::create_bonded_account(123), 100);
		BondedPool::<Runtime> {
			id: 123,
			inner: BondedPoolInner {
				commission: Commission::default(),
				roles: DEFAULT_ROLES,
				state: PoolState::Open,
				metadata: PoolMetadata { name: Default::default(), icon: Default::default() },
			},
		}
		.put();
		let _ = Lst::join(RuntimeOrigin::signed(11), 420, 123);
	});
}
