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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, ReservableCurrency},
	PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
	traits::{Get, Zero},
	DispatchResult,
};
use sp_std::prelude::*;

mod functions;
mod impls;
mod rpc;
mod types;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod mock_evm;
// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
use crate::types::BalanceOf;

pub use module::*;
pub use weights::WeightInfo;

#[frame_support::pallet(dev_mode)]
pub mod module {
	use super::*;
	use scale_info::prelude::fmt::Debug;
	use sp_runtime::Saturating;
	use tangle_primitives::{
		jobs::v2::{ApprovalPrefrence, Field, MaxFields, MaxFieldsSize, ServiceBlueprint},
		roles::RoleType,
	};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// `PalletId` for the jobs pallet.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The service blueprint was not found.
		BlueprintNotFound,
		/// The caller is already registered as a service provider.
		AlreadyRegistered,
		/// The caller does not have the requirements to be a service provider.
		InvalidRegistrationInput,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new service blueprint has been created.
		BlueprintCreated {
			/// The account that created the service blueprint.
			owner: T::AccountId,
			/// The ID of the service blueprint.
			blueprint_id: u64,
		},
		/// A new service provider has been registered.
		Registered {
			/// The account that registered as a service provider.
			provider: T::AccountId,
			/// The ID of the service blueprint.
			blueprint_id: u64,
			/// The approval preference for the service provider for this specific blueprint.
			approval_preference: ApprovalPrefrence,
		},
		/// A service provider has been deregistered.
		Deregistered {
			/// The account that deregistered as a service provider.
			provider: T::AccountId,
			/// The ID of the service blueprint.
			blueprint_id: u64,
		},
		/// The approval preference for a service provider has been updated.
		ApprovalPreferenceUpdated {
			/// The account that updated the approval preference.
			provider: T::AccountId,
			/// The ID of the service blueprint.
			blueprint_id: u64,
			/// The new approval preference.
			approval_preference: ApprovalPrefrence,
		},

		/// A new service has been requested.
		ServiceRequested {
			/// The account that requested the service.
			owner: T::AccountId,
			/// The ID of the service request.
			request_id: u64,
			/// The ID of the service blueprint.
			blueprint_id: u64,
			/// The list of service providers that need to approve the service.
			required_approvals: Vec<T::AccountId>,
			/// The list of service providers that atomaticaly approved the service.
			approved: Vec<T::AccountId>,
		},
		/// A service request has been approved.
		ServiceRequestApproved {
			/// The account that approved the service.
			provider: T::AccountId,
			/// The ID of the service request.
			request_id: u64,
			/// The ID of the service blueprint.
			blueprint_id: u64,
			/// The list of service providers that need to approve the service.
			required_approvals: Vec<T::AccountId>,
			/// The list of service providers that atomaticaly approved the service.
			approved: Vec<T::AccountId>,
		},
		/// A service request has been rejected.
		ServiceRequestRejected {
			/// The account that rejected the service.
			provider: T::AccountId,
			/// The ID of the service request.
			request_id: u64,
			/// The ID of the service blueprint.
			blueprint_id: u64,
		},

		/// A service request has been updated or modified.
		ServiceRequestUpdated {
			/// The account that requested the service.
			owner: T::AccountId,
			/// The ID of the service request.
			request_id: u64,
			/// The ID of the service blueprint.
			blueprint_id: u64,
			/// The list of service providers that need to approve the service.
			required_approvals: Vec<T::AccountId>,
			/// The list of service providers that atomaticaly approved the service.
			approved: Vec<T::AccountId>,
		},
		/// A service has been initiated.
		ServiceInitiated {
			/// The owner of the service.
			owner: T::AccountId,
			/// The ID of the service request that got approved.
			request_id: u64,
			/// The ID of the service.
			service_id: u64,
			/// The ID of the service blueprint.
			blueprint_id: u64,
		},

		/// A service has been terminated.
		ServiceTerminated {
			/// The owner of the service.
			owner: T::AccountId,
			/// The ID of the service.
			service_id: u64,
			/// The ID of the service blueprint.
			blueprint_id: u64,
		},

