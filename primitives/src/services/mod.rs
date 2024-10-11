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

//! Services primitives.

use educe::Educe;
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{ecdsa, RuntimeDebug};

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

pub mod field;
pub use field::*;

/// A Higher level abstraction of all the constraints.
pub trait Constraints {
	/// Maximum number of fields in a job call.
	type MaxFields: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Maximum size of a field in a job call.
	type MaxFieldsSize: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Maximum length of metadata string length.
	type MaxMetadataLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Maximum number of jobs per service.
	type MaxJobsPerService: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Maximum number of Operators per service.
	type MaxOperatorsPerService: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Maximum number of permitted callers per service.
	type MaxPermittedCallers: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Maximum number of services per operator.
	type MaxServicesPerOperator: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Maximum number of blueprints per operator.
	type MaxBlueprintsPerOperator: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Maximum number of services per user.
	type MaxServicesPerUser: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Maximum number of binaries per gadget.
	type MaxBinariesPerGadget: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Maximum number of sources per gadget.
	type MaxSourcesPerGadget: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Git owner maximum length.
	type MaxGitOwnerLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Git repository maximum length.
	type MaxGitRepoLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Git tag maximum length.
	type MaxGitTagLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// binary name maximum length.
	type MaxBinaryNameLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// IPFS hash maximum length.
	type MaxIpfsHashLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Container registry maximum length.
	type MaxContainerRegistryLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Container image name maximum length.
	type MaxContainerImageNameLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Container image tag maximum length.
	type MaxContainerImageTagLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Maximum number of assets per service.
	type MaxAssetsPerService: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
}

/// A Job Definition is a definition of a job that can be called.
/// It contains the input and output fields of the job with the permitted caller.

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct JobDefinition<C: Constraints> {
	/// The metadata of the job.
	pub metadata: JobMetadata<C>,
	/// These are parameters that are required for this job.
	/// i.e. the input.
	pub params: BoundedVec<FieldType, C::MaxFields>,
	/// These are the result, the return values of this job.
	/// i.e. the output.
	pub result: BoundedVec<FieldType, C::MaxFields>,
	/// The verifier of the job result.
	pub verifier: JobResultVerifier,
}

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct JobMetadata<C: Constraints> {
	/// The Job name.
	pub name: BoundedString<C::MaxMetadataLength>,
	/// The Job description.
	pub description: Option<BoundedString<C::MaxMetadataLength>>,
}

/// A Job Call is a call to execute a job using it's job definition.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(
    Default(bound(AccountId: Default)),
    Clone(bound(AccountId: Clone)),
    PartialEq(bound(AccountId: PartialEq)),
    Eq
)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebugNoBound))]
#[cfg_attr(
	feature = "std",
	derive(Serialize, Deserialize),
	serde(bound(serialize = "AccountId: Serialize", deserialize = "AccountId: Deserialize<'de>")),
    educe(Debug(bound(AccountId: core::fmt::Debug)))
)]
pub struct JobCall<C: Constraints, AccountId> {
	/// The Service ID that this call is for.
	pub service_id: u64,
	/// The job definition index in the service that this call is for.
	pub job: u8,
	/// The supplied arguments for this job call.
	pub args: BoundedVec<Field<C, AccountId>, C::MaxFields>,
}

/// Type checks the supplied arguments against the parameters.
pub fn type_checker<C: Constraints, AccountId: Encode + Clone>(
	params: &[FieldType],
	args: &[Field<C, AccountId>],
) -> Result<(), TypeCheckError> {
	if params.len() != args.len() {
		return Err(TypeCheckError::NotEnoughArguments {
			expected: params.len() as u8,
			actual: args.len() as u8,
		});
	}
	for i in 0..args.len() {
		let arg = &args[i];
		let expected = &params[i];
		if arg != expected {
			return Err(TypeCheckError::ArgumentTypeMismatch {
				index: i as u8,
				expected: expected.clone(),
				actual: arg.clone().into(),
			});
		}
	}
	Ok(())
}

