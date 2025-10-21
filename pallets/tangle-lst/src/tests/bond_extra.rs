use super::*;
use crate::Event;
use frame_support::assert_ok;

#[test]
fn bond_extra_from_free_balance_creator() {
	ExtBuilder::default().build_and_execute(|| {
		// 10 is the owner and a member in pool 1, give them some more funds.
		Currency::make_free_balance_be(&10, 100);

		// given
		assert_eq!(Currency::free_balance(10), 100);

		// when
		assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), 1, BondExtra::FreeBalance(10)));

		// then
		assert_eq!(Currency::free_balance(10), 90);

		assert_eq!(pool_events_since_last_call(), vec![
			Event::Created { depositor: 10, pool_id: 1 },
			Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
			Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: false }
		]);

		// when
		assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), 1, BondExtra::FreeBalance(20)));

		// then
		assert_eq!(Currency::free_balance(10), 70);

		assert_eq!(pool_events_since_last_call(), vec![Event::Bonded {
			member: 10,
			pool_id: 1,
			bonded: 20,
			joined: false
		}]);
	})
}
