// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.

use super::*;
use frame_support::{assert_err, assert_ok};
use pallet_staking::{MaxNominationsOf, RewardDestination};
use sp_runtime::Percent;
use sp_staking::StakingAccount;

#[test]
fn test_basic_native_restaking_slash() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, .. } = deploy();

		// Setup native restaking
		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);
		let stake_amount = 10_000;

		// Setup staking for delegator
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(delegator.clone()),
			stake_amount,
			RewardDestination::Staked,
		));

		// Delegate via native restaking
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			stake_amount / 2, // Delegate half the stake
			Default::default(),
		));

		// Verify initial state
		let staking_ledger = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap();
		assert_eq!(staking_ledger.active, stake_amount);

		// Create and apply slash
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			slash_percent
		));

		// Verify nomination was migrated to slash recipient
		let slash_recipient = SlashRecipient::get();
		let recipient_nominations = Staking::nominators(slash_recipient).unwrap();
		assert!(recipient_nominations.targets.contains(&operator));

		// Original delegator's stake should be reduced
		let updated_staking_ledger =
			Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap();
		assert_eq!(updated_staking_ledger.active, stake_amount / 2);
	});
}

#[test]
fn test_mixed_native_and_regular_delegation_slash() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, .. } = deploy();

		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);
		let native_stake = 10_000;
		let regular_stake = 5_000;

		// Setup staking for native restaking
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(delegator.clone()),
			native_stake,
			RewardDestination::Staked,
		));

		// Setup regular delegation
		create_and_mint_tokens(USDC, delegator.clone(), regular_stake);
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(delegator.clone()),
			Asset::Custom(USDC),
			regular_stake,
			None,
			None,
		));

		// Delegate both native and regular stakes
		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			native_stake / 2,
			Default::default(),
		));

		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			Asset::Custom(USDC),
			regular_stake,
			Default::default(),
		));

		// Apply slash
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			slash_percent
		));

		// Verify native stake migration
		let slash_recipient = SlashRecipient::get();
		let recipient_nominations = Staking::nominators(slash_recipient).unwrap();
		assert!(recipient_nominations.targets.contains(&operator));

		// Verify regular delegation transfer
		assert_eq!(Assets::balance(USDC, slash_recipient), regular_stake / 2);
	});
}

#[test]
fn test_native_restaking_slash_during_unbonding() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, .. } = deploy();

		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);
		let stake_amount = 10_000;

		// Setup and delegate
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(delegator.clone()),
			stake_amount,
			RewardDestination::Staked,
		));

		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			stake_amount,
			Default::default(),
		));

		// Verify initial state
		let initial_ledger = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap();
		assert_eq!(initial_ledger.active, stake_amount);
		assert_eq!(initial_ledger.unlocking.len(), 0);

		// Create multiple unbonding schedules
		let unbond_amount_1 = stake_amount / 4;
		let unbond_amount_2 = stake_amount / 4;

		assert_ok!(MultiAssetDelegation::schedule_nomination_unstake(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			unbond_amount_1,
			Default::default(),
		));

		run_to_block(System::block_number() + 100);

		assert_ok!(MultiAssetDelegation::schedule_nomination_unstake(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			unbond_amount_2,
			Default::default(),
		));

		// Verify pre-slash state
		let pre_slash_ledger = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap();
		assert_eq!(pre_slash_ledger.active, stake_amount - unbond_amount_1 - unbond_amount_2);
		assert_eq!(pre_slash_ledger.unlocking.len(), 2);

		// Apply slash during unbonding
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			slash_percent
		));

		// Verify post-slash state
		let post_slash_ledger = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap();

		// Active amount should be reduced by slash percentage
		assert_eq!(
			post_slash_ledger.active,
			(stake_amount - unbond_amount_1 - unbond_amount_2) / 2
		);

		// Each unbonding chunk should be reduced by slash percentage
		let mut found_first_chunk = false;
		let mut found_second_chunk = false;

		for chunk in post_slash_ledger.unlocking.iter() {
			let chunk_amount = chunk.amount();
			if chunk_amount == unbond_amount_1 / 2 {
				found_first_chunk = true;
			} else if chunk_amount == unbond_amount_2 / 2 {
				found_second_chunk = true;
			}
		}

		assert!(found_first_chunk, "First unbonding chunk not found or incorrect amount");
		assert!(found_second_chunk, "Second unbonding chunk not found or incorrect amount");

		// Verify slash recipient received the correct amount
		let slash_recipient = SlashRecipient::get();
		let recipient_nominations = Staking::nominators(slash_recipient.clone()).unwrap();
		assert!(recipient_nominations.targets.contains(&operator));

		// Verify events
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			crate::Event::NominationSlashed {
				delegator: delegator.clone(),
				operator: operator.clone(),
				amount: stake_amount / 2,
			},
		));

		// Verify operator state
		let operator_info = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(
			operator_info.total_stake(),
			(stake_amount - unbond_amount_1 - unbond_amount_2) / 2
		);
	});
}

