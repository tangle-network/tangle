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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::type_complexity)]

#[cfg(not(feature = "std"))]
extern crate alloc;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency},
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
	traits::{Get, Zero},
	DispatchResult,
};
use tangle_primitives::{
	services::{AssetSecurityCommitment, AssetSecurityRequirement, MembershipModel},
	traits::MultiAssetDelegationInfo,
	BlueprintId, InstanceId, JobCallId, ServiceRequestId,
};

pub mod functions;
mod impls;
mod rpc;
pub mod types;
use types::*;

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

#[cfg(feature = "runtime-benchmarks")]
pub use impls::BenchmarkingOperatorDelegationManager;

#[allow(clippy::too_many_arguments)]
#[frame_support::pallet(dev_mode)]
pub mod module {
	use super::*;
	use frame_support::{
		dispatch::PostDispatchInfo,
		traits::fungibles::{Inspect, Mutate},
		PalletId,
	};
	use sp_core::H160;
	use sp_runtime::{traits::MaybeSerializeDeserialize, Percent};
	use sp_std::vec::Vec;
	use tangle_primitives::services::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The origin which may set filter.
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The fungibles trait used for managing fungible assets.
		type Fungibles: Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = BalanceOf<Self>>
			+ Mutate<Self::AccountId, AssetId = Self::AssetId>;

		/// PalletId used for deriving the AccountId and EVM address.
		/// This account receives slashed assets upon slash event processing.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type SlashRecipient: Get<Self::AccountId>;

		/// A type that implements the `EvmRunner` trait for the execution of EVM
		/// transactions.
		type EvmRunner: tangle_primitives::services::EvmRunner<Self>;

		/// A type that implements the `EvmGasWeightMapping` trait for the conversion of EVM gas to
		/// Substrate weight and vice versa.
		type EvmGasWeightMapping: tangle_primitives::services::EvmGasWeightMapping;

		/// A type that implements the `EvmAddressMapping` trait for the conversion of EVM address
		type EvmAddressMapping: tangle_primitives::services::EvmAddressMapping<Self::AccountId>;

		/// The asset ID type.
		type AssetId: AssetIdT;

		/// Maximum number of fields in a job call.
		#[pallet::constant]
		type MaxFields: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Maximum size of a field in a job call.
		#[pallet::constant]
		type MaxFieldsSize: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Maximum length of metadata string length.
		#[pallet::constant]
		type MaxMetadataLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Maximum number of jobs per service.
		#[pallet::constant]
		type MaxJobsPerService: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Maximum number of Operators per service.
		#[pallet::constant]
		type MaxOperatorsPerService: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Maximum number of permitted callers per service.
		#[pallet::constant]
		type MaxPermittedCallers: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Maximum number of services per operator.
		#[pallet::constant]
		type MaxServicesPerOperator: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Maximum number of blueprints per operator.
		#[pallet::constant]
		type MaxBlueprintsPerOperator: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Maximum number of services per user.
		#[pallet::constant]
		type MaxServicesPerUser: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Maximum number of binaries per gadget.
		#[pallet::constant]
		type MaxBinariesPerGadget: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Maximum number of sources per gadget.
		#[pallet::constant]
		type MaxSourcesPerGadget: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Git owner maximum length.
		#[pallet::constant]
		type MaxGitOwnerLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Git repository maximum length.
		#[pallet::constant]
		type MaxGitRepoLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Git tag maximum length.
		#[pallet::constant]
		type MaxGitTagLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// binary name maximum length.
		#[pallet::constant]
		type MaxBinaryNameLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// IPFS hash maximum length.
		#[pallet::constant]
		type MaxIpfsHashLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Container registry maximum length.
		#[pallet::constant]
		type MaxContainerRegistryLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Container image name maximum length.
		#[pallet::constant]
		type MaxContainerImageNameLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Container image tag maximum length.
		#[pallet::constant]
		type MaxContainerImageTagLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Maximum number of assets per service.
		#[pallet::constant]
		type MaxAssetsPerService: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Maximum number of versions of Master Blueprint Service Manager allowed.
		#[pallet::constant]
		type MaxMasterBlueprintServiceManagerVersions: Get<u32>
			+ Default
			+ Parameter
			+ MaybeSerializeDeserialize;

		/// The constraints for the service module.
		/// use [crate::types::ConstraintsOf] with `Self` to implement this trait.
		type Constraints: Constraints;

		/// Manager for getting operator stake and delegation info
		type OperatorDelegationManager: tangle_primitives::traits::MultiAssetDelegationInfo<
			Self::AccountId,
			BalanceOf<Self>,
			BlockNumberFor<Self>,
			Self::AssetId,
		>;

		/// Manager for slashing that dispatches slash operations to `pallet-multi-asset-delegation`.
		type SlashManager: tangle_primitives::traits::SlashManager<
			Self::AccountId,
			BalanceOf<Self>,
			Self::AssetId,
		>;

		/// Number of eras that slashes are deferred by, after computation.
		///
		/// This should be less than the bonding duration. Set to 0 if slashes
		/// should be applied immediately, without opportunity for intervention.
		#[pallet::constant]
		type SlashDeferDuration: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;

		/// The origin which can manage Add a new Master Blueprint Service Manager revision.
		type MasterBlueprintServiceManagerUpdateOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The minimum percentage of native token stake that operators must expose for slashing.
		#[pallet::constant]
		type NativeExposureMinimum: Get<Percent> + Default + Parameter + MaybeSerializeDeserialize;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn integrity_test() {
			// Ensure that the pallet's configuration is valid.
			// 1. Make sure that pallet's substrate address maps correctly back to the EVM address
			let evm_address = T::EvmAddressMapping::into_address(Self::pallet_account());
			assert_eq!(
				evm_address,
				Self::pallet_evm_account(),
				"Services: EVM address mapping is incorrect."
			);
		}

		/// On initialize, we should check for any unapplied slashes and apply them.
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			let mut weight = Zero::zero();
			let current_era = T::OperatorDelegationManager::get_current_round();
			let slash_defer_duration = T::SlashDeferDuration::get();

