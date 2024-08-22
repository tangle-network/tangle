use super::*;

use crate::{functions::calculate_real_weight, PoolInfo};
use frame_support::{dispatch::WithPostDispatchInfo, traits::WithdrawReasons};
use pallet_staking::{Exposure, IndividualExposure};
use std::collections::HashMap;

/// Simulate calling `payout_rewards` by setting the balances of the pool reward accounts
pub fn simulate(validator_stash: AccountId, era: EraIndex) {
	let rewards = MockRewards::get();
	for (pool_id, amount) in rewards.get(&(validator_stash, era)).unwrap().iter() {
		let account_id = pool_reward_account(*pool_id);
		let balance = Balances::free_balance(account_id);
		Balances::make_free_balance_be(&account_id, balance + *amount);
	}
}

fn prepare_validator(stash: AccountId, controller: AccountId) {
	pallet_staking::Bonded::<Runtime>::insert(stash, controller);
	pallet_staking::Ledger::<Runtime>::insert(
		controller,
		pallet_staking::StakingLedger::<Runtime>::new(stash, Balances::minimum_balance() - 1),
	);
}

/// Sets up an era for payout. This requires calling [`set_reward`] first.
fn prepare_era(validator: AccountId, era: EraIndex, set_to_current: bool) {
	// exposure uses the pool ids from the mock payout rewards
	let exposure = Exposure {
		others: MockRewards::get()
			.get(&(validator, era))
			.unwrap()
			.iter()
			.map(|(pool_id, _)| IndividualExposure { who: pool_bonded_account(*pool_id), value: 0 })
			.collect(),
		..Default::default()
	};
	if set_to_current {
		pallet_staking::CurrentEra::<Runtime>::put(era);
	}
	pallet_staking::ErasStakersClipped::<T>::insert(era, validator, exposure);
	pallet_staking::ErasValidatorReward::<Runtime>::insert(era, 1);
}

/// Pool info for testing
pub(crate) struct PoolInfo {
	pub pool_id: PoolId,
	pub creator: AccountId,
	pub token_id: TokenId,
	pub duration: EraIndex,
	pub commission: Option<Perbill>,
}

/// Creates a pool and sets it up with validators
pub(crate) fn prepare_pool(info: &PoolInfo) {
	let depositor = info.creator;
	mint_pool_token(info.token_id, depositor);
	Balances::make_free_balance_be(&depositor, 100_000 + Balances::minimum_balance());
	let pool_id = NextPoolId::<Runtime>::get();
	Pools::create(
		RuntimeOrigin::signed(depositor),
		info.token_id,
		100,
		1_000,
		info.duration,
		Default::default(),
	)
	.unwrap();

	// set commission
	assert_eq!(pool_id, info.pool_id);
	if let Some(commission) = info.commission.as_ref() {
		Pools::mutate(
			RuntimeOrigin::signed(depositor),
			pool_id,
			PoolMutationOf::<Runtime> {
				new_commission: SomeMutation(Some(*commission)),
				..Default::default()
			},
		)
		.unwrap();
	}
}

/// make sure the mock functions are working
#[test]
fn test_mock_payout_functions() {
	ExtBuilder::default().without_pool().build_and_execute(|| {
		let validator = 623;
		set_reward(validator, 0, 1, 100);
		set_reward(validator, 0, 2, 200);
		set_reward(validator, 1, 1, 500);
		set_reward(validator, 1, 2, 700);

		assert_eq!(get_reward(validator, 0, 1), 100);
		assert_eq!(get_reward(validator, 0, 2), 200);
		assert_eq!(get_reward(validator, 1, 1), 500);
		assert_eq!(get_total_reward(validator, 0), 300);
		assert_eq!(get_total_reward(validator, 1), 1_200);
	})
}

