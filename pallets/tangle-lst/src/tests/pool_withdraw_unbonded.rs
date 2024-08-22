use super::*;

#[test]
fn pool_withdraw_unbonded_works() {
	ExtBuilder::default().build_and_execute(|| {
		// Given 10 unbond'ed directly against the pool account
		assert_ok!(StakingMock::unbond(&default_bonded_account(), 5));
		// and the pool account only has 10 balance
		assert_eq!(StakingMock::active_stake(&default_bonded_account()), Ok(5));
		assert_eq!(StakingMock::total_stake(&default_bonded_account()), Ok(10));
		assert_eq!(Balances::free_balance(default_bonded_account()), 10);

		// When
		assert_ok!(Pools::pool_withdraw_unbonded(RuntimeOrigin::signed(10), 0, 0));

		// Then there unbonding balance is no longer locked
		assert_eq!(StakingMock::active_stake(&default_bonded_account()), Ok(5));
		assert_eq!(StakingMock::total_stake(&default_bonded_account()), Ok(5));
		assert_eq!(Balances::free_balance(default_bonded_account()), 10);
	});
}

#[test]
fn test_pool_withdraw_unbonded_when_pool_destroying() {
	ExtBuilder::default()
		.min_join_bond(10)
		.add_members(vec![(20, 20)])
		.build_and_execute(|| {
			let pool_id = NextPoolId::<Runtime>::get() - 1;

			// user unbonds the entire balance
			assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 20));
			assert_eq!(Pools::member_points(pool_id, 20), 0);
			assert_eq!(
				UnbondingMembers::<Runtime>::get(pool_id, 20).unwrap().unbonding_points(),
				20
			);

			// call the destroy extrinsic
			assert_ok!(Pools::destroy(RuntimeOrigin::signed(10), pool_id));
			assert_last_event!(Event::StateChanged { pool_id, new_state: PoolState::Destroying });

			// should not be able to unbond deposit since a member is still unbonding
			assert_noop!(
				Pools::unbond_deposit(RuntimeOrigin::signed(10), pool_id),
				Error::<T>::PoolMembersRemaining
			);

			// sanity check should not be able to unbond either since a member is still unbonding
			let deposit_account = Pools::deposit_account_id(pool_id);
			assert_noop!(
				Pools::unbond(RuntimeOrigin::signed(10), pool_id, deposit_account, 20),
				Error::<T>::PartialUnbondNotAllowedPermissionlessly
			);

			// the user leaves after unbonding period
			CurrentEra::set(3);
			assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(20), pool_id, 20, 20));

			// the pool creator can now unbond
			assert_ok!(Pools::unbond_deposit(RuntimeOrigin::signed(10), pool_id));
			CurrentEra::set(6);
			assert_ok!(Pools::withdraw_deposit(RuntimeOrigin::signed(10), pool_id));

			// check events
			assert_event_deposited!(Event::Unbonded {
				member: Pallet::<Runtime>::deposit_account_id(pool_id),
				pool_id,
				balance: 10,
				points: 10,
				era: 6
			});
		})
}