			// Only process slashes from eras that have completed their deferral period
			let process_era = current_era.saturating_sub(slash_defer_duration);

			// Get all unapplied slashes for this era
			let prefix_iter = UnappliedSlashes::<T>::iter_prefix(process_era);
			for (index, slash) in prefix_iter {
				match Self::apply_slash(slash) {
					Ok(weight_used) => {
						weight = weight_used.checked_add(&weight).unwrap_or_else(Zero::zero);
						// Remove the slash from storage after successful application
						UnappliedSlashes::<T>::remove(process_era, index);
					},
					Err(_) => {
						log::error!("Failed to apply slash for index: {:?}", index);
					},
				}
			}
			weight
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The service blueprint was not found.
		BlueprintNotFound,
		/// Blueprint creation is interrupted.
		BlueprintCreationInterrupted,
		/// The caller is already registered as a operator.
		AlreadyRegistered,
		/// The caller does not have the requirements to be a operator.
		InvalidRegistrationInput,
		/// The Operator is not allowed to unregister.
		NotAllowedToUnregister,
		/// The Operator is not allowed to update their price targets.
		NotAllowedToUpdatePriceTargets,
		/// The caller does not have the requirements to request a service.
		InvalidRequestInput,
		/// The caller does not have the requirements to call a job.
		InvalidJobCallInput,
		/// The caller provided an invalid job result.
		InvalidJobResult,
		/// The caller is not registered as a operator.
		NotRegistered,
		/// Approval Process is interrupted.
		ApprovalInterrupted,
		/// Rejection Process is interrupted.
		RejectionInterrupted,
		/// The service request was not found.
		ServiceRequestNotFound,
		/// Service Initialization interrupted.
		ServiceInitializationInterrupted,
		/// The service was not found.
		ServiceNotFound,
		/// The termination of the service was interrupted.
		TerminationInterrupted,
		/// An error occurred while type checking the provided input input.
		TypeCheck(TypeCheckError),
		/// The maximum number of permitted callers per service has been exceeded.
		MaxPermittedCallersExceeded,
		/// The maximum number of operators per service has been exceeded.
		MaxServiceProvidersExceeded,
		/// The maximum number of services per user has been exceeded.
		MaxServicesPerUserExceeded,
		/// The maximum number of fields per request has been exceeded.
		MaxFieldsExceeded,
		/// The approval is not requested for the operator (the caller).
		ApprovalNotRequested,
		/// The requested job definition does not exist.
		/// This error is returned when the requested job definition does not exist in the service
		/// blueprint.
		JobDefinitionNotFound,
		/// Either the service or the job call was not found.
		ServiceOrJobCallNotFound,
		/// The result of the job call was not found.
		JobCallResultNotFound,
		/// An error occurred while encoding the EVM ABI.
		EVMAbiEncode,
		/// An error occurred while decoding the EVM ABI.
		EVMAbiDecode,
		/// Operator profile not found.
		OperatorProfileNotFound,
		/// Maximum number of services per Provider reached.
		MaxServicesPerProviderExceeded,
		/// The operator is not active, ensure operator status is ACTIVE in multi-asset-delegation
		OperatorNotActive,
		/// No assets provided for the service, at least one asset is required.
		NoAssetsProvided,
		/// Duplicate assets provided
		DuplicateAsset,
		/// The maximum number of assets per service has been exceeded.
		MaxAssetsPerServiceExceeded,
		/// Assets don't match
		InvalidAssetMatching,
		/// Offender is not a registered operator.
		OffenderNotOperator,
		/// Offender is not an active operator.
		OffenderNotActiveOperator,
		/// The Service Blueprint did not return a slashing origin for this service.
		NoSlashingOrigin,
		/// The Service Blueprint did not return a dispute origin for this service.
		NoDisputeOrigin,
		/// The Unapplied Slash are not found.
		UnappliedSlashNotFound,
		/// The Supplied Master Blueprint Service Manager Revision is not found.
		MasterBlueprintServiceManagerRevisionNotFound,
		/// Maximum number of Master Blueprint Service Manager revisions reached.
		MaxMasterBlueprintServiceManagerVersionsExceeded,
		/// The ERC20 transfer failed.
		ERC20TransferFailed,
		/// Missing EVM Origin for the EVM execution.
		MissingEVMOrigin,
		/// Expected the account to be an EVM address.
		ExpectedEVMAddress,
		/// Expected the account to be an account ID.
		ExpectedAccountId,
		/// Request hook failure
		OnRequestFailure,
		/// Register hook failure
		OnRegisterHookFailed,
		/// ERC20 transfer hook failure
		OnErc20TransferFailure,
		/// Approve service request hook failure
		OnApproveFailure,
		/// Reject service request hook failure
		OnRejectFailure,
		/// Service init hook
		OnServiceInitHook,
		/// Membership model not supported by blueprint
		UnsupportedMembershipModel,
		/// Service does not support dynamic membership
		DynamicMembershipNotSupported,
		/// Cannot join service - rejected by blueprint
		JoinRejected,
		/// Cannot leave service - rejected by blueprint
		LeaveRejected,
		/// Invalid minimum # of operators (zero) or greater than max
		InvalidMinOperators,
		/// Maximum operators reached
		MaxOperatorsReached,
		/// Insufficient # of operators
		InsufficientOperators,
		/// Can join hook failure
		OnCanJoinFailure,
		/// Can leave hook failure
		OnCanLeaveFailure,
		/// Operator join hook failure
		OnOperatorJoinFailure,
		/// Operator leave hook failure
		OnOperatorLeaveFailure,
		/// Operator is a member or has already joined the service
		AlreadyJoined,
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
		/// An operator has pre-registered for a service blueprint.
		PreRegistration {
			/// The account that pre-registered as an operator.
			operator: T::AccountId,
			/// The ID of the service blueprint.
			blueprint_id: u64,
		},
		/// An new operator has been registered.
		Registered {
			/// The account that registered as a operator.
			provider: T::AccountId,
			/// The ID of the service blueprint.
			blueprint_id: u64,
			/// The preferences for the operator for this specific blueprint.
			preferences: OperatorPreferences,
			/// The arguments used for registration.
			registration_args: Vec<Field<T::Constraints, T::AccountId>>,
		},
		/// An operator has been unregistered.
		Unregistered {
			/// The account that unregistered as am operator.
			operator: T::AccountId,
			/// The ID of the service blueprint.
			blueprint_id: u64,
		},

