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
use crate::{
	types::{DelegatorBlueprintSelection::Fixed, OperatorStatus},
	CurrentRound, Error,
};
use frame_support::{assert_noop, assert_ok};
use sp_keyring::AccountKeyring::{Alice, Bob, Eve};
use tangle_primitives::{
	services::{Asset, UnappliedSlash},
	traits::SlashManager,
};

#[test]
fn join_operator_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			bond_amount
		));

		let operator_info = MultiAssetDelegation::operator_info(Alice.to_account_id()).unwrap();
		assert_eq!(operator_info.stake, bond_amount);
		assert_eq!(operator_info.delegation_count, 0);
		assert_eq!(operator_info.request, None);
		assert_eq!(operator_info.status, OperatorStatus::Active);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(Event::OperatorJoined {
			who: Alice.to_account_id(),
		}));
	});
}

#[test]
fn join_operator_already_operator() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			bond_amount
		));
		assert_noop!(
			MultiAssetDelegation::join_operators(
				RuntimeOrigin::signed(Alice.to_account_id()),
				bond_amount
			),
			Error::<Runtime>::AlreadyOperator
		);
	});
}

#[test]
fn join_operator_insufficient_bond() {
	new_test_ext().execute_with(|| {
		let insufficient_bond = 5_000;

		assert_noop!(
			MultiAssetDelegation::join_operators(
				RuntimeOrigin::signed(Eve.to_account_id()),
				insufficient_bond
			),
			Error::<Runtime>::BondTooLow
		);
	});
}

#[test]
fn join_operator_insufficient_funds() {
	new_test_ext().execute_with(|| {
		let bond_amount = 350_000; // User 4 has only 200_000

		assert_noop!(
			MultiAssetDelegation::join_operators(
				RuntimeOrigin::signed(Alice.to_account_id()),
				bond_amount
			),
			pallet_balances::Error::<Runtime, _>::InsufficientBalance
		);
	});
}

#[test]
fn join_operator_minimum_bond() {
	new_test_ext().execute_with(|| {
		let minimum_bond = 10_000;
		let exact_bond = minimum_bond;

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			exact_bond
		));

		let operator_info = MultiAssetDelegation::operator_info(Alice.to_account_id()).unwrap();
		assert_eq!(operator_info.stake, exact_bond);
	});
}

#[test]
fn schedule_leave_operator_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		// Schedule leave operators without joining
		assert_noop!(
			MultiAssetDelegation::schedule_leave_operators(RuntimeOrigin::signed(
				Alice.to_account_id()
			)),
			Error::<Runtime>::NotAnOperator
		);

		// Set the current round
		<CurrentRound<Runtime>>::put(5);

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			bond_amount
		));

		// Schedule leave operators
		assert_ok!(MultiAssetDelegation::schedule_leave_operators(RuntimeOrigin::signed(
			Alice.to_account_id()
		)));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(Alice.to_account_id()).unwrap();
		assert_eq!(operator_info.status, OperatorStatus::Leaving(15)); // current_round (5) + leave_operators_delay (10)

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			Event::OperatorLeavingScheduled { who: Alice.to_account_id() },
		));
	});
}

