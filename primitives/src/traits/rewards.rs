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

use crate::services::Asset;
use frame_support::traits::Currency;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::traits::Zero;

/// Trait for managing rewards in the Tangle network.
/// This trait allows other pallets to record and query rewards.
pub trait RewardsManager<AccountId, AssetId, Balance, BlockNumber> {
	/// Record a delegation deposit reward.
	/// This is used by the multi-asset-delegation pallet to record rewards from delegations.
	///
	/// Parameters:
	/// - `account_id`: The account receiving the reward
	/// - `asset`: The asset being delegated
	/// - `amount`: The amount of the reward
	/// - `lock_multiplier`: The multiplier for the lock period (e.g., 1 for 1 month, 3 for 3 months)
	fn record_delegation_reward(
		account_id: &AccountId,
		asset: Asset<AssetId>,
		amount: Balance,
		lock_multiplier: u32,
	) -> Result<(), &'static str>;

	/// Record a service payment reward.
	/// This is used by the services pallet to record rewards from service payments.
	///
	/// Parameters:
	/// - `account_id`: The account receiving the reward
	/// - `asset`: The asset used for payment
	/// - `amount`: The amount of the reward
	fn record_service_reward(
		account_id: &AccountId,
		asset: Asset<AssetId>,
		amount: Balance,
	) -> Result<(), &'static str>;

	/// Query the total rewards for an account and asset.
	/// Returns a tuple of (delegation_rewards, service_rewards).
	///
	/// Parameters:
	/// - `account_id`: The account to query
	/// - `asset`: The asset to query
	fn query_rewards(
		account_id: &AccountId,
		asset: Asset<AssetId>,
	) -> Result<(Balance, Balance), &'static str>;

	/// Query only the delegation rewards for an account and asset.
	fn query_delegation_rewards(
		account_id: &AccountId,
		asset: Asset<AssetId>,
	) -> Result<Balance, &'static str>;

	/// Query only the service rewards for an account and asset.
	fn query_service_rewards(
		account_id: &AccountId,
		asset: Asset<AssetId>,
	) -> Result<Balance, &'static str>;
}

impl<AccountId, AssetId, Balance, BlockNumber>
	RewardsManager<AccountId, AssetId, Balance, BlockNumber> for ()
where
	Balance: Zero,
{
	fn record_delegation_reward(
		_account_id: &AccountId,
		_asset: Asset<AssetId>,
		_amount: Balance,
		_lock_multiplier: u32,
	) -> Result<(), &'static str> {
		Ok(())
	}

	fn record_service_reward(
		_account_id: &AccountId,
		_asset: Asset<AssetId>,
		_amount: Balance,
	) -> Result<(), &'static str> {
		Ok(())
	}

	fn query_rewards(
		_account_id: &AccountId,
		_asset: Asset<AssetId>,
	) -> Result<(Balance, Balance), &'static str> {
		Ok((Balance::zero(), Balance::zero()))
	}

	fn query_delegation_rewards(
		_account_id: &AccountId,
		_asset: Asset<AssetId>,
	) -> Result<Balance, &'static str> {
		Ok(Balance::zero())
	}

	fn query_service_rewards(
		_account_id: &AccountId,
		_asset: Asset<AssetId>,
	) -> Result<Balance, &'static str> {
		Ok(Balance::zero())
	}
}
