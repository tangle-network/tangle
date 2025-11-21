// This file is part of Tangle.
// Copyright (C) 2022-2025 Tangle Foundation.
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
	Config, ServiceRequests,
	types::{
		ConstraintsFor, MaxAssetsPerServiceOf, MaxFieldsOf, MaxOperatorsPerServiceOf,
		MaxPermittedCallersOf,
	},
};
use frame_support::{pallet_prelude::*, traits::UncheckedOnRuntimeUpgrade, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_std::{marker::PhantomData, vec::Vec};
use tangle_primitives::{
	BlueprintId,
	services::{
		ApprovalState, AssetIdT, AssetSecurityCommitment, AssetSecurityRequirement, Field,
		MembershipModel, OperatorsWithApprovalState, ServiceRequest,
	},
};

/// Collection of storage item formats from the previous storage version.
///
/// Required so we can read values in the v0 storage format during the migration.
mod v0 {
	use super::*;
	use frame_support::storage_alias;

	/// Historical version of `ApprovalState` where the `Approved` variant stored an
	/// unbounded `Vec` of security commitments.
	#[derive(Encode, Decode)]
	pub enum OldApprovalState<AssetId: AssetIdT> {
		/// The operator has not yet responded to the request.
		Pending,
		/// The operator has approved the request with unrestricted commitments.
		Approved { security_commitments: Vec<AssetSecurityCommitment<AssetId>> },
		/// The operator rejected the request.
		Rejected,
	}

	/// Historical version of `ServiceRequest` which still referenced the
	/// `OldApprovalState`.
	///
	/// The only difference from the current format is that `OldApprovalState::Approved`
	/// contains an unbounded `Vec` instead of a `BoundedVec` for security commitments.
	#[derive(Encode, Decode)]
	pub struct OldServiceRequest<T: Config> {
		pub blueprint: BlueprintId,
		pub owner: T::AccountId,
		pub security_requirements:
			BoundedVec<AssetSecurityRequirement<T::AssetId>, MaxAssetsPerServiceOf<T>>,
		pub ttl: BlockNumberFor<T>,
		pub args: BoundedVec<Field<ConstraintsFor<T>, T::AccountId>, MaxFieldsOf<T>>,
		pub permitted_callers: BoundedVec<T::AccountId, MaxPermittedCallersOf<T>>,
		pub operators_with_approval_state:
			BoundedVec<(T::AccountId, OldApprovalState<T::AssetId>), MaxOperatorsPerServiceOf<T>>,
		pub membership_model: MembershipModel,
	}

	/// V0 type for [`crate::ServiceRequests`].
	#[storage_alias]
	pub type ServiceRequests<T: Config> =
		StorageMap<crate::Pallet<T>, Identity, u64, OldServiceRequest<T>>;
}

/// Implements [`UncheckedOnRuntimeUpgrade`], migrating the state of this pallet from V0 to V1.
///
/// In V0, `ApprovalState::Approved` stored an unbounded `Vec` of security commitments.
/// In V1, it has been upgraded to use `BoundedVec` with `MaxAssetsPerService` limit.
///
/// This migration converts all existing `ServiceRequest` entries to the new format.
pub struct ApprovalStateOfServiceRequestsMigration<T: Config>(PhantomData<T>);

impl<T: Config> UncheckedOnRuntimeUpgrade for ApprovalStateOfServiceRequestsMigration<T> {
	/// Return the count of service requests so we can verify it in `post_upgrade`.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
		// Count how many entries we have pre-migration
		// @dev: Using u32, Max ~4.2 billion entries
		let count = v0::ServiceRequests::<T>::iter().count() as u32;
		Ok(count.encode())
	}

	/// Migrate the storage from V0 to V1.
	///
	/// Converts all `OldApprovalState::Approved` variants from `Vec` to `BoundedVec`.
	fn on_runtime_upgrade() -> Weight {
		let mut migrated = 0u64;
		let mut weight = Weight::from_parts(0, 0);

		for (request_id, old_request) in v0::ServiceRequests::<T>::drain() {
			// Read operation
			weight = weight.saturating_add(T::DbWeight::get().reads(1));

			// Convert operators with approval states
			let converted_operators_vec: Vec<_> = old_request
				.operators_with_approval_state
				.into_iter()
				.map(|(operator, state)| {
					let converted_state = match state {
						v0::OldApprovalState::Pending => ApprovalState::Pending,
						v0::OldApprovalState::Rejected => ApprovalState::Rejected,
						v0::OldApprovalState::Approved { security_commitments } => {
							let commitments = BoundedVec::<
								AssetSecurityCommitment<T::AssetId>,
								MaxAssetsPerServiceOf<T>,
							>::truncate_from(security_commitments);
							ApprovalState::Approved { security_commitments: commitments }
						},
					};

					(operator, converted_state)
				})
				.collect();

			let converted_operators = OperatorsWithApprovalState::<
				T::AccountId,
				T::AssetId,
				T::Constraints,
			>::truncate_from(converted_operators_vec);

			// Create new request with converted approval states
			let new_request = ServiceRequest {
				blueprint: old_request.blueprint,
				owner: old_request.owner,
				security_requirements: old_request.security_requirements,
				ttl: old_request.ttl,
				args: old_request.args,
				permitted_callers: old_request.permitted_callers,
				operators_with_approval_state: converted_operators,
				membership_model: old_request.membership_model,
			};

			// Write the migrated request
			ServiceRequests::<T>::insert(request_id, new_request);
			weight = weight.saturating_add(T::DbWeight::get().writes(1));

			migrated = migrated.saturating_add(1);
		}

		log::info!(
			"ApprovalStateOfServiceRequestsMigration: Migrated {} service requests",
			migrated
		);

		weight
	}

	/// Verifies the storage was migrated correctly.
	///
	/// - Ensures the same number of entries exist post-migration.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		let pre_count = u32::decode(&mut &state[..]).map_err(|_| {
			sp_runtime::TryRuntimeError::Other("Failed to decode pre-migration count")
		})?;
		let post_count = ServiceRequests::<T>::iter().count() as u32;

		if pre_count != post_count {
			return Err(sp_runtime::TryRuntimeError::Other(
				"Number of service requests changed during migration",
			));
		}

		log::info!(
			"ApprovalStateOfServiceRequestsMigration: Successfully migrated {} service requests",
			post_count
		);

		Ok(())
	}
}

/// [`UncheckedOnRuntimeUpgrade`] implementation [`ApprovalStateOfServiceRequestsMigration`] wrapped
/// in a [`VersionedMigration`](frame_support::migrations::VersionedMigration), which ensures that:
/// - The migration only runs once when the on-chain storage version is 0
/// - The on-chain storage version is updated to `1` after the migration executes
/// - Reads/Writes from checking/settings the on-chain storage version are accounted for
pub type MigrateV0ToV1<T> = frame_support::migrations::VersionedMigration<
	0, // The migration will only execute when the on-chain storage version is 0
	1, // The on-chain storage version will be set to 1 after the migration is complete
	ApprovalStateOfServiceRequestsMigration<T>,
	crate::Pallet<T>,
	<T as frame_system::Config>::DbWeight,
>;
