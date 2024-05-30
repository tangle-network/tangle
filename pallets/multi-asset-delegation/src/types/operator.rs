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

#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
/// Snapshot of Operator state at the start of the round
pub struct OperatorSnapshot<AccountId, Balance, AssetId> {
	/// The total value locked by the Operator.
	pub bond: Balance,

	/// The rewardable delegations. This list is a subset of total delegators, where certain
	/// delegators are adjusted based on their scheduled
	pub delegations: Vec<Bond<AccountId, Balance, AssetId>>,

	/// The total counted value locked for the Operator, including the self bond + total staked by
	/// top delegators.
	pub total: Balance,
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
/// The activity status of the Operator
pub enum OperatorStatus {
	/// Committed to be online
	Active,
	/// Temporarily inactive and excused for inactivity
	Inactive,
	/// Bonded until the inner round
	Leaving(RoundIndex),
}

impl Default for OperatorStatus {
	fn default() -> OperatorStatus {
		OperatorStatus::Active
	}
}

#[derive(PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo, Eq)]
/// Request scheduled to change the Operator Operator self-bond
pub struct OperatorBondLessRequest<Balance> {
	pub amount: Balance,
	pub request_time: RoundIndex,
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, Clone, Eq, PartialEq)]
pub struct OperatorMetadata<Balance> {
	/// This Operator's self bond amount
	pub bond: Balance,
	/// Total number of delegations to this Operator
	pub delegation_count: u32,
	/// Maximum 1 pending request to decrease Operator self bond at any given time
	pub request: Option<OperatorBondLessRequest<Balance>>,
	/// Current status of the Operator
	pub status: OperatorStatus,
}
