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

/// Represents a request to withdraw a specific amount of an asset.
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct WithdrawRequest<AssetId, Balance> {
	/// The ID of the asset to be withdrawd.
	pub asset_id: AssetId,
	/// The amount of the asset to be withdrawd.
	pub amount: Balance,
	/// The round in which the withdraw was requested.
	pub requested_round: RoundIndex,
}

/// Represents a request to reduce the bonded amount of a specific asset.
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct BondLessRequest<AccountId, AssetId, Balance> {
	/// The account ID of the operator.
	pub operator: AccountId,
	/// The ID of the asset to reduce the stake of.
	pub asset_id: AssetId,
	/// The amount by which to reduce the stake.
	pub amount: Balance,
	/// The round in which the stake reduction was requested.
	pub requested_round: RoundIndex,
}

/// Stores the state of a delegator, including deposits, delegations, and requests.
#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct DelegatorMetadata<AccountId, Balance, AssetId: Encode + Decode + TypeInfo> {
	/// A map of deposited assets and their respective amounts.
	pub deposits: BTreeMap<AssetId, Balance>,
	/// A vector of withdraw requests.
	pub withdraw_requests: Vec<WithdrawRequest<AssetId, Balance>>,
	/// A list of all current delegations.
	pub delegations: Vec<BondInfoDelegator<AccountId, Balance, AssetId>>,
	/// A vector of requests to reduce the bonded amount.
	pub delegator_unstake_requests: Vec<BondLessRequest<AccountId, AssetId, Balance>>,
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
			withdraw_requests: Vec::new(),
			delegator_unstake_requests: Vec::new(),
		}
	}
}

