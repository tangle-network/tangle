use super::*;
use pallet_staking::{ValidatorCount, ValidatorPrefs};
use payout_rewards::{prepare_pool, PoolInfo};

type Error = crate::Error<Runtime>;

#[test]
fn test_process_payouts() {
	ExtBuilder::default().without_pool().build_and_execute(|| {
		let era = 0;
		pallet_staking::CurrentEra::<Runtime>::put(era);

		// create pools
		let pool_ids = [0, 1];
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
		let pool_count = pool_infos.len() as u32;

		// insert validators. The count of payments must match the number of validators.
		let validators = [51, 70, 83];
		for validator in validators {
			pallet_staking::Validators::<Runtime>::insert(validator, ValidatorPrefs::default());
		}
		let validator_count = validators.len() as u32;
		ValidatorCount::<Runtime>::set(validator_count);

		// doesn't work with wrong pool count
		assert_noop!(Pools::process_payouts(RuntimeOrigin::signed(10), 1), Error::WrongPoolCount);

		// doesn't work with wrong payout count
		assert_noop!(
			Pools::process_payouts(RuntimeOrigin::signed(10), pool_count),
			Error::MissingPayouts
		);

		// setting to wrong era but correct count doesn't fix it
		EraPayoutInfo::<Runtime>::set(EraPayout {
			era: 32,
			payout_count: validator_count,
			..Default::default()
		});
		assert_noop!(
			Pools::process_payouts(RuntimeOrigin::signed(10), pool_count),
			Error::MissingPayouts
		);

		// now set correct payout count
		EraPayoutInfo::<Runtime>::set(EraPayout {
			era,
			payout_count: validator_count,
			..Default::default()
		});

		// clear pool events
		pool_events_since_last_call();

		// give each pool a bonus so they will be processed
		for pool_id in pool_ids {
			let account = Pools::compute_pool_account_id(pool_id, AccountType::Bonus);
			Balances::make_free_balance_be(&account, 100 * UNIT);
		}

		// the extrinsic succeeds
		Pools::process_payouts(RuntimeOrigin::signed(10), pool_count).unwrap();
		assert_eq!(
			EraPayoutInfo::<Runtime>::get(),
			EraPayout {
				era,
				payout_count: validator_count,
				payouts_processed: true,
				..Default::default()
			}
		);

		// make sure is one EraRewardsProcessed event per pool. We don't care about the contents.
		let events = pool_events_since_last_call();
		assert_eq!(events.len(), 2);
		for event in pool_events_since_last_call() {
			match event {
				Event::EraRewardsProcessed { .. } => {},
				_ => panic!(),
			}
		}

		// make sure bonuses_paid is set
		for pool_id in pool_ids {
			assert_eq!(BondedPool::<Runtime>::get(pool_id).unwrap().bonuses_paid, vec![era]);
		}

		// calling it again fails
		assert_noop!(
			Pools::process_payouts(RuntimeOrigin::signed(10), pool_count),
			Error::PayoutsAlreadyProcessed
		);
	})
}

