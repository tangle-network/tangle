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
use crate::{types::*, Pallet as MultiAssetDelegation};
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::fungibles;
use frame_support::{
	ensure,
	pallet_prelude::DispatchResult,
	traits::{Currency, Get, ReservableCurrency},
};
use frame_system::RawOrigin;
use sp_runtime::{traits::Zero, DispatchError};

const SEED: u32 = 0;
const NATIVE_ASSET_ID: u32 = 0;
const FOREIGN_ASSET_ID: u32 = 1;

benchmarks! {
	join_operators {

		let caller: T::AccountId = whitelisted_caller();
		let bond_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
	}: _(RawOrigin::Signed(caller.clone()), bond_amount)
	verify {
		assert!(Operators::<T>::contains_key(&caller));
	}

	schedule_leave_operators {

		let caller: T::AccountId = whitelisted_caller();
		let bond_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		match operator.status {
			OperatorStatus::Leaving(_) => {},
			_ => panic!("Operator should be in Leaving status"),
		}
	}

	cancel_leave_operators {

		let caller: T::AccountId = whitelisted_caller();
		let bond_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		MultiAssetDelegation::<T>::schedule_leave_operators(RawOrigin::Signed(caller.clone()).into())?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		assert_eq!(operator.status, OperatorStatus::Active);
	}

	execute_leave_operators {

		let caller: T::AccountId = whitelisted_caller();
		let bond_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		MultiAssetDelegation::<T>::schedule_leave_operators(RawOrigin::Signed(caller.clone()).into())?;
		let current_round = Pallet::<T>::current_round();
		CurrentRound::<T>::put(current_round + T::LeaveOperatorsDelay::get());
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(!Operators::<T>::contains_key(&caller));
	}

	operator_bond_more {

		let caller: T::AccountId = whitelisted_caller();
		let bond_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		let additional_bond: BalanceOf<T> = T::Currency::minimum_balance() * 5u32.into();
	}: _(RawOrigin::Signed(caller.clone()), additional_bond)
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		assert_eq!(operator.stake, bond_amount + additional_bond);
	}

	schedule_operator_unstake {

		let caller: T::AccountId = whitelisted_caller();
		let bond_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		let unstake_amount: BalanceOf<T> = T::Currency::minimum_balance() * 5u32.into();
	}: _(RawOrigin::Signed(caller.clone()), unstake_amount)
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		let request = operator.request.unwrap();
		assert_eq!(request.amount, unstake_amount);
	}

	execute_operator_unstake {

		let caller: T::AccountId = whitelisted_caller();
		let bond_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		let unstake_amount: BalanceOf<T> = T::Currency::minimum_balance() * 5u32.into();
		MultiAssetDelegation::<T>::schedule_operator_unstake(RawOrigin::Signed(caller.clone()).into(), unstake_amount)?;
		let current_round = Pallet::<T>::current_round();
		CurrentRound::<T>::put(current_round + T::OperatorBondLessDelay::get());
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		assert_eq!(operator.stake, bond_amount - unstake_amount);
	}

	cancel_operator_unstake {
		let caller: T::AccountId = whitelisted_caller();
		let bond_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		let unstake_amount: BalanceOf<T> = T::Currency::minimum_balance() * 5u32.into();
		MultiAssetDelegation::<T>::schedule_operator_unstake(RawOrigin::Signed(caller.clone()).into(), unstake_amount)?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		assert!(operator.request.is_none());
	}

	go_offline {
		let caller: T::AccountId = whitelisted_caller();
		let bond_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		assert_eq!(operator.status, OperatorStatus::Inactive);
	}

	go_online {

		let caller: T::AccountId = whitelisted_caller();
		let bond_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		MultiAssetDelegation::<T>::go_offline(RawOrigin::Signed(caller.clone()).into())?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		assert_eq!(operator.status, OperatorStatus::Active);
	}

	deposit {
		let caller: T::AccountId = whitelisted_caller();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let evm_address = Some(H160::repeat_byte(1));
		let lock_multiplier = Some(LockMultiplier::One);
		let asset = Asset::Native(NATIVE_ASSET_ID.into());
	}: _(RawOrigin::Signed(caller.clone()), asset, amount, evm_address, lock_multiplier)
	verify {
		let deposit = Deposits::<T>::get(&caller, asset).unwrap();
		assert_eq!(deposit.amount, amount);
	}

	schedule_withdraw {
		let caller: T::AccountId = whitelisted_caller();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Native(NATIVE_ASSET_ID.into());
		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			amount,
			None,
			None
		)?;
	}: _(RawOrigin::Signed(caller.clone()), asset, amount)
	verify {
		let withdraw = WithdrawRequests::<T>::get(&caller, asset).unwrap();
		assert_eq!(withdraw.amount, amount);
	}

	execute_withdraw {
		let caller: T::AccountId = whitelisted_caller();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Native(NATIVE_ASSET_ID.into());
		let evm_address = Some(H160::repeat_byte(1));
		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			amount,
			None,
			None
		)?;
		MultiAssetDelegation::<T>::schedule_withdraw(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			amount
		)?;
		let current_round = Pallet::<T>::current_round();
		CurrentRound::<T>::put(current_round + T::WithdrawDelay::get());
	}: _(RawOrigin::Signed(caller.clone()), evm_address)
	verify {
		assert!(!WithdrawRequests::<T>::contains_key(&caller, asset));
	}

	cancel_withdraw {
		let caller: T::AccountId = whitelisted_caller();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Native(NATIVE_ASSET_ID.into());
		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			amount,
			None,
			None
		)?;
		MultiAssetDelegation::<T>::schedule_withdraw(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			amount
		)?;
	}: _(RawOrigin::Signed(caller.clone()), asset, amount)
	verify {
		assert!(!WithdrawRequests::<T>::contains_key(&caller, asset));
	}

	delegate {
		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Native(NATIVE_ASSET_ID.into());
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(vec![1.into()]);

		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			amount,
			None,
			None
		)?;
		MultiAssetDelegation::<T>::join_operators(
			RawOrigin::Signed(operator.clone()).into(),
			amount
		)?;
	}: _(RawOrigin::Signed(caller.clone()), operator, asset, amount, blueprint_selection)
	verify {
		let delegation = Delegations::<T>::get(&caller, &operator, asset).unwrap();
		assert_eq!(delegation.amount, amount);
	}

	schedule_delegator_unstake {
		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Native(NATIVE_ASSET_ID.into());
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(vec![1.into()]);

		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			amount,
			None,
			None
		)?;
		MultiAssetDelegation::<T>::join_operators(
			RawOrigin::Signed(operator.clone()).into(),
			amount
		)?;
		MultiAssetDelegation::<T>::delegate(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			asset,
			amount,
			blueprint_selection
		)?;
	}: _(RawOrigin::Signed(caller.clone()), operator, asset, amount)
	verify {
		let request = UnstakeRequests::<T>::get(&caller, &operator, asset).unwrap();
		assert_eq!(request.amount, amount);
	}

	execute_delegator_unstake {
		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Native(NATIVE_ASSET_ID.into());
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(vec![1.into()]);

		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			amount,
			None,
			None
		)?;
		MultiAssetDelegation::<T>::join_operators(
			RawOrigin::Signed(operator.clone()).into(),
			amount
		)?;
		MultiAssetDelegation::<T>::delegate(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			asset,
			amount,
			blueprint_selection
		)?;
		MultiAssetDelegation::<T>::schedule_delegator_unstake(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			asset,
			amount
		)?;
		let current_round = Pallet::<T>::current_round();
		CurrentRound::<T>::put(current_round + T::UnstakeDelay::get());
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(!UnstakeRequests::<T>::contains_key(&caller, &operator, asset));
	}

	cancel_delegator_unstake {
		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Native(NATIVE_ASSET_ID.into());
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(vec![1.into()]);

		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			amount,
			None,
			None
		)?;
		MultiAssetDelegation::<T>::join_operators(
			RawOrigin::Signed(operator.clone()).into(),
			amount
		)?;
		MultiAssetDelegation::<T>::delegate(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			asset,
			amount,
			blueprint_selection
		)?;
		MultiAssetDelegation::<T>::schedule_delegator_unstake(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			asset,
			amount
		)?;
	}: _(RawOrigin::Signed(caller.clone()), operator, asset, amount)
	verify {
		assert!(!UnstakeRequests::<T>::contains_key(&caller, &operator, asset));
	}

	add_blueprint_id {
		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Native(NATIVE_ASSET_ID.into());
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(vec![]);
		let blueprint_id: BlueprintId = 1.into();

		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			amount,
			None,
			None
		)?;
		MultiAssetDelegation::<T>::join_operators(
			RawOrigin::Signed(operator.clone()).into(),
			amount
		)?;
		MultiAssetDelegation::<T>::delegate(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			asset,
			amount,
			blueprint_selection
		)?;
	}: _(RawOrigin::Signed(caller.clone()), blueprint_id)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		assert!(delegator.blueprint_ids.contains(&blueprint_id));
	}

	remove_blueprint_id {
		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Native(NATIVE_ASSET_ID.into());
		let blueprint_id: BlueprintId = 1.into();
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(vec![blueprint_id]);

		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			amount,
			None,
			None
		)?;
		MultiAssetDelegation::<T>::join_operators(
			RawOrigin::Signed(operator.clone()).into(),
			amount
		)?;
		MultiAssetDelegation::<T>::delegate(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			asset,
			amount,
			blueprint_selection
		)?;
	}: _(RawOrigin::Signed(caller.clone()), blueprint_id)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		assert!(!delegator.blueprint_ids.contains(&blueprint_id));
	}
}
