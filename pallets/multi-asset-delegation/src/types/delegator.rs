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

/// Represents the status of a delegator.
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, Default)]
pub enum DelegatorStatus {
	/// The delegator is active.
	#[default]
	Active,
	/// The delegator has scheduled an exit to revoke all ongoing delegations.
	LeavingScheduled(RoundIndex),
}

/// Represents a request to unstake a specific amount of an asset.
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct UnstakeRequest<AssetId, Balance> {
	/// The ID of the asset to be unstaked.
	pub asset_id: AssetId,
	/// The amount of the asset to be unstaked.
	pub amount: Balance,
	/// The round in which the unstake was requested.
	pub requested_round: RoundIndex,
}

/// Represents a request to reduce the bonded amount of a specific asset.
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct BondLessRequest<AssetId, Balance> {
	/// The ID of the asset to reduce the bond of.
	pub asset_id: AssetId,
	/// The amount by which to reduce the bond.
	pub amount: Balance,
	/// The round in which the bond reduction was requested.
	pub requested_round: RoundIndex,
}

/// Stores the state of a delegator, including deposits, delegations, and requests.
#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct DelegatorMetadata<AccountId, Balance, AssetId: Encode + Decode + TypeInfo> {
	/// A map of deposited assets and their respective amounts.
	pub deposits: BTreeMap<AssetId, Balance>,
	/// An optional unstake request, with only one allowed at a time.
	pub unstake_request: Option<UnstakeRequest<AssetId, Balance>>,
	/// A list of all current delegations.
	pub delegations: Vec<Bond<AccountId, Balance, AssetId>>,
	/// An optional request to reduce the bonded amount, with only one allowed at a time.
	pub delegator_bond_less_request: Option<BondLessRequest<AssetId, Balance>>,
	/// The current status of the delegator.
	pub status: DelegatorStatus,
}

impl<AccountId, Balance, AssetId: Encode + Decode + TypeInfo> Default for DelegatorMetadata<AccountId, Balance, AssetId> {
	fn default() -> Self {
		DelegatorMetadata {
			deposits: BTreeMap::new(),
			delegations: Vec::new(),
			status: DelegatorStatus::default(),
			unstake_request: None,
			delegator_bond_less_request: None,
		}
	}
}

/// Represents a deposit of a specific asset.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Deposit<AssetId, Balance> {
	/// The amount of the asset deposited.
	pub amount: Balance,
	/// The ID of the deposited asset.
	pub asset_id: AssetId,
}

/// Represents a bond between a delegator and an operator.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Bond<AccountId, Balance, AssetId> {
	/// The account ID of the operator.
	pub operator: AccountId,
	/// The amount bonded.
	pub amount: Balance,
	/// The ID of the bonded asset.
	pub asset_id: AssetId,
}