impl<AccountId, Balance, AssetId: Encode + Decode + TypeInfo>
	DelegatorMetadata<AccountId, Balance, AssetId>
{
	/// Returns a reference to the vector of withdraw requests.
	pub fn get_withdraw_requests(&self) -> &Vec<WithdrawRequest<AssetId, Balance>> {
		&self.withdraw_requests
	}

	/// Returns a reference to the list of delegations.
	pub fn get_delegations(&self) -> &Vec<BondInfoDelegator<AccountId, Balance, AssetId>> {
		&self.delegations
	}

	/// Returns a reference to the vector of unstake requests.
	pub fn get_delegator_unstake_requests(
		&self,
	) -> &Vec<BondLessRequest<AccountId, AssetId, Balance>> {
		&self.delegator_unstake_requests
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
	) -> Vec<&BondInfoDelegator<AccountId, Balance, AssetId>>
	where
		AccountId: Eq + PartialEq,
	{
		self.delegations.iter().filter(|&stake| stake.operator == operator).collect()
	}

	/// Calculates the total delegation amount for a specific asset and service
	pub fn calculate_delegation_by_asset_and_service(&self, asset_id: AssetId, service: ServiceId) -> Balance
	where
		Balance: Default + core::ops::AddAssign + Clone,
		AssetId: Eq + PartialEq,
	{
		let mut total = Balance::default();
		for stake in &self.delegations {
			if stake.asset_id == asset_id && stake.services.contains(&service) {
				total += stake.amount.clone();
			}
		}
		total
	}

	/// Returns a list of delegations to a specific operator for a specific service
	pub fn calculate_delegation_by_operator_and_service(
		&self,
		operator: AccountId,
		service: ServiceId,
	) -> Vec<&BondInfoDelegator<AccountId, Balance, AssetId>>
	where
		AccountId: Eq + PartialEq,
	{
		self.delegations
			.iter()
			.filter(|&stake| stake.operator == operator && stake.services.contains(&service))
			.collect()
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

/// Represents a stake between a delegator and an operator, including service participation
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
pub struct BondInfoDelegator<AccountId, Balance, AssetId> {
	/// The account ID of the operator
	pub operator: AccountId,
	/// The amount bonded
	pub amount: Balance,
	/// The ID of the bonded asset
	pub asset_id: AssetId,
	/// The set of service IDs this delegation participates in
	pub services: BTreeSet<ServiceId>,
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
	fn get_withdraw_requests_should_work() {
		let withdraw_requests = vec![
			WithdrawRequest {
				asset_id: MockAssetId(1),
				amount: MockBalance(50),
				requested_round: 1,
			},
			WithdrawRequest {
				asset_id: MockAssetId(2),
				amount: MockBalance(75),
				requested_round: 2,
			},
		];
		let metadata: DelegatorMetadata<MockAccountId, MockBalance, MockAssetId> =
			DelegatorMetadata {
				withdraw_requests: withdraw_requests.clone(),
				..Default::default()
			};

		assert_eq!(metadata.get_withdraw_requests(), &withdraw_requests);
	}

	#[test]
	fn get_delegations_should_work() {
		let delegations = vec![
			BondInfoDelegator {
				operator: MockAccountId(1),
				amount: MockBalance(50),
				asset_id: MockAssetId(1),
				services: BTreeSet::new(),
			},
			BondInfoDelegator {
				operator: MockAccountId(2),
				amount: MockBalance(75),
				asset_id: MockAssetId(2),
				services: BTreeSet::new(),
			},
		];
		let metadata: DelegatorMetadata<MockAccountId, MockBalance, MockAssetId> =
			DelegatorMetadata { delegations: delegations.clone(), ..Default::default() };

		assert_eq!(metadata.get_delegations(), &delegations);
	}

	#[test]
	fn get_delegator_unstake_requests_should_work() {
		let unstake_requests = vec![
			BondLessRequest {
				asset_id: MockAssetId(1),
				amount: MockBalance(50),
				requested_round: 1,
				operator: MockAccountId(1),
			},
			BondLessRequest {
				asset_id: MockAssetId(2),
				amount: MockBalance(75),
				requested_round: 2,
				operator: MockAccountId(1),
			},
		];
		let metadata: DelegatorMetadata<MockAccountId, MockBalance, MockAssetId> =
			DelegatorMetadata {
				delegator_unstake_requests: unstake_requests.clone(),
				..Default::default()
			};

		assert_eq!(metadata.get_delegator_unstake_requests(), &unstake_requests);
	}

	#[test]
	fn is_delegations_empty_should_work() {
		let metadata_with_delegations = DelegatorMetadata {
			delegations: vec![BondInfoDelegator {
				operator: MockAccountId(1),
				amount: MockBalance(50),
				asset_id: MockAssetId(1),
				services: BTreeSet::new(),
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
			BondInfoDelegator {
				operator: MockAccountId(1),
				amount: MockBalance(50),
				asset_id: MockAssetId(1),
				services: BTreeSet::new(),
			},
			BondInfoDelegator {
				operator: MockAccountId(2),
				amount: MockBalance(75),
				asset_id: MockAssetId(1),
				services: BTreeSet::new(),
			},
			BondInfoDelegator {
				operator: MockAccountId(3),
				amount: MockBalance(25),
				asset_id: MockAssetId(2),
				services: BTreeSet::new(),
			},
		];
		let metadata = DelegatorMetadata { delegations, ..Default::default() };

		assert_eq!(metadata.calculate_delegation_by_asset(MockAssetId(1)), MockBalance(125));
		assert_eq!(metadata.calculate_delegation_by_asset(MockAssetId(2)), MockBalance(25));
		assert_eq!(metadata.calculate_delegation_by_asset(MockAssetId(3)), MockBalance(0));
	}

	#[test]
	fn calculate_delegation_by_operator_should_work() {
		let delegations = vec![
			BondInfoDelegator {
				operator: MockAccountId(1),
				amount: MockBalance(50),
				asset_id: MockAssetId(1),
				services: BTreeSet::new(),
			},
			BondInfoDelegator {
				operator: MockAccountId(1),
				amount: MockBalance(75),
				asset_id: MockAssetId(2),
			},
			BondInfoDelegator {
				operator: MockAccountId(2),
				amount: MockBalance(25),
				asset_id: MockAssetId(1),
			},
		];
		let metadata = DelegatorMetadata { delegations, ..Default::default() };

		let result = metadata.calculate_delegation_by_operator(MockAccountId(1));
		assert_eq!(result.len(), 2);
		assert_eq!(result[0].operator, MockAccountId(1));
		assert_eq!(result[1].operator, MockAccountId(1));
	}

	#[test]
	fn service_specific_delegation_should_work() {
		let service_a = ServiceId(1);
		let service_b = ServiceId(2);
		let service_c = ServiceId(3);
		let service_d = ServiceId(4);

		// Create service sets
		let all_services: BTreeSet<ServiceId> = vec![service_a, service_b, service_c, service_d]
			.into_iter()
			.collect();
		let limited_services: BTreeSet<ServiceId> = vec![service_a, service_b, service_c]
			.into_iter()
			.collect();

		let delegations = vec![
			BondInfoDelegator {
				operator: MockAccountId(1), // Bob
				amount: MockBalance(100),
				asset_id: MockAssetId(1),
				services: all_services.clone(),
			},
			BondInfoDelegator {
				operator: MockAccountId(1), // Bob
				amount: MockBalance(50),
				asset_id: MockAssetId(2),
				services: limited_services.clone(),
			},
		];

		let metadata = DelegatorMetadata {
			delegations,
			..Default::default()
		};

		// Test delegation calculations for different services
		assert_eq!(
			metadata.calculate_delegation_by_asset_and_service(MockAssetId(1), service_d),
			MockBalance(100)
		);
		assert_eq!(
			metadata.calculate_delegation_by_asset_and_service(MockAssetId(2), service_d),
			MockBalance(0)
		);
		assert_eq!(
			metadata.calculate_delegation_by_asset_and_service(MockAssetId(1), service_a),
			MockBalance(100)
		);
		assert_eq!(
			metadata.calculate_delegation_by_asset_and_service(MockAssetId(2), service_a),
			MockBalance(50)
		);

		// Test operator-specific service delegations
		let service_d_delegations = metadata.calculate_delegation_by_operator_and_service(MockAccountId(1), service_d);
		assert_eq!(service_d_delegations.len(), 1);
		assert!(service_d_delegations[0].services.contains(&service_d));

		let service_a_delegations = metadata.calculate_delegation_by_operator_and_service(MockAccountId(1), service_a);
		assert_eq!(service_a_delegations.len(), 2);
		assert!(service_a_delegations.iter().all(|d| d.services.contains(&service_a)));
	}
}
