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

impl<AccountId, Balance, AssetId: Encode + Decode + TypeInfo> Default
	for DelegatorMetadata<AccountId, Balance, AssetId>
{
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

impl<AccountId, Balance, AssetId: Encode + Decode + TypeInfo>
	DelegatorMetadata<AccountId, Balance, AssetId>
{
	/// Returns a reference to the unstake request, if it exists.
	pub fn get_unstake_request(&self) -> Option<&UnstakeRequest<AssetId, Balance>> {
		self.unstake_request.as_ref()
	}

	/// Returns a reference to the list of delegations.
	pub fn get_delegations(&self) -> &Vec<Bond<AccountId, Balance, AssetId>> {
		&self.delegations
	}

	/// Returns a reference to the bond less request, if it exists.
	pub fn get_delegator_bond_less_request(&self) -> Option<&BondLessRequest<AssetId, Balance>> {
		self.delegator_bond_less_request.as_ref()
	}

	/// Checks if the list of delegations is empty.
	pub fn is_delegations_empty(&self) -> bool {
		self.delegations.is_empty()
	}

	/// Calculates the total delegation amount for a specific asset.
	pub fn calculate_delegation_by_asset(&self, asset_id: AssetId) -> Balance
	where
		Balance: Default + core::ops::AddAssign + Clone,
		AssetId: Eq + PartialEq,
	{
		let mut total = Balance::default();
		for bond in &self.delegations {
			if bond.asset_id == asset_id {
				total += bond.amount.clone();
			}
		}
		total
	}

	/// Returns a list of delegations to a specific operator.
	pub fn calculate_delegation_by_operator(
		&self,
		operator: AccountId,
	) -> Vec<&Bond<AccountId, Balance, AssetId>>
	where
		AccountId: Eq + PartialEq,
	{
		self.delegations.iter().filter(|&bond| bond.operator == operator).collect()
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
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
pub struct Bond<AccountId, Balance, AssetId> {
	/// The account ID of the operator.
	pub operator: AccountId,
	/// The amount bonded.
	pub amount: Balance,
	/// The ID of the bonded asset.
	pub asset_id: AssetId,
}

// ------ Test for helper functions ------ //

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::BTreeMap;
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
	fn get_unstake_request_should_work() {
		let unstake_request = UnstakeRequest {
			asset_id: MockAssetId(1),
			amount: MockBalance(50),
			requested_round: 1,
		};
		let metadata: DelegatorMetadata<MockAccountId, MockBalance, MockAssetId> =
			DelegatorMetadata {
				unstake_request: Some(unstake_request.clone()),
				..Default::default()
			};

		assert_eq!(metadata.get_unstake_request(), Some(&unstake_request));
	}

	#[test]
	fn get_delegations_should_work() {
		let delegations = vec![
			Bond { operator: MockAccountId(1), amount: MockBalance(50), asset_id: MockAssetId(1) },
			Bond { operator: MockAccountId(2), amount: MockBalance(75), asset_id: MockAssetId(2) },
		];
		let metadata: DelegatorMetadata<MockAccountId, MockBalance, MockAssetId> =
			DelegatorMetadata { delegations: delegations.clone(), ..Default::default() };

		assert_eq!(metadata.get_delegations(), &delegations);
	}

	#[test]
	fn get_delegator_bond_less_request_should_work() {
		let bond_less_request = BondLessRequest {
			asset_id: MockAssetId(1),
			amount: MockBalance(50),
			requested_round: 1,
		};
		let metadata: DelegatorMetadata<MockAccountId, MockBalance, MockAssetId> =
			DelegatorMetadata {
				delegator_bond_less_request: Some(bond_less_request.clone()),
				..Default::default()
			};

		assert_eq!(metadata.get_delegator_bond_less_request(), Some(&bond_less_request));
	}

	#[test]
	fn is_delegations_empty_should_work() {
		let metadata_with_delegations = DelegatorMetadata {
			delegations: vec![Bond {
				operator: MockAccountId(1),
				amount: MockBalance(50),
				asset_id: MockAssetId(1),
			}],
			..Default::default()
		};

		let metadata_without_delegations: DelegatorMetadata<
			MockAccountId,
			MockBalance,
			MockAssetId,
		> = Default::default();

		assert!(!metadata_with_delegations.is_delegations_empty());
		assert!(metadata_without_delegations.is_delegations_empty());
	}

	#[test]
	fn calculate_delegation_by_asset_should_work() {
		let delegations = vec![
			Bond { operator: MockAccountId(1), amount: MockBalance(50), asset_id: MockAssetId(1) },
			Bond { operator: MockAccountId(2), amount: MockBalance(75), asset_id: MockAssetId(1) },
			Bond { operator: MockAccountId(3), amount: MockBalance(25), asset_id: MockAssetId(2) },
		];
		let metadata = DelegatorMetadata { delegations, ..Default::default() };

		assert_eq!(metadata.calculate_delegation_by_asset(MockAssetId(1)), MockBalance(125));
		assert_eq!(metadata.calculate_delegation_by_asset(MockAssetId(2)), MockBalance(25));
		assert_eq!(metadata.calculate_delegation_by_asset(MockAssetId(3)), MockBalance(0));
	}

	#[test]
	fn calculate_delegation_by_operator_should_work() {
		let delegations = vec![
			Bond { operator: MockAccountId(1), amount: MockBalance(50), asset_id: MockAssetId(1) },
			Bond { operator: MockAccountId(1), amount: MockBalance(75), asset_id: MockAssetId(2) },
			Bond { operator: MockAccountId(2), amount: MockBalance(25), asset_id: MockAssetId(1) },
		];
		let metadata = DelegatorMetadata { delegations, ..Default::default() };

		let result = metadata.calculate_delegation_by_operator(MockAccountId(1));
		assert_eq!(result.len(), 2);
		assert_eq!(result[0].operator, MockAccountId(1));
		assert_eq!(result[1].operator, MockAccountId(1));
	}
}