#[test]
fn test_native_restaking_slash_with_max_nominations() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, .. } = deploy();

		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);
		let stake_amount = 10_000;

		// Setup maximum nominations for slash recipient
		let slash_recipient = SlashRecipient::get();
		let max_nominations = MaxNominationsOf::<Runtime>::get();

		// Fill up recipient's nomination slots
		for i in 0..max_nominations {
			let validator = mock_pub_key(i as u8);
			assert_ok!(Staking::nominate(
				RuntimeOrigin::signed(slash_recipient.clone()),
				vec![validator]
			));
		}

		// Setup and delegate for slashing
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(delegator.clone()),
			stake_amount,
			RewardDestination::Staked,
		));

		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			stake_amount,
			Default::default(),
		));

		// Attempt slash
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Should fail or handle gracefully due to max nominations
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				operator.clone(),
				service_id,
				slash_percent
			),
			Error::<Runtime>::MaxNominationsReached
		);
	});
}

#[test]
fn test_native_restaking_slash_with_multiple_delegators() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, .. } = deploy();

		let operator = mock_pub_key(BOB);
		let delegator1 = mock_pub_key(CHARLIE);
		let delegator2 = mock_pub_key(DAVE);
		let delegator3 = mock_pub_key(EVE);

		let stake_amount_1 = 10_000;
		let stake_amount_2 = 20_000;
		let stake_amount_3 = 15_000;

		// Setup staking for all delegators
		for (delegator, amount) in [
			(delegator1.clone(), stake_amount_1),
			(delegator2.clone(), stake_amount_2),
			(delegator3.clone(), stake_amount_3),
		] {
			assert_ok!(Staking::bond(
				RuntimeOrigin::signed(delegator.clone()),
				amount,
				RewardDestination::Staked,
			));

			assert_ok!(MultiAssetDelegation::delegate_nomination(
				RuntimeOrigin::signed(delegator.clone()),
				operator.clone(),
				amount,
				Default::default(),
			));

			// Verify initial delegation state
			let initial_ledger = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap();
			assert_eq!(initial_ledger.active, amount);
		}

		// Verify operator's initial state
		let initial_operator_info = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(
			initial_operator_info.total_stake(),
			stake_amount_1 + stake_amount_2 + stake_amount_3
		);

		// Apply slash
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			slash_percent
		));

		// Verify slash recipient state
		let slash_recipient = SlashRecipient::get();
		let recipient_nominations = Staking::nominators(slash_recipient.clone()).unwrap();
		assert!(recipient_nominations.targets.contains(&operator));

		// Verify each delegator's final state
		for (delegator, original_amount) in [
			(delegator1.clone(), stake_amount_1),
			(delegator2.clone(), stake_amount_2),
			(delegator3.clone(), stake_amount_3),
		] {
			// Verify ledger state
			let final_ledger = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap();
			assert_eq!(final_ledger.active, original_amount / 2);

			// Verify delegation state in MultiAssetDelegation
			let delegator_info = MultiAssetDelegation::delegators(delegator.clone()).unwrap();
			assert_eq!(
				delegator_info.get_delegation(&operator, Asset::Native).unwrap().amount,
				original_amount / 2
			);

			// Verify events
			System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
				crate::Event::NominationSlashed {
					delegator: delegator.clone(),
					operator: operator.clone(),
					amount: original_amount / 2,
				},
			));
		}

		// Verify operator's final state
		let final_operator_info = MultiAssetDelegation::operator_info(operator.clone()).unwrap();
		assert_eq!(
			final_operator_info.total_stake(),
			(stake_amount_1 + stake_amount_2 + stake_amount_3) / 2
		);

		// Verify total slashed amount in recipient
		let total_slashed = (stake_amount_1 + stake_amount_2 + stake_amount_3) / 2;
		assert_eq!(
			Staking::ledger(StakingAccount::Stash(slash_recipient.clone())).unwrap().active,
			total_slashed
		);
	});
}