/// Test that the payout works as expected
#[test]
fn test_payout_rewards() {
	ExtBuilder::default().without_pool().build_and_execute(|| {
		let validator = 159;

		// set up mock rewards
		set_reward(validator, 0, 0, 2_700);
		set_reward(validator, 0, 1, 6_300);
		set_reward(validator, 1, 0, 3_000);
		set_reward(validator, 1, 1, 7_000);

		let pool_infos = [
			PoolInfo {
				pool_id: 0,
				creator: 72,
				token_id: 1,
				duration: <<Runtime as Config>::MaxDuration as Get<_>>::get(),
				commission: Some(Perbill::from_percent(2)),
			},
			PoolInfo {
				pool_id: 1,
				creator: 61,
				token_id: 2,
				duration: <<Runtime as Config>::MinDuration as Get<_>>::get(),
				commission: Some(Perbill::from_percent(5)),
			},
		];

		for pool_info in &pool_infos {
			prepare_pool(pool_info);
		}

		// setup variables
		let minimum_balance = Balances::minimum_balance();
		let eighty_percent = Perbill::from_percent(80);
		let error_weight = WeightInfoOf::<Runtime>::payout_rewards(0);

		let pool_ids = [0, 1];
		let initial_creator_balances = [
			Balances::free_balance(pool_infos[0].creator),
			Balances::free_balance(pool_infos[1].creator),
		];

		// prepare era 0
		let era = 0;
		pallet_staking::CurrentEra::<Runtime>::put(era);

		// get the bonded amounts before calling payout for referencing later
		let initial_bonds: HashMap<PoolId, Balance> = [
			(0, StakingMock::active_stake(&pool_bonded_account(0)).unwrap_or_default()),
			(1, StakingMock::active_stake(&pool_bonded_account(1)).unwrap_or_default()),
		]
		.into_iter()
		.collect();

		// check balances are as expected
		for pool_id in pool_ids {
			assert_eq!(Balances::free_balance(pool_reward_account(pool_id)), minimum_balance);
			assert_eq!(Balances::free_balance(pool_bonus_account(pool_id)), minimum_balance);
		}

		// dummy value that will cause failures
		pallet_staking::ErasStakersClipped::<T>::insert(era, validator, Exposure::default());

		// fails because ErasValidatorReward key is missing
		assert_noop!(
			Pools::payout_rewards(RuntimeOrigin::signed(10), validator, era),
			pallet_staking::Error::<Runtime>::InvalidEraToReward.with_weight(error_weight)
		);
		pallet_staking::ErasValidatorReward::<Runtime>::insert(era, 0);

		// fails because validator is not bonded
		assert_noop!(
			Pools::payout_rewards(RuntimeOrigin::signed(10), validator, era),
			pallet_staking::Error::<Runtime>::NotStash.with_weight(error_weight)
		);
		let controller = 842;
		pallet_staking::Bonded::<Runtime>::insert(validator, controller);

		// fails because ledger doesn't exist
		assert_noop!(
			Pools::payout_rewards(RuntimeOrigin::signed(10), validator, era),
			pallet_staking::Error::<Runtime>::NotController.with_weight(error_weight)
		);
		pallet_staking::Ledger::<Runtime>::insert(
			controller,
			pallet_staking::StakingLedger::<Runtime>::new(
				validator,
				Balances::minimum_balance() - 1,
			),
		);

		// clear events
		pool_events_since_last_call();

		// payout era 0
		prepare_era(validator, era, true);
		Pools::payout_rewards(RuntimeOrigin::signed(10), validator, era).unwrap();
		assert_eq!(
			EraPayoutInfo::<Runtime>::get(),
			EraPayout { era, payout_count: 1, ..Default::default() }
		);

		// no new bonus events
		let events = pool_events_since_last_call();
		assert_eq!(filter_events!(events, Event::EraRewardsProcessed { .. }), vec![]);

		let real_weights = [
			Perbill::from_percent(100).mul_floor(get_reward(validator, 0, 0)),
			Perbill::from_percent(0).mul_floor(get_reward(validator, 0, 1)),
		];

		// set up necessary variables for bonus calculation
		let total_reward = get_total_reward(validator, era);
		assert_eq!(total_reward, 9_000);

		let total_bonus = Perbill::from_percent(20).mul_floor(total_reward);
		let total_base_bonus_amount = Perbill::from_percent(25).mul_floor(total_bonus);
		let total_weighted_bonus_amount = total_bonus - total_base_bonus_amount;

		let base_bonus_factor = Perbill::from_rational(total_base_bonus_amount, total_reward);
		let weighted_bonus_factor =
			Perbill::from_rational(total_weighted_bonus_amount, real_weights[0] + real_weights[1]);

		// check per pool values
		for pool_id in pool_ids.iter().copied() {
			// calculate values
			let index = pool_id as usize;
			let reward_from_validator = get_reward(validator, era, pool_id);
			let reward = eighty_percent.mul_floor(reward_from_validator);
			let bonus = base_bonus_factor.mul_floor(reward_from_validator) +
				weighted_bonus_factor.mul_floor(real_weights[index]);

			// check events
			assert_eq!(
				events[index],
				Event::RewardPaid { pool_id, era: 0, validator_stash: validator, reward, bonus }
			);
			// check pool account balances
			assert_eq!(
				Balances::free_balance(pool_reward_account(pool_id)),
				reward + minimum_balance
			);
			assert_eq!(
				Balances::free_balance(pool_bonus_account(pool_id)),
				bonus + minimum_balance
			);

			// creator balances did not change because no commission sent yet
			assert_eq!(
				Balances::free_balance(pool_infos[index].creator),
				initial_creator_balances[index]
			);

			// bonded balance does not change because no rebonding occurs
			assert_eq!(
				StakingMock::active_stake(&pool_bonded_account(pool_id)).unwrap_or_default(),
				*initial_bonds.get(&pool_id).unwrap()
			);
		}

		let era_0_bonus_balances = pool_ids
			.iter()
			.map(|pool_id| Balances::free_balance(pool_bonus_account(*pool_id)))
			.collect::<Vec<_>>();

		// payout era 1
		let era = 1;

		prepare_era(validator, era, true);
		Pools::payout_rewards(RuntimeOrigin::signed(10), validator, era).unwrap();
		assert_eq!(
			EraPayoutInfo::<Runtime>::get(),
			EraPayout { era, payout_count: 1, ..Default::default() }
		);

		// set expected values that we will check while iterating
		let bonuses_transferred = [1, 10];
		let commissions_paid = [43, 252];
		// the amount reinvested is 80% reward minus commission plus bonus paid
		let reinvested_amounts = pool_ids
			.iter()
			.map(|pool_id| {
				let index = *pool_id as usize;
				let reward = eighty_percent.mul_floor(get_reward(validator, 0, *pool_id));
				reward - commissions_paid[index] + bonuses_transferred[index]
			})
			.collect::<Vec<_>>();

		// we only have two durations: `max` and `min`, so the normalized weights are: 1 and 0
		let real_weights = [
			Perbill::from_percent(100).mul_floor(get_reward(validator, 1, 0)),
			Perbill::from_percent(0).mul_floor(get_reward(validator, 1, 1)),
		];

		// set up weights
		let total_reward = get_total_reward(validator, era);
		assert_eq!(total_reward, 10_000);

		let total_bonus = Perbill::from_percent(20).mul_floor(total_reward);
		let total_base_bonus_amount = Perbill::from_percent(25).mul_floor(total_bonus);
		let total_weighted_bonus_amount = total_bonus - total_base_bonus_amount;

		let base_bonus_factor = Perbill::from_rational(total_base_bonus_amount, total_reward);
		let weighted_bonus_factor =
			Perbill::from_rational(total_weighted_bonus_amount, real_weights[0] + real_weights[1]);

		// collect events
		let events = pool_events_since_last_call();
		let bonus_events = filter_events!(events, Event::EraRewardsProcessed { .. });
		let reward_events = filter_events!(events, Event::RewardPaid { .. });

		for pool_id in pool_ids {
			// calculate values
			let index = pool_id as usize;
			let reward_from_validator = get_reward(validator, era, pool_id);
			let reward = eighty_percent.mul_floor(reward_from_validator);
			let bonus = base_bonus_factor.mul_floor(reward_from_validator) +
				weighted_bonus_factor.mul_floor(real_weights[index]);

			// check events
			assert_eq!(
				reward_events[index],
				Event::RewardPaid { pool_id, era: 1, validator_stash: validator, reward, bonus }
			);
			assert_eq!(
				bonus_events[index],
				Event::EraRewardsProcessed {
					pool_id,
					era: era - 1,
					commission: Some(CommissionPayment {
						beneficiary: pool_infos[index].creator,
						amount: commissions_paid[index]
					}),
					bonus: bonuses_transferred[index],
					reinvested: reinvested_amounts[index],
					bonus_cycle_ended: false,
				}
			);

			// commissions were delivered to creators
			assert_eq!(
				Balances::free_balance(pool_infos[index].creator),
				initial_creator_balances[index] + commissions_paid[index]
			);

			// era reward amount - 20%. The rest was reinvested.
			assert_eq!(
				Balances::free_balance(pool_reward_account(pool_id)),
				reward + minimum_balance
			);

			// check bonus account balance
			assert_eq!(
				Balances::free_balance(pool_bonus_account(pool_id)) - era_0_bonus_balances[index],
				bonus - bonuses_transferred[index]
			);

			// correct amount was reinvested
			assert_eq!(
				StakingMock::active_stake(&pool_bonded_account(pool_id)).unwrap_or_default(),
				*initial_bonds.get(&pool_id).unwrap() + reinvested_amounts[index]
			);
		}
	})
}

