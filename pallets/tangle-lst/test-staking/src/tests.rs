//! Integration tests with the staking pallet

use crate::mock::{Runtime as Test, *};
use frame_support::{
	assert_noop, assert_ok,
	traits::{Currency, OnUnbalanced},
};
use pallet_balances::NegativeImbalance;
use pallet_nomination_pools::{
	AccountType, BondedPool, BondedPools, Error as PoolsError, Event as PoolsEvent, NextPoolId,
	Pallet, PoolId, PoolMember, PoolMutationOf, PoolState, StakingInfo, StakingInformation,
	UnbondingMembers,
};
use pallet_staking::{CurrentEra, Event as StakingEvent, Payee, RewardDestination, ValidatorPrefs};
use sp_runtime::{
	bounded_btree_map,
	traits::{AccountIdConversion, Saturating},
	Perbill,
};
use sp_staking::EraIndex;

// Advances through blocks and increases era every block
pub fn run_to_era(era: EraIndex) {
	use frame_support::pallet_prelude::*;

	while Staking::current_era().unwrap_or(0) < era {
		let block_number = System::block_number();
		let weight = System::block_weight();
		let max_weight = <Test as frame_system::Config>::BlockWeights::get().max_block;
		let remaining_weight = max_weight.saturating_sub(weight.total());

		// Execute on_idle.
		if remaining_weight.all_gt(Weight::zero()) {
			let used_weight = Pools::on_idle(block_number, remaining_weight);

			<frame_system::Pallet<Test>>::register_extra_weight_unchecked(
				used_weight,
				DispatchClass::Mandatory,
			);
		}

		// increase era and advance to next block
		CurrentEra::<Test>::mutate(|era| {
			*era = Some(era.unwrap_or(0) + 1);
		});
		System::set_block_number(block_number + 1);
	}
}

pub const DEFAULT_TOKEN_ID: TokenId = 3521;
const DEFAULT_DURATION: EraIndex = 30;

/// Result of `Pools::compute_pool_account_id(0, AccountType::Bonded)`
pub(crate) const POOL_0_BONDED: AccountId = 114950033592811772221625495405;
/// Result of `Pools::compute_pool_account_id(0, AccountType::Reward)`
pub(crate) const POOL_0_REWARD: AccountId = 194178196107076109815169445741;

#[test]
fn nominator_avg_commission_is_calculated() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::minimum_balance(), 5);
		assert_eq!(Staking::current_era(), None);

		// create the pool, we know this has id 1.
		let pool_id = NextPoolId::<Test>::get();
		let token_id = pool_id as TokenId;
		assert_eq!(pool_id, 0);

		assert_ok!(Pools::create(
			RuntimeOrigin::signed(10),
			DEFAULT_TOKEN_ID,
			50,
			1_000,
			DEFAULT_DURATION,
			Default::default(),
		));

		println!("bonded: {}", Pools::compute_pool_account_id(0, AccountType::Bonded));
		println!("reward: {}", Pools::compute_pool_account_id(0, AccountType::Reward));
		// have the pool nominate.
		assert_ok!(Pools::nominate(RuntimeOrigin::signed(10), pool_id, vec![1, 2]));

		// pool and validator does not have any commission
		assert_eq!(
			<Pools as pallet_nomination_pools::traits::Inspect>::current_commission(token_id)
				.unwrap(),
			Perbill::zero()
		);

		// one validator charges 10% commission
		let validator_prefs =
			ValidatorPrefs { commission: Perbill::from_percent(10), blocked: false };
		pallet_staking::Validators::<Test>::insert(1_u128, validator_prefs.clone());

		// one of the validator charges 10%, so the average should be 5%
		assert_eq!(
			<Pools as pallet_nomination_pools::traits::Inspect>::current_commission(token_id)
				.unwrap(),
			Perbill::from_percent(5)
		);

		// other validator charges 10% commission
		pallet_staking::Validators::<Test>::insert(2_u128, validator_prefs);

		// both validator charges 10%, so the average should be 10%
		assert_eq!(
			<Pools as pallet_nomination_pools::traits::Inspect>::current_commission(token_id)
				.unwrap(),
			Perbill::from_percent(10)
		);

		// the pool adds a 2% commission
		assert_ok!(Pools::mutate(
			RuntimeOrigin::signed(10),
			pool_id,
			PoolMutationOf::<Test> {
				new_commission: ShouldMutate::SomeMutation(Some(Perbill::from_percent(2))),
				..Default::default()
			}
		));

		// the total commission should be 10% + 2%
		assert_eq!(
			<Pools as pallet_nomination_pools::traits::Inspect>::current_commission(token_id)
				.unwrap(),
			Perbill::from_percent(12)
		);
	});
}

