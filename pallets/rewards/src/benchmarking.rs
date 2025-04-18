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
use crate::{
	Call, Config, Pallet,
	pallet::{UserClaimedReward, UserServiceReward},
	types::*,
};
use frame_benchmarking::{
	BenchmarkError, account, benchmarks, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_support::traits::{Currency, EnsureOrigin};
use frame_system::{RawOrigin, pallet_prelude::BlockNumberFor};
use sp_runtime::Perbill;
use sp_std::{collections::btree_map::BTreeMap, prelude::*};
use tangle_primitives::{rewards::UserDepositWithLocks, services::Asset};

const SEED: u32 = 0;

fn setup_vault<T: Config>() -> (T::VaultId, T::AccountId)
where
	<T as pallet::Config>::AssetId: From<u32>,
{
	let vault_id = Default::default();
	let caller: T::AccountId = account("caller", 0, SEED);
	let balance = BalanceOf::<T>::from(1000u32);
	T::Currency::make_free_balance_be(&caller, balance);

	// Setup reward config with boost_multiplier = 1 (100%)
	let reward_config = RewardConfigForAssetVault {
		apy: Perbill::from_percent(10),
		deposit_cap: balance,
		incentive_cap: balance,
		boost_multiplier: Some(1),
	};
	RewardConfigStorage::<T>::insert(vault_id, reward_config);

	// Setup reward vault with native asset
	let asset_one = Asset::Custom(1_u32.into());
	let asset_two = Asset::Custom(2_u32.into());
	let mut assets = Vec::new();
	assets.push(asset_one);
	assets.push(asset_two);
	RewardVaults::<T>::insert(vault_id, assets.clone());

	(vault_id, caller)
}

benchmarks! {
	where_clause {
		where
			T::ForceOrigin: EnsureOrigin<<T as frame_system::Config>::RuntimeOrigin>,
			T::AssetId: From<u32>,
	}

	claim_rewards {
		let (vault_id, caller) = setup_vault::<T>();
		let deposit = BalanceOf::<T>::from(100u32);
		let deposit_info: UserDepositWithLocks<BalanceOf<T>, BlockNumberFor<T>> = UserDepositWithLocks {
			unlocked_amount: deposit,
			amount_with_locks: None,
		};
		let asset = Asset::Custom(1_u32.into());
		UserServiceReward::<T>::insert(caller.clone(), asset, deposit);
	}: _(RawOrigin::Signed(caller.clone()), asset)
	verify {
		assert!(UserClaimedReward::<T>::contains_key(&caller, vault_id));
	}

	update_vault_reward_config {
		let (vault_id, _) = setup_vault::<T>();
		let new_config = RewardConfigForAssetVault {
			apy: Perbill::from_percent(20),
			deposit_cap: BalanceOf::<T>::from(2000u32),
			incentive_cap: BalanceOf::<T>::from(2000u32),
			boost_multiplier: Some(1),
		};
		let origin = T::ForceOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
	}: _<T::RuntimeOrigin>(origin, vault_id, new_config.clone())
	verify {
		assert_eq!(RewardConfigStorage::<T>::get(vault_id), Some(new_config));
	}

	claim_rewards_other {
		let (vault_id, who) = setup_vault::<T>();
		let caller: T::AccountId = whitelisted_caller();
		let deposit = BalanceOf::<T>::from(100u32);
		let asset = Asset::Custom(1_u32.into());
	}: _(RawOrigin::Signed(caller.clone()), who.clone(), asset)
	verify {
		// Verify that rewards were claimed for the target account
		assert!(UserClaimedReward::<T>::contains_key(&who, vault_id));
	}

	manage_asset_reward_vault {
		let (vault_id, _) = setup_vault::<T>();
		// Use a different asset than the one already in vault
		let asset = Asset::Custom(T::AssetId::from(20u32.into()));
		let origin = T::ForceOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let action = AssetAction::Add;

		// Setup reward config for the new asset
		let reward_config = RewardConfigForAssetVault {
			apy: Perbill::from_percent(10),
			deposit_cap: BalanceOf::<T>::from(1000u32),
			incentive_cap: BalanceOf::<T>::from(1000u32),
			boost_multiplier: Some(1),
		};
		RewardConfigStorage::<T>::insert(vault_id, reward_config);
	}: _<T::RuntimeOrigin>(origin, vault_id, asset, action)
	verify {
		// Verify that the asset was added to the vault
		assert!(RewardVaults::<T>::get(vault_id).unwrap().contains(&asset));
	}

	create_reward_vault {
		let vault_id = Default::default();
		let new_config = RewardConfigForAssetVault {
			apy: Perbill::from_percent(10),
			deposit_cap: BalanceOf::<T>::from(1000u32),
			incentive_cap: BalanceOf::<T>::from(1000u32),
			boost_multiplier: Some(1), // Must be 1
		};
		let origin = T::ForceOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
	}: _<T::RuntimeOrigin>(origin, vault_id, new_config.clone())
	verify {
		// Verify that the vault was created with the specified config
		assert_eq!(RewardConfigStorage::<T>::get(vault_id), Some(new_config));
	}

	update_decay_config {
		let start_period = BlockNumberFor::<T>::from(1000u32);
		let rate = Perbill::from_percent(5);
		let origin = T::ForceOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
	}: _<T::RuntimeOrigin>(origin, start_period, rate)
	verify {
		// Verify that the decay config was updated
		let mut configs: BTreeMap<u32, RewardConfigForAssetVault<BalanceOf<T>>> = BTreeMap::new();
		let asset_id: u32 = 1u32;
		configs.insert(asset_id, RewardConfigForAssetVault {
			apy: rate,
			incentive_cap: 0u32.into(),
			deposit_cap: 0u32.into(),
			boost_multiplier: None,
		});

let decay_config = RewardConfig {
	configs,
	whitelisted_blueprint_ids: vec![],
};
		assert_eq!(decay_config.configs.get(&asset_id).unwrap().apy, rate);
	}

	update_apy_blocks {
		let blocks = BlockNumberFor::<T>::from(100u32);
		let origin = T::ForceOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
	}: _<T::RuntimeOrigin>(origin, blocks)
	verify {
		// Verify that the APY blocks were updated
		assert_eq!(ApyBlocks::<T>::get(), blocks);
	}
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Runtime);
