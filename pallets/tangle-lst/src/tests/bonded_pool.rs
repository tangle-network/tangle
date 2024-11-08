use super::*;
use crate::mock::Currency;
use frame_support::traits::Currency as CurrencyT;
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_setup_works() {
	ExtBuilder::default().build_and_execute(|| {
		// Check counts
		assert_eq!(BondedPools::<Runtime>::count(), 1);
		assert_eq!(RewardPools::<Runtime>::count(), 1);
		assert_eq!(SubPoolsStorage::<Runtime>::count(), 0);
		assert_eq!(UnbondingMembers::<Runtime>::count(), 0);

		// Check staking duration
		assert_eq!(StakingMock::bonding_duration(), 3);

		// Check metadata
		assert!(Metadata::<T>::contains_key(1));

		// Check total value locked
		assert_eq!(TotalValueLocked::<T>::get(), 10);

		let last_pool = LastPoolId::<Runtime>::get();

		// Check BondedPool
		let bonded_pool = BondedPool::<Runtime>::get(last_pool).unwrap();
		assert_eq!(bonded_pool.id, last_pool);
		assert_eq!(bonded_pool.inner.commission, Commission::default());
		assert_eq!(bonded_pool.inner.roles, DEFAULT_ROLES);
		assert_eq!(bonded_pool.inner.state, PoolState::Open);

		// Check RewardPool
		let reward_pool = RewardPools::<Runtime>::get(last_pool).unwrap();
		assert_eq!(reward_pool.last_recorded_reward_counter, Zero::zero());
		assert_eq!(reward_pool.last_recorded_total_payouts, 0);
		assert_eq!(reward_pool.total_rewards_claimed, 0);
		assert_eq!(reward_pool.total_commission_claimed, 0);
		assert_eq!(reward_pool.total_commission_pending, 0);

		let bonded_account = Lst::create_bonded_account(last_pool);
		let reward_account = Lst::create_reward_account(last_pool);

		// Check bonded account stake
		assert_eq!(StakingMock::active_stake(&bonded_account).unwrap(), 10);
		assert_eq!(StakingMock::total_stake(&bonded_account).unwrap(), 10);

		// Check nominations
		assert!(Nominations::get().is_none());

		// Check reward account balance
		assert_eq!(
			Currency::free_balance(reward_account),
			<Balances as CurrencyT<AccountId>>::minimum_balance()
		);
	})
}

#[test]
fn balance_to_point_works() {
	ExtBuilder::default().build_and_execute(|| {
		let bonded_pool = BondedPool::<Runtime> {
			id: 123123,
			inner: BondedPoolInner {
				commission: Commission::default(),
				roles: DEFAULT_ROLES,
				state: PoolState::Open,
				metadata: PoolMetadata { name: Default::default(), icon: Default::default() },
			},
		};

		// 1 points : 1 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 100);
		mint_lst(bonded_pool.id, &bonded_pool.bonded_account(), 100);
		assert_eq!(bonded_pool.balance_to_point(10), 10);
		assert_eq!(bonded_pool.balance_to_point(0), 0);

		// 2 points : 1 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 50);
		assert_eq!(bonded_pool.balance_to_point(10), 20);

		// 1 points : 2 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 100);
		burn_lst(bonded_pool.id, &bonded_pool.bonded_account(), 50);
		assert_eq!(bonded_pool.balance_to_point(10), 5);

		// 100 points : 0 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 0);
		mint_lst(bonded_pool.id, &bonded_pool.bonded_account(), 50);
		assert_eq!(bonded_pool.balance_to_point(10), 100 * 10);

		// 0 points : 100 balance
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 100);
		burn_lst(bonded_pool.id, &bonded_pool.bonded_account(), 100);
		assert_eq!(bonded_pool.balance_to_point(10), 10);

		// 10 points : 3 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 30);
		mint_lst(bonded_pool.id, &bonded_pool.bonded_account(), 100);
		assert_eq!(bonded_pool.balance_to_point(10), 33);

		// 2 points : 3 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 300);
		mint_lst(bonded_pool.id, &bonded_pool.bonded_account(), 100);
		assert_eq!(bonded_pool.balance_to_point(10), 6);

		// 4 points : 9 balance ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 900);
		mint_lst(bonded_pool.id, &bonded_pool.bonded_account(), 200);
		assert_eq!(bonded_pool.balance_to_point(90), 40);
	})
}

#[test]
fn points_to_balance_works() {
	ExtBuilder::default().build_and_execute(|| {
		// 1 balance : 1 points ratio
		let bonded_pool = BondedPool::<Runtime> {
			id: 123123,
			inner: BondedPoolInner {
				commission: Commission::default(),
				roles: DEFAULT_ROLES,
				state: PoolState::Open,
				metadata: PoolMetadata { name: Default::default(), icon: Default::default() },
			},
		};

		mint_lst(bonded_pool.id, &bonded_pool.bonded_account(), 100);
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 100);
		assert_eq!(bonded_pool.points_to_balance(10), 10);
		assert_eq!(bonded_pool.points_to_balance(0), 0);

		// 2 balance : 1 points ratio
		burn_lst(bonded_pool.id, &bonded_pool.bonded_account(), 50);
		assert_eq!(bonded_pool.points_to_balance(10), 20);

		// 100 balance : 0 points ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 100);
		burn_lst(bonded_pool.id, &bonded_pool.bonded_account(), 50);
		assert_eq!(bonded_pool.points_to_balance(10), 0);

		// 0 balance : 100 points ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 0);
		mint_lst(bonded_pool.id, &bonded_pool.bonded_account(), 100);
		assert_eq!(bonded_pool.points_to_balance(10), 0);

		// 10 balance : 3 points ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 100);
		burn_lst(bonded_pool.id, &bonded_pool.bonded_account(), 70);
		assert_eq!(bonded_pool.points_to_balance(10), 33);

		// 2 balance : 3 points ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 200);
		mint_lst(bonded_pool.id, &bonded_pool.bonded_account(), 270);
		assert_eq!(bonded_pool.points_to_balance(10), 6);

		// 4 balance : 9 points ratio
		StakingMock::set_bonded_balance(bonded_pool.bonded_account(), 400);
		mint_lst(bonded_pool.id, &bonded_pool.bonded_account(), 600);
		assert_eq!(bonded_pool.points_to_balance(90), 40);
	})
}

#[test]
fn ok_to_join_with_works() {
	ExtBuilder::default().build_and_execute(|| {
		let pool = BondedPool::<Runtime> {
			id: 123,
			inner: BondedPoolInner {
				commission: Commission::default(),
				roles: DEFAULT_ROLES,
				state: PoolState::Open,
				metadata: PoolMetadata { name: Default::default(), icon: Default::default() },
			},
		};

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
