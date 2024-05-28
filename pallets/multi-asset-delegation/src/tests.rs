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
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use crate::{Event};
use crate::types::OperatorStatus;
use crate::tests::RuntimeEvent;
use crate::CurrentRound;

#[test]
    fn join_operator_success() {
        new_test_ext().execute_with(|| {
            let bond_amount = 10_000;

            assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(1), bond_amount));

            let operator_info = MultiAssetDelegation::operator_info(1).unwrap();
            assert_eq!(operator_info.bond, bond_amount);
            assert_eq!(operator_info.delegation_count, 0);
            assert_eq!(operator_info.total_counted, bond_amount);
            assert_eq!(operator_info.request, None);
            assert_eq!(operator_info.status, OperatorStatus::Active);

            // Verify event
            System::assert_has_event(RuntimeEvent::MultiAssetDelegation(Event::OperatorJoined { who : 1 } ));
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
            let leave_operators_delay = 10;
            

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
            System::assert_has_event(RuntimeEvent::MultiAssetDelegation(Event::OperatorLeavingScheduled { who : 1 }));
        });
    }