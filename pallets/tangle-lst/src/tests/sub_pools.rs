use super::*;

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
