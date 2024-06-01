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
use crate::tests::RuntimeEvent;
use crate::types::DelegatorStatus;
use crate::types::OperatorStatus;
use crate::CurrentRound;
use crate::Event;
use crate::{mock::*, Error};
use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::ArithmeticError;
use sp_runtime::DispatchError;

// helper function
pub fn mint_tokens(
	asset_id: AssetId,
	recipient: <Test as frame_system::Config>::AccountId,
	amount: Balance,
) {
	assert_ok!(Assets::force_create(RuntimeOrigin::root(), asset_id, 1, false, 1));
	assert_ok!(Assets::mint(RuntimeOrigin::signed(1), asset_id, recipient, amount));
}

#[test]
fn deposit_should_work_for_fungible_asset() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let amount = 200;

		mint_tokens(vDOT, who, amount);

		// Act
		assert_ok!(MultiAssetDelegation::deposit(RuntimeOrigin::signed(who), Some(vDOT), amount,));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert_eq!(metadata.deposits.get(&vDOT), Some(&amount));
		assert_eq!(
			System::events().last().unwrap().event,
			RuntimeEvent::MultiAssetDelegation(crate::Event::Deposited {
				who,
				amount,
				asset_id: Some(vDOT),
			})
			.into()
		);
	});
}

#[test]
fn deposit_should_fail_for_insufficient_balance() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let amount = 2000;

		mint_tokens(vDOT, who, 100);

		// Act & Assert
		assert_noop!(
			MultiAssetDelegation::deposit(RuntimeOrigin::signed(who), Some(vDOT), amount,),
			ArithmeticError::Underflow
		);
	});
}

#[test]
fn deposit_should_fail_if_already_delegator() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let amount = 200;

		mint_tokens(vDOT, who, amount);

		// First deposit
		assert_ok!(MultiAssetDelegation::deposit(RuntimeOrigin::signed(who), Some(vDOT), amount,));

		// Act & Assert
		assert_noop!(
			MultiAssetDelegation::deposit(RuntimeOrigin::signed(who), Some(vDOT), amount,),
			Error::<Test>::AlreadyDelegator
		);
	});
}

#[test]
fn deposit_should_fail_for_bond_too_low() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let amount = 50; // Below the minimum bond amount

		mint_tokens(vDOT, who, amount);

		// Act & Assert
		assert_noop!(
			MultiAssetDelegation::deposit(RuntimeOrigin::signed(who), Some(vDOT), amount,),
			Error::<Test>::BondTooLow
		);
	});
}

#[test]
fn schedule_unstake_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let asset_id = vDOT;
		let amount = 100;

		mint_tokens(vDOT, who, 100);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));

		// Act
		assert_ok!(MultiAssetDelegation::schedule_unstake(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert_eq!(metadata.deposits.get(&asset_id), None);
		assert!(metadata.unstake_request.is_some());
		let request = metadata.unstake_request.unwrap();
		assert_eq!(request.asset_id, asset_id);
		assert_eq!(request.amount, amount);
	});
}

#[test]
fn schedule_unstake_should_fail_if_not_delegator() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let asset_id = vDOT;
		let amount = 100;

		mint_tokens(vDOT, who, 100);

		// Act & Assert
		assert_noop!(
			MultiAssetDelegation::schedule_unstake(
				RuntimeOrigin::signed(who),
				Some(asset_id),
				amount,
			),
			Error::<Test>::NotDelegator
		);
	});
}

#[test]
fn schedule_unstake_should_fail_for_insufficient_balance() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let asset_id = vDOT;
		let amount = 200;

		mint_tokens(vDOT, who, 100);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(RuntimeOrigin::signed(who), Some(asset_id), 100,));

		// Act & Assert
		assert_noop!(
			MultiAssetDelegation::schedule_unstake(
				RuntimeOrigin::signed(who),
				Some(asset_id),
				amount,
			),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn schedule_unstake_should_fail_if_withdraw_request_exists() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let asset_id = vDOT;
		let amount = 100;

		mint_tokens(vDOT, who, 100);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));

		// Schedule the first unstake
		assert_ok!(MultiAssetDelegation::schedule_unstake(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));

		// Act & Assert
		assert_noop!(
			MultiAssetDelegation::schedule_unstake(
				RuntimeOrigin::signed(who),
				Some(asset_id),
				amount,
			),
			Error::<Test>::WithdrawRequestAlreadyExists
		);
	});
}

