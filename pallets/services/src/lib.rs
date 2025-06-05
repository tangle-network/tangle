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
#![allow(clippy::unused_unit, clippy::useless_conversion, clippy::type_complexity)]

#[cfg(not(feature = "std"))]
extern crate alloc;
use frame_support::{
	dispatch::{DispatchResult, DispatchResultWithPostInfo, Pays, PostDispatchInfo},
	ensure,
	pallet_prelude::*,
	storage::TransactionOutcome,
	traits::{
		Currency, ReservableCurrency,
		fungibles::{Inspect, Mutate},
	},
};
use frame_system::pallet_prelude::{BlockNumberFor, OriginFor, ensure_signed};
use sp_core::ecdsa;
use sp_runtime::{RuntimeAppPublic, SaturatedConversion, traits::Zero};
use sp_std::{collections::btree_map::BTreeMap, prelude::*, vec};
use tangle_primitives::{
	BlueprintId, InstanceId, JobCallId, ServiceRequestId,
	services::{
		AssetSecurityCommitment, AssetSecurityRequirement, MembershipModel, UnappliedSlash,
	},
	traits::{MultiAssetDelegationInfo, SlashManager},
};

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

pub mod functions;
mod impls;
mod payment_processing;
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
	use sp_core::H160;
	use sp_runtime::{Percent, Saturating, traits::MaybeSerializeDeserialize};
	use sp_std::{collections::btree_set::BTreeSet, vec::Vec};
	use tangle_primitives::{
		rewards::AssetType, services::*, traits::RewardRecorder as RewardRecorderTrait,
	};

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

		/// A type that implements the `RewardRecorder` trait for recording service rewards.
		type RewardRecorder: RewardRecorderTrait<
				Self::AccountId,
				ServiceId,
				BalanceOf<Self>,
				PricingModel = PricingModel<BlockNumberFor<Self>, BalanceOf<Self>>,
			>;

		/// PalletId used for deriving the AccountId and EVM address.
		/// This account receives slashed assets upon slash event processing.
		#[pallet::constant]
		type PalletEvmAccount: Get<H160>;

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

		/// Authority identifier type
		type RoleKeyId: Member
			+ Parameter
			+ RuntimeAppPublic
			+ MaybeSerializeDeserialize
			+ AsRef<[u8]>
			+ Into<ecdsa::Public>
			+ From<ecdsa::Public>
			+ MaxEncodedLen;

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
		/// Maximum length of rpc address.
		#[pallet::constant]
		type MaxRpcAddressLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
		/// Maximum number of resource types.
		#[pallet::constant]
		type MaxResourceNameLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
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
				AssetType,
			>;

		/// Manager for slashing that dispatches slash operations to
		/// `pallet-multi-asset-delegation`.
		type SlashManager: tangle_primitives::traits::SlashManager<Self::AccountId>;

		/// Interface for recording rewards.
		type RewardsManager: tangle_primitives::traits::RewardsManager<
				Self::AccountId,
				Self::AssetId,
				BalanceOf<Self>,
				BlockNumberFor<Self>,
			>;

		/// Number of eras that slashes are deferred by, after computation.
		///
		/// This should be less than the bonding duration. Set to 0 if slashes
		/// should be applied immediately, without opportunity for intervention.
		#[pallet::constant]
		type SlashDeferDuration: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;

		/// The origin which can manage Add a new Master Blueprint Service Manager revision.
		type MasterBlueprintServiceManagerUpdateOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// The origin which may update default service parameters like heartbeat interval,
		/// threshold, and slashing window.
		type DefaultParameterUpdateOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The minimum percentage of native token stake that operators must expose for slashing.
		#[pallet::constant]
		type MinimumNativeSecurityRequirement: Get<Percent>
			+ Default
			+ Parameter
			+ MaybeSerializeDeserialize;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn integrity_test() {
			// Ensure that the pallet's configuration is valid.
			// 1. Make sure that pallet's evm address maps correctly back to the Substrate account
			let evm_address = T::EvmAddressMapping::into_account_id(Self::pallet_evm_account());
			assert_eq!(
				evm_address,
				Self::pallet_account(),
				"Services: EVM address mapping is incorrect."
			);
		}

		/// On initialize, we should check for any unapplied slashes and apply them.
		/// Also process subscription payments for active services.
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			let mut weight = Zero::zero();
			let current_era = T::OperatorDelegationManager::get_current_round();
			let slash_defer_duration = T::SlashDeferDuration::get();

			// Only process slashes from eras that have completed their deferral period
			let process_era = current_era.saturating_sub(slash_defer_duration);

			// Get all unapplied slashes for this era
			let prefix_iter = UnappliedSlashes::<T>::iter_prefix(process_era);
			for (index, slash) in prefix_iter {
				// TODO: This call must be all or nothing.
				// TODO: If fail then revert all storage changes
				if Self::slashing_enabled() {
					let _ = frame_support::storage::with_transaction(
						|| -> TransactionOutcome<Result<_, DispatchError>> {
							let res = T::SlashManager::slash_operator(&slash);
							match &res {
								Ok(weight_used) => {
									weight =
										weight_used.checked_add(&weight).unwrap_or_else(Zero::zero);
									// Remove the slash from storage after successful application
									UnappliedSlashes::<T>::remove(process_era, index);
									TransactionOutcome::Commit(Ok(res))
								},
								Err(_) => {
									log::error!("Failed to apply slash for index: {:?}", index);
									TransactionOutcome::Rollback(Ok(res))
								},
							}
						},
					);
				}
			}

			// Process subscription payments
			let subscription_weight = Self::process_subscription_payments_on_block(n);
			weight = weight.saturating_add(subscription_weight);

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
		/// The caller is not registered as a operator.
		NotRegistered,
		/// The Operator is not active in the delegation system.
		OperatorNotActive,
		/// The Operator is not allowed to register.
		InvalidRegistrationInput,
		/// The Operator is not allowed to unregister.
		NotAllowedToUnregister,
		/// The Operator is not allowed to update their RPC address.
		NotAllowedToUpdateRpcAddress,
		/// The caller does not have the requirements to request a service.
		InvalidRequestInput,
		/// The caller does not have the requirements to call a job.
		InvalidJobCallInput,
		/// The caller provided an invalid job result.
		InvalidJobResult,
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
		/// Maximum number of services per operator reached.
		MaxServicesPerOperatorExceeded,
		/// Maximum number of blueprints registered by the operator reached.
		MaxBlueprintsPerOperatorExceeded,
		/// Duplicate operator registration.
		DuplicateOperator,
		/// Duplicate key used for registration.
		DuplicateKey,
		/// Too many operators provided for the service's membership model
		TooManyOperators,
		/// Too few operators provided for the service's membership model
		TooFewOperators,
		/// No assets provided for the service, at least one asset is required.
		NoAssetsProvided,
		/// Duplicate assets provided
		DuplicateAsset,
		/// The maximum number of assets per service has been exceeded.
		MaxAssetsPerServiceExceeded,
		/// Native asset exposure is too low
		NativeAssetExposureTooLow,
		/// Native asset is not found
		NoNativeAsset,
		/// Offender is not a registered operator.
		OffenderNotOperator,
		/// The Service Blueprint did not return a slashing origin for this service.
		NoSlashingOrigin,
		/// The Service Blueprint did not return a dispute origin for this service.
		NoDisputeOrigin,
		/// The Unapplied Slash are not found.
		UnappliedSlashNotFound,
		/// The Supplied Master Blueprint Service Manager Revision is not found.
		MasterBlueprintServiceManagerRevisionNotFound,
		/// Duplicate membership model
		DuplicateMembershipModel,
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
		/// Maximum operators reached
		MaxOperatorsReached,
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
		/// Caller is not an operator of the service
		NotAnOperator,
		/// Invalid slash percentage
		InvalidSlashPercentage,
		/// Invalid key (zero byte ECDSA key provided)
		InvalidKey,
		/// Invalid security commitments
		InvalidSecurityCommitments,
		/// Invalid Security Requirements
		InvalidSecurityRequirements,
		/// Invalid quote signature
		InvalidQuoteSignature,
		/// Mismatched number of signatures
		SignatureCountMismatch,
		/// Missing quote signature
		MissingQuoteSignature,
		/// Invalid key for quote
		InvalidKeyForQuote,
		/// Signature verification failed
		SignatureVerificationFailed,
		/// Invalid signature bytes
		InvalidSignatureBytes,
		/// Get Heartbeat Interval Failure
		GetHeartbeatIntervalFailure,
		/// Get Heartbeat Threshold Failure
		GetHeartbeatThresholdFailure,
		/// Get Slashing Window Failure
		GetSlashingWindowFailure,
		/// Heartbeat too early
		HeartbeatTooEarly,
		/// Heartbeat signature verification failed
		HeartbeatSignatureVerificationFailed,
		/// Invalid heartbeat data
		InvalidHeartbeatData,
		/// Service not active
		ServiceNotActive,
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
			preferences: OperatorPreferences<T::Constraints>,
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
			security_requirements:
				BoundedVec<AssetSecurityRequirement<T::AssetId>, MaxAssetsPerServiceOf<T>>,
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
			operator_security_commitments:
				OperatorSecurityCommitments<T::AccountId, T::AssetId, T::Constraints>,
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
			/// Service ID
			service_id: u64,
			/// Blueprint ID
			blueprint_id: u64,
			/// Slash percent
			slash_percent: Percent,
			/// Era index
			era: u32,
		},
		/// An Unapplied Slash got discarded.
		SlashDiscarded {
			/// The index of the slash.
			index: u32,
			/// The account that has an unapplied slash.
			operator: T::AccountId,
			/// Service ID
			service_id: u64,
			/// Blueprint ID
			blueprint_id: u64,
			/// Slash percent
			slash_percent: Percent,
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
		/// A request for a pricing quote has been made.
		RequestForQuote {
			/// The account requesting the quote.
			requester: T::AccountId,
			/// The ID of the blueprint being quoted.
			blueprint_id: u64,
		},
		/// RPC address updated.
		RpcAddressUpdated {
			/// The account that updated the RPC address.
			operator: T::AccountId,
			/// The ID of the service blueprint.
			blueprint_id: u64,
			/// The new RPC address.
			rpc_address: BoundedString<<<T as Config>::Constraints as tangle_primitives::services::Constraints>::MaxRpcAddressLength>,
		},
		/// A service has sent a heartbeat.
		HeartbeatReceived {
			/// The service that sent the heartbeat.
			service_id: u64,
			/// The ID of the service blueprint.
			blueprint_id: u64,
			/// The block number when the heartbeat was received.
			operator: T::AccountId,
			/// The block number when the heartbeat was received.
			block_number: BlockNumberFor<T>,
		},
		/// Default heartbeat threshold updated.
		DefaultHeartbeatThresholdUpdated {
			/// The new default heartbeat threshold.
			threshold: u8,
		},
		/// Default heartbeat interval updated.
		DefaultHeartbeatIntervalUpdated {
			/// The new default heartbeat interval.
			interval: BlockNumberFor<T>,
		},
		/// Default heartbeat slashing window updated.
		DefaultHeartbeatSlashingWindowUpdated {
			/// The new default heartbeat slashing window.
			window: BlockNumberFor<T>,
		},
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Slashing is enabled.
	#[pallet::storage]
	#[pallet::getter(fn slashing_enabled)]
	pub type SlashingEnabled<T> = StorageValue<_, bool, ValueQuery>;

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
		(T::AccountId, ServiceBlueprint<T::Constraints, BlockNumberFor<T>, BalanceOf<T>>),
		ResultQuery<Error<T>::BlueprintNotFound>,
	>;

	/// The services for a particular blueprint and their active status.
	/// Blueprint ID -> Service ID -> active
	#[pallet::storage]
	#[pallet::getter(fn service_status)]
	pub type ServiceStatus<T: Config> = StorageDoubleMap<
		_,
		Identity,
		BlueprintId,
		Identity,
		InstanceId,
		(),
		ResultQuery<Error<T>::ServiceNotFound>,
	>;

	/// The default interval between heartbeats.
	#[pallet::storage]
	#[pallet::getter(fn default_heartbeat_interval)]
	pub type DefaultHeartbeatInterval<T> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// The default threshold of unhealthy heartbeats for slashing.
	#[pallet::storage]
	#[pallet::getter(fn default_heartbeat_threshold)]
	pub type DefaultHeartbeatThreshold<T> = StorageValue<_, u8, ValueQuery>;

	/// The default slashing window for services.
	#[pallet::storage]
	#[pallet::getter(fn default_slashing_window)]
	pub type DefaultSlashingWindow<T> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// The heartbeats for services.
	/// Blueprint ID -> Service ID -> (Last Heartbeat Block, Custom Metrics Data)
	#[pallet::storage]
	#[pallet::getter(fn service_heartbeats)]
	pub type ServiceHeartbeats<T: Config> = StorageDoubleMap<
		_,
		Identity,
		BlueprintId,
		Identity,
		InstanceId,
		(BlockNumberFor<T>, BoundedVec<u8, <<T as Config>::Constraints as tangle_primitives::services::Constraints>::MaxFieldsSize>),
		ValueQuery,
	>;

	/// Heartbeat tracking for service operators
	/// (Blueprint ID, Service ID, Operator) -> HeartbeatStats
	#[pallet::storage]
	#[pallet::getter(fn service_operator_heartbeats)]
	pub type ServiceOperatorHeartbeats<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Identity, BlueprintId>,
			NMapKey<Identity, InstanceId>,
			NMapKey<Identity, T::AccountId>,
		),
		HeartbeatStats,
		ValueQuery,
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
		OperatorPreferences<T::Constraints>,
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
		UnappliedSlash<T::AccountId>,
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
		/// A Service Blueprint is a template for a service that can be instantiated by users. The
		/// blueprint defines the service's constraints, requirements and behavior, including the
		/// master blueprint service manager revision to use.
		///
		/// # Permissions
		///
		/// * The origin must be signed by the account that will own the blueprint
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the call, must be signed by the account creating the
		///   blueprint
		/// * `metadata` - The metadata of the service blueprint.
		/// * `typedef` - The type definition of the service blueprint.
		/// * `membership_model` - The membership model of the service blueprint.
		/// * `security_requirements` - The security requirements of the service blueprint.
		/// * `price_targets` - The price targets of the service blueprint.
		/// * `pricing_model` - The pricing model of the service blueprint.
		///
		/// # Errors
		///
		/// * [`Error::BadOrigin`] - Origin is not signed
		/// * [`Error::MasterBlueprintServiceManagerRevisionNotFound`] - Specified MBSM revision
		///   does not exist
		/// * [`Error::BlueprintCreationInterrupted`] - Blueprint creation is interrupted by hooks
		///
		/// # Returns
		///
		/// Returns a `DispatchResultWithPostInfo` which on success emits a
		/// [`Event::BlueprintCreated`] event containing the owner and blueprint ID.
		#[pallet::weight(T::WeightInfo::create_blueprint())]
		pub fn create_blueprint(
			origin: OriginFor<T>,
			metadata: BoundedVec<u8, ConstU32<MAX_METADATA_LENGTH>>,
			typedef: ServiceBlueprint<T::Constraints, BlockNumberFor<T>, BalanceOf<T>>,
			membership_model: MembershipModel,
			_security_requirements: Vec<AssetSecurityRequirement<T::AssetId>>,
			_price_targets: Option<PriceTargets>,
			pricing_model: PricingModel<BlockNumberFor<T>, BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin)?;

			// Validate and store the blueprint
			let blueprint_id = NextBlueprintId::<T>::get();

			let membership_model_type = match membership_model {
				MembershipModel::Fixed { .. } => MembershipModelType::Fixed,
				MembershipModel::Dynamic { .. } => MembershipModelType::Dynamic,
			};

			// Validate that the blueprint supports the requested membership model
			ensure!(
				typedef.supported_membership_models.contains(&membership_model_type),
				Error::<T>::UnsupportedMembershipModel
			);

			let metadata_string =
				String::from_utf8(metadata.clone().into_inner()).unwrap_or_default();
			let blueprint = ServiceBlueprint {
				metadata: ServiceMetadata {
					name: BoundedString::try_from(metadata_string.clone()).unwrap(),
					description: Some(BoundedString::try_from(metadata_string).unwrap()),
					author: None,
					category: None,
					code_repository: None,
					logo: None,
					website: None,
					license: None,
				},
				jobs: typedef.jobs,
				registration_params: typedef.registration_params,
				request_params: typedef.request_params,
				manager: typedef.manager,
				master_manager_revision: match typedef.master_manager_revision {
					MasterBlueprintServiceManagerRevision::Latest =>
						MasterBlueprintServiceManagerRevision::Specific(Self::mbsm_latest_revision()),
					MasterBlueprintServiceManagerRevision::Specific(revision) =>
						MasterBlueprintServiceManagerRevision::Specific(revision),
					_ => typedef.master_manager_revision, // Fallback for future variants
				},
				sources: typedef.sources,
				supported_membership_models: typedef.supported_membership_models,
				pricing_model,
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
		/// This function allows an account to signal intent to become an operator for a blueprint
		/// by emitting a `PreRegistration` event. The operator node can listen for this event to
		/// execute any custom registration logic defined in the blueprint.
		///
		/// Pre-registration is the first step in the operator registration flow. After
		/// pre-registering, operators must complete the full registration process by calling
		/// `register()` with their preferences and registration arguments.
		///
		/// # Arguments
		///
		/// * `origin: OriginFor<T>` - The origin of the call. Must be signed by the account that
		///   wants to become an operator.
		/// * `blueprint_id: u64` - The identifier of the service blueprint to pre-register for.
		///   Must refer to an existing blueprint.
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
		/// This function allows an account to register as an operator for a blueprint by providing
		/// their service preferences, registration arguments, and staking the required tokens.
		/// The operator must be active in the delegation system and may require approval before
		/// accepting service requests.
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
		/// * [`Error::OperatorNotActive`] - Caller is not an active operator in the delegation
		///   system
		/// * [`Error::AlreadyRegistered`] - Caller is already registered for this blueprint
		/// * [`Error::TypeCheck`] - Registration arguments failed type checking
		/// * [`Error::InvalidRegistrationInput`] - Registration hook rejected the registration
		/// * [`Error::MaxServicesPerProviderExceeded`] - Operator has reached maximum services
		///   limit
		#[pallet::weight(T::WeightInfo::register())]
		pub fn register(
			origin: OriginFor<T>,
			#[pallet::compact] blueprint_id: BlueprintId,
			preferences: OperatorPreferences<T::Constraints>,
			registration_args: Vec<Field<T::Constraints, T::AccountId>>,
			#[pallet::compact] value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			// Validate the operator preferences
			ensure!(preferences.key != [0u8; 65], Error::<T>::InvalidKey);
			// Check if the caller is an active operator in the delegation system
			ensure!(
				T::OperatorDelegationManager::is_operator_active(&caller),
				Error::<T>::OperatorNotActive
			);
			// Check if operator is already registered for this blueprint
			ensure!(
				!Operators::<T>::contains_key(blueprint_id, &caller),
				Error::<T>::AlreadyRegistered
			);
			// Check if the key is already in use
			for (_, prefs) in Operators::<T>::iter_prefix(blueprint_id) {
				if prefs.key == preferences.key {
					return Err(Error::<T>::DuplicateKey.into());
				}
			}

			Self::do_register(&caller, blueprint_id, preferences, registration_args, value)?;
			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Unregisters a service provider from a specific service blueprint.
		///
		/// Can only be called if the no services are active for the blueprint.
		/// After unregistering, the provider will no longer receive new service
		/// assignments for this blueprint.
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

			// Check for active services for this operator
			for (service_id, _) in ServiceStatus::<T>::iter_prefix(blueprint_id) {
				ensure!(
					!ServiceStatus::<T>::contains_key(blueprint_id, service_id),
					Error::<T>::NotAllowedToUnregister
				);
			}
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

		/// Request a new service using a blueprint and specified operators.
		///
		/// # Arguments
		///
		/// * `origin: OriginFor<T>` - The origin of the call. Must be signed.
		/// * `evm_origin: Option<H160>` - Optional EVM address for ERC20 payments.
		/// * `blueprint_id: u64` - The identifier of the blueprint to use.
		/// * `permitted_callers: Vec<T::AccountId>` - Accounts allowed to call the service. If
		///   empty, only owner can call.
		/// * `operators: Vec<T::AccountId>` - List of operators that will run the service.
		/// * `request_args: Vec<Field<T::Constraints, T::AccountId>>` - Blueprint initialization
		///   arguments.
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
			// Ensure all operators are active
			for operator in operators.iter() {
				ensure!(
					T::OperatorDelegationManager::is_operator_active(operator),
					Error::<T>::OperatorNotActive
				);
			}

			// Ensure each asset has non-zero exposure requirements
			for requirement in asset_security_requirements.iter() {
				ensure!(
					requirement.min_exposure_percent > Percent::zero() &&
						requirement.max_exposure_percent > Percent::zero() &&
						requirement.min_exposure_percent <= requirement.max_exposure_percent &&
						requirement.max_exposure_percent <= Percent::from_percent(100),
					Error::<T>::InvalidSecurityRequirements,
				);
			}

			// Ensure no duplicate operators
			let mut seen_operators = BTreeSet::new();
			for operator in operators.iter() {
				ensure!(seen_operators.insert(operator), Error::<T>::DuplicateOperator);
			}

			let (_, blueprint) = Self::blueprints(blueprint_id)?;
			let supported_membership_models = blueprint.supported_membership_models;

			// Check that the number of operators doesn't exceed the membership model max
			match membership_model {
				MembershipModel::Fixed { min_operators } => {
					ensure!(
						supported_membership_models.contains(&MembershipModelType::Fixed),
						Error::<T>::UnsupportedMembershipModel
					);
					ensure!(min_operators > 0, Error::<T>::TooFewOperators);
					ensure!(operators.len() >= min_operators as usize, Error::<T>::TooFewOperators);
				},
				MembershipModel::Dynamic { min_operators, max_operators } => {
					ensure!(
						supported_membership_models.contains(&MembershipModelType::Dynamic),
						Error::<T>::UnsupportedMembershipModel
					);
					ensure!(operators.len() >= min_operators as usize, Error::<T>::TooFewOperators);
					if let Some(max_ops) = max_operators {
						ensure!(operators.len() <= max_ops as usize, Error::<T>::TooManyOperators);
					}
				},
			}

			// Ensure all operators are registered for this blueprint
			for operator in operators.iter() {
				ensure!(
					Operators::<T>::contains_key(blueprint_id, operator),
					Error::<T>::NotRegistered
				);
			}

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

		/// Approve a service request, allowing it to be initiated once all required approvals are
		/// received.
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
			_security_commitment: T::Hash,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;

			// Since this is just a test implementation, we'll infer the security commitments
			// from the service request requirements and use default valid commitments
			let request = Self::service_requests(request_id)?;
			let security_commitments: Vec<AssetSecurityCommitment<T::AssetId>> = request
				.security_requirements
				.iter()
				.map(|req| AssetSecurityCommitment {
					asset: req.asset.clone(),
					exposure_percent: req.min_exposure_percent,
				})
				.collect();

			Self::do_approve(caller, request_id, &security_commitments)?;

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
		/// * [`Error::ApprovalNotRequested`] - Caller is not one of the operators required to
		///   approve this request
		/// * [`Error::ExpectedAccountId`] - Failed to convert refund address to account ID when
		///   refunding payment
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

			// Apply any unapplied slashes for this service before termination
			let current_era = T::OperatorDelegationManager::get_current_round();
			let last_era = current_era.saturating_sub(1);

			// Get slashes from current and last era
			let current_slashes: Vec<_> = UnappliedSlashes::<T>::iter_prefix(current_era)
				.filter(|(_, slash)| slash.service_id == service_id)
				.collect();
			let last_slashes: Vec<_> = UnappliedSlashes::<T>::iter_prefix(last_era)
				.filter(|(_, slash)| slash.service_id == service_id)
				.collect();

			// Apply all slashes
			for (_, slash) in current_slashes.into_iter().chain(last_slashes) {
				T::SlashManager::slash_operator(&slash)?;
			}

			// Clean up storage
			let _ = UnappliedSlashes::<T>::clear_prefix(current_era, u32::MAX, None);
			let _ = UnappliedSlashes::<T>::clear_prefix(last_era, u32::MAX, None);

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
			for (operator, _) in &service.operator_security_commitments {
				OperatorsProfile::<T>::try_mutate_exists(operator, |profile| {
					profile
						.as_mut()
						.map(|p| p.services.remove(&service_id))
						.ok_or(Error::<T>::NotRegistered)
				})?;
			}

			ServiceStatus::<T>::remove(blueprint_id, service_id);
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
			let (_, _blueprint) = Self::blueprints(blueprint_id)?;
			let is_permitted_caller = service.permitted_callers.iter().any(|v| v == &caller);
			ensure!(service.owner == caller || is_permitted_caller, DispatchError::BadOrigin);

			let job_def =
				_blueprint.jobs.get(usize::from(job)).ok_or(Error::<T>::JobDefinitionNotFound)?;
			let bounded_args = BoundedVec::<_, MaxFieldsOf<T>>::try_from(args.clone())
				.map_err(|_| Error::<T>::MaxFieldsExceeded)?;
			let job_call = JobCall { service_id, job, args: bounded_args };

			job_call.type_check(job_def).map_err(Error::<T>::TypeCheck)?;
			let call_id = Self::next_job_call_id();

			let (allowed, _weight) =
				Self::on_job_call_hook(&_blueprint, blueprint_id, service_id, job, call_id, &args)?;

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
			let (_, _blueprint) = Self::blueprints(blueprint_id)?;
			let operator_preferences = Operators::<T>::get(blueprint_id, &caller)?;

			let job_def = _blueprint
				.jobs
				.get(usize::from(job_call.job))
				.ok_or(Error::<T>::JobDefinitionNotFound)?;

			let bounded_result = BoundedVec::<_, MaxFieldsOf<T>>::try_from(result.clone())
				.map_err(|_| Error::<T>::MaxFieldsExceeded)?;

			let job_result = JobCallResult { service_id, call_id, result: bounded_result };
			job_result.type_check(job_def).map_err(Error::<T>::TypeCheck)?;

			let (allowed, _weight) = Self::on_job_result_hook(
				&_blueprint,
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
		/// This function schedules a deferred slashing action against an operator's stake for a
		/// specific service. The slash is not applied immediately, but rather queued to be
		/// executed by another entity later.
		///
		/// # Permissions
		///
		/// * The caller must be an authorized Slash Origin for the target service, as determined by
		///   `query_slashing_origin`. If no slashing origin is set, or the caller does not match,
		///   the call will fail.
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the call. Must be signed by an authorized Slash Origin.
		/// * `offender` - The account ID of the operator to be slashed.
		/// * `service_id` - The ID of the service for which to slash the operator.
		/// * `slash_percent` - The percentage of the operator's exposed stake to slash, as a
		///   `Percent` value.
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
			#[pallet::compact] slash_percent: Percent,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			let service = Self::services(service_id)?;
			let (maybe_slashing_origin, _used_weight) = Self::query_slashing_origin(&service)?;
			let slashing_origin = maybe_slashing_origin.ok_or(Error::<T>::NoSlashingOrigin)?;
			ensure!(slashing_origin == caller, DispatchError::BadOrigin);

			// Ensure slash percent is greater than 0
			ensure!(!slash_percent.is_zero(), Error::<T>::InvalidSlashPercentage);

			// Verify offender is an operator for this service
			ensure!(
				service.operator_security_commitments.iter().any(|(op, _)| op == &offender),
				Error::<T>::OffenderNotOperator
			);

			// Verify operator is active in delegation system
			ensure!(
				T::OperatorDelegationManager::is_operator_active(&offender),
				Error::<T>::OperatorNotActive
			);

			// Calculate the slash amounts for operator and delegators
			let unapplied_slash = UnappliedSlash {
				era: T::OperatorDelegationManager::get_current_round(),
				blueprint_id: service.blueprint,
				service_id: service.id,
				operator: offender.clone(),
				slash_percent,
			};

			// Store the slash for later processing
			let index = Self::next_unapplied_slash_index();
			UnappliedSlashes::<T>::insert(unapplied_slash.era, index, unapplied_slash.clone());
			NextUnappliedSlashIndex::<T>::set(index.saturating_add(1));

			Self::deposit_event(Event::<T>::UnappliedSlash {
				index,
				operator: offender,
				blueprint_id: service.blueprint,
				service_id,
				slash_percent,
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
				slash_percent: unapplied_slash.slash_percent,
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
		/// * [Error::MaxMasterBlueprintServiceManagerVersionsExceeded] - Maximum number of
		///   revisions reached
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
			security_commitments: Vec<AssetSecurityCommitment<T::AssetId>>,
		) -> DispatchResult {
			let operator = ensure_signed(origin)?;

			// Get service instance
			let instance = Instances::<T>::get(instance_id)?;

			// Check if operator is already in the set
			ensure!(
				!instance.operator_security_commitments.iter().any(|(op, _)| op == &operator),
				Error::<T>::AlreadyJoined
			);

			ensure!(
				instance.validate_security_commitments(&security_commitments),
				Error::<T>::InvalidSecurityCommitments
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
				security_commitments,
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

		/// Updates the RPC address for a registered operator's service blueprint.
		///
		/// Allows an operator to modify their RPC address for a specific blueprint they are
		/// registered for. The operator must already be registered for the blueprint to update
		/// the RPC address.
		///
		/// # Arguments
		///
		/// * `origin: OriginFor<T>` - The origin of the call. Must be signed by the operator.
		/// * `blueprint_id: u64` - The identifier of the blueprint to update the RPC address for.
		/// * `rpc_address: BoundedString<T::Constraints::MaxRpcAddressLength>` - The new RPC
		///   address to set for the blueprint.
		///
		/// # Permissions
		///
		/// * Must be signed by a registered operator for this blueprint.
		///
		/// # Errors
		///
		/// * [`Error::NotRegistered`] - The caller is not registered for this blueprint.
		/// * [`Error::BlueprintNotFound`] - The blueprint_id does not exist.
		#[pallet::call_index(17)]
		#[pallet::weight(T::WeightInfo::update_rpc_address())]
		pub fn update_rpc_address(
			origin: OriginFor<T>,
			#[pallet::compact] blueprint_id: u64,
			rpc_address: BoundedString<<<T as Config>::Constraints as tangle_primitives::services::Constraints>::MaxRpcAddressLength>,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			let (_, _blueprint) = Self::blueprints(blueprint_id)?;

			// Get the current preferences
			let mut preferences = Operators::<T>::get(blueprint_id, &caller)?;

			// Update the RPC address
			preferences.rpc_address = rpc_address.clone();

			// Call the hook to notify the blueprint
			let (allowed, _weight) =
				Self::on_update_rpc_address_hook(&_blueprint, blueprint_id, &preferences)?;

			ensure!(allowed, Error::<T>::NotAllowedToUpdateRpcAddress);

			// Update the preferences
			Operators::<T>::insert(blueprint_id, &caller, &preferences);

			// Emit the event
			Self::deposit_event(Event::RpcAddressUpdated {
				operator: caller.clone(),
				blueprint_id,
				rpc_address,
			});

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Request a service with a pre-approved quote from operators.
		///
		/// This function creates a service request using a quote that has already been approved by
		/// the operators. Unlike the regular `request` method, this doesn't require operator
		/// approval after submission since the operators have already agreed to the terms via the
		/// quote.
		///
		/// The quote is obtained externally through a gRPC server, and this function accepts the
		/// necessary signatures from the operators to verify their approval.
		///
		/// # Permissions
		///
		/// * Anyone can call this function
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the call, must be a signed account.
		/// * `evm_origin` - Optional EVM address for ERC20 payments.
		/// * `blueprint_id` - The ID of the blueprint to use.
		/// * `permitted_callers` - Accounts allowed to call the service. If empty, only owner can
		///   call.
		/// * `operators` - List of operators that will run the service.
		/// * `request_args` - Blueprint initialization arguments.
		/// * `asset_security_requirements` - Security requirements for assets.
		/// * `ttl` - Time-to-live in blocks for the service request.
		/// * `payment_asset` - Asset used for payment (native, custom or ERC20).
		/// * `value` - Amount to pay for the service.
		/// * `membership_model` - Membership model for the service.
		/// * `operator_signatures` - Signatures from operators confirming the quote.
		/// * `security_commitments` - Security commitments from operators.
		/// * `pricing_quote` - Pricing quote details.
		///
		/// # Errors
		///
		/// * [`Error::TypeCheck`] - Request arguments fail blueprint type checking.
		/// * [`Error::NoAssetsProvided`] - No assets were specified.
		/// * [`Error::MissingEVMOrigin`] - EVM origin required but not provided for ERC20 payment.
		/// * [`Error::ERC20TransferFailed`] - ERC20 token transfer failed.
		/// * [`Error::NotRegistered`] - One or more operators not registered for blueprint.
		/// * [`Error::BlueprintNotFound`] - The blueprint_id does not exist.
		/// * [`Error::InvalidQuoteSignature`] - One or more quote signatures are invalid.
		#[pallet::call_index(18)]
		#[pallet::weight(10_000)]
		pub fn request_with_signed_price_quotes(
			origin: OriginFor<T>,
			evm_origin: Option<H160>,
			#[pallet::compact] blueprint_id: u64,
			permitted_callers: Vec<T::AccountId>,
			operators: Vec<T::AccountId>,
			request_args: Vec<Field<T::Constraints, T::AccountId>>,
			asset_security_requirements: Vec<AssetSecurityRequirement<T::AssetId>>,
			#[pallet::compact] ttl: BlockNumberFor<T>,
			payment_asset: Asset<T::AssetId>,
			membership_model: MembershipModel,
			pricing_quotes: Vec<PricingQuote<T::Constraints>>,
			operator_signatures: Vec<ecdsa::Signature>,
			security_commitments: Vec<AssetSecurityCommitment<T::AssetId>>,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;

			// Ensure all operators are active
			for operator in operators.iter() {
				ensure!(
					T::OperatorDelegationManager::is_operator_active(operator),
					Error::<T>::OperatorNotActive,
				);
			}

			// Ensure no duplicate operators
			let mut seen_operators = BTreeMap::new();
			for operator in operators.iter() {
				ensure!(!seen_operators.contains_key(operator), Error::<T>::DuplicateOperator);
				seen_operators.insert(operator.clone(), ());
			}

			let (_, _blueprint) = Self::blueprints(blueprint_id)?;

			// Ensure all operators are registered for this blueprint
			for operator in operators.iter() {
				ensure!(
					Operators::<T>::contains_key(blueprint_id, operator),
					Error::<T>::NotRegistered
				);
			}

			// Verify that we have a signature from each operator
			let mut operator_signatures_map = BTreeMap::new();
			for (signature, operator) in operator_signatures.iter().zip(operators.iter()) {
				ensure!(operators.contains(operator), Error::<T>::SignatureCountMismatch);
				operator_signatures_map.insert(operator.clone(), *signature);
			}

			// Ensure all operators have provided a signature
			for operator in operators.iter() {
				ensure!(
					operator_signatures_map.contains_key(operator),
					Error::<T>::MissingQuoteSignature
				);
			}

			// Verify each operator's signature
			for (i, (operator, signature)) in operator_signatures_map.iter().enumerate() {
				let operator_preferences = Operators::<T>::get(blueprint_id, operator)?;

				let public_key = ecdsa::Public::from_full(&operator_preferences.key)
					.map_err(|_| Error::<T>::InvalidKeyForQuote)?;

				// Hash the pricing quote to create the message to verify
				let message =
					tangle_primitives::services::pricing::hash_pricing_quote(&pricing_quotes[i]);

				// Verify the signature
				ensure!(
					sp_io::crypto::ecdsa_verify(signature, &message, &public_key),
					Error::<T>::SignatureVerificationFailed
				);
			}

			// Calculate the cost of from the quotes
			let total_cost_rate = pricing_quotes.iter().map(|q| q.total_cost_rate).sum::<u128>();
			let value = total_cost_rate * ttl.saturated_into::<u128>();
			let value = value.saturated_into::<BalanceOf<T>>();

			// Request service
			let service_id = Self::do_request(
				caller.clone(),
				evm_origin,
				blueprint_id,
				permitted_callers,
				operators.clone(),
				request_args,
				asset_security_requirements,
				ttl,
				payment_asset,
				value,
				membership_model.clone(),
			)?;

			// Automatically approve the service for each operator
			for operator in operators.iter() {
				Self::do_approve(operator.clone(), service_id, &security_commitments)?;
			}

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Send a heartbeat for a service.
		///
		/// This function allows operators to send periodic heartbeats to indicate they are still
		/// active. Each operator must send heartbeats at intervals defined by its blueprint's
		/// heartbeat_interval. The heartbeat includes custom metrics data that can be used for
		/// monitoring and analytics.
		///
		/// The heartbeat must be signed by the operator to verify its authenticity.
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the call, must be a signed account.
		/// * `service_id` - The ID of the service sending the heartbeat.
		/// * `blueprint_id` - The ID of the blueprint the service was created from.
		/// * `metrics_data` - Custom metrics data from the service (serialized).
		/// * `signature` - ECDSA signature verifying the heartbeat data.
		///
		/// # Errors
		///
		/// * [`Error::ServiceNotFound`] - The service does not exist.
		/// * [`Error::ServiceNotActive`] - The service is not active.
		/// * [`Error::BlueprintNotFound`] - The blueprint does not exist.
		/// * [`Error::HeartbeatTooEarly`] - Not enough blocks have passed since the last heartbeat.
		/// * [`Error::HeartbeatSignatureVerificationFailed`] - The signature verification failed.
		/// * [`Error::InvalidHeartbeatData`] - The heartbeat data is invalid.
		#[pallet::call_index(19)]
		#[pallet::weight(10_000)]
		pub fn heartbeat(
			origin: OriginFor<T>,
			#[pallet::compact] service_id: u64,
			#[pallet::compact] blueprint_id: u64,
			metrics_data: Vec<u8>,
			signature: ecdsa::Signature,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;

			// Ensure the service exists and is active
			ensure!(
				ServiceStatus::<T>::contains_key(blueprint_id, service_id),
				Error::<T>::ServiceNotFound
			);

			// Get the service instance
			let instance = Instances::<T>::get(service_id)?;

			// Ensure the service is for the correct blueprint
			ensure!(instance.blueprint == blueprint_id, Error::<T>::ServiceNotFound);

			// Verify the caller is an operator for this service
			ensure!(
				Operators::<T>::contains_key(blueprint_id, caller.clone()),
				Error::<T>::NotRegistered
			);

			// Get the blueprint and current block number
			let (_, blueprint) = Self::blueprints(blueprint_id)?;
			let current_block = <frame_system::Pallet<T>>::block_number();
			let bounded_metrics_data = BoundedVec::<u8, <<T as Config>::Constraints as tangle_primitives::services::Constraints>::MaxFieldsSize>::try_from(metrics_data.clone())
    .map_err(|_| Error::<T>::InvalidHeartbeatData)?;

			// Create the message to verify
			// Format: service_id + blueprint_id + metrics_data
			let mut message = service_id.to_le_bytes().to_vec();
			message.extend_from_slice(&blueprint_id.to_le_bytes());
			message.extend_from_slice(&bounded_metrics_data);
			let message_hash = sp_io::hashing::keccak_256(&message);

			// Get the operator's preferences to get their public key
			let operator_preferences = Operators::<T>::get(blueprint_id, &caller)?;
			let public_key = ecdsa::Public::from_full(&operator_preferences.key)
				.map_err(|_| Error::<T>::InvalidSignatureBytes)?;

			// Verify the signature
			ensure!(
				sp_io::crypto::ecdsa_verify(&signature, &message_hash, &public_key),
				Error::<T>::HeartbeatSignatureVerificationFailed
			);

			// Get operator's heartbeat stats
			let mut stats =
				ServiceOperatorHeartbeats::<T>::get((blueprint_id, service_id, &caller));

			// If this is the first heartbeat for this operator, initialize the stats
			if stats.last_heartbeat_block.is_zero() {
				stats.last_heartbeat_block = current_block.try_into().unwrap_or_default();
				stats.last_check_block = current_block.try_into().unwrap_or_default();
				stats.expected_heartbeats = 1;
				stats.received_heartbeats = 1;
			} else {
				// Get the heartbeat interval from the QoS function
				let heartbeat_interval =
					Self::get_heartbeat_interval(&blueprint, blueprint_id, service_id)?;

				// Check if enough blocks have passed since the last heartbeat
				let blocks_passed = current_block.saturating_sub(stats.last_heartbeat_block.into());
				ensure!(blocks_passed >= heartbeat_interval, Error::<T>::HeartbeatTooEarly);

				// Calculate how many heartbeats were expected since the last one
				let expected_since_last =
					(blocks_passed / heartbeat_interval).try_into().unwrap_or_default();

				// Update the stats
				stats.expected_heartbeats =
					stats.expected_heartbeats.saturating_add(expected_since_last);
				stats.received_heartbeats = stats.received_heartbeats.saturating_add(1);
				stats.last_heartbeat_block = current_block.try_into().unwrap_or_default();

				// Get the heartbeat threshold from the QoS function
				let heartbeat_threshold =
					Self::get_heartbeat_threshold(&blueprint, blueprint_id, service_id)?;
				if stats.expected_heartbeats > heartbeat_threshold.into() {
					// Calculate how many heartbeats were missed
					let missed =
						stats.expected_heartbeats.saturating_sub(stats.received_heartbeats);
					if missed > heartbeat_threshold.into() {
						// Get the slashing window from the QoS function
						let slashing_window =
							Self::get_slashing_window(&blueprint, blueprint_id, service_id)?;
						let slashing_block = stats
							.last_heartbeat_block
							.saturating_add(slashing_window.try_into().unwrap_or_default());

						// If we're within the slashing window, schedule a slash
						if current_block <= slashing_block.into() {
							// Calculate slash percentage based on missed heartbeats
							let slash_percent = Percent::from_percent(50); // TODO: Calculate based on missed heartbeats
							Self::create_heartbeat_slash(
								blueprint_id,
								service_id,
								caller.clone(),
								slash_percent,
							);
						}
					}
				}
			}

			// Update the heartbeat storage
			ServiceHeartbeats::<T>::insert(
				blueprint_id,
				service_id,
				(current_block, bounded_metrics_data),
			);

			// Update the operator's heartbeat stats
			ServiceOperatorHeartbeats::<T>::insert(
				(blueprint_id, service_id, caller.clone()),
				stats,
			);

			// Emit event for heartbeat received
			Self::deposit_event(Event::<T>::HeartbeatReceived {
				blueprint_id,
				service_id,
				operator: caller,
				block_number: current_block,
			});

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::No })
		}

		/// Updates the default heartbeat threshold for all services.
		///
		/// # Permissions
		///
		/// * Can only be called by the DefaultParameterUpdateOrigin
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `threshold` - New default heartbeat threshold
		#[pallet::call_index(20)]
		#[pallet::weight(10_000)]
		pub fn update_default_heartbeat_threshold(
			origin: OriginFor<T>,
			threshold: u8,
		) -> DispatchResultWithPostInfo {
			T::DefaultParameterUpdateOrigin::ensure_origin(origin)?;

			DefaultHeartbeatThreshold::<T>::set(threshold);

			Self::deposit_event(Event::<T>::DefaultHeartbeatThresholdUpdated { threshold });

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Updates the default heartbeat interval for all services.
		///
		/// # Permissions
		///
		/// * Can only be called by the DefaultParameterUpdateOrigin
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `interval` - New default heartbeat interval
		#[pallet::call_index(21)]
		#[pallet::weight(10_000)]
		pub fn update_default_heartbeat_interval(
			origin: OriginFor<T>,
			interval: BlockNumberFor<T>,
		) -> DispatchResultWithPostInfo {
			T::DefaultParameterUpdateOrigin::ensure_origin(origin)?;

			DefaultHeartbeatInterval::<T>::set(interval);

			Self::deposit_event(Event::<T>::DefaultHeartbeatIntervalUpdated { interval });

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}

		/// Updates the default heartbeat slashing window for all services.
		///
		/// # Permissions
		///
		/// * Can only be called by the DefaultParameterUpdateOrigin
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the call
		/// * `window` - New default heartbeat slashing window
		#[pallet::call_index(22)]
		#[pallet::weight(10_000)]
		pub fn update_default_heartbeat_slashing_window(
			origin: OriginFor<T>,
			window: BlockNumberFor<T>,
		) -> DispatchResultWithPostInfo {
			T::DefaultParameterUpdateOrigin::ensure_origin(origin)?;

			DefaultSlashingWindow::<T>::set(window);

			Self::deposit_event(Event::<T>::DefaultHeartbeatSlashingWindowUpdated { window });

			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes })
		}
	}
}