impl<C: Constraints, AccountId: Encode + Clone> JobCall<C, AccountId> {
	/// Check if the supplied arguments match the job definition types.
	pub fn type_check(&self, job_def: &JobDefinition<C>) -> Result<(), TypeCheckError> {
		type_checker(&job_def.params, &self.args)
	}
}

/// A Job Call Result is the result of a job call.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(
    Default(bound(AccountId: Default)),
    Clone(bound(AccountId: Clone)),
    PartialEq(bound(AccountId: PartialEq)),
    Eq
)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebugNoBound))]
#[cfg_attr(
	feature = "std",
	derive(Serialize, Deserialize),
	serde(bound(serialize = "AccountId: Serialize", deserialize = "AccountId: Deserialize<'de>")),
    educe(Debug(bound(AccountId: core::fmt::Debug)))
)]
pub struct JobCallResult<C: Constraints, AccountId> {
	/// The id of the service.
	pub service_id: u64,
	/// The id of the job call.
	pub call_id: u64,
	/// The result of the job call.
	pub result: BoundedVec<Field<C, AccountId>, C::MaxFields>,
}

impl<C: Constraints, AccountId: Encode + Clone> JobCallResult<C, AccountId> {
	/// Check if the supplied result match the job definition types.
	pub fn type_check(&self, job_def: &JobDefinition<C>) -> Result<(), TypeCheckError> {
		type_checker(&job_def.result, &self.result)
	}
}

/// A Job Result verifier is a verifier that will verify the result of a job call
/// using different verification methods.
#[derive(Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum JobResultVerifier {
	/// No verification is needed.
	#[default]
	None,
	/// An EVM Contract Address that will verify the result.
	Evm(sp_core::H160),
	// NOTE(@shekohex): Add more verification methods here.
}

/// An error that can occur during type checking.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TypeCheckError {
	/// The argument type does not match the expected type.
	ArgumentTypeMismatch {
		/// The index of the argument.
		index: u8,
		/// The expected type.
		expected: FieldType,
		/// The actual type.
		actual: FieldType,
	},
	/// Not enough arguments were supplied.
	NotEnoughArguments {
		/// The number of arguments that were expected.
		expected: u8,
		/// The number of arguments that were supplied.
		actual: u8,
	},
	/// The result type does not match the expected type.
	ResultTypeMismatch {
		/// The index of the argument.
		index: u8,
		/// The expected type.
		expected: FieldType,
		/// The actual type.
		actual: FieldType,
	},
}

impl frame_support::traits::PalletError for TypeCheckError {
	const MAX_ENCODED_SIZE: usize = 2;
}

// -*** Service ***-

/// Service Registration hook is a hook that will be called before registering the restaker as
/// an operator for the service.
#[derive(
	Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, Copy, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ServiceRegistrationHook {
	/// No hook is needed, the restaker will be registered immediately.
	#[default]
	None,
	/// A Smart contract that will be called to determine if the restaker will be registered.
	Evm(sp_core::H160),
}

/// Service Request hook is a hook that will be called before creating a service from the service
/// blueprint.
#[derive(
	Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, Copy, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ServiceRequestHook {
	/// No hook is needed, the caller will get the service created immediately.
	#[default]
	None,
	/// A Smart contract that will be called to determine if the caller meets the requirements to
	/// create a service.
	Evm(sp_core::H160),
}

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct ServiceMetadata<C: Constraints> {
	/// The Service name.
	pub name: BoundedString<C::MaxMetadataLength>,
	/// The Service description.
	pub description: Option<BoundedString<C::MaxMetadataLength>>,
	/// The Service author.
	/// Could be a company or a person.
	pub author: Option<BoundedString<C::MaxMetadataLength>>,
	/// The Job category.
	pub category: Option<BoundedString<C::MaxMetadataLength>>,
	/// Code Repository URL.
	/// Could be a github, gitlab, or any other code repository.
	pub code_repository: Option<BoundedString<C::MaxMetadataLength>>,
	/// Service Logo URL.
	pub logo: Option<BoundedString<C::MaxMetadataLength>>,
	/// Service Website URL.
	pub website: Option<BoundedString<C::MaxMetadataLength>>,
	/// Service License.
	pub license: Option<BoundedString<C::MaxMetadataLength>>,
}

