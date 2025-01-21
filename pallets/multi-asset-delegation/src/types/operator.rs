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
use frame_support::{pallet_prelude::*, BoundedVec};
use sp_runtime::traits::CheckedAdd;
use tangle_primitives::services::Asset;

/// A snapshot of the operator state at the start of the round.
#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct OperatorSnapshot<AccountId, Balance, AssetId: Encode + Decode, MaxDelegations: Get<u32>>
{
	/// The total value locked by the operator.
	pub stake: Balance,

	/// The rewardable delegations. This list is a subset of total delegators, where certain
	/// delegators are adjusted based on their scheduled status.
	pub delegations: BoundedVec<DelegatorBond<AccountId, Balance, AssetId>, MaxDelegations>,
}

impl<AccountId, Balance, AssetId: Encode + Decode, MaxDelegations: Get<u32>>
	OperatorSnapshot<AccountId, Balance, AssetId, MaxDelegations>
where
	AssetId: PartialEq + Ord + Copy,
	Balance: Default + core::ops::AddAssign + Copy + CheckedAdd,
{
	/// Calculates the total stake for a specific asset ID from all delegations.
	pub fn get_stake_by_asset_id(&self, asset_id: Asset<AssetId>) -> Balance {
		let mut total_stake = Balance::default();
		for stake in &self.delegations {
			if stake.asset_id == asset_id {
				total_stake = total_stake.checked_add(&stake.amount).unwrap_or(total_stake);
			}
		}
		total_stake
	}

	/// Calculates the total stake for each asset and returns a list of (asset_id, total_stake).
	pub fn get_total_stake_by_assets(&self) -> Vec<(Asset<AssetId>, Balance)> {
		let mut stake_by_asset: BTreeMap<Asset<AssetId>, Balance> = BTreeMap::new();

		for stake in &self.delegations {
			let entry = stake_by_asset.entry(stake.asset_id).or_default();
			*entry = entry.checked_add(&stake.amount).unwrap_or(*entry);
		}

		stake_by_asset.into_iter().collect()
	}
}

/// The activity status of the operator.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Default)]
pub enum OperatorStatus {
	/// Committed to be online.
	#[default]
	Active,
	/// Temporarily inactive and excused for inactivity.
	Inactive,
	/// Bonded until the specified round.
	Leaving(RoundIndex),
}

/// A request scheduled to change the operator self-stake.
#[derive(PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo, Eq)]
pub struct OperatorBondLessRequest<Balance> {
	/// The amount by which the stake is to be decreased.
	pub amount: Balance,
	/// The round in which the request was made.
	pub request_time: RoundIndex,
}

/// Stores the metadata of an operator.
#[derive(Encode, Decode, RuntimeDebug, TypeInfo, Clone, Eq, PartialEq)]
pub struct OperatorMetadata<
	AccountId,
	Balance,
	AssetId: Encode + Decode,
	MaxDelegations: Get<u32>,
	MaxBlueprints: Get<u32>,
> {
	/// The operator's self-stake amount.
	pub stake: Balance,
	/// The total number of delegations to this operator.
	pub delegation_count: u32,
	/// An optional pending request to decrease the operator's self-stake, with only one allowed at
	/// any given time.
	pub request: Option<OperatorBondLessRequest<Balance>>,
	/// A list of all current delegations.
	pub delegations: BoundedVec<DelegatorBond<AccountId, Balance, AssetId>, MaxDelegations>,
	/// The current status of the operator.
	pub status: OperatorStatus,
	/// The set of blueprint IDs this operator works with.
	pub blueprint_ids: BoundedVec<u32, MaxBlueprints>,
}

impl<
		AccountId,
		Balance,
		AssetId: Encode + Decode,
		MaxDelegations: Get<u32>,
		MaxBlueprints: Get<u32>,
	> Default for OperatorMetadata<AccountId, Balance, AssetId, MaxDelegations, MaxBlueprints>
where
	Balance: Default,
{
	fn default() -> Self {
		Self {
			stake: Balance::default(),
			delegation_count: 0,
			request: None,
			delegations: BoundedVec::default(),
			status: OperatorStatus::default(),
			blueprint_ids: BoundedVec::default(),
		}
	}
}

/// Represents a stake for an operator
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
pub struct DelegatorBond<AccountId, Balance, AssetId: Encode + Decode> {
	/// The account ID of the delegator.
	pub delegator: AccountId,
	/// The amount bonded.
	pub amount: Balance,
	/// The ID of the bonded asset.
	pub asset_id: Asset<AssetId>,
}
