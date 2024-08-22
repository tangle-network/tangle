use super::*;

// Advances through blocks and increases era every block
pub fn run_to_era(era: EraIndex) {
	while CurrentEra::get() < era {
		let block_number = System::block_number();

		// increase era and advance to next block
		CurrentEra::mutate(|era| {
			*era += 1;
		});
		System::set_block_number(block_number + 1);
	}
}

#[test]
fn mutate_duration_works() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = 0;
		let token_owner = 10;
		let admin = 900;
		let nominator = 901;

		let mutation = |duration: Option<EraIndex>| PoolMutationOf::<Runtime> {
			duration,
			..Default::default()
		};

		// fails because pool does not exist
		assert_noop!(
			Pools::mutate(RuntimeOrigin::signed(admin), pool_id + 1, mutation(Some(50))),
			Error::<Runtime>::PoolNotFound
		);

		// fails if mutation is a no-op
		assert_noop!(
			Pools::mutate(RuntimeOrigin::signed(admin), pool_id, Default::default()),
			Error::<Runtime>::NoopMutation
		);

		// fails because caller (nominator) is neither admin nor token_owner
		assert_noop!(
			Pools::mutate(RuntimeOrigin::signed(nominator), pool_id, mutation(Some(50))),
			Error::<Runtime>::DoesNotHavePermission,
		);

		// fails because duration is above maximum
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(token_owner),
				pool_id,
				mutation(Some(parameters::nomination_pools::MAX_POOL_DURATION + 1)),
			),
			Error::<Runtime>::DurationOutOfBounds,
		);

		// fails because duration is below minimum
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(token_owner),
				pool_id,
				mutation(Some(parameters::nomination_pools::MIN_POOL_DURATION - 1)),
			),
			Error::<Runtime>::DurationOutOfBounds,
		);

		// token_owner can set duration to minimum
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(token_owner),
			pool_id,
			mutation(Some(parameters::nomination_pools::MIN_POOL_DURATION)),
		));
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(token_owner),
			pool_id,
			mutation(Some(parameters::nomination_pools::MIN_POOL_DURATION)),
		));

		// duration is set as pending update for next cycle
		let pool = BondedPool::<Runtime>::get(pool_id).unwrap();
		assert_eq!(
			pool.bonus_cycle.pending_duration,
			Some(parameters::nomination_pools::MIN_POOL_DURATION)
		);

		// event is emitted
		assert_last_event!(Event::<Runtime>::PoolMutated {
			pool_id,
			mutation: mutation(Some(parameters::nomination_pools::MIN_POOL_DURATION)),
		});
	})
}

#[test]
fn pool_duration_changes_after_cycle() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(Staking::current_era(), None);

		let pool_id = NextPoolId::<Runtime>::get() - 1;

		// make sure the duration is set correctly
		let pool = BondedPools::<Runtime>::get(pool_id).unwrap();
		let cycle = pool.bonus_cycle;
		assert_eq!(cycle.start, 0);
		assert_eq!(cycle.end, DEFAULT_DURATION);
		assert_eq!(cycle.pending_duration, None);

		// mutate the duration
		let new_duration = 50;
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(DEFAULT_MANAGER),
			pool_id,
			PoolMutationOf::<Runtime> { duration: Some(new_duration), ..Default::default() }
		));

		// make sure the pending duration is set correctly
		let mut pool = BondedPool::<Runtime>::get(pool_id).unwrap();
		let pool = pool.into_mut();
		let cycle = pool.bonus_cycle.clone();
		assert_eq!(cycle.start, 0);
		assert_eq!(cycle.end, DEFAULT_DURATION);
		assert_eq!(cycle.pending_duration, Some(new_duration));

		// set the current era to the end of the duration
		pallet_staking::CurrentEra::<Runtime>::set(Some(DEFAULT_DURATION));
		pool.end_era(DEFAULT_DURATION).unwrap();

		// make sure the duration is set correctly after the cycle
		let cycle = &pool.bonus_cycle;
		assert_eq!(cycle.start, DEFAULT_DURATION + 1);
		assert_eq!(cycle.end, DEFAULT_DURATION + 1 + new_duration);
		assert_eq!(cycle.pending_duration, None);
	});
}

