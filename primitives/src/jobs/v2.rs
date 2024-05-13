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

//! Jobs v2 module.

use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{ecdsa, RuntimeDebug};

mod field;
pub use field::*;

/// Maximum number of fields in a job call.
pub type MaxFields = ConstU32<64>;
/// Maximum size of a field in a job call.
pub type MaxFieldsSize = ConstU32<1024>;
/// Maximum length of metadata string length.
pub type MaxMetadataLength = ConstU32<1024>;
/// Maximum number of jobs per service.
pub type MaxJobsPerService = ConstU32<32>;
/// Maximum number of Operators per service.
pub type MaxOperatorsPerService = ConstU32<512>;
/// Maximum number of permitted callers per service.
pub type MaxPermittedCallers = ConstU32<32>;
/// Maximum number of services per operator.
pub type MaxServicesPerOperator = ConstU32<32>;
/// Maximum number of blueprints per operator.
pub type MaxBlueprintsPerOperator = ConstU32<32>;

/// A Job Definition is a definition of a job that can be called.
/// It contains the input and output fields of the job with the permitted caller.
#[derive(Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct JobDefinition {
	/// The metadata of the job.
	pub metadata: JobMetadata,
	/// These are parameters that are required for this job.
	/// i.e. the input.
	pub params: BoundedVec<FieldType, MaxFields>,
	/// These are the result, the return values of this job.
	/// i.e. the output.
	pub result: BoundedVec<FieldType, MaxFields>,
	/// The verifier of the job result.
	pub verifier: JobResultVerifier,
}

#[derive(Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct JobMetadata {
	/// The Job name.
	pub name: BoundedString<MaxMetadataLength>,
	/// The Job description.
	pub description: Option<BoundedString<MaxMetadataLength>>,
}

/// A Job Call is a call to execute a job using it's job definition.
#[derive(Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct JobCall<AccountId> {
	/// The Service ID that this call is for.
	pub service_id: u64,
	/// The job definition index in the service that this call is for.
	pub job: u8,
	/// The supplied arguments for this job call.
	pub args: BoundedVec<Field<AccountId>, MaxFields>,
}

/// Type checks the supplied arguments against the parameters.
pub fn type_checker<AccountId: Clone>(
	params: &[FieldType],
	args: &[Field<AccountId>],
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

impl<AccountId: Clone> JobCall<AccountId> {
	/// Check if the supplied arguments match the job definition types.
	pub fn type_check(&self, job_def: &JobDefinition) -> Result<(), TypeCheckError> {
		type_checker(&job_def.params, &self.args)
	}
}

/// A Job Call Result is the result of a job call.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct JobCallResult<AccountId> {
	/// The id of the service.
	pub service_id: u64,
	/// The id of the job call.
	pub call_id: u64,
	/// The result of the job call.
	pub result: BoundedVec<Field<AccountId>, MaxFields>,
}

impl<AccountId: Clone> JobCallResult<AccountId> {
	/// Check if the supplied result match the job definition types.
	pub fn type_check(&self, job_def: &JobDefinition) -> Result<(), TypeCheckError> {
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

/// Service Request hook is a hook that will be called before creating a service from the service blueprint.
#[derive(
	Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, Copy, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ServiceRequestHook {
	/// No hook is needed, the caller will get the service created immediately.
	#[default]
	None,
	/// A Smart contract that will be called to determine if the caller meets the requirements to create a service.
	Evm(sp_core::H160),
}

#[derive(Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ServiceMetadata {
	/// The Service name.
	pub name: BoundedString<MaxMetadataLength>,
	/// The Service description.
	pub description: Option<BoundedString<MaxMetadataLength>>,
	/// The Service author.
	/// Could be a company or a person.
	pub author: Option<BoundedString<MaxMetadataLength>>,
	/// The Job category.
	pub category: Option<BoundedString<MaxMetadataLength>>,
	/// Code Repository URL.
	/// Could be a github, gitlab, or any other code repository.
	pub code_repository: Option<BoundedString<MaxMetadataLength>>,
	/// Service Logo URL.
	pub logo: Option<BoundedString<MaxMetadataLength>>,
	/// Service Website URL.
	pub website: Option<BoundedString<MaxMetadataLength>>,
	/// Service License.
	pub license: Option<BoundedString<MaxMetadataLength>>,
}

/// A Service Blueprint is a the main definition of a service.
/// it contains the metadata of the service, the job definitions, and other hooks, along with the
/// gadget that will be executed when one of the jobs is calling this service.
#[derive(Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ServiceBlueprint {
	/// The metadata of the service.
	pub metadata: ServiceMetadata,
	/// The job definitions that are available in this service.
	pub jobs: BoundedVec<JobDefinition, MaxJobsPerService>,
	/// The registration hook that will be called before restaker registration.
	pub registration_hook: ServiceRegistrationHook,
	/// The parameters that are required for the service registration.
	pub registration_params: BoundedVec<FieldType, MaxFields>,
	/// The request hook that will be called before creating a service from the service blueprint.
	pub request_hook: ServiceRequestHook,
	/// The parameters that are required for the service request.
	pub request_params: BoundedVec<FieldType, MaxFields>,
	/// The gadget that will be executed for the service.
	pub gadget: Gadget,
}

impl ServiceBlueprint {
	/// Check if the supplied arguments match the registration parameters.
	pub fn type_check_registration<AccountId: Clone>(
		&self,
		args: &[Field<AccountId>],
	) -> Result<(), TypeCheckError> {
		type_checker(&self.registration_params, args)
	}

	/// Check if the supplied arguments match the request parameters.
	pub fn type_check_request<AccountId: Clone>(
		&self,
		args: &[Field<AccountId>],
	) -> Result<(), TypeCheckError> {
		type_checker(&self.request_params, args)
	}
}