#[test]
fn execute_unstake_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let asset_id = vDOT;
		let amount = 100;

		mint_tokens(vDOT, who, 100);

		// Deposit and schedule unstake first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));
		assert_ok!(MultiAssetDelegation::schedule_unstake(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));

		// Simulate round passing
		let current_round = 1;
		<CurrentRound<Test>>::put(current_round);

		// Act
		assert_ok!(MultiAssetDelegation::execute_unstake(RuntimeOrigin::signed(who),));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(metadata.unstake_request.is_none());

		// Check event
		System::assert_last_event(
			RuntimeEvent::MultiAssetDelegation(crate::Event::ExecutedUnstake { who }).into(),
		);
	});
}

#[test]
fn execute_unstake_should_fail_if_not_delegator() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;

		// Act & Assert
		assert_noop!(
			MultiAssetDelegation::execute_unstake(RuntimeOrigin::signed(who),),
			Error::<Test>::NotDelegator
		);
	});
}

#[test]
fn execute_unstake_should_fail_if_no_withdraw_request() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let asset_id = vDOT;
		let amount = 100;

		mint_tokens(vDOT, who, 100);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));

		// Act & Assert
		assert_noop!(
			MultiAssetDelegation::execute_unstake(RuntimeOrigin::signed(who),),
			Error::<Test>::NoWithdrawRequest
		);
	});
}

#[test]
fn execute_unstake_should_fail_if_unstake_not_ready() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let asset_id = vDOT;
		let amount = 100;

		mint_tokens(vDOT, who, 100);

		// Deposit and schedule unstake first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));
		assert_ok!(MultiAssetDelegation::schedule_unstake(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));

		// Simulate round passing but not enough
		let current_round = 0;
		<CurrentRound<Test>>::put(current_round);

		// Act & Assert
		assert_noop!(
			MultiAssetDelegation::execute_unstake(RuntimeOrigin::signed(who),),
			Error::<Test>::UnstakeNotReady
		);
	});
}

#[test]
fn cancel_unstake_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let asset_id = vDOT;
		let amount = 100;

		mint_tokens(vDOT, who, 100);

		// Deposit and schedule unstake first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));
		assert_ok!(MultiAssetDelegation::schedule_unstake(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));

		// Act
		assert_ok!(MultiAssetDelegation::cancel_unstake(RuntimeOrigin::signed(who),));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who).unwrap();
		assert!(metadata.unstake_request.is_none());
		assert_eq!(metadata.deposits.get(&asset_id), Some(&amount));
		assert_eq!(metadata.status, DelegatorStatus::Active);

		// Check event
		System::assert_last_event(
			RuntimeEvent::MultiAssetDelegation(crate::Event::CancelledUnstake { who }).into(),
		);
	});
}

#[test]
fn cancel_unstake_should_fail_if_not_delegator() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;

		// Act & Assert
		assert_noop!(
			MultiAssetDelegation::cancel_unstake(RuntimeOrigin::signed(who),),
			Error::<Test>::NotDelegator
		);
	});
}

#[test]
fn cancel_unstake_should_fail_if_no_withdraw_request() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who = 1;
		let asset_id = vDOT;
		let amount = 100;

		mint_tokens(vDOT, who, 100);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who),
			Some(asset_id),
			amount,
		));

		// Act & Assert
		assert_noop!(
			MultiAssetDelegation::cancel_unstake(RuntimeOrigin::signed(who),),
			Error::<Test>::NoWithdrawRequest
		);
	});
}
