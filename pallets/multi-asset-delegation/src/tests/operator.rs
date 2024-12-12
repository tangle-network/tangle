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
use crate::types::DelegatorBlueprintSelection::Fixed;
use crate::{types::OperatorStatus, CurrentRound, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::Percent;

#[test]
fn join_operator_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.stake, bond_amount);
		assert_eq!(operator_info.delegation_count, 0);
		assert_eq!(operator_info.request, None);
		assert_eq!(operator_info.status, OperatorStatus::Active);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(Event::OperatorJoined {
			who: 1,
		}));
	});
}

#[test]
fn join_operator_already_operator() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));
		assert_noop!(
			MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount),
			Error::<Test>::AlreadyOperator
		);
	});
}

#[test]
fn join_operator_insufficient_bond() {
	new_test_ext().execute_with(|| {
		let insufficient_bond = 5_000;

		assert_noop!(
			MultiAssetDelegation::join_operators(RuntimeOrigin::signed(4), insufficient_bond),
			Error::<Test>::BondTooLow
		);
	});
}

#[test]
fn join_operator_insufficient_funds() {
	new_test_ext().execute_with(|| {
		let bond_amount = 15_000; // User 4 has only 5_000

		assert_noop!(
			MultiAssetDelegation::join_operators(RuntimeOrigin::signed(4), bond_amount),
			pallet_balances::Error::<Test, _>::InsufficientBalance
		);
	});
}

#[test]
fn join_operator_minimum_bond() {
	new_test_ext().execute_with(|| {
		let minimum_bond = 10_000;
		let exact_bond = minimum_bond;

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), exact_bond));

		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.stake, exact_bond);
	});
}

#[test]
fn schedule_leave_operator_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		// Schedule leave operators without joining
		assert_noop!(
			MultiAssetDelegation::schedule_leave_operators(RuntimeOrigin::signed(1)),
			Error::<Test>::NotAnOperator
		);

		// Set the current round
		<CurrentRound<Test>>::put(5);

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Schedule leave operators
		assert_ok!(MultiAssetDelegation::schedule_leave_operators(RuntimeOrigin::signed(1)));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.status, OperatorStatus::Leaving(15)); // current_round (5) + leave_operators_delay (10)

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			Event::OperatorLeavingScheduled { who: 1 },
		));
	});
}

#[test]
fn cancel_leave_operator_tests() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Set the current round
		<CurrentRound<Test>>::put(5);

		// Schedule leave operators
		assert_ok!(MultiAssetDelegation::schedule_leave_operators(RuntimeOrigin::signed(1)));

		// Verify operator metadata after cancellation
		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.status, OperatorStatus::Leaving(15)); // current_round (5) + leave_operators_delay (10)

		// Test: Cancel leave operators successfully
		assert_ok!(MultiAssetDelegation::cancel_leave_operators(RuntimeOrigin::signed(1)));

		// Verify operator metadata after cancellation
		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.status, OperatorStatus::Active); // current_round (5) + leave_operators_delay (10)

		// Verify event for cancellation
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			Event::OperatorLeaveCancelled { who: 1 },
		));

		// Test: Cancel leave operators without being in leaving state
		assert_noop!(
			MultiAssetDelegation::cancel_leave_operators(RuntimeOrigin::signed(1)),
			Error::<Test>::NotLeavingOperator
		);

		// Test: Schedule leave operators again
		assert_ok!(MultiAssetDelegation::schedule_leave_operators(RuntimeOrigin::signed(1)));

		// Test: Cancel leave operators without being an operator
		assert_noop!(
			MultiAssetDelegation::cancel_leave_operators(RuntimeOrigin::signed(2)),
			Error::<Test>::NotAnOperator
		);
	});
}

#[test]
fn operator_bond_more_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;
		let additional_bond = 5_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// stake more TNT
		assert_ok!(MultiAssetDelegation::operator_bond_more(
			RuntimeOrigin::signed(1),
			additional_bond
		));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.stake, bond_amount + additional_bond);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(Event::OperatorBondMore {
			who: 1,
			additional_bond,
		}));
	});
}

#[test]
fn operator_bond_more_not_an_operator() {
	new_test_ext().execute_with(|| {
		let additional_bond = 5_000;

		// Attempt to stake more without being an operator
		assert_noop!(
			MultiAssetDelegation::operator_bond_more(RuntimeOrigin::signed(1), additional_bond),
			Error::<Test>::NotAnOperator
		);
	});
}

