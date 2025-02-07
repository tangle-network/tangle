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
use sp_runtime::Percent;
use sp_staking::StakingAccount;

#[test]
fn unapplied_slash() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let eve = mock_pub_key(EVE);
		let bob = mock_pub_key(BOB);

		// Set up a job call that will result in an invalid submission
		let job_call_id = Services::next_job_call_id();
		assert_ok!(Services::call(
			RuntimeOrigin::signed(eve.clone()),
			service_id,
			KEYGEN_JOB_ID,
			bounded_vec![Field::Uint8(1)],
		));

		// Submit an invalid result that should trigger slashing
		let mut dkg = vec![0u8; 33];
		dkg[32] = 1;
		assert_ok!(Services::submit_result(
			RuntimeOrigin::signed(bob.clone()),
			0,
			job_call_id,
			bounded_vec![Field::from(BoundedVec::try_from(dkg).unwrap())],
		));

		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Slash the operator for the invalid result
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		// Verify the slash was recorded but not yet applied
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		// Verify the correct event was emitted
		System::assert_has_event(RuntimeEvent::Services(crate::Event::UnappliedSlash {
			era: 0,
			index: 0,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * bob_exposed_restake_percentage).mul_floor(
				<Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob),
			),
		}));
	});
}

#[test]
fn slash_account_not_an_operator() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let karen = mock_pub_key(23);

		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		let slash_percent = Percent::from_percent(50);

		// Try to slash an operator that is not active in this service
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				karen.clone(),
				service_id,
				slash_percent
			),
			Error::<Runtime>::OffenderNotOperator
		);
	});
}

#[test]
fn dispute() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let bob = mock_pub_key(BOB);
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Create a slash
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		let era = 0;
		let slash_index = 0;

		// Dispute the slash
		let dispute_origin =
			Services::query_dispute_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		assert_ok!(Services::dispute(
			RuntimeOrigin::signed(dispute_origin.clone()),
			era,
			slash_index
		));

		// Verify the slash was removed
		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 0);

		// Calculate expected slash amount
		let bob_stake = <Runtime as Config>::OperatorDelegationManager::get_operator_stake(&bob);
		let expected_slash_amount =
			(slash_percent * bob_exposed_restake_percentage).mul_floor(bob_stake);

		// Verify the correct event was emitted
		System::assert_has_event(RuntimeEvent::Services(crate::Event::SlashDiscarded {
			era,
			index: slash_index,
			operator: bob.clone(),
			blueprint_id,
			service_id,
			amount: expected_slash_amount,
		}));
	});
}

#[test]
fn dispute_with_unauthorized_origin() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let eve = mock_pub_key(EVE);
		let bob = mock_pub_key(BOB);
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Create a slash
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		let era = 0;
		let slash_index = 0;

		// Try to dispute with an invalid origin
		assert_err!(
			Services::dispute(RuntimeOrigin::signed(eve.clone()), era, slash_index),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn dispute_an_already_applied_slash() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { service_id, .. } = deploy();
		let eve = mock_pub_key(EVE);
		let bob = mock_pub_key(BOB);
		let slash_percent = Percent::from_percent(50);
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Create a slash
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			bob.clone(),
			service_id,
			slash_percent
		));

		assert_eq!(UnappliedSlashes::<Runtime>::iter_keys().collect::<Vec<_>>().len(), 1);

		let era = 0;
		let slash_index = 0;

		// Simulate a slash being applied by removing it
		UnappliedSlashes::<Runtime>::remove(era, slash_index);

		// Try to dispute an already applied slash
		assert_err!(
			Services::dispute(RuntimeOrigin::signed(eve.clone()), era, slash_index),
			Error::<Runtime>::UnappliedSlashNotFound
		);
	});
}

