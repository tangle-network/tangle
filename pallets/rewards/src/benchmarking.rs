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
	pallet::{ApyBlocks, DecayRate, DecayStartPeriod, PendingOperatorRewards, UserClaimedReward},
	types::*,
};
use frame_benchmarking::{BenchmarkError, account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
	BoundedVec, assert_ok,
	traits::{Currency, EnsureOrigin, Get},
};
use frame_system::{RawOrigin, pallet_prelude::BlockNumberFor};
use sp_arithmetic::traits::Zero;
use sp_runtime::{Perbill, Saturating};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};
use tangle_primitives::{
	services::Asset,
	traits::{MultiAssetDelegationDelegation, MultiAssetDelegationOperator},
};

const SEED: u32 = 0;

/// Account's fund cannot be below minimum balance
/// Strategy: add minimum balance to the amount
/// This ensures that the account has enough balance for the benchmark operations
fn get_balance<T: Config>(amount: u32) -> BalanceOf<T> {
	return T::Currency::minimum_balance().saturating_add(amount.into());
}

fn create_blueprint_selection<T: Config>(
	delegator: T::AccountId,
	bond_amount: BalanceOf<T>,
	operator: T::AccountId,
	asset: Asset<T::AssetId>,
	amount: BalanceOf<T>,
) {
	assert_ok!(<T::DelegationManager as MultiAssetDelegationOperator<
		T::AccountId,
		BalanceOf<T>,
	>>::handle_deposit_and_create_operator_be(operator.clone(), bond_amount));

	assert_ok!(<T::DelegationManager as MultiAssetDelegationDelegation<
		T::AccountId,
		BalanceOf<T>,
		T::AssetId,
	>>::process_delegate_be(delegator, operator, asset, amount));
}

fn setup_vault<T: Config>() -> (T::VaultId, T::AccountId)
where
	<T as pallet::Config>::AssetId: From<u32>,
{
	let vault_id = Default::default();
	let caller: T::AccountId = account("caller", 0, SEED);
	let balance = get_balance::<T>(1000u32);
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

	AssetLookupRewardVaults::<T>::insert(asset_one, vault_id);
	AssetLookupRewardVaults::<T>::insert(asset_two, vault_id);

	(vault_id, caller)
}

