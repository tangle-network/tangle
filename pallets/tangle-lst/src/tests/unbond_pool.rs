use super::*;

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