#[test]
fn pool_lifecycle_e2e() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::minimum_balance(), 5);
		assert_eq!(Staking::current_era(), None);

		// create the pool, we know this has id 1.
		let pool_id = NextPoolId::<Test>::get();
		assert_eq!(pool_id, 0);
		assert_ok!(Pools::create(
			RuntimeOrigin::signed(10),
			DEFAULT_TOKEN_ID,
			50,
			1_000,
			DEFAULT_DURATION,
			Default::default(),
		));

		// have the pool nominate.
		assert_ok!(Pools::nominate(RuntimeOrigin::signed(10), pool_id, vec![1, 2, 3]));

		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Bonded { stash: POOL_0_BONDED, amount: 50 }]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				PoolsEvent::Created { creator: 10, pool_id, capacity: 1_000 },
				PoolsEvent::Bonded {
					member: Pools::deposit_account_id(pool_id),
					pool_id,
					bonded: 50
				},
				PoolsEvent::Nominated { pool_id, validators: vec![1, 2, 3] }
			]
		);

		// have two members join
		assert_ok!(Pools::bond(RuntimeOrigin::signed(20), pool_id, 10.into()));
		assert_ok!(Pools::bond(RuntimeOrigin::signed(21), pool_id, 10.into()));

		assert_eq!(
			staking_events_since_last_call(),
			vec![
				StakingEvent::Bonded { stash: POOL_0_BONDED, amount: 10 },
				StakingEvent::Bonded { stash: POOL_0_BONDED, amount: 10 },
			]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				PoolsEvent::Bonded { member: 20, pool_id, bonded: 10 },
				PoolsEvent::Bonded { member: 21, pool_id, bonded: 10 },
			]
		);

		// pool goes into destroying
		assert_ok!(Pallet::<Test>::set_state(pool_id, PoolState::Destroying));

		// now the members want to unbond.
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 10));
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(21), pool_id, 21, 10));

		assert_eq!(UnbondingMembers::<Test>::get(pool_id, 20).unwrap().unbonding_eras.len(), 1);
		assert_eq!(Pools::member_points(pool_id, 20), 0);
		assert_eq!(UnbondingMembers::<Test>::get(pool_id, 21).unwrap().unbonding_eras.len(), 1);
		assert_eq!(Pools::member_points(pool_id, 21), 0);

		assert_eq!(
			staking_events_since_last_call(),
			vec![
				StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: 10 },
				StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: 10 },
			]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				PoolsEvent::StateChanged { pool_id, new_state: PoolState::Destroying },
				PoolsEvent::Unbonded { member: 20, pool_id, points: 10, balance: 10, era: 3 },
				PoolsEvent::Unbonded { member: 21, pool_id, points: 10, balance: 10, era: 3 },
			]
		);

		for e in 1..BondingDuration::get() {
			CurrentEra::<Test>::set(Some(e));
			assert_noop!(
				Pools::withdraw_unbonded(RuntimeOrigin::signed(20), pool_id, 20, 0),
				PoolsError::<Test>::CannotWithdrawAny
			);
		}

		// members are now unlocked.
		CurrentEra::<Test>::set(Some(BondingDuration::get()));

		// but members can now withdraw.
		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(20), pool_id, 20, 0));
		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(21), pool_id, 21, 0));
		assert!(UnbondingMembers::<Test>::get(pool_id, 20).is_none());
		assert!(UnbondingMembers::<Test>::get(pool_id, 21).is_none());

		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Withdrawn { stash: POOL_0_BONDED, amount: 20 },]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				PoolsEvent::Withdrawn { member: 20, pool_id, points: 10, balance: 10 },
				PoolsEvent::Withdrawn { member: 21, pool_id, points: 10, balance: 10 },
			]
		);

		// as soon as all members have left, the depositor can try to unbond, but since the
		// min-nominator intention is set, they must chill first.
		assert_ok!(Pools::chill(RuntimeOrigin::signed(10), pool_id));
		assert_ok!(Pools::unbond_deposit(RuntimeOrigin::signed(10), pool_id));

		assert_eq!(
			staking_events_since_last_call(),
			vec![
				StakingEvent::Chilled { stash: POOL_0_BONDED },
				StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: 50 },
			]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![PoolsEvent::Unbonded {
				member: Pools::deposit_account_id(pool_id),
				pool_id,
				points: 50,
				balance: 50,
				era: 6
			}]
		);

		// waiting another bonding duration:
		CurrentEra::<Test>::set(Some(BondingDuration::get() * 2));
		assert_ok!(Pools::withdraw_deposit(RuntimeOrigin::signed(10), pool_id));

		// pools is fully destroyed now.
		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Withdrawn { stash: POOL_0_BONDED, amount: 50 },]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				PoolsEvent::Withdrawn { member: 10, pool_id, points: 50, balance: 50 },
				PoolsEvent::Destroyed { pool_id }
			]
		);
	})
}

