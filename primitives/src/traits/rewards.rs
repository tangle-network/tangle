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
use crate::types::rewards::LockMultiplier;
use sp_runtime::{traits::Zero, DispatchResult};

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

	/// Gets the maximum deposit cap for an asset at a given block number.
	/// This represents the maximum amount that can be deposited for this asset.
	///
	/// # Parameters
	/// * `asset` - The asset to query the deposit cap for
	///
	/// # Returns
	/// * `Ok(Balance)` - The maximum deposit cap for the asset
	/// * `Err(Self::Error)` - If there was an error retrieving the cap
	fn get_asset_deposit_cap_remaining(asset: Asset<AssetId>) -> Result<Balance, Self::Error>;

	/// Gets the incentive cap for an asset at a given block number.
	/// This represents the minimum amount required to receive full incentives.
	///
	/// # Parameters
	/// * `asset` - The asset to query the incentive cap for
	///
	/// # Returns
	/// * `Ok(Balance)` - The incentive cap for the asset
	/// * `Err(Self::Error)` - If there was an error retrieving the cap
	fn get_asset_incentive_cap(asset: Asset<AssetId>) -> Result<Balance, Self::Error>;
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

	fn get_asset_deposit_cap_remaining(_asset: Asset<AssetId>) -> Result<Balance, Self::Error> {
		Ok(Balance::zero())
	}

	fn get_asset_incentive_cap(_asset: Asset<AssetId>) -> Result<Balance, Self::Error> {
		Ok(Balance::zero())
	}
}

/// Defines the interface for managing reward distributions and contributions.
pub trait RewardDistributor<AccountId, Balance> {
	fn distribute_reward(operator: &AccountId, amount: Balance) -> DispatchResult;
}

/// A trait for recording rewards for services.
///
/// This trait is intended to be implemented by a rewards management pallet,
/// allowing other pallets (like a services pallet) to record that a reward
/// should be credited to an operator for a specific service.
pub trait RewardRecorder<AccountId, ServiceId, Balance> {
    /// The type of pricing model associated with the reward.
    type PricingModel;

    /// Records a reward for a given operator and service.
    ///
    /// This function should handle the accumulation of rewards, which can then
    /// be claimed by the operator at a later time.
    ///
    /// # Parameters
    /// - `operator`: The account ID of the operator who earned the reward.
    /// - `service_id`: The unique identifier of the service for which the reward is being recorded.
    /// - `amount`: The amount of the reward to be recorded.
    /// - `model`: A reference to the pricing model that determined this reward.
    fn record_reward(
        operator: &AccountId,
        service_id: ServiceId,
        amount: Balance,
        model: &Self::PricingModel,
    ) -> DispatchResult;
}

/// A no-operation implementation of `RewardRecorder`.
/// This can be used in runtime configurations where reward recording is not needed
/// or handled by a different mechanism.
impl<AccountId, ServiceId, Balance> RewardRecorder<AccountId, ServiceId, Balance> for () {
    type PricingModel = ();

    fn record_reward(
        _operator: &AccountId,
        _service_id: ServiceId,
        _amount: Balance,
        _model: &Self::PricingModel,
    ) -> DispatchResult {
        Ok(())
    }
}
