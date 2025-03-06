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

use crate::{Config, DelegatorMetadata, Delegators};
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight};
use sp_runtime::{Perbill, Percent};
use sp_std::marker::PhantomData;

/// Migration to convert APY from percentage to Perbill in `RewardConfigForAssetVault`
pub struct PercentageToPerbillMigration<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for PercentageToPerbillMigration<T> {
	fn on_runtime_upgrade() -> Weight {
		let mut weight = Weight::from_parts(0, 0);

		/// Stores the state of a delegator, including deposits, delegations, and requests.
		#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
		pub struct OldDelegatorMetadata<
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
			pub withdraw_requests: BoundedVec<OldWithdrawRequest<AssetId, Balance>, MaxWithdrawRequests>,
			/// A list of all current delegations.
			pub delegations:
				BoundedVec<OldBondInfoDelegator<AccountId, Balance, AssetId, MaxBlueprints>, MaxDelegations>,
			/// A vector of requests to reduce the bonded amount.
			pub delegator_unstake_requests:
				BoundedVec<OldBondLessRequest<AccountId, AssetId, Balance, MaxBlueprints>, MaxUnstakeRequests>,
			/// The current status of the delegator.
			pub status: DelegatorStatus,
		}

		/// Represents a request to withdraw a specific amount of an asset.
		#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
		pub struct OldWithdrawRequest<AssetId: Encode + Decode, Balance> {
			/// The ID of the asset to be withdrawd.
			pub asset_id: Asset<AssetId>,
			/// The amount of the asset to be withdrawn.
			pub amount: Balance,
			/// The round in which the withdraw was requested.
			pub requested_round: RoundIndex,
		}

		/// Represents a request to reduce the bonded amount of a specific asset.
		#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
		pub struct OldBondLessRequest<AccountId, AssetId: Encode + Decode, Balance, MaxBlueprints: Get<u32>> {
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

		/// Represents a delegation bond from a delegator to an operator.
		#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
		pub struct OldBondInfoDelegator<AccountId, Balance, AssetId: Encode + Decode, MaxBlueprints: Get<u32>>
		{
			/// The operator being delegated to.
			pub operator: AccountId,
			/// The amount being delegated.
			pub amount: Balance,
			/// The asset being delegated.
			pub asset_id: Asset<AssetId>,
			/// The blueprint selection for this delegation.
			pub blueprint_selection: DelegatorBlueprintSelection<MaxBlueprints>,
		}

		// Iterate through all entries in the RewardConfigStorage
		let iter = Delegators::<T>::iter();

		let mut migrated_count = 0;

		for (account, metadata) in iter {
			// Read operation
			weight = weight.saturating_add(T::DbWeight::get().reads(1_u64));

			// Convert old metadata to new metadata

			// Write operation
			weight = weight.saturating_add(T::DbWeight::get().writes(1_u64));

			migrated_count += 1;
		}

		log::info!(
			"PercentageToPerbillMigration: Migrated {} reward configurations",
			migrated_count
		);

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		// Count how many entries we have pre-migration
		let count = RewardConfigStorage::<T>::iter().count() as u32;
		Ok(count.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		Ok(())
	}
}