#[test]
fn cancel_leave_operator_tests() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			bond_amount
		));

		// Set the current round
		<CurrentRound<Runtime>>::put(5);

		// Schedule leave operators
		assert_ok!(MultiAssetDelegation::schedule_leave_operators(RuntimeOrigin::signed(
			Alice.to_account_id()
		)));

		// Verify operator metadata after cancellation
		let operator_info = MultiAssetDelegation::operator_info(Alice.to_account_id()).unwrap();
		assert_eq!(operator_info.status, OperatorStatus::Leaving(15)); // current_round (5) + leave_operators_delay (10)

		// Test: Cancel leave operators successfully
		assert_ok!(MultiAssetDelegation::cancel_leave_operators(RuntimeOrigin::signed(
			Alice.to_account_id()
		)));

		// Verify operator metadata after cancellation
		let operator_info = MultiAssetDelegation::operator_info(Alice.to_account_id()).unwrap();
		assert_eq!(operator_info.status, OperatorStatus::Active); // current_round (5) + leave_operators_delay (10)

		// Verify event for cancellation
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			Event::OperatorLeaveCancelled { who: Alice.to_account_id() },
		));

		// Test: Cancel leave operators without being in leaving state
		assert_noop!(
			MultiAssetDelegation::cancel_leave_operators(RuntimeOrigin::signed(
				Alice.to_account_id()
			)),
			Error::<Runtime>::NotLeavingOperator
		);

		// Test: Schedule leave operators again
		assert_ok!(MultiAssetDelegation::schedule_leave_operators(RuntimeOrigin::signed(
			Alice.to_account_id()
		)));

		// Test: Cancel leave operators without being an operator
		assert_noop!(
			MultiAssetDelegation::cancel_leave_operators(RuntimeOrigin::signed(
				Bob.to_account_id()
			)),
			Error::<Runtime>::NotAnOperator
		);
	});
}

#[test]
fn operator_bond_more_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;
		let additional_bond = 5_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			bond_amount
		));

		// stake more TNT
		assert_ok!(MultiAssetDelegation::operator_bond_more(
			RuntimeOrigin::signed(Alice.to_account_id()),
			additional_bond
		));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(Alice.to_account_id()).unwrap();
		assert_eq!(operator_info.stake, bond_amount + additional_bond);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(Event::OperatorBondMore {
			who: Alice.to_account_id(),
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
			MultiAssetDelegation::operator_bond_more(
				RuntimeOrigin::signed(Alice.to_account_id()),
				additional_bond
			),
			Error::<Runtime>::NotAnOperator
		);
	});
}

#[test]
fn operator_bond_more_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;
		let additional_bond = 1_150_000; // Exceeds available balance

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			bond_amount
		));

		// Attempt to stake more with insufficient balance
		assert_noop!(
			MultiAssetDelegation::operator_bond_more(
				RuntimeOrigin::signed(Alice.to_account_id()),
				additional_bond
			),
			pallet_balances::Error::<Runtime>::InsufficientBalance
		);
	});
}

#[test]
fn schedule_operator_unstake_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 20_000; // Increased initial bond
		let unstake_amount = 5_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			bond_amount
		));

		// Schedule unstake
		assert_ok!(MultiAssetDelegation::schedule_operator_unstake(
			RuntimeOrigin::signed(Alice.to_account_id()),
			unstake_amount
		));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(Alice.to_account_id()).unwrap();
		assert_eq!(operator_info.request.unwrap().amount, unstake_amount);

		// Verify remaining stake is above minimum
		assert!(
			operator_info.stake.saturating_sub(unstake_amount)
				>= MinOperatorBondAmount::get().into()
		);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			Event::OperatorBondLessScheduled { who: Alice.to_account_id(), unstake_amount },
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
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			bond_amount
		));

		// Attempt to schedule unstake that would leave less than minimum
		assert_noop!(
			MultiAssetDelegation::schedule_operator_unstake(
				RuntimeOrigin::signed(Alice.to_account_id()),
				unstake_amount
			),
			Error::<Runtime>::InsufficientStakeRemaining
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
				RuntimeOrigin::signed(Alice.to_account_id()),
				unstake_amount
			),
			Error::<Runtime>::NotAnOperator
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
//         assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(Alice.
// to_account_id()), bond_amount));

//         // Manually set the operator's delegation count to simulate active services
//         Operators::<Runtime>::mutate(1, |operator| {
//             if let Some(ref mut operator) = operator {
//                 operator.delegation_count = 1;
//             }
//         });

//         // Attempt to schedule unstake with active services
//         assert_noop!(
//
// MultiAssetDelegation::schedule_operator_unstake(RuntimeOrigin::signed(Alice.to_account_id()),
// unstake_amount),             Error::<Runtime>::ActiveServicesUsingTNT
//         );
//     });
// }