#[test]
fn pool_slash_e2e() {
	new_test_ext().execute_with(|| {
		ExistentialDeposit::set(1);
		assert_eq!(Balances::minimum_balance(), 1);
		assert_eq!(Staking::current_era(), None);

		// create the pool, we know this has id 1.
		let pool_id = NextPoolId::<Test>::get();
		assert_eq!(pool_id, 0);

		assert_ok!(Pools::create(
			RuntimeOrigin::signed(10),
			DEFAULT_TOKEN_ID,
			40,
			1_000,
			DEFAULT_DURATION,
			Default::default(),
		));

		Pools::bond(RuntimeOrigin::signed(10), pool_id, 40.into()).unwrap();

		assert_eq!(
			staking_events_since_last_call(),
			vec![
				StakingEvent::Bonded { stash: POOL_0_BONDED, amount: 40 },
				StakingEvent::Bonded { stash: POOL_0_BONDED, amount: 40 },
			]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				PoolsEvent::Created { creator: 10, pool_id, capacity: 1_000 },
				PoolsEvent::Bonded {
					member: Pools::deposit_account_id(pool_id),
					pool_id,
					bonded: 40
				},
				PoolsEvent::Bonded { member: 10, pool_id, bonded: 40 },
			]
		);

		assert_eq!(
			Payee::<Test>::get(POOL_0_BONDED).unwrap(),
			RewardDestination::Account(POOL_0_REWARD)
		);

		// have two members join
		assert_ok!(Pools::bond(RuntimeOrigin::signed(20), pool_id, 20.into()));
		assert_ok!(Pools::bond(RuntimeOrigin::signed(21), pool_id, 20.into()));

		assert_eq!(
			staking_events_since_last_call(),
			vec![
				StakingEvent::Bonded { stash: POOL_0_BONDED, amount: 20 },
				StakingEvent::Bonded { stash: POOL_0_BONDED, amount: 20 }
			]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				PoolsEvent::Bonded { member: 20, pool_id, bonded: 20 },
				PoolsEvent::Bonded { member: 21, pool_id, bonded: 20 },
			]
		);

		// now let's progress a bit.
		CurrentEra::<Test>::set(Some(1));

		// 20 / 80 of the total funds are unlocked, and safe from any further slash.
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 10));
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 10));

		assert_eq!(
			staking_events_since_last_call(),
			vec![
				StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: 10 },
				StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: 10 }
			]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				PoolsEvent::Unbonded { member: 10, pool_id, balance: 10, points: 10, era: 4 },
				PoolsEvent::Unbonded { member: 20, pool_id, balance: 10, points: 10, era: 4 }
			]
		);

		CurrentEra::<Test>::set(Some(2));

		assert_ok!(Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 10));
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, 10));
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(21), pool_id, 21, 10));

		assert_eq!(
			staking_events_since_last_call(),
			vec![
				StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: 10 },
				StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: 10 },
				StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: 10 },
			]
		);

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				PoolsEvent::Unbonded { member: 10, pool_id, balance: 10, points: 10, era: 5 },
				PoolsEvent::Unbonded { member: 20, pool_id, balance: 10, points: 10, era: 5 },
				PoolsEvent::Unbonded { member: 21, pool_id, balance: 10, points: 10, era: 5 },
			]
		);

		// At this point, 20 are safe from slash, 30 are unlocking but vulnerable to slash, and and
		// another 70 are active and vulnerable to slash. Let's slash half of them.
		pallet_staking::slashing::do_slash::<Test>(
			&POOL_0_BONDED,
			50,
			&mut Default::default(),
			&mut Default::default(),
			2, // slash era 2, affects chunks at era 5 onwards.
		);

		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Slashed { staker: POOL_0_BONDED, amount: 50 }]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				// 30 has been slashed to 15 (15 slash)
				PoolsEvent::UnbondingPoolSlashed { pool_id, era: 5, balance: 15 },
				// 30 has been slashed to 15 (15 slash)
				PoolsEvent::PoolSlashed { pool_id, balance: 35 }
			]
		);

		CurrentEra::<Test>::set(Some(3));
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(21), pool_id, 21, 10));

		assert_eq!(
			UnbondingMembers::<Test>::get(pool_id, 21).unwrap(),
			PoolMember {
				// the 10 points unlocked just now correspond to 5 points in the unbond pool.
				unbonding_eras: bounded_btree_map!(5 => 10, 6 => 5)
			}
		);
		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: 5 }]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![PoolsEvent::Unbonded { member: 21, pool_id, balance: 5, points: 5, era: 6 }]
		);

		// now we start withdrawing. we do it all at once, at era 6 where 20 and 21 are fully free.
		CurrentEra::<Test>::set(Some(6));

		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(20), pool_id, 20, 0));

		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(21), pool_id, 21, 0));

		assert_eq!(
			pool_events_since_last_call(),
			vec![
				// 20 had unbonded 10 safely, and 10 got slashed by half.
				PoolsEvent::Withdrawn { member: 20, pool_id, balance: 10 + 5, points: 20 },
				// 21 unbonded all of it after the slash
				PoolsEvent::Withdrawn { member: 21, pool_id, balance: 5 + 5, points: 15 },
			]
		);
		assert_eq!(
			staking_events_since_last_call(),
			// a 10 (un-slashed) + 10/2 (slashed) balance from 10 has also been unlocked
			vec![StakingEvent::Withdrawn { stash: POOL_0_BONDED, amount: 15 + 10 + 15 }]
		);

		// now, finally, we can unbond the member 10 further than their current limit.
		assert_ok!(Pallet::<Test>::set_state(pool_id, PoolState::Destroying));
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(10), pool_id, 10, 20));

		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: 10 }]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				PoolsEvent::StateChanged { pool_id, new_state: PoolState::Destroying },
				PoolsEvent::Unbonded { member: 10, pool_id, points: 10, balance: 10, era: 9 }
			]
		);

		CurrentEra::<Test>::set(Some(9));
		assert_eq!(
			UnbondingMembers::<Test>::get(pool_id, 10).unwrap(),
			PoolMember { unbonding_eras: bounded_btree_map!(4 => 10, 5 => 10, 9 => 10) }
		);
		// withdraw the member 10, they should lose 12 balance in total due to slash.
		assert_ok!(Pools::withdraw_unbonded(RuntimeOrigin::signed(10), pool_id, 10, 0));

		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Withdrawn { stash: POOL_0_BONDED, amount: 10 }]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![PoolsEvent::Withdrawn { member: 10, pool_id, balance: 10 + 15, points: 30 },]
		);

		// now unbond the deposit
		assert_ok!(Pools::unbond_deposit(RuntimeOrigin::signed(10), pool_id));

		// and withdraw it
		CurrentEra::<Test>::set(Some(12));
		assert_ok!(Pools::withdraw_deposit(RuntimeOrigin::signed(10), pool_id));

		assert_eq!(
			staking_events_since_last_call(),
			vec![
				StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: 20 }, /* half of the
				                                                              * deposit is
				                                                              * slashed */
				StakingEvent::Withdrawn { stash: POOL_0_BONDED, amount: 20 }
			]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				PoolsEvent::Unbonded {
					member: Pools::deposit_account_id(pool_id),
					pool_id,
					balance: 20,
					points: 20,
					era: 12
				},
				PoolsEvent::Withdrawn { member: 10, pool_id, balance: 20, points: 20 },
				PoolsEvent::Destroyed { pool_id }
			]
		);
	});
}