#[test]
fn mutate_commission_works() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = 0;
		let token_owner = 10;
		let admin = 900;

		let mutation = |new_commission: Option<Perbill>| PoolMutationOf::<Runtime> {
			new_commission: ShouldMutate::SomeMutation(new_commission),
			..Default::default()
		};

		// fails because pool does not exist
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(token_owner),
				2,
				mutation(Some(Perbill::from_percent(9)))
			),
			Error::<Runtime>::PoolNotFound,
		);

		// fails because caller (admin) is not token_owner
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(admin),
				pool_id,
				mutation(Some(Perbill::from_percent(9))),
			),
			Error::<Runtime>::DoesNotHavePermission,
		);

		// fails if mutation is a no-op
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(token_owner),
				pool_id,
				PoolMutationOf::<Runtime>::default(),
			),
			Error::<Runtime>::NoopMutation
		);

		// fails if set outside the bounds
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(token_owner),
				pool_id,
				mutation(Some(Perbill::from_percent(11))),
			),
			Error::<Runtime>::CommissionExceedsMaximum,
		);

		// works if set by the `token_owner` role within the bounds
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(token_owner),
			pool_id,
			mutation(Some(Perbill::from_percent(9)))
		));

		// bonded pool has correct commission set
		assert_eq!(
			BondedPools::<Runtime>::get(pool_id).unwrap().commission,
			Commission::<Runtime> {
				current: Some(Perbill::from_percent(9)),
				change_rate: None,
				max: None,
				throttle_from: Some(1)
			}
		);

		// events are emitted
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { creator: 10, pool_id, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
				Event::PoolMutated { pool_id, mutation: mutation(Some(Perbill::from_percent(9))) }
			]
		);

		// removing the commission works
		assert_ok!(Pools::mutate(RuntimeOrigin::signed(token_owner), pool_id, mutation(None)));

		// events are emitted
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::PoolMutated { pool_id, mutation: mutation(None) }]
		);

		// bonded pool doesnt have commission set
		assert_eq!(
			BondedPools::<Runtime>::get(pool_id).unwrap().commission,
			Commission::<Runtime> {
				current: None,
				change_rate: None,
				max: None,
				throttle_from: Some(1)
			}
		);
	});
}

