mod create {
	use super::*;
	use frame_support::traits::fungible::InspectFreeze;

	#[test]
	fn create_works() {
		ExtBuilder::default().build_and_execute(|| {
			// next pool id is 2.
			let next_pool_stash = Lst::create_bonded_account(2);
			let ed = Currency::minimum_balance();

			assert_eq!(TotalValueLocked::<T>::get(), 10);
			assert!(!BondedPools::<Runtime>::contains_key(2));
			assert!(!RewardPools::<Runtime>::contains_key(2));
			assert!(!PoolMembers::<Runtime>::contains_key(11));
			assert_err!(StakingMock::active_stake(&next_pool_stash), "balance not found");

			Currency::set_balance(&11, StakingMock::minimum_nominator_bond() + ed);
			assert_ok!(Lst::create(
				RuntimeOrigin::signed(11),
				StakingMock::minimum_nominator_bond(),
				123,
				456,
				789
			));
			assert_eq!(TotalValueLocked::<T>::get(), 10 + StakingMock::minimum_nominator_bond());

			assert_eq!(Currency::free_balance(&11), 0);
			assert_eq!(
				PoolMembers::<Runtime>::get(11).unwrap(),
				PoolMember {
					pool_id: 2,
					points: StakingMock::minimum_nominator_bond(),
					..Default::default()
				}
			);
			assert_eq!(
				BondedPool::<Runtime>::get(2).unwrap(),
				BondedPool {
					id: 2,
					inner: BondedPoolInner {
						commission: Commission::default(),
						points: StakingMock::minimum_nominator_bond(),
						member_counter: 1,
						roles: PoolRoles {
							depositor: 11,
							root: Some(123),
							nominator: Some(456),
							bouncer: Some(789)
						},
						state: PoolState::Open,
					}
				}
			);
			assert_eq!(
				StakingMock::active_stake(&next_pool_stash).unwrap(),
				StakingMock::minimum_nominator_bond()
			);
			assert_eq!(
				RewardPools::<Runtime>::get(2).unwrap(),
				RewardPool { ..Default::default() }
			);

			// make sure ED is frozen on pool creation.
			assert_eq!(
				Currency::balance_frozen(
					&FreezeReason::PoolMinBalance.into(),
					&default_reward_account()
				),
				Currency::minimum_balance()
			);

			assert_eq!(
				pool_events_since_last_call(),
				vec![
					Event::Created { depositor: 10, pool_id: 1 },
					Event::Bonded { member: 10, pool_id: 1, bonded: 10, joined: true },
					Event::Created { depositor: 11, pool_id: 2 },
					Event::Bonded { member: 11, pool_id: 2, bonded: 10, joined: true }
				]
			);
		});
	}

	#[test]
	fn create_errors_correctly() {
		ExtBuilder::default().with_check(0).build_and_execute(|| {
			assert_noop!(
				Lst::create(RuntimeOrigin::signed(10), 420, 123, 456, 789),
				Error::<Runtime>::AccountBelongsToOtherPool
			);

			// Given
			assert_eq!(MinCreateBond::<Runtime>::get(), 2);
			assert_eq!(StakingMock::minimum_nominator_bond(), 10);

			// Then
			assert_noop!(
				Lst::create(RuntimeOrigin::signed(11), 9, 123, 456, 789),
				Error::<Runtime>::MinimumBondNotMet
			);

			// Given
			MinCreateBond::<Runtime>::put(20);

			// Then
			assert_noop!(
				Lst::create(RuntimeOrigin::signed(11), 19, 123, 456, 789),
				Error::<Runtime>::MinimumBondNotMet
			);

			// Given
			BondedPool::<Runtime> {
				id: 2,
				inner: BondedPoolInner {
					commission: Commission::default(),
					member_counter: 1,
					points: 10,
					roles: DEFAULT_ROLES,
					state: PoolState::Open,
				},
			}
			.put();
			assert_eq!(MaxPools::<Runtime>::get(), Some(2));
			assert_eq!(BondedPools::<Runtime>::count(), 2);

			// Then
			assert_noop!(
				Lst::create(RuntimeOrigin::signed(11), 20, 123, 456, 789),
				Error::<Runtime>::MaxPools
			);

			// Given
			assert_eq!(PoolMembers::<Runtime>::count(), 1);
			MaxPools::<Runtime>::put(3);
			MaxPoolMembers::<Runtime>::put(1);
			Currency::set_balance(&11, 5 + 20);

			// Then
			let create = RuntimeCall::Lst(Call::<Runtime>::create {
				amount: 20,
				root: 11,
				nominator: 11,
				bouncer: 11,
			});
			assert_noop!(
				create.dispatch(RuntimeOrigin::signed(11)),
				Error::<Runtime>::MaxPoolMembers
			);
		});
	}

	#[test]
	fn create_with_pool_id_works() {
		ExtBuilder::default().build_and_execute(|| {
			let ed = Currency::minimum_balance();

			Currency::set_balance(&11, StakingMock::minimum_nominator_bond() + ed);
			assert_ok!(Lst::create(
				RuntimeOrigin::signed(11),
				StakingMock::minimum_nominator_bond(),
				123,
				456,
				789
			));

			assert_eq!(Currency::free_balance(&11), 0);
			// delete the initial pool created, then pool_Id `1` will be free

			assert_noop!(
				Lst::create_with_pool_id(RuntimeOrigin::signed(12), 20, 234, 654, 783, 1),
				Error::<Runtime>::PoolIdInUse
			);

			assert_noop!(
				Lst::create_with_pool_id(RuntimeOrigin::signed(12), 20, 234, 654, 783, 3),
				Error::<Runtime>::InvalidPoolId
			);

			// start dismantling the pool.
			assert_ok!(Lst::set_state(RuntimeOrigin::signed(902), 1, PoolState::Destroying));
			assert_ok!(fully_unbond_permissioned(10));

			CurrentEra::set(3);
			assert_ok!(Lst::withdraw_unbonded(RuntimeOrigin::signed(10), 10, 10));

			assert_ok!(Lst::create_with_pool_id(RuntimeOrigin::signed(10), 20, 234, 654, 783, 1));
		});
	}
}

#[test]
fn set_claimable_actor_works() {
	ExtBuilder::default().build_and_execute(|| {
		// Given
		Currency::set_balance(&11, ExistentialDeposit::get() + 2);
		assert!(!PoolMembers::<Runtime>::contains_key(11));

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

		// Make permissionless
		assert_eq!(ClaimPermissions::<Runtime>::get(11), ClaimPermission::Permissioned);
		assert_noop!(
			Lst::set_claim_permission(
				RuntimeOrigin::signed(12),
				ClaimPermission::PermissionlessAll
			),
			Error::<T>::PoolMemberNotFound
		);
		assert_ok!(Lst::set_claim_permission(
			RuntimeOrigin::signed(11),
			ClaimPermission::PermissionlessAll
		));

		// then
		assert_eq!(ClaimPermissions::<Runtime>::get(11), ClaimPermission::PermissionlessAll);
	});
}
