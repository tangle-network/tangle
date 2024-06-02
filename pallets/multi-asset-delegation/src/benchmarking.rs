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

	schedule_operator_bond_less {

		let caller: T::AccountId = whitelisted_caller();
		let bond_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		let bond_less_amount: BalanceOf<T> = T::Currency::minimum_balance() * 5u32.into();
	}: _(RawOrigin::Signed(caller.clone()), bond_less_amount)
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		let request = operator.request.unwrap();
		assert_eq!(request.amount, bond_less_amount);
	}

	execute_operator_bond_less {

		let caller: T::AccountId = whitelisted_caller();
		let bond_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		let bond_less_amount: BalanceOf<T> = T::Currency::minimum_balance() * 5u32.into();
		MultiAssetDelegation::<T>::schedule_operator_bond_less(RawOrigin::Signed(caller.clone()).into(), bond_less_amount)?;
		let current_round = Pallet::<T>::current_round();
		CurrentRound::<T>::put(current_round + T::OperatorBondLessDelay::get());
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let operator = Operators::<T>::get(&caller).unwrap();
		assert_eq!(operator.bond, bond_amount - bond_less_amount);
	}

	cancel_operator_bond_less {

		let caller: T::AccountId = whitelisted_caller();
		let bond_amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(caller.clone()).into(), bond_amount)?;
		let bond_less_amount: BalanceOf<T> = T::Currency::minimum_balance() * 5u32.into();
		MultiAssetDelegation::<T>::schedule_operator_bond_less(RawOrigin::Signed(caller.clone()).into(), bond_less_amount)?;
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

	schedule_unstake {

		let caller: T::AccountId = whitelisted_caller();
		let asset_id: T::AssetId = 1_u32.into();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
	}: _(RawOrigin::Signed(caller.clone()), Some(asset_id), amount)
	verify {
		let metadata = Delegators::<T>::get(&caller).unwrap();
		assert!(metadata.unstake_request.is_some());
	}

	execute_unstake {

		let caller: T::AccountId = whitelisted_caller();
		let asset_id: T::AssetId = 1_u32.into();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
		MultiAssetDelegation::<T>::schedule_unstake(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
		let current_round = Pallet::<T>::current_round();
		CurrentRound::<T>::put(current_round + T::LeaveDelegatorsDelay::get());
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let metadata = Delegators::<T>::get(&caller).unwrap();
		assert!(metadata.unstake_request.is_none());
	}

	cancel_unstake {

		let caller: T::AccountId = whitelisted_caller();
		let asset_id: T::AssetId = 1_u32.into();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
		MultiAssetDelegation::<T>::schedule_unstake(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let metadata = Delegators::<T>::get(&caller).unwrap();
		assert!(metadata.unstake_request.is_none());
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

	schedule_delegator_bond_less {

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
		assert!(metadata.delegator_bond_less_request.is_some());
	}

	execute_delegator_bond_less {

		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let asset_id: T::AssetId = 1_u32.into();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(operator.clone()).into(), T::Currency::minimum_balance() * 20u32.into())?;
		MultiAssetDelegation::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
		MultiAssetDelegation::<T>::delegate(RawOrigin::Signed(caller.clone()).into(), operator.clone(), asset_id, amount)?;
		MultiAssetDelegation::<T>::schedule_delegator_bond_less(RawOrigin::Signed(caller.clone()).into(), operator.clone(), asset_id, amount)?;
		let current_round = Pallet::<T>::current_round();
		CurrentRound::<T>::put(current_round + T::DelegationBondLessDelay::get());
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let metadata = Delegators::<T>::get(&caller).unwrap();
		assert!(metadata.delegator_bond_less_request.is_none());
	}

	cancel_delegator_bond_less {

		let caller: T::AccountId = whitelisted_caller();
		let operator: T::AccountId = account("operator", 1, SEED);
		let asset_id: T::AssetId = 1_u32.into();
		let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();
		MultiAssetDelegation::<T>::join_operators(RawOrigin::Signed(operator.clone()).into(), T::Currency::minimum_balance() * 20u32.into())?;
		MultiAssetDelegation::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), Some(asset_id), amount)?;
		MultiAssetDelegation::<T>::delegate(RawOrigin::Signed(caller.clone()).into(), operator.clone(), asset_id, amount)?;
		MultiAssetDelegation::<T>::schedule_delegator_bond_less(RawOrigin::Signed(caller.clone()).into(), operator.clone(), asset_id, amount)?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		let metadata = Delegators::<T>::get(&caller).unwrap();
		assert!(metadata.delegator_bond_less_request.is_none());
	}

	set_whitelisted_assets {
		let caller: T::AccountId = whitelisted_caller();
		let assets: Vec<T::AssetId> = vec![1u32.into()];
	}: _(RawOrigin::Root, assets.clone())
	verify {
		assert_eq!(WhitelistedAssets::<T>::get(), assets);
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