#[test]
fn test_native_restaking_slash_across_eras() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, .. } = deploy();

		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);
		let stake_amount = 10_000;

		// Setup initial staking
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(delegator.clone()),
			stake_amount,
			RewardDestination::Staked,
		));

		assert_ok!(MultiAssetDelegation::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			stake_amount,
			Default::default(),
		));

		// Record initial state
		let initial_era = Staking::current_era().unwrap();
		let initial_ledger = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap();

		// Advance era and distribute some rewards
		advance_era();
		distribute_rewards(100); // Distribute 100 tokens as rewards

		// Record pre-slash state after rewards
		let pre_slash_ledger = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap();
		assert!(
			pre_slash_ledger.active > initial_ledger.active,
			"Rewards should have increased stake"
		);

		let rewards_earned = pre_slash_ledger.active - initial_ledger.active;

		// Apply slash during new era
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			slash_percent
		));

		// Verify post-slash state
		let post_slash_ledger = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap();

		// Both original stake and rewards should be slashed
		let expected_remaining = (stake_amount + rewards_earned) / 2;
		assert_eq!(post_slash_ledger.active, expected_remaining);

		// Verify slash recipient received correct amount including reward portion
		let slash_recipient = SlashRecipient::get();
		let recipient_ledger =
			Staking::ledger(StakingAccount::Stash(slash_recipient.clone())).unwrap();
		assert_eq!(recipient_ledger.active, (stake_amount + rewards_earned) / 2);

		// Advance another era and verify reward distribution
		advance_era();
		distribute_rewards(100); // Distribute more rewards

		// Verify final state after another reward distribution
		let final_ledger = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap();
		assert!(
			final_ledger.active > post_slash_ledger.active,
			"Should still earn rewards after slash"
		);

		// Verify operator's exposure in current era
		let current_era = Staking::current_era().unwrap();
		let exposure = Staking::eras_stakers(current_era, &operator);
		assert_eq!(exposure.own, stake_amount / 2);

		// Verify events
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			crate::Event::NominationSlashed {
				delegator: delegator.clone(),
				operator: operator.clone(),
				amount: (stake_amount + rewards_earned) / 2,
			},
		));

		// Verify no pending slash remains
		assert!(Staking::unapplied_slashes(current_era).is_empty());
	});
}

#[test]
fn test_native_restaking_slash_with_zero_stake() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, .. } = deploy();
		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);

		// Try to delegate with zero stake
		assert_err!(
			Services::delegate_nomination(
				RuntimeOrigin::signed(delegator.clone()),
				operator.clone(),
				0,
				Default::default(),
			),
			Error::<Runtime>::BondTooLow
		);

		// Try to slash zero stake
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				operator.clone(),
				service_id,
				Percent::from_percent(0)
			),
			Error::<Runtime>::InvalidAmount
		);
	});
}

