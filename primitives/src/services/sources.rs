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

use super::{BoundedString, constraints::Constraints};
use educe::Educe;
use frame_support::pallet_prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(tag = "type", bound = ""))]
pub enum BlueprintSource<C: Constraints> {
	/// A blueprint that is a WASM binary that will be executed with the specified runtime.
	Wasm {
		runtime: WasmRuntime,
		#[cfg_attr(feature = "std", serde(flatten))]
		fetcher: WasmFetcher<C>,
	},
	/// A blueprint that is a native binary that will be executed.
	Native(NativeFetcher<C>),
	/// A blueprint contained in a container image.
	Container(ImageRegistryFetcher<C>),
	/// A binary source used for testing the blueprint.
	Testing(TestFetcher<C>),
}

impl<C: Constraints> Default for BlueprintSource<C> {
	fn default() -> Self {
		BlueprintSource::Wasm {
			runtime: WasmRuntime::Wasmtime,
			fetcher: WasmFetcher::Github(Default::default()),
		}
	}
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

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = "", untagged))]
pub enum WasmFetcher<C: Constraints> {
	/// A WASM binary that will be fetched from the IPFS.
	#[codec(index = 0)]
	IPFS(BoundedVec<u8, C::MaxIpfsHashLength>),
	/// A WASM binary that will be fetched from a GitHub release.
	#[codec(index = 1)]
	Github(GithubFetcher<C>),
}

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[codec(encode_bound(skip_type_params(C)))]
#[codec(decode_bound(skip_type_params(C)))]
#[codec(mel_bound(skip_type_params(C)))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = "", untagged))]
pub enum NativeFetcher<C: Constraints> {
	/// A blueprint that will be fetched from the IPFS.
	#[codec(index = 0)]
	IPFS(BoundedVec<u8, C::MaxIpfsHashLength>),
	/// A blueprint that will be fetched from a GitHub release.
	#[codec(index = 1)]
	Github(GithubFetcher<C>),
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

/// A binary that is stored in a GitHub release.
///
/// This will construct the URL to the release and download the binary.
/// The URL will be in the following format:
///
/// `https://github.com/<owner>/<repo>/releases/download/v<tag>/<path>`
#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
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
	pub binaries: BoundedVec<BlueprintBinary<C>, C::MaxBinariesPerGadget>,
}

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Default(bound()), Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
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
pub struct BlueprintBinary<C: Constraints> {
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
