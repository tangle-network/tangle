use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use crate::{Event};
use crate::types::OperatorStatus;
use crate::tests::RuntimeEvent;

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
            System::assert_has_event(RuntimeEvent::MultiAssetDelegation(Event::OperatorJoined(1)));
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