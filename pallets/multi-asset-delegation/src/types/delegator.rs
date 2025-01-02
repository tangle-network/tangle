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
use frame_support::{pallet_prelude::Get, BoundedVec};
use tangle_primitives::{services::Asset, BlueprintId};
use tangle_primitives::types::rewards::LockMultiplier;

/// Represents how a delegator selects which blueprints to work with.
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, Eq)]
pub enum DelegatorBlueprintSelection<MaxBlueprints: Get<u32>> {
	/// The delegator works with a fixed set of blueprints.
	Fixed(BoundedVec<BlueprintId, MaxBlueprints>),
	/// The delegator works with all available blueprints.
	All,
}

impl<MaxBlueprints: Get<u32>> Default for DelegatorBlueprintSelection<MaxBlueprints> {
	fn default() -> Self {
		DelegatorBlueprintSelection::Fixed(Default::default())
	}
}

/// Represents the status of a delegator.
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, Default)]
pub enum DelegatorStatus {
	/// The delegator is active.
	#[default]
	Active,
	/// The delegator has scheduled an exit to revoke all ongoing delegations.
	LeavingScheduled(RoundIndex),
}

/// Represents a request to withdraw a specific amount of an asset.
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct WithdrawRequest<AssetId: Encode + Decode, Balance> {
	/// The ID of the asset to be withdrawd.
	pub asset_id: Asset<AssetId>,
	/// The amount of the asset to be withdrawd.
	pub amount: Balance,
	/// The round in which the withdraw was requested.
	pub requested_round: RoundIndex,
}

/// Represents a request to reduce the bonded amount of a specific asset.
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct BondLessRequest<AccountId, AssetId: Encode + Decode, Balance, MaxBlueprints: Get<u32>> {
	/// The account ID of the operator.
	pub operator: AccountId,
	/// The ID of the asset to reduce the stake of.
	pub asset_id: Asset<AssetId>,
	/// The amount by which to reduce the stake.
	pub amount: Balance,
	/// The round in which the stake reduction was requested.
	pub requested_round: RoundIndex,
	/// The blueprint selection of the delegator.
	pub blueprint_selection: DelegatorBlueprintSelection<MaxBlueprints>,
}

/// Stores the state of a delegator, including deposits, delegations, and requests.
#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct DelegatorMetadata<
	AccountId,
	Balance,
	AssetId: Encode + Decode + TypeInfo,
	MaxWithdrawRequests: Get<u32>,
	MaxDelegations: Get<u32>,
	MaxUnstakeRequests: Get<u32>,
	MaxBlueprints: Get<u32>,
	BlockNumber,
	MaxLocks: Get<u32>,
> {
	/// A map of deposited assets and their respective amounts.
	pub deposits: BTreeMap<Asset<AssetId>, Balance>,
	/// A vector of withdraw requests.
	pub withdraw_requests: BoundedVec<WithdrawRequest<AssetId, Balance>, MaxWithdrawRequests>,
	/// A list of all current delegations.
	pub delegations:
		BoundedVec<BondInfoDelegator<AccountId, Balance, AssetId, MaxBlueprints, BlockNumber, MaxLocks>, MaxDelegations>,
	/// A vector of requests to reduce the bonded amount.
	pub delegator_unstake_requests:
		BoundedVec<BondLessRequest<AccountId, AssetId, Balance, MaxBlueprints>, MaxUnstakeRequests>,
	/// The current status of the delegator.
	pub status: DelegatorStatus,
}

impl<
		AccountId,
		Balance,
		AssetId: Encode + Decode + TypeInfo,
		MaxWithdrawRequests: Get<u32>,
		MaxDelegations: Get<u32>,
		MaxUnstakeRequests: Get<u32>,
		MaxBlueprints: Get<u32>,
		BlockNumber,
		MaxLocks: Get<u32>,
	> Default
	for DelegatorMetadata<
		AccountId,
		Balance,
		AssetId,
		MaxWithdrawRequests,
		MaxDelegations,
		MaxUnstakeRequests,
		MaxBlueprints,
		BlockNumber,
		MaxLocks,
	>
{
	fn default() -> Self {
		DelegatorMetadata {
			deposits: BTreeMap::new(),
			delegations: BoundedVec::default(),
			status: DelegatorStatus::default(),
			withdraw_requests: BoundedVec::default(),
			delegator_unstake_requests: BoundedVec::default(),
		}
	}
}