/// tests pools with multiple validators
#[test]
fn test_payout_rewards_multiple_validators() {
	ExtBuilder::default().without_pool().build_and_execute(|| {
		let bonus_percentage = <Runtime as Config>::BonusPercentage::get();
		let base_reward_percentage = Perbill::from_percent(100) - bonus_percentage;
		let minimum_balance = Balances::minimum_balance();
		let validators = [135, 598];
		for validator in validators {
			prepare_validator(validator, validator + 1);
		}
		let pool_infos = [
			PoolInfo { pool_id: 0, creator: 31, token_id: 1, duration: 100, commission: None },
			PoolInfo {
				pool_id: 1,
				creator: 61,
				token_id: 2,
				duration: 100,
				commission: Some(Perbill::from_percent(4)),
			},
			PoolInfo {
				pool_id: 2,
				creator: 915,
				token_id: 3,
				duration: 100,
				commission: Some(Perbill::from_percent(8)),
			},
			PoolInfo { pool_id: 3, creator: 672, token_id: 4, duration: 100, commission: None },
		];

		for pool_info in &pool_infos {
			prepare_pool(pool_info);
		}

		// we will use the same rewards every era
		let set_rewards = |era| {
			set_reward(validators[0], era, 0, 1_000);
			set_reward(validators[0], era, 1, 2_000);
			set_reward(validators[0], era, 2, 7_000);
			set_reward(validators[0], era, 3, 3_000);

			// validator 1 has 5_000 in rewards. pool 3 does not use this validator.
			set_reward(validators[1], era, 0, 2_000);
			set_reward(validators[1], era, 1, 4_000);
		};

		// prepare era 0
		let era = 0;
		set_rewards(era);

		// clear events
		pool_events_since_last_call();

		// payout era
		for (i, validator) in validators.into_iter().enumerate() {
			prepare_era(validator, era, true);
			Pools::payout_rewards(RuntimeOrigin::signed(10), validator, era).unwrap();
			assert_eq!(
				EraPayoutInfo::<Runtime>::get(),
				EraPayout { era, payout_count: i as u32 + 1, ..Default::default() }
			);
		}

		let reward_events = pool_events_since_last_call();
		// there are 4 nominators of validator 0, and 2 nominators of validator 1
		assert_eq!(reward_events.len(), 6);

		// function for checking reward events. needs to be called once for each era.
		let check_reward_events = || {
			// check first 4 events (validator 0)
			for pool_info in &pool_infos {
				let pool_id = pool_info.pool_id;
				let validator = validators[0];
				assert_eq!(
					reward_events[pool_id as usize],
					Event::RewardPaid {
						pool_id,
						era,
						validator_stash: validator,
						reward: base_reward_percentage
							.mul_floor(get_reward(validator, era, pool_id)),
						bonus: bonus_percentage.mul_floor(get_reward(validator, era, pool_id))
					}
				);
			}

			// check last 2 events (validator 1) - only pools 1 and 2 nominate it
			for pool_id in 0..2 {
				let pool_id = pool_id as PoolId;
				let validator = validators[1];
				assert_eq!(
					// checking events at index 4 and 5
					reward_events[(4 + pool_id) as usize],
					Event::RewardPaid {
						pool_id,
						era,
						validator_stash: validator,
						reward: base_reward_percentage
							.mul_floor(get_reward(validator, era, pool_id)),
						bonus: bonus_percentage.mul_floor(get_reward(validator, era, pool_id))
					}
				);
			}
		};
		check_reward_events();

		// check pool 1
		{
			// reward should include both validators
			let reward = 3_000;
			assert_eq!(
				Balances::free_balance(pool_reward_account(0)),
				Perbill::from_percent(80).mul_floor(reward) + minimum_balance
			);
		}

		// prepare era 1
		let era = 1;
		set_rewards(era);

		// payout era
		for (i, validator) in validators.into_iter().enumerate() {
			prepare_era(validator, era, true);
			Pools::payout_rewards(RuntimeOrigin::signed(10), validator, era).unwrap();
			assert_eq!(
				EraPayoutInfo::<Runtime>::get(),
				EraPayout { era, payout_count: i as u32 + 1, ..Default::default() }
			);
		}

		let events = pool_events_since_last_call();
		let reward_events = filter_events!(events, Event::RewardPaid { .. });
		assert_eq!(reward_events.len(), 6);

		// check bonus events
		assert_eq!(
			filter_events!(events, Event::EraRewardsProcessed { .. }),
			[
				Event::EraRewardsProcessed {
					pool_id: 0,
					era: era - 1,
					commission: None,
					bonus: 6,
					reinvested: 2_400 + 6,
					bonus_cycle_ended: false,
				},
				Event::EraRewardsProcessed {
					pool_id: 1,
					era: era - 1,
					commission: Some(CommissionPayment { beneficiary: 61, amount: 192 }),
					bonus: 12,
					reinvested: 4_800 - 192 + 12,
					bonus_cycle_ended: false,
				},
				Event::EraRewardsProcessed {
					pool_id: 2,
					era: era - 1,
					commission: Some(CommissionPayment { beneficiary: 915, amount: 449 }),
					bonus: 14,
					reinvested: 5_600 - 449 + 14,
					bonus_cycle_ended: false,
				},
				Event::EraRewardsProcessed {
					pool_id: 3,
					era: era - 1,
					commission: None,
					bonus: 6,
					reinvested: 2_400 + 6,
					bonus_cycle_ended: false,
				},
			]
		);

		// check reward events for second era
		check_reward_events();
	})
}