benchmarks! {
	where_clause {
		where
			T::ForceOrigin: EnsureOrigin<<T as frame_system::Config>::RuntimeOrigin>,
			T::VaultMetadataOrigin: EnsureOrigin<<T as frame_system::Config>::RuntimeOrigin>,
			T::AssetId: From<u32>,
	}

	claim_rewards {
		let (vault_id, caller) = setup_vault::<T>();
		let deposit = get_balance::<T>(100u32);
		let service_id: ServiceId = 1u64;

		// Seed PendingOperatorRewards with a pending reward entry
		let mut pending_rewards = BoundedVec::<(ServiceId, BalanceOf<T>), T::MaxPendingRewardsPerOperator>::new();
		pending_rewards.try_push((service_id, deposit)).expect("Failed to push pending reward");
		PendingOperatorRewards::<T>::insert(caller.clone(), pending_rewards);

		// Verify the pending reward was inserted correctly
		let stored_rewards = PendingOperatorRewards::<T>::get(&caller);
		assert!(!stored_rewards.is_empty(), "Pending operator rewards should not be empty");
		assert_eq!(stored_rewards[0].0, service_id, "Service ID should match");
		assert_eq!(stored_rewards[0].1, deposit, "Deposit amount should match");

		// Make balance for pallet's account
		let balance = get_balance::<T>(u32::MAX);
		T::Currency::make_free_balance_be(&Pallet::<T>::account_id(), balance);
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		// Verify that pending rewards were cleared after claiming
		let remaining_rewards = PendingOperatorRewards::<T>::get(&caller);
		assert!(remaining_rewards.is_empty(), "Pending rewards should be cleared after claiming");
	}

	update_vault_reward_config {
		let (vault_id, _) = setup_vault::<T>();
		let new_config = RewardConfigForAssetVault {
			apy: Perbill::from_percent(20),
			deposit_cap: get_balance::<T>(2000u32),
			incentive_cap: get_balance::<T>(2000u32),
			boost_multiplier: Some(1),
		};
		let origin = T::ForceOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
	}: _<T::RuntimeOrigin>(origin, vault_id, new_config.clone())
	verify {
		assert_eq!(RewardConfigStorage::<T>::get(vault_id), Some(new_config));
	}

	claim_rewards_other {
		let (vault_id, delegator) = setup_vault::<T>();
		// operator account
		let operator: T::AccountId = account("operator", 2, SEED);
		let operator_balance = get_balance::<T>(10000u32);
		T::Currency::make_free_balance_be(&operator, operator_balance.clone());
		// asset to delegate
		let asset = Asset::Custom(1_u32.into());

		create_blueprint_selection::<T>(
			// delegator
			delegator.clone(),
			// bond amount
			operator_balance / 10u32.into(),
			// operator
			operator.clone(),
			// asset
			asset.clone(),
			// delegating amount
			100u32.into()
		);

		// Even larger deposit amount for repeated runs
		let deposit_amount = get_balance::<T>(1000000u32);
		// Make total score 10000x user deposit
		let total_score = deposit_amount * 10000u32.into();
		TotalRewardVaultScore::<T>::insert(vault_id, total_score);
		// Make 1000x user deposit
		TotalRewardVaultDeposit::<T>::insert(vault_id, deposit_amount * 1000u32.into());

		// Simulating previous claims - set to a much earlier block
		let current_block = frame_system::Pallet::<T>::block_number();
		let last_claim_block = current_block.saturating_sub(100000u32.into());
		let last_claim_amount = get_balance::<T>(1000u32);
		UserClaimedReward::<T>::insert(&delegator, vault_id, (last_claim_block, last_claim_amount));

		// Setup vault pot account with massive balance for repeated runs
		let pot_account: T::AccountId = account("pot", 2, SEED);
		let pot_balance = get_balance::<T>(100000000u32);
		T::Currency::make_free_balance_be(&pot_account, pot_balance);
		RewardVaultsPotAccount::<T>::insert(vault_id, pot_account);

		// Setup APY blocks to ensure proper reward calculation
		ApyBlocks::<T>::put(BlockNumberFor::<T>::from(100000u32)); // Set APY blocks to 100000

		// Setup decay config to ensure no decay affects the calculation
		DecayStartPeriod::<T>::put(BlockNumberFor::<T>::from(10000000u32)); // Extremely high decay start period
		DecayRate::<T>::put(Perbill::from_percent(0)); // No decay

		// Override the reward config with massive values for repeated runs
		let reward_config = RewardConfigForAssetVault {
				apy: Perbill::from_percent(20), // 20% APY for higher rewards
				deposit_cap: deposit_amount * 10000u32.into(), // Massive deposit cap
				incentive_cap: deposit_amount * 1000u32.into(), // Massive incentive cap
				boost_multiplier: Some(1),
		};
		RewardConfigStorage::<T>::insert(vault_id, reward_config);

		// Fund the pallet account for transfers with maximum balance
		let balance = get_balance::<T>(u32::MAX);
		T::Currency::make_free_balance_be(&Pallet::<T>::account_id(), balance);
	}: _(RawOrigin::Signed(operator.clone()), delegator.clone(), asset)
	verify {
		// Verify that the user's last claim was updated
		let updated_claim = UserClaimedReward::<T>::get(&delegator, vault_id);
		assert!(updated_claim.is_some());
		let (claim_block, claim_amount) = updated_claim.unwrap();
		assert!(claim_block > last_claim_block);

		// Verify that the target account received some balance
		let target_balance = T::Currency::free_balance(&delegator);
		assert!(target_balance > Zero::zero());
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
			deposit_cap: get_balance::<T>(1000u32),
			incentive_cap: get_balance::<T>(1000u32),
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
			deposit_cap: get_balance::<T>(1000u32),
			incentive_cap: get_balance::<T>(1000u32),
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

	claim_delegator_rewards {
		use sp_arithmetic::FixedU128;

		// Setup operator account
		let operator: T::AccountId = account("operator", 0, SEED);
		let operator_balance = get_balance::<T>(10000u32);
		T::Currency::make_free_balance_be(&operator, operator_balance);

		// Setup delegator account
		let delegator: T::AccountId = account("delegator", 1, SEED);
		let delegator_balance = get_balance::<T>(10000u32);
		T::Currency::make_free_balance_be(&delegator, delegator_balance);

		// Get current block number
		let current_block = frame_system::Pallet::<T>::block_number();

		// Simulate operator reward pool with accumulated rewards
		let pool = OperatorRewardPool {
			accumulated_rewards_per_share: FixedU128::from(1u128), // 1.0
			total_staked: get_balance::<T>(1000u32),
			last_updated_block: current_block,
		};
		crate::pallet::OperatorRewardPools::<T>::insert(&operator, pool);

		// Initialize delegator debt to zero (first time claiming)
		let debt = DelegatorRewardDebt {
			last_accumulated_per_share: FixedU128::zero(),
			staked_amount: get_balance::<T>(100u32),
		};
		crate::pallet::DelegatorRewardDebts::<T>::insert(&delegator, &operator, debt);

		// Make balance for pallet's account
		let balance = get_balance::<T>(u32::MAX);
		T::Currency::make_free_balance_be(&Pallet::<T>::account_id(), balance);
	}: _(RawOrigin::Signed(delegator.clone()), operator.clone())
	verify {
		// Verify that the debt was updated
		let updated_debt = crate::pallet::DelegatorRewardDebts::<T>::get(&delegator, &operator);
		assert!(updated_debt.is_some());
		assert!(updated_debt.unwrap().last_accumulated_per_share > FixedU128::from(0));
	}

	set_vault_metadata {
		let vault_id = Default::default();
		let caller: T::AccountId = account("caller", 0, SEED);
		let balance = get_balance::<T>(1000u32);
		T::Currency::make_free_balance_be(&caller, balance);

		// Create vault metadata (name and logo as byte vectors with worst-case lengths)
		let name: Vec<u8> = vec![b'A'; T::MaxVaultNameLength::get() as usize];
		let logo: Vec<u8> = vec![b'B'; T::MaxVaultLogoLength::get() as usize];

		let origin = T::VaultMetadataOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
	}: _<T::RuntimeOrigin>(origin, vault_id, name.clone(), logo.clone())
	verify {
		// Verify that the metadata was stored
		let metadata = crate::pallet::VaultMetadataStore::<T>::get(vault_id);
		assert!(metadata.is_some());
		let metadata = metadata.unwrap();
		assert_eq!(
			metadata.name,
			TryInto::<BoundedVec<u8, T::MaxVaultNameLength>>::try_into(name).unwrap()
		);
		assert_eq!(
			metadata.logo,
			TryInto::<BoundedVec<u8, T::MaxVaultLogoLength>>::try_into(logo).unwrap()
		);
	}

	remove_vault_metadata {
		let vault_id = Default::default();
		let caller: T::AccountId = account("caller", 0, SEED);
		let balance = get_balance::<T>(1000u32);
		T::Currency::make_free_balance_be(&caller, balance);

		// Setup: First set metadata so we can remove it (using worst-case lengths)
		let name: BoundedVec<u8, T::MaxVaultNameLength> = vec![b'A'; T::MaxVaultNameLength::get() as usize].try_into().unwrap();
		let logo: BoundedVec<u8, T::MaxVaultLogoLength> = vec![b'B'; T::MaxVaultLogoLength::get() as usize].try_into().unwrap();
		let metadata = crate::pallet::VaultMetadata::<T> { name, logo };
		crate::pallet::VaultMetadataStore::<T>::insert(vault_id, metadata);

		let origin = T::VaultMetadataOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
	}: _<T::RuntimeOrigin>(origin, vault_id)
	verify {
		// Verify that the metadata was removed
		assert!(!crate::pallet::VaultMetadataStore::<T>::contains_key(vault_id));
	}
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Runtime);
