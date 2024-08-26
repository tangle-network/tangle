mod bond_extra {
	use super::*;
	use crate::Event;

	#[test]
	fn bond_extra_from_free_balance_creator() {
		ExtBuilder::default().build_and_execute(|| {
			// 10 is the owner and a member in pool 1, give them some more funds.
			Currency::set_balance(&10, 100);

			// given
			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 10);
			assert_eq!(Currency::free_balance(&10), 100);

			// when
			assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), BondExtra::FreeBalance(10)));

			// then
			assert_eq!(Currency::free_balance(&10), 90);
			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 20);

			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { depositor: 10, pool_id: 1 },
					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: false }
				]
			);

			// when
			assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), BondExtra::FreeBalance(20)));

			// then
			assert_eq!(Currency::free_balance(&10), 70);
			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 40);

			assert_eq!(
				pool_events_since_last_call(),
				vec![Event::Bonded { member: 10, pool_id: 1, bonded: 20, joined: false }]
			);
		})
	}

	#[test]
	fn bond_extra_from_rewards_creator() {
		ExtBuilder::default().build_and_execute(|| {
			// put some money in the reward account, all of which will belong to 10 as the only
			// member of the pool.
			Currency::set_balance(&default_reward_account(), 7);
			// ... if which only 2 is claimable to make sure the reward account does not die.
			let claimable_reward = 7 - ExistentialDeposit::get();

			// given
			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 10);
			assert_eq!(Currency::free_balance(&10), 35);

			// when
			assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), BondExtra::Rewards));

			// then
			assert_eq!(Currency::free_balance(&10), 35);
			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 10 + claimable_reward);

			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { depositor: 10, pool_id: 1 },
					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
					Event::PaidOut { member: 10, pool_id: 1, payout: claimable_reward },
					Event::Bonded {
						member: 10,
						pool_id: 1,
						bonded: claimable_reward,
						joined: false
					}
				]
			);
		})
	}

	#[test]
	fn bond_extra_from_rewards_joiner() {
		ExtBuilder::default().add_members(vec![(20, 20)]).build_and_execute(|| {
			// put some money in the reward account, all of which will belong to 10 as the only
			// member of the pool.
			Currency::set_balance(&default_reward_account(), 8);
			// ... if which only 3 is claimable to make sure the reward account does not die.
			let claimable_reward = 8 - ExistentialDeposit::get();
			// NOTE: easier to read of we use 3, so let's use the number instead of variable.
			assert_eq!(claimable_reward, 3, "test is correct if rewards are divisible by 3");

			// given
			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 30);

			assert_eq!(Currency::free_balance(&10), 35);
			assert_eq!(Currency::free_balance(&20), 20);
			assert_eq!(TotalValueLocked::<T>::get(), 30);

			// when
			assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(10), BondExtra::Rewards));
			assert_eq!(Currency::free_balance(&default_reward_account()), 7);

			// then
			assert_eq!(Currency::free_balance(&10), 35);
			assert_eq!(TotalValueLocked::<T>::get(), 31);

			// 10's share of the reward is 1/3, since they gave 10/30 of the total shares.
			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 30 + 1);

			// when
			assert_ok!(Lst::bond_extra(RuntimeOrigin::signed(20), BondExtra::Rewards));

			// then
			assert_eq!(Currency::free_balance(&20), 20);
			assert_eq!(TotalValueLocked::<T>::get(), 33);

			// 20's share of the rewards is the other 2/3 of the rewards, since they have 20/30 of
			// the shares
			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 30 + 3);

			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { depositor: 10, pool_id: 1 },
					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
					Event::Bonded { member: 20, pool_id: 1, bonded: 20, joined: true },
					Event::PaidOut { member: 10, pool_id: 1, payout: 1 },
					Event::Bonded { member: 10, pool_id: 1, bonded: 1, joined: false },
					Event::PaidOut { member: 20, pool_id: 1, payout: 2 },
					Event::Bonded { member: 20, pool_id: 1, bonded: 2, joined: false }
				]
			);
		})
	}

	#[test]
	fn bond_extra_other() {
		ExtBuilder::default().add_members(vec![(20, 20)]).build_and_execute(|| {
			Currency::set_balance(&default_reward_account(), 8);
			// ... of which only 3 are claimable to make sure the reward account does not die.
			let claimable_reward = 8 - ExistentialDeposit::get();
			// NOTE: easier to read if we use 3, so let's use the number instead of variable.
			assert_eq!(claimable_reward, 3, "test is correct if rewards are divisible by 3");

			// given
			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 30);
			assert_eq!(Currency::free_balance(&10), 35);
			assert_eq!(Currency::free_balance(&20), 20);

			// Permissioned by default
			assert_noop!(
				Lst::bond_extra_other(RuntimeOrigin::signed(80), 20, BondExtra::Rewards),
				Error::<Runtime>::DoesNotHavePermission
			);

			assert_ok!(Lst::set_claim_permission(
				RuntimeOrigin::signed(10),
				ClaimPermission::PermissionlessAll
			));
			assert_ok!(Lst::bond_extra_other(RuntimeOrigin::signed(50), 10, BondExtra::Rewards));
			assert_eq!(Currency::free_balance(&default_reward_account()), 7);

			// then
			assert_eq!(Currency::free_balance(&10), 35);
			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 30 + 1);

			// when
			assert_noop!(
				Lst::bond_extra_other(RuntimeOrigin::signed(40), 40, BondExtra::Rewards),
				Error::<Runtime>::PoolMemberNotFound
			);

			// when
			assert_ok!(Lst::bond_extra_other(
				RuntimeOrigin::signed(20),
				20,
				BondExtra::FreeBalance(10)
			));

			// then
			assert_eq!(Currency::free_balance(&20), 12);
			assert_eq!(Currency::free_balance(&default_reward_account()), 5);
			assert_eq!(BondedPools::<Runtime>::get(1).unwrap().points, 41);
		})
	}
}