#[test]
fn execute_operator_unstake_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 20_000;
		let unstake_amount = 5_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			bond_amount
		));

		// Schedule unstake
		assert_ok!(MultiAssetDelegation::schedule_operator_unstake(
			RuntimeOrigin::signed(Alice.to_account_id()),
			unstake_amount
		));

		// Set the current round to simulate passage of time
		<CurrentRound<Runtime>>::put(15);

		let reserved_balance = Balances::reserved_balance(Alice.to_account_id());
		// Execute unstake
		assert_ok!(MultiAssetDelegation::execute_operator_unstake(RuntimeOrigin::signed(
			Alice.to_account_id()
		)));

		let reserved_balance_after = Balances::reserved_balance(Alice.to_account_id());
		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(Alice.to_account_id()).unwrap();
		assert_eq!(operator_info.stake, bond_amount - unstake_amount);
		assert_eq!(operator_info.request, None);
		assert_eq!(reserved_balance - reserved_balance_after, unstake_amount);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			Event::OperatorBondLessExecuted { who: Alice.to_account_id() },
		));
	});
}

#[test]
fn execute_operator_unstake_not_an_operator() {
	new_test_ext().execute_with(|| {
		// Attempt to execute unstake without being an operator
		assert_noop!(
			MultiAssetDelegation::execute_operator_unstake(RuntimeOrigin::signed(
				Alice.to_account_id()
			)),
			Error::<Runtime>::NotAnOperator
		);
	});
}

#[test]
fn execute_operator_unstake_no_scheduled_unstake() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			bond_amount
		));

		// Attempt to execute unstake without scheduling it
		assert_noop!(
			MultiAssetDelegation::execute_operator_unstake(RuntimeOrigin::signed(
				Alice.to_account_id()
			)),
			Error::<Runtime>::NoScheduledBondLess
		);
	});
}

#[test]
fn execute_operator_unstake_request_not_satisfied() {
	new_test_ext().execute_with(|| {
		let bond_amount = 20_000;
		let unstake_amount = 5_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			bond_amount
		));

		// Schedule unstake
		assert_ok!(MultiAssetDelegation::schedule_operator_unstake(
			RuntimeOrigin::signed(Alice.to_account_id()),
			unstake_amount
		));

		// Attempt to execute unstake before request is satisfied
		assert_noop!(
			MultiAssetDelegation::execute_operator_unstake(RuntimeOrigin::signed(
				Alice.to_account_id()
			)),
			Error::<Runtime>::BondLessRequestNotSatisfied
		);
	});
}

#[test]
fn cancel_operator_unstake_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 20_000;
		let unstake_amount = 5_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			bond_amount
		));

		// Schedule unstake
		assert_ok!(MultiAssetDelegation::schedule_operator_unstake(
			RuntimeOrigin::signed(Alice.to_account_id()),
			unstake_amount
		));

		// Cancel unstake
		assert_ok!(MultiAssetDelegation::cancel_operator_unstake(RuntimeOrigin::signed(
			Alice.to_account_id()
		)));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(Alice.to_account_id()).unwrap();
		assert_eq!(operator_info.request, None);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(
			Event::OperatorBondLessCancelled { who: Alice.to_account_id() },
		));
	});
}

#[test]
fn cancel_operator_unstake_not_an_operator() {
	new_test_ext().execute_with(|| {
		// Attempt to cancel unstake without being an operator
		assert_noop!(
			MultiAssetDelegation::cancel_operator_unstake(RuntimeOrigin::signed(
				Alice.to_account_id()
			)),
			Error::<Runtime>::NotAnOperator
		);
	});
}

#[test]
fn cancel_operator_unstake_no_scheduled_unstake() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			bond_amount
		));

		// Attempt to cancel unstake without scheduling it
		assert_noop!(
			MultiAssetDelegation::cancel_operator_unstake(RuntimeOrigin::signed(
				Alice.to_account_id()
			)),
			Error::<Runtime>::NoScheduledBondLess
		);
	});
}