#[test]
fn test_slash_with_multiple_asset_types() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);

		// Setup native stake
		let native_stake = 10_000;
		assert_ok!(Services::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			native_stake,
			Default::default(),
		));

		// Setup ERC20 stake (USDC)
		let usdc_stake = 5_000;
		create_and_mint_tokens(USDC, delegator.clone(), usdc_stake);
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(delegator.clone()),
			Asset::Custom(USDC),
			usdc_stake,
			None,
			None,
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			Asset::Custom(USDC),
			usdc_stake,
			Default::default(),
		));

		// Setup another ERC20 stake (WETH)
		let weth_stake = 2_000;
		create_and_mint_tokens(WETH, delegator.clone(), weth_stake);
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(delegator.clone()),
			Asset::Custom(WETH),
			weth_stake,
			None,
			None,
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			Asset::Custom(WETH),
			weth_stake,
			Default::default(),
		));

		// Apply slash
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();
		let slash_percent = Percent::from_percent(50);

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			slash_percent
		));

		// Verify native stake slash
		let native_stake_after =
			Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap().active;
		assert_eq!(native_stake_after, native_stake / 2);

		// Verify USDC stake slash
		let usdc_balance_after = Assets::balance(USDC, delegator.clone());
		assert_eq!(usdc_balance_after, usdc_stake / 2);

		// Verify WETH stake slash
		let weth_balance_after = Assets::balance(WETH, delegator.clone());
		assert_eq!(weth_balance_after, weth_stake / 2);

		// Verify events for each asset type
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			crate::Event::NominationSlashed {
				delegator: delegator.clone(),
				operator: operator.clone(),
				amount: native_stake / 2,
			},
		));

		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			crate::Event::DelegationSlashed {
				delegator: delegator.clone(),
				operator: operator.clone(),
				asset: Asset::Custom(USDC),
				amount: usdc_stake / 2,
			},
		));

		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			crate::Event::DelegationSlashed {
				delegator: delegator.clone(),
				operator: operator.clone(),
				asset: Asset::Custom(WETH),
				amount: weth_stake / 2,
			},
		));
	});
}

#[test]
fn test_slash_with_concurrent_delegations() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let operator = mock_pub_key(BOB);
		let delegator1 = mock_pub_key(CHARLIE);
		let delegator2 = mock_pub_key(DAVE);

		// Initial setup
		let initial_stake = 10_000;
		assert_ok!(Services::delegate_nomination(
			RuntimeOrigin::signed(delegator1.clone()),
			operator.clone(),
			initial_stake,
			Default::default(),
		));

		// Start a slash
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();
		let slash_percent = Percent::from_percent(50);

		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			slash_percent
		));

		// Add new delegation during slash processing
		let new_stake = 5_000;
		assert_ok!(Services::delegate_nomination(
			RuntimeOrigin::signed(delegator2.clone()),
			operator.clone(),
			new_stake,
			Default::default(),
		));

		// Remove some delegation during slash processing
		assert_ok!(Services::schedule_nomination_unstake(
			RuntimeOrigin::signed(delegator1.clone()),
			operator.clone(),
			initial_stake / 4,
			Default::default(),
		));

		// Apply the slash
		let slashes: Vec<_> = UnappliedSlashes::<Runtime>::iter().collect();
		for (era, index) in slashes.iter().map(|((era, index), _)| (*era, *index)) {
			assert_ok!(Services::apply_slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				era,
				index
			));
		}

		// Verify final states
		let delegator1_final_stake =
			Staking::ledger(StakingAccount::Stash(delegator1.clone())).unwrap().active;
		let delegator2_final_stake =
			Staking::ledger(StakingAccount::Stash(delegator2.clone())).unwrap().active;

		// Delegator1's stake should reflect both the slash and unstaking
		assert_eq!(
			delegator1_final_stake,
			initial_stake / 2 - initial_stake / 4,
			"Delegator1's stake should be halved by slash and reduced by unstaking"
		);

		// Delegator2's new stake should be unaffected by the slash
		assert_eq!(
			delegator2_final_stake, new_stake,
			"Delegator2's stake should be unaffected as it was added after slash"
		);

		// Verify events
		System::assert_has_event(RuntimeEvent::Services(crate::Event::SlashApplied {
			era: 0,
			index: 0,
			operator: operator.clone(),
			blueprint_id,
			service_id,
			amount: (slash_percent * bob_exposed_restake_percentage).mul_floor(initial_stake),
		}));
	});
}