#[test]
fn pool_slash_proportional() {
	// a typical example where 3 pool members unbond in era 99, 100, and 101, and a slash that
	// happened in era 100 should only affect the latter two.
	new_test_ext().execute_with(|| {
		ExistentialDeposit::set(1);
		BondingDuration::set(28);
		assert_eq!(Balances::minimum_balance(), 1);
		assert_eq!(Staking::current_era(), None);

		// create the pool, we know this has id 0.
		let pool_id = NextPoolId::<Test>::get();
		assert_eq!(pool_id, 0);
		assert_ok!(Pools::create(
			RuntimeOrigin::signed(10),
			DEFAULT_TOKEN_ID,
			40,
			1_000,
			DEFAULT_DURATION,
			Default::default(),
		));

		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Bonded { stash: POOL_0_BONDED, amount: 40 }]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				PoolsEvent::Created { creator: 10, pool_id, capacity: 1_000 },
				PoolsEvent::Bonded {
					member: Pools::deposit_account_id(pool_id),
					pool_id,
					bonded: 40
				},
			]
		);

		// have two members join
		let bond = 20;
		assert_ok!(Pools::bond(RuntimeOrigin::signed(20), pool_id, bond.into()));
		assert_ok!(Pools::bond(RuntimeOrigin::signed(21), pool_id, bond.into()));
		assert_ok!(Pools::bond(RuntimeOrigin::signed(22), pool_id, bond.into()));

		assert_eq!(
			staking_events_since_last_call(),
			vec![
				StakingEvent::Bonded { stash: POOL_0_BONDED, amount: bond },
				StakingEvent::Bonded { stash: POOL_0_BONDED, amount: bond },
				StakingEvent::Bonded { stash: POOL_0_BONDED, amount: bond },
			]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				PoolsEvent::Bonded { member: 20, pool_id, bonded: bond },
				PoolsEvent::Bonded { member: 21, pool_id, bonded: bond },
				PoolsEvent::Bonded { member: 22, pool_id, bonded: bond },
			]
		);

		// now let's progress a lot.
		CurrentEra::<Test>::set(Some(99));

		// and unbond
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, bond));

		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: bond },]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![PoolsEvent::Unbonded {
				member: 20,
				pool_id,
				balance: bond,
				points: bond,
				era: 127
			}]
		);

		CurrentEra::<Test>::set(Some(100));
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(21), pool_id, 21, bond));
		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: bond },]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![PoolsEvent::Unbonded {
				member: 21,
				pool_id,
				balance: bond,
				points: bond,
				era: 128
			}]
		);

		CurrentEra::<Test>::set(Some(101));
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(22), pool_id, 22, bond));
		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: bond },]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![PoolsEvent::Unbonded {
				member: 22,
				pool_id,
				balance: bond,
				points: bond,
				era: 129
			}]
		);

		// Apply a slash that happened in era 100. This is typically applied with a delay.
		// Of the total 100, 50 is slashed.
		assert_eq!(BondedPool::<Test>::get(pool_id).unwrap().points(), 40);
		pallet_staking::slashing::do_slash::<Test>(
			&POOL_0_BONDED,
			50,
			&mut Default::default(),
			&mut Default::default(),
			100,
		);

		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Slashed { staker: POOL_0_BONDED, amount: 50 }]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				// This era got slashed 12.5, which rounded up to 13.
				PoolsEvent::UnbondingPoolSlashed { pool_id, era: 128, balance: 7 },
				// This era got slashed 12 instead of 12.5 because an earlier chunk got 0.5 more
				// slashed, and 12 is all the remaining slash
				PoolsEvent::UnbondingPoolSlashed { pool_id, era: 129, balance: 8 },
				// Bonded pool got slashed for 25, remaining 15 in it.
				PoolsEvent::PoolSlashed { pool_id, balance: 15 }
			]
		);
	});
}

