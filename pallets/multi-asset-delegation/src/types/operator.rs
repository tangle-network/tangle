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
	pub delegations: Vec<DelegatorBond<AccountId, Balance, AssetId>>,
}

impl<AccountId, Balance, AssetId> OperatorSnapshot<AccountId, Balance, AssetId>
where
	AssetId: PartialEq + Ord + Copy,
	Balance: Default + core::ops::AddAssign + Copy,
{
	/// Calculates the total stake for a specific asset ID from all delegations.
	pub fn get_stake_by_asset_id(&self, asset_id: AssetId) -> Balance {
		let mut total_stake = Balance::default();
		for bond in &self.delegations {
			if bond.asset_id == asset_id {
				total_stake += bond.amount;
			}
		}
		total_stake
	}

	/// Calculates the total stake for each asset and returns a list of (asset_id, total_stake).
	pub fn get_total_stake_by_assets(&self) -> Vec<(AssetId, Balance)> {
		let mut stake_by_asset: BTreeMap<AssetId, Balance> = BTreeMap::new();

		for bond in &self.delegations {
			let entry = stake_by_asset.entry(bond.asset_id).or_default();
			*entry += bond.amount;
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
pub struct OperatorMetadata<AccountId, Balance, AssetId> {
	/// The operator's self-bond amount.
	pub bond: Balance,
	/// The total number of delegations to this operator.
	pub delegation_count: u32,
	/// An optional pending request to decrease the operator's self-bond, with only one allowed at
	/// any given time.
	pub request: Option<OperatorBondLessRequest<Balance>>,
	/// A list of all current delegations.
	pub delegations: Vec<DelegatorBond<AccountId, Balance, AssetId>>,
	/// The current status of the operator.
	pub status: OperatorStatus,
}

/// Represents a bond for an operator
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
pub struct DelegatorBond<AccountId, Balance, AssetId> {
	/// The account ID of the delegator.
	pub delegator: AccountId,
	/// The amount bonded.
	pub amount: Balance,
	/// The ID of the bonded asset.
	pub asset_id: AssetId,
}

// ------ Test for helper functions ------ //

#[cfg(test)]
mod tests {
	use super::*;
	use std::ops::AddAssign;

	#[derive(
		Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, Clone, Copy, PartialOrd, Ord,
	)]
	pub struct MockAssetId(pub u32);

	#[derive(Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, Clone, Copy)]
	pub struct MockAccountId(pub u64);

	#[derive(Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, Clone, Copy, Default)]
	pub struct MockBalance(pub u64);

	impl AddAssign for MockBalance {
		fn add_assign(&mut self, other: Self) {
			*self = MockBalance(self.0 + other.0);
		}
	}

	#[test]
	fn get_stake_by_asset_id_should_work() {
		let snapshot = OperatorSnapshot {
			bond: MockBalance(100),
			delegations: vec![
				DelegatorBond {
					delegator: MockAccountId(1),
					amount: MockBalance(50),
					asset_id: MockAssetId(1),
				},
				DelegatorBond {
					delegator: MockAccountId(2),
					amount: MockBalance(75),
					asset_id: MockAssetId(1),
				},
				DelegatorBond {
					delegator: MockAccountId(3),
					amount: MockBalance(25),
					asset_id: MockAssetId(2),
				},
			],
		};

		assert_eq!(snapshot.get_stake_by_asset_id(MockAssetId(1)), MockBalance(125));
		assert_eq!(snapshot.get_stake_by_asset_id(MockAssetId(2)), MockBalance(25));
		assert_eq!(snapshot.get_stake_by_asset_id(MockAssetId(3)), MockBalance(0));
	}

	#[test]
	fn get_total_stake_by_assets_should_work() {
		let snapshot = OperatorSnapshot {
			bond: MockBalance(100),
			delegations: vec![
				DelegatorBond {
					delegator: MockAccountId(1),
					amount: MockBalance(50),
					asset_id: MockAssetId(1),
				},
				DelegatorBond {
					delegator: MockAccountId(2),
					amount: MockBalance(75),
					asset_id: MockAssetId(1),
				},
				DelegatorBond {
					delegator: MockAccountId(3),
					amount: MockBalance(25),
					asset_id: MockAssetId(2),
				},
				DelegatorBond {
					delegator: MockAccountId(4),
					amount: MockBalance(100),
					asset_id: MockAssetId(2),
				},
			],
		};

		let result = snapshot.get_total_stake_by_assets();
		let expected_result =
			vec![(MockAssetId(1), MockBalance(125)), (MockAssetId(2), MockBalance(125))];

		assert_eq!(result, expected_result);
	}
}