		/// The price targets for an operator has been updated.
		PriceTargetsUpdated {
			/// The account that updated the approval preference.
			operator: T::AccountId,
			/// The ID of the service blueprint.
			blueprint_id: u64,
			/// The new price targets.
			price_targets: PriceTargets,
		},

		/// A new service has been requested.
		ServiceRequested {
			/// The account that requested the service.
			owner: T::AccountId,
			/// The ID of the service request.
			request_id: u64,
			/// The ID of the service blueprint.
			blueprint_id: u64,
			/// The list of operators that need to approve the service.
			pending_approvals: Vec<T::AccountId>,
			/// The list of operators that automatically approved the service.
			approved: Vec<T::AccountId>,
			/// The list of asset security requirements for the service.
			asset_security: Vec<(Asset<T::AssetId>, Vec<(T::AccountId, Percent)>)>,
		},
		/// A service request has been approved.
		ServiceRequestApproved {
			/// The account that approved the service.
			operator: T::AccountId,
			/// The ID of the service request.
			request_id: u64,
			/// The ID of the service blueprint.
			blueprint_id: u64,
			/// The list of operators that need to approve the service.
			pending_approvals: Vec<T::AccountId>,
			/// The list of operators that atomaticaly approved the service.
			approved: Vec<T::AccountId>,
		},
		/// A service request has been rejected.
		ServiceRequestRejected {
			/// The account that rejected the service.
			operator: T::AccountId,
			/// The ID of the service request.
			request_id: u64,
			/// The ID of the service blueprint.
			blueprint_id: u64,
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
			/// The list of assets that are being used to secure the service.
			assets: Vec<Asset<T::AssetId>>,
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
			/// The index of the job.
			job: u8,
			/// The arguments of the job.
			args: Vec<Field<T::Constraints, T::AccountId>>,
		},

		/// A job result has been submitted.
		JobResultSubmitted {
			/// The account that submitted the job result.
			operator: T::AccountId,
			/// The ID of the service.
			service_id: u64,
			/// The ID of the call.
			call_id: u64,
			/// The index of the job.
			job: u8,
			/// The result of the job.
			result: Vec<Field<T::Constraints, T::AccountId>>,
		},
		/// EVM execution reverted with a reason.
		EvmReverted { from: H160, to: H160, data: Vec<u8>, reason: Vec<u8> },
		/// An Operator has an unapplied slash.
		UnappliedSlash {
			/// The index of the slash.
			index: u32,
			/// The account that has an unapplied slash.
			operator: T::AccountId,
			/// The amount of the slash.
			amount: BalanceOf<T>,
			/// Service ID
			service_id: u64,
			/// Blueprint ID
			blueprint_id: u64,
			/// Era index
			era: u32,
		},
		/// An Unapplied Slash got discarded.
		SlashDiscarded {
			/// The index of the slash.
			index: u32,
			/// The account that has an unapplied slash.
			operator: T::AccountId,
			/// The amount of the slash.
			amount: BalanceOf<T>,
			/// Service ID
			service_id: u64,
			/// Blueprint ID
			blueprint_id: u64,
			/// Era index
			era: u32,
		},
		/// An Operator has been slashed.
		OperatorSlashed {
			/// The account that has been slashed.
			operator: T::AccountId,
			/// The amount of the slash.
			amount: BalanceOf<T>,
			/// Service ID
			service_id: u64,
			/// Blueprint ID
			blueprint_id: u64,
			/// Era index
			era: u32,
		},
		/// A Delegator has been slashed.
		DelegatorSlashed {
			/// The account that has been slashed.
			delegator: T::AccountId,
			/// The amount of the slash.
			amount: BalanceOf<T>,
			/// Service ID
			service_id: u64,
			/// Blueprint ID
			blueprint_id: u64,
			/// Era index
			era: u32,
		},
		/// The Master Blueprint Service Manager has been revised.
		MasterBlueprintServiceManagerRevised {
			/// The revision number of the Master Blueprint Service Manager.
			revision: u32,
			/// The address of the Master Blueprint Service Manager.
			address: H160,
		},
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	// Counters

	/// The next free ID for a service blueprint.
	#[pallet::storage]
	#[pallet::getter(fn next_blueprint_id)]
	pub type NextBlueprintId<T> = StorageValue<_, BlueprintId, ValueQuery>;

	/// The next free ID for a service request.
	#[pallet::storage]
	#[pallet::getter(fn next_service_request_id)]
	pub type NextServiceRequestId<T> = StorageValue<_, ServiceRequestId, ValueQuery>;

	/// The next free ID for a service Instance.
	#[pallet::storage]
	#[pallet::getter(fn next_instance_id)]
	pub type NextInstanceId<T> = StorageValue<_, InstanceId, ValueQuery>;

	/// The next free ID for a service call.
	#[pallet::storage]
	#[pallet::getter(fn next_job_call_id)]
	pub type NextJobCallId<T> = StorageValue<_, JobCallId, ValueQuery>;

	/// The next free ID for a unapplied slash.
	#[pallet::storage]
	#[pallet::getter(fn next_unapplied_slash_index)]
	pub type NextUnappliedSlashIndex<T> = StorageValue<_, u32, ValueQuery>;

	/// The service blueprints along with their owner.
	#[pallet::storage]
	#[pallet::getter(fn blueprints)]
	pub type Blueprints<T: Config> = StorageMap<
		_,
		Identity,
		u64,
		(T::AccountId, ServiceBlueprint<T::Constraints>),
		ResultQuery<Error<T>::BlueprintNotFound>,
	>;

