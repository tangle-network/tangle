use super::*;
use crate::mock::RewardImbalance::{Deficit, Surplus};
use crate::{mock::Currency, mock::*, Event};
use frame_support::traits::Currency as CurrencyT;
use frame_support::{assert_err, assert_noop, assert_ok, assert_storage_noop};

#[test]
fn ed_change_causes_reward_deficit() {
	ExtBuilder::default().max_members_per_pool(Some(5)).build_and_execute(|| {
		// original ED
		ExistentialDeposit::set(5);

		// 11 joins the pool
		Currency::make_free_balance_be(&11, 500);
		assert_ok!(Lst::join(RuntimeOrigin::signed(11), 90, 1));

		// new delegator does not have any pending rewards
		assert_eq!(pending_rewards_for_delegator(11, 1), 0);

		// give the pool some rewards
		deposit_rewards(100);

		// all existing delegator has pending rewards
		assert_eq!(pending_rewards_for_delegator(11, 1), 90);
		assert_eq!(pending_rewards_for_delegator(10, 1), 10);
		assert_eq!(reward_imbalance(1), Surplus(0));

		// 12 joins the pool.
		Currency::make_free_balance_be(&12, 500);
		assert_ok!(Lst::join(RuntimeOrigin::signed(12), 100, 1));

		// Current reward balance is committed to last recorded reward counter of
		// the pool before the increase in ED.
		let bonded_pool = BondedPools::<Runtime>::get(1).unwrap();
		let reward_pool = RewardPools::<Runtime>::get(1).unwrap();
		assert_eq!(
			reward_pool.last_recorded_reward_counter,
			reward_pool
				.current_reward_counter(1, bonded_pool.points(), Perbill::zero())
				.unwrap()
				.0
		);

		// reward pool before ED increase and reward counter getting committed.
		let reward_pool_1 = RewardPools::<Runtime>::get(1).unwrap();

		// increase ED from 5 to 50
		ExistentialDeposit::set(50);

		// There is now an expected deficit of ed_diff
		assert_eq!(reward_imbalance(1), Deficit(45));

		// 13 joins the pool which commits the reward counter to reward pool.
		Currency::make_free_balance_be(&13, 500);
		assert_ok!(Lst::join(RuntimeOrigin::signed(13), 100, 1));

		// still a deficit
		assert_eq!(reward_imbalance(1), Deficit(45));

		// reward pool after ED increase
		let reward_pool_2 = RewardPools::<Runtime>::get(1).unwrap();

		// last recorded total payout does not decrease even as ED increases.
		assert_eq!(
			reward_pool_1.last_recorded_total_payouts,
			reward_pool_2.last_recorded_total_payouts
		);

		// Topping up pool decreases deficit
		deposit_rewards(10);
		assert_eq!(reward_imbalance(1), Deficit(35));

		// top up the pool to remove the deficit
		deposit_rewards(35);
		// No deficit anymore
		assert_eq!(reward_imbalance(1), Surplus(0));

		// fix the ed deficit
		assert_ok!(Currency::mint_into(&10, 45));
		assert_ok!(Lst::adjust_pool_deposit(RuntimeOrigin::signed(10), 1));
	});
}

#[test]
fn ed_adjust_fixes_reward_deficit() {
	ExtBuilder::default().max_members_per_pool(Some(5)).build_and_execute(|| {
		// Given: pool has a reward deficit

		// original ED
		ExistentialDeposit::set(5);

		// 11 joins the pool
		Currency::make_free_balance_be(&11, 500);
		assert_ok!(Lst::join(RuntimeOrigin::signed(11), 90, 1));

		// Pool some rewards
		deposit_rewards(100);

		// 12 joins the pool.
		Currency::make_free_balance_be(&12, 500);
		assert_ok!(Lst::join(RuntimeOrigin::signed(12), 10, 1));

		// When: pool ends up in reward deficit
		// increase ED
		ExistentialDeposit::set(50);
		assert_eq!(reward_imbalance(1), Deficit(45));

		// clear events
		pool_events_since_last_call();

		// Then: Anyone can permissionlessly can adjust ED deposit.

		// make sure caller has enough funds..
		assert_ok!(Currency::mint_into(&99, 100));
		let pre_balance = Currency::free_balance(&99);
		// adjust ED
		assert_ok!(Lst::adjust_pool_deposit(RuntimeOrigin::signed(99), 1));
		// depositor's balance should decrease by 45
		assert_eq!(Currency::free_balance(&99), pre_balance - 45);
		assert_eq!(reward_imbalance(1), Surplus(0));

		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::MinBalanceDeficitAdjusted { pool_id: 1, amount: 45 },]
		);

		// Trying to top up again does not work
		assert_err!(
			Lst::adjust_pool_deposit(RuntimeOrigin::signed(10), 1),
			Error::<T>::NothingToAdjust
		);

		// When: ED is decreased and reward account has excess ED frozen
		ExistentialDeposit::set(5);

		// And:: adjust ED deposit is called
		let pre_balance = Currency::free_balance(&100);
		assert_ok!(Lst::adjust_pool_deposit(RuntimeOrigin::signed(100), 1));

		// Then: excess ED is claimed by the caller
		assert_eq!(Currency::free_balance(&100), pre_balance + 45);

		assert_eq!(
			pool_events_since_last_call(),
			vec![Event::MinBalanceExcessAdjusted { pool_id: 1, amount: 45 },]
		);
	});
}

#[test]
fn topping_up_does_not_work_for_pools_with_no_deficit() {
	ExtBuilder::default().max_members_per_pool(Some(5)).build_and_execute(|| {
		// 11 joins the pool
		Currency::make_free_balance_be(&11, 500);
		assert_ok!(Lst::join(RuntimeOrigin::signed(11), 90, 1));

		// Pool some rewards
		deposit_rewards(100);
		assert_eq!(reward_imbalance(1), Surplus(0));

		// Topping up fails
		assert_err!(
			Lst::adjust_pool_deposit(RuntimeOrigin::signed(10), 1),
			Error::<T>::NothingToAdjust
		);
	});
}