#[test]
fn test_payout_rewards_bonus_weights() {
	// Expected real weights and bonuses are calculated in the original excel sheet for bonus
	// distribution formula.
	ExtBuilder::default().without_pool().build_and_execute(|| {
		let durations = [120, 110, 100, 90, 80, 70, 60, 50, 40, 30];

		// each reward with 10^8 to avoid rounding errors
		let basic_rewards = [
			1050000000000,
			1025000000000,
			850000000000,
			412400000000,
			503000000000,
			928400000000,
			1584900000000,
			105700000000,
			524800000000,
			1015800000000,
		];

		let expected_real_weights: [Balance; 10] = [
			1050000000000,
			911111110200,
			661111110450,
			274933333058,
			279444444165,
			412622221809,
			528299999471,
			23488888865,
			58311111052,
			0,
		];

		let expected_base_bonus: [Balance; 10] = [
			65625000000,
			64062500000,
			53125000000,
			25775000000,
			31437500000,
			58025000000,
			99056250000,
			6606250000,
			32800000000,
			63487500000,
		]; // sums up to `5_000 * 10^8 - remainder`

		let expected_weighted_bonus: [Balance; 10] = [
			375060525000,
			325449344118,
			236149219208,
			98206324034,
			99817695177,
			147388863941,
			188709023961,
			8390242847,
			20828758023,
			0,
		]; // sums up to `15_000 * 10^8 - remainder`

		// if basic reward is 80% of all rewards, the rest 20% is bonus
		let total_pool_rewards = basic_rewards.into_iter().map(|r| r / 4 * 5).collect::<Vec<_>>();

		let validator = 135;
		prepare_validator(validator, validator + 1);

		let pool_infos = durations
			.into_iter()
			.enumerate()
			.map(|(index, duration)| {
				let pool_info = PoolInfo {
					pool_id: index as PoolId,
					creator: index as Balance + 31,
					token_id: index as TokenId + 1,
					duration,
					commission: None, // no commission to simplify the test
				};
				prepare_pool(&pool_info);
				pool_info
			})
			.collect::<Vec<_>>();

		// set the rewards for the given era
		let set_rewards = |era| {
			for (index, pool_info) in pool_infos.iter().enumerate() {
				set_reward(validator, era, pool_info.pool_id, total_pool_rewards[index]);
			}
		};

		// record remainder recipient balance before
		//
		// when we calculate rewards, we use `Perbill::mul_floor` which rounds down
		// so we need to check that the remainder is not lost in the process
		// all the remainder imbalance should be sent to the remainder recipient
		let remainder_recipient_balance_before =
			Balances::free_balance(<Runtime as Config>::LstCollectionOwner::get());

		// prepare era 0
		let era = 0;
		set_rewards(era);

		// clear events
		pool_events_since_last_call();

		// payout era
		prepare_era(validator, era, true);
		Pools::payout_rewards(RuntimeOrigin::signed(10), validator, era).unwrap();

		let events = pool_events_since_last_call();

		let mut total_paid_bonus = 0;

		// check real weights and bonus rewards
		for (index, pool_info) in pool_infos.iter().enumerate() {
			assert_eq!(
				calculate_real_weight::<Runtime>(
					durations[index],
					durations[durations.len() - 1],
					durations[0] - durations[durations.len() - 1],
					basic_rewards[index],
				),
				expected_real_weights[index],
			);

			let pool_id = pool_info.pool_id;
			let expected_bonus = expected_base_bonus[index] + expected_weighted_bonus[index];

			// check events
			assert_eq!(
				events[index],
				Event::RewardPaid {
					pool_id,
					era: 0,
					validator_stash: validator,
					reward: basic_rewards[index],
					bonus: expected_bonus
				}
			);

			assert_eq!(
				Balances::free_balance(pool_bonus_account(pool_id)),
				expected_bonus + Balances::minimum_balance(),
			);
			total_paid_bonus += expected_bonus;
		}

		// check remainder recipient balance
		let remainder_recipient_balance_after =
			Balances::free_balance(<Runtime as Config>::LstCollectionOwner::get());

		assert_eq!(
			remainder_recipient_balance_after - remainder_recipient_balance_before,
			Perbill::from_percent(20).mul_floor(total_pool_rewards.iter().sum::<Balance>()) -
				total_paid_bonus,
		);
	})
}

