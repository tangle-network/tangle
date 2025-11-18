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
use crate::{Pallet as MultiAssetDelegation, types::*};
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::{
	BoundedVec, assert_ok,
	traits::{Currency, Get},
};
use frame_system::RawOrigin;
use sp_core::H160;
use sp_runtime::Saturating;
use sp_staking::StakingInterface;
use sp_std::{vec, vec::Vec};
use tangle_primitives::{
	BlueprintId,
	rewards::LockMultiplier,
	services::{Asset, EvmAddressMapping},
};

const SEED: u32 = 0;
const INITIAL_BALANCE: u32 = 1_000_000;

fn native_asset_id<T: Config>() -> T::AssetId
where
	T::AssetId: From<u32>,
{
	0u32.into()
}

fn fund_account<T: Config>(who: &T::AccountId)
where
	T::AssetId: From<u32>,
{
	let balance = T::Currency::minimum_balance() * INITIAL_BALANCE.into();
	// Add enough to cover deposits and delegations used in benchmarks (typically 10x minimums)
	let balance = balance
		.saturating_add(T::MinDelegateAmount::get() * 10u32.into())
		.saturating_add(T::MinOperatorBondAmount::get() * 10u32.into());

	T::Currency::make_free_balance_be(who, balance);
}

fn setup_benchmark<T: Config>() -> Result<T::AccountId, &'static str>
where
	T::AssetId: From<u32>,
{
	let caller: T::AccountId = whitelisted_caller();
	// Fund account
	fund_account::<T>(&caller);
	Ok(caller)
}

/// Setup an account as a nominator in the staking system
/// This mirrors the test setup: creates staking ledger entry directly via storage
/// Following the pattern from tangle-lst benchmarks which access pallet_staking storage
fn setup_nominator<T: Config>(
	who: &T::AccountId,
	operator: &T::AccountId,
	asset_id: Asset<T::AssetId>,
	stake_amount: BalanceOf<T>,
	delegation_amount: BalanceOf<T>,
	nomination_amount: BalanceOf<T>,
) -> Result<(), &'static str> {
	let delegation_amount = T::MinDelegateAmount::get().saturating_add(delegation_amount);

	assert_ok!(MultiAssetDelegation::<T>::join_operators(
		RawOrigin::Signed(operator.clone()).into(),
		T::MinOperatorBondAmount::get().saturating_add(stake_amount)
	));

	assert_ok!(MultiAssetDelegation::<T>::deposit(
		RawOrigin::Signed(who.clone()).into(),
		asset_id.clone(),
		delegation_amount,
		None,
		None,
	));

	// Create a regular delegation
	assert_ok!(MultiAssetDelegation::<T>::delegate(
		RawOrigin::Signed(who.clone()).into(),
		operator.clone(),
		asset_id.clone(),
		delegation_amount,
		Default::default(),
	));

	// Create the ledger entry with bonded balance
	assert_ok!(T::StakingInterface::bond(who, nomination_amount, who));

	assert_ok!(T::StakingInterface::nominate(who, vec![operator.clone()],));

	Ok(())
}

