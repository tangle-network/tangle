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
use crate::CurrentRound;
use frame_support::assert_ok;
use sp_keyring::AccountKeyring::{Alice, Bob, Charlie, Dave};
use tangle_primitives::services::Asset;

#[test]
fn handle_round_change_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = Bob.to_account_id();
		let operator = Alice.to_account_id();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		CurrentRound::<Runtime>::put(1);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator.clone()),
			10_000
		));

		create_and_mint_tokens(VDOT, who.clone(), amount);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			amount,
			None
		));

		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(who.clone()),
			operator.clone(),
			asset_id,
			amount,
			Default::default(),
			None
		));

		assert_ok!(Pallet::<Runtime>::handle_round_change());

		// Assert
		let current_round = MultiAssetDelegation::current_round();
		assert_eq!(current_round, 2);

		let snapshot1 = MultiAssetDelegation::at_stake(current_round, operator.clone()).unwrap();
		assert_eq!(snapshot1.stake, 10_000);
		assert_eq!(snapshot1.delegations.len(), 1);
		assert_eq!(snapshot1.delegations[0].amount, amount);
		assert_eq!(snapshot1.delegations[0].asset_id, asset_id);
	});
}

#[test]
fn handle_round_change_with_unstake_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let delegator1 = Alice.to_account_id();
		let delegator2 = Bob.to_account_id();
		let operator1 = Charlie.to_account_id();
		let operator2 = Dave.to_account_id();
		let asset_id = Asset::Custom(VDOT);
		let amount1 = 1_000_000_000_000;
		let amount2 = 1_000_000_000_000;
		let unstake_amount = 50;

		CurrentRound::<Runtime>::put(1);

		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator1.clone()),
			10_000
		));
		assert_ok!(MultiAssetDelegation::join_operators(
			RuntimeOrigin::signed(operator2.clone()),
			10_000
		));

		create_and_mint_tokens(VDOT, delegator1.clone(), amount1);
		mint_tokens(delegator1.clone(), VDOT, delegator2.clone(), amount2);

		// Deposit and delegate first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(delegator1.clone()),
			asset_id,
			amount1,
			None,
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(delegator1.clone()),
			operator1.clone(),
			asset_id,
			amount1,
			Default::default(),
			None
		));

		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(delegator2.clone()),
			asset_id,
			amount2,
			None
		));
		assert_ok!(MultiAssetDelegation::delegate(
			RuntimeOrigin::signed(delegator2.clone()),
			operator2.clone(),
			asset_id,
			amount2,
			Default::default(),
			None
		));

		// Delegator1 schedules unstake
		assert_ok!(MultiAssetDelegation::schedule_delegator_unstake(
			RuntimeOrigin::signed(delegator1.clone()),
			operator1.clone(),
			asset_id,
			unstake_amount,
		));

		assert_ok!(Pallet::<Runtime>::handle_round_change());

		// Assert
		let current_round = MultiAssetDelegation::current_round();
		assert_eq!(current_round, 2);

		// Check the snapshot for operator1
		let snapshot1 = MultiAssetDelegation::at_stake(current_round, operator1.clone()).unwrap();
		assert_eq!(snapshot1.stake, 10_000);
		assert_eq!(snapshot1.delegations.len(), 1);
		assert_eq!(snapshot1.delegations[0].delegator, delegator1.clone());
		assert_eq!(snapshot1.delegations[0].amount, amount1 - unstake_amount); // Amount reduced by unstake_amount
		assert_eq!(snapshot1.delegations[0].asset_id, asset_id);

		// Check the snapshot for operator2
		let snapshot2 = MultiAssetDelegation::at_stake(current_round, operator2.clone()).unwrap();
		assert_eq!(snapshot2.stake, 10000);
		assert_eq!(snapshot2.delegations.len(), 1);
		assert_eq!(snapshot2.delegations[0].delegator, delegator2.clone());
		assert_eq!(snapshot2.delegations[0].amount, amount2);
		assert_eq!(snapshot2.delegations[0].asset_id, asset_id);
	});
}