impl<
		AccountId,
		Balance,
		AssetId: Encode + Decode + TypeInfo,
		MaxWithdrawRequests: Get<u32>,
		MaxDelegations: Get<u32>,
		MaxUnstakeRequests: Get<u32>,
		MaxBlueprints: Get<u32>,
		BlockNumber,
		MaxLocks: Get<u32>,
	>
	DelegatorMetadata<
		AccountId,
		Balance,
		AssetId,
		MaxWithdrawRequests,
		MaxDelegations,
		MaxUnstakeRequests,
		MaxBlueprints,
		BlockNumber,
		MaxLocks,
	>
{
	/// Returns a reference to the vector of withdraw requests.
	pub fn get_withdraw_requests(&self) -> &Vec<WithdrawRequest<AssetId, Balance>> {
		&self.withdraw_requests
	}

	/// Returns a reference to the list of delegations.
	pub fn get_delegations(
		&self,
	) -> &Vec<BondInfoDelegator<AccountId, Balance, AssetId, MaxBlueprints, BlockNumber, MaxLocks>> {
		&self.delegations
	}

	/// Returns a reference to the vector of unstake requests.
	pub fn get_delegator_unstake_requests(
		&self,
	) -> &Vec<BondLessRequest<AccountId, AssetId, Balance, MaxBlueprints>> {
		&self.delegator_unstake_requests
	}

	/// Checks if the list of delegations is empty.
	pub fn is_delegations_empty(&self) -> bool {
		self.delegations.is_empty()
	}

	/// Calculates the total delegation amount for a specific asset.
	pub fn calculate_delegation_by_asset(&self, asset_id: Asset<AssetId>) -> Balance
	// Asset<AssetId>) -> Balance
	where
		Balance: Default + core::ops::AddAssign + Clone,
		AssetId: Eq + PartialEq,
	{
		let mut total = Balance::default();
		for stake in &self.delegations {
			if stake.asset_id == asset_id {
				total += stake.amount.clone();
			}
		}
		total
	}

	/// Returns a list of delegations to a specific operator.
	pub fn calculate_delegation_by_operator(
		&self,
		operator: AccountId,
	) -> Vec<&BondInfoDelegator<AccountId, Balance, AssetId, MaxBlueprints, BlockNumber, MaxLocks>>
	where
		AccountId: Eq + PartialEq,
	{
		self.delegations.iter().filter(|&stake| stake.operator == operator).collect()
	}
}

/// Represents a deposit of a specific asset.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Deposit<AssetId: Encode + Decode, Balance> {
	/// The amount of the asset deposited.
	pub amount: Balance,
	/// The ID of the deposited asset.
	pub asset_id: Asset<AssetId>,
}

/// Represents a stake between a delegator and an operator.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
pub struct BondInfoDelegator<AccountId, Balance, AssetId: Encode + Decode, MaxBlueprints: Get<u32>, BlockNumber, MaxLocks: Get<u32>>
{
	/// The account ID of the operator.
	pub operator: AccountId,
	/// The amount bonded.
	pub amount: Balance,
	/// The ID of the bonded asset.
	pub asset_id: Asset<AssetId>,
	/// The blueprint selection mode for this delegator.
	pub blueprint_selection: DelegatorBlueprintSelection<MaxBlueprints>,
	/// The locks associated with this delegation.
	pub locks: Option<BoundedVec<LockInfo<Balance, BlockNumber>, MaxLocks>>,
}

/// Struct to store the lock info 
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
pub struct LockInfo<Balance, BlockNumber> {
	pub amount: Balance,
	pub lock_multiplier: LockMultiplier,
	pub expiry_block: BlockNumber,
}