#[test]
fn pool_slash_non_proportional_only_bonded_pool() {
	// A typical example where a pool member unbonds in era 99, and they can get away with a slash
	// that happened in era 100, as long as the pool has enough active bond to cover the slash. If
	// everything else in the slashing/staking system works, this should always be the case.
	// Nonetheless, `ledger.slash` has been written such that it will slash greedily from any chunk
	// if it runs out of chunks that it thinks should be affected by the slash.
	new_test_ext().execute_with(|| {
		ExistentialDeposit::set(1);
		BondingDuration::set(28);
		assert_eq!(Balances::minimum_balance(), 1);
		assert_eq!(Staking::current_era(), None);

		// create the pool, we know this has id 0.
		let pool_id = NextPoolId::<Test>::get();
		assert_ok!(Pools::create(
			RuntimeOrigin::signed(10),
			DEFAULT_TOKEN_ID,
			40,
			1_000,
			DEFAULT_DURATION,
			Default::default(),
		));
		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Bonded { stash: POOL_0_BONDED, amount: 40 }]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				PoolsEvent::Created { creator: 10, pool_id, capacity: 1_000 },
				PoolsEvent::Bonded {
					member: Pools::deposit_account_id(pool_id),
					pool_id,
					bonded: 40
				},
			]
		);

		// have two members join
		let bond = 20;
		assert_ok!(Pools::bond(RuntimeOrigin::signed(20), pool_id, bond.into()));
		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Bonded { stash: POOL_0_BONDED, amount: bond }]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![PoolsEvent::Bonded { member: 20, pool_id, bonded: bond }]
		);

		// progress and unbond.
		CurrentEra::<Test>::set(Some(99));
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, bond));
		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: bond }]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![PoolsEvent::Unbonded {
				member: 20,
				pool_id,
				balance: bond,
				points: bond,
				era: 127
			}]
		);

		// slash for 30. This will be deducted only from the bonded pool.
		CurrentEra::<Test>::set(Some(100));
		assert_eq!(BondedPool::<Test>::get(pool_id).unwrap().points(), 40);
		pallet_staking::slashing::do_slash::<Test>(
			&POOL_0_BONDED,
			30,
			&mut Default::default(),
			&mut Default::default(),
			100,
		);

		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Slashed { staker: POOL_0_BONDED, amount: 30 }]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![PoolsEvent::PoolSlashed { pool_id, balance: 10 }]
		);
	});
}

