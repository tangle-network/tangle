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

use crate::{Config, Pallet, UserRewards, UserRewardsOf, BoostInfo, LockMultiplier};
use frame_support::traits::Currency;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::traits::{Zero, Saturating};
use tangle_primitives::{
    services::Asset,
    traits::rewards::RewardsManager,
};

impl<T: Config> RewardsManager<T::AccountId, T::AssetId, T::Balance, BlockNumberFor<T>> for Pallet<T> {
    fn record_delegation_reward(
        account_id: &T::AccountId,
        asset: Asset<T::AssetId>,
        amount: T::Balance,
        lock_multiplier: u32,
    ) -> Result<(), &'static str> {
        // Check if asset is whitelisted
        if !Self::is_asset_whitelisted(asset) {
            return Err("Asset not whitelisted");
        }

        // Get current rewards or create new if none exist
        let mut rewards = Self::user_rewards(account_id, asset);

        // Convert lock_multiplier to LockMultiplier enum
        let multiplier = match lock_multiplier {
            1 => LockMultiplier::OneMonth,
            2 => LockMultiplier::TwoMonths,
            3 => LockMultiplier::ThreeMonths,
            6 => LockMultiplier::SixMonths,
            _ => return Err("Invalid lock multiplier"),
        };

        // Update boost rewards
        rewards.boost_rewards = BoostInfo {
            amount: rewards.boost_rewards.amount.saturating_add(amount),
            multiplier,
            expiry: frame_system::Pallet::<T>::block_number().saturating_add(
                BlockNumberFor::<T>::from(30u32 * lock_multiplier as u32)
            ),
        };

        // Store updated rewards
        <crate::UserRewards<T>>::insert(account_id, asset, rewards);

        Ok(())
    }

    fn record_service_reward(
        account_id: &T::AccountId,
        asset: Asset<T::AssetId>,
        amount: T::Balance,
    ) -> Result<(), &'static str> {
        // Check if asset is whitelisted
        if !Self::is_asset_whitelisted(asset) {
            return Err("Asset not whitelisted");
        }

        // Get current rewards or create new if none exist
        let mut rewards = Self::user_rewards(account_id, asset);

        // Update service rewards
        rewards.service_rewards = rewards.service_rewards.saturating_add(amount);

        // Store updated rewards
        <crate::UserRewards<T>>::insert(account_id, asset, rewards);

        Ok(())
    }

    fn query_rewards(
        account_id: &T::AccountId,
        asset: Asset<T::AssetId>,
    ) -> Result<(T::Balance, T::Balance), &'static str> {
        let rewards = Self::user_rewards(account_id, asset);
        Ok((rewards.boost_rewards.amount, rewards.service_rewards))
    }

    fn query_delegation_rewards(
        account_id: &T::AccountId,
        asset: Asset<T::AssetId>,
    ) -> Result<T::Balance, &'static str> {
        let rewards = Self::user_rewards(account_id, asset);
        Ok(rewards.boost_rewards.amount)
    }

    fn query_service_rewards(
        account_id: &T::AccountId,
        asset: Asset<T::AssetId>,
    ) -> Result<T::Balance, &'static str> {
        let rewards = Self::user_rewards(account_id, asset);
        Ok(rewards.service_rewards)
    }
}