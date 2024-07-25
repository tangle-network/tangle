// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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
use crate::{types::OperatorStatus, CurrentRound, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn join_operator_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.bond, bond_amount);
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
		assert_eq!(operator_info.bond, exact_bond);
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

		// Bond more TNT
		assert_ok!(MultiAssetDelegation::operator_bond_more(
			RuntimeOrigin::signed(1),
			additional_bond
		));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.bond, bond_amount + additional_bond);

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

		// Attempt to bond more without being an operator
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

		// Attempt to bond more with insufficient balance
		assert_noop!(
			MultiAssetDelegation::operator_bond_more(RuntimeOrigin::signed(1), additional_bond),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn schedule_operator_bond_less_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;
		let bond_less_amount = 5_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Schedule bond less
		assert_ok!(MultiAssetDelegation::schedule_operator_bond_less(
			RuntimeOrigin::signed(1),
			bond_less_amount
		));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.request.unwrap().amount, bond_less_amount);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			Event::OperatorBondLessScheduled { who: 1, bond_less_amount },
		));
	});
}

#[test]
fn schedule_operator_bond_less_not_an_operator() {
	new_test_ext().execute_with(|| {
		let bond_less_amount = 5_000;

		// Attempt to schedule bond less without being an operator
		assert_noop!(
			MultiAssetDelegation::schedule_operator_bond_less(
				RuntimeOrigin::signed(1),
				bond_less_amount
			),
			Error::<Test>::NotAnOperator
		);
	});
}

// TO DO
// #[test]
// fn schedule_operator_bond_less_active_services() {
//     new_test_ext().execute_with(|| {
//         let bond_amount = 10_000;
//         let bond_less_amount = 5_000;

//         // Join operator first
//         assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

//         // Manually set the operator's delegation count to simulate active services
//         Operators::<Test>::mutate(1, |operator| {
//             if let Some(ref mut operator) = operator {
//                 operator.delegation_count = 1;
//             }
//         });

//         // Attempt to schedule bond less with active services
//         assert_noop!(
//             MultiAssetDelegation::schedule_operator_bond_less(RuntimeOrigin::signed(1),
// bond_less_amount),             Error::<Test>::ActiveServicesUsingTNT
//         );
//     });
// }

#[test]
fn execute_operator_bond_less_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;
		let bond_less_amount = 5_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Schedule bond less
		assert_ok!(MultiAssetDelegation::schedule_operator_bond_less(
			RuntimeOrigin::signed(1),
			bond_less_amount
		));

		// Set the current round to simulate passage of time
		<CurrentRound<Test>>::put(15);

		// Execute bond less
		assert_ok!(MultiAssetDelegation::execute_operator_bond_less(RuntimeOrigin::signed(1)));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
		assert_eq!(operator_info.bond, bond_amount - bond_less_amount);
		assert_eq!(operator_info.request, None);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			Event::OperatorBondLessExecuted { who: 1 },
		));
	});
}

#[test]
fn execute_operator_bond_less_not_an_operator() {
	new_test_ext().execute_with(|| {
		// Attempt to execute bond less without being an operator
		assert_noop!(
			MultiAssetDelegation::execute_operator_bond_less(RuntimeOrigin::signed(1)),
			Error::<Test>::NotAnOperator
		);
	});
}

#[test]
fn execute_operator_bond_less_no_scheduled_bond_less() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Attempt to execute bond less without scheduling it
		assert_noop!(
			MultiAssetDelegation::execute_operator_bond_less(RuntimeOrigin::signed(1)),
			Error::<Test>::NoScheduledBondLess
		);
	});
}

#[test]
fn execute_operator_bond_less_request_not_satisfied() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;
		let bond_less_amount = 5_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Schedule bond less
		assert_ok!(MultiAssetDelegation::schedule_operator_bond_less(
			RuntimeOrigin::signed(1),
			bond_less_amount
		));

		// Attempt to execute bond less before request is satisfied
		assert_noop!(
			MultiAssetDelegation::execute_operator_bond_less(RuntimeOrigin::signed(1)),
			Error::<Test>::BondLessRequestNotSatisfied
		);
	});
}

#[test]
fn cancel_operator_bond_less_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;
		let bond_less_amount = 5_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Schedule bond less
		assert_ok!(MultiAssetDelegation::schedule_operator_bond_less(
			RuntimeOrigin::signed(1),
			bond_less_amount
		));

		// Cancel bond less
		assert_ok!(MultiAssetDelegation::cancel_operator_bond_less(RuntimeOrigin::signed(1)));

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
fn cancel_operator_bond_less_not_an_operator() {
	new_test_ext().execute_with(|| {
		// Attempt to cancel bond less without being an operator
		assert_noop!(
			MultiAssetDelegation::cancel_operator_bond_less(RuntimeOrigin::signed(1)),
			Error::<Test>::NotAnOperator
		);
	});
}

#[test]
fn cancel_operator_bond_less_no_scheduled_bond_less() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

		// Attempt to cancel bond less without scheduling it
		assert_noop!(
			MultiAssetDelegation::cancel_operator_bond_less(RuntimeOrigin::signed(1)),
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