#[test]
fn go_offline_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			bond_amount
		));

		// Go offline
		assert_ok!(MultiAssetDelegation::go_offline(RuntimeOrigin::signed(Alice.to_account_id())));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(Alice.to_account_id()).unwrap();
		assert_eq!(operator_info.status, OperatorStatus::Inactive);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(Event::OperatorWentOffline {
			who: Alice.to_account_id(),
		}));
	});
}

#[test]
fn go_offline_not_an_operator() {
	new_test_ext().execute_with(|| {
		// Attempt to go offline without being an operator
		assert_noop!(
			MultiAssetDelegation::go_offline(RuntimeOrigin::signed(Alice.to_account_id())),
			Error::<Runtime>::NotAnOperator
		);
	});
}

#[test]
fn go_online_success() {
	new_test_ext().execute_with(|| {
		let bond_amount = 10_000;

		// Join operator first
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			bond_amount
		));

		// Go offline first
		assert_ok!(MultiAssetDelegation::go_offline(RuntimeOrigin::signed(Alice.to_account_id())));

		// Go online
		assert_ok!(MultiAssetDelegation::go_online(RuntimeOrigin::signed(Alice.to_account_id())));

		// Verify operator metadata
		let operator_info = MultiAssetDelegation::operator_info(Alice.to_account_id()).unwrap();
		assert_eq!(operator_info.status, OperatorStatus::Active);

		// Verify event
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(Event::OperatorWentOnline {
			who: Alice.to_account_id(),
		}));
	});
}

#[test]
fn slash_operator_success() {
	new_test_ext().execute_with(|| {
		// Setup operator
		let operator_stake = 10_000;
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			operator_stake
		));

		// Setup delegators with different assets and blueprint selections
		let delegator1_stake = 5_000;
		let delegator2_stake = 3_000;
		let asset1 = Asset::Custom(1);
		let asset2 = Asset::Custom(2);
		let blueprint_id = 1;
		let service_id = 42;

		// Setup first delegator with asset1 and selected blueprint
		create_and_mint_tokens(1, Bob.to_account_id(), delegator1_stake);
		mint_tokens(Bob.to_account_id(), 1, Bob.to_account_id(), delegator1_stake);

		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(Bob.to_account_id()),
			asset1,
			delegator1_stake,
			None,
			None
		));

		assert_ok!(MultiAssetDelegation::add_blueprint_id(
			RuntimeOrigin::signed(Bob.to_account_id()),
			blueprint_id
		));

		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(Bob.to_account_id()),
			Alice.to_account_id(),
			asset1,
			delegator1_stake,
			Fixed(vec![blueprint_id].try_into().unwrap()),
		));

		// Setup second delegator with asset2 but without selecting the blueprint
		create_and_mint_tokens(2, Eve.to_account_id(), delegator2_stake);
		mint_tokens(Eve.to_account_id(), 2, Eve.to_account_id(), delegator2_stake);

		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(Eve.to_account_id()),
			asset2,
			delegator2_stake,
			None,
			None
		));

		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(Eve.to_account_id()),
			Alice.to_account_id(),
			asset2,
			delegator2_stake,
			Fixed(vec![].try_into().unwrap()),
		));

		// Create UnappliedSlash with 50% slash for operator and first delegator only
		let exposed_stake = operator_stake / 2; // 50% of operator stake
		let exposed_delegation = delegator1_stake / 2; // 50% of delegator1 stake

		let unapplied_slash = UnappliedSlash {
			era: 1,
			blueprint_id,
			service_id,
			operator: Alice.to_account_id(),
			own: exposed_stake,
			others: vec![(Bob.to_account_id(), asset1, exposed_delegation)],
			reporters: vec![Eve.to_account_id()], // reporter doesn't matter for this test
		};

		// Apply the slash
		assert_ok!(MultiAssetDelegation::slash_operator(&unapplied_slash));

		// Verify operator stake was slashed
		let operator_info = MultiAssetDelegation::operator_info(Alice.to_account_id()).unwrap();
		assert_eq!(operator_info.stake, operator_stake - exposed_stake);

		// Verify first delegator (Bob) was slashed
		let delegator1 = MultiAssetDelegation::delegators(Bob.to_account_id()).unwrap();
		let delegation1 = delegator1
			.delegations
			.iter()
			.find(|d| d.operator == Alice.to_account_id() && d.asset_id == asset1)
			.unwrap();
		assert_eq!(delegation1.amount, delegator1_stake - exposed_delegation);

		// Verify second delegator (Eve) was NOT slashed since they didn't select the blueprint
		let delegator2 = MultiAssetDelegation::delegators(Eve.to_account_id()).unwrap();
		let delegation2 = delegator2
			.delegations
			.iter()
			.find(|d| d.operator == Alice.to_account_id() && d.asset_id == asset2)
			.unwrap();
		assert_eq!(delegation2.amount, delegator2_stake); // Amount unchanged

		// Verify events
		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(Event::OperatorSlashed {
			who: Alice.to_account_id(),
			amount: exposed_stake,
		}));

		System::assert_has_event(RuntimeEvent::MultiAssetDelegation(Event::DelegatorSlashed {
			who: Bob.to_account_id(),
			amount: exposed_delegation,
		}));
	});
}

