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
	ensure,
	pallet_prelude::DispatchResult,
	traits::{Currency, Get, ReservableCurrency},
};
use frame_system::RawOrigin;
use sp_runtime::{traits::Zero, DispatchError};

const SEED: u32 = 0;

fn setup_vault<T: Config>() -> (T::VaultId, T::AccountId) {
	let vault_id = T::VaultId::zero();
	let caller: T::AccountId = account("caller", 0, SEED);
	let balance = BalanceOf::<T>::from(1000u32);
	T::Currency::make_free_balance_be(&caller, balance);

	// Setup reward config
	let reward_config = RewardConfigForAssetVault {
		apy: Perbill::from_percent(10),
		deposit_cap: balance,
		incentive_cap: balance,
		boost_multiplier: Some(150),
	};
	RewardConfigStorage::<T>::insert(vault_id, reward_config);

	(vault_id, caller)
}

benchmarks! {
	claim_rewards {
		let (vault_id, caller) = setup_vault::<T>();
		let deposit = BalanceOf::<T>::from(100u32);
		let deposit_info = UserDepositWithLocks {
			unlocked_amount: deposit,
			amount_with_locks: None,
		};
	}: _(RawOrigin::Signed(caller.clone()), vault_id)
	verify {
		assert!(UserClaimedReward::<T>::contains_key(&caller, vault_id));
	}

	force_claim_rewards {
		let (vault_id, caller) = setup_vault::<T>();
		let deposit = BalanceOf::<T>::from(100u32);
		let deposit_info = UserDepositWithLocks {
			unlocked_amount: deposit,
			amount_with_locks: None,
		};
		let origin = T::ForceOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
	}: _<T::RuntimeOrigin>(origin, caller.clone(), vault_id)
	verify {
		assert!(UserClaimedReward::<T>::contains_key(&caller, vault_id));
	}

	update_vault_reward_config {
		let (vault_id, _) = setup_vault::<T>();
		let new_config = RewardConfigForAssetVault {
			apy: Perbill::from_percent(20),
			deposit_cap: BalanceOf::<T>::from(2000u32),
			incentive_cap: BalanceOf::<T>::from(2000u32),
			boost_multiplier: Some(200),
		};
		let origin = T::ForceOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
	}: _<T::RuntimeOrigin>(origin, vault_id, new_config.clone())
	verify {
		assert_eq!(RewardConfigStorage::<T>::get(vault_id), Some(new_config));
	}
}

impl_benchmark_test_suite!(RewardsPallet, crate::mock::new_test_ext(), crate::mock::Runtime);
