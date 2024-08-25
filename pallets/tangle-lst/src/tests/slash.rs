use super::*;

#[test]
fn slash_no_subpool_is_tracked() {
	let bonded = |points, member_counter| BondedPool::<Runtime> {
		id: 1,
		inner: BondedPoolInner {
			commission: Commission::default(),
			member_counter,
			points,
			roles: DEFAULT_ROLES,
			state: PoolState::Open,
		},
	};
	ExtBuilder::default().with_check(0).build_and_execute(|| {
		// Given
		Currency::set_balance(&11, ExistentialDeposit::get() + 2);
		assert!(!PoolMembers::<Runtime>::contains_key(11));
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

		assert_eq!(
			PoolMembers::<Runtime>::get(11).unwrap(),
			PoolMember::<Runtime> { pool_id: 1, points: 2, ..Default::default() }
		);
		assert_eq!(BondedPool::<Runtime>::get(1).unwrap(), bonded(12, 2));

		// Given
		// The bonded balance is slashed in half
		StakingMock::slash_by(1, 6);

		// And
		Currency::set_balance(&12, ExistentialDeposit::get() + 12);
		assert!(!PoolMembers::<Runtime>::contains_key(12));

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

		assert_eq!(
			PoolMembers::<Runtime>::get(12).unwrap(),
			PoolMember::<Runtime> { pool_id: 1, points: 24, ..Default::default() }
		);
		assert_eq!(BondedPool::<Runtime>::get(1).unwrap(), bonded(12 + 24, 3));
	});
}