/// A Service Blueprint is a the main definition of a service.
/// it contains the metadata of the service, the job definitions, and other hooks, along with the
/// gadget that will be executed when one of the jobs is calling this service.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct ServiceBlueprint<C: Constraints> {
	/// The metadata of the service.
	pub metadata: ServiceMetadata<C>,
	/// The job definitions that are available in this service.
	pub jobs: BoundedVec<JobDefinition<C>, C::MaxJobsPerService>,
	/// The registration hook that will be called before restaker registration.
	pub registration_hook: ServiceRegistrationHook,
	/// The parameters that are required for the service registration.
	pub registration_params: BoundedVec<FieldType, C::MaxFields>,
	/// The request hook that will be called before creating a service from the service blueprint.
	pub request_hook: ServiceRequestHook,
	/// The parameters that are required for the service request.
	pub request_params: BoundedVec<FieldType, C::MaxFields>,
	/// The gadget that will be executed for the service.
	pub gadget: Gadget<C>,
}

impl<C: Constraints> ServiceBlueprint<C> {
	/// Check if the supplied arguments match the registration parameters.
	pub fn type_check_registration<AccountId: Encode + Clone>(
		&self,
		args: &[Field<C, AccountId>],
	) -> Result<(), TypeCheckError> {
		type_checker(&self.registration_params, args)
	}

	/// Check if the supplied arguments match the request parameters.
	pub fn type_check_request<AccountId: Encode + Clone>(
		&self,
		args: &[Field<C, AccountId>],
	) -> Result<(), TypeCheckError> {
		type_checker(&self.request_params, args)
	}
}

/// A service request is a request to create a service from a service blueprint.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(
    Default(bound(AccountId: Default, BlockNumber: Default, AssetId: Default)),
    Clone(bound(AccountId: Clone, BlockNumber: Clone, AssetId: Clone)),
    PartialEq(bound(AccountId: PartialEq, BlockNumber: PartialEq, AssetId: PartialEq)),
    Eq
)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebugNoBound))]
#[cfg_attr(
	feature = "std",
	derive(Serialize, Deserialize),
	serde(bound(
        serialize = "AccountId: Serialize, BlockNumber: Serialize, AssetId: Serialize",
        deserialize = "AccountId: Deserialize<'de>, BlockNumber: Deserialize<'de>, AssetId: Deserialize<'de>"
    )),
    educe(Debug(bound(AccountId: core::fmt::Debug, BlockNumber: core::fmt::Debug, AssetId: core::fmt::Debug)))
)]
pub struct ServiceRequest<C: Constraints, AccountId, BlockNumber, AssetId> {
	/// The service blueprint ID.
	pub blueprint: u64,
	/// The owner of the service.
	pub owner: AccountId,
	/// The permitted caller(s) of the service.
	pub permitted_callers: BoundedVec<AccountId, C::MaxPermittedCallers>,
	/// Asset(s) used to secure the service instance.
	pub assets: BoundedVec<AssetId, C::MaxAssetsPerService>,
	/// The Lifetime of the service.
	pub ttl: BlockNumber,
	/// The supplied arguments for the service request.
	pub args: BoundedVec<Field<C, AccountId>, C::MaxFields>,
	/// The Selected Operator(s) with their approval state.
	pub operators_with_approval_state:
		BoundedVec<(AccountId, ApprovalState), C::MaxOperatorsPerService>,
}

impl<C: Constraints, AccountId, BlockNumber, AssetId>
	ServiceRequest<C, AccountId, BlockNumber, AssetId>
{
	/// Returns true if all the operators are [ApprovalState::Approved].
	pub fn is_approved(&self) -> bool {
		self.operators_with_approval_state
			.iter()
			.all(|(_, state)| state == &ApprovalState::Approved)
	}

	/// Returns true if any the operators are [ApprovalState::Pending].
	pub fn is_pending(&self) -> bool {
		self.operators_with_approval_state
			.iter()
			.any(|(_, state)| state == &ApprovalState::Pending)
	}

	/// Returns true if any the operators are [ApprovalState::Rejected].
	pub fn is_rejected(&self) -> bool {
		self.operators_with_approval_state
			.iter()
			.any(|(_, state)| state == &ApprovalState::Rejected)
	}
}

