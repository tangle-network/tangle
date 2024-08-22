use super::*;
use crate::traits::Inspect;
use sp_runtime::FixedU128;

#[test]
fn balance_to_point_works() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = 123123;
		let bonded_pool = BondedPool::<Runtime> {
			id: pool_id,
			inner: BondedPoolInner {
				token_id: DEFAULT_TOKEN_ID,
				state: PoolState::Open,
				capacity: 1_000,
				commission: Commission::default(),
			},
		};

		// 1 points : 1 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 100);
		mint_points(pool_id, 100);
		assert_eq!(bonded_pool.balance_to_point(10), 10);
		assert_eq!(bonded_pool.balance_to_point(0), 0);

		// 2 points : 1 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 50);
		assert_eq!(bonded_pool.balance_to_point(10), 20);

		// 1 points : 2 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 100);
		burn_points(pool_id, 50);
		assert_eq!(bonded_pool.balance_to_point(10), 5);

		// 100 points : 0 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 0);
		mint_points(pool_id, 50);
		assert_eq!(bonded_pool.balance_to_point(10), 100 * 10);

		// 0 points : 100 balance
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 100);
		burn_points(pool_id, 100);
		assert_eq!(bonded_pool.balance_to_point(10), 10);

		// 10 points : 3 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 30);
		mint_points(pool_id, 100);
		assert_eq!(bonded_pool.balance_to_point(10), 33);

		// 2 points : 3 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 300);
		mint_points(pool_id, 100);
		assert_eq!(bonded_pool.balance_to_point(10), 6);

		// 4 points : 9 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 900);
		mint_points(pool_id, 200);
		assert_eq!(bonded_pool.balance_to_point(90), 40);
	})
}

#[test]
fn average_reward_rate_calculation_works() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = 123123;
		let bonded_pool = BondedPool::<Runtime> {
			id: pool_id,
			inner: BondedPoolInner {
				token_id: pool_id.into(),
				state: PoolState::Open,
				capacity: 1_000,
				commission: Commission::default(),
			},
		};
		BondedPools::<Runtime>::insert(pool_id, bonded_pool.clone().inner);

		// 1 points : 1 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 100);
		mint_points(pool_id, 100);
		assert_eq!(Pools::average_reward_rate(pool_id.into()), FixedU128::from_rational(100, 100));

		// 2 points : 1 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 50);
		assert_eq!(bonded_pool.balance_to_point(10), 20);
		assert_eq!(Pools::average_reward_rate(pool_id.into()), FixedU128::from_rational(50, 100));

		// 1 points : 2 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 100);
		burn_points(pool_id, 50);
		assert_eq!(Pools::average_reward_rate(pool_id.into()), FixedU128::from_rational(100, 50));

		// 100 points : 0 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 0);
		mint_points(pool_id, 50);
		assert_eq!(Pools::average_reward_rate(pool_id.into()), Default::default()); // reward rate zero

		// 0 points : 100 balance
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 100);
		burn_points(pool_id, 100);
		assert_eq!(Pools::average_reward_rate(pool_id.into()), Default::default()); // reward rate zero

		// 100 points : 103 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 103);
		mint_points(pool_id, 100);
		assert_eq!(Pools::average_reward_rate(pool_id.into()), FixedU128::from_rational(103, 100));

		// 1000 points : 1030 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 1030);
		mint_points(pool_id, 900);
		assert_eq!(
			Pools::average_reward_rate(pool_id.into()),
			FixedU128::from_rational(1030, 1000)
		);
	})
}

#[test]
fn points_to_balance_works() {
	ExtBuilder::default().without_pool().build_and_execute(|| {
		// 1 balance : 1 points ratio
		let pool_id = 123123;
		let bonded_pool = BondedPool::<Runtime> {
			id: pool_id,
			inner: BondedPoolInner {
				token_id: DEFAULT_TOKEN_ID,
				state: PoolState::Open,
				capacity: 1_000,
				commission: Commission::default(),
			},
		};

		mint_points(pool_id, 100);
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 100);
		assert_eq!(bonded_pool.points_to_balance(10), 10);
		assert_eq!(bonded_pool.points_to_balance(0), 0);

		// 2 balance : 1 points ratio
		burn_points(pool_id, 50);
		assert_eq!(bonded_pool.points_to_balance(10), 20);

		// 100 balance : 0 points ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 100);
		burn_points(pool_id, 50);
		assert_eq!(bonded_pool.points_to_balance(10), 0);

		// 0 balance : 100 points ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 0);
		mint_points(pool_id, 100);
		assert_eq!(bonded_pool.points_to_balance(10), 0);

		// 10 balance : 3 points ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 100);
		burn_points(pool_id, 70);
		assert_eq!(bonded_pool.points(), 30);
		assert_eq!(bonded_pool.points_to_balance(10), 33);

		// 2 balance : 3 points ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 200);
		mint_points(pool_id, 270);
		assert_eq!(bonded_pool.points(), 300);
		assert_eq!(bonded_pool.points_to_balance(10), 6);

		// 4 balance : 9 points ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 400);
		mint_points(pool_id, 600);
		assert_eq!(bonded_pool.points_to_balance(90), 40);
	})
}

#[test]
fn ok_to_join_with_works() {
	ExtBuilder::default().without_pool().build_and_execute(|| {
		let pool_id = 123;
		let pool = BondedPool::<Runtime> {
			id: pool_id,
			inner: BondedPoolInner {
				token_id: DEFAULT_TOKEN_ID,
				state: PoolState::Open,
				capacity: 1_000,
				commission: Commission::default(),
			},
		};

		// add points to pool
		mint_points(pool_id, 100);

		let max_points_to_balance: u128 =
			<<Runtime as Config>::MaxPointsToBalance as Get<u8>>::get().into();

		// Simulate a 100% slashed pool
		StakingMock::set_bonded_balance(pool.bonded_account(), 0);
		assert_noop!(pool.ok_to_join(), Error::<Runtime>::OverflowRisk);

		// Simulate a slashed pool at `MaxPointsToBalance` + 1 slashed pool
		StakingMock::set_bonded_balance(
			pool.bonded_account(),
			max_points_to_balance.saturating_add(1),
		);
		assert_ok!(pool.ok_to_join());

		// Simulate a slashed pool at `MaxPointsToBalance`
		StakingMock::set_bonded_balance(pool.bonded_account(), max_points_to_balance);
		assert_noop!(pool.ok_to_join(), Error::<Runtime>::OverflowRisk);

		StakingMock::set_bonded_balance(
			pool.bonded_account(),
			Balance::MAX / max_points_to_balance,
		);

		// and a sanity check
		StakingMock::set_bonded_balance(
			pool.bonded_account(),
			Balance::MAX / max_points_to_balance - 1,
		);
		assert_ok!(pool.ok_to_join());
	});
}