benchmarks! {
	where_clause {
		where
			T::AssetId: From<u32>,
	}
	join_operators {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let bond_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() * 10u32.into();
	}: _(RawOrigin::Signed(caller.clone()), bond_amount)
	verify {
		assert!(Operators::<T>::contains_key(&caller));
	}

	schedule_leave_operators {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let bond_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() * 10u32.into();
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
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let bond_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		MultiAssetDelegation::<T>::schedule_leave_operators(RawOrigin::Signed(caller.clone()).into())?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		assert_eq!(operator.status, OperatorStatus::Active);
	}

	execute_leave_operators {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let bond_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		MultiAssetDelegation::<T>::schedule_leave_operators(RawOrigin::Signed(caller.clone()).into())?;
		let current_round = Pallet::<T>::current_round();
		CurrentRound::<T>::put(current_round + T::LeaveOperatorsDelay::get());
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(!Operators::<T>::contains_key(&caller));
	}

	operator_bond_more {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let bond_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		let additional_bond: BalanceOf<T> = T::Currency::minimum_balance() * 5u32.into();
	}: _(RawOrigin::Signed(caller.clone()), additional_bond)
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		assert_eq!(operator.stake, bond_amount + additional_bond);
	}

	schedule_operator_unstake {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let bond_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		let unstake_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() * 5u32.into();
	}: _(RawOrigin::Signed(caller.clone()), unstake_amount)
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		let request = operator.request.unwrap();
		assert_eq!(request.amount, unstake_amount);
	}

	execute_operator_unstake {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let bond_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		let unstake_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() * 5u32.into();
		MultiAssetDelegation::<T>::schedule_operator_unstake(RawOrigin::Signed(caller.clone()).into(), unstake_amount)?;
		let current_round = Pallet::<T>::current_round();
		// Execute withdraw uses LeaveDelegatorsDelay for readiness
		CurrentRound::<T>::put(current_round + T::LeaveDelegatorsDelay::get());
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		assert_eq!(operator.stake, bond_amount - unstake_amount);
	}

	cancel_operator_unstake {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let bond_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		let unstake_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() * 5u32.into();
		MultiAssetDelegation::<T>::schedule_operator_unstake(RawOrigin::Signed(caller.clone()).into(), unstake_amount)?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		assert!(operator.request.is_none());
	}

	go_offline {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let bond_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		assert_eq!(operator.status, OperatorStatus::Inactive);
	}

	go_online {
		let caller = setup_benchmark::<T>()?;
		let bond_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		MultiAssetDelegation::<T>::go_offline(RawOrigin::Signed(caller.clone()).into())?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		assert_eq!(operator.status, OperatorStatus::Active);
	}

	deposit_with_no_evm_address {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let deposit_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() + T::Currency::minimum_balance();
		let evm_address = None; // For Asset::Custom, evm_address must be None
		let lock_multiplier = Some(LockMultiplier::default());
		let asset = Asset::Custom(native_asset_id::<T>());
	}: deposit(RawOrigin::Signed(caller.clone()), asset, deposit_amount, evm_address, lock_multiplier)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		let delegator_deposit = delegator.deposits.get(&asset).unwrap();
		assert_eq!(delegator_deposit.amount, deposit_amount);
	}

	deposit_with_evm_address {
		let deposit_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() + T::Currency::minimum_balance();
		let evm_address = Some(H160::repeat_byte(1));
		let lock_multiplier = Some(LockMultiplier::default());
		let asset = Asset::Custom(native_asset_id::<T>());
		let evm_account: T::AccountId = T::EvmAddressMapping::into_account_id(evm_address.unwrap());
		fund_account::<T>(&evm_account);
	}: deposit(RawOrigin::Signed(evm_account.clone()), asset, deposit_amount, evm_address, lock_multiplier)
	verify {
		let delegator = Delegators::<T>::get(&evm_account).unwrap();
		let delegator_deposit = delegator.deposits.get(&asset).unwrap();
		assert_eq!(delegator_deposit.amount, deposit_amount);
	}

	schedule_withdraw {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let deposit_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() + T::Currency::minimum_balance();
		let asset = Asset::Custom(native_asset_id::<T>());
		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			deposit_amount,
			None,
			None
		)?;
	}: _(RawOrigin::Signed(caller.clone()), asset, deposit_amount)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		let withdraw = delegator.withdraw_requests.iter().find(|r| r.asset == asset).unwrap();
		assert_eq!(withdraw.amount, deposit_amount);
	}

	execute_withdraw_with_no_evm_address {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let deposit_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() + T::Currency::minimum_balance();
		let asset = Asset::Custom(native_asset_id::<T>());
		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			deposit_amount,
			None,
			None,
		)?;
		MultiAssetDelegation::<T>::schedule_withdraw(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			deposit_amount
		)?;
		// Verify withdraw request exists before execution
		let metadata = Delegators::<T>::get(&caller).unwrap();
		assert!(
			metadata
				.withdraw_requests
				.iter()
				.any(|r| r.asset == asset && r.amount == deposit_amount),
			"Withdraw request must exist before execution"
		);
		// Execute withdraw uses LeaveDelegatorsDelay for readiness check
		let current_round = Pallet::<T>::current_round();
		CurrentRound::<T>::put(current_round + T::LeaveDelegatorsDelay::get());
	}: execute_withdraw(RawOrigin::Signed(caller.clone()), None)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		assert!(!delegator.withdraw_requests.iter().any(|r| r.asset == asset));
	}

	execute_withdraw_with_evm_address {
		let pallet_account_id: T::AccountId = MultiAssetDelegation::<T>::pallet_account();
		let deposit_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() + T::Currency::minimum_balance();
		let asset = Asset::Custom(native_asset_id::<T>());
		let evm_address = Some(H160::repeat_byte(1));
		let evm_account: T::AccountId = T::EvmAddressMapping::into_account_id(evm_address.unwrap());
		fund_account::<T>(&evm_account);
		fund_account::<T>(&pallet_account_id);
		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(evm_account.clone()).into(),
			asset,
			deposit_amount,
			None,
			None,
		)?;
		MultiAssetDelegation::<T>::schedule_withdraw(
			RawOrigin::Signed(evm_account.clone()).into(),
			asset,
			deposit_amount
		)?;
		// Verify withdraw request exists before execution
		let metadata = Delegators::<T>::get(&evm_account).unwrap();
		assert!(
			metadata
				.withdraw_requests
				.iter()
				.any(|r| r.asset == asset && r.amount == deposit_amount),
			"Withdraw request must exist before execution"
		);
		// Execute withdraw uses LeaveDelegatorsDelay for readiness check
		let current_round = Pallet::<T>::current_round();
		CurrentRound::<T>::put(current_round + T::LeaveDelegatorsDelay::get());
	}: execute_withdraw(RawOrigin::Signed(pallet_account_id.clone()), evm_address)
	verify {
		let delegator = Delegators::<T>::get(&evm_account).unwrap();
		assert!(
			!delegator
				.withdraw_requests
				.iter()
				.any(|r| r.asset == asset && r.amount == deposit_amount)
		);
	}

	cancel_withdraw {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let deposit_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() + T::Currency::minimum_balance();
		let asset = Asset::Custom(native_asset_id::<T>());
		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			deposit_amount,
			None,
			None
		)?;
		MultiAssetDelegation::<T>::schedule_withdraw(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			deposit_amount
		)?;
	}: _(RawOrigin::Signed(caller.clone()), asset, deposit_amount)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		assert!(!delegator.withdraw_requests.iter().any(|r| r.asset == asset));
	}

	delegate {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let operator: T::AccountId = account("operator", 1, SEED);
		let deposit_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() + T::Currency::minimum_balance();
		let delegation_amount: BalanceOf<T> = T::MinDelegateAmount::get() + T::Currency::minimum_balance();
		let asset = Asset::Custom(native_asset_id::<T>());
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(vec![1u64]).unwrap());

		fund_account::<T>(&operator);
		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			deposit_amount,
			None,
			None
		)?;
		MultiAssetDelegation::<T>::join_operators(
			RawOrigin::Signed(operator.clone()).into(),
			deposit_amount
		)?;
	}: _(RawOrigin::Signed(caller.clone()), operator.clone(), asset, delegation_amount, blueprint_selection)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		let delegation = delegator.delegations.iter().find(|d| d.operator == operator && d.asset == asset).unwrap();
		assert_eq!(delegation.amount, delegation_amount);
	}

	schedule_delegator_unstake {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let operator: T::AccountId = account("operator", 1, SEED);
		let deposit_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() + T::Currency::minimum_balance();
		let delegation_amount: BalanceOf<T> = T::MinDelegateAmount::get() + T::Currency::minimum_balance();
		let asset = Asset::Custom(native_asset_id::<T>());
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(vec![1u64]).unwrap());

		fund_account::<T>(&operator);
		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			deposit_amount,
			None,
			None
		)?;
		MultiAssetDelegation::<T>::join_operators(
			RawOrigin::Signed(operator.clone()).into(),
			deposit_amount
		)?;
		MultiAssetDelegation::<T>::delegate(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			asset,
			delegation_amount,
			blueprint_selection
		)?;
	}: _(RawOrigin::Signed(caller.clone()), operator.clone(), asset, delegation_amount)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		let request = delegator.delegator_unstake_requests.iter().find(|r| r.operator == operator && r.asset == asset).unwrap();
		assert_eq!(request.amount, delegation_amount);
	}

	execute_delegator_unstake {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let operator: T::AccountId = account("operator", 1, SEED);
		let deposit_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() + T::Currency::minimum_balance();
		let delegation_amount: BalanceOf<T> = T::MinDelegateAmount::get() + T::Currency::minimum_balance();
		let asset = Asset::Custom(native_asset_id::<T>());
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(vec![1u64]).unwrap());

		fund_account::<T>(&operator);
		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			deposit_amount,
			None,
			None
		)?;
		MultiAssetDelegation::<T>::join_operators(
			RawOrigin::Signed(operator.clone()).into(),
			deposit_amount
		)?;
		MultiAssetDelegation::<T>::delegate(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			asset,
			delegation_amount,
			blueprint_selection
		)?;
		MultiAssetDelegation::<T>::schedule_delegator_unstake(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			asset,
			delegation_amount
		)?;
		let current_round = Pallet::<T>::current_round();
		CurrentRound::<T>::put(current_round + T::DelegationBondLessDelay::get());
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		assert!(!delegator.delegator_unstake_requests.iter().any(|r| r.operator == operator && r.asset == asset));
	}

	cancel_delegator_unstake {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let operator: T::AccountId = account("operator", 1, SEED);
		let deposit_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() + T::Currency::minimum_balance();
		let delegation_amount: BalanceOf<T> = T::MinDelegateAmount::get() + T::Currency::minimum_balance();
		let asset = Asset::Custom(native_asset_id::<T>());
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(vec![1u64]).unwrap());

		fund_account::<T>(&operator);
		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			deposit_amount,
			None,
			None
		)?;
		MultiAssetDelegation::<T>::join_operators(
			RawOrigin::Signed(operator.clone()).into(),
			deposit_amount
		)?;
		MultiAssetDelegation::<T>::delegate(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			asset,
			delegation_amount,
			blueprint_selection
		)?;
		MultiAssetDelegation::<T>::schedule_delegator_unstake(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			asset,
			delegation_amount
		)?;
	}: _(RawOrigin::Signed(caller.clone()), operator.clone(), asset, delegation_amount)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		assert!(!delegator.delegator_unstake_requests.iter().any(|r| r.operator == operator && r.asset == asset));
	}

	add_blueprint_id {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let operator: T::AccountId = account("operator", 1, SEED);
		let deposit_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() + T::Currency::minimum_balance();
		let delegation_amount: BalanceOf<T> = T::MinDelegateAmount::get() + T::Currency::minimum_balance();
		let asset = Asset::Custom(native_asset_id::<T>());
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(vec![]).unwrap());
		let blueprint_id: BlueprintId = 1u64;

		fund_account::<T>(&operator);
		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			deposit_amount,
			None,
			None
		)?;
		MultiAssetDelegation::<T>::join_operators(
			RawOrigin::Signed(operator.clone()).into(),
			deposit_amount
		)?;
		MultiAssetDelegation::<T>::delegate(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			asset,
			delegation_amount,
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
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let operator: T::AccountId = account("operator", 1, SEED);
		let deposit_amount: BalanceOf<T> = T::MinOperatorBondAmount::get() + T::Currency::minimum_balance();
		let delegation_amount: BalanceOf<T> = T::MinDelegateAmount::get() + T::Currency::minimum_balance();
		let asset = Asset::Custom(native_asset_id::<T>());
		let blueprint_id: BlueprintId = 1u64;
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(vec![blueprint_id]).unwrap());

		fund_account::<T>(&operator);
		MultiAssetDelegation::<T>::deposit(
			RawOrigin::Signed(caller.clone()).into(),
			asset,
			deposit_amount,
			None,
			None
		)?;
		MultiAssetDelegation::<T>::join_operators(
			RawOrigin::Signed(operator.clone()).into(),
			deposit_amount
		)?;
		MultiAssetDelegation::<T>::delegate(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			asset,
			delegation_amount,
			blueprint_selection
		)?;
	}: _(RawOrigin::Signed(caller.clone()), blueprint_id)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		if let DelegatorBlueprintSelection::Fixed(ids) = &delegator.delegations[0].blueprint_selection {
			assert!(!ids.contains(&blueprint_id));
		}
	}

	delegate_nomination {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let operator: T::AccountId = account("operator", 1, SEED);
		let asset_id = Asset::Custom(native_asset_id::<T>());
		let delegation_amount = T::Currency::minimum_balance();
		let stake_amount = T::Currency::minimum_balance();
		let nomination_amount = T::Currency::minimum_balance();
		// Use worst-case blueprint selection with maximum blueprints
		let max_blueprints = T::MaxDelegatorBlueprints::get();
		let blueprint_ids: Vec<BlueprintId> = (1..=max_blueprints as u64).collect();
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(blueprint_ids).unwrap());

		// Setup operator
		fund_account::<T>(&operator);

		setup_nominator::<T>(
			&caller,
			&operator,
			asset_id.clone(),
			stake_amount.clone(),
			delegation_amount.clone(),
			nomination_amount.clone(),
		)?;

	}: _(RawOrigin::Signed(caller.clone()), operator.clone(), nomination_amount, blueprint_selection)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		let nomination_delegation = delegator.delegations.iter()
			.find(|d| d.operator == operator && d.is_nomination)
			.expect("Nomination delegation must exist");
		assert_eq!(nomination_delegation.amount, nomination_amount);
		assert_eq!(nomination_delegation.asset, asset_id);
	}

	schedule_nomination_unstake {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let operator: T::AccountId = account("operator", 1, SEED);
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset_id = Asset::Custom(native_asset_id::<T>());
		let stake_amount = T::Currency::minimum_balance();
		let delegation_amount = T::Currency::minimum_balance();
		let nomination_amount = T::Currency::minimum_balance();
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(vec![1u64]).unwrap());

		fund_account::<T>(&operator);
		setup_nominator::<T>(
			&caller,
			&operator,
			asset_id.clone(),
			stake_amount.clone(),
			delegation_amount.clone(),
			nomination_amount.clone(),
		)?;
		assert_ok!(MultiAssetDelegation::<T>::delegate_nomination(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			nomination_amount.clone(),
			blueprint_selection.clone()
		));
	}: _(RawOrigin::Signed(caller.clone()), operator.clone(), nomination_amount, blueprint_selection)
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		let request = delegator.delegator_unstake_requests.iter()
			.find(|r| r.operator == operator && r.asset == asset_id && r.is_nomination)
			.expect("Unstake request must exist");
		assert_eq!(request.amount, nomination_amount);
	}

	execute_nomination_unstake {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let operator: T::AccountId = account("operator", 1, SEED);
		let nomination_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset_id = Asset::Custom(native_asset_id::<T>());
		let stake_amount = T::Currency::minimum_balance();
		let delegation_amount = T::Currency::minimum_balance();
		let nomination_amount = T::Currency::minimum_balance();
		// Use worst-case blueprint selection with maximum blueprints
		let max_blueprints = T::MaxDelegatorBlueprints::get();
		let blueprint_ids: Vec<BlueprintId> = (1..=max_blueprints as u64).collect();
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(blueprint_ids.clone()).unwrap());

		// Setup operator
		fund_account::<T>(&operator);
		setup_nominator::<T>(
			&caller,
			&operator,
			asset_id.clone(),
			stake_amount.clone(),
			delegation_amount.clone(),
			nomination_amount.clone(),
		)?;

		// Setup nomination delegation
		assert_ok!(MultiAssetDelegation::<T>::delegate_nomination(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			nomination_amount.clone(),
			blueprint_selection.clone()
		));

		// Schedule unstake
		assert_ok!(MultiAssetDelegation::<T>::schedule_nomination_unstake(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			nomination_amount.clone(),
			blueprint_selection.clone()
		));

		// Advance round to make request executable
		let current_round = Pallet::<T>::current_round();
		CurrentRound::<T>::put(current_round + T::DelegationBondLessDelay::get());
	}: _(RawOrigin::Signed(caller.clone()), operator.clone())
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		assert!(
			!delegator.delegator_unstake_requests.iter()
				.any(|r| r.operator == operator && r.asset == asset_id && r.is_nomination && r.amount == nomination_amount),
			"Unstake request must be removed after execution"
		);
	}

	cancel_nomination_unstake {
		let caller: T::AccountId = setup_benchmark::<T>()?;
		let operator: T::AccountId = account("operator", 1, SEED);
		let nomination_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		let asset_id = Asset::Custom(native_asset_id::<T>());
		let stake_amount = T::Currency::minimum_balance();
		let delegation_amount = T::Currency::minimum_balance();
		let nomination_amount = T::Currency::minimum_balance();
		// Use worst-case blueprint selection with maximum blueprints
		let max_blueprints = T::MaxDelegatorBlueprints::get();
		let blueprint_ids: Vec<BlueprintId> = (1..=max_blueprints as u64).collect();
		let blueprint_selection = DelegatorBlueprintSelection::Fixed(BoundedVec::try_from(blueprint_ids.clone()).unwrap());

		// Setup operator
		fund_account::<T>(&operator);
		setup_nominator::<T>(
			&caller,
			&operator,
			asset_id.clone(),
			stake_amount.clone(),
			delegation_amount.clone(),
			nomination_amount.clone(),
		)?;

		// Setup nomination delegation
		assert_ok!(MultiAssetDelegation::<T>::delegate_nomination(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			nomination_amount.clone(),
			blueprint_selection.clone()
		));

		// Schedule unstake
		assert_ok!(MultiAssetDelegation::<T>::schedule_nomination_unstake(
			RawOrigin::Signed(caller.clone()).into(),
			operator.clone(),
			nomination_amount.clone(),
			blueprint_selection.clone()
		));
	}: _(RawOrigin::Signed(caller.clone()), operator.clone())
	verify {
		let delegator = Delegators::<T>::get(&caller).unwrap();
		assert!(
			!delegator.delegator_unstake_requests.iter()
				.any(|r| r.operator == operator && r.asset == asset_id && r.is_nomination && r.amount == nomination_amount),
			"Unstake request must be removed after cancellation"
		);
	}
}

frame_benchmarking::impl_benchmark_test_suite!(
	Pallet,
	crate::mock::new_test_ext(),
	crate::mock::Runtime,
);