#[test]
fn pool_slash_non_proportional_bonded_pool_and_chunks() {
	// An uncommon example where even though some funds are unlocked such that they should not be
	// affected by a slash, we still slash out of them. This should not happen at all. If a
	// nomination has unbonded, from the next era onwards, their exposure will drop, so if an era
	// happens in that era, then their share of that slash should naturally be less, such that only
	// their active ledger stake is enough to compensate it.
	new_test_ext().execute_with(|| {
		ExistentialDeposit::set(1);
		BondingDuration::set(28);
		assert_eq!(Balances::minimum_balance(), 1);
		assert_eq!(Staking::current_era(), None);

		// create the pool, we know this has id 1.
		let pool_id = NextPoolId::<Test>::get();
		assert_ok!(Pools::create(
			RuntimeOrigin::signed(10),
			DEFAULT_TOKEN_ID,
			40,
			1_000,
			DEFAULT_DURATION,
			Default::default(),
		));
		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Bonded { stash: POOL_0_BONDED, amount: 40 }]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				PoolsEvent::Created { creator: 10, pool_id, capacity: 1_000 },
				PoolsEvent::Bonded {
					member: Pools::deposit_account_id(pool_id),
					pool_id,
					bonded: 40
				},
			]
		);

		// have two members join
		let bond = 20;
		assert_ok!(Pools::bond(RuntimeOrigin::signed(20), pool_id, bond.into()));
		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Bonded { stash: POOL_0_BONDED, amount: bond }]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![PoolsEvent::Bonded { member: 20, pool_id, bonded: bond }]
		);

		// progress and unbond.
		CurrentEra::<Test>::set(Some(99));
		assert_ok!(Pools::unbond(RuntimeOrigin::signed(20), pool_id, 20, bond));
		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Unbonded { stash: POOL_0_BONDED, amount: bond }]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![PoolsEvent::Unbonded {
				member: 20,
				pool_id,
				balance: bond,
				points: bond,
				era: 127
			}]
		);

		// slash 50. This will be deducted only from the bonded pool and one of the unbonding pools.
		CurrentEra::<Test>::set(Some(100));
		assert_eq!(BondedPool::<Test>::get(pool_id).unwrap().points(), 40);
		pallet_staking::slashing::do_slash::<Test>(
			&POOL_0_BONDED,
			50,
			&mut Default::default(),
			&mut Default::default(),
			100,
		);

		assert_eq!(
			staking_events_since_last_call(),
			vec![StakingEvent::Slashed { staker: POOL_0_BONDED, amount: 50 }]
		);
		assert_eq!(
			pool_events_since_last_call(),
			vec![
				// out of 20, 10 was taken.
				PoolsEvent::UnbondingPoolSlashed { pool_id, era: 127, balance: 10 },
				// out of 40, all was taken.
				PoolsEvent::PoolSlashed { pool_id, balance: 0 }
			]
		);
	});
}