#[test]
fn mutate_max_commission_works() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = 0;
		let token_owner = 10;
		let admin = 900;

		let global_max = GlobalMaxCommission::<Runtime>::get().unwrap();

		let mutation = |max_commission: Option<Perbill>| PoolMutationOf::<Runtime> {
			max_commission,
			..Default::default()
		};

		let commission_mutation = |new_commission: Option<Perbill>| PoolMutationOf::<Runtime> {
			new_commission: ShouldMutate::SomeMutation(new_commission),
			..Default::default()
		};

		// cannot set commission max above GlobalMaxCommission
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(token_owner),
				pool_id,
				mutation(Some(global_max + Perbill::from_percent(1)))
			),
			Error::<Runtime>::CommissionExceedsMaximum,
		);

		// only token_owner can set commission max
		assert_noop!(
			Pools::mutate(RuntimeOrigin::signed(admin), pool_id, mutation(Some(global_max))),
			Error::<Runtime>::DoesNotHavePermission,
		);

		// can set commission max to GlobalMaxCommission
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(token_owner),
			pool_id,
			mutation(Some(global_max)),
		));

		// commission max is set
		assert_eq!(
			BondedPools::<Runtime>::get(pool_id).unwrap().commission,
			Commission::<Runtime> {
				current: None,
				change_rate: None,
				max: Some(global_max),
				throttle_from: None
			}
		);

		// events are emitted
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { creator: 10, pool_id, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
				Event::PoolMutated { pool_id, mutation: mutation(Some(global_max)) },
			]
		);

		// force set max to 9%
		let mut pool = BondedPool::<Runtime>::get(pool_id).unwrap();
		pool.commission = Commission::<Runtime> {
			current: None,
			change_rate: None,
			max: Some(global_max - Perbill::from_percent(1)),
			throttle_from: None,
		};
		pool.put();

		// cannot set max higher than current
		assert_noop!(
			Pools::mutate(RuntimeOrigin::signed(token_owner), pool_id, mutation(Some(global_max))),
			Error::<Runtime>::MaxCommissionRestricted,
		);

		// can set max lower than current
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(token_owner),
			pool_id,
			mutation(Some(global_max - Perbill::from_percent(2))),
		));

		// max is set
		assert_eq!(
			BondedPools::<Runtime>::get(pool_id).unwrap().commission,
			Commission::<Runtime> {
				current: None,
				change_rate: None,
				max: Some(global_max - Perbill::from_percent(2)),
				throttle_from: None
			}
		);

		// events are emitted
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::PoolMutated {
				pool_id,
				mutation: mutation(Some(global_max - Perbill::from_percent(2))),
			}]
		);

		// cannot set current commission above max
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(token_owner),
				pool_id,
				PoolMutationOf::<Runtime> {
					new_commission: ShouldMutate::SomeMutation(Some(
						global_max - Perbill::from_percent(1)
					)),
					..Default::default()
				}
			),
			Error::<Runtime>::CommissionExceedsMaximum,
		);

		// can set current commission to max
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(token_owner),
			pool_id,
			commission_mutation(Some(global_max - Perbill::from_percent(2)))
		));

		// current commission is set
		assert_eq!(
			BondedPools::<Runtime>::get(pool_id).unwrap().commission,
			Commission::<Runtime> {
				current: Some(global_max - Perbill::from_percent(2)),
				change_rate: None,
				max: Some(global_max - Perbill::from_percent(2)),
				throttle_from: Some(1),
			}
		);

		// events are emitted
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::PoolMutated {
				pool_id,
				mutation: commission_mutation(Some(global_max - Perbill::from_percent(2))),
			}]
		);

		// changing max to value below current updates current to max
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(token_owner),
			pool_id,
			mutation(Some(global_max - Perbill::from_percent(3))),
		));

		// max and current commissions are set
		assert_eq!(
			BondedPools::<Runtime>::get(pool_id).unwrap().commission,
			Commission::<Runtime> {
				current: Some(global_max - Perbill::from_percent(3)),
				change_rate: None,
				max: Some(global_max - Perbill::from_percent(3)),
				throttle_from: Some(1),
			}
		);

		// events are emitted
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::CommissionUpdated {
					pool_id,
					current: Some(global_max - Perbill::from_percent(3)),
				},
				Event::PoolMutated {
					pool_id,
					mutation: mutation(Some(global_max - Perbill::from_percent(3))),
				},
			]
		);
	})
}

