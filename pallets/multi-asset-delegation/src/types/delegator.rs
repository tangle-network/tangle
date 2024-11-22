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
use frame_support::{pallet_prelude::Get, BoundedVec};
use tangle_primitives::BlueprintId;

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
pub struct BondLessRequest<AccountId, AssetId, Balance, MaxBlueprints: Get<u32>> {
	/// The account ID of the operator.
	pub operator: AccountId,
	/// The ID of the asset to reduce the stake of.
	pub asset_id: AssetId,
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
> {
	/// A map of deposited assets and their respective amounts.
	pub deposits: BTreeMap<AssetId, Balance>,
	/// A vector of withdraw requests.
	pub withdraw_requests: BoundedVec<WithdrawRequest<AssetId, Balance>, MaxWithdrawRequests>,
	/// A list of all current delegations.
	pub delegations:
		BoundedVec<BondInfoDelegator<AccountId, Balance, AssetId, MaxBlueprints>, MaxDelegations>,
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
	> Default
	for DelegatorMetadata<
		AccountId,
		Balance,
		AssetId,
		MaxWithdrawRequests,
		MaxDelegations,
		MaxUnstakeRequests,
		MaxBlueprints,
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
	>
	DelegatorMetadata<
		AccountId,
		Balance,
		AssetId,
		MaxWithdrawRequests,
		MaxDelegations,
		MaxUnstakeRequests,
		MaxBlueprints,
	>
{
	/// Returns a reference to the vector of withdraw requests.
	pub fn get_withdraw_requests(&self) -> &Vec<WithdrawRequest<AssetId, Balance>> {
		&self.withdraw_requests
	}

	/// Returns a reference to the list of delegations.
	pub fn get_delegations(
		&self,
	) -> &Vec<BondInfoDelegator<AccountId, Balance, AssetId, MaxBlueprints>> {
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
	) -> Vec<&BondInfoDelegator<AccountId, Balance, AssetId, MaxBlueprints>>
	where
		AccountId: Eq + PartialEq,
	{
		self.delegations.iter().filter(|&stake| stake.operator == operator).collect()
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

/// Represents a stake between a delegator and an operator.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
pub struct BondInfoDelegator<AccountId, Balance, AssetId, MaxBlueprints: Get<u32>> {
	/// The account ID of the operator.
	pub operator: AccountId,
	/// The amount bonded.
	pub amount: Balance,
	/// The ID of the bonded asset.
	pub asset_id: AssetId,
	/// The blueprint selection mode for this delegator.
	pub blueprint_selection: DelegatorBlueprintSelection<MaxBlueprints>,
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
		pub const MaxWithdrawRequests: u32 = 10;
		pub const MaxDelegations: u32 = 10;
		pub const MaxUnstakeRequests: u32 = 10;
		#[derive(
			Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq, Clone, Copy, Ord, PartialOrd,
		)]
		pub const MaxBlueprints: u32 = 10;
	}

	type TestDelegatorMetadata = DelegatorMetadata<
		MockAccountId,
		MockBalance,
		MockAssetId,
		MaxWithdrawRequests,
		MaxDelegations,
		MaxUnstakeRequests,
		MaxBlueprints,
	>;

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
		let metadata = TestDelegatorMetadata {
			withdraw_requests: BoundedVec::try_from(withdraw_requests.clone()).unwrap(),
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
				blueprint_selection: Default::default(),
			},
			BondInfoDelegator {
				operator: MockAccountId(2),
				amount: MockBalance(75),
				asset_id: MockAssetId(2),
				blueprint_selection: Default::default(),
			},
		];

		let metadata = TestDelegatorMetadata {
			delegations: delegations.clone().try_into().unwrap(),
			..Default::default()
		};

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
				blueprint_selection: Default::default(),
			},
			BondLessRequest {
				asset_id: MockAssetId(2),
				amount: MockBalance(75),
				requested_round: 2,
				operator: MockAccountId(1),
				blueprint_selection: Default::default(),
			},
		];
		let metadata = TestDelegatorMetadata {
			delegator_unstake_requests: BoundedVec::try_from(unstake_requests.clone()).unwrap(),
			..Default::default()
		};

		assert_eq!(metadata.get_delegator_unstake_requests(), &unstake_requests);
	}

	#[test]
	fn is_delegations_empty_should_work() {
		let metadata_with_delegations = TestDelegatorMetadata {
			delegations: BoundedVec::try_from(vec![BondInfoDelegator {
				operator: MockAccountId(1),
				amount: MockBalance(50),
				asset_id: MockAssetId(1),
				blueprint_selection: Default::default(),
			}])
			.unwrap(),
			..Default::default()
		};

		let metadata_without_delegations = TestDelegatorMetadata::default();

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
				blueprint_selection: Default::default(),
			},
			BondInfoDelegator {
				operator: MockAccountId(2),
				amount: MockBalance(75),
				asset_id: MockAssetId(1),
				blueprint_selection: Default::default(),
			},
			BondInfoDelegator {
				operator: MockAccountId(3),
				amount: MockBalance(25),
				asset_id: MockAssetId(2),
				blueprint_selection: Default::default(),
			},
		];
		let metadata = TestDelegatorMetadata {
			delegations: BoundedVec::try_from(delegations).unwrap(),
			..Default::default()
		};

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
				blueprint_selection: Default::default(),
			},
			BondInfoDelegator {
				operator: MockAccountId(1),
				amount: MockBalance(75),
				asset_id: MockAssetId(2),
				blueprint_selection: Default::default(),
			},
			BondInfoDelegator {
				operator: MockAccountId(2),
				amount: MockBalance(25),
				asset_id: MockAssetId(1),
				blueprint_selection: Default::default(),
			},
		];
		let metadata = TestDelegatorMetadata {
			delegations: BoundedVec::try_from(delegations).unwrap(),
			..Default::default()
		};

		let result = metadata.calculate_delegation_by_operator(MockAccountId(1));
		assert_eq!(result.len(), 2);
		assert_eq!(result[0].operator, MockAccountId(1));
		assert_eq!(result[1].operator, MockAccountId(1));
	}
}
