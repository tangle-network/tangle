use super::*;

#[test]
fn nominate_works() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = 0;
		let _pool = BondedPool::<Runtime>::get(pool_id).unwrap();

		// Root can nominate
		assert_ok!(Pools::nominate(RuntimeOrigin::signed(DEFAULT_MANAGER), pool_id, vec![21]));
		assert_eq!(*Nominations::get().unwrap(), vec![21]);
		assert_event_deposited!(Event::Nominated { pool_id, validators: vec![21] });

		// Nominator can nominate
		assert_ok!(Pools::nominate(RuntimeOrigin::signed(DEFAULT_MANAGER), pool_id, vec![31]));
		assert_eq!(*Nominations::get().unwrap(), vec![31]);
		assert_event_deposited!(Event::Nominated { pool_id, validators: vec![31] });

		// Can't nominate for a pool that doesn't exist
		assert_noop!(
			Pools::nominate(RuntimeOrigin::signed(DEFAULT_MANAGER), 123, vec![21]),
			Error::<Runtime>::PoolNotFound
		);
	});
}