#[test]
fn mutate_commission_change_rate_works() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = 0;
		let token_owner = 10;
		let admin = 900;
		let initial_balance = Balances::free_balance(token_owner);
		let pool = BondedPool::<Runtime>::get(pool_id).unwrap();

		let mutation =
			|change_rate: Option<CommissionChangeRate<BlockNumber>>| PoolMutationOf::<Runtime> {
				change_rate,
				..Default::default()
			};

		let commission_mutation = |new_commission: Option<Perbill>| PoolMutationOf::<Runtime> {
			new_commission: ShouldMutate::SomeMutation(new_commission),
			..Default::default()
		};

		// set initial commission
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(token_owner),
			pool_id,
			commission_mutation(Some(Perbill::from_percent(5)))
		));

		// cannot set on non-existent pool
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(token_owner),
				2,
				mutation(Some(CommissionChangeRate::<BlockNumber> {
					max_delta: Perbill::from_percent(2),
					min_delay: 1,
				})),
			),
			Error::<Runtime>::PoolNotFound,
		);

		// can only be set by token_owner account
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(admin),
				pool_id,
				mutation(Some(CommissionChangeRate::<BlockNumber> {
					max_delta: Perbill::from_percent(2),
					min_delay: 1,
				})),
			),
			Error::<Runtime>::DoesNotHavePermission,
		);

		// can be set by the `token_owner` who has the pool token
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(token_owner),
			pool_id,
			mutation(Some(CommissionChangeRate::<BlockNumber> {
				max_delta: Perbill::from_percent(2),
				min_delay: 3,
			})),
		));

		// change rate is set
		assert_eq!(
			BondedPools::<Runtime>::get(pool_id).unwrap().commission,
			Commission::<Runtime> {
				current: Some(Perbill::from_percent(5)),
				change_rate: Some(CommissionChangeRate::<BlockNumber> {
					max_delta: Perbill::from_percent(2),
					min_delay: 3,
				}),
				max: None,
				throttle_from: Some(1)
			}
		);

		// events are emitted
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				Event::Created { creator: 10, pool_id, capacity: 1_000 },
				Event::Bonded { member: pool_bonded_account(pool_id), pool_id, bonded: 10 },
				Event::PoolMutated {
					pool_id,
					mutation: commission_mutation(Some(Perbill::from_percent(5))),
				},
				Event::PoolMutated {
					pool_id,
					mutation: mutation(Some(CommissionChangeRate::<BlockNumber> {
						max_delta: Perbill::from_percent(2),
						min_delay: 3,
					}))
				}
			]
		);

		// change rate max_delta cannot be set to less restrictive
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(token_owner),
				pool_id,
				mutation(Some(CommissionChangeRate::<BlockNumber> {
					max_delta: Perbill::from_percent(3),
					min_delay: 3,
				}))
			),
			Error::<Runtime>::CommissionChangeRateNotAllowed,
		);

		// change rate min_delay cannot be set to less restrictive
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(token_owner),
				pool_id,
				mutation(Some(CommissionChangeRate::<BlockNumber> {
					max_delta: Perbill::from_percent(2),
					min_delay: 2,
				}))
			),
			Error::<Runtime>::CommissionChangeRateNotAllowed,
		);

		// change rate can be set to more restrictive
		let max_delta = Perbill::from_percent(1);
		let min_delay = 4;
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(token_owner),
			pool_id,
			mutation(Some(CommissionChangeRate::<BlockNumber> { max_delta, min_delay })),
		));

		// change rate is set
		let commission = Perbill::from_percent(5);
		assert_eq!(
			BondedPools::<Runtime>::get(pool_id).unwrap().commission,
			Commission::<Runtime> {
				current: Some(commission),
				change_rate: Some(CommissionChangeRate::<BlockNumber> { max_delta, min_delay }),
				max: None,
				throttle_from: Some(1)
			}
		);

		// events are emitted
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::PoolMutated {
				pool_id,
				mutation: mutation(Some(CommissionChangeRate::<BlockNumber> {
					max_delta: Perbill::from_percent(1),
					min_delay,
				})),
			}]
		);

		// cannot set commission before change rate min_delay has passed
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(token_owner),
				pool_id,
				commission_mutation(Some(Perbill::from_percent(5))),
			),
			Error::<Runtime>::CommissionChangeThrottled,
		);

		run_to_block(5);

		// cannot increase commission by more than change rate max_delta
		let bad_commission = commission + max_delta + Perbill::from_percent(1);
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(token_owner),
				pool_id,
				commission_mutation(Some(bad_commission)),
			),
			Error::<Runtime>::CommissionChangeThrottled,
		);

		// cannot decrease commission by more than change rate max_delta
		let bad_commission = commission - max_delta - Perbill::from_percent(1);
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(token_owner),
				pool_id,
				commission_mutation(Some(bad_commission))
			),
			Error::<Runtime>::CommissionChangeThrottled,
		);

		// can increase the commission by change rate max_delta after min_delay has passed
		let new_commission = commission + max_delta;
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(token_owner),
			pool_id,
			commission_mutation(Some(new_commission))
		));

		// commission is set
		assert_eq!(
			BondedPools::<Runtime>::get(pool_id).unwrap().commission,
			Commission::<Runtime> {
				current: Some(new_commission),
				change_rate: Some(CommissionChangeRate::<BlockNumber> { max_delta, min_delay }),
				max: None,
				throttle_from: Some(5)
			}
		);

		// events are emitted
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::PoolMutated {
				pool_id,
				mutation: commission_mutation(Some(new_commission))
			}]
		);

		// can decrease the commission by change rate max_delta after min_delay has passed
		let new_commission = new_commission - max_delta;
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(token_owner),
			pool_id,
			commission_mutation(Some(new_commission)),
		));

		// commission is set
		assert_eq!(
			BondedPools::<Runtime>::get(pool_id).unwrap().commission,
			Commission::<Runtime> {
				current: Some(new_commission),
				change_rate: Some(CommissionChangeRate::<BlockNumber> { max_delta, min_delay }),
				max: None,
				throttle_from: Some(5)
			}
		);

		// events are emitted
		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::PoolMutated {
				pool_id,
				mutation: commission_mutation(Some(new_commission))
			}]
		);

		// can claim commission
		let reward_account = pool.reward_account();
		deposit_rewards(100);
		let total_rewards = Balances::free_balance(reward_account);
		let pool = BondedPool::<Runtime>::get(pool_id).unwrap();
		let payment = pool.claim_commission().unwrap();

		// commission is paid out
		let expected_commission = total_rewards * 5 / 100;
		assert_eq!(
			payment.unwrap(),
			CommissionPayment { beneficiary: token_owner, amount: expected_commission }
		);
		assert_eq!(Balances::free_balance(token_owner), initial_balance + expected_commission);
		assert_eq!(Balances::free_balance(reward_account), total_rewards - expected_commission);
	})
}

