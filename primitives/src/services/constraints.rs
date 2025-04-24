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

use frame_support::pallet_prelude::*;

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
	/// Maximum length of an rpc address.
	type MaxRpcAddressLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
	/// Maximum number of resource types for pricing.
	type MaxResourceNameLength: Get<u32> + Default + Parameter + MaybeSerializeDeserialize;
}
