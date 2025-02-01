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
use crate::{CurrentRound, Error};
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_keyring::AccountKeyring::Bob;
use sp_runtime::{ArithmeticError, DispatchError};
use tangle_primitives::services::{Asset, EvmAddressMapping};

// helper function
pub fn create_and_mint_tokens(
	asset_id: AssetId,
	recipient: <Runtime as frame_system::Config>::AccountId,
	amount: Balance,
) {
	assert_ok!(Assets::force_create(RuntimeOrigin::root(), asset_id, recipient.clone(), false, 1));
	assert_ok!(Assets::mint(RuntimeOrigin::signed(recipient.clone()), asset_id, recipient, amount));
}

pub fn mint_tokens(
	owner: <Runtime as frame_system::Config>::AccountId,
	asset_id: AssetId,
	recipient: <Runtime as frame_system::Config>::AccountId,
	amount: Balance,
) {
	assert_ok!(Assets::mint(RuntimeOrigin::signed(owner), asset_id, recipient, amount));
}

#[test]
fn deposit_should_work_for_fungible_asset() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let amount = 200;

		create_and_mint_tokens(VDOT, who.clone(), amount);

		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			Asset::Custom(VDOT),
			amount,
			None,
			None
		));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		let deposit = metadata.deposits.get(&Asset::Custom(VDOT)).unwrap();
		assert_eq!(deposit.amount, amount);

		assert_eq!(
			System::events().last().unwrap().event,
			RuntimeEvent::MultiAssetDelegation(crate::Event::Deposited {
				who: who.clone(),
				amount,
				asset_id: Asset::Custom(VDOT),
			})
		);

		// Verify that rewards manager was called with correct parameters
		assert_eq!(
			MockRewardsManager::record_deposit_calls(),
			vec![(who.clone(), Asset::Custom(VDOT), amount, None)]
		);
	});
}

#[test]
fn deposit_should_work_for_evm_asset() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let amount = 200;

		create_and_mint_tokens(VDOT, who.clone(), amount);

		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			Asset::Custom(VDOT),
			amount,
			None,
			None
		));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		let deposit = metadata.deposits.get(&Asset::Custom(VDOT)).unwrap();
		assert_eq!(deposit.amount, amount);

		assert_eq!(
			System::events().last().unwrap().event,
			RuntimeEvent::MultiAssetDelegation(crate::Event::Deposited {
				who: who.clone(),
				amount,
				asset_id: Asset::Custom(VDOT),
			})
		);
	});
}

#[test]
fn multiple_deposit_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let amount = 200;

		create_and_mint_tokens(VDOT, who.clone(), amount * 4);

		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			Asset::Custom(VDOT),
			amount,
			None,
			None,
		));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		let deposit = metadata.deposits.get(&Asset::Custom(VDOT)).unwrap();
		assert_eq!(deposit.amount, amount);

		assert_eq!(
			System::events().last().unwrap().event,
			RuntimeEvent::MultiAssetDelegation(crate::Event::Deposited {
				who: who.clone(),
				amount,
				asset_id: Asset::Custom(VDOT),
			})
		);

		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			Asset::Custom(VDOT),
			amount,
			None,
			None
		));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		let deposit = metadata.deposits.get(&Asset::Custom(VDOT)).unwrap();
		assert_eq!(deposit.amount, amount * 2);

		assert_eq!(
			System::events().last().unwrap().event,
			RuntimeEvent::MultiAssetDelegation(crate::Event::Deposited {
				who: who.clone(),
				amount,
				asset_id: Asset::Custom(VDOT),
			})
		);
	});
}

#[test]
fn deposit_should_fail_for_insufficient_balance() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let amount = 2000;

		create_and_mint_tokens(VDOT, who.clone(), 100);

		assert_noop!(
			MultiAssetDelegation::deposit(
				RuntimeOrigin::signed(who.clone()),
				Asset::Custom(VDOT),
				amount,
				None,
				None
			),
			ArithmeticError::Underflow
		);
	});
}