#[test]
fn test_mutate_capacity() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = 0;
		let token_owner = 10;
		let admin = 900;
		let nominator = 901;

		let mutation = |capacity: Option<Balance>| PoolMutationOf::<Runtime> {
			capacity,
			..Default::default()
		};

		// fails because pool does not exist
		assert_noop!(
			Pools::mutate(RuntimeOrigin::signed(admin), pool_id + 1, mutation(Some(1))),
			Error::<Runtime>::PoolNotFound
		);

		// fails if mutation is a no-op
		assert_noop!(
			Pools::mutate(RuntimeOrigin::signed(admin), pool_id, mutation(None)),
			Error::<Runtime>::NoopMutation
		);

		// fails if non-admin or non-token-owner tries to mutate
		assert_noop!(
			Pools::mutate(RuntimeOrigin::signed(nominator), pool_id, mutation(Some(1))),
			Error::<Runtime>::DoesNotHavePermission
		);

		// fails when capacity is set lower that current points of the pool
		let pool = BondedPool::<Runtime>::get(pool_id).unwrap();
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(token_owner),
				pool_id,
				mutation(Some(pool.points() - 1))
			),
			Error::<Runtime>::CapacityExceeded,
		);

		// fails when capacity is set below `num of pool validators X min validator stake`

		// make sure pool is nominating validators
		let validators = vec![21, 31];
		assert_ok!(Pools::nominate(RuntimeOrigin::signed(token_owner), pool_id, validators));

		// make sure the pool points (30) are bigger than validators (2) times min_stake (10) = 20
		Balances::make_free_balance_be(&admin, 10_000 * UNIT);
		assert_ok!(Pools::bond(RuntimeOrigin::signed(token_owner), pool_id, 10.into()));
		assert_ok!(Pools::bond(RuntimeOrigin::signed(admin), pool_id, 10.into()));

		let min_validator_stake = StakingMinBond::get();
		let num_validators = Nominations::get().map(|x| x.len()).unwrap_or_default();

		// sanity check, cannot bond more than capacity
		assert_noop!(
			Pools::bond(RuntimeOrigin::signed(admin), pool_id, 1200.into()),
			Error::<Runtime>::CapacityExceeded
		);

		let min_capacity = min_validator_stake * num_validators as u128;
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(token_owner),
				pool_id,
				mutation(Some(min_capacity - 1))
			),
			Error::<Runtime>::CapacityExceeded,
		);

		// works as expected when mutated within 14 eras from cycle start
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(token_owner),
			pool_id,
			mutation(Some(1500))
		),);

		// pool capacity was mutated
		assert_eq!(BondedPool::<Runtime>::get(pool_id).unwrap().capacity, 1500);

		// should be able to bond an amount larger than previous pool capacity
		assert_ok!(Pools::bond(RuntimeOrigin::signed(admin), pool_id, 1200.into()));

		// fails when mutating after 14 eras from cycle start
		run_to_era(<Runtime as Config>::CapacityMutationPeriod::get() + 1);
		assert_noop!(
			Pools::mutate(RuntimeOrigin::signed(token_owner), pool_id, mutation(Some(100))),
			Error::<Runtime>::CapacityMutationRestricted
		);
	});
}

