use super::*;
use crate::{mock::Currency, mock::*, Event};
use frame_support::traits::Currency as CurrencyT;
use frame_support::{assert_err, assert_noop, assert_ok, assert_storage_noop};

macro_rules! unbonding_pools_with_era {
	($($k:expr => $v:expr),* $(,)?) => {{
		use sp_std::iter::{Iterator, IntoIterator};
		let not_bounded: BTreeMap<_, _> = Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]));
		BoundedBTreeMap::<EraIndex, UnbondPool<T>, TotalUnbondingPools<T>>::try_from(not_bounded).unwrap()
	}};
}

#[test]
fn points_to_issue_works() {
	ExtBuilder::default().build_and_execute(|| {
		// 1 points : 1 balance ratio
		let unbond_pool = UnbondPool::<Runtime> { points: 100, balance: 100 };
		assert_eq!(unbond_pool.balance_to_point(10), 10);
		assert_eq!(unbond_pool.balance_to_point(0), 0);

		// 2 points : 1 balance ratio
		let unbond_pool = UnbondPool::<Runtime> { points: 100, balance: 50 };
		assert_eq!(unbond_pool.balance_to_point(10), 20);

		// 1 points : 2 balance ratio
		let unbond_pool = UnbondPool::<Runtime> { points: 50, balance: 100 };
		assert_eq!(unbond_pool.balance_to_point(10), 5);

		// 100 points : 0 balance ratio
		let unbond_pool = UnbondPool::<Runtime> { points: 100, balance: 0 };
		assert_eq!(unbond_pool.balance_to_point(10), 100 * 10);

		// 0 points : 100 balance
		let unbond_pool = UnbondPool::<Runtime> { points: 0, balance: 100 };
		assert_eq!(unbond_pool.balance_to_point(10), 10);

		// 10 points : 3 balance ratio
		let unbond_pool = UnbondPool::<Runtime> { points: 100, balance: 30 };
		assert_eq!(unbond_pool.balance_to_point(10), 33);

		// 2 points : 3 balance ratio
		let unbond_pool = UnbondPool::<Runtime> { points: 200, balance: 300 };
		assert_eq!(unbond_pool.balance_to_point(10), 6);

		// 4 points : 9 balance ratio
		let unbond_pool = UnbondPool::<Runtime> { points: 400, balance: 900 };
		assert_eq!(unbond_pool.balance_to_point(90), 40);
	})
}

#[test]
fn balance_to_unbond_works() {
	// 1 balance : 1 points ratio
	let unbond_pool = UnbondPool::<Runtime> { points: 100, balance: 100 };
	assert_eq!(unbond_pool.point_to_balance(10), 10);
	assert_eq!(unbond_pool.point_to_balance(0), 0);

	// 1 balance : 2 points ratio
	let unbond_pool = UnbondPool::<Runtime> { points: 100, balance: 50 };
	assert_eq!(unbond_pool.point_to_balance(10), 5);

	// 2 balance : 1 points ratio
	let unbond_pool = UnbondPool::<Runtime> { points: 50, balance: 100 };
	assert_eq!(unbond_pool.point_to_balance(10), 20);

	// 100 balance : 0 points ratio
	let unbond_pool = UnbondPool::<Runtime> { points: 0, balance: 100 };
	assert_eq!(unbond_pool.point_to_balance(10), 0);

	// 0 balance : 100 points ratio
	let unbond_pool = UnbondPool::<Runtime> { points: 100, balance: 0 };
	assert_eq!(unbond_pool.point_to_balance(10), 0);

	// 10 balance : 3 points ratio
	let unbond_pool = UnbondPool::<Runtime> { points: 30, balance: 100 };
	assert_eq!(unbond_pool.point_to_balance(10), 33);

	// 2 balance : 3 points ratio
	let unbond_pool = UnbondPool::<Runtime> { points: 300, balance: 200 };
	assert_eq!(unbond_pool.point_to_balance(10), 6);

	// 4 balance : 9 points ratio
	let unbond_pool = UnbondPool::<Runtime> { points: 900, balance: 400 };
	assert_eq!(unbond_pool.point_to_balance(90), 40);
}

#[test]
fn maybe_merge_pools_works() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(TotalUnbondingPools::<Runtime>::get(), 5);
		assert_eq!(BondingDuration::get(), 3);
		assert_eq!(PostUnbondingPoolsWindow::get(), 2);

		// Given
		let mut sub_pool_0 = SubPools::<Runtime> {
			no_era: UnbondPool::<Runtime>::default(),
			with_era: unbonding_pools_with_era! {
				0 => UnbondPool::<Runtime> { points: 10, balance: 10 },
				1 => UnbondPool::<Runtime> { points: 10, balance: 10 },
				2 => UnbondPool::<Runtime> { points: 20, balance: 20 },
				3 => UnbondPool::<Runtime> { points: 30, balance: 30 },
				4 => UnbondPool::<Runtime> { points: 40, balance: 40 },
			},
		};

		// When `current_era < TotalUnbondingPools`,
		let sub_pool_1 = sub_pool_0.clone().maybe_merge_pools(0);

		// Then it exits early without modifications
		assert_eq!(sub_pool_1, sub_pool_0);

		// When `current_era == TotalUnbondingPools`,
		let sub_pool_1 = sub_pool_1.maybe_merge_pools(1);

		// Then it exits early without modifications
		assert_eq!(sub_pool_1, sub_pool_0);

		// When  `current_era - TotalUnbondingPools == 0`,
		let mut sub_pool_1 = sub_pool_1.maybe_merge_pools(2);

		// Then era 0 is merged into the `no_era` pool
		sub_pool_0.no_era = sub_pool_0.with_era.remove(&0).unwrap();
		assert_eq!(sub_pool_1, sub_pool_0);

		// Given we have entries for era 1..=5
		sub_pool_1
			.with_era
			.try_insert(5, UnbondPool::<Runtime> { points: 50, balance: 50 })
			.unwrap();
		sub_pool_0
			.with_era
			.try_insert(5, UnbondPool::<Runtime> { points: 50, balance: 50 })
			.unwrap();

		// When `current_era - TotalUnbondingPools == 1`
		let sub_pool_2 = sub_pool_1.maybe_merge_pools(3);
		let era_1_pool = sub_pool_0.with_era.remove(&1).unwrap();

		// Then era 1 is merged into the `no_era` pool
		sub_pool_0.no_era.points += era_1_pool.points;
		sub_pool_0.no_era.balance += era_1_pool.balance;
		assert_eq!(sub_pool_2, sub_pool_0);

		// When `current_era - TotalUnbondingPools == 5`, so all pools with era <= 4 are removed
		let sub_pool_3 = sub_pool_2.maybe_merge_pools(7);

		// Then all eras <= 5 are merged into the `no_era` pool
		for era in 2..=5 {
			let to_merge = sub_pool_0.with_era.remove(&era).unwrap();
			sub_pool_0.no_era.points += to_merge.points;
			sub_pool_0.no_era.balance += to_merge.balance;
		}
		assert_eq!(sub_pool_3, sub_pool_0);
	});
}
