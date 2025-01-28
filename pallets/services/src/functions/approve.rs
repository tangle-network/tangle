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
	types::*, Config, Error, Event, Instances, NextInstanceId, OperatorsProfile, Pallet,
	ServiceRequests, StagingServicePayments, UserServices,
};
use frame_support::{
	pallet_prelude::*,
	traits::{fungibles::Mutate, tokens::Preservation, Currency, ExistenceRequirement},
};
use frame_system::pallet_prelude::*;
use sp_runtime::{traits::Zero, Percent};
use sp_std::vec::Vec;
use tangle_primitives::{
	services::{
		ApprovalState, Asset, AssetSecurityCommitment, Constraints, EvmAddressMapping, Service,
		ServiceRequest, StagingServicePayment,
	},
	BlueprintId,
};

impl<T: Config> Pallet<T> {
	/// Process an operator's approval for a service request.
	///
	/// This function handles the approval workflow for a service request, including:
	/// 1. Validating the operator's eligibility to approve
	/// 2. Updating the approval state with security commitments
	/// 3. Checking if all operators have approved
	/// 4. Initializing the service if fully approved
	/// 5. Processing payments to the MBSM
	///
	/// # Arguments
	///
	/// * `operator` - The account ID of the approving operator
	/// * `request_id` - The ID of the service request being approved
	/// * `native_exposure_percent` - Percentage of native token stake to expose
	/// * `asset_exposure` - Vector of asset-specific exposure commitments
	///
	/// # Returns
	///
	/// Returns a DispatchResult indicating success or the specific error that occurred
	pub fn do_approve(
		operator: T::AccountId,
		request_id: u64,
		native_exposure_percent: Percent,
		asset_exposures: Vec<AssetSecurityCommitment<T::AssetId>>,
	) -> DispatchResult {
		// Retrieve and validate the service request
		let mut request = Self::service_requests(request_id)?;

		// Ensure asset exposures don't exceed max assets per service
		ensure!(
			asset_exposures.len() <= T::MaxAssetsPerService::get() as usize,
			Error::<T>::MaxAssetsPerServiceExceeded
		);
		// Ensure asset exposures length matches requested assets length
		ensure!(
			asset_exposures.len() == request.non_native_asset_security.len(),
			Error::<T>::InvalidAssetMatching
		);
		// Ensure no duplicate assets in exposures
		let mut seen_assets = sp_std::collections::btree_set::BTreeSet::new();
		for exposure in asset_exposures.iter() {
			ensure!(seen_assets.insert(&exposure.asset), Error::<T>::DuplicateAsset);
		}

		// Ensure all assets in request have matching exposures in same order
		for (i, required_asset) in request.non_native_asset_security.iter().enumerate() {
			ensure!(
				asset_exposures[i].asset == required_asset.asset,
				Error::<T>::InvalidAssetMatching
			);
		}

		// Find and update operator's approval state
		let updated = request
			.operators_with_approval_state
			.iter_mut()
			.find(|(op, _)| op == &operator)
			.map(|(_, state)| {
				*state = ApprovalState::Approved {
					native_exposure_percent,
					asset_exposure: asset_exposures.clone(),
				}
			});
		ensure!(updated.is_some(), Error::<T>::ApprovalNotRequested);

		let blueprint_id = request.blueprint;
		let (_, blueprint) = Self::blueprints(blueprint_id)?;
		let preferences = Self::operators(blueprint_id, operator.clone())?;

		// Validate operator commitments against service requirements
		ensure!(
			native_exposure_percent >= T::NativeExposureMinimum::get(),
			Error::<T>::InvalidRequestInput
		);
		ensure!(request.validate_commitments(&asset_exposures), Error::<T>::InvalidRequestInput);

		// Call approval hook
		let (allowed, _weight) = Self::on_approve_hook(
			&blueprint,
			blueprint_id,
			&preferences,
			request_id,
			native_exposure_percent.deconstruct(),
		)
		.map_err(|_| Error::<T>::OnApproveFailure)?;
		ensure!(allowed, Error::<T>::ApprovalInterrupted);

		// Get lists of approved and pending operators
		let approved = request
			.operators_with_approval_state
			.iter()
			.filter_map(|(op, state)| {
				if matches!(state, ApprovalState::Approved { .. }) {
					Some(op.clone())
				} else {
					None
				}
			})
			.collect::<Vec<_>>();

		let pending_approvals = request
			.operators_with_approval_state
			.iter()
			.filter_map(|(op, state)| {
				if matches!(state, ApprovalState::Pending) {
					Some(op.clone())
				} else {
					None
				}
			})
			.collect::<Vec<_>>();

		// Emit approval event
		Self::deposit_event(Event::ServiceRequestApproved {
			operator: operator.clone(),
			request_id,
			blueprint_id,
			pending_approvals,
			approved: approved.clone(),
		});

		// If all operators have approved, initialize the service
		if request.is_approved() {
			Self::initialize_approved_service(request_id, request)?;
		} else {
			// Update the service request if still pending approvals
			ServiceRequests::<T>::insert(request_id, request);
		}

		Ok(())
	}

