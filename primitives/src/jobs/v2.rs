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
use sp_core::RuntimeDebug;
use sp_runtime::traits::Get;

mod field;
pub use field::*;

/// A Job Definition is a definition of a job that can be called.
/// It contains the input and output fields of the job with the permitted caller.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct JobDefinition<AccountId, MaxFields: Get<u32>, MaxMetadataLength: Get<u32>> {
	/// The metadata of the job.
	pub metadata: JobMetadata<MaxMetadataLength>,
	/// These are parameters that are required for this job.
	/// i.e. the input.
	pub params: BoundedVec<FieldType, MaxFields>,
	/// These are the result, the return values of this job.
	/// i.e. the output.
	pub result: BoundedVec<FieldType, MaxFields>,
	/// The caller who can trigger this submission of this job type
	pub permitted_caller: Option<AccountId>,
	/// The verifier of the job result.
	pub verifier: JobResultVerifier,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct JobMetadata<MaxLength: Get<u32>> {
	/// The Job name.
	pub name: BoundedString<MaxLength>,
	/// The Job description.
	pub description: Option<BoundedString<MaxLength>>,
}

/// A Job Call is a call to execute a job using it's job definition.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct JobCall<AccountId, MaxFieldsSize: Get<u32>, MaxFields: Get<u32>> {
	/// The Service ID that this call is for.
	pub service_id: u64,
	/// The job definition index in the service that this call is for.
	pub job_index: u64,
	/// The supplied arguments for this job call.
	pub args: BoundedVec<Field<AccountId, MaxFieldsSize>, MaxFields>,
}

impl<AccountId: Clone, MaxFieldsSize: Get<u32> + Clone, MaxFields: Get<u32> + Clone>
	JobCall<AccountId, MaxFieldsSize, MaxFields>
{
	/// Check if the supplied arguments match the job definition types.
	pub fn type_check<MaxMetadataLength: Get<u32>>(
		&self,
		job_def: &JobDefinition<AccountId, MaxFields, MaxMetadataLength>,
	) -> Result<(), TypeCheckError> {
		if job_def.params.len() != self.args.len() {
			return Err(TypeCheckError::NotEnoughArguments {
				expected: job_def.params.len() as u8,
				actual: self.args.len() as u8,
			});
		}

		for i in 0..self.args.len() {
			let arg = &self.args[i];
			let expected = &job_def.params[i];
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
}

impl<AccountId: Clone, MaxFieldsSize: Get<u32> + Clone, MaxFields: Get<u32> + Clone>
	JobCallResult<AccountId, MaxFieldsSize, MaxFields>
{
	/// Check if the supplied result match the job definition types.
	pub fn type_check<MaxMetadataLength: Get<u32>>(
		&self,
		job_def: &JobDefinition<AccountId, MaxFields, MaxMetadataLength>,
	) -> Result<(), TypeCheckError> {
		if job_def.result.len() != self.result.len() {
			return Err(TypeCheckError::NotEnoughArguments {
				expected: job_def.result.len() as u8,
				actual: self.result.len() as u8,
			});
		}

		for i in 0..self.result.len() {
			let arg = &self.result[i];
			let expected = &job_def.result[i];
			if arg != expected {
				return Err(TypeCheckError::ResultTypeMismatch {
					index: i as u8,
					expected: expected.clone(),
					actual: arg.clone().into(),
				});
			}
		}

		Ok(())
	}
}

/// A Job Call Result is the result of a job call.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct JobCallResult<AccountId, MaxFieldsSize: Get<u32>, MaxFields: Get<u32>> {
	/// The id of the job call.
	pub call_id: u64,
	/// The result of the job call.
	pub result: BoundedVec<Field<AccountId, MaxFieldsSize>, MaxFields>,
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

// -*** Service ***-

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ServiceMetadata<MaxLength: Get<u32>> {
	/// The Service name.
	pub name: BoundedString<MaxLength>,
	/// The Service description.
	pub description: Option<BoundedString<MaxLength>>,
	/// The Service author.
	/// Could be a company or a person.
	pub author: Option<BoundedString<MaxLength>>,
	/// The Job category.
	pub category: Option<BoundedString<MaxLength>>,
	/// Code Repository URL.
	/// Could be a github, gitlab, or any other code repository.
	pub code_repository: Option<BoundedString<MaxLength>>,
	/// Service Logo URL.
	pub logo: Option<BoundedString<MaxLength>>,
	/// Service Website URL.
	pub website: Option<BoundedString<MaxLength>>,
	/// Service License.
	pub license: Option<BoundedString<MaxLength>>,
}

/// A Service is a collection of job definitions, where each job definition is a job that can be
/// called in the service.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Service<AccountId, MaxJobs: Get<u32>, MaxFields: Get<u32>, MaxMetadataLength: Get<u32>> {
	/// The metadata of the service.
	pub metadata: ServiceMetadata<MaxMetadataLength>,
	/// The jobs that are available in this service.
	pub jobs: BoundedVec<JobDefinition<AccountId, MaxFields, MaxMetadataLength>, MaxJobs>,
	/// The gadget that will be executed for the service.
	pub gadget: Gadget,
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

/// A WASM binary that is stored in the Github release.
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
	/// The path to the WASM binary in the release.
	pub path: BoundedString<ConstU32<512>>,
	/// The sha256 hash of the WASM binary.
	/// your service will check if the downloaded binary matches this hash.
	pub sha256: BoundedVec<u8, ConstU32<32>>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct RemoteFetcher {
	/// The URL of the remote server.
	pub url: BoundedString<ConstU32<1024>>,
	/// The sha256 hash of the binary.
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
	IPFS(BoundedVec<u8, ConstU32<64>>),
	/// A WASM binary that is stored in the Github release.
	Github(GithubFetcher),
	/// A WASM binary that is stored in the remote server.
	Remote(RemoteFetcher),
}

/// A Native binary that contains all the gadget code.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum NativeGadget {
	/// A Native binary that is stored in the IPFS.
	IPFS(BoundedVec<u8, ConstU32<64>>),
	/// A Native binary that is stored in the Github release.
	Github(GithubFetcher),
	/// A Native binary that is stored in the remote server.
	Remote(RemoteFetcher),
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ContainerGadget {
	/// An Image that is stored in the IPFS.
	IPFS(BoundedVec<u8, ConstU32<64>>),
	/// An Image that is stored in a remote container registry.
	Registry(ImageRegistryFetcher),
}