#[test]
fn test_bonus_rewards_zero_division() {
	// case where difference between max and min duration is 0
	// this case is checked and weights are simply distributed equally
	ExtBuilder::default().without_pool().build_and_execute(|| {
		let durations = [100, 100, 100];

		let validator = 135;

		prepare_validator(validator, validator + 1);

		let pool_infos = [
			PoolInfo {
				pool_id: 0,
				creator: 31,
				token_id: 1,
				duration: durations[0],
				commission: None, // no commission to simplify the test
			},
			PoolInfo {
				pool_id: 1,
				creator: 32,
				token_id: 2,
				duration: durations[1],
				commission: None, // no commission to simplify the test
			},
			PoolInfo {
				pool_id: 2,
				creator: 33,
				token_id: 3,
				duration: durations[2],
				commission: None, // no commission to simplify the test
			},
		];

		for pool_info in &pool_infos {
			prepare_pool(pool_info);
		}

		// set the rewards for the given era
		let set_rewards = |era| {
			for pool_info in &pool_infos {
				set_reward(validator, era, pool_info.pool_id, 10 ^ 8);
			}
		};

		// prepare era 0
		let era = 0;
		set_rewards(era);

		// clear events
		pool_events_since_last_call();

		// payout era
		prepare_era(validator, era, true);
		assert_ok!(Pools::payout_rewards(RuntimeOrigin::signed(10), validator, era));

		assert_eq!(
			filter_events!(pool_events_since_last_call(), Event::EraRewardsProcessed { .. }),
			vec![]
		);

		// bonus rewards should be equally distributed
		for pool_info in pool_infos {
			let pool_id = pool_info.pool_id;
			assert_eq!(
				Balances::free_balance(pool_bonus_account(pool_id)),
				Perbill::from_percent(25).mul_floor(10 ^ 8) / 3 + Balances::minimum_balance(),
			);
		}
	})
}