#[test]
fn join_multiple_pools_e2e() {
	new_test_ext().execute_with(|| {
		assert_eq!(Staking::current_era(), None);

		let ed = Balances::minimum_balance();
		let min_bond = Pools::depositor_min_bond();
		let creator_initial_balance = Balances::free_balance(20);

		let pool_creators = [20, 21, 22];
		let joiners = [30, 31, 32, 33, 34];

		// make balance of all pool creators the same
		for i in pool_creators.iter() {
			Balances::make_free_balance_be(i, creator_initial_balance);
		}

		let total_members = joiners.len() + 1;
		let total_bond = min_bond * total_members as u128;
		let total_reward = total_bond / 2;

		// give joiners some balance to join pools
		for joiner in joiners.iter() {
			Balances::make_free_balance_be(joiner, ed + min_bond * pool_creators.len() as u128);
		}

		// give pool creators NFTs allowing to create pools
		for pool_creator in pool_creators.iter() {
			mint(pool_creator + DEFAULT_TOKEN_ID, *pool_creator);
		}

		let mut pools: Vec<u32> = vec![];

		// create 3 pools by 3 different creators
		for pool_creator in pool_creators.iter() {
			let pool_id = NextPoolId::<Test>::get();

			assert_ok!(Pools::create(
				RuntimeOrigin::signed(*pool_creator),
				pool_creator + DEFAULT_TOKEN_ID,
				min_bond,
				// capacity should allow exactly 5 joiners + 1 initial creator bond
				total_bond,
				DEFAULT_DURATION,
				Default::default(),
			));

			// have the pool nominate
			assert_ok!(Pools::nominate(
				RuntimeOrigin::signed(*pool_creator),
				pool_id,
				vec![1, 2, 3]
			));

			// ensure correct balance of the pool creator
			assert_eq!(
				Balances::free_balance(pool_creator),
				creator_initial_balance - ed * 2 - min_bond
			);

			pools.push(pool_id);
		}

		// join all pools by each joiner
		for joiner in joiners.iter() {
			for pool_id in pools.iter() {
				assert_ok!(Pools::bond(RuntimeOrigin::signed(*joiner), *pool_id, min_bond.into()));
			}

			// ensure the joiner has only ED left in balance
			assert_eq!(Balances::free_balance(joiner), ed);
		}

		// ensure all joiners are in all pools and points are correct
		for pool_id in pools.iter() {
			// ensure the points and member count are correct
			let pool = BondedPool::<Test>::get(*pool_id as PoolId).unwrap();
			assert_eq!(pool.points(), total_bond);

			// ensure pool capacity was reached
			assert_eq!(pool.points(), pool.capacity);

			// inflate reward account by half of `total_bond`
			// giving each member `min_bond` worth of rewards
			let reward_account = Pools::compute_pool_account_id(*pool_id, AccountType::Reward);
			Balances::make_free_balance_be(
				&reward_account,
				Balances::free_balance(reward_account) + total_reward,
			);

			// ensure the reward account has the correct balance
			assert_eq!(Balances::free_balance(reward_account), ed + total_reward);

			for joiner in joiners.iter() {
				// ensure account is a member of the pool
				assert_eq!(Pools::member_points(*pool_id, *joiner), min_bond);
			}
		}

		for joiner in joiners.iter() {
			assert_eq!(Balances::free_balance(joiner), ed);

			for pool_id in pools.iter() {
				// ensure account has correct points
				assert_eq!(Pools::member_points(*pool_id, *joiner), min_bond);
			}
		}

		// ensure each pool reward account has the correct balance (ED + reward of pool creator)
		for pool_id in pools.iter() {
			let reward_account = Pools::compute_pool_account_id(*pool_id, AccountType::Reward);
			assert_eq!(Balances::free_balance(reward_account), 35);

			// ensure the reward account has the correct balance (ED)
			assert_eq!(Balances::free_balance(reward_account), 35);
		}

		// ensure each pool creator has the correct balance
		for (i, pool_creator) in pool_creators.iter().enumerate() {
			assert_eq!(
				Balances::free_balance(pool_creator),
				creator_initial_balance - ed * 2 - min_bond
			);

			// destroy pool
			assert_ok!(Pools::destroy(RuntimeOrigin::signed(*pool_creator), pools[i]));
		}

		run_to_era(DEFAULT_DURATION + 1);

		for (i, pool_id) in pools.iter().enumerate() {
			let pool = BondedPools::<Test>::get(pool_id).unwrap();
			let pool_creator = pool_creators[i];

			assert_eq!(pool.state, PoolState::Destroying);

			// joiners can all unbond
			for joiner in joiners.iter() {
				assert_ok!(Pools::unbond(
					RuntimeOrigin::signed(*joiner),
					*pool_id,
					*joiner,
					min_bond
				));

				// ensure account is no longer a member of the pool and has unbonding eras
				let pool_member = UnbondingMembers::<Test>::get(pool_id, joiner).unwrap();
				assert_eq!(pool_member.unbonding_eras.len(), 1);
				assert_eq!(Pools::member_points(*pool_id, *joiner), 0);
			}

			// ensure pool points are correct (only pool creator bond left)
			let pool = BondedPool::<Test>::get(*pool_id as PoolId).unwrap();
			assert_eq!(pool.points(), min_bond);

			// set last era before bonding period ends
			let current_era = CurrentEra::<Test>::get().unwrap();
			let unbond_era = current_era + BondingDuration::get();

			for e in current_era..unbond_era {
				CurrentEra::<Test>::set(Some(e));

				// ensure any joiner still cannot withdraw unbonded funds
				for joiner in joiners.iter() {
					assert_noop!(
						Pools::withdraw_unbonded(
							RuntimeOrigin::signed(*joiner),
							*pool_id,
							*joiner,
							0
						),
						PoolsError::<Test>::CannotWithdrawAny
					);
				}
			}

			// set final era when bonded funds are unlocked
			CurrentEra::<Test>::set(Some(unbond_era));

			// joiners can now withdraw unbonded funds
			for joiner in joiners.iter() {
				assert_ok!(Pools::withdraw_unbonded(
					RuntimeOrigin::signed(*joiner),
					*pool_id,
					*joiner,
					0
				));

				// ensure account is no longer a member of the pool and has unbonding eras
				assert!(UnbondingMembers::<Test>::get(pool_id, joiner).is_none());
			}

			// as soon as all members have left, the creator can try to unbond, but since the
			// min-nominator intention is set, they must chill first
			assert_ok!(Pools::chill(RuntimeOrigin::signed(pool_creator), *pool_id));

			// deposit can be permissionlessly unbonded
			assert_ok!(Pools::unbond_deposit(RuntimeOrigin::signed(420), *pool_id,));

			// wait another bonding duration
			let current_era = CurrentEra::<Test>::get().unwrap();
			let unbond_era = current_era + BondingDuration::get();
			CurrentEra::<Test>::set(Some(unbond_era));

			// deposit also can be withdrawn permissionlessly
			assert_ok!(Pools::withdraw_deposit(RuntimeOrigin::signed(123), *pool_id,));

			// pool is now fully destroyed
			assert!(BondedPools::<Test>::get(pool_id).is_none());
		}
	})
}