	/// The operators for a specific service blueprint.
	/// Blueprint ID -> Operator -> Operator Preferences
	#[pallet::storage]
	#[pallet::getter(fn operators)]
	pub type Operators<T: Config> = StorageDoubleMap<
		_,
		Identity,
		u64,
		Identity,
		T::AccountId,
		OperatorPreferences,
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
		ServiceRequest<T::Constraints, T::AccountId, BlockNumberFor<T>, T::AssetId>,
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
		Service<T::Constraints, T::AccountId, BlockNumberFor<T>, T::AssetId>,
		ResultQuery<Error<T>::ServiceNotFound>,
	>;

	/// User Service Instances
	/// User Account ID -> Service ID
	#[pallet::storage]
	#[pallet::getter(fn user_services)]
	pub type UserServices<T: Config> = StorageMap<
		_,
		Identity,
		T::AccountId,
		BoundedBTreeSet<u64, MaxServicesPerUserOf<T>>,
		ValueQuery,
	>;

	/// The Service Job Calls
	/// Service ID -> Call ID -> Job Call
	#[pallet::storage]
	#[pallet::getter(fn job_calls)]
	pub type JobCalls<T: Config> = StorageDoubleMap<
		_,
		Identity,
		u64,
		Identity,
		u64,
		JobCall<T::Constraints, T::AccountId>,
		ResultQuery<Error<T>::ServiceOrJobCallNotFound>,
	>;

	/// The Service Job Call Results
	/// Service ID -> Call ID -> Job Call Result
	#[pallet::storage]
	#[pallet::getter(fn job_results)]
	pub type JobResults<T: Config> = StorageDoubleMap<
		_,
		Identity,
		u64,
		Identity,
		u64,
		JobCallResult<T::Constraints, T::AccountId>,
		ResultQuery<Error<T>::ServiceOrJobCallNotFound>,
	>;

