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

#[cfg(test)]
mod mock;
#[cfg(test)]
mod mock_evm;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

pub use module::*;
pub use weights::WeightInfo;

#[frame_support::pallet(dev_mode)]
pub mod module {
	use super::*;
	use sp_runtime::Saturating;
	use tangle_primitives::jobs::v2::{
		ApprovalPrefrence, ApprovalState, Field, MaxFields, MaxPermittedCallers,
		MaxProvidersPerService, Service, ServiceBlueprint, ServiceRequest, TypeCheckError,
	};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The origin which may set filter.
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
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
		/// The caller is not registered as a service provider.
		NotRegistered,
		/// The service request was not found.
		ServiceRequestNotFound,
		/// The service was not found.
		ServiceNotFound,
		/// An error occurred while type checking the provided input input.
		TypeCheck(TypeCheckError),
		/// The maximum number of permitted callers per service has been exceeded.
		MaxPermittedCallersExceeded,
		/// The maximum number of service providers per service has been exceeded.
		MaxServiceProvidersExceeded,
		/// The maximum number of fields per request has been exceeded.
		MaxFieldsExceeded,
		/// The approval is not requested for the service provider (the caller).
		ApprovalNotRequested,
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
			/// The arguments used for registration.
			registration_args: Vec<Field<T::AccountId>>,
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
			/// The ID of the service request that got approved (if required).
			request_id: Option<u64>,
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

	// Counters

	/// The next free ID for a service blueprint.
	#[pallet::storage]
	#[pallet::getter(fn next_blueprint_id)]
	pub type NextBlueprintId<T> = StorageValue<_, u64, ValueQuery>;

	/// The next free ID for a service request.
	#[pallet::storage]
	#[pallet::getter(fn next_service_request_id)]
	pub type NextServiceRequestId<T> = StorageValue<_, u64, ValueQuery>;

	/// The next free ID for a service Instance.
	#[pallet::storage]
	#[pallet::getter(fn next_instance_id)]
	pub type NextInstanceId<T> = StorageValue<_, u64, ValueQuery>;

	/// The service blueprints along with their owner.
	#[pallet::storage]
	#[pallet::getter(fn blueprints)]
	pub type Blueprints<T: Config> = StorageMap<
		_,
		Identity,
		u64,
		(T::AccountId, ServiceBlueprint),
		ResultQuery<Error<T>::BlueprintNotFound>,
	>;

	/// The service providers for a specific service blueprint.
	#[pallet::storage]
	#[pallet::getter(fn service_providers)]
	pub type ServiceProviders<T: Config> = StorageDoubleMap<
		_,
		Identity,
		u64,
		Identity,
		T::AccountId,
		ApprovalPrefrence,
		ResultQuery<Error<T>::NotRegistered>,
	>;

	/// The service requests along with their owner.
	/// Request ID -> Service Request
	#[pallet::storage]
	#[pallet::getter(fn service_requests)]
	pub type ServiceRequests<T: Config> = StorageMap<
		_,
		Identity,
		u64,
		ServiceRequest<T::AccountId, BlockNumberFor<T>>,
		ResultQuery<Error<T>::ServiceRequestNotFound>,
	>;

	/// The Services Instances
	/// Service ID -> Service
	#[pallet::storage]
	#[pallet::getter(fn services)]
	pub type Instances<T: Config> = StorageMap<
		_,
		Identity,
		u64,
		Service<T::AccountId, BlockNumberFor<T>>,
		ResultQuery<Error<T>::ServiceNotFound>,
	>;

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
			let owner = ensure_signed(origin)?;
			let blueprint_id = NextBlueprintId::<T>::get();
			Blueprints::<T>::insert(blueprint_id, (owner.clone(), blueprint));
			NextBlueprintId::<T>::set(blueprint_id.saturating_add(1));

