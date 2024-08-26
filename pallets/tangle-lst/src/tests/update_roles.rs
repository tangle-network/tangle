use super::*;
use frame_support::assert_err;
use frame_support::assert_noop;
use frame_support::assert_ok;

#[test]
fn update_roles_works() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(
			BondedPools::<Runtime>::get(1).unwrap().roles,
			PoolRoles { depositor: 10, root: Some(900), nominator: Some(901), bouncer: Some(902) },
		);

		// non-existent pools
		assert_noop!(
			Lst::update_roles(
				RuntimeOrigin::signed(1),
				2,
				ConfigOp::Set(5),
				ConfigOp::Set(6),
				ConfigOp::Set(7)
			),
			Error::<Runtime>::PoolNotFound,
		);

		// depositor cannot change roles.
		assert_noop!(
			Lst::update_roles(
				RuntimeOrigin::signed(1),
				1,
				ConfigOp::Set(5),
				ConfigOp::Set(6),
				ConfigOp::Set(7)
			),
			Error::<Runtime>::DoesNotHavePermission,
		);

		// nominator cannot change roles.
		assert_noop!(
			Lst::update_roles(
				RuntimeOrigin::signed(901),
				1,
				ConfigOp::Set(5),
				ConfigOp::Set(6),
				ConfigOp::Set(7)
			),
			Error::<Runtime>::DoesNotHavePermission,
		);
		// bouncer
		assert_noop!(
			Lst::update_roles(
				RuntimeOrigin::signed(902),
				1,
				ConfigOp::Set(5),
				ConfigOp::Set(6),
				ConfigOp::Set(7)
			),
			Error::<Runtime>::DoesNotHavePermission,
		);

		// but root can
		assert_ok!(Lst::update_roles(
			RuntimeOrigin::signed(900),
			1,
			ConfigOp::Set(5),
			ConfigOp::Set(6),
			ConfigOp::Set(7)
		));

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
				Event::RolesUpdated { root: Some(5), bouncer: Some(7), nominator: Some(6) }
			]
		);
		assert_eq!(
			BondedPools::<Runtime>::get(1).unwrap().roles,
			PoolRoles { depositor: 10, root: Some(5), nominator: Some(6), bouncer: Some(7) },
		);

		// also root origin can
		assert_ok!(Lst::update_roles(
			RuntimeOrigin::root(),
			1,
			ConfigOp::Set(1),
			ConfigOp::Set(2),
			ConfigOp::Set(3)
		));

		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::RolesUpdated { root: Some(1), bouncer: Some(3), nominator: Some(2) }]
		);
		assert_eq!(
			BondedPools::<Runtime>::get(1).unwrap().roles,
			PoolRoles { depositor: 10, root: Some(1), nominator: Some(2), bouncer: Some(3) },
		);

		// Noop works
		assert_ok!(Lst::update_roles(
			RuntimeOrigin::root(),
			1,
			ConfigOp::Set(11),
			ConfigOp::Noop,
			ConfigOp::Noop
		));

		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::RolesUpdated { root: Some(11), bouncer: Some(3), nominator: Some(2) }]
		);

		assert_eq!(
			BondedPools::<Runtime>::get(1).unwrap().roles,
			PoolRoles { depositor: 10, root: Some(11), nominator: Some(2), bouncer: Some(3) },
		);

		// Remove works
		assert_ok!(Lst::update_roles(
			RuntimeOrigin::root(),
			1,
			ConfigOp::Set(69),
			ConfigOp::Remove,
			ConfigOp::Remove
		));

		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::RolesUpdated { root: Some(69), bouncer: None, nominator: None }]
		);

		assert_eq!(
			BondedPools::<Runtime>::get(1).unwrap().roles,
			PoolRoles { depositor: 10, root: Some(69), nominator: None, bouncer: None },
		);
	})
}

const DOT: Balance = 10u128.pow(10u32);
const POLKADOT_TOTAL_ISSUANCE_GENESIS: Balance = DOT * 10u128.pow(9u32);

const fn inflation(years: u128) -> u128 {
	let mut i = 0;
	let mut start = POLKADOT_TOTAL_ISSUANCE_GENESIS;
	while i < years {
		start = start + start / 10;
		i += 1
	}
	start
}

#[test]
fn reward_counter_update_can_fail_if_pool_is_highly_slashed() {
	// create a pool that has roughly half of the polkadot issuance in 10 years.
	let pool_bond = inflation(10) / 2;
	ExtBuilder::default().ed(DOT).min_bond(pool_bond).build_and_execute(|| {
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded {
					member: 10,
					pool_id: 1,
					bonded: 12_968_712_300_500_000_000,
					joined: true,
				}
			]
		);

		// slash this pool by 99% of that.
		StakingMock::slash_by(1, pool_bond * 99 / 100);

		// some whale now joins with the other half ot the total issuance. This will trigger an
		// overflow. This test is actually a bit too lenient because all the reward counters are
		// set to zero. In other tests that we want to assert a scenario won't fail, we should
		// also set the reward counters to some large value.
		Currency::set_balance(&20, pool_bond * 2);
		assert_err!(Lst::join(RuntimeOrigin::signed(20), pool_bond, 1), Error::<T>::OverflowRisk);
	})
}