/// A Service is an instance of a service blueprint.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(
    Default(bound(AccountId: Default, BlockNumber: Default, AssetId: Default)),
    Clone(bound(AccountId: Clone, BlockNumber: Clone, AssetId: Clone)),
    PartialEq(bound(AccountId: PartialEq, BlockNumber: PartialEq, AssetId: PartialEq)),
    Eq
)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebugNoBound))]
#[cfg_attr(
	feature = "std",
	derive(Serialize, Deserialize),
	serde(bound(
        serialize = "AccountId: Serialize, BlockNumber: Serialize, AssetId: Serialize",
        deserialize = "AccountId: Deserialize<'de>, BlockNumber: Deserialize<'de>, AssetId: Deserialize<'de>",
    )),
    educe(Debug(bound(AccountId: core::fmt::Debug, BlockNumber: core::fmt::Debug, AssetId: core::fmt::Debug)))
)]
pub struct Service<C: Constraints, AccountId, BlockNumber, AssetId> {
	/// The service ID.
	pub id: u64,
	/// The Blueprint ID of the service.
	pub blueprint: u64,
	/// The owner of the service.
	pub owner: AccountId,
	/// The Permitted caller(s) of the service.
	pub permitted_callers: BoundedVec<AccountId, C::MaxPermittedCallers>,
	/// The Selected operators(s) for this service.
	pub operators: BoundedVec<AccountId, C::MaxOperatorsPerService>,
	/// Asset(s) used to secure the service instance.
	pub assets: BoundedVec<AssetId, C::MaxAssetsPerService>,
	/// The Lifetime of the service.
	pub ttl: BlockNumber,
}

/// Operator's Approval Prefrence.
#[derive(
	Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Copy, Clone, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ApprovalPreference {
	/// No approval is required to provide the service.
	#[codec(index = 0)]
	#[default]
	None,
	/// The approval is required to provide the service.
	#[codec(index = 1)]
	Required,
}

#[derive(
	Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Copy, Clone, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ApprovalState {
	/// The operator is pending approval.
	#[codec(index = 0)]
	#[default]
	Pending,
	/// The operator is approved to provide the service.
	#[codec(index = 1)]
	Approved,
	/// The operator is rejected to provide the service.
	#[codec(index = 2)]
	Rejected,
}

/// Represents the pricing structure for various hardware resources.
/// All prices are specified in USD/hr, calculated based on the average block time.
#[derive(
	PartialEq, Eq, Default, Encode, Decode, RuntimeDebug, TypeInfo, Copy, Clone, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PriceTargets {
	/// Price per vCPU per hour
	pub cpu: u64,
	/// Price per MB of memory per hour
	pub mem: u64,
	/// Price per GB of HDD storage per hour
	pub storage_hdd: u64,
	/// Price per GB of SSD storage per hour
	pub storage_ssd: u64,
	/// Price per GB of NVMe storage per hour
	pub storage_nvme: u64,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Copy, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OperatorPreferences {
	/// The operator ECDSA public key.
	pub key: ecdsa::Public,
	/// The approval prefrence of the operator.
	pub approval: ApprovalPreference,
	/// The pricing targets for the operator's resources.
	pub price_targets: PriceTargets,
}

impl OperatorPreferences {
	/// Encode the fields to ethabi bytes.
	pub fn to_ethabi(&self) -> Vec<ethabi::Token> {
		let tokens: Vec<ethabi::Token> = vec![
			ethabi::Token::Bytes(self.key.0.to_vec()),
			// TODO: Add ApprovalPreference to ethabi.
		];
		tokens
	}
}

/// Operator Profile is a profile of an operator that
/// contains metadata about the services that the operator is providing.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct OperatorProfile<C: Constraints> {
	/// The Service IDs that I'm currently providing.
	pub services: BoundedBTreeSet<u64, C::MaxServicesPerOperator>,
	/// The Blueprint IDs that I'm currently registered for.
	pub blueprints: BoundedBTreeSet<u64, C::MaxBlueprintsPerOperator>,
}

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub enum Gadget<C: Constraints> {
	/// A Gadget that is a WASM binary that will be executed.
	/// inside the shell using the wasm runtime.
	Wasm(WasmGadget<C>),
	/// A Gadget that is a native binary that will be executed.
	/// inside the shell using the OS.
	Native(NativeGadget<C>),
	/// A Gadget that is a container that will be executed.
	/// inside the shell using the container runtime (e.g. Docker, Podman, etc.)
	Container(ContainerGadget<C>),
}

