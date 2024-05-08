// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
//
// This file is part of pallet-evm-precompile-preimage package, originally developed by Purestake
// Inc. Pallet-evm-precompile-preimage package used in Tangle Network in terms of GPLv3.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

use fp_evm::{ExitRevert, PrecompileFailure, PrecompileHandle};
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::ConstU32,
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_evm::AddressMapping;
use pallet_services::Call as ServicesCall;
use precompile_utils::prelude::*;
use sp_core::H256;
use sp_runtime::traits::Dispatchable;
use sp_std::{marker::PhantomData, vec::Vec};
use tangle_primitives::{jobs::v2::ServiceBlueprint, types::BlockNumber};

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

/// A precompile to wrap the functionality from pallet-services.
pub struct ServicesPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> ServicesPrecompile<Runtime>
where
	Runtime: pallet_services::Config + pallet_evm::Config + frame_system::Config,
	<Runtime as frame_system::Config>::Hash: TryFrom<H256> + Into<H256>,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::Hash: Into<H256>,
	<Runtime as frame_system::Config>::RuntimeCall: From<ServicesCall<Runtime>>,
	BlockNumberFor<Runtime>: From<u64>,
{
	/// Returns a list of service providers for a given service blueprint.
	#[precompile::public("serviceProviders(uint64)")]
	#[precompile::public("service_providers(uint64)")]
	#[precompile::view]
	pub fn service_providers(
		handle: &mut impl PrecompileHandle,
		blueprint_id: u64,
	) -> EvmResult<Vec<BoundedBytes<ConstU32<33>>>> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let service_providers = pallet_services::Operators::<Runtime>::iter_prefix(blueprint_id)
			.map(|(_, prefs)| BoundedBytes::from(prefs.key.0))
			.collect::<Vec<_>>();
		Ok(service_providers)
	}
}