#[test]
fn test_mutate_capacity_with_attributes() {
	ExtBuilder::default().build_and_execute(|| {
		let token_id = DEFAULT_TOKEN_ID;
		let pool_id = 0;
		// Balances::make_free_balance_be(&11, 21_000_000 * UNIT);

		// given global max is set at 20M TNT
		// and default capacity is 500K TNT

		// mutating a pool without attribute must fallback to default 500K max capacity
		// and if we try to mutate a pool with capacity above 500K TNT, it should fail
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(10),
				pool_id,
				PoolMutationOf::<Runtime> { capacity: Some(500_001 * UNIT), ..Default::default() },
			),
			Error::<Runtime>::CapacityExceeded
		);

		// non number capacity attribute should fail
		assert_ok!(<Runtime as Config>::FungibleHandler::set_attribute(
			RuntimeOrigin::signed(10),
			<<Runtime as Config>::PoolCollectionId as Get<CollectionIdOf<Runtime>>>::get(),
			Some(token_id),
			b"max_pool_capacity".to_vec().try_into().unwrap(),
			b"not-capacity".to_vec().try_into().unwrap(),
			None
		));

		// mutating a pool with non number capacity attribute should fail
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(10),
				pool_id,
				PoolMutationOf::<Runtime> { capacity: Some(1_000 * UNIT), ..Default::default() },
			),
			Error::<Runtime>::AttributeValueDecodeFailed
		);

		// set max_pool_capacity attribute to exceed global 20M TNT
		assert_ok!(<Runtime as Config>::FungibleHandler::set_attribute(
			RuntimeOrigin::signed(10),
			<<Runtime as Config>::PoolCollectionId as Get<CollectionIdOf<Runtime>>>::get(),
			Some(token_id),
			b"max_pool_capacity".to_vec().try_into().unwrap(),
			(20_000_001 * UNIT).to_string().as_bytes().to_vec().try_into().unwrap(),
			None
		));

		// mutating a pool with attribute capacity exceeding global capacity should fail
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(10),
				pool_id,
				PoolMutationOf::<Runtime> { capacity: Some(1_000 * UNIT), ..Default::default() }
			),
			Error::<Runtime>::AttributeCapacityExceedsGlobalCapacity
		);

		// set max_pool_capacity attribute to 1M TNT
		assert_ok!(<Runtime as Config>::FungibleHandler::set_attribute(
			RuntimeOrigin::signed(10),
			<<Runtime as Config>::PoolCollectionId as Get<CollectionIdOf<Runtime>>>::get(),
			Some(token_id),
			b"max_pool_capacity".to_vec().try_into().unwrap(),
			(1_000_000 * UNIT).to_string().as_bytes().to_vec().try_into().unwrap(),
			None
		));

		// mutating a pool with capacity exceeding attribute capacity should fail
		assert_noop!(
			Pools::mutate(
				RuntimeOrigin::signed(10),
				pool_id,
				PoolMutationOf::<Runtime> {
					capacity: Some(1_000_001 * UNIT),
					..Default::default()
				}
			),
			Error::<Runtime>::CapacityExceeded
		);

		// mutating a pool with capacity below attribute capacity should succeed
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(10),
			pool_id,
			PoolMutationOf::<Runtime> { capacity: Some(1_000_000 * UNIT), ..Default::default() }
		));
	})
}

#[test]
fn test_mutate_name() {
	ExtBuilder::default().build_and_execute(|| {
		let pool_id = 0;
		let name: PoolNameOf<Runtime> = b"new_name".to_vec().try_into().unwrap();

		Pools::mutate(
			RuntimeOrigin::signed(DEFAULT_MANAGER),
			pool_id,
			PoolMutation { name: Some(name.clone()), ..Default::default() },
		)
		.unwrap();

		assert_eq!(BondedPool::<Runtime>::get(pool_id).unwrap().name, name);
	});
}
