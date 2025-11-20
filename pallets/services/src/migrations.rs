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
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_std::{marker::PhantomData, vec::Vec};
use tangle_primitives::{
	BlueprintId,
	services::{
		ApprovalState, AssetIdT, AssetSecurityCommitment, AssetSecurityRequirement, Field,
		MembershipModel, OperatorsWithApprovalState, ServiceRequest,
	},
};

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

/// Migration to convert ApprovalState::Approval.security_commitments from Vec to BoundedVec
/// primitives/src/services/types.rs:209
pub struct ApprovalStateOfServiceRequestsMigration<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for ApprovalStateOfServiceRequestsMigration<T> {
	fn on_runtime_upgrade() -> Weight {
		let mut migrated = 0u64;

		ServiceRequests::<T>::translate::<OldServiceRequest<T>, _>(|request_id, old_request| {
			migrated = migrated.saturating_add(1);

			let converted_operators_vec = old_request
				.operators_with_approval_state
				.into_iter()
				.map(|(operator, state)| {
					let converted_state = match state {
						OldApprovalState::Pending => ApprovalState::Pending,
						OldApprovalState::Rejected => ApprovalState::Rejected,
						OldApprovalState::Approved { security_commitments } => {
							let commitments = BoundedVec::<
								AssetSecurityCommitment<T::AssetId>,
								MaxAssetsPerServiceOf<T>,
							>::truncate_from(security_commitments);
							ApprovalState::Approved { security_commitments: commitments }
						},
					};

					(operator, converted_state)
				})
				.collect::<Vec<_>>();

			let converted_operators = OperatorsWithApprovalState::<
				T::AccountId,
				T::AssetId,
				T::Constraints,
			>::truncate_from(converted_operators_vec);

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

			log::debug!("ApprovalStateOfServiceRequestsMigration: migrated request {}", request_id);

			Some(new_request)
		});

		log::info!(
			"ApprovalStateOfServiceRequestsMigration: Migrated {} service requests",
			migrated
		);

		T::DbWeight::get().reads_writes(migrated, migrated)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		// Count how many entries we have pre-migration
		// @dev: Error " the method `encode` exists for type `usize`, but its trait bounds were not
		// satisfied" With u32, Max ~4.2 billion entries
		let count = ServiceRequests::<T>::iter().count() as u32;
		Ok(count.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		use sp_runtime::DispatchError;

		// Ensure we have the same number of entries post-migration
		let pre_count = u32::decode(&mut &state[..])
			.map_err(|_| DispatchError::Other("Failed to decode pre-migration count"))?;
		let post_count = ServiceRequests::<T>::iter().count() as u32;

		if pre_count != post_count {
			return Err(DispatchError::Other("Number of service requests changed during migration"));
		}

		log::info!(
			"ApprovalStateOfServiceRequestsMigration: Successfully migrated {} service requests",
			post_count
		);

		Ok(())
	}
}
