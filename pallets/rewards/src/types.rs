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
use crate::Config;
use frame_support::traits::Currency;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, Perbill, RuntimeDebug};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Configuration for rewards associated with a specific asset.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
pub struct RewardConfigForAssetVault<Balance> {
	// The annual Perbillage yield (APY) for the asset, represented as a Perbill
	pub apy: Perbill,
	// The minimum amount required before the asset can be rewarded.
	pub incentive_cap: Balance,
	// The maximum amount of asset that can be deposited.
	pub deposit_cap: Balance,
	// Boost multiplier for this asset, if None boost multiplier is not enabled
	pub boost_multiplier: Option<u32>,
}

/// Configuration for rewards in the system.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct RewardConfig<VaultId, Balance> {
	// A map of asset IDs to their respective reward configurations.
	pub configs: BTreeMap<VaultId, RewardConfigForAssetVault<Balance>>,
	// A list of blueprint IDs that are whitelisted for rewards.
	pub whitelisted_blueprint_ids: Vec<u64>,
}

/// Asset action for vaults
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub enum AssetAction {
	Add,
	Remove,
}

/// Type for subaccounts
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub enum SubaccountType {
	RewardPot,
}

/// Pool-based reward accumulator for efficient delegator reward distribution.
///
/// This structure implements the "accumulated rewards per share" pattern,
/// which allows O(1) reward recording regardless of delegator count.
///
/// # How It Works
/// - When a reward is recorded: `accumulated_per_share += reward / total_staked`
/// - When delegator claims: `owed = stake * (current_accumulated - last_claimed_accumulated)`
///
/// This is the same pattern used in Cosmos SDK's x/distribution module.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
#[scale_info(skip_type_params(BlockNumber))]
pub struct OperatorRewardPool<Balance, BlockNumber> {
	/// Cumulative sum of (reward_i / total_stake_i) over all reward events.
	/// This value ONLY INCREASES and represents the total rewards per unit of stake.
	/// Stored as FixedU128 for high precision (18 decimal places).
	pub accumulated_rewards_per_share: sp_arithmetic::FixedU128,

	/// Current total amount staked with this operator by all delegators.
	/// Updated when delegators join/leave or change stake amounts.
	pub total_staked: Balance,

	/// Last block when this pool was updated (for monitoring/debugging).
	pub last_updated_block: BlockNumber,
}

impl<Balance: Default, BlockNumber: Default> Default for OperatorRewardPool<Balance, BlockNumber> {
	fn default() -> Self {
		Self {
			accumulated_rewards_per_share: sp_arithmetic::FixedU128::zero(),
			total_staked: Balance::default(),
			last_updated_block: BlockNumber::default(),
		}
	}
}

/// Tracks a delegator's position in the reward pool for calculating owed rewards.
///
/// The "debt" represents the delegator's snapshot of the pool's accumulated_per_share
/// when they last claimed rewards. The difference between the current pool accumulator
/// and this debt determines how much the delegator has earned since last claim.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct DelegatorRewardDebt<Balance> {
	/// Snapshot of the operator pool's accumulated_rewards_per_share when delegator last claimed.
	/// This creates a "checkpoint" - rewards earned since this point can be calculated as:
	/// owed = stake * (current_pool_accumulated - this_value)
	pub last_accumulated_per_share: sp_arithmetic::FixedU128,

	/// Delegator's current staked amount with this operator (cached for efficiency).
	/// MUST be updated whenever delegation amount changes.
	pub staked_amount: Balance,
}
