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
use frame_support::{
	traits::{Currency, Get},
	BoundedVec,
};
use frame_system::RawOrigin;
use sp_core::H160;
use sp_std::vec;
use tangle_primitives::{rewards::LockMultiplier, services::Asset, BlueprintId};

const SEED: u32 = 0;
const INITIAL_BALANCE: u32 = 1_000_000;

fn native_asset_id<T: Config>() -> T::AssetId
where
	T::AssetId: From<u32>,
{
	0u32.into()
}

fn blueprint_id() -> BlueprintId {
	1u64
}

fn setup_benchmark<T: Config>() -> Result<T::AccountId, &'static str>
where
	T::AssetId: From<u32>,
{
	let caller: T::AccountId = whitelisted_caller();
	let balance = T::Currency::minimum_balance() * INITIAL_BALANCE.into();

	// Fund account
	T::Currency::make_free_balance_be(&caller, balance);
	Ok(caller)
}

benchmarks! {
	where_clause {
		where
			T::AssetId: From<u32>,
	}
	join_operators {
		let caller = setup_benchmark::<T>()?;
		let bond_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
	}: _(RawOrigin::Signed(caller.clone()), bond_amount)
	verify {
		assert!(Operators::<T>::contains_key(&caller));
	}

	schedule_leave_operators {
		let caller = setup_benchmark::<T>()?;
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
		let caller = setup_benchmark::<T>()?;
		let bond_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		MultiAssetDelegation::<T>::schedule_leave_operators(RawOrigin::Signed(caller.clone()).into())?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		assert_eq!(operator.status, OperatorStatus::Active);
	}

	execute_leave_operators {
		let caller = setup_benchmark::<T>()?;
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
		CurrentRound::<T>::put(current_round + T::DelegationBondLessDelay::get());
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
		let lock_multiplier = Some(LockMultiplier::default());
		let asset = Asset::Custom(native_asset_id::<T>());
	}: _(RawOrigin::Signed(caller.clone()), asset, amount, evm_address, lock_multiplier)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		let delegator_deposit = delegator.deposits.get(&asset).unwrap();
		assert_eq!(delegator_deposit.amount, amount);
	}

	schedule_withdraw {
		let caller: T::AccountId = whitelisted_caller();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Custom(native_asset_id::<T>());
		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			amount,
			None,
			None
		)?;
	}: _(RawOrigin::Signed(caller.clone()), asset, amount)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		let withdraw = delegator.withdraw_requests.iter().find(|r| r.asset == asset).unwrap();
		assert_eq!(withdraw.amount, amount);
	}

	execute_withdraw {
		let caller: T::AccountId = whitelisted_caller();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Custom(native_asset_id::<T>());
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
		CurrentRound::<T>::put(current_round + T::DelegationBondLessDelay::get());
	}: _(RawOrigin::Signed(caller.clone()), evm_address)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		assert!(!delegator.withdraw_requests.iter().any(|r| r.asset == asset));
	}

	cancel_withdraw {
		let caller: T::AccountId = whitelisted_caller();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Custom(native_asset_id::<T>());
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
		let delegator = Delegators::<T>::get(&caller).unwrap();
		assert!(!delegator.withdraw_requests.iter().any(|r| r.asset == asset));
	}

	delegate {
		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Custom(native_asset_id::<T>());
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(vec![1u64]).unwrap());

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
	}: _(RawOrigin::Signed(caller.clone()), operator.clone(), asset, amount, blueprint_selection)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		let delegation = delegator.delegations.iter().find(|d| d.operator == operator && d.asset == asset).unwrap();
		assert_eq!(delegation.amount, amount);
	}

	schedule_delegator_unstake {
		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Custom(native_asset_id::<T>());
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(vec![1u64]).unwrap());

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
	}: _(RawOrigin::Signed(caller.clone()), operator.clone(), asset, amount)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		let request = delegator.delegator_unstake_requests.iter().find(|r| r.operator == operator && r.asset == asset).unwrap();
		assert_eq!(request.amount, amount);
	}

	execute_delegator_unstake {
		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Custom(native_asset_id::<T>());
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(vec![1u64]).unwrap());

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
		CurrentRound::<T>::put(current_round + T::DelegationBondLessDelay::get());
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		assert!(!delegator.delegator_unstake_requests.iter().any(|r| r.operator == operator && r.asset == asset));
	}

	cancel_delegator_unstake {
		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Custom(native_asset_id::<T>());
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(vec![1u64]).unwrap());

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
	}: _(RawOrigin::Signed(caller.clone()), operator.clone(), asset, amount)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		assert!(!delegator.delegator_unstake_requests.iter().any(|r| r.operator == operator && r.asset == asset));
	}

	add_blueprint_id {
		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Custom(native_asset_id::<T>());
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(vec![]).unwrap());
		let blueprint_id: BlueprintId = 1u64;

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
		if let DelegatorBlueprintSelection::Fixed(ids) = &delegator.delegations[0].blueprint_selection {
			assert!(ids.contains(&blueprint_id));
		}
	}

	remove_blueprint_id {
		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset = Asset::Custom(native_asset_id::<T>());
		let blueprint_id: BlueprintId = 1u64;
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(vec![blueprint_id]).unwrap());

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
		if let DelegatorBlueprintSelection::Fixed(ids) = &delegator.delegations[0].blueprint_selection {
			assert!(!ids.contains(&blueprint_id));
		}
	}
}