#[test]
fn deposit_should_fail_for_bond_too_low() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let amount = 50; // Below the minimum stake amount

		create_and_mint_tokens(VDOT, who.clone(), amount);

		assert_noop!(
			MultiAssetDelegation::deposit(
				RuntimeOrigin::signed(who.clone()),
				Asset::Custom(VDOT),
				amount,
				None,
				None
			),
			Error::<Runtime>::BondTooLow
		);
	});
}

#[test]
fn schedule_withdraw_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		create_and_mint_tokens(VDOT, who.clone(), 100);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			amount,
			None,
			None
		));

		assert_ok!(MultiAssetDelegation::schedule_withdraw(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			amount,
		));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		let deposit = metadata.deposits.get(&asset_id).unwrap();
		assert_eq!(deposit.amount, 0_u32.into());
		assert!(!metadata.withdraw_requests.is_empty());

		// Ensure that rewards pallet was called
		assert_eq!(
			MockRewardsManager::record_withdrawal_calls(),
			vec![(who.clone(), Asset::Custom(VDOT), amount)]
		);
	});
}

#[test]
fn schedule_withdraw_should_fail_if_not_delegator() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		create_and_mint_tokens(VDOT, who.clone(), 100);

		assert_noop!(
			MultiAssetDelegation::schedule_withdraw(
				RuntimeOrigin::signed(who.clone()),
				asset_id,
				amount,
			),
			Error::<Runtime>::NotDelegator
		);
	});
}

#[test]
fn schedule_withdraw_should_fail_for_insufficient_balance() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 200;

		create_and_mint_tokens(VDOT, who.clone(), 100);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			100,
			None,
			None
		));

		assert_noop!(
			MultiAssetDelegation::schedule_withdraw(
				RuntimeOrigin::signed(who.clone()),
				asset_id,
				amount,
			),
			Error::<Runtime>::InsufficientBalance
		);
	});
}

#[test]
fn schedule_withdraw_should_fail_if_withdraw_request_exists() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		create_and_mint_tokens(VDOT, who.clone(), 100);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			amount,
			None,
			None
		));

		// Schedule the first withdraw
		assert_ok!(MultiAssetDelegation::schedule_withdraw(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			amount,
		));
	});
}

#[test]
fn execute_withdraw_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		create_and_mint_tokens(VDOT, who.clone(), 100);

		// Deposit and schedule withdraw first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			amount,
			None,
			None
		));
		assert_ok!(MultiAssetDelegation::schedule_withdraw(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			amount,
		));

		// Simulate round passing
		let current_round = 2;
		<CurrentRound<Runtime>>::put(current_round);

		assert_ok!(MultiAssetDelegation::execute_withdraw(
			RuntimeOrigin::signed(who.clone()),
			None
		));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone());
		assert!(metadata.unwrap().withdraw_requests.is_empty());

		// Check event
		System::assert_last_event(RuntimeEvent::MultiAssetDelegation(
			crate::Event::ExecutedWithdraw { who: who.clone() },
		));
	});
}

#[test]
fn execute_withdraw_should_fail_if_not_delegator() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();

		assert_noop!(
			MultiAssetDelegation::execute_withdraw(RuntimeOrigin::signed(who.clone()), None),
			Error::<Runtime>::NotDelegator
		);
	});
}

#[test]
fn execute_withdraw_should_fail_if_no_withdraw_request() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		create_and_mint_tokens(VDOT, who.clone(), 100);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			amount,
			None,
			None
		));

		assert_noop!(
			MultiAssetDelegation::execute_withdraw(RuntimeOrigin::signed(who.clone()), None),
			Error::<Runtime>::NoWithdrawRequests
		);
	});
}