#[test]
fn test_native_restaking_slash_with_invalid_operator() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, .. } = deploy();
		let invalid_operator = mock_pub_key(99); // Non-existent operator

		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Try to slash an invalid operator
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				invalid_operator.clone(),
				service_id,
				Percent::from_percent(50)
			),
			Error::<Runtime>::OffenderNotOperator
		);
	});
}

#[test]
fn test_native_restaking_slash_with_insufficient_balance() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, .. } = deploy();
		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);

		// Set up a small stake amount
		let stake_amount = 100;
		assert_ok!(Services::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			stake_amount,
			Default::default(),
		));

		// Try to slash more than available
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Attempt to slash 200% which should fail
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				operator.clone(),
				service_id,
				Percent::from_percent(200)
			),
			Error::<Runtime>::InsufficientStakeRemaining
		);
	});
}

#[test]
fn test_native_restaking_slash_with_multiple_services() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);

		// Deploy first service
		let Deployment { blueprint_id: blueprint_id1, service_id: service_id1, .. } = deploy();

		// Deploy second service
		let alice = mock_pub_key(ALICE);
		let blueprint = cggmp21_blueprint();
		assert_ok!(Services::create_blueprint(RuntimeOrigin::signed(alice.clone()), blueprint));
		let bob = mock_pub_key(BOB);
		assert_ok!(Services::register(
			RuntimeOrigin::signed(bob.clone()),
			1,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		// Request second service
		let eve = mock_pub_key(EVE);
		assert_ok!(Services::request(
			RuntimeOrigin::signed(eve.clone()),
			None,
			1,
			vec![alice.clone()],
			vec![bob.clone()],
			Default::default(),
			vec![get_security_requirement(WETH, &[10, 20])],
			100,
			Asset::Custom(USDC),
			0,
			MembershipModel::Fixed { min_operators: 1 },
		));

		// Approve second service
		assert_ok!(Services::approve(
			RuntimeOrigin::signed(bob.clone()),
			1,
			Percent::from_percent(10),
			vec![get_security_commitment(WETH, 10)],
		));

		let delegator = mock_pub_key(CHARLIE);
		let stake_amount = 1000;

		// Delegate to both services
		assert_ok!(Services::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			bob.clone(),
			stake_amount,
			Default::default(),
		));

		// Slash in first service
		let service1 = Services::services(service_id1).unwrap();
		let slashing_origin1 =
			Services::query_slashing_origin(&service1).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin1.clone()),
			bob.clone(),
			service_id1,
			Percent::from_percent(50)
		));

		// Verify first slash was recorded
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// Slash in second service
		let service2 = Services::services(1).unwrap();
		let slashing_origin2 =
			Services::query_slashing_origin(&service2).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin2.clone()),
			bob.clone(),
			1,
			Percent::from_percent(25)
		));

		// Verify both slashes are recorded
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 2);

		// Advance era to apply slashes
		advance_era();

		// Verify final state after both slashes
		let final_stake = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap().active;
		assert_eq!(final_stake, stake_amount / 4); // Should have 25% remaining after both slashes
	});
}

#[test]
fn test_native_restaking_slash_with_rewards_distribution() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, .. } = deploy();
		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);

		// Set up initial stake
		let stake_amount = 1000;
		assert_ok!(Services::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			stake_amount,
			Default::default(),
		));

		// Distribute some rewards
		let rewards = 500;
		distribute_rewards(rewards);

		// Verify rewards were received
		let pre_slash_balance =
			Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap().active;
		assert!(pre_slash_balance > stake_amount, "Should have earned rewards");

		// Apply slash
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			Percent::from_percent(50)
		));

		// Advance era to apply slash
		advance_era();

		// Distribute more rewards
		distribute_rewards(rewards);

		// Verify final state
		let final_balance =
			Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap().active;

		// Should still be earning rewards on remaining stake
		assert!(
			final_balance > pre_slash_balance / 2,
			"Should continue earning rewards after slash"
		);

		// Verify events
		System::assert_has_event(RuntimeEvent::Services(crate::Event::UnappliedSlash {
			era: 0,
			index: 0,
			operator: operator.clone(),
			blueprint_id,
			service_id,
			amount: pre_slash_balance / 2,
		}));
	});
}

