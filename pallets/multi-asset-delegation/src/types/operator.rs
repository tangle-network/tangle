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

/// A snapshot of the operator state at the start of the round.
#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct OperatorSnapshot<AccountId, Balance, AssetId, MaxDelegations: Get<u32>> {
	/// The total value locked by the operator.
	pub stake: Balance,

	/// The rewardable delegations. This list is a subset of total delegators, where certain
	/// delegators are adjusted based on their scheduled status.
	pub delegations: BoundedVec<DelegatorBond<AccountId, Balance, AssetId>, MaxDelegations>,
}

impl<AccountId, Balance, AssetId, MaxDelegations: Get<u32>>
	OperatorSnapshot<AccountId, Balance, AssetId, MaxDelegations>
where
	AssetId: PartialEq + Ord + Copy,
	Balance: Default + core::ops::AddAssign + Copy,
{
	/// Calculates the total stake for a specific asset ID from all delegations.
	pub fn get_stake_by_asset_id(&self, asset_id: AssetId) -> Balance {
		let mut total_stake = Balance::default();
		for stake in &self.delegations {
			if stake.asset_id == asset_id {
				total_stake += stake.amount;
			}
		}
		total_stake
	}

	/// Calculates the total stake for each asset and returns a list of (asset_id, total_stake).
	pub fn get_total_stake_by_assets(&self) -> Vec<(AssetId, Balance)> {
		let mut stake_by_asset: BTreeMap<AssetId, Balance> = BTreeMap::new();

		for stake in &self.delegations {
			let entry = stake_by_asset.entry(stake.asset_id).or_default();
			*entry += stake.amount;
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
	AssetId,
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

impl<AccountId, Balance, AssetId, MaxDelegations: Get<u32>, MaxBlueprints: Get<u32>> Default
	for OperatorMetadata<AccountId, Balance, AssetId, MaxDelegations, MaxBlueprints>
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
	use core::ops::Add;
	use frame_support::{parameter_types, BoundedVec};
	use sp_runtime::traits::Zero;

	#[derive(Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, Clone, Copy, Default)]
	pub struct MockBalance(pub u32);

	impl Zero for MockBalance {
		fn zero() -> Self {
			MockBalance(0)
		}

		fn is_zero(&self) -> bool {
			self.0 == 0
		}
	}

	impl Add for MockBalance {
		type Output = Self;

		fn add(self, other: Self) -> Self {
			Self(self.0 + other.0)
		}
	}

	impl core::ops::AddAssign for MockBalance {
		fn add_assign(&mut self, other: Self) {
			self.0 += other.0;
		}
	}

	#[derive(Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, Clone, Copy)]
	pub struct MockAccountId(pub u32);

	#[derive(
		Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, Clone, Copy, Ord, PartialOrd,
	)]
	pub struct MockAssetId(pub u32);

	parameter_types! {
		pub const MaxDelegators: u32 = 10;
		pub const MaxUnstakeRequests: u32 = 10;
		pub const MaxBlueprints: u32 = 10;
	}

	#[test]
	fn get_stake_by_asset_id_should_work() {
		let snapshot: OperatorSnapshot<MockAccountId, MockBalance, MockAssetId, MaxDelegators> =
			OperatorSnapshot {
				stake: MockBalance(100),
				delegations: BoundedVec::try_from(vec![
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
				])
				.unwrap(),
			};

		assert_eq!(snapshot.get_stake_by_asset_id(MockAssetId(1)).0, 125);
		assert_eq!(snapshot.get_stake_by_asset_id(MockAssetId(2)).0, 25);
		assert_eq!(snapshot.get_stake_by_asset_id(MockAssetId(3)).0, 0);
	}

	#[test]
	fn get_total_stake_by_assets_should_work() {
		let snapshot: OperatorSnapshot<MockAccountId, MockBalance, MockAssetId, MaxDelegators> =
			OperatorSnapshot {
				stake: MockBalance(100),
				delegations: BoundedVec::try_from(vec![
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
				])
				.unwrap(),
			};

		let result = snapshot.get_total_stake_by_assets();
		let expected_result =
			vec![(MockAssetId(1), MockBalance(125)), (MockAssetId(2), MockBalance(125))];

		assert_eq!(result, expected_result);
	}
}