			Self::deposit_event(Event::BlueprintCreated { owner, blueprint_id });
			Ok(())
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
			registration_args: Vec<Field<T::AccountId>>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let (_, blueprint) = Blueprints::<T>::get(blueprint_id)?;
			let already_registered = ServiceProviders::<T>::contains_key(blueprint_id, &caller);
			ensure!(!already_registered, Error::<T>::AlreadyRegistered);
			// TODO: check if the caller has the valid requirements to be a service provider.
			// TODO: call into EVM here.
			blueprint
				.type_check_registration(&registration_args)
				.map_err(Error::<T>::TypeCheck)?;
			ServiceProviders::<T>::insert(blueprint_id, &caller, approval_preference);

			Self::deposit_event(Event::Registered {
				provider: caller.clone(),
				blueprint_id,
				approval_preference,
				registration_args,
			});

			Ok(())
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
			ensure!(Blueprints::<T>::contains_key(blueprint_id), Error::<T>::BlueprintNotFound);
			let registered = ServiceProviders::<T>::contains_key(blueprint_id, &caller);
			ensure!(registered, Error::<T>::NotRegistered);
			// TODO: check if the caller is not providing any service for the blueprint.
			ServiceProviders::<T>::remove(blueprint_id, &caller);

			Self::deposit_event(Event::Deregistered { provider: caller.clone(), blueprint_id });
			Ok(())
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
			ensure!(Blueprints::<T>::contains_key(blueprint_id), Error::<T>::BlueprintNotFound);
			ServiceProviders::<T>::try_mutate_exists(
				blueprint_id,
				&caller,
				|current_approval_preference| {
					current_approval_preference
						.as_mut()
						.map(|v| *v = approval_preference)
						.ok_or(Error::<T>::NotRegistered)
				},
			)?;

			Self::deposit_event(Event::ApprovalPreferenceUpdated {
				provider: caller.clone(),
				blueprint_id,
				approval_preference,
			});
			Ok(())
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
			#[pallet::compact] ttl: BlockNumberFor<T>,
			request_args: Vec<Field<T::AccountId>>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let (_, blueprint) = Blueprints::<T>::get(blueprint_id)?;

			blueprint.type_check_request(&request_args).map_err(Error::<T>::TypeCheck)?;
			// TODO: check if all the service providers are registered.
			// TODO: check if any of the service providers are required approval.
			let mut required_approvals = Vec::new();
			let mut approved = Vec::new();
			for provider in &service_providers {
				let approval_preference = ServiceProviders::<T>::get(blueprint_id, provider)?;
				if approval_preference == ApprovalPrefrence::Required {
					required_approvals.push(provider.clone());
				} else {
					approved.push(provider.clone());
				}
			}

