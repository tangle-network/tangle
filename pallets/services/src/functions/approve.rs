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
	BalanceOf, Config, Error, Event, Instances, NextInstanceId, OperatorsProfile, Pallet,
	ServiceRequests, ServiceStatus, StagingServicePayments, UserServices,
};
use frame_support::{
	BoundedVec,
	dispatch::DispatchResult,
	ensure,
	traits::{
		Currency, ExistenceRequirement,
		fungibles::{Inspect, Mutate},
		tokens::Preservation,
	},
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::traits::Zero;
use sp_std::vec::Vec;
use tangle_primitives::{
	services::{
		ApprovalState, Asset, AssetSecurityCommitment, AssetSecurityRequirement, EvmAddressMapping,
		Service, ServiceRequest, StagingServicePayment,
	},
	traits::MultiAssetDelegationInfo,
};

impl<T: Config> Pallet<T> {
	/// Validate operator security commitments for service approval
	pub fn validate_operator_security_commitments(
		operator: &T::AccountId,
		requirements: &[AssetSecurityRequirement<T::AssetId>],
		commitments: &[AssetSecurityCommitment<T::AssetId>],
	) -> DispatchResult {
		// Ensure operator is active
		ensure!(
			T::OperatorDelegationManager::is_operator_active(operator),
			Error::<T>::OperatorNotActive
		);

		// Validate asset existence for each commitment
		for commitment in commitments {
			Self::validate_asset_existence(&commitment.asset)?;
		}

		// Check for commitment-requirement matching
		for requirement in requirements {
			let commitment = commitments
				.iter()
				.find(|c| c.asset == requirement.asset)
				.ok_or(Error::<T>::MissingAssetCommitment)?;

			// Validate commitment percentage is within requirement bounds
			ensure!(
				commitment.exposure_percent >= requirement.min_exposure_percent,
				Error::<T>::CommitmentBelowMinimum
			);
			ensure!(
				commitment.exposure_percent <= requirement.max_exposure_percent,
				Error::<T>::CommitmentAboveMaximum
			);

			// Get operator's total delegated stake for this asset
			let asset_stake = match &requirement.asset {
				Asset::Custom(asset_id) => {
					if *asset_id == Zero::zero() {
						// Native asset - get operator stake from nomination delegations
						T::OperatorDelegationManager::get_operator_stake(operator)
					} else {
						// For custom assets, get total delegation for this specific asset
						T::OperatorDelegationManager::get_total_delegation_by_asset(
							operator,
							&requirement.asset,
						)
					}
				},
				Asset::Erc20(_) => {
					// For ERC20 tokens, get total delegation for this specific asset
					T::OperatorDelegationManager::get_total_delegation_by_asset(
						operator,
						&requirement.asset,
					)
				},
			};

			// Ensure operator has sufficient delegated stake for the commitment
			ensure!(asset_stake > BalanceOf::<T>::zero(), Error::<T>::NoOperatorStake);
		}

		// Check for unexpected commitments (commitments without requirements)
		for commitment in commitments {
			ensure!(
				requirements.iter().any(|r| r.asset == commitment.asset),
				Error::<T>::UnexpectedAssetCommitment
			);
		}

		Ok(())
	}

	/// Validate that an asset exists in the system
	fn validate_asset_existence(asset: &Asset<T::AssetId>) -> DispatchResult {
		match asset {
			Asset::Custom(asset_id) => {
				// Native asset (asset_id == 0) always exists
				if *asset_id != Zero::zero() {
					// For custom assets, verify they exist in the Fungibles system
					ensure!(
						T::Fungibles::asset_exists(asset_id.clone()),
						Error::<T>::AssetNotFound
					);
				}
			},
			Asset::Erc20(token_address) => {
				// Validate ERC20 token address is not zero address
				ensure!(*token_address != sp_core::H160::zero(), Error::<T>::InvalidErc20Address);
				// Note: We could add more sophisticated ERC20 validation here
				// such as checking if the contract exists at the address
			},
		}
		Ok(())
	}
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
		security_commitments: &[AssetSecurityCommitment<T::AssetId>],
	) -> DispatchResult {
		// Retrieve and validate the service request
		let mut request = Self::service_requests(request_id)?;

		// First validate that commitments match the service request requirements (primitive
		// validation)
		ensure!(
			request.validate_security_commitments(security_commitments),
			Error::<T>::InvalidSecurityCommitments
		);

		// Then validate operator security commitments (asset existence and stake validation)
		Self::validate_operator_security_commitments(
			&operator,
			&request.security_requirements,
			security_commitments,
		)?;

		// Find and update operator's approval state
		let updated = request
			.operators_with_approval_state
			.iter_mut()
			.find(|(op, _)| op == &operator)
			.map(|(_, state)| {
				*state =
					ApprovalState::Approved { security_commitments: security_commitments.to_vec() }
			});
		ensure!(updated.is_some(), Error::<T>::ApprovalNotRequested);

		let blueprint_id = request.blueprint;
		let (_, blueprint) = Self::blueprints(blueprint_id)?;
		let preferences = Self::operators(blueprint_id, operator.clone())?;

		// Call approval hook
		// TODO: Update the approval hook CC @shekohex @1xstj
		let (allowed, _weight) =
			Self::on_approve_hook(&blueprint, blueprint_id, &preferences, request_id, 0u8)
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
				if matches!(state, ApprovalState::Pending) { Some(op.clone()) } else { None }
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
		let operator_security_commitments = request
			.operators_with_approval_state
			.into_iter()
			.filter_map(|(op, state)| match state {
				ApprovalState::Approved { security_commitments } => {
					// This is okay because we assert that each operators approval state contains
					// a bounded list of asset exposures in the initial `do_approve` call.
					let security_commitments = BoundedVec::try_from(security_commitments).unwrap();
					Some((op, security_commitments))
				},
				_ => None,
			})
			.collect::<Vec<_>>();

		// Update operator profiles
		for (operator, _) in &operator_security_commitments {
			OperatorsProfile::<T>::try_mutate_exists(operator, |profile| {
				profile
					.as_mut()
					.and_then(|p| p.services.try_insert(service_id).ok())
					.ok_or(Error::<T>::NotRegistered)
			})?;
		}

		// Create bounded vectors for service instance
		let operator_security_commitments = BoundedVec::try_from(operator_security_commitments)
			.map_err(|_| Error::<T>::MaxServiceProvidersExceeded)?;

		// Create the service instance
		let service = Service {
			id: service_id,
			blueprint: request.blueprint,
			owner: request.owner.clone(),
			args: request.args,
			operator_security_commitments: operator_security_commitments.clone(),
			security_requirements: request.security_requirements,
			permitted_callers: request.permitted_callers.clone(),
			ttl: request.ttl,
			membership_model: request.membership_model,
		};

		UserServices::<T>::try_mutate(&request.owner, |service_ids| {
			Instances::<T>::insert(service_id, service.clone());
			ServiceStatus::<T>::insert(request.blueprint, service_id, ());
			NextInstanceId::<T>::set(service_id.saturating_add(1));
			service_ids
				.try_insert(service_id)
				.map_err(|_| Error::<T>::MaxServicesPerUserExceeded)
		})?;

		// Process payment if it exists - Transfer payment to MBSM
		if let Some(payment) = Self::service_payment(request_id) {
			// Transfer the payment to the MBSM
			Self::transfer_payment_to_mbsm(request.blueprint, &payment)?;

			// Remove the payment from staging
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
			operator_security_commitments,
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
	/// * `blueprint_id` - The blueprint ID to get the MBSM address
	/// * `payment` - The payment details including asset type and amount
	///
	/// # Returns
	///
	/// Returns a DispatchResult indicating success or the specific error that occurred
	pub(crate) fn transfer_payment_to_mbsm(
		blueprint_id: u64,
		payment: &StagingServicePayment<T::AccountId, T::AssetId, BalanceOf<T>>,
	) -> DispatchResult {
		let (_, blueprint) = Self::blueprints(blueprint_id)?;

		// send payments to the MBSM
		let mbsm_address = Self::mbsm_address_of(&blueprint)?;
		let mbsm_account_id = T::EvmAddressMapping::into_account_id(mbsm_address);
		match payment.asset.clone() {
			Asset::Custom(asset_id) if asset_id == Zero::zero() => {
				T::Currency::transfer(
					&Self::pallet_account(),
					&mbsm_account_id,
					payment.amount,
					ExistenceRequirement::AllowDeath,
				)?;
			},
			Asset::Custom(asset_id) => {
				T::Fungibles::transfer(
					asset_id,
					&Self::pallet_account(),
					&mbsm_account_id,
					payment.amount,
					Preservation::Expendable,
				)?;
			},
			Asset::Erc20(token) => {
				let (success, _weight) = Self::erc20_transfer(
					token,
					Self::pallet_evm_account(),
					mbsm_address,
					payment.amount,
				)
				.map_err(|_| Error::<T>::ERC20TransferFailed)?;
				ensure!(success, Error::<T>::ERC20TransferFailed);
			},
		}

		Ok(())
	}
}