#[test]
fn test_native_restaking_slash_with_concurrent_operations() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, .. } = deploy();
		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);

		// Set up initial stake
		let stake_amount = 1000;
		assert_ok!(Services::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			stake_amount,
			Default::default(),
		));

		// Start unbonding some stake
		assert_ok!(Staking::unbond(RuntimeOrigin::signed(delegator.clone()), stake_amount / 4,));

		// Try to bond more stake concurrently
		assert_ok!(Services::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			stake_amount / 2,
			Default::default(),
		));

		// Apply slash during these operations
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			Percent::from_percent(50)
		));

		// Advance era to apply slash
		advance_era();

		// Try another operation immediately after slash
		assert_ok!(Services::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			100,
			Default::default(),
		));

		// Verify final state is consistent
		let final_stake = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap().active;
		assert!(final_stake > 0, "Should have remaining stake after all operations");

		// Verify events are in correct order
		let events = System::events();
		let mut found_slash = false;
		let mut found_post_slash_delegation = false;

		for event in events {
			match event.event {
				RuntimeEvent::Services(crate::Event::UnappliedSlash { .. }) => {
					found_slash = true;
				},
				RuntimeEvent::Services(crate::Event::NominationDelegated { .. }) => {
					if found_slash {
						found_post_slash_delegation = true;
					}
				},
				_ => {},
			}
		}

		assert!(
			found_slash && found_post_slash_delegation,
			"Events should show correct operation order"
		);
	});
}

#[test]
fn test_atomic_slashing_operations() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);

		// Get initial stakes
		let initial_bob_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob);
		let initial_charlie_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&charlie);

		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Record multiple slashes
		let slash_percent = Percent::from_percent(50);
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			charlie.clone(),
			service_id,
			slash_percent
		));

		// Verify slashes are recorded
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 2);

		// Verify the correct events were emitted
		System::assert_has_event(RuntimeEvent::Services(crate::Event::UnappliedSlash {
			era: 0,
			index: 0,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * bob_exposed_restake_percentage).mul_floor(initial_bob_stake),
		}));
		System::assert_has_event(RuntimeEvent::Services(crate::Event::UnappliedSlash {
			era: 0,
			index: 1,
			operator: charlie.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * bob_exposed_restake_percentage)
				.mul_floor(initial_charlie_stake),
		}));

		// Apply slashes
		let slashes: Vec<_> = UnappliedSlashes::<Runtime>::iter().collect();
		for (era, index) in slashes.iter().map(|((era, index), _)| (*era, *index)) {
			assert_ok!(Services::apply_slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				era,
				index
			));
		}

		// Verify stakes are reduced
		let bob_final_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob);
		let charlie_final_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&charlie);

		let expected_bob_stake = initial_bob_stake
			- (slash_percent * bob_exposed_restake_percentage).mul_floor(initial_bob_stake);
		let expected_charlie_stake = initial_charlie_stake
			- (slash_percent * bob_exposed_restake_percentage).mul_floor(initial_charlie_stake);

		assert_eq!(bob_final_stake, expected_bob_stake);
		assert_eq!(charlie_final_stake, expected_charlie_stake);

		// Verify slashes are removed after application
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		// Verify slash applied events
		System::assert_has_event(RuntimeEvent::Services(crate::Event::SlashApplied {
			era: 0,
			index: 0,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * bob_exposed_restake_percentage).mul_floor(initial_bob_stake),
		}));
		System::assert_has_event(RuntimeEvent::Services(crate::Event::SlashApplied {
			era: 0,
			index: 1,
			operator: charlie.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * bob_exposed_restake_percentage)
				.mul_floor(initial_charlie_stake),
		}));
	});
}

