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

use fp_evm::{ExitRevert, PrecompileFailure, PrecompileHandle};
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::ConstU32,
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_evm::AddressMapping;
use pallet_jobs::Call as JobsCall;
use precompile_utils::{prelude::*, solidity::revert::revert_as_bytes};
use sp_core::H256;
use sp_runtime::traits::Dispatchable;
use sp_std::{marker::PhantomData, vec::Vec};
use tangle_primitives::{
	jobs::{DKGTSSPhaseOneJobType, DKGTSSPhaseTwoJobType, JobId, JobSubmission, JobType},
	roles::RoleType,
	types::BlockNumber,
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub const JOB_SUBMISSION_SIZE_LIMIT: u32 = 2u32.pow(16);
type GetJobSubmissionSizeLimit = ConstU32<JOB_SUBMISSION_SIZE_LIMIT>;

/// A precompile to wrap the functionality from pallet-preimage.
pub struct JobsPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> JobsPrecompile<Runtime>
where
	Runtime: pallet_jobs::Config + pallet_evm::Config + frame_system::Config,
	<Runtime as frame_system::Config>::Hash: TryFrom<H256> + Into<H256>,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::Hash: Into<H256>,
	<Runtime as frame_system::Config>::RuntimeCall: From<JobsCall<Runtime>>,
	BlockNumberFor<Runtime>: From<u64>,
{
	/// Submits a job for Distributed Key Generation (DKG) phase one.
	///
	/// # Parameters
	///
	/// - `handle`: A mutable reference to the `PrecompileHandle` implementation.
	/// - `expiry`: The expiration period for the submitted job
	/// - `ttl`: The time-to-live period for the submitted job
	/// - `participants`: A vector containing Ethereum addresses of the participants in the DKG.
	/// - `threshold`: The threshold number of participants required for the DKG to succeed (u8).
	/// - `permitted_caller`: The Ethereum address of the permitted caller.
	///
	/// # Returns
	///
	/// Returns an `EvmResult`, indicating the success or failure of the operation.
	#[precompile::public("submitDkgPhaseOneJob(uint64,uint64,address[],uint8,uint16,address)")]
	fn submit_dkg_phase_one_job(
		handle: &mut impl PrecompileHandle,
		expiry: BlockNumber,
		ttl: BlockNumber,
		participants: Vec<Address>,
		threshold: u8,
		role_type: u16,
		permitted_caller: Address,
	) -> EvmResult {
		// Convert Ethereum address to Substrate account ID
		let permitted_caller = Runtime::AddressMapping::into_account_id(permitted_caller.0);

		// Convert Ethereum addresses of participants to Substrate account IDs
		let participants = participants
			.iter()
			.map(|x| Runtime::AddressMapping::into_account_id(x.0))
			.collect::<Vec<_>>()
			.try_into()
			.unwrap();

		// Convert (u16) role type to RoleType
		let role_type = Self::convert_role_type(role_type);

		// Check if job key is valid, otherwise return an error
		if role_type.is_none() {
			return Err(PrecompileFailure::Revert {
				exit_status: ExitRevert::Reverted,
				output: revert_as_bytes("Invalid role type!"),
			})
		}

		let threshold_signature_role = match role_type {
			Some(RoleType::Tss(role)) => role,
			_ =>
				return Err(PrecompileFailure::Revert {
					exit_status: ExitRevert::Reverted,
					output: revert_as_bytes("Invalid role type!"),
				}),
		};

		// Create DKG job type with the provided parameters
		let job_type = DKGTSSPhaseOneJobType {
			role_type: threshold_signature_role,
			participants,
			threshold,
			permitted_caller: Some(permitted_caller),
		};

		// Convert expiration period to Substrate block number
		let expiry_block: BlockNumberFor<Runtime> = expiry.into();
		let ttl_block: BlockNumberFor<Runtime> = ttl.into();

		// Create job submission object
		let job = JobSubmission {
			expiry: expiry_block,
			ttl: ttl_block,
			job_type: JobType::DKGTSSPhaseOne(job_type),
		};

		// Convert caller's Ethereum address to Substrate account ID
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		// Create the call to the Jobs module's submit_job function
		let call = JobsCall::<Runtime>::submit_job { job };

		// Dispatch the call using the RuntimeHelper
		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Submits a job for Distributed Key Generation (DKG) phase two, including the signature
	/// submission.
	///
	/// # Parameters
	///
	/// - `handle`: A mutable reference to the `PrecompileHandle` implementation.
	/// - `expiry`: The expiration period for the submitted job
	/// - `ttl`: The time-to-live period for the submitted job
	/// - `phase_one_id`: The identifier of the corresponding phase one DKG job (u32).
	/// - `submission`: The signature submission for the DKG phase two, represented as
	///   `BoundedBytes`.
	///
	/// # Returns
	///
	/// Returns an `EvmResult`, indicating the success or failure of the operation.
	#[precompile::public("submitDkgPhaseTwoJob(uint64,uint64,uint64,bytes)")]
	fn submit_dkg_phase_two_job(
		handle: &mut impl PrecompileHandle,
		expiry: BlockNumber,
		ttl: BlockNumber,
		phase_one_id: JobId,
		submission: BoundedBytes<GetJobSubmissionSizeLimit>,
	) -> EvmResult {
		// Convert BoundedBytes to Vec<u8>
		let submission: Vec<u8> = submission.try_into().unwrap();

		// Convert caller's Ethereum address to Substrate account ID
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		// Convert expiration period to Substrate block number
		let expiry_block: BlockNumberFor<Runtime> = expiry.into();
		let ttl_block: BlockNumberFor<Runtime> = ttl.into();

		// Create DKG signature job type with the provided parameters
		match pallet_jobs::SubmittedJobsRole::<Runtime>::get(phase_one_id) {
			Some(role_type) => {
				// Parse the inner role type. It should be a TSS role.
				let threshold_signature_role = match role_type {
					RoleType::Tss(role) => role,
					_ =>
						return Err(PrecompileFailure::Revert {
							exit_status: ExitRevert::Reverted,
							output: revert_as_bytes("Invalid role type!"),
						}),
				};

				// Construct the phase 2 job type.
				let job_type = DKGTSSPhaseTwoJobType {
					role_type: threshold_signature_role,
					phase_one_id,
					submission: submission.try_into().unwrap(),
				};

				// Create job submission object
				let job = JobSubmission {
					expiry: expiry_block,
					ttl: ttl_block,
					job_type: JobType::DKGTSSPhaseTwo(job_type),
				};

				// Create the call to the Jobs module's submit_job function
				let call = JobsCall::<Runtime>::submit_job { job };

				// Dispatch the call using the RuntimeHelper
				<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

				Ok(())
			},
			None => Err(PrecompileFailure::Revert {
				exit_status: ExitRevert::Reverted,
				output: revert_as_bytes("Invalid job ID!"),
			}),
		}
	}

	/// Sets a new permitted caller for a specific job type identified by the given key and job ID.
	///
	/// # Parameters
	///
	/// - `handle`: A mutable reference to the `PrecompileHandle` implementation.
	/// - `role_type`: An identifier specifying the role of the job to update the permitted caller
	///   for (u16) - first byte is the top-level role (TSS, ZkSaaS, etc.), second byte is the
	///   sub-role.
	/// - `job_id`: The unique identifier of the job for which the permitted caller is being updated
	///   (u32).
	/// - `new_permitted_caller`: The Ethereum address of the new permitted caller.
	///
	/// # Returns
	///
	/// Returns an `EvmResult`, indicating the success or failure of the operation.
	#[precompile::public("setPermittedCaller(uint16,uint64,address)")]
	fn set_permitted_caller(
		handle: &mut impl PrecompileHandle,
		role_type: u16,
		job_id: JobId,
		new_permitted_caller: Address,
	) -> EvmResult {
		// Convert (u16) role_type to RoleType
		let role_type = Self::convert_role_type(role_type);

		// Check if role type is valid, otherwise return an error
		if role_type.is_none() {
			return Err(PrecompileFailure::Revert {
				exit_status: ExitRevert::Reverted,
				output: revert_as_bytes("Invalid role type!"),
			})
		}

		// Convert Ethereum address to Substrate account ID
		let new_permitted_caller = Runtime::AddressMapping::into_account_id(new_permitted_caller.0);

		// Convert caller's Ethereum address to Substrate account ID
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		// Create the call to the Jobs module's set_permitted_caller function
		let call = JobsCall::<Runtime>::set_permitted_caller {
			role_type: role_type.unwrap(),
			job_id,
			new_permitted_caller,
		};

		// Dispatch the call using the RuntimeHelper
		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	fn convert_role_type(role_type: u16) -> Option<RoleType> {
		role_type.try_into().ok()
	}
}
