// Copyright 2022 Webb Technologies Inc.
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

use fp_evm::PrecompileHandle;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::ConstU32,
};
use pallet_evm::AddressMapping;
use pallet_preimage::Call as PreimageCall;
use precompile_utils::prelude::*;
use sp_core::{Hasher, H256};
use sp_std::{marker::PhantomData, vec::Vec};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub const ENCODED_PROPOSAL_SIZE_LIMIT: u32 = 2u32.pow(16);
type GetEncodedProposalSizeLimit = ConstU32<ENCODED_PROPOSAL_SIZE_LIMIT>;

/// Solidity selector of the PreimageNoted log, which is the Keccak of the Log signature.
pub(crate) const SELECTOR_LOG_PREIMAGE_NOTED: [u8; 32] = keccak256!("PreimageNoted(bytes32)");

/// Solidity selector of the PreimageUnnoted log, which is the Keccak of the Log signature.
pub(crate) const SELECTOR_LOG_PREIMAGE_UNNOTED: [u8; 32] = keccak256!("PreimageUnnoted(bytes32)");

/// A precompile to wrap the functionality from pallet-preimage.
pub struct PreimagePrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> PreimagePrecompile<Runtime>
where
	Runtime: pallet_preimage::Config + pallet_evm::Config + frame_system::Config,
	<Runtime as frame_system::Config>::Hash: TryFrom<H256> + Into<H256>,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::Hash: Into<H256>,
	<Runtime as frame_system::Config>::RuntimeCall: From<PreimageCall<Runtime>>,
{
	/// Register a preimage on-chain.
	///
	/// Parameters:
	/// * encoded_proposal: The preimage registered on-chain
	#[precompile::public("submitJob(bytes)")]
	fn submit_job(
		handle: &mut impl PrecompileHandle,
		encoded_proposal: BoundedBytes<GetEncodedProposalSizeLimit>,
	) -> EvmResult<H256> {
		let bytes: Vec<u8> = encoded_proposal.into();
		let hash: H256 = Runtime::Hashing::hash(&bytes).into();

		let event = log1(
			handle.context().address,
			SELECTOR_LOG_PREIMAGE_NOTED,
			solidity::encode_arguments(hash),
		);
		handle.record_log_costs(&[&event])?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = PreimageCall::<Runtime>::note_preimage { bytes };

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		event.record(handle)?;
		Ok(hash)
	}
}