#[test]
fn test_slash_with_partial_amounts() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);

		// Setup precise stake amounts
		let stake_amount = 10_000;
		assert_ok!(Services::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			stake_amount,
			Default::default(),
		));

		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Test various partial slash percentages
		let slash_percentages = vec![
			Percent::from_percent(33), // 33%
			Percent::from_percent(17), // 17%
			Percent::from_percent(7),  // 7%
		];

		let mut remaining_stake = stake_amount;
		for (index, slash_percent) in slash_percentages.iter().enumerate() {
			assert_ok!(Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				operator.clone(),
				service_id,
				*slash_percent
			));

			// Apply the slash
			assert_ok!(Services::apply_slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				0,
				index as u32
			));

			// Calculate expected remaining stake
			let slash_amount =
				(*slash_percent * bob_exposed_restake_percentage).mul_floor(remaining_stake);
			remaining_stake = remaining_stake.saturating_sub(slash_amount);

			// Verify current stake
			let current_stake =
				Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap().active;
			assert_eq!(
				current_stake, remaining_stake,
				"Stake should be reduced by exact slash percentage"
			);

			// Verify events
			System::assert_has_event(RuntimeEvent::Services(crate::Event::SlashApplied {
				era: 0,
				index: index as u32,
				operator: operator.clone(),
				blueprint_id,
				service_id,
				amount: slash_amount,
			}));
		}

		// Verify final stake is exactly what we expect after all partial slashes
		let final_stake = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap().active;
		let expected_final_stake = stake_amount
			.saturating_sub(
				(Percent::from_percent(33) * bob_exposed_restake_percentage)
					.mul_floor(stake_amount),
			)
			.saturating_sub(
				(Percent::from_percent(17) * bob_exposed_restake_percentage)
					.mul_floor(stake_amount),
			)
			.saturating_sub(
				(Percent::from_percent(7) * bob_exposed_restake_percentage).mul_floor(stake_amount),
			);
		assert_eq!(
			final_stake, expected_final_stake,
			"Final stake should reflect all partial slashes precisely"
		);
	});
}

#[test]
fn test_slash_with_zero_stake() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);

		// Register operator but don't delegate any stake
		assert_ok!(Services::register(
			RuntimeOrigin::signed(operator.clone()),
			blueprint_id,
			OperatorPreferences { key: test_ecdsa_key(), price_targets: Default::default() },
			Default::default(),
			0,
		));

		// Attempt to slash operator with zero stake
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();
		let slash_percent = Percent::from_percent(50);

		// Should fail due to insufficient stake
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				operator.clone(),
				service_id,
				slash_percent
			),
			Error::<Runtime>::InsufficientStake
		);

		// Verify no slash events were emitted
		assert!(!System::events()
			.iter()
			.any(|r| matches!(r.event, RuntimeEvent::Services(crate::Event::SlashApplied { .. }))));
	});
}

#[test]
fn test_slash_with_invalid_operator() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let invalid_operator = mock_pub_key(99); // Non-existent operator
		let delegator = mock_pub_key(CHARLIE);

		// Attempt to slash an unregistered operator
		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();
		let slash_percent = Percent::from_percent(50);

		// Should fail because operator is not registered
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				invalid_operator.clone(),
				service_id,
				slash_percent
			),
			Error::<Runtime>::OffenderNotOperator
		);

		// Try to slash with an account that is registered but not as an operator
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				delegator.clone(),
				service_id,
				slash_percent
			),
			Error::<Runtime>::OffenderNotOperator
		);

		// Verify no slash events were emitted
		assert!(!System::events()
			.iter()
			.any(|r| matches!(r.event, RuntimeEvent::Services(crate::Event::SlashApplied { .. }))));
	});
}