#[test]
fn operator_bond_more_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;
		let additional_bond = 115_000; // Exceeds available balance

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Attempt to stake more with insufficient balance
		assert_noop!(
			MultiAssetDelegation::operator_bond_more(RuntimeOrigin::signed(1), additional_bond),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn schedule_operator_unstake_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 20_000; // Increased initial bond
		let unstake_amount = 5_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Schedule unstake
		assert_ok!(MultiAssetDelegation::schedule_operator_unstake(
			RuntimeOrigin::signed(1),
			unstake_amount
		));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.request.unwrap().amount, unstake_amount);

		// Verify remaining stake is above minimum
		assert!(operator_info.stake.saturating_sub(unstake_amount) >= MinOperatorBondAmount::get());

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			Event::OperatorBondLessScheduled { who: 1, unstake_amount },
		));
	});
}

// Add test for minimum stake requirement
#[test]
fn schedule_operator_unstake_respects_minimum_stake() {
	new_test_ext().execute_with(|| {
		let bond_amount = 20_000;
		let unstake_amount = 15_000; // Would leave less than minimum required

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Attempt to schedule unstake that would leave less than minimum
		assert_noop!(
			MultiAssetDelegation::schedule_operator_unstake(
				RuntimeOrigin::signed(1),
				unstake_amount
			),
			Error::<Test>::InsufficientStakeRemaining
		);
	});
}

#[test]
fn schedule_operator_unstake_not_an_operator() {
	new_test_ext().execute_with(|| {
		let unstake_amount = 5_000;

		// Attempt to schedule unstake without being an operator
		assert_noop!(
			MultiAssetDelegation::schedule_operator_unstake(
				RuntimeOrigin::signed(1),
				unstake_amount
			),
			Error::<Test>::NotAnOperator
		);
	});
}

// TO DO
// #[test]
// fn schedule_operator_unstake_active_services() {
//     new_test_ext().execute_with(|| {
//         let bond_amount = 10_000;
//         let unstake_amount = 5_000;

//         // Join operator first
//         assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

//         // Manually set the operator's delegation count to simulate active services
//         Operators::<Test>::mutate(1, |operator| {
//             if let Some(ref mut operator) = operator {
//                 operator.delegation_count = 1;
//             }
//         });

//         // Attempt to schedule unstake with active services
//         assert_noop!(
//             MultiAssetDelegation::schedule_operator_unstake(RuntimeOrigin::signed(1),
// unstake_amount),             Error::<Test>::ActiveServicesUsingTNT
//         );
//     });
// }

#[test]
fn execute_operator_unstake_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 20_000;
		let unstake_amount = 5_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Schedule unstake
		assert_ok!(MultiAssetDelegation::schedule_operator_unstake(
			RuntimeOrigin::signed(1),
			unstake_amount
		));

		// Set the current round to simulate passage of time
		<CurrentRound<Test>>::put(15);

		// Execute unstake
		assert_ok!(MultiAssetDelegation::execute_operator_unstake(RuntimeOrigin::signed(1)));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.stake, bond_amount - unstake_amount);
		assert_eq!(operator_info.request, None);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			Event::OperatorBondLessExecuted { who: 1 },
		));
	});
}

#[test]
fn execute_operator_unstake_not_an_operator() {
	new_test_ext().execute_with(|| {
		// Attempt to execute unstake without being an operator
		assert_noop!(
			MultiAssetDelegation::execute_operator_unstake(RuntimeOrigin::signed(1)),
			Error::<Test>::NotAnOperator
		);
	});
}

#[test]
fn execute_operator_unstake_no_scheduled_unstake() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Attempt to execute unstake without scheduling it
		assert_noop!(
			MultiAssetDelegation::execute_operator_unstake(RuntimeOrigin::signed(1)),
			Error::<Test>::NoScheduledBondLess
		);
	});
}

#[test]
fn execute_operator_unstake_request_not_satisfied() {
	new_test_ext().execute_with(|| {
		let bond_amount = 20_000;
		let unstake_amount = 5_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Schedule unstake
		assert_ok!(MultiAssetDelegation::schedule_operator_unstake(
			RuntimeOrigin::signed(1),
			unstake_amount
		));

		// Attempt to execute unstake before request is satisfied
		assert_noop!(
			MultiAssetDelegation::execute_operator_unstake(RuntimeOrigin::signed(1)),
			Error::<Test>::BondLessRequestNotSatisfied
		);
	});
}

#[test]
fn cancel_operator_unstake_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 20_000;
		let unstake_amount = 5_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Schedule unstake
		assert_ok!(MultiAssetDelegation::schedule_operator_unstake(
			RuntimeOrigin::signed(1),
			unstake_amount
		));

		// Cancel unstake
		assert_ok!(MultiAssetDelegation::cancel_operator_unstake(RuntimeOrigin::signed(1)));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.request, None);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			Event::OperatorBondLessCancelled { who: 1 },
		));
	});
}

#[test]
fn cancel_operator_unstake_not_an_operator() {
	new_test_ext().execute_with(|| {
		// Attempt to cancel unstake without being an operator
		assert_noop!(
			MultiAssetDelegation::cancel_operator_unstake(RuntimeOrigin::signed(1)),
			Error::<Test>::NotAnOperator
		);
	});
}