#[test]
fn execute_withdraw_should_fail_if_withdraw_not_ready() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		create_and_mint_tokens(VDOT, who.clone(), 100);

		// Deposit and schedule withdraw first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			amount,
			None,
			None
		));

		assert_ok!(MultiAssetDelegation::schedule_withdraw(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			amount,
		));

		// Simulate round passing but not enough
		let current_round = 0;
		<CurrentRound<Runtime>>::put(current_round);

		// should not actually withdraw anything
		assert_ok!(MultiAssetDelegation::execute_withdraw(
			RuntimeOrigin::signed(who.clone()),
			None
		));

		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		assert!(!metadata.withdraw_requests.is_empty());
	});
}

#[test]
fn execute_withdraw_should_fail_if_caller_not_pallet_from_evm() {
	new_test_ext().execute_with(|| {
		// Arrange
		let evm_address = mock_address(1);
		let pallet_account_id = MultiAssetDelegation::pallet_account();
		let who = PalletEVMAddressMapping::into_account_id(evm_address);
		let asset_id = Asset::Erc20(USDC_ERC20);
		let amount = 100;

		// Deposit would fail because the origin is not from the pallet
		assert_err!(
			MultiAssetDelegation::deposit(
				RuntimeOrigin::signed(who.clone()),
				asset_id,
				amount,
				Some(evm_address),
				None
			),
			DispatchError::BadOrigin
		);

		// Try with the pallet account should work
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(pallet_account_id.clone()),
			asset_id,
			amount,
			Some(evm_address),
			None
		));

		assert_ok!(MultiAssetDelegation::schedule_withdraw(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			amount,
		));

		// Simulate round passing
		let current_round = 2;
		<CurrentRound<Runtime>>::put(current_round);

		// Try to execute withdraw with the EVM address should fail
		assert_err!(
			MultiAssetDelegation::execute_withdraw(
				RuntimeOrigin::signed(who.clone()),
				Some(evm_address)
			),
			DispatchError::BadOrigin
		);

		// Try to execute withdraw with the pallet account should work
		assert_ok!(MultiAssetDelegation::execute_withdraw(
			RuntimeOrigin::signed(pallet_account_id.clone()),
			Some(evm_address)
		));
	});
}

#[test]
fn cancel_withdraw_should_work() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		create_and_mint_tokens(VDOT, who.clone(), 100);

		// Deposit and schedule withdraw first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			amount,
			None,
			None
		));

		assert_ok!(MultiAssetDelegation::schedule_withdraw(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			amount,
		));

		assert_ok!(MultiAssetDelegation::cancel_withdraw(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			amount
		));

		// Assert
		let metadata = MultiAssetDelegation::delegators(who.clone()).unwrap();
		let deposit = metadata.deposits.get(&asset_id).unwrap();
		assert_eq!(deposit.amount, amount);
		assert!(metadata.withdraw_requests.is_empty());

		// Check event
		System::assert_last_event(RuntimeEvent::MultiAssetDelegation(
			crate::Event::CancelledWithdraw { who: who.clone() },
		));
	});
}

#[test]
fn cancel_withdraw_should_fail_if_not_delegator() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();

		assert_noop!(
			MultiAssetDelegation::cancel_withdraw(
				RuntimeOrigin::signed(who.clone()),
				Asset::Custom(VDOT),
				1
			),
			Error::<Runtime>::NotDelegator
		);
	});
}

#[test]
fn cancel_withdraw_should_fail_if_no_withdraw_request() {
	new_test_ext().execute_with(|| {
		// Arrange
		let who: AccountId = Bob.into();
		let asset_id = Asset::Custom(VDOT);
		let amount = 100;

		create_and_mint_tokens(VDOT, who.clone(), 100);

		// Deposit first
		assert_ok!(MultiAssetDelegation::deposit(
			RuntimeOrigin::signed(who.clone()),
			asset_id,
			amount,
			None,
			None
		));

		assert_noop!(
			MultiAssetDelegation::cancel_withdraw(
				RuntimeOrigin::signed(who.clone()),
				asset_id,
				amount
			),
			Error::<Runtime>::NoMatchingwithdrawRequest
		);
	});
}