impl<C: Constraints> Default for Gadget<C> {
	fn default() -> Self {
		Gadget::Wasm(WasmGadget { runtime: WasmRuntime::Wasmtime, sources: Default::default() })
	}
}

/// A binary that is stored in the Github release.
/// this will constuct the URL to the release and download the binary.
/// The URL will be in the following format:
/// https://github.com/<owner>/<repo>/releases/download/v<tag>/<path>
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct GithubFetcher<C: Constraints> {
	/// The owner of the repository.
	pub owner: BoundedString<C::MaxGitOwnerLength>,
	/// The repository name.
	pub repo: BoundedString<C::MaxGitRepoLength>,
	/// The release tag of the repository.
	/// NOTE: The tag should be a valid semver tag.
	pub tag: BoundedString<C::MaxGitTagLength>,
	/// The names of the binary in the release by the arch and the os.
	pub binaries: BoundedVec<GadgetBinary<C>, C::MaxBinariesPerGadget>,
}

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct TestFetcher<C: Constraints> {
	/// The cargo package name that contains the blueprint logic
	pub cargo_package: BoundedString<C::MaxBinaryNameLength>,
	/// The specific binary name that contains the blueprint logic.
	/// Should match up what is in the Cargo.toml file under [[bin]]/name
	pub cargo_bin: BoundedString<C::MaxBinaryNameLength>,
	/// The base path to the workspace/crate
	pub base_path: BoundedString<C::MaxMetadataLength>,
}

/// The CPU or System architecture.
#[derive(
	PartialEq,
	PartialOrd,
	Ord,
	Eq,
	Encode,
	Decode,
	RuntimeDebug,
	TypeInfo,
	Clone,
	Copy,
	MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Architecture {
	/// WebAssembly architecture (32-bit).
	#[codec(index = 0)]
	Wasm,
	/// WebAssembly architecture (64-bit).
	#[codec(index = 1)]
	Wasm64,
	/// WASI architecture (32-bit).
	#[codec(index = 2)]
	Wasi,
	/// WASI architecture (64-bit).
	#[codec(index = 3)]
	Wasi64,
	/// Amd architecture (32-bit).
	#[codec(index = 4)]
	Amd,
	/// Amd64 architecture (x86_64).
	#[codec(index = 5)]
	Amd64,
	/// Arm architecture (32-bit).
	#[codec(index = 6)]
	Arm,
	/// Arm64 architecture (64-bit).
	#[codec(index = 7)]
	Arm64,
	/// Risc-V architecture (32-bit).
	#[codec(index = 8)]
	RiscV,
	/// Risc-V architecture (64-bit).
	#[codec(index = 9)]
	RiscV64,
}

/// Operating System that the binary is compiled for.
#[derive(
	Default,
	PartialEq,
	PartialOrd,
	Ord,
	Eq,
	Encode,
	Decode,
	RuntimeDebug,
	TypeInfo,
	Clone,
	Copy,
	MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum OperatingSystem {
	/// Unknown operating system.
	/// This is used when the operating system is not known
	/// for example, for WASM, where the OS is not relevant.
	#[default]
	#[codec(index = 0)]
	Unknown,
	/// Linux operating system.
	#[codec(index = 1)]
	Linux,
	/// Windows operating system.
	#[codec(index = 2)]
	Windows,
	/// MacOS operating system.
	#[codec(index = 3)]
	MacOS,
	/// BSD operating system.
	#[codec(index = 4)]
	BSD,
}

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct GadgetBinary<C: Constraints> {
	/// CPU or System architecture.
	pub arch: Architecture,
	/// Operating System that the binary is compiled for.
	pub os: OperatingSystem,
	/// The name of the binary.
	pub name: BoundedString<C::MaxBinaryNameLength>,
	/// The sha256 hash of the binary.
	/// used to verify the downloaded binary.
	pub sha256: [u8; 32],
}

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct GadgetSource<C: Constraints> {
	/// The fetcher that will fetch the gadget from a remote source.
	fetcher: GadgetSourceFetcher<C>,
}