#[test]
fn test_slash_with_insufficient_balance() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
		let operator = mock_pub_key(BOB);
		let delegator = mock_pub_key(CHARLIE);

		// Setup a small stake amount
		let stake_amount = 100;
		assert_ok!(Services::delegate_nomination(
			RuntimeOrigin::signed(delegator.clone()),
			operator.clone(),
			stake_amount,
			Default::default(),
		));

		let service = Services::services(service_id).unwrap();
		let slashing_origin =
			Services::query_slashing_origin(&service).map(|(o, _)| o.unwrap()).unwrap();

		// Try to slash more than available stake (200%)
		let excessive_slash = Percent::from_percent(200);
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				operator.clone(),
				service_id,
				excessive_slash
			),
			Error::<Runtime>::InsufficientStakeRemaining
		);

		// Verify original stake remains unchanged
		let final_stake = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap().active;
		assert_eq!(final_stake, stake_amount, "Stake should remain unchanged after failed slash");

		// Try multiple smaller slashes that would exceed total stake
		let slash_percent = Percent::from_percent(60);

		// First slash should succeed
		assert_ok!(Services::slash(
			RuntimeOrigin::signed(slashing_origin.clone()),
			operator.clone(),
			service_id,
			slash_percent
		));

		// Second slash should fail due to insufficient remaining stake
		assert_err!(
			Services::slash(
				RuntimeOrigin::signed(slashing_origin.clone()),
				operator.clone(),
				service_id,
				slash_percent
			),
			Error::<Runtime>::InsufficientStakeRemaining
		);

		// Verify only one slash event was emitted
		let slash_events: Vec<_> = System::events()
			.iter()
			.filter(|r| {
				matches!(r.event, RuntimeEvent::Services(crate::Event::UnappliedSlash { .. }))
			})
			.collect();
		assert_eq!(slash_events.len(), 1, "Should only have one successful slash recorded");
	});
}

#[test]
fn test_slash_with_multiple_services() {
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

		// Apply slashes
		let slashes: Vec<_> = UnappliedSlashes::<Runtime>::iter().collect();
		for (era, index) in slashes.iter().map(|((era, index), _)| (*era, *index)) {
			assert_ok!(Services::apply_slash(
				RuntimeOrigin::signed(slashing_origin1.clone()),
				era,
				index
			));
		}

		// Verify final stake after both slashes
		let final_stake = Staking::ledger(StakingAccount::Stash(delegator.clone())).unwrap().active;
		assert_eq!(final_stake, stake_amount / 4, "Should have 25% remaining after both slashes");

		// Verify events
		let slash_events: Vec<_> = System::events()
			.iter()
			.filter(|r| {
				matches!(r.event, RuntimeEvent::Services(crate::Event::SlashApplied { .. }))
			})
			.collect();
		assert_eq!(slash_events.len(), 2, "Should have two slash events");
	});
}

#[test]
fn test_slash_with_rewards_distribution() {
	new_test_ext(vec![ALICE, BOB, CHARLIE, DAVE, EVE]).execute_with(|| {
		System::set_block_number(1);
		let Deployment { blueprint_id, service_id, bob_exposed_restake_percentage } = deploy();
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

		// Apply the slash
		assert_ok!(Services::apply_slash(RuntimeOrigin::signed(slashing_origin.clone()), 0, 0));

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
		System::assert_has_event(RuntimeEvent::Services(crate::Event::SlashApplied {
			era: 0,
			index: 0,
			operator: operator.clone(),
			blueprint_id,
			service_id,
			amount: (Percent::from_percent(50) * bob_exposed_restake_percentage)
				.mul_floor(pre_slash_balance),
		}));

		// Verify reward rate is proportional to remaining stake
		let post_slash_rewards = final_balance - pre_slash_balance / 2;
		let pre_slash_rewards = pre_slash_balance - stake_amount;

		// The post-slash rewards should be approximately half of pre-slash rewards
		// (allowing for some rounding differences)
		let reward_ratio = post_slash_rewards * 100 / pre_slash_rewards;
		assert!(
			reward_ratio >= 45 && reward_ratio <= 55,
			"Post-slash rewards should be ~50% of pre-slash rewards"
		);
	});
}
