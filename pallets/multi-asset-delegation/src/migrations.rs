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

use crate::{
	Config, Delegators,
	types::{DelegatorMetadata, delegator::*},
};
use frame_support::{
	pallet_prelude::*,
	storage::unhashed,
	traits::{Currency, OnRuntimeUpgrade},
	weights::Weight,
};
use frame_system::{self, pallet_prelude::BlockNumberFor};
use sp_std::{collections::btree_map::BTreeMap, marker::PhantomData};
use tangle_primitives::{RoundIndex, services::Asset};

/// Migration to update DelegatorMetadata structure with new field names and add is_nomination field
pub struct DelegatorMetadataMigration<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for DelegatorMetadataMigration<T> {
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
			pub withdraw_requests:
				BoundedVec<OldWithdrawRequest<AssetId, Balance>, MaxWithdrawRequests>,
			/// A list of all current delegations.
			pub delegations: BoundedVec<
				OldBondInfoDelegator<AccountId, Balance, AssetId, MaxBlueprints>,
				MaxDelegations,
			>,
			/// A vector of requests to reduce the bonded amount.
			pub delegator_unstake_requests: BoundedVec<
				OldBondLessRequest<AccountId, AssetId, Balance, MaxBlueprints>,
				MaxUnstakeRequests,
			>,
			/// The current status of the delegator.
			pub status: DelegatorStatus,
		}

		/// Represents a request to withdraw a specific amount of an asset.
		#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
		pub struct OldWithdrawRequest<AssetId: Encode + Decode, Balance> {
			/// The ID of the asset to be withdrawn.
			pub asset_id: Asset<AssetId>,
			/// The amount of the asset to be withdrawn.
			pub amount: Balance,
			/// The round in which the withdraw was requested.
			pub requested_round: RoundIndex,
		}

		/// Represents a request to reduce the bonded amount of a specific asset.
		#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
		pub struct OldBondLessRequest<
			AccountId,
			AssetId: Encode + Decode,
			Balance,
			MaxBlueprints: Get<u32>,
		> {
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
		pub struct OldBondInfoDelegator<
			AccountId,
			Balance,
			AssetId: Encode + Decode,
			MaxBlueprints: Get<u32>,
		> {
			/// The operator being delegated to.
			pub operator: AccountId,
			/// The amount being delegated.
			pub amount: Balance,
			/// The asset being delegated.
			pub asset_id: Asset<AssetId>,
			/// The blueprint selection for this delegation.
			pub blueprint_selection: DelegatorBlueprintSelection<MaxBlueprints>,
		}

		log::info!("Starting DelegatorMetadataMigration...");

		// Iterate through all entries in the storage
		let iter = Delegators::<T>::iter_keys();
		let mut migrated_count = 0;
		let mut failed_count = 0;

		for account_id in iter {
			// Read operation
			weight = weight.saturating_add(T::DbWeight::get().reads(1_u64));

			// Get the raw bytes of the metadata
			let raw_key = Delegators::<T>::hashed_key_for(&account_id);
			let raw_value = match unhashed::get_raw(&raw_key) {
				Some(bytes) => bytes,
				None => {
					log::warn!("No raw metadata found for account: {:?}", account_id);
					continue;
				},
			};

			// Try to decode using our old metadata format
			let old_metadata = match OldDelegatorMetadata::<
				<T as frame_system::Config>::AccountId,
				<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
				T::AssetId,
				T::MaxWithdrawRequests,
				T::MaxDelegations,
				T::MaxUnstakeRequests,
				T::MaxDelegatorBlueprints,
				BlockNumberFor<T>,
				<T as Config>::MaxDelegations,
			>::decode(&mut &raw_value[..]) {
				Ok(metadata) => metadata,
				Err(e) => {
					log::error!("Failed to decode delegator metadata for account {:?}: {:?}", account_id, e);
					failed_count += 1;
					continue;
				}
			};

			// Create new bounded vectors for the new metadata format
			let mut new_withdraw_requests =
				BoundedVec::<
					WithdrawRequest<
						T::AssetId,
						<<T as Config>::Currency as Currency<
							<T as frame_system::Config>::AccountId,
						>>::Balance,
					>,
					T::MaxWithdrawRequests,
				>::default();

			let mut new_delegations =
				BoundedVec::<
					BondInfoDelegator<
						<T as frame_system::Config>::AccountId,
						<<T as Config>::Currency as Currency<
							<T as frame_system::Config>::AccountId,
						>>::Balance,
						T::AssetId,
						T::MaxDelegatorBlueprints,
					>,
					T::MaxDelegations,
				>::default();

			let mut new_unstake_requests =
				BoundedVec::<
					BondLessRequest<
						<T as frame_system::Config>::AccountId,
						T::AssetId,
						<<T as Config>::Currency as Currency<
							<T as frame_system::Config>::AccountId,
						>>::Balance,
						T::MaxDelegatorBlueprints,
					>,
					T::MaxUnstakeRequests,
				>::default();

			// Migrate withdraw requests (rename asset_id -> asset)
			for old_req in old_metadata.withdraw_requests.into_iter() {
				if new_withdraw_requests
					.try_push(WithdrawRequest {
						asset: old_req.asset_id, // Rename field
						amount: old_req.amount,
						requested_round: old_req.requested_round,
					})
					.is_err()
				{
					log::warn!(
						"Failed to migrate a withdraw request for account {:?}: exceeded capacity",
						account_id
					);
					break;
				}
			}

			// Migrate delegations (rename asset_id -> asset, add is_nomination = false)
			for old_delegation in old_metadata.delegations.into_iter() {
				if new_delegations
					.try_push(BondInfoDelegator {
						operator: old_delegation.operator,
						amount: old_delegation.amount,
						asset: old_delegation.asset_id, // Rename field
						blueprint_selection: old_delegation.blueprint_selection,
						is_nomination: false, // New field with default value
					})
					.is_err()
				{
					log::warn!(
						"Failed to migrate a delegation for account {:?}: exceeded capacity",
						account_id
					);
					break;
				}
			}

			// Migrate unstake requests (rename asset_id -> asset, add is_nomination = false)
			for old_req in old_metadata.delegator_unstake_requests.into_iter() {
				if new_unstake_requests
					.try_push(BondLessRequest {
						operator: old_req.operator,
						asset: old_req.asset_id, // Rename field
						amount: old_req.amount,
						requested_round: old_req.requested_round,
						blueprint_selection: old_req.blueprint_selection,
						is_nomination: false, // New field with default value
					})
					.is_err()
				{
					log::warn!(
						"Failed to migrate an unstake request for account {:?}: exceeded capacity",
						account_id
					);
					break;
				}
			}

			// Create the new metadata with converted components
			let new_metadata = DelegatorMetadata {
				deposits: old_metadata.deposits,
				withdraw_requests: new_withdraw_requests,
				delegations: new_delegations,
				delegator_unstake_requests: new_unstake_requests,
				status: old_metadata.status,
			};

			// Update the storage with the new metadata
			Delegators::<T>::insert(account_id, new_metadata);

			// Write operation
			weight = weight.saturating_add(T::DbWeight::get().writes(1_u64));

			migrated_count += 1;
		}

		log::info!(
			"DelegatorMetadataMigration completed: Migrated {} delegator metadata entries, Failed: {}",
			migrated_count,
			failed_count
		);

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		// Count how many entries we have pre-migration
		let count = Delegators::<T>::iter().count() as u32;
		log::info!("DelegatorMetadataMigration pre_upgrade: Found {} delegator entries", count);

		// Store the count and a sample of the first few entries for verification
		let mut state = Vec::new();
		state.extend_from_slice(&count.encode());

		// Sample up to 5 entries to verify later
		let mut sample_count = 0;
		for (account_id, _) in Delegators::<T>::iter().take(5) {
			state.extend_from_slice(&account_id.encode());
			sample_count += 1;
		}

		state.extend_from_slice(&sample_count.encode());

		Ok(state)
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		// Decode the state from pre_upgrade
		let mut state_cursor = &state[..];

		let pre_count =
			u32::decode(&mut state_cursor).map_err(|_| "Failed to decode pre-migration count")?;

		// Get the current count
		let post_count = Delegators::<T>::iter().count() as u32;

		// Ensure we have at least the same number of entries
		if post_count < pre_count {
			log::error!(
				"DelegatorMetadataMigration post_upgrade: Entry count mismatch. Pre: {}, Post: {}",
				pre_count,
				post_count
			);
			return Err("Entry count decreased after migration");
		}

		// Verify the sampled accounts still exist
		let sample_count =
			u32::decode(&mut state_cursor).map_err(|_| "Failed to decode sample count")?;

		for _ in 0..sample_count {
			let account_id = <T as frame_system::Config>::AccountId::decode(&mut state_cursor)
				.map_err(|_| "Failed to decode account ID")?;

			if !Delegators::<T>::contains_key(&account_id) {
				log::error!(
					"DelegatorMetadataMigration post_upgrade: Account {:?} missing after migration",
					account_id
				);
				return Err("Account missing after migration");
			}

			// Verify the new structure has the expected fields
			let metadata =
				Delegators::<T>::get(&account_id).ok_or("Failed to get metadata for account")?;

			// Check that delegations have is_nomination field
			for delegation in metadata.delegations.iter() {
				// Just accessing this field verifies it exists
				let _ = delegation.is_nomination;
			}

			// Check that unstake requests have is_nomination field
			for request in metadata.delegator_unstake_requests.iter() {
				// Just accessing this field verifies it exists
				let _ = request.is_nomination;
			}
		}

		log::info!(
			"DelegatorMetadataMigration post_upgrade: Successfully verified migration. Pre: {}, Post: {}",
			pre_count,
			post_count
		);

		Ok(())
	}
}