	/// All unapplied slashes that are queued for later.
	///
	/// EraIndex -> Index -> UnappliedSlash
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn unapplied_slashes)]
	pub type UnappliedSlashes<T: Config> = StorageDoubleMap<
		_,
		Identity,
		u32,
		Identity,
		u32,
		UnappliedSlash<T::AccountId, BalanceOf<T>, T::AssetId>,
		ResultQuery<Error<T>::UnappliedSlashNotFound>,
	>;

	/// All the Master Blueprint Service Managers revisions.
	///
	/// Where the index is the revision number.
	#[pallet::storage]
	#[pallet::getter(fn mbsm_revisions)]
	pub type MasterBlueprintServiceManagerRevisions<T: Config> =
		StorageValue<_, BoundedVec<H160, T::MaxMasterBlueprintServiceManagerVersions>, ValueQuery>;

	// *** auxiliary storage and maps ***
	#[pallet::storage]
	#[pallet::getter(fn operator_profile)]
	pub type OperatorsProfile<T: Config> = StorageMap<
		_,
		Identity,
		T::AccountId,
		OperatorProfile<T::Constraints>,
		ResultQuery<Error<T>::OperatorProfileNotFound>,
	>;
	/// Holds the service payment information for a service request.
	/// Once the service is initiated, the payment is transferred to the MBSM and this
	/// information is removed.
	///
	/// Service Requst ID -> Service Payment
	#[pallet::storage]
	#[pallet::getter(fn service_payment)]
	pub type StagingServicePayments<T: Config> =
		StorageMap<_, Identity, u64, StagingServicePayment<T::AccountId, T::AssetId, BalanceOf<T>>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new service blueprint.
		///
		/// A Service Blueprint is a template for a service that can be instantiated by users. The blueprint
		/// defines the service's constraints, requirements and behavior, including the master blueprint service
		/// manager revision to use.
		///
		/// # Permissions
		///
		/// * The origin must be signed by the account that will own the blueprint
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the call, must be signed by the account creating the blueprint
		/// * `blueprint` - The service blueprint containing:
		///   - Service constraints and requirements
		///   - Master blueprint service manager revision (Latest or Specific)
		///   - Template configuration for service instantiation
		///
		/// # Errors
		///
		/// * [`Error::BadOrigin`] - Origin is not signed
		/// * [`Error::MasterBlueprintServiceManagerRevisionNotFound`] - Specified MBSM revision does not exist
		/// * [`Error::BlueprintCreationInterrupted`] - Blueprint creation is interrupted by hooks
		///
		/// # Returns
		///
		/// Returns a `DispatchResultWithPostInfo` which on success emits a [`Event::BlueprintCreated`] event
		/// containing the owner and blueprint ID.
		#[pallet::weight(T::WeightInfo::create_blueprint())]
		pub fn create_blueprint(
			origin: OriginFor<T>,
			mut blueprint: ServiceBlueprint<T::Constraints>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin)?;
			let blueprint_id = Self::next_blueprint_id();
			// Ensure the master blueprint service manager exists and if it uses
			// latest, pin it to the latest revision.
			match blueprint.master_manager_revision {
				MasterBlueprintServiceManagerRevision::Latest => {
					let latest_revision = Self::mbsm_latest_revision();
					blueprint.master_manager_revision =
						MasterBlueprintServiceManagerRevision::Specific(latest_revision);
				},
				MasterBlueprintServiceManagerRevision::Specific(rev) => {
					ensure!(
						rev <= Self::mbsm_latest_revision(),
						Error::<T>::MasterBlueprintServiceManagerRevisionNotFound,
					);
				},
				_ => unreachable!("MasterBlueprintServiceManagerRevision case is not implemented"),
			};

			let (allowed, _weight) =
				Self::on_blueprint_created_hook(&blueprint, blueprint_id, &owner)?;

			ensure!(allowed, Error::<T>::BlueprintCreationInterrupted);

			Blueprints::<T>::insert(blueprint_id, (owner.clone(), blueprint));
			NextBlueprintId::<T>::set(blueprint_id.saturating_add(1));

			Self::deposit_event(Event::BlueprintCreated { owner, blueprint_id });
			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Pre-register the caller as an operator for a specific blueprint.
		///
		/// This function allows an account to signal intent to become an operator for a blueprint by emitting
		/// a `PreRegistration` event. The operator node can listen for this event to execute any custom
		/// registration logic defined in the blueprint.
		///
		/// Pre-registration is the first step in the operator registration flow. After pre-registering,
		/// operators must complete the full registration process by calling `register()` with their preferences
		/// and registration arguments.
		///
		/// # Arguments
		///
		/// * `origin: OriginFor<T>` - The origin of the call. Must be signed by the account that wants to
		///   become an operator.
		/// * `blueprint_id: u64` - The identifier of the service blueprint to pre-register for. Must refer
		///   to an existing blueprint.
		///
		/// # Permissions
		///
		/// * The caller must be a signed account.
		///
		/// # Events
		///
		/// * [`Event::PreRegistration`] - Emitted when pre-registration is successful, containing:
		///   - `operator: T::AccountId` - The account ID of the pre-registering operator
		///   - `blueprint_id: u64` - The ID of the blueprint being pre-registered for
		///
		/// # Errors
		///
		/// * [`Error::BadOrigin`] - The origin was not signed.
		#[pallet::weight(T::WeightInfo::pre_register())]
		pub fn pre_register(
			origin: OriginFor<T>,
			#[pallet::compact] blueprint_id: u64,
		) -> DispatchResult {
			let operator_controller = ensure_signed(origin)?;

			// Emit the PreRegistration event
			Self::deposit_event(Event::PreRegistration {
				operator: operator_controller.clone(),
				blueprint_id,
			});

			Ok(())
		}

		/// Register the caller as an operator for a specific blueprint.
		///
		/// This function allows an account to register as an operator for a blueprint by providing their
		/// service preferences, registration arguments, and staking the required tokens. The operator must
		/// be active in the delegation system and may require approval before accepting service requests.
		///
		/// # Permissions
		///
		/// * The caller must be a signed account
		/// * The caller must be an active operator in the delegation system
		/// * The caller must not already be registered for this blueprint
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the call. Must be signed.
		/// * `blueprint_id` - The identifier of the service blueprint to register for
		/// * `preferences` - The operator's service preferences and configuration
		/// * `registration_args` - Registration arguments required by the blueprint
		/// * `value` - Amount of tokens to stake for registration
		///
		/// # Errors
		///
		/// * [`Error::OperatorNotActive`] - Caller is not an active operator in the delegation system
		/// * [`Error::AlreadyRegistered`] - Caller is already registered for this blueprint
		/// * [`Error::TypeCheck`] - Registration arguments failed type checking
		/// * [`Error::InvalidRegistrationInput`] - Registration hook rejected the registration
		/// * [`Error::MaxServicesPerProviderExceeded`] - Operator has reached maximum services limit
		#[pallet::weight(T::WeightInfo::register())]
		pub fn register(
			origin: OriginFor<T>,
			#[pallet::compact] blueprint_id: u64,
			preferences: OperatorPreferences,
			registration_args: Vec<Field<T::Constraints, T::AccountId>>,
			#[pallet::compact] value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			Self::do_register(&caller, blueprint_id, preferences, registration_args, value)?;
			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Unregisters a service provider from a specific service blueprint.
		///
		/// After unregistering, the provider will no longer receive new service assignments for this blueprint.
		/// However, they must continue servicing any active assignments until completion to avoid penalties.
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the call. Must be signed.
		/// * `blueprint_id` - The identifier of the service blueprint to unregister from.
		///
		/// # Permissions
		///
		/// * Must be signed by a registered service provider
		///
		/// # Errors
		///
		/// * [`Error::NotRegistered`] - The caller is not registered for this blueprint
		/// * [`Error::NotAllowedToUnregister`] - Unregistration is currently restricted
		/// * [`Error::BlueprintNotFound`] - The blueprint_id does not exist
		#[pallet::weight(T::WeightInfo::unregister())]
		pub fn unregister(
			origin: OriginFor<T>,
			#[pallet::compact] blueprint_id: u64,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			let (_, blueprint) = Self::blueprints(blueprint_id)?;
			let preferences = Operators::<T>::get(blueprint_id, &caller)?;
			let (allowed, _weight) =
				Self::on_unregister_hook(&blueprint, blueprint_id, &preferences)?;
			ensure!(allowed, Error::<T>::NotAllowedToUnregister);
			Operators::<T>::remove(blueprint_id, &caller);

			let removed = OperatorsProfile::<T>::try_mutate_exists(&caller, |profile| {
				profile
					.as_mut()
					.map(|p| p.blueprints.remove(&blueprint_id))
					.ok_or(Error::<T>::NotRegistered)
			})?;

			ensure!(removed, Error::<T>::NotRegistered);
			Self::deposit_event(Event::Unregistered { operator: caller.clone(), blueprint_id });
			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Updates the price targets for a registered operator's service blueprint.
		///
		/// Allows an operator to modify their price targets for a specific blueprint they are registered for.
		/// The operator must already be registered for the blueprint to update prices.
		///
		/// # Arguments
		///
		/// * `origin: OriginFor<T>` - The origin of the call. Must be signed by the operator.
		/// * `blueprint_id: u64` - The identifier of the blueprint to update price targets for.
		/// * `price_targets: PriceTargets` - The new price targets to set for the blueprint.
		///
		/// # Permissions
		///
		/// * Must be signed by a registered operator for this blueprint.
		///
		/// # Errors
		///
		/// * [`Error::NotRegistered`] - The caller is not registered for this blueprint.
		/// * [`Error::NotAllowedToUpdatePriceTargets`] - Price target updates are currently restricted.
		/// * [`Error::BlueprintNotFound`] - The blueprint_id does not exist.
		#[pallet::weight(T::WeightInfo::update_price_targets())]
		pub fn update_price_targets(
			origin: OriginFor<T>,
			#[pallet::compact] blueprint_id: u64,
			price_targets: PriceTargets,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			let (_, blueprint) = Self::blueprints(blueprint_id)?;

			let updated_preferences =
				Operators::<T>::try_mutate_exists(blueprint_id, &caller, |current_preferences| {
					current_preferences
						.as_mut()
						.map(|v| {
							v.price_targets = price_targets;
							*v
						})
						.ok_or(Error::<T>::NotRegistered)
				})?;

			let (allowed, _weight) =
				Self::on_update_price_targets(&blueprint, blueprint_id, &updated_preferences)?;

			ensure!(allowed, Error::<T>::NotAllowedToUpdatePriceTargets);

			Self::deposit_event(Event::PriceTargetsUpdated {
				operator: caller.clone(),
				blueprint_id,
				price_targets,
			});
			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Request a new service using a blueprint and specified operators.
		///
		/// # Arguments
		///
		/// * `origin: OriginFor<T>` - The origin of the call. Must be signed.
		/// * `evm_origin: Option<H160>` - Optional EVM address for ERC20 payments.
		/// * `blueprint_id: u64` - The identifier of the blueprint to use.
		/// * `permitted_callers: Vec<T::AccountId>` - Accounts allowed to call the service. If empty, only owner can call.
		/// * `operators: Vec<T::AccountId>` - List of operators that will run the service.
		/// * `request_args: Vec<Field<T::Constraints, T::AccountId>>` - Blueprint initialization arguments.
		/// * `assets: Vec<T::AssetId>` - Required assets for the service.
		/// * `ttl: BlockNumberFor<T>` - Time-to-live in blocks for the service request.
		/// * `payment_asset: Asset<T::AssetId>` - Asset used for payment (native, custom or ERC20).
		/// * `value: BalanceOf<T>` - Payment amount for the service.
		///
		/// # Permissions
		///
		/// * Must be signed by an account with sufficient balance to pay for the service.
		/// * For ERC20 payments, the EVM origin must match the caller's mapped account.
		///
		/// # Errors
		///
		/// * [`Error::TypeCheck`] - Request arguments fail blueprint type checking.
		/// * [`Error::NoAssetsProvided`] - No assets were specified.
		/// * [`Error::MissingEVMOrigin`] - EVM origin required but not provided for ERC20 payment.
		/// * [`Error::ERC20TransferFailed`] - ERC20 token transfer failed.
		/// * [`Error::NotRegistered`] - One or more operators not registered for blueprint.
		/// * [`Error::BlueprintNotFound`] - The blueprint_id does not exist.
		#[pallet::weight(T::WeightInfo::request())]
		pub fn request(
			origin: OriginFor<T>,
			evm_origin: Option<H160>,
			#[pallet::compact] blueprint_id: u64,
			permitted_callers: Vec<T::AccountId>,
			operators: Vec<T::AccountId>,
			request_args: Vec<Field<T::Constraints, T::AccountId>>,
			asset_security_requirements: Vec<AssetSecurityRequirement<T::AssetId>>,
			#[pallet::compact] ttl: BlockNumberFor<T>,
			payment_asset: Asset<T::AssetId>,
			#[pallet::compact] value: BalanceOf<T>,
			membership_model: MembershipModel,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;

			Self::do_request(
				caller,
				evm_origin,
				blueprint_id,
				permitted_callers,
				operators,
				request_args,
				asset_security_requirements,
				ttl,
				payment_asset,
				value,
				membership_model,
			)?;

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Approve a service request, allowing it to be initiated once all required approvals are received.
		///
		/// # Permissions
		///
		/// * Caller must be a registered operator for the service blueprint
		/// * Caller must be in the pending approvals list for this request
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the call, must be a signed account
		/// * `request_id` - The ID of the service request to approve
		/// * `native_exposure_percent` - Percentage of native token stake to expose
		/// * `asset_exposure` - Vector of asset-specific exposure commitments
		///
		/// # Errors
		///
		/// * [`Error::ApprovalNotRequested`] - Caller is not in the pending approvals list
		/// * [`Error::ApprovalInterrupted`] - Approval was rejected by blueprint hook
		/// * [`Error::InvalidRequestInput`] - Asset exposure commitments don't meet requirements
		#[pallet::weight(T::WeightInfo::approve())]
		pub fn approve(
			origin: OriginFor<T>,
			#[pallet::compact] request_id: u64,
			#[pallet::compact] native_asset_exposure: Percent,
			non_native_asset_exposures: Vec<AssetSecurityCommitment<T::AssetId>>,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			Self::do_approve(
				caller,
				request_id,
				native_asset_exposure,
				non_native_asset_exposures,
			)?;
			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Reject a service request, preventing its initiation.
		///
		/// The service request will remain in the system but marked as rejected. The requester will
		/// need to update the service request to proceed.
		///
		/// # Permissions
		///
		/// * Caller must be a registered operator for the blueprint associated with this request
		/// * Caller must be one of the operators required to approve this request
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the call, must be a signed account
		/// * `request_id` - The ID of the service request to reject
		///
		/// # Errors
		///
		/// * [`Error::ApprovalNotRequested`] - Caller is not one of the operators required to approve this request
		/// * [`Error::ExpectedAccountId`] - Failed to convert refund address to account ID when refunding payment
		/// * [`Error::RejectionInterrupted`] - Rejection was interrupted by blueprint hook
		#[pallet::weight(T::WeightInfo::reject())]
		pub fn reject(
			origin: OriginFor<T>,
			#[pallet::compact] request_id: u64,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			Self::do_reject(caller, request_id)?;
			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Terminates a running service instance.
		///
		/// # Permissions
		///
		/// * Must be signed by the service owner
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the call
		/// * `service_id` - The identifier of the service to terminate
		///
		/// # Errors
		///
		/// * [`Error::ServiceNotFound`] - The service_id does not exist
		/// * [`Error::NotRegistered`] - Service operator not registered
		/// * [`Error::TerminationInterrupted`] - Service termination was interrupted by hooks
		/// * [`DispatchError::BadOrigin`] - Caller is not the service owner
		#[pallet::weight(T::WeightInfo::terminate())]
		pub fn terminate(
			origin: OriginFor<T>,
			#[pallet::compact] service_id: u64,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			let service = Self::services(service_id)?;
			ensure!(service.owner == caller, DispatchError::BadOrigin);
			let removed = UserServices::<T>::try_mutate(&caller, |service_ids| {
				Result::<_, Error<T>>::Ok(service_ids.remove(&service_id))
			})?;
			ensure!(removed, Error::<T>::ServiceNotFound);
			Instances::<T>::remove(service_id);
			let blueprint_id = service.blueprint;
			let (_, blueprint) = Self::blueprints(blueprint_id)?;
			let (allowed, _weight) = Self::on_service_termination_hook(
				&blueprint,
				blueprint_id,
				service_id,
				&service.owner,
			)?;

			ensure!(allowed, Error::<T>::TerminationInterrupted);
			// Remove the service from the operator's profile.
			for (operator, _) in &service.native_asset_security {
				OperatorsProfile::<T>::try_mutate_exists(operator, |profile| {
					profile
						.as_mut()
						.map(|p| p.services.remove(&service_id))
						.ok_or(Error::<T>::NotRegistered)
				})?;
			}

			Self::deposit_event(Event::ServiceTerminated {
				owner: caller.clone(),
				service_id,
				blueprint_id,
			});
			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Call a job in the service with the provided arguments.
		///
		/// # Permissions
		///
		/// * Must be signed by the service owner or a permitted caller
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the call
		/// * `service_id` - The service identifier
		/// * `job` - The job index to call
		/// * `args` - The arguments to pass to the job
		///
		/// # Errors
		///
		/// * [`Error::ServiceNotFound`] - The service_id does not exist
		/// * [`Error::JobDefinitionNotFound`] - The job index is invalid
		/// * [`Error::MaxFieldsExceeded`] - Too many arguments provided
		/// * [`Error::TypeCheck`] - Arguments fail type checking
		/// * [`Error::InvalidJobCallInput`] - Job call was rejected by hooks
		/// * [`DispatchError::BadOrigin`] - Caller is not owner or permitted caller
		#[pallet::weight(T::WeightInfo::call())]
		pub fn call(
			origin: OriginFor<T>,
			#[pallet::compact] service_id: u64,
			#[pallet::compact] job: u8,
			args: Vec<Field<T::Constraints, T::AccountId>>,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			let service = Self::services(service_id)?;
			let blueprint_id = service.blueprint;
			let (_, blueprint) = Self::blueprints(blueprint_id)?;
			let is_permitted_caller = service.permitted_callers.iter().any(|v| v == &caller);
			ensure!(service.owner == caller || is_permitted_caller, DispatchError::BadOrigin);

			let job_def =
				blueprint.jobs.get(usize::from(job)).ok_or(Error::<T>::JobDefinitionNotFound)?;
			let bounded_args = BoundedVec::<_, MaxFieldsOf<T>>::try_from(args.clone())
				.map_err(|_| Error::<T>::MaxFieldsExceeded)?;
			let job_call = JobCall { service_id, job, args: bounded_args };

			job_call.type_check(job_def).map_err(Error::<T>::TypeCheck)?;
			let call_id = Self::next_job_call_id();

			let (allowed, _weight) =
				Self::on_job_call_hook(&blueprint, blueprint_id, service_id, job, call_id, &args)?;

			ensure!(allowed, Error::<T>::InvalidJobCallInput);

			JobCalls::<T>::insert(service_id, call_id, job_call);
			NextJobCallId::<T>::set(call_id.saturating_add(1));
			Self::deposit_event(Event::JobCalled {
				caller: caller.clone(),
				service_id,
				call_id,
				job,
				args,
			});

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Submit a result for a previously called job.
		///
		/// # Arguments
		///
		/// * `service_id` - ID of the service
		/// * `call_id` - ID of the job call
		/// * `result` - Vector of result fields
		///
		/// # Permissions
		///
		/// * Caller must be an operator of the service
		///
		/// # Errors
		///
		/// * [`Error::ServiceNotFound`] - The service_id does not exist
		/// * [`Error::JobCallNotFound`] - The call_id does not exist
		/// * [`Error::JobDefinitionNotFound`] - The job index is invalid
		/// * [`Error::MaxFieldsExceeded`] - Too many result fields provided
		/// * [`Error::TypeCheck`] - Result fields fail type checking
		/// * [`Error::InvalidJobResult`] - Job result was rejected by hooks
		/// * [`DispatchError::BadOrigin`] - Caller is not an operator
		#[pallet::weight(T::WeightInfo::submit_result())]
		pub fn submit_result(
			origin: OriginFor<T>,
			#[pallet::compact] service_id: u64,
			#[pallet::compact] call_id: u64,
			result: Vec<Field<T::Constraints, T::AccountId>>,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			let job_call = Self::job_calls(service_id, call_id)?;
			let service = Self::services(job_call.service_id)?;
			let blueprint_id = service.blueprint;
			let (_, blueprint) = Self::blueprints(blueprint_id)?;

			let is_operator = service.native_asset_security.iter().any(|(v, _)| v == &caller);
			ensure!(is_operator, DispatchError::BadOrigin);
			let operator_preferences = Operators::<T>::get(blueprint_id, &caller)?;

			let job_def = blueprint
				.jobs
				.get(usize::from(job_call.job))
				.ok_or(Error::<T>::JobDefinitionNotFound)?;

			let bounded_result = BoundedVec::<_, MaxFieldsOf<T>>::try_from(result.clone())
				.map_err(|_| Error::<T>::MaxFieldsExceeded)?;

			let job_result = JobCallResult { service_id, call_id, result: bounded_result };
			job_result.type_check(job_def).map_err(Error::<T>::TypeCheck)?;

			let (allowed, _weight) = Self::on_job_result_hook(
				&blueprint,
				blueprint_id,
				service_id,
				job_call.job,
				call_id,
				&operator_preferences,
				&job_call.args,
				&result,
			)?;

			ensure!(allowed, Error::<T>::InvalidJobResult);

			JobResults::<T>::insert(service_id, call_id, job_result);
			Self::deposit_event(Event::JobResultSubmitted {
				operator: caller.clone(),
				service_id,
				call_id,
				job: job_call.job,
				result,
			});

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Slash an operator's stake for a service by scheduling a deferred slashing action.
		///
		/// This function schedules a deferred slashing action against an operator's stake for a specific service.
		/// The slash is not applied immediately, but rather queued to be executed by another entity later.
		///
		/// # Permissions
		///
		/// * The caller must be an authorized Slash Origin for the target service, as determined by
		///   `query_slashing_origin`. If no slashing origin is set, or the caller does not match, the call
		///   will fail.
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the call. Must be signed by an authorized Slash Origin.
		/// * `offender` - The account ID of the operator to be slashed.
		/// * `service_id` - The ID of the service for which to slash the operator.
		/// * `percent` - The percentage of the operator's exposed stake to slash, as a `Percent` value.
		///
		/// # Errors
		///
		/// * `NoSlashingOrigin` - No slashing origin is set for the service
		/// * `BadOrigin` - Caller is not the authorized slashing origin
		/// * `OffenderNotOperator` - Target account is not an operator for this service
		/// * `OffenderNotActiveOperator` - Target operator is not currently active
		pub fn slash(
			origin: OriginFor<T>,
			offender: T::AccountId,
			#[pallet::compact] service_id: u64,
			#[pallet::compact] percent: Percent,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			let service = Self::services(service_id)?;
			let (maybe_slashing_origin, _used_weight) = Self::query_slashing_origin(&service)?;
			let slashing_origin = maybe_slashing_origin.ok_or(Error::<T>::NoSlashingOrigin)?;
			ensure!(slashing_origin == caller, DispatchError::BadOrigin);

			// Verify offender is an operator for this service
			ensure!(
				service.native_asset_security.iter().any(|(op, _)| op == &offender),
				Error::<T>::OffenderNotOperator
			);

			// Verify operator is active in delegation system
			ensure!(
				T::OperatorDelegationManager::is_operator_active(&offender),
				Error::<T>::OffenderNotActiveOperator
			);

			// Calculate the slash amounts for operator and delegators
			let unapplied_slash = Self::calculate_slash(&caller, &service, &offender, percent)?;

			// Store the slash for later processing
			let index = Self::next_unapplied_slash_index();
			UnappliedSlashes::<T>::insert(unapplied_slash.era, index, unapplied_slash.clone());
			NextUnappliedSlashIndex::<T>::set(index.saturating_add(1));

			Self::deposit_event(Event::<T>::UnappliedSlash {
				index,
				operator: offender,
				blueprint_id: service.blueprint,
				service_id,
				amount: unapplied_slash.own,
				era: unapplied_slash.era,
			});

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Disputes and removes an [UnappliedSlash] from storage.
		///
		/// The slash will not be applied once disputed and is permanently removed.
		///
		/// # Permissions
		///
		/// * Caller must be the authorized dispute origin for the service
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `era` - Era containing the slash to dispute  
		/// * `index` - Index of the slash within the era
		///
		/// # Errors
		///
		/// * [Error::NoDisputeOrigin] - Service has no dispute origin configured
		/// * [DispatchError::BadOrigin] - Caller is not the authorized dispute origin
		///

		pub fn dispute(
			origin: OriginFor<T>,
			#[pallet::compact] era: u32,
			#[pallet::compact] index: u32,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			let unapplied_slash = Self::unapplied_slashes(era, index)?;
			let service = Self::services(unapplied_slash.service_id)?;
			let (maybe_dispute_origin, _used_weight) = Self::query_dispute_origin(&service)?;
			let dispute_origin = maybe_dispute_origin.ok_or(Error::<T>::NoDisputeOrigin)?;
			ensure!(dispute_origin == caller, DispatchError::BadOrigin);

			UnappliedSlashes::<T>::remove(era, index);

			Self::deposit_event(Event::<T>::SlashDiscarded {
				index,
				operator: unapplied_slash.operator,
				blueprint_id: service.blueprint,
				service_id: unapplied_slash.service_id,
				amount: unapplied_slash.own,
				era,
			});

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Updates the Master Blueprint Service Manager by adding a new revision.
		///
		/// # Permissions
		///
		/// * Caller must be an authorized Master Blueprint Service Manager Update Origin
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `address` - New manager address to add
		///
		/// # Errors
		///
		/// * [Error::MaxMasterBlueprintServiceManagerVersionsExceeded] - Maximum number of revisions reached
		pub fn update_master_blueprint_service_manager(
			origin: OriginFor<T>,
			address: H160,
		) -> DispatchResultWithPostInfo {
			T::MasterBlueprintServiceManagerUpdateOrigin::ensure_origin(origin)?;

			MasterBlueprintServiceManagerRevisions::<T>::try_append(address)
				.map_err(|_| Error::<T>::MaxMasterBlueprintServiceManagerVersionsExceeded)?;

			let revision = Self::mbsm_latest_revision();
			Self::deposit_event(Event::<T>::MasterBlueprintServiceManagerRevised {
				revision,
				address,
			});

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Join a service instance as an operator
		#[pallet::call_index(15)]
		#[pallet::weight(10_000)]
		pub fn join_service(
			origin: OriginFor<T>,
			instance_id: u64,
			native_asset_exposure: Percent,
			non_native_asset_exposures: Vec<AssetSecurityCommitment<T::AssetId>>,
		) -> DispatchResult {
			let operator = ensure_signed(origin)?;

			// Get service instance
			let instance = Instances::<T>::get(instance_id)?;

			// Check if operator is already in the set
			ensure!(
				!instance.native_asset_security.iter().any(|(op, _)| op == &operator),
				Error::<T>::AlreadyJoined
			);

			let (_, blueprint) = Self::blueprints(instance.blueprint)?;
			let preferences = Self::operators(instance.blueprint, operator.clone())?;

			// Call membership implementation
			Self::do_join_service(
				&blueprint,
				instance.blueprint,
				instance_id,
				&operator,
				&preferences,
				native_asset_exposure,
				non_native_asset_exposures,
			)?;

			Ok(())
		}

		/// Leave a service instance as an operator
		#[pallet::call_index(16)]
		#[pallet::weight(10_000)]
		pub fn leave_service(origin: OriginFor<T>, instance_id: u64) -> DispatchResult {
			let operator = ensure_signed(origin)?;

			// Get service instance
			let instance = Instances::<T>::get(instance_id)?;

			// Get blueprint
			let (_, blueprint) = Self::blueprints(instance.blueprint)?;
			let _ = Self::operators(instance.blueprint, operator.clone())?;

			// Call membership implementation
			Self::do_leave_service(&blueprint, instance.blueprint, instance_id, &operator)?;

			Ok(())
		}
	}
}
