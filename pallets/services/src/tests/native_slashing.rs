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
use pallet_staking::RewardDestination;
use sp_runtime::Percent;
use sp_staking::StakingAccount;

#[test]
fn test_basic_native_restaking_slash() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();

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
		let Deployment { service_id, .. } = deploy();

		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);
		let native_stake = 10_000;
		let regular_stake = 5_000;

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
		let recipient_nominations = Staking::nominators(slash_recipient.clone()).unwrap();
		assert!(recipient_nominations.targets.contains(&operator));

		// Verify regular delegation transfer
		assert_eq!(Assets::balance(USDC, slash_recipient), regular_stake / 2);
	});
}

#[test]
fn test_native_restaking_slash_with_invalid_operator() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
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
fn test_native_restaking_slash_with_multiple_services() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);

		// Deploy first service
		let Deployment { service_id: service_id1, .. } = deploy();

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
			vec![get_security_commitment(WETH, 10), get_security_commitment(TNT, 10)],
		));

		let delegator = mock_pub_key(CHARLIE);
		let stake_amount = 1000;

		// Delegate to both services
		assert_ok!(MultiAssetDelegation::delegate_nomination(
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
		// Verify slash data
		let slashes: Vec<_> = UnappliedSlashes::<Runtime>::iter_prefix(0).collect();
		assert_eq!(slashes.len(), 2);

		// First slash should be 50% from service_id1
		let (_, first_slash) = &slashes[0];
		assert_eq!(first_slash.service_id, service_id1);
		assert_eq!(first_slash.operator, bob);
		assert_eq!(Percent::from_percent(10).mul_floor(first_slash.own), stake_amount / 2); // 50% of exposed stake_amount
		assert_eq!(first_slash.others.len(), 1);
		assert_eq!(first_slash.others[0].0, delegator);
		assert_eq!(first_slash.others[0].2, stake_amount / 2); // 50% of delegator stake

		// Second slash should be 25% from service_id 1
		let (_, second_slash) = &slashes[1];
		assert_eq!(second_slash.service_id, 1);
		assert_eq!(second_slash.operator, bob);
		assert_eq!(second_slash.own, stake_amount / 4); // 25% of stake_amount
		assert_eq!(second_slash.others.len(), 1);
		assert_eq!(second_slash.others[0].0, delegator);
		assert_eq!(second_slash.others[0].2, stake_amount / 4); // 25% of delegator stake

		// Apply slashes
		let slashes: Vec<_> = UnappliedSlashes::<Runtime>::iter_prefix(0).collect();
		for (_, slash) in slashes {
			assert_ok!(Services::apply_slash(slash));
		}

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
		assert_ok!(MultiAssetDelegation::delegate_nomination(
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
fn test_atomic_slashing_operations() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id } = deploy();
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
			amount: (slash_percent * Percent::from_percent(10)).mul_floor(initial_bob_stake),
		}));
		System::assert_has_event(RuntimeEvent::Services(crate::Event::UnappliedSlash {
			era: 0,
			index: 1,
			operator: charlie.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * Percent::from_percent(10)).mul_floor(initial_charlie_stake),
		}));

		// Apply slashes
		let slashes: Vec<_> = UnappliedSlashes::<Runtime>::iter_prefix(0).collect();
		for (_, slash) in slashes {
			assert_ok!(Services::apply_slash(slash));
		}

		// Verify stakes are reduced
		let bob_final_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob);
		let charlie_final_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&charlie);

		let expected_bob_stake = initial_bob_stake
			- (slash_percent * Percent::from_percent(10)).mul_floor(initial_bob_stake);
		let expected_charlie_stake = initial_charlie_stake
			- (slash_percent * Percent::from_percent(10)).mul_floor(initial_charlie_stake);

		assert_eq!(bob_final_stake, expected_bob_stake);
		assert_eq!(charlie_final_stake, expected_charlie_stake);

		// Verify slashes are removed after application
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		// Verify slash applied events
		System::assert_has_event(RuntimeEvent::Services(crate::Event::OperatorSlashed {
			era: 0,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * Percent::from_percent(10)).mul_floor(initial_bob_stake),
		}));
		System::assert_has_event(RuntimeEvent::Services(crate::Event::OperatorSlashed {
			era: 0,
			operator: charlie.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * Percent::from_percent(10)).mul_floor(initial_charlie_stake),
		}));
	});
}