#[test]
fn test_process_payouts_validator_count() {
	ExtBuilder::default().without_pool().build_and_execute(|| {
		let era = 0;
		pallet_staking::CurrentEra::<Runtime>::put(era);

		// set 10 validators
		let validators = [100, 101, 102, 103, 104, 105, 106, 107, 108, 109];
		for validator in validators {
			pallet_staking::Validators::<Runtime>::insert(validator, ValidatorPrefs::default());
		}
		let validator_count = validators.len() as u32;
		ValidatorCount::<Runtime>::set(validator_count);

		// set payout count to match and process payouts
		EraPayoutInfo::<Runtime>::set(EraPayout {
			era,
			payout_count: validator_count,
			..Default::default()
		});
		Pools::process_payouts(RuntimeOrigin::signed(10), 0).unwrap();

		// set payout count to one less and it fails
		EraPayoutInfo::<Runtime>::mutate(|x| {
			x.payout_count = validator_count - 1;
			x.payouts_processed = false
		});
		assert_noop!(Pools::process_payouts(RuntimeOrigin::signed(10), 0), Error::MissingPayouts);

		// decreasing validator count fixes it because it uses minimum of both
		ValidatorCount::<Runtime>::set(validator_count - 1);
		Pools::process_payouts(RuntimeOrigin::signed(10), 0).unwrap();

		// now do the inverse. set number of validators one higher than ValidatorCount
		EraPayoutInfo::<Runtime>::mutate(|info| {
			info.payout_count = validator_count;
			info.payouts_processed = false
		});
		ValidatorCount::<Runtime>::set(validator_count + 1);
		Pools::process_payouts(RuntimeOrigin::signed(10), 0).unwrap();

		// set payout count to 5 and it fails
		EraPayoutInfo::<Runtime>::mutate(|info| {
			info.payout_count = 5;
			info.payouts_processed = false
		});
		assert_noop!(Pools::process_payouts(RuntimeOrigin::signed(10), 0), Error::MissingPayouts);

		// setting to 55% still fails because it requires 6
		EraPayoutInfo::<Runtime>::mutate(|info| {
			info.required_payments_percent = Perbill::from_percent(55);
		});
		assert_noop!(Pools::process_payouts(RuntimeOrigin::signed(10), 0), Error::MissingPayouts);

		// set to 50% and it succeeds
		EraPayoutInfo::<Runtime>::mutate(|info| {
			info.required_payments_percent = Perbill::from_percent(50);
		});
		Pools::process_payouts(RuntimeOrigin::signed(10), 0).unwrap();

		// setting to 0% succeeds with no payouts
		EraPayoutInfo::<Runtime>::mutate(|info| {
			info.required_payments_percent = Perbill::from_percent(0);
			info.payouts_processed = false;
			info.payout_count = 0;
		});
		Pools::process_payouts(RuntimeOrigin::signed(10), 0).unwrap();
	});
}

#[test]
fn test_process_payouts_handles_era_depth_correctly() {
	ExtBuilder::default().without_pool().build_and_execute(|| {
		let era = 0;
		pallet_staking::CurrentEra::<Runtime>::put(era);

		// create pools
		let pool_ids = [0, 1];
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
		let pool_count = pool_infos.len() as u32;

		// insert validators. The count of payments must match the number of validators.
		let validators = [51, 70, 83];
		for validator in validators {
			pallet_staking::Validators::<Runtime>::insert(validator, ValidatorPrefs::default());
		}
		let validator_count = validators.len() as u32;
		ValidatorCount::<Runtime>::set(validator_count);

		// now set correct payout count
		EraPayoutInfo::<Runtime>::set(EraPayout {
			era,
			payout_count: validator_count,
			..Default::default()
		});

		// clear pool events
		pool_events_since_last_call();

		let history_depth =
			<<Runtime as pallet_staking::Config>::HistoryDepth as sp_core::Get<u32>>::get();
		for era in 0..(history_depth + 1) {
			pallet_staking::CurrentEra::<Runtime>::put(era);

			EraPayoutInfo::<Runtime>::set(EraPayout {
				era,
				payout_count: validator_count,
				..Default::default()
			});

			// give each pool a bonus so they will be processed
			for pool_id in pool_ids {
				let account = Pools::compute_pool_account_id(pool_id, AccountType::Bonus);
				Balances::make_free_balance_be(&account, 100 * UNIT);
			}

			// the extrinsic succeeds
			Pools::process_payouts(RuntimeOrigin::signed(10), pool_count).unwrap();
			assert_eq!(
				EraPayoutInfo::<Runtime>::get(),
				EraPayout {
					era,
					payout_count: validator_count,
					payouts_processed: true,
					..Default::default()
				}
			);

			// make sure is one EraRewardsProcessed event per pool. We don't care about the
			// contents.
			let events = pool_events_since_last_call();
			assert_eq!(events.len(), 2);
			for event in pool_events_since_last_call() {
				match event {
					Event::EraRewardsProcessed { .. } => {},
					_ => panic!(),
				}
			}
		}
	})
}