#[test]
fn slash_operator_not_an_operator() {
	new_test_ext().execute_with(|| {
		let unapplied_slash = UnappliedSlash {
			era: 1,
			blueprint_id: 1,
			service_id: 42,
			operator: Alice.to_account_id(),
			own: 5_000,
			others: vec![],
			reporters: vec![Eve.to_account_id()],
		};

		assert_noop!(
			MultiAssetDelegation::slash_operator(&unapplied_slash),
			Error::<Runtime>::NotAnOperator
		);
	});
}

#[test]
fn slash_operator_not_active() {
	new_test_ext().execute_with(|| {
		// Setup and deactivate operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			10_000
		));
		assert_ok!(MultiAssetDelegation::go_offline(RuntimeOrigin::signed(Alice.to_account_id())));

		let unapplied_slash = UnappliedSlash {
			era: 1,
			blueprint_id: 1,
			service_id: 42,
			operator: Alice.to_account_id(),
			own: 5_000,
			others: vec![],
			reporters: vec![Eve.to_account_id()],
		};

		assert_noop!(
			MultiAssetDelegation::slash_operator(&unapplied_slash),
			Error::<Runtime>::NotActiveOperator
		);
	});
}

#[test]
fn slash_delegator_fixed_blueprint_not_selected() {
	new_test_ext().execute_with(|| {
		// Setup operator
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(Alice.to_account_id()),
			10_000
		));

		// Setup delegator with fixed blueprint selection
		let delegator_stake = 5_000;
		let asset_id = Asset::Custom(1);
		create_and_mint_tokens(1, Bob.to_account_id(), delegator_stake);

		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(Bob.to_account_id()),
			asset_id,
			delegator_stake,
			None,
			None
		));

		assert_ok!(MultiAssetDelegation::add_blueprint_id(
			RuntimeOrigin::signed(Bob.to_account_id()),
			1
		));

		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(Bob.to_account_id()),
			Alice.to_account_id(),
			asset_id,
			delegator_stake,
			Fixed(vec![2].try_into().unwrap()), // Selected blueprint 2, not 1
		));

		// Create UnappliedSlash for blueprint 1
		let unapplied_slash = UnappliedSlash {
			era: 1,
			blueprint_id: 1,
			service_id: 42,
			operator: Alice.to_account_id(),
			own: 5_000,
			others: vec![(Bob.to_account_id(), asset_id, 2_500)],
			reporters: vec![Eve.to_account_id()],
		};

		// Verify delegator is not slashed since they didn't select blueprint 1
		assert_ok!(MultiAssetDelegation::slash_operator(&unapplied_slash));
		let delegator = MultiAssetDelegation::delegators(Bob.to_account_id()).unwrap();
		let delegation = delegator
			.delegations
			.iter()
			.find(|d| d.operator == Alice.to_account_id())
			.unwrap();
		assert_eq!(delegation.amount, delegator_stake); // Amount unchanged
	});
}
