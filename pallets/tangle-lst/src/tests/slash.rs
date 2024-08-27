use super::*;
use frame_support::assert_ok;

#[test]
fn slash_no_subpool_is_tracked() {
	let bonded = |points| BondedPool::<Runtime> {
		id: 1,
		inner: BondedPoolInner {
			commission: Commission::default(),
			roles: DEFAULT_ROLES,
			state: PoolState::Open,
		},
	};
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

		assert_eq!(BondedPool::<Runtime>::get(1).unwrap(), bonded(12, 2));

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
		assert_eq!(BondedPool::<Runtime>::get(1).unwrap(), bonded(12 + 24, 3));
	});
}
