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
use crate::types::*;
use crate::Pallet as MultiAssetDelegation;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::ensure;
use frame_support::pallet_prelude::DispatchResult;
use frame_support::traits::Currency;
use frame_support::traits::Get;
use frame_support::traits::ReservableCurrency;
use frame_system::RawOrigin;
use sp_runtime::traits::Zero;
use sp_runtime::DispatchError;

const SEED: u32 = 0;

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
		assert_eq!(operator.bond, bond_amount + additional_bond);
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
		assert_eq!(operator.bond, bond_amount - unstake_amount);
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
		let asset_id: T::AssetId = 1_u32.into();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
	}: _(RawOrigin::Signed(caller.clone()), Some(asset_id), amount)
	verify {
		let metadata = Delegators::<T>::get(&caller).unwrap();
		assert_eq!(metadata.deposits.get(&asset_id).unwrap(), &amount);
	}

	schedule_withdraw {

		let caller: T::AccountId = whitelisted_caller();
		let asset_id: T::AssetId = 1_u32.into();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
	}: _(RawOrigin::Signed(caller.clone()), Some(asset_id), amount)
	verify {
		let metadata = Delegators::<T>::get(&caller).unwrap();
		assert!(metadata.withdraw_requests.is_some());
	}

	execute_withdraw {

		let caller: T::AccountId = whitelisted_caller();
		let asset_id: T::AssetId = 1_u32.into();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
		MultiAssetDelegation::<T>::schedule_withdraw(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
		let current_round = Pallet::<T>::current_round();
		CurrentRound::<T>::put(current_round + T::LeaveDelegatorsDelay::get());
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let metadata = Delegators::<T>::get(&caller).unwrap();
		assert!(metadata.withdraw_requests.is_none());
	}

	cancel_withdraw {

		let caller: T::AccountId = whitelisted_caller();
		let asset_id: T::AssetId = 1_u32.into();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
		MultiAssetDelegation::<T>::schedule_withdraw(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let metadata = Delegators::<T>::get(&caller).unwrap();
		assert!(metadata.withdraw_requests.is_none());
	}

	delegate {

		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let asset_id: T::AssetId = 1_u32.into();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(operator.clone()).into(), T::Currency::minimum_balance() * 20u32.into())?;
		MultiAssetDelegation::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
	}: _(RawOrigin::Signed(caller.clone()), operator.clone(), asset_id, amount)
	verify {
		let metadata = Delegators::<T>::get(&caller).unwrap();
		let delegation = metadata.delegations.iter().find(|d| d.operator == operator && d.asset_id == asset_id).unwrap();
		assert_eq!(delegation.amount, amount);
	}

	schedule_delegator_unstake {

		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let asset_id: T::AssetId = 1_u32.into();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(operator.clone()).into(), T::Currency::minimum_balance() * 20u32.into())?;
		MultiAssetDelegation::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
		MultiAssetDelegation::<T>::delegate(RawOrigin::Signed(caller.clone()).into(), operator.clone(), asset_id, amount)?;
	}: _(RawOrigin::Signed(caller.clone()), operator.clone(), asset_id, amount)
	verify {
		let metadata = Delegators::<T>::get(&caller).unwrap();
		assert!(metadata.delegator_unstake_requests.is_some());
	}

	execute_delegator_unstake {

		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let asset_id: T::AssetId = 1_u32.into();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(operator.clone()).into(), T::Currency::minimum_balance() * 20u32.into())?;
		MultiAssetDelegation::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
		MultiAssetDelegation::<T>::delegate(RawOrigin::Signed(caller.clone()).into(), operator.clone(), asset_id, amount)?;
		MultiAssetDelegation::<T>::schedule_delegator_unstake(RawOrigin::Signed(caller.clone()).into(), operator.clone(), asset_id, amount)?;
		let current_round = Pallet::<T>::current_round();
		CurrentRound::<T>::put(current_round + T::DelegationBondLessDelay::get());
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let metadata = Delegators::<T>::get(&caller).unwrap();
		assert!(metadata.delegator_unstake_requests.is_none());
	}

	cancel_delegator_unstake {

		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let asset_id: T::AssetId = 1_u32.into();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(operator.clone()).into(), T::Currency::minimum_balance() * 20u32.into())?;
		MultiAssetDelegation::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
		MultiAssetDelegation::<T>::delegate(RawOrigin::Signed(caller.clone()).into(), operator.clone(), asset_id, amount)?;
		MultiAssetDelegation::<T>::schedule_delegator_unstake(RawOrigin::Signed(caller.clone()).into(), operator.clone(), asset_id, amount)?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let metadata = Delegators::<T>::get(&caller).unwrap();
		assert!(metadata.delegator_unstake_requests.is_none());
	}

	set_incentive_apy_and_cap {

		let caller: T::AccountId = whitelisted_caller();
		let asset_id: T::AssetId = 1_u32.into();
		let apy: u128 = 1000;
		let cap: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
	}: _(RawOrigin::Root, asset_id, apy, cap)
	verify {
		let config = RewardConfigStorage::<T>::get().unwrap();
		let asset_config = config.configs.get(&asset_id).unwrap();
		assert_eq!(asset_config.apy, apy);
		assert_eq!(asset_config.cap, cap);
	}

	whitelist_blueprint_for_rewards {

		let caller: T::AccountId = whitelisted_caller();
		let blueprint_id: u32 = 1;
	}: _(RawOrigin::Root, blueprint_id)
	verify {
		let config = RewardConfigStorage::<T>::get().unwrap();
		assert!(config.whitelisted_blueprint_ids.contains(&blueprint_id));
	}
}