			let permitted_callers =
				BoundedVec::<_, MaxPermittedCallers>::try_from(permitted_callers)
					.map_err(|_| Error::<T>::MaxPermittedCallersExceeded)?;
			if required_approvals.is_empty() {
				// No approval is required, initiate the service immediately.
				let service_id = NextInstanceId::<T>::get();
				let service = Service {
					blueprint: blueprint_id,
					owner: caller.clone(),
					permitted_callers,
					ttl,
				};
				Instances::<T>::insert(service_id, service);
				NextInstanceId::<T>::set(service_id.saturating_add(1));
				Self::deposit_event(Event::ServiceInitiated {
					owner: caller.clone(),
					request_id: None,
					service_id,
					blueprint_id,
				});

				Ok(())
			} else {
				let request_id = NextServiceRequestId::<T>::get();
				let providers = required_approvals
					.iter()
					.cloned()
					.map(|v| (v, ApprovalState::Pending))
					.chain(approved.iter().cloned().map(|v| (v, ApprovalState::Approved)))
					.collect::<Vec<_>>();

				let args = BoundedVec::<_, MaxFields>::try_from(request_args)
					.map_err(|_| Error::<T>::MaxFieldsExceeded)?;

				let providers_with_approval_state =
					BoundedVec::<_, MaxProvidersPerService>::try_from(providers)
						.map_err(|_| Error::<T>::MaxServiceProvidersExceeded)?;
				let service_request = ServiceRequest {
					blueprint: blueprint_id,
					owner: caller.clone(),
					ttl,
					args,
					permitted_callers,
					providers_with_approval_state,
				};
				ServiceRequests::<T>::insert(request_id, service_request);
				NextServiceRequestId::<T>::set(request_id.saturating_add(1));

				Self::deposit_event(Event::ServiceRequested {
					owner: caller.clone(),
					request_id,
					blueprint_id,
					required_approvals,
					approved,
				});

				Ok(())
			}
		}

		/// Approve a service request, so that the service can be initiated.
		pub fn approve(origin: OriginFor<T>, #[pallet::compact] request_id: u64) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let mut request = ServiceRequests::<T>::get(request_id)?;
			let updated = request
				.providers_with_approval_state
				.iter_mut()
				.find(|(v, _)| v == &caller)
				.map(|(_, s)| *s = ApprovalState::Approved);
			ensure!(updated.is_some(), Error::<T>::ApprovalNotRequested);

			let approved = request
				.providers_with_approval_state
				.iter()
				.filter(|(_, s)| *s == ApprovalState::Approved)
				.map(|(v, _)| v.clone())
				.collect::<Vec<_>>();
			let required_approvals = request
				.providers_with_approval_state
				.iter()
				.filter(|(_, s)| *s == ApprovalState::Pending)
				.map(|(v, _)| v.clone())
				.collect::<Vec<_>>();

			let blueprint_id = request.blueprint;
			let owner = request.owner.clone();
			let is_approved = request.is_approved();
			let ttl = request.ttl;
			let permitted_callers = request.permitted_callers.clone();

			// we emit this event regardless of the outcome of the approval.
			Self::deposit_event(Event::ServiceRequestApproved {
				provider: caller.clone(),
				request_id,
				blueprint_id,
				required_approvals,
				approved,
			});

			if is_approved {
				// remove the service request.
				ServiceRequests::<T>::remove(request_id);

				let service_id = NextInstanceId::<T>::get();
				let service = Service {
					blueprint: blueprint_id,
					owner: owner.clone(),
					permitted_callers,
					ttl,
				};
				Instances::<T>::insert(service_id, service);
				NextInstanceId::<T>::set(service_id.saturating_add(1));

				Self::deposit_event(Event::ServiceInitiated {
					owner,
					request_id: Some(request_id),
					service_id,
					blueprint_id,
				});
			} else {
				// Update the service request.
				ServiceRequests::<T>::insert(request_id, request);
			}
			Ok(())
		}

		/// Reject a service request.
		/// The service will not be initiated, and the requester will need to update the service request.
		pub fn reject(origin: OriginFor<T>, #[pallet::compact] request_id: u64) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let mut request = ServiceRequests::<T>::get(request_id)?;
			let updated = request
				.providers_with_approval_state
				.iter_mut()
				.find(|(v, _)| v == &caller)
				.map(|(_, s)| *s = ApprovalState::Rejected);
			ensure!(updated.is_some(), Error::<T>::ApprovalNotRequested);

			Self::deposit_event(Event::ServiceRequestRejected {
				provider: caller.clone(),
				request_id,
				blueprint_id: request.blueprint,
			});
			Ok(())
		}

		/// Terminates the service by the owner of the service.
		pub fn terminate(
			origin: OriginFor<T>,
			#[pallet::compact] service_id: u64,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let service = Instances::<T>::get(service_id)?;
			ensure!(service.owner == caller, DispatchError::BadOrigin);
			Instances::<T>::remove(service_id);

			Self::deposit_event(Event::ServiceTerminated {
				owner: caller.clone(),
				service_id,
				blueprint_id: service.blueprint,
			});
			Ok(())
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
