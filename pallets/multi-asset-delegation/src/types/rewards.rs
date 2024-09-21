// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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
use sp_runtime::Percent;

/// Configuration for rewards associated with a specific asset.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct RewardConfigForAssetVault<Balance> {
	// The annual percentage yield (APY) for the asset, represented as a Percent
	pub apy: Percent,
	// The minimum amount required before the asset can be rewarded.
	pub cap: Balance,
}

/// Configuration for rewards in the system.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct RewardConfig<VaultId, Balance> {
	// A map of asset IDs to their respective reward configurations.
	pub configs: BTreeMap<VaultId, RewardConfigForAssetVault<Balance>>,
	// A list of blueprint IDs that are whitelisted for rewards.
	pub whitelisted_blueprint_ids: Vec<u32>,
}

/// Asset action for vaults
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub enum AssetAction {
	Add,
	Remove,
}
