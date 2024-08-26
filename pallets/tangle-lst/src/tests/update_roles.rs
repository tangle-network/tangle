use super::*;
use frame_support::assert_err;
use frame_support::assert_noop;
use frame_support::assert_ok;
use frame_support::traits::fungible::InspectFreeze;

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

fn default_pool_reward_counter() -> FixedU128 {
	let bonded_pool = BondedPools::<T>::get(1).unwrap();
	RewardPools::<Runtime>::get(1)
		.unwrap()
		.current_reward_counter(1, bonded_pool.points(), bonded_pool.commission.current())
		.unwrap()
		.0
}

#[test]
fn smallest_claimable_reward() {
	// create a pool that has all of the polkadot issuance in 50 years.
	let pool_bond = inflation(50);
	ExtBuilder::default().ed(DOT).min_bond(pool_bond).build_and_execute(|| {
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded {
					member: 10,
					pool_id: 1,
					bonded: 1173908528796953165005,
					joined: true,
				}
			]
		);

		// the smallest reward that this pool can handle is
		let expected_smallest_reward = inflation(50) / 10u128.pow(18);

		// tad bit less. cannot be paid out.
		deposit_rewards(expected_smallest_reward - 1);
		assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
		assert_eq!(pool_events_since_last_call(), vec![]);
		// revert it.

		remove_rewards(expected_smallest_reward - 1);

		// tad bit more. can be claimed.
		deposit_rewards(expected_smallest_reward + 1);
		assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::PaidOut { member: 10, pool_id: 1, payout: 1173 }]
		);
	})
}

#[test]
fn massive_reward_in_small_pool() {
	let tiny_bond = 1000 * DOT;
	ExtBuilder::default().ed(DOT).min_bond(tiny_bond).build_and_execute(|| {
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { depositor: 10, pool_id: 1 },
				Event::Bonded { member: 10, pool_id: 1, bonded: 10000000000000, joined: true }
			]
		);

		Currency::set_balance(&20, tiny_bond);
		assert_ok!(Lst::join(RuntimeOrigin::signed(20), tiny_bond / 2, 1));

		// Suddenly, add a shit ton of rewards.
		deposit_rewards(inflation(1));

		// now claim.
		assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
		assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Bonded { member: 20, pool_id: 1, bonded: 5000000000000, joined: true },
				Event::PaidOut { member: 10, pool_id: 1, payout: 7333333333333333333 },
				Event::PaidOut { member: 20, pool_id: 1, payout: 3666666666666666666 }
			]
		);
	})
}

#[test]
fn reward_counter_calc_wont_fail_in_normal_polkadot_future() {
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

		// in 10 years, the total claimed rewards are large values as well. assuming that a pool
		// is earning all of the inflation per year (which is really unrealistic, but worse
		// case), that will be:
		let pool_total_earnings_10_years = inflation(10) - POLKADOT_TOTAL_ISSUANCE_GENESIS;
		deposit_rewards(pool_total_earnings_10_years);

		// some whale now joins with the other half ot the total issuance. This will bloat all
		// the calculation regarding current reward counter.
		Currency::set_balance(&20, pool_bond * 2);
		assert_ok!(Lst::join(RuntimeOrigin::signed(20), pool_bond, 1));

		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::Bonded {
				member: 20,
				pool_id: 1,
				bonded: 12_968_712_300_500_000_000,
				joined: true
			}]
		);

		assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
		assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));

		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::PaidOut { member: 10, pool_id: 1, payout: 15937424600999999996 }]
		);

		// now let a small member join with 10 DOTs.
		Currency::set_balance(&30, 20 * DOT);
		assert_ok!(Lst::join(RuntimeOrigin::signed(30), 10 * DOT, 1));

		// and give a reasonably small reward to the pool.
		deposit_rewards(DOT);

		assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(30)));
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Bonded { member: 30, pool_id: 1, bonded: 100000000000, joined: true },
				// quite small, but working fine.
				Event::PaidOut { member: 30, pool_id: 1, payout: 38 }
			]
		);
	})
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

#[test]
fn if_small_member_waits_long_enough_they_will_earn_rewards() {
	// create a pool that has a quarter of the current polkadot issuance
	ExtBuilder::default()
		.ed(DOT)
		.min_bond(POLKADOT_TOTAL_ISSUANCE_GENESIS / 4)
		.build_and_execute(|| {
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { depositor: 10, pool_id: 1 },
					Event::Bonded {
						member: 10,
						pool_id: 1,
						bonded: 2500000000000000000,
						joined: true,
					}
				]
			);

			// and have a tiny fish join the pool as well..
			Currency::set_balance(&20, 20 * DOT);
			assert_ok!(Lst::join(RuntimeOrigin::signed(20), 10 * DOT, 1));

			// earn some small rewards
			deposit_rewards(DOT / 1000);

			// no point in claiming for 20 (nonetheless, it should be harmless)
			assert!(pending_rewards(20).unwrap().is_zero());
			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Bonded { member: 20, pool_id: 1, bonded: 100000000000, joined: true },
					Event::PaidOut { member: 10, pool_id: 1, payout: 9999997 }
				]
			);

			// earn some small more, still nothing can be claimed for 20, but 10 claims their
			// share.
			deposit_rewards(DOT / 1000);
			assert!(pending_rewards(20).unwrap().is_zero());
			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
			assert_eq!(
				pool_events_since_last_call(),
				vec![Event::PaidOut { member: 10, pool_id: 1, payout: 10000000 }]
			);

			// earn some more rewards, this time 20 can also claim.
			deposit_rewards(DOT / 1000);
			assert_eq!(pending_rewards(20).unwrap(), 1);
			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::PaidOut { member: 10, pool_id: 1, payout: 10000000 },
					Event::PaidOut { member: 20, pool_id: 1, payout: 1 }
				]
			);
		});
}

#[test]
fn zero_reward_claim_does_not_update_reward_counter() {
	// create a pool that has a quarter of the current polkadot issuance
	ExtBuilder::default()
		.ed(DOT)
		.min_bond(POLKADOT_TOTAL_ISSUANCE_GENESIS / 4)
		.build_and_execute(|| {
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { depositor: 10, pool_id: 1 },
					Event::Bonded {
						member: 10,
						pool_id: 1,
						bonded: 2500000000000000000,
						joined: true,
					}
				]
			);

			// and have a tiny fish join the pool as well..
			Currency::set_balance(&20, 20 * DOT);
			assert_ok!(Lst::join(RuntimeOrigin::signed(20), 10 * DOT, 1));

			// earn some small rewards
			deposit_rewards(DOT / 1000);

			// if 20 claims now, their reward counter should stay the same, so that they have a
			// chance of claiming this if they let it accumulate. Also see
			// `if_small_member_waits_long_enough_they_will_earn_rewards`
			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(10)));
			assert_ok!(Lst::claim_payout(RuntimeOrigin::signed(20)));
			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Bonded { member: 20, pool_id: 1, bonded: 100000000000, joined: true },
					Event::PaidOut { member: 10, pool_id: 1, payout: 9999997 }
				]
			);

			let current_reward_counter = default_pool_reward_counter();
		});
}
