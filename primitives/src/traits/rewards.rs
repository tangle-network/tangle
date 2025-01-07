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
/// This trait provides functionality to record deposits, withdrawals, and service rewards,
/// as well as query total deposits for accounts.
pub trait RewardsManager<AccountId, AssetId, Balance, BlockNumber> {
	type Error;

	/// Records a deposit for an account with an optional lock multiplier.
	///
	/// # Parameters
	/// * `account_id` - The account making the deposit
	/// * `asset` - The asset being deposited
	/// * `amount` - The amount being deposited
	/// * `lock_multiplier` - Optional multiplier for locked deposits
	fn record_deposit(
		account_id: &AccountId,
		asset: Asset<AssetId>,
		amount: Balance,
		lock_multiplier: Option<LockMultiplier>,
	) -> Result<(), Self::Error>;

	/// Records a withdrawal for an account.
	///
	/// # Parameters
	/// * `account_id` - The account making the withdrawal
	/// * `asset` - The asset being withdrawn
	/// * `amount` - The amount being withdrawn
	fn record_withdrawal(
		account_id: &AccountId,
		asset: Asset<AssetId>,
		amount: Balance,
	) -> Result<(), Self::Error>;

	/// Records a service reward for an account.
	///
	/// # Parameters
	/// * `account_id` - The account receiving the reward
	/// * `asset` - The asset being rewarded
	/// * `amount` - The reward amount
	fn record_service_reward(
		account_id: &AccountId,
		asset: Asset<AssetId>,
		amount: Balance,
	) -> Result<(), Self::Error>;

	/// Queries the total deposit for an account and asset.
	///
	/// # Parameters
	/// * `account_id` - The account to query
	/// * `asset` - The asset to query
	fn query_total_deposit(
		account_id: &AccountId,
		asset: Asset<AssetId>,
	) -> Result<Balance, Self::Error>;
}

impl<AccountId, AssetId, Balance, BlockNumber>
	RewardsManager<AccountId, AssetId, Balance, BlockNumber> for ()
where
	Balance: Zero,
{
	type Error = &'static str;

	fn record_deposit(
		_account_id: &AccountId,
		_asset: Asset<AssetId>,
		_amount: Balance,
		_lock_multiplier: Option<LockMultiplier>,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn record_withdrawal(
		_account_id: &AccountId,
		_asset: Asset<AssetId>,
		_amount: Balance,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn record_service_reward(
		_account_id: &AccountId,
		_asset: Asset<AssetId>,
		_amount: Balance,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn query_total_deposit(
		_account_id: &AccountId,
		_asset: Asset<AssetId>,
	) -> Result<Balance, Self::Error> {
		Ok(Balance::zero())
	}
}
