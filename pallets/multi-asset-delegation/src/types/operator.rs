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

/// A snapshot of the operator state at the start of the round.
#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct OperatorSnapshot<AccountId, Balance, AssetId> {
    /// The total value locked by the operator.
    pub bond: Balance,

    /// The rewardable delegations. This list is a subset of total delegators, where certain
    /// delegators are adjusted based on their scheduled status.
    pub delegations: Vec<Bond<AccountId, Balance, AssetId>>,

    /// The total counted value locked for the operator, including the self bond + total staked by
    /// top delegators.
    pub total: Balance,
}

/// The activity status of the operator.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum OperatorStatus {
    /// Committed to be online.
    Active,
    /// Temporarily inactive and excused for inactivity.
    Inactive,
    /// Bonded until the specified round.
    Leaving(RoundIndex),
}

impl Default for OperatorStatus {
    fn default() -> OperatorStatus {
        OperatorStatus::Active
    }
}

/// A request scheduled to change the operator self-bond.
#[derive(PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo, Eq)]
pub struct OperatorBondLessRequest<Balance> {
    /// The amount by which the bond is to be decreased.
    pub amount: Balance,
    /// The round in which the request was made.
    pub request_time: RoundIndex,
}

/// Stores the metadata of an operator.
#[derive(Encode, Decode, RuntimeDebug, TypeInfo, Clone, Eq, PartialEq)]
pub struct OperatorMetadata<Balance> {
    /// The operator's self-bond amount.
    pub bond: Balance,
    /// The total number of delegations to this operator.
    pub delegation_count: u32,
    /// An optional pending request to decrease the operator's self-bond, with only one allowed at any given time.
    pub request: Option<OperatorBondLessRequest<Balance>>,
    /// The current status of the operator.
    pub status: OperatorStatus,
}
