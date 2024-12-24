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
use crate::types::{BalanceOf, LockMultiplier, OperatorStatus};
use sp_runtime::{traits::Zero, Percent};
use sp_std::prelude::*;
use tangle_primitives::{
	services::Asset, traits::MultiAssetDelegationInfo, BlueprintId, RoundIndex,
};

/// Trait for handling reward operations
pub trait RewardsHandler<AccountId, AssetId, Balance> {
	/// Record rewards for a specific user and asset
	fn record_rewards(
		beneficiary: AccountId,
		asset: Asset<AssetId>,
		amount: Balance,
		reward_type: RewardType,
		boost_multiplier: Option<u32>,
		boost_expiry: Option<<AccountId as frame_system::Config>::BlockNumber>,
	) -> DispatchResult;

	/// Record a deposit for a specific account and asset with a lock multiplier
	fn record_deposit(
		account_id: AccountId,
		asset_id: AssetId,
		amount: Balance,
		lock_multiplier: LockMultiplier,
	) -> DispatchResult;

	/// Query the deposit for a specific account and asset
	fn query_deposit(account_id: AccountId, asset_id: AssetId) -> Balance;

	/// Record a service payment for a specific account and asset
	fn record_service_payment(
		account_id: AccountId,
		asset_id: AssetId,
		amount: Balance,
	) -> DispatchResult;

	/// Check if an asset is whitelisted for rewards
	fn is_asset_whitelisted(asset: Asset<AssetId>) -> bool;
}

impl<T: crate::Config> RewardsHandler<T::AccountId, T::AssetId, BalanceOf<T>> for crate::Pallet<T> {
	fn record_rewards(
		beneficiary: T::AccountId,
		asset: Asset<T::AssetId>,
		amount: BalanceOf<T>,
		reward_type: RewardType,
		boost_multiplier: Option<u32>,
		boost_expiry: Option<T::BlockNumber>,
	) -> DispatchResult {
		// Check if asset is whitelisted
		ensure!(Self::is_asset_whitelisted(asset), Error::<T>::AssetNotWhitelisted);

		// Update rewards storage
		UserRewards::<T>::mutate(beneficiary.clone(), asset, |rewards| {
			match reward_type {
				RewardType::Restaking => {
					rewards.restaking_rewards = rewards.restaking_rewards.saturating_add(amount);
				},
				RewardType::Boost => {
					if let (Some(multiplier), Some(expiry)) = (boost_multiplier, boost_expiry) {
						rewards.boost_rewards = BoostInfo {
							amount: rewards.boost_rewards.amount.saturating_add(amount),
							multiplier,
							expiry,
						};
					}
				},
				RewardType::Service => {
					rewards.service_rewards = rewards.service_rewards.saturating_add(amount);
				},
			}
		});

		// Emit event
		Self::deposit_event(Event::RewardsAdded {
			account: beneficiary,
			asset,
			amount,
			reward_type,
			boost_multiplier,
			boost_expiry,
		});

		Ok(())
	}

	fn record_deposit(
		account_id: T::AccountId,
		asset_id: T::AssetId,
		amount: BalanceOf<T>,
		lock_multiplier: LockMultiplier,
	) -> DispatchResult {
		let asset = Asset::new(asset_id);
		ensure!(Self::is_asset_whitelisted(asset), Error::<T>::AssetNotWhitelisted);

		UserRewards::<T>::mutate(account_id.clone(), asset, |rewards| {
			rewards.restaking_rewards = rewards.restaking_rewards.saturating_add(amount);
		});

		Self::deposit_event(Event::DepositRecorded {
			account: account_id,
			asset,
			amount,
			lock_multiplier,
		});

		Ok(())
	}

	fn query_deposit(account_id: T::AccountId, asset_id: T::AssetId) -> BalanceOf<T> {
		let asset = Asset::new(asset_id);
		UserRewards::<T>::get(account_id, asset).restaking_rewards
	}

	fn record_service_payment(
		account_id: T::AccountId,
		asset_id: T::AssetId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		let asset = Asset::new(asset_id);
		ensure!(Self::is_asset_whitelisted(asset), Error::<T>::AssetNotWhitelisted);

		UserRewards::<T>::mutate(account_id.clone(), asset, |rewards| {
			rewards.service_rewards = rewards.service_rewards.saturating_add(amount);
		});

		Self::deposit_event(Event::ServicePaymentRecorded {
			account: account_id,
			asset,
			amount,
		});

		Ok(())
	}

	fn is_asset_whitelisted(asset: Asset<T::AssetId>) -> bool {
		AllowedRewardAssets::<T>::get(&asset)
	}
}
