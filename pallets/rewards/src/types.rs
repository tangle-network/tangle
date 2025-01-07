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
use crate::Config;
use frame_support::traits::Currency;
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::Percent;
use sp_runtime::RuntimeDebug;
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
use tangle_primitives::types::rewards::UserRewards;
use tangle_primitives::{services::Asset, types::RoundIndex};

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Configuration for rewards associated with a specific asset.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
pub struct RewardConfigForAssetVault<Balance> {
	// The annual percentage yield (APY) for the asset, represented as a Percent
	pub apy: Percent,
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
