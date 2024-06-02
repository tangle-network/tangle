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
use crate::CurrentRound;
use frame_support::assert_ok;

#[test]
fn handle_round_change_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let operator = 2;
		let asset_id = VDOT;
		let amount = 100;

		CurrentRound::<Test>::put(1);

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator), 10_000));

		create_and_mint_tokens(VDOT, who, amount);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));

		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who),
			operator,
			asset_id,
			amount,
		));

		assert_ok!(Pallet::<Test>::handle_round_change());

		// Assert
		let current_round = MultiAssetDelegation::current_round();
		assert_eq!(current_round, 2);

		let snapshot1 = MultiAssetDelegation::at_stake(current_round, operator).unwrap();
		assert_eq!(snapshot1.bond, 10_000);
		assert_eq!(snapshot1.delegations.len(), 1);
		assert_eq!(snapshot1.delegations[0].amount, amount);
		assert_eq!(snapshot1.delegations[0].asset_id, asset_id);
	});
}

#[test]
fn handle_round_change_with_bond_less_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let delegator1 = 1;
		let delegator2 = 2;
		let operator1 = 3;
		let operator2 = EVE;
		let asset_id = VDOT;
		let amount1 = 100;
		let amount2 = 200;
		let bond_less_amount = 50;

		CurrentRound::<Test>::put(1);

		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator1), 10_000));
		assert_ok!(MultiAssetDelegation::join_operators(RuntimeOrigin::signed(operator2), 10_000));

		create_and_mint_tokens(VDOT, delegator1, amount1);
		mint_tokens(delegator1, VDOT, delegator2, amount2);

		// Deposit and delegate first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(delegator1),
			Some(asset_id),
			amount1,
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(delegator1),
			operator1,
			asset_id,
			amount1,
		));

		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(delegator2),
			Some(asset_id),
			amount2,
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(delegator2),
			operator2,
			asset_id,
			amount2,
		));

		// Delegator1 schedules bond less
		assert_ok!(MultiAssetDelegation::schedule_delegator_bond_less(
			RuntimeOrigin::signed(delegator1),
			operator1,
			asset_id,
			bond_less_amount,
		));

		assert_ok!(Pallet::<Test>::handle_round_change());

		// Assert
		let current_round = MultiAssetDelegation::current_round();
		assert_eq!(current_round, 2);

		// Check the snapshot for operator1
		let snapshot1 = MultiAssetDelegation::at_stake(current_round, operator1).unwrap();
		assert_eq!(snapshot1.bond, 10_000);
		assert_eq!(snapshot1.delegations.len(), 1);
		assert_eq!(snapshot1.delegations[0].delegator, delegator1);
		assert_eq!(snapshot1.delegations[0].amount, amount1 - bond_less_amount); // Amount reduced by bond_less_amount
		assert_eq!(snapshot1.delegations[0].asset_id, asset_id);

		// Check the snapshot for operator2
		let snapshot2 = MultiAssetDelegation::at_stake(current_round, operator2).unwrap();
		assert_eq!(snapshot2.bond, 10000);
		assert_eq!(snapshot2.delegations.len(), 1);
		assert_eq!(snapshot2.delegations[0].delegator, delegator2);
		assert_eq!(snapshot2.delegations[0].amount, amount2);
		assert_eq!(snapshot2.delegations[0].asset_id, asset_id);
	});
}