/// Tests paying out rewards for eras in different sequences. Mostly focuses on bonus.
#[test]
fn test_payout_eras_out_of_sequence() {
	/// Set the era
	fn set_current_era(era: EraIndex) {
		pallet_staking::CurrentEra::<Runtime>::set(Some(era));
	}

	fn check_first_event(bonus: Balance, reinvested: Balance) {
		if let Event::EraRewardsProcessed {
			bonus: event_bonus, reinvested: event_reinvested, ..
		} = pool_events_since_last_call()[0]
		{
			assert_eq!(event_bonus, bonus);
			assert_eq!(event_reinvested, reinvested);
		} else {
			panic!("invalid event");
		}
	}

	ExtBuilder::default().without_pool().build_and_execute(|| {
		let minimum_balance = Balances::minimum_balance();
		let error_weight = WeightInfoOf::<Runtime>::payout_rewards(0);

		// prepare validators
		let validators = [135, 598];
		for validator in validators {
			prepare_validator(validator, validator + 1);
		}

		// set up pool
		let pool_id = 0;
		let pool_info =
			PoolInfo { pool_id, creator: 61, token_id: 2, duration: 30, commission: None };
		prepare_pool(&pool_info);
		let bonus_account = get_pool(pool_id).unwrap().bonus_account();

		// we will use the same rewards every era (30_000 total reward)
		let set_rewards = |era| {
			set_reward(validators[0], era, pool_id, 10_000);
			set_reward(validators[1], era, pool_id, 20_000);
		};
		// the reward is 80 percent of the total reward (the other 20 percent goes to bonus)
		let reward = Perbill::from_percent(80).mul_floor(10_000 + 20_000);

		// function for paying the rewards
		let payout_rewards = |era| {
			// to make bonuses easier to understand, we set the bonus account to 1_000 every era
			Balances::make_free_balance_be(&bonus_account, 1_000);
			set_rewards(era);
			for validator in validators {
				prepare_era(validator, era, false);
				Pools::payout_rewards(RuntimeOrigin::signed(10), validator, era).unwrap();
			}
		};

		// set current era to 30
		set_current_era(30);
		pool_events_since_last_call();

		// make sure future era doesn't work
		set_rewards(31);
		prepare_era(validators[0], 31, false);
		assert_noop!(
			Pools::payout_rewards(RuntimeOrigin::signed(10), validators[0], 31),
			pallet_staking::Error::<Runtime>::InvalidEraToReward.with_weight(error_weight)
		);

		// no bonus on first payout
		payout_rewards(0);
		assert!(filter_events!(pool_events_since_last_call(), Event::EraRewardsProcessed { .. })
			.is_empty());

		// some bonus halfway through cycle
		payout_rewards(15);
		check_first_event(62, reward + 62);

		// make sure current era didn't change
		assert_eq!(pallet_staking::CurrentEra::<Runtime>::get(), Some(30));

		// smaller bonus than era 15 because it's earlier in the cycle
		payout_rewards(5);
		check_first_event(38, reward + 38);

		// full bonus on final era of cycle
		set_current_era(50);
		payout_rewards(30);
		check_first_event(1_000 - minimum_balance, reward + 995);

		// bonus has still not been cycled
		assert!(get_pool(pool_id).unwrap().inner.bonus_cycle.previous_start.is_none());

		// bonus gets cycled on 31 because it ends era 30
		payout_rewards(31);
		assert_eq!(get_pool(pool_id).unwrap().inner.bonus_cycle.previous_start, Some(0));
		check_first_event(0, reward);

		set_current_era(100);

		// pool gets cycled (zero bonus)
		payout_rewards(65);
		assert_eq!(get_pool(pool_id).unwrap().inner.bonus_cycle.previous_start, Some(31));
		check_first_event(0, reward);

		// pool cycled again
		payout_rewards(100);
		assert_eq!(get_pool(pool_id).unwrap().inner.bonus_cycle.previous_start, Some(62));
		check_first_event(0, reward);

		// zero bonus because more than two cycles in the past
		payout_rewards(48);
		check_first_event(0, reward);
	})
}