#[test]
fn test_complete_slash_to_zero() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);

		// Get initial stakes
		let initial_bob_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob);
		let initial_charlie_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&charlie);
		assert!(initial_bob_stake > 0, "Bob should have initial stake");
		assert!(initial_charlie_stake > 0, "Charlie should have initial stake");

		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// First slash: 50%
		let slash_percent = Percent::from_percent(50);
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		// Second slash: 50% of remaining
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		// Third slash: Remaining amount
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			Percent::from_percent(100)
		));

		// Verify all slashes are recorded
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 3);

		// Apply all slashes
		let slashes: Vec<_> = UnappliedSlashes::<Runtime>::iter().collect();
		for (era, index) in slashes.iter().map(|((era, index), _)| (*era, *index)) {
			assert_ok!(Services::apply_slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				era,
				index
			));
		}

		// Verify operator stake is zero
		let final_bob_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob);
		assert_eq!(final_bob_stake, 0);

		// Verify all slashes are removed after application
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		// Try to slash again - should fail since stake is zero
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				bob.clone(),
				service_id,
				slash_percent
			),
			Error::<Runtime>::InsufficientStake
		);

		// Verify proper events were emitted for each slash
		let remaining_after_first = initial_bob_stake
			- (slash_percent * bob_exposed_restake_percentage).mul_floor(initial_bob_stake);

		System::assert_has_event(RuntimeEvent::Services(crate::Event::SlashApplied {
			era: 0,
			index: 0,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * bob_exposed_restake_percentage).mul_floor(initial_bob_stake),
		}));

		let remaining_after_second = remaining_after_first
			- (slash_percent * bob_exposed_restake_percentage).mul_floor(remaining_after_first);

		System::assert_has_event(RuntimeEvent::Services(crate::Event::SlashApplied {
			era: 0,
			index: 1,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * bob_exposed_restake_percentage)
				.mul_floor(remaining_after_first),
		}));

		System::assert_has_event(RuntimeEvent::Services(crate::Event::SlashApplied {
			era: 0,
			index: 2,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: remaining_after_second,
		}));
	});
}

#[test]
fn test_slash_with_unstaking_states() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);
		let dave = mock_pub_key(DAVE);

		// Get initial stakes
		let initial_bob_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob);
		let initial_charlie_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&charlie);
		let initial_dave_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&dave);

		// Schedule unstaking for Charlie (half of stake)
		let charlie_unstake_amount = initial_charlie_stake / 2;
		assert_ok!(<Runtime as Config>::OperatorDelegationManager::schedule_delegator_unstake(
			&charlie,
			&bob,
			Asset::Native,
			charlie_unstake_amount
		));

		// Execute unstaking for Dave
		assert_ok!(<Runtime as Config>::OperatorDelegationManager::schedule_delegator_unstake(
			&dave,
			&bob,
			Asset::Native,
			initial_dave_stake
		));
		advance_era();
		assert_ok!(<Runtime as Config>::OperatorDelegationManager::execute_delegator_unstake(
			&dave
		));

		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();
		let slash_percent = Percent::from_percent(50);

		// Attempt to slash operator
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		// Verify slash is recorded
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// Apply the slash
		assert_ok!(Services::apply_slash(RuntimeOrigin::signed(slashing_origin.clone()), 0, 0));

		// Verify Charlie's scheduled unstaking amount is properly slashed
		let charlie_remaining_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&charlie);
		let expected_charlie_stake = initial_charlie_stake
			- (slash_percent * bob_exposed_restake_percentage).mul_floor(initial_charlie_stake);
		assert_eq!(charlie_remaining_stake, expected_charlie_stake);

		// Verify Charlie's unstaking request is adjusted
		let charlie_unstake_requests =
			<Runtime as Config>::OperatorDelegationManager::get_unstaking_requests(&charlie);
		let adjusted_unstake_amount = charlie_unstake_amount
			- (slash_percent * bob_exposed_restake_percentage).mul_floor(charlie_unstake_amount);
		assert_eq!(
			charlie_unstake_requests.iter().find(|r| r.operator == bob).map(|r| r.amount),
			Some(adjusted_unstake_amount)
		);

		// Verify Dave's completed unstaking is not affected
		let dave_final_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&dave);
		assert_eq!(dave_final_stake, 0, "Dave's stake should remain zero after unstaking");

		// Verify proper events are emitted
		System::assert_has_event(RuntimeEvent::Services(crate::Event::SlashApplied {
			era: 0,
			index: 0,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * bob_exposed_restake_percentage).mul_floor(initial_bob_stake),
		}));

		// Verify unstaking can still be executed after slash
		advance_era();
		assert_ok!(<Runtime as Config>::OperatorDelegationManager::execute_delegator_unstake(
			&charlie
		));
		let charlie_final_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&charlie);
		assert_eq!(
			charlie_final_stake,
			expected_charlie_stake - adjusted_unstake_amount,
			"Charlie's final stake should reflect both slash and unstaking"
		);
	});
}