/// A service request is a request to create a service from a service blueprint.
#[derive(Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ServiceRequest<AccountId, BlockNumber> {
	/// The service blueprint ID.
	pub blueprint: u64,
	/// The owner of the service.
	pub owner: AccountId,
	/// The permitted caller(s) of the service.
	pub permitted_callers: BoundedVec<AccountId, MaxPermittedCallers>,
	/// The Lifetime of the service.
	pub ttl: BlockNumber,
	/// The supplied arguments for the service request.
	pub args: BoundedVec<Field<AccountId>, MaxFields>,
	/// The Selected Operator(s) with their approval state.
	pub operators_with_approval_state:
		BoundedVec<(AccountId, ApprovalState), MaxOperatorsPerService>,
}

impl<AccountId, BlockNumber> ServiceRequest<AccountId, BlockNumber> {
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
#[derive(Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[scale_info(skip_type_params(AccountId, BlockNumber))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Service<AccountId, BlockNumber> {
	/// The Blueprint ID of the service.
	pub blueprint: u64,
	/// The owner of the service.
	pub owner: AccountId,
	/// The Permitted caller(s) of the service.
	pub permitted_callers: BoundedVec<AccountId, MaxPermittedCallers>,
	/// The Selected operators(s) for this service.
	pub operators: BoundedVec<AccountId, MaxOperatorsPerService>,
	/// The Lifetime of the service.
	pub ttl: BlockNumber,
}

/// Operator's Approval Prefrence.
#[derive(
	Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Copy, Clone, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ApprovalPrefrence {
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

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Copy, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OperatorPreferences {
	/// The operator ECDSA public key.
	pub key: ecdsa::Public,
	/// The approval prefrence of the operator.
	pub approval: ApprovalPrefrence,
}

impl OperatorPreferences {
	/// Encode the fields to ethabi bytes.
	pub fn to_ethabi(&self) -> Vec<ethabi::Token> {
		let tokens: Vec<ethabi::Token> = vec![
			ethabi::Token::Bytes(self.key.0.to_vec()),
			// TODO: Add ApprovalPrefrence to ethabi.
		];
		tokens
	}
}

/// Operator Profile is a profile of an operator that
/// contains metadata about the services that the operator is providing.
#[derive(Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OperatorProfile {
	/// The Service IDs that I'm currently providing.
	pub services: BoundedBTreeSet<u64, MaxServicesPerOperator>,
	/// The Blueprint IDs that I'm currently registered for.
	pub blueprints: BoundedBTreeSet<u64, MaxBlueprintsPerOperator>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Gadget {
	/// A Gadget that is a WASM binary that will be executed.
	/// inside the shell using the wasm runtime.
	Wasm(WasmGadget),
	/// A Gadget that is a native binary that will be executed.
	/// inside the shell using the OS.
	Native(NativeGadget),
	/// A Gadget that is a container that will be executed.
	/// inside the shell using the container runtime (e.g. Docker, Podman, etc.)
	Container(ContainerGadget),
}

impl Default for Gadget {
	fn default() -> Self {
		Gadget::Wasm(WasmGadget::IPFS(Default::default()))
	}
}

/// A binary that is stored in the Github release.
/// this will constuct the URL to the release and download the binary.
/// The URL will be in the following format:
/// https://github.com/<owner>/<repo>/releases/download/v<tag>/<path>
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GithubFetcher {
	/// The owner of the repository.
	pub owner: BoundedString<ConstU32<512>>,
	/// The repository name.
	pub repo: BoundedString<ConstU32<512>>,
	/// The release tag of the repository.
	/// NOTE: The tag should be a valid semver tag.
	pub tag: BoundedString<ConstU32<512>>,
	/// The names of the binary in the release by the arch and the os.
	pub binaries: BoundedBTreeSet<GadgetBinary, ConstU32<32>>,
	/// The sha256 hash of the WASM binary.
	/// your service will check if the downloaded binary matches this hash.
	pub sha256: BoundedVec<u8, ConstU32<32>>,
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

#[derive(
	Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GadgetBinary {
	/// CPU or System architecture.
	pub arch: Architecture,
	/// Operating System that the binary is compiled for.
	pub os: OperatingSystem,
	/// The name of the binary.
	pub name: BoundedString<ConstU32<256>>,
	/// The sha256 hash of the binary.
	/// used to verify the downloaded binary.
	pub sha256: BoundedVec<u8, ConstU32<32>>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ImageRegistryFetcher {
	/// The URL of the container registry.
	registry: BoundedString<ConstU32<256>>,
	/// The name of the image.
	image: BoundedString<ConstU32<256>>,
	/// The tag of the image.
	tag: BoundedString<ConstU32<256>>,
}

/// A WASM binary that contains all the compiled gadget code.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum WasmGadget {
	/// A WASM binary that is stored in the IPFS.
	#[codec(index = 0)]
	IPFS(BoundedVec<u8, ConstU32<64>>),
	/// A WASM binary that is stored in the Github release.
	#[codec(index = 1)]
	Github(GithubFetcher),
}

/// A Native binary that contains all the gadget code.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum NativeGadget {
	/// A Native binary that is stored in the IPFS.
	#[codec(index = 0)]
	IPFS(BoundedVec<u8, ConstU32<64>>),
	/// A Native binary that is stored in the Github release.
	#[codec(index = 1)]
	Github(GithubFetcher),
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ContainerGadget {
	/// An Image that is stored in the IPFS.
	#[codec(index = 0)]
	IPFS(BoundedVec<u8, ConstU32<64>>),
	/// An Image that is stored in a remote container registry.
	#[codec(index = 1)]
	Registry(ImageRegistryFetcher),
}