/// Tests bonus returned from [`BondedPool::maybe_cycle_and_calculate_bonus`]
#[test]
fn test_bonus() {
	/// Withdraws the bonus amount after it's calculated
	fn calculate_bonus_and_burn(pool: &mut BondedPool<Runtime>, era: EraIndex) -> Balance {
		let bonus = pool.maybe_cycle_and_calculate_bonus(era).unwrap().0;
		let _ = Balances::withdraw(
			&pool.bonus_account(),
			bonus,
			WithdrawReasons::TRANSFER,
			ExistenceRequirement::AllowDeath,
		)
		.unwrap();
		bonus
	}

	ExtBuilder::default().build_and_execute(|| {
		let mut pool = BondedPool::<Runtime>::get(0).unwrap();

		// use duration of 10 to make it simple
		let duration = 10;
		pool.bonus_cycle.end = duration;
		assert_eq!(pool.bonus_cycle.duration(), duration);
		let balance = 100;
		Balances::make_free_balance_be(&pool.bonus_account(), balance);

		// when the balance is burned, it is distributed linearly
		let values = (0..duration)
			.map(|era| calculate_bonus_and_burn(&mut pool, era))
			.collect::<Vec<_>>();
		assert_eq!(values.len(), duration as usize);
		assert_eq!(values, std::iter::repeat(10).take(10).collect::<Vec<_>>());

		// if we keep the balance consistent, it is distributed exponentially
		assert_eq!(pool.bonus_cycle.previous_start, None);
		Balances::make_free_balance_be(&pool.bonus_account(), balance);
		let values = (0..duration)
			.map(|era| pool.maybe_cycle_and_calculate_bonus(era).unwrap().0)
			.collect::<Vec<_>>();
		assert_eq!(values, [10, 11, 12, 14, 16, 20, 25, 33, 50, 100]);

		// the next era will cycle the pool and return balance of zero
		assert_eq!(pool.maybe_cycle_and_calculate_bonus(10).unwrap(), (0, true));
		assert_eq!(pool.bonus_cycle.previous_start, Some(0));

		// now the cycle should continue exactly as before
		let values = (11..21)
			.map(|era| pool.maybe_cycle_and_calculate_bonus(era).unwrap().0)
			.collect::<Vec<_>>();
		assert_eq!(values, [10, 11, 12, 14, 16, 20, 25, 33, 50, 100]);

		// give it a future start date so we can check before it started
		pool.bonus_cycle.previous_start = None;
		pool.bonus_cycle.start = 10;
		pool.bonus_cycle.end = 20;

		// it should be all zeroes until it starts
		let values = (5..11)
			.map(|era| pool.maybe_cycle_and_calculate_bonus(era).unwrap().0)
			.collect::<Vec<_>>();
		assert_eq!(values, vec![0, 0, 0, 0, 0, 10]);
	})
}

/// Tests [`calculate_real_weight`]
#[test]
fn test_calculate_real_weight() {
	fn check(
		duration: EraIndex,
		min_duration: EraIndex,
		max_duration: EraIndex,
		reward: Balance,
	) -> Balance {
		calculate_real_weight::<Runtime>(
			duration,
			min_duration,
			max_duration - min_duration,
			reward,
		)
	}

	assert_eq!(check(0, 0, 100, 1_000), 0);
	assert_eq!(check(50, 0, 100, 1_000), 500);
	assert_eq!(check(75, 0, 100, 1_000), 750);
	assert_eq!(check(100, 0, 100, 1_000), 1_000);
}