#[test]
fn test_complete_slash_to_zero() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id } = deploy();
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

		// Apply slashes
		let slashes: Vec<_> = UnappliedSlashes::<Runtime>::iter_prefix(0).collect();
		for (_, slash) in slashes {
			assert_ok!(Services::apply_slash(slash));
		}

		// Verify operator stake is zero
		let final_bob_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob);
		assert_eq!(final_bob_stake, 0);

		// Verify all slashes are removed after application
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		// Try to slash again - should succeed but applying the slash will fail due to zero stake
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		// Verify applying the slash fails due to insufficient stake
		let slashes: Vec<_> = UnappliedSlashes::<Runtime>::iter_prefix(0).collect();
		for (_, slash) in slashes {
			assert_err!(Services::apply_slash(slash), Error::<Runtime>::OperatorNotActive);
		}

		// Verify proper events were emitted for each slash
		let remaining_after_first = initial_bob_stake
			- (slash_percent * Percent::from_percent(10)).mul_floor(initial_bob_stake);

		System::assert_has_event(RuntimeEvent::Services(crate::Event::OperatorSlashed {
			era: 0,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * Percent::from_percent(10)).mul_floor(initial_bob_stake),
		}));

		let remaining_after_second = remaining_after_first
			- (slash_percent * Percent::from_percent(10)).mul_floor(remaining_after_first);

		System::assert_has_event(RuntimeEvent::Services(crate::Event::OperatorSlashed {
			era: 0,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * Percent::from_percent(10)).mul_floor(remaining_after_first),
		}));

		System::assert_has_event(RuntimeEvent::Services(crate::Event::OperatorSlashed {
			era: 0,
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
		let Deployment { blueprint_id, service_id } = deploy();
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
			RuntimeOrigin::signed(charlie.clone()),
			bob.clone(),
			Asset::Custom(TNT),
			charlie_unstake_amount
		));

		// Execute unstaking for Dave
		assert_ok!(<Runtime as Config>::OperatorDelegationManager::schedule_delegator_unstake(
			RuntimeOrigin::signed(dave.clone()),
			bob.clone(),
			Asset::Custom(TNT),
			initial_dave_stake
		));
		advance_era();
		assert_ok!(<Runtime as Config>::OperatorDelegationManager::execute_delegator_unstake(
			RuntimeOrigin::signed(dave.clone()),
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

		// Apply slashes
		let slashes: Vec<_> = UnappliedSlashes::<Runtime>::iter_prefix(0).collect();
		for (_, slash) in slashes {
			assert_ok!(Services::apply_slash(slash));
		}

		// Verify Charlie's scheduled unstaking amount is properly slashed
		let charlie_remaining_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&charlie);
		let expected_charlie_stake = initial_charlie_stake
			- (slash_percent * Percent::from_percent(10)).mul_floor(initial_charlie_stake);
		assert_eq!(charlie_remaining_stake, expected_charlie_stake);

		// Verify Charlie's unstaking request is adjusted
		let charlie_unstake_requests = MultiAssetDelegation::delegators(&charlie)
			.map(|metadata| metadata.delegator_unstake_requests)
			.unwrap_or_default();
		let adjusted_unstake_amount = charlie_unstake_amount
			- (slash_percent * Percent::from_percent(10)).mul_floor(charlie_unstake_amount);
		assert_eq!(
			charlie_unstake_requests.iter().find(|r| r.operator == bob).map(|r| r.amount),
			Some(adjusted_unstake_amount)
		);

		// Verify Dave's completed unstaking is not affected
		let dave_final_stake =
			<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&dave);
		assert_eq!(dave_final_stake, 0, "Dave's stake should remain zero after unstaking");

		// Verify proper events are emitted
		System::assert_has_event(RuntimeEvent::Services(crate::Event::OperatorSlashed {
			era: 0,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * Percent::from_percent(10)).mul_floor(initial_bob_stake),
		}));

		// Verify unstaking can still be executed after slash
		advance_era();
		assert_ok!(<Runtime as Config>::OperatorDelegationManager::execute_delegator_unstake(
			RuntimeOrigin::signed(charlie.clone()),
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