		/// A job has been called.
		JobCalled {
			/// The account that called the job.
			caller: T::AccountId,
			/// The ID of the service.
			service_id: u64,
			/// The ID of the call.
			call_id: u64,
			/// The ID of the job.
			job: u8,
			/// The arguments of the job.
			args: Vec<Field<T::AccountId>>,
		},
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new service blueprint.
		///
		/// A Service Blueprint is a template for a service that can be instantiated later on by a
		/// user.
		///
		/// # Parameters
		/// - `origin`: The account that is creating the service blueprint.
		/// - `blueprint`: The blueprint of the service.
		pub fn create_blueprint(
			origin: OriginFor<T>,
			blueprint: ServiceBlueprint,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			// TODO: get the next blueprint id.
			let blueprint_id = Zero::zero();
			// TODO: store the blueprint.
			Self::deposit_event(Event::BlueprintCreated { owner: caller.clone(), blueprint_id });
			todo!()
		}

		/// Register the caller as a service provider for a specific blueprint.
		///
		/// The caller may require an approval first before they can accept to provide the service
		/// for the users.
		pub fn register(
			origin: OriginFor<T>,
			#[pallet::compact] blueprint_id: u64,
			approval_preference: ApprovalPrefrence,
			// NOTE: add role profiles here.
			// profile: Profile,
			registration_input: Vec<u8>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			// TODO: check if the blueprint exists.
			// TODO: check if the caller is not already registered.
			// TODO: check if the caller has the valid requirements to be a service provider.
			// TODO: store the registration.
			Self::deposit_event(Event::Registered {
				provider: caller.clone(),
				blueprint_id,
				approval_preference,
			});
			todo!()
		}

		/// Deregister the caller from being a service provider for the service blueprint
		/// so that, no more services will assigned to the caller for this specific blueprint.
		/// Note that, the caller needs to keep providing service for other active service
		/// that uses this blueprint, until the end of service time, otherwise they may get reported
		/// and slashed.
		pub fn deregister(
			origin: OriginFor<T>,
			#[pallet::compact] blueprint_id: u64,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			// TODO: check if the blueprint exists.
			// TODO: check if the caller is registered.
			// TODO: check if the caller is not providing any service for the blueprint.
			// TODO: remove the registration.
			Self::deposit_event(Event::Deregistered { provider: caller.clone(), blueprint_id });
			todo!()
		}

		/// Update the approval preference for the caller for a specific service blueprint.
		///
		/// See [`Self::register`] for more information.
		pub fn update_approval_preference(
			origin: OriginFor<T>,
			#[pallet::compact] blueprint_id: u64,
			approval_preference: ApprovalPrefrence,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			// TODO: check if the blueprint exists.
			// TODO: check if the caller is registered.
			// TODO: update the approval preference.
			// TODO: store the registration.
			Self::deposit_event(Event::ApprovalPreferenceUpdated {
				provider: caller.clone(),
				blueprint_id,
				approval_preference,
			});
			todo!()
		}
		/// Request a new service to be initiated using the provided blueprint with a list of
		/// service providers that will run your service. Optionally, you can specifiy who is permitted caller
		/// of this service, by default anyone could use this service.
		///
		/// Note that, if anyone of the participants set their [`ApprovalPreference`] to `ApprovalPreference::RequireApproval`
		/// you will need to wait until they are approve your request, otherwise (if none), the service is initiated immediately.
		pub fn request(
			origin: OriginFor<T>,
			#[pallet::compact] blueprint_id: u64,
			permitted_callers: Vec<T::AccountId>,
			service_providers: Vec<T::AccountId>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			// TODO: check if the blueprint exists.
			// TODO: check if all the service providers are registered.
			// TODO: check if any of the service providers are required approval.
			// TODO: create a new service, and store it.
			// TODO: notify the service providers, by storing the service id in their storage.
			// TODO: emit an event.
			Self::deposit_event(Event::ServiceRequested {
				owner: caller.clone(),
				request_id: 0,
				blueprint_id,
				required_approvals: vec![],
				approved: vec![],
			});
			todo!()
		}

