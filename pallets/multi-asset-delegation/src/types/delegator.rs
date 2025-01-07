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
use frame_support::ensure;
use frame_support::{pallet_prelude::Get, BoundedVec};
use sp_std::fmt::Debug;
use sp_std::vec;
use tangle_primitives::types::rewards::LockInfo;
use tangle_primitives::types::rewards::LockMultiplier;
use tangle_primitives::{services::Asset, BlueprintId};

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
	pub deposits: BTreeMap<Asset<AssetId>, Deposit<Balance, BlockNumber, MaxLocks>>,
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
	) -> Vec<&BondInfoDelegator<AccountId, Balance, AssetId, MaxBlueprints>>
	where
		AccountId: Eq + PartialEq,
	{
		self.delegations.iter().filter(|&stake| stake.operator == operator).collect()
	}
}

/// Represents a deposit of a specific asset.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
pub struct Deposit<Balance, BlockNumber, MaxLocks: Get<u32>> {
	/// The total amount deposited by the user (includes both delegated and non-delegated).
	pub amount: Balance,
	/// The total delegated amount by the user (this can never be greater than `amount`).
	pub delegated_amount: Balance,
	/// The locks associated with this deposit.
	pub locks: Option<BoundedVec<LockInfo<Balance, BlockNumber>, MaxLocks>>,
}

impl<
		Balance: Debug + Default + Clone + sp_runtime::Saturating + sp_std::cmp::PartialOrd + From<u32>,
		BlockNumber: Debug + sp_runtime::Saturating + sp_std::convert::From<u32> + sp_std::cmp::PartialOrd,
		MaxLocks: Get<u32>,
	> Deposit<Balance, BlockNumber, MaxLocks>
{
	pub fn new(
		amount: Balance,
		lock_multiplier: Option<LockMultiplier>,
		current_block_number: BlockNumber,
	) -> Self {
		let locks = lock_multiplier.map(|multiplier| {
			let expiry_block = current_block_number.saturating_add(multiplier.get_blocks().into());
			BoundedVec::try_from(vec![LockInfo {
				amount: amount.clone(),
				expiry_block,
				lock_multiplier: multiplier,
			}])
			.expect("This should not happen since only one lock exists!")
		});

		Deposit { amount, delegated_amount: Balance::default(), locks }
	}

	pub fn get_total_amount(&self) -> Balance {
		self.amount.clone()
	}

	pub fn increase_delegated_amount(
		&mut self,
		amount_to_increase: Balance,
	) -> Result<(), &'static str> {
		// sanity check that the proposed amount when added to the current delegated amount is not greater than the total amount
		let new_delegated_amount =
			self.delegated_amount.clone().saturating_add(amount_to_increase.clone());
		ensure!(
			new_delegated_amount <= self.amount,
			"delegated amount cannot be greater than total amount"
		);
		self.delegated_amount = new_delegated_amount;
		Ok(())
	}

	pub fn decrease_delegated_amount(
		&mut self,
		amount_to_decrease: Balance,
	) -> Result<(), &'static str> {
		self.delegated_amount = self.delegated_amount.clone().saturating_sub(amount_to_decrease);
		Ok(())
	}

	pub fn increase_deposited_amount(
		&mut self,
		amount_to_increase: Balance,
		lock_multiplier: Option<LockMultiplier>,
		current_block_number: BlockNumber,
	) -> Result<(), &'static str> {
		// Update the total amount first
		self.amount = self.amount.clone().saturating_add(amount_to_increase.clone());

		// If there's a lock multiplier, add a new lock
		if let Some(multiplier) = lock_multiplier {
			let lock_blocks = multiplier.get_blocks();
			let expiry_block = current_block_number.saturating_add(lock_blocks.into());

			let new_lock =
				LockInfo { amount: amount_to_increase, expiry_block, lock_multiplier: multiplier };

			// Initialize locks if None or push to existing locks
			if let Some(locks) = &mut self.locks {
				locks
					.try_push(new_lock)
					.map_err(|_| "Failed to push new lock - exceeded MaxLocks bound")?;
			} else {
				self.locks = Some(
					BoundedVec::try_from(vec![new_lock])
						.expect("This should not happen since only one lock exists!"),
				);
			}
		}

		Ok(())
	}

	pub fn decrease_deposited_amount(
		&mut self,
		amount_to_decrease: Balance,
		current_block_number: BlockNumber,
	) -> Result<(), &'static str> {
		let total_locked = self.locks.as_ref().map_or(Balance::from(0_u32), |locks| {
			locks
				.iter()
				.filter(|lock| lock.expiry_block > current_block_number)
				.fold(Balance::from(0_u32), |acc, lock| acc.saturating_add(lock.amount.clone()))
		});

		let free_amount = self.amount.clone().saturating_sub(total_locked);
		ensure!(
			free_amount >= amount_to_decrease,
			"total free amount cannot be lesser than amount to decrease"
		);

		self.amount = self.amount.clone().saturating_sub(amount_to_decrease);
		ensure!(
			self.amount >= self.delegated_amount,
			"delegated amount cannot be greater than total amount"
		);

		Ok(())
	}
}

/// Represents a stake between a delegator and an operator.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
pub struct BondInfoDelegator<AccountId, Balance, AssetId: Encode + Decode, MaxBlueprints: Get<u32>>
{
	/// The account ID of the operator.
	pub operator: AccountId,
	/// The amount bonded.
	pub amount: Balance,
	/// The ID of the bonded asset.
	pub asset_id: Asset<AssetId>,
	/// The blueprint selection mode for this delegator.
	pub blueprint_selection: DelegatorBlueprintSelection<MaxBlueprints>,
}
