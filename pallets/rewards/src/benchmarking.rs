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

benchmarks! {
	whitelist_asset {
		let asset: Asset<T::AssetId> = Asset::Custom(1u32.into());
	}: _(RawOrigin::Root, asset)
	verify {
		assert!(AllowedRewardAssets::<T>::get(&asset));
	}

	remove_asset {
		let asset: Asset<T::AssetId> = Asset::Custom(1u32.into());
		Rewards::<T>::whitelist_asset(RawOrigin::Root.into(), asset)?;
	}: _(RawOrigin::Root, asset)
	verify {
		assert!(!AllowedRewardAssets::<T>::get(&asset));
	}

	claim_rewards {
		let caller: T::AccountId = whitelisted_caller();
		let asset: Asset<T::AssetId> = Asset::Custom(1u32.into());
		let reward_type = RewardType::Restaking;
		let reward_amount: BalanceOf<T> = T::Currency::minimum_balance() * 100u32.into();

		// Setup: Whitelist asset and add rewards
		Rewards::<T>::whitelist_asset(RawOrigin::Root.into(), asset)?;
		let pallet_account = Rewards::<T>::account_id();
		T::Currency::make_free_balance_be(&pallet_account, reward_amount * 2u32.into());

		// Add rewards for the user
		UserRewards::<T>::insert(caller.clone(), asset, UserRewardInfo {
			restaking_rewards: reward_amount,
			boost_rewards: Zero::zero(),
			service_rewards: Zero::zero(),
		});

	}: _(RawOrigin::Signed(caller.clone()), asset, reward_type)
	verify {
		let rewards = UserRewards::<T>::get(caller.clone(), asset).unwrap_or_default();
		assert!(rewards.restaking_rewards.is_zero());
	}
}