/// A Gadget Source Fetcher is a fetcher that will fetch the gadget
/// from a remote source.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub enum GadgetSourceFetcher<C: Constraints> {
	/// A Gadget that will be fetched from the IPFS.
	#[codec(index = 0)]
	IPFS(BoundedVec<u8, C::MaxIpfsHashLength>),
	/// A Gadget that will be fetched from the Github release.
	#[codec(index = 1)]
	Github(GithubFetcher<C>),
	/// A Gadgets that will be fetched from the container registry.
	#[codec(index = 2)]
	ContainerImage(ImageRegistryFetcher<C>),
	/// For tests only
	#[codec(index = 3)]
	Testing(TestFetcher<C>),
}

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct ImageRegistryFetcher<C: Constraints> {
	/// The URL of the container registry.
	registry: BoundedString<C::MaxContainerRegistryLength>,
	/// The name of the image.
	image: BoundedString<C::MaxContainerImageNameLength>,
	/// The tag of the image.
	tag: BoundedString<C::MaxContainerImageTagLength>,
}

/// A WASM binary that contains all the compiled gadget code.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct WasmGadget<C: Constraints> {
	/// Which runtime to use to execute the WASM binary.
	pub runtime: WasmRuntime,
	/// Where the WASM binary is stored.
	pub sources: BoundedVec<GadgetSource<C>, C::MaxSourcesPerGadget>,
}

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub enum WasmRuntime {
	/// The WASM binary will be executed using the WASMtime runtime.
	#[codec(index = 0)]
	Wasmtime,
	/// The WASM binary will be executed using the Wasmer runtime.
	#[codec(index = 1)]
	Wasmer,
}

/// A Native binary that contains all the gadget code.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct NativeGadget<C: Constraints> {
	/// Where the WASM binary is stored.
	pub sources: BoundedVec<GadgetSource<C>, C::MaxSourcesPerGadget>,
}

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct ContainerGadget<C: Constraints> {
	/// Where the Image of the gadget binary is stored.
	pub sources: BoundedVec<GadgetSource<C>, C::MaxSourcesPerGadget>,
}

// -***- RPC -***-

/// RPC Response for query the blueprint along with the services instances of that blueprint.
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(
    Default(bound(AccountId: Default, BlockNumber: Default, AssetId: Default)),
    Clone(bound(AccountId: Clone, BlockNumber: Clone, AssetId: Clone)),
    PartialEq(bound(AccountId: PartialEq, BlockNumber: PartialEq, AssetId: PartialEq)),
    Eq
)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebugNoBound))]
#[cfg_attr(
	feature = "std",
	derive(Serialize, Deserialize),
	serde(bound(
        serialize = "AccountId: Serialize, BlockNumber: Serialize, AssetId: Serialize",
        deserialize = "AccountId: Deserialize<'de>, BlockNumber: Deserialize<'de>, AssetId: Deserialize<'de>",
    )),
    educe(Debug(bound(AccountId: core::fmt::Debug, BlockNumber: core::fmt::Debug, AssetId: core::fmt::Debug)))
)]
pub struct RpcServicesWithBlueprint<C: Constraints, AccountId, BlockNumber, AssetId> {
	/// The blueprint ID.
	pub blueprint_id: u64,
	/// The service blueprint.
	pub blueprint: ServiceBlueprint<C>,
	/// The services instances of that blueprint.
	pub services: Vec<Service<C, AccountId, BlockNumber, AssetId>>,
}