#[test]
fn test_slash_with_failed_processing() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let bob = mock_pub_key(BOB);
		let charlie = mock_pub_key(CHARLIE);

		// Get initial stakes
		let initial_bob_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob);
		let initial_charlie_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&charlie);

		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Record multiple slashes
		let slash_percent = Percent::from_percent(50);
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			charlie.clone(),
			service_id,
			slash_percent
		));

		// Verify slashes are recorded
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 2);

		// Verify the correct events were emitted
		System::assert_has_event(RuntimeEvent::Services(crate::Event::UnappliedSlash {
			era: 0,
			index: 0,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * bob_exposed_restake_percentage).mul_floor(initial_bob_stake),
		}));
		System::assert_has_event(RuntimeEvent::Services(crate::Event::UnappliedSlash {
			era: 0,
			index: 1,
			operator: charlie.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * bob_exposed_restake_percentage)
				.mul_floor(initial_charlie_stake),
		}));

		// Force Charlie's stake to zero to cause a failure
		<Runtime as Config>::OperatorDelegationManager::force_set_stake(&charlie, 0);

		// Attempt to apply slashes - should fail for Charlie
		assert_err!(
			Services::apply_slash(RuntimeOrigin::signed(slashing_origin.clone()), 0, 1),
			Error::<Runtime>::InsufficientStake
		);

		// Verify Bob's slash was not applied (atomic rollback)
		let bob_final_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob);
		assert_eq!(
			bob_final_stake, initial_bob_stake,
			"Bob's stake should remain unchanged after failed slash"
		);

		// Verify slashes are still recorded
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 2);

		// Verify no SlashApplied events were emitted
		assert!(!System::events().iter().any(|record| matches!(
			record.event,
			RuntimeEvent::Services(crate::Event::SlashApplied { .. })
		)));

		// Verify we can still process valid slashes individually
		assert_ok!(Services::apply_slash(RuntimeOrigin::signed(slashing_origin.clone()), 0, 0));

		// Verify Bob's stake is now reduced
		let bob_final_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob);
		let expected_bob_stake = initial_bob_stake
			- (slash_percent * bob_exposed_restake_percentage).mul_floor(initial_bob_stake);
		assert_eq!(bob_final_stake, expected_bob_stake);

		// Verify proper events were emitted for successful slash
		System::assert_has_event(RuntimeEvent::Services(crate::Event::SlashApplied {
			era: 0,
			index: 0,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * bob_exposed_restake_percentage).mul_floor(initial_bob_stake),
		}));
	});
}