#[test]
fn cancel_operator_unstake_no_scheduled_unstake() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Attempt to cancel unstake without scheduling it
		assert_noop!(
			MultiAssetDelegation::cancel_operator_unstake(RuntimeOrigin::signed(1)),
			Error::<Test>::NoScheduledBondLess
		);
	});
}

#[test]
fn go_offline_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Go offline
		assert_ok!(MultiAssetDelegation::go_offline(RuntimeOrigin::signed(1)));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.status, OperatorStatus::Inactive);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(Event::OperatorWentOffline {
			who: 1,
		}));
	});
}

#[test]
fn go_offline_not_an_operator() {
	new_test_ext().execute_with(|| {
		// Attempt to go offline without being an operator
		assert_noop!(
			MultiAssetDelegation::go_offline(RuntimeOrigin::signed(1)),
			Error::<Test>::NotAnOperator
		);
	});
}

#[test]
fn go_online_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Go offline first
		assert_ok!(MultiAssetDelegation::go_offline(RuntimeOrigin::signed(1)));

		// Go online
		assert_ok!(MultiAssetDelegation::go_online(RuntimeOrigin::signed(1)));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.status, OperatorStatus::Active);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(Event::OperatorWentOnline {
			who: 1,
		}));
	});
}

#[test]
fn slash_operator_success() {
	new_test_ext().execute_with(|| {
		// Setup operator
		let operator_stake = 10_000;
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), operator_stake));

		// Setup delegators
		let delegator_stake = 5_000;
		let asset_id = 1;
		let blueprint_id = 1;

		create_and_mint_tokens(asset_id, 2, delegator_stake);
		mint_tokens(1, asset_id, 3, delegator_stake);

		// Setup delegator with fixed blueprint selection
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(2),
			asset_id,
			delegator_stake
		));
		assert_ok!(MultiAssetDelegation::add_blueprint_id(RuntimeOrigin::signed(2), blueprint_id));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(2),
			1,
			asset_id,
			delegator_stake,
			Fixed(vec![blueprint_id].try_into().unwrap()),
		));

		// Setup delegator with all blueprints
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(3),
			asset_id,
			delegator_stake
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(3),
			1,
			asset_id,
			delegator_stake,
			Fixed(vec![blueprint_id].try_into().unwrap()),
		));

		// Slash 50% of stakes
		let slash_percentage = Percent::from_percent(50);
		assert_ok!(MultiAssetDelegation::slash_operator(&1, blueprint_id, slash_percentage));

		// Verify operator stake was slashed
		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.stake, operator_stake / 2);

		// Verify fixed delegator stake was slashed
		let delegator_2 = MultiAssetDelegation::delegators(2).unwrap();
		let delegation_2 = delegator_2.delegations.iter().find(|d| d.operator == 1).unwrap();
		assert_eq!(delegation_2.amount, delegator_stake / 2);

		// Verify all-blueprints delegator stake was slashed
		let delegator_3 = MultiAssetDelegation::delegators(3).unwrap();
		let delegation_3 = delegator_3.delegations.iter().find(|d| d.operator == 1).unwrap();
		assert_eq!(delegation_3.amount, delegator_stake / 2);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(Event::OperatorSlashed {
			who: 1,
			amount: operator_stake / 2,
		}));
	});
}

#[test]
fn slash_operator_not_an_operator() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			MultiAssetDelegation::slash_operator(&1, 1, Percent::from_percent(50)),
			Error::<Test>::NotAnOperator
		);
	});
}

#[test]
fn slash_operator_not_active() {
	new_test_ext().execute_with(|| {
		// Setup and deactivate operator
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), 10_000));
		assert_ok!(MultiAssetDelegation::go_offline(RuntimeOrigin::signed(1)));

		assert_noop!(
			MultiAssetDelegation::slash_operator(&1, 1, Percent::from_percent(50)),
			Error::<Test>::NotActiveOperator
		);
	});
}

#[test]
fn slash_delegator_fixed_blueprint_not_selected() {
	new_test_ext().execute_with(|| {
		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), 10_000));

		create_and_mint_tokens(1, 2, 10_000);

		// Setup delegator with fixed blueprint selection
		assert_ok!(MultiAssetDelegation::deposit(RuntimeOrigin::signed(2), 1, 5_000));

		assert_ok!(MultiAssetDelegation::add_blueprint_id(RuntimeOrigin::signed(2), 1));

		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(2),
			1,
			1,
			5_000,
			Fixed(vec![2].try_into().unwrap()),
		));

		// Try to slash with unselected blueprint
		assert_noop!(
			MultiAssetDelegation::slash_delegator(&2, &1, 5, Percent::from_percent(50)),
			Error::<Test>::BlueprintNotSelected
		);
	});
}