	/// Initialize a service after all operators have approved.
	///
	/// This is a helper function that handles the service initialization process including:
	/// - Creating the service instance
	/// - Processing payments
	/// - Updating operator profiles
	/// - Emitting events
	///
	/// # Arguments
	///
	/// * `request` - The approved service request to initialize
	fn initialize_approved_service(
		request_id: u64,
		request: ServiceRequest<T::Constraints, T::AccountId, BlockNumberFor<T>, T::AssetId>,
	) -> DispatchResult {
		// Remove the service request since it's now approved
		ServiceRequests::<T>::remove(request_id);

		let service_id = Self::next_instance_id();

		// Collect operator commitments
		let (native_exposures, non_native_exposures): (
			Vec<(T::AccountId, Percent)>,
			Vec<(
				T::AccountId,
				BoundedVec<
					AssetSecurityCommitment<T::AssetId>,
					// TODO: Verify this doesn't cause issues. Constraints and `T::MaxAssetsPerService` as conflicting.
					<T::Constraints as Constraints>::MaxAssetsPerService,
				>,
			)>,
		) = request
			.operators_with_approval_state
			.into_iter()
			.filter_map(|(op, state)| match state {
				ApprovalState::Approved { native_exposure_percent, asset_exposure } => {
					// This is okay because we assert that each operators approval state contains
					// a bounded list of asset exposures in the initial `do_approve` call.
					let bounded_asset_exposure = BoundedVec::try_from(asset_exposure).unwrap();
					Some(((op.clone(), native_exposure_percent), (op, bounded_asset_exposure)))
				},
				_ => None,
			})
			.unzip();

		// Update operator profiles
		for (operator, _) in &native_exposures {
			OperatorsProfile::<T>::try_mutate_exists(operator, |profile| {
				profile
					.as_mut()
					.and_then(|p| p.services.try_insert(service_id).ok())
					.ok_or(Error::<T>::NotRegistered)
			})?;
		}

		// Create bounded vectors for service instance
		let native_exposures = BoundedVec::try_from(native_exposures)
			.map_err(|_| Error::<T>::MaxServiceProvidersExceeded)?;
		let non_native_exposures = BoundedVec::try_from(non_native_exposures)
			.map_err(|_| Error::<T>::MaxServiceProvidersExceeded)?;

		// Create the service instance
		let service = Service {
			id: service_id,
			blueprint: request.blueprint,
			owner: request.owner.clone(),
			non_native_asset_security: non_native_exposures,
			native_asset_security: native_exposures,
			permitted_callers: request.permitted_callers.clone(),
			ttl: request.ttl,
			membership_model: request.membership_model,
		};

		// Update storage
		UserServices::<T>::try_mutate(&request.owner, |service_ids| {
			Instances::<T>::insert(service_id, service.clone());
			NextInstanceId::<T>::set(service_id.saturating_add(1));
			service_ids
				.try_insert(service_id)
				.map_err(|_| Error::<T>::MaxServicesPerUserExceeded)
		})?;

		// Process payment if it exists
		if let Some(payment) = Self::service_payment(request_id) {
			Self::process_service_payment(request.blueprint, &payment)?;
			StagingServicePayments::<T>::remove(request_id);
		}

		// Call service initialization hook
		let (_, blueprint) = Self::blueprints(request.blueprint)?;
		let (allowed, _weight) = Self::on_service_init_hook(
			&blueprint,
			request.blueprint,
			request_id,
			service_id,
			&request.owner,
			&request.permitted_callers,
			request.ttl,
		)
		.map_err(|_| Error::<T>::OnServiceInitHook)?;
		ensure!(allowed, Error::<T>::ServiceInitializationInterrupted);

		// Emit service initiated event
		Self::deposit_event(Event::ServiceInitiated {
			owner: request.owner,
			request_id,
			service_id,
			blueprint_id: request.blueprint,
			assets: request.non_native_asset_security.iter().map(|a| a.asset.clone()).collect(),
		});

		Ok(())
	}

	/// Process a service payment by transferring funds to the MBSM.
	///
	/// This function handles transferring payment from the pallet account to the MBSM account
	/// based on the payment asset type (native, custom, or ERC20).
	///
	/// # Arguments
	///
	/// * `payment` - The payment details including asset type and amount
	///
	/// # Returns
	///
	/// Returns a DispatchResult indicating success or the specific error that occurred
	pub(crate) fn process_service_payment(
		blueprint_id: BlueprintId,
		payment: &StagingServicePayment<T::AccountId, T::AssetId, BalanceOf<T>>,
	) -> DispatchResult {
		let (_, blueprint) = Self::blueprints(blueprint_id)?;

		// send payments to the MBSM
		let mbsm_address = Self::mbsm_address_of(&blueprint)?;
		let mbsm_account_id = T::EvmAddressMapping::into_account_id(mbsm_address);
		match payment.asset.clone() {
			Asset::Custom(asset_id) if asset_id == Zero::zero() => {
				T::Currency::transfer(
					&Self::account_id(),
					&mbsm_account_id,
					payment.amount,
					ExistenceRequirement::AllowDeath,
				)?;
			},
			Asset::Custom(asset_id) => {
				T::Fungibles::transfer(
					asset_id,
					&Self::account_id(),
					&mbsm_account_id,
					payment.amount,
					Preservation::Expendable,
				)?;
			},
			Asset::Erc20(token) => {
				let (success, _weight) =
					Self::erc20_transfer(token, Self::address(), mbsm_address, payment.amount)
						.map_err(|_| Error::<T>::OnErc20TransferFailure)?;
				ensure!(success, Error::<T>::ERC20TransferFailed);
			},
		}

		Ok(())
	}
}