#[test]
fn era_payout_inflation() {
	new_test_ext().execute_with(|| {
		let rate = Perbill::from_percent(10);
		let collator_cut = Perbill::from_percent(15);
		let treasury_cut = Perbill::from_percent(25);

		StakingInformation::<Test>::set(Some(StakingInfo {
			annual_inflation_rate: rate,
			collator_payout_cut: collator_cut,
			treasury_payout_cut: treasury_cut,
		}));

		// this will issue 100 UNITs per payout
		let total_issuance = 175_200_000 * UNIT;

		let epoch_duration = EpochDuration::get();
		let epochs_per_era = SessionsPerEra::get();
		let era_in_blocks = epoch_duration * epochs_per_era as u64;
		let payouts_per_day = (DAYS as u64) / era_in_blocks;

		let daily_payout = rate * total_issuance / 365;
		let payout = daily_payout / payouts_per_day as u128;

		let validator_cut = Perbill::from_percent(100)
			.saturating_sub(collator_cut)
			.saturating_sub(treasury_cut);

		let expected_collator_payout = collator_cut.mul_floor(payout);
		let expected_treasury_payout = treasury_cut.mul_floor(payout);

		let expected_validator_payout = validator_cut.mul_floor(payout);
		let expected_remainder = payout.saturating_sub(expected_validator_payout);

		assert_eq!(validator_payout, expected_validator_payout);
		assert_eq!(remainder, expected_remainder);

		let collator_pool_account: AccountId =
			<Test as pallet_nomination_pools::Config>::CollatorRewardPool::get()
				.into_account_truncating();

		// make sure the balance is zero
		assert_eq!(Balances::free_balance(collator_pool_account), 0);
		assert_eq!(Balances::free_balance(treasury_account), 0);

		let amount = NegativeImbalance::<Test>::new(remainder);
		
		// make sure the balance is updated correctly
		assert_eq!(Balances::free_balance(collator_pool_account), expected_collator_payout);
		assert_eq!(Balances::free_balance(treasury_account), expected_treasury_payout);
	})
}