		/// Approve a service request, so that the service can be initiated.
		pub fn approve(origin: OriginFor<T>, #[pallet::compact] request_id: u64) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			// TODO: check if the service exists.
			// TODO: check if the caller is a service provider.
			// TODO: check if the caller is required to approve the service.
			// TODO: approve the service.
			Self::deposit_event(Event::ServiceRequestApproved {
				provider: caller.clone(),
				request_id,
				blueprint_id: 0,
				required_approvals: vec![],
				approved: vec![],
			});
			// TODO: check if all the required approvals are done.
			// TODO: initiate the service.
			Self::deposit_event(Event::ServiceInitiated {
				owner: todo!(),
				request_id,
				service_id: 0,
				blueprint_id: 0,
			});
			todo!()
		}

		/// Reject a service request.
		/// The service will not be initiated, and the requester will need to update the service request.
		pub fn reject(origin: OriginFor<T>, #[pallet::compact] request_id: u64) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			// TODO: check if the service request exists.
			// TODO: check if the caller is a service provider.
			// TODO: reject the service.
			// TODO: emit an event.
			Self::deposit_event(Event::ServiceRequestRejected {
				provider: caller.clone(),
				request_id,
				blueprint_id: 0,
			});
			todo!()
		}

		/// Update the service request.
		/// The owner of the service request can update the service request, and the service providers will need to approve it again.
		pub fn update_request(
			origin: OriginFor<T>,
			#[pallet::compact] request_id: u64,
			permitted_callers: Vec<T::AccountId>,
			service_providers: Vec<T::AccountId>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			// TODO: check if the service request exists.
			// TODO: check if the caller is the owner of the service request.
			// TODO: check if all the service providers are registered.
			// TODO: check if any of the service providers are required approval.
			// TODO: update the service request.
			// TODO: notify the service providers, by storing the service id in their storage.
			// TODO: emit an event.
			Self::deposit_event(Event::ServiceRequestUpdated {
				owner: caller.clone(),
				request_id,
				blueprint_id: 0,
				required_approvals: vec![],
				approved: vec![],
			});
			todo!()
		}

		/// Terminates the service by the owner of the service.
		pub fn terminate(
			origin: OriginFor<T>,
			#[pallet::compact] service_id: u64,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			// TODO: check if the service exists.
			// TODO: check if the caller is the owner of the service.
			// TODO: terminate the service.
			Self::deposit_event(Event::ServiceTerminated {
				owner: caller.clone(),
				service_id,
				blueprint_id: 0,
			});
			todo!()
		}

		/// Call a Job in the service.
		/// The caller needs to be the owner of the service, or a permitted caller.
		pub fn job_call(
			origin: OriginFor<T>,
			#[pallet::compact] service_id: u64,
			#[pallet::compact] job: u8,
			args: BoundedVec<Field<T::AccountId>, MaxFields>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			// TODO: check if the service exists.
			// TODO: check if the caller is the owner of the service, or a permitted caller.
			// TODO: check if the job exists.
			// TODO: check if the job input is correct.
			// TODO: call the job.
			// TODO: emit an event.
			Self::deposit_event(Event::JobCalled {
				caller: caller.clone(),
				service_id,
				call_id: 0,
				job,
				args: args.into(),
			});
			todo!()
		}

		/// Submit the job result by using the call id.
		/// The caller needs to be one of the service providers for this specific service.
		pub fn job_submit(
			origin: OriginFor<T>,
			#[pallet::compact] call_id: u64,
			result: BoundedVec<Field<T::AccountId>, MaxFields>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			// TODO: check if the call exists.
			// TODO: check if the service exists, from the call_id.
			// TODO: check if the caller is a service provider.
			// TODO: check if the caller is a service provider for this specific service.
			// TODO: check if the job result is correct.
			// TODO: verify the job result.
			// TODO: store the job result.
			// TODO: emit an event.
			todo!()
		}
	}
}
