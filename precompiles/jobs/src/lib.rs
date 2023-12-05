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

use fp_evm::{ExitRevert, PrecompileFailure, PrecompileHandle};
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::ConstU32,
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_evm::AddressMapping;
use pallet_jobs::Call as JobsCall;
use precompile_utils::{prelude::*, solidity::revert::revert_as_bytes};
use sp_core::H256;
use sp_std::{marker::PhantomData, vec::Vec};
use tangle_primitives::jobs::{
	DKGJobType, DKGSignatureJobType, JobKey, JobSubmission, JobType, ZkSaasPhaseOneJobType,
	ZkSaasPhaseTwoJobType,
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
	/// - `participants`: A vector containing Ethereum addresses of the participants in the DKG.
	/// - `threshold`: The threshold number of participants required for the DKG to succeed (u8).
	/// - `permitted_caller`: The Ethereum address of the permitted caller.
	///
	/// # Returns
	///
	/// Returns an `EvmResult`, indicating the success or failure of the operation.
	#[precompile::public("submitDkgPhaseOneJob(uint64,address[],uint8,address)")]
	fn submit_dkg_phase_one_job(
		handle: &mut impl PrecompileHandle,
		expiry: u64,
		participants: Vec<Address>,
		threshold: u8,
		permitted_caller: Address,
	) -> EvmResult {
		// Convert Ethereum address to Substrate account ID
		let permitted_caller = Runtime::AddressMapping::into_account_id(permitted_caller.0);

		// Convert Ethereum addresses of participants to Substrate account IDs
		let participants = participants
			.iter()
			.map(|x| Runtime::AddressMapping::into_account_id(x.0))
			.collect();

		// Create DKG job type with the provided parameters
		let job_type =
			DKGJobType { participants, threshold, permitted_caller: Some(permitted_caller) };

		// Convert expiration period to Substrate block number
		let expiry_block: BlockNumberFor<Runtime> = expiry.into();

		// Create job submission object
		let job = JobSubmission { expiry: expiry_block, job_type: JobType::DKG(job_type) };

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
	/// - `phase_one_id`: The identifier of the corresponding phase one DKG job (u32).
	/// - `submission`: The signature submission for the DKG phase two, represented as
	///   `BoundedBytes`.
	///
	/// # Returns
	///
	/// Returns an `EvmResult`, indicating the success or failure of the operation.
	#[precompile::public("submitDkgPhaseTwoJob(uint64,uint32,bytes)")]
	fn submit_dkg_phase_two_job(
		handle: &mut impl PrecompileHandle,
		expiry: u64,
		phase_one_id: u32,
		submission: BoundedBytes<GetJobSubmissionSizeLimit>,
	) -> EvmResult {
		// Convert BoundedBytes to Vec<u8>
		let submission: Vec<u8> = submission.into();

		// Convert caller's Ethereum address to Substrate account ID
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		// Convert expiration period to Substrate block number
		let expiry_block: BlockNumberFor<Runtime> = expiry.into();

		// Create DKG signature job type with the provided parameters
		let job_type = DKGSignatureJobType { phase_one_id, submission };

		// Create job submission object
		let job = JobSubmission { expiry: expiry_block, job_type: JobType::DKGSignature(job_type) };

		// Create the call to the Jobs module's submit_job function
		let call = JobsCall::<Runtime>::submit_job { job };

		// Dispatch the call using the RuntimeHelper
		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Submits a job for the first phase of the zksaas protocol.
	///
	/// # Parameters
	///
	/// - `handle`: A mutable reference to the `PrecompileHandle` implementation.
	/// - `expiry`: The expiration period for the submitted job
	/// - `participants`: A vector containing Ethereum addresses of the participants in the ZkSaas
	///   phase one.
	/// - `permitted_caller`: The Ethereum address of the permitted caller.
	///
	/// # Returns
	///
	/// Returns an `EvmResult`, indicating the success or failure of the operation.
	#[precompile::public("submitDkgzkSaasPhaseOneJob(uint64,address[],address)")]
	fn submit_zksaas_phase_one_job(
		handle: &mut impl PrecompileHandle,
		expiry: u64,
		participants: Vec<Address>,
		permitted_caller: Address,
	) -> EvmResult {
		// Convert Ethereum address to Substrate account ID
		let permitted_caller = Runtime::AddressMapping::into_account_id(permitted_caller.0);

		// Convert Ethereum addresses of participants to Substrate account IDs
		let participants = participants
			.iter()
			.map(|x| Runtime::AddressMapping::into_account_id(x.0))
			.collect();

		// Create ZkSaas phase one job type with the provided parameters
		let job_type =
			ZkSaasPhaseOneJobType { participants, permitted_caller: Some(permitted_caller) };

		// Convert expiration period to Substrate block number
		let expiry_block: BlockNumberFor<Runtime> = expiry.into();

		// Create job submission object
		let job =
			JobSubmission { expiry: expiry_block, job_type: JobType::ZkSaasPhaseOne(job_type) };

		// Convert caller's Ethereum address to Substrate account ID
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		// Create the call to the Jobs module's submit_job function
		let call = JobsCall::<Runtime>::submit_job { job };

		// Dispatch the call using the RuntimeHelper
		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Submits a job for the second phase of the zksaas protocol,
	/// including the submission of data from the first phase.
	///
	/// # Parameters
	///
	/// - `handle`: A mutable reference to the `PrecompileHandle` implementation.
	/// - `expiry`: The expiration period for the submitted job
	/// - `phase_one_id`: The identifier of the corresponding phase one ZkSaas job (u32).
	/// - `submission`: The data submission for the ZkSaas phase two, represented as `BoundedBytes`.
	///
	/// # Returns
	///
	/// Returns an `EvmResult`, indicating the success or failure of the operation.
	#[precompile::public("submitzkSaasPhaseTwoJob(uint64,uint32,bytes)")]
	fn submit_zksaas_phase_two_job(
		handle: &mut impl PrecompileHandle,
		expiry: u64,
		phase_one_id: u32,
		submission: BoundedBytes<GetJobSubmissionSizeLimit>,
	) -> EvmResult {
		// Convert BoundedBytes to Vec<u8>
		let submission: Vec<u8> = submission.into();

		// Convert caller's Ethereum address to Substrate account ID
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		// Convert expiration period to Substrate block number
		let expiry_block: BlockNumberFor<Runtime> = expiry.into();

		// Create ZkSaas phase two job type with the provided parameters
		let job_type = ZkSaasPhaseTwoJobType { phase_one_id, submission };

		// Create job submission object
		let job =
			JobSubmission { expiry: expiry_block, job_type: JobType::ZkSaasPhaseTwo(job_type) };

		// Create the call to the Jobs module's submit_job function
		let call = JobsCall::<Runtime>::submit_job { job };

		// Dispatch the call using the RuntimeHelper
		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Withdraws accumulated rewards for the caller from the Jobs module.
	///
	/// # Parameters
	///
	/// - `handle`: A mutable reference to the `PrecompileHandle` implementation.
	///
	/// # Returns
	///
	/// Returns an `EvmResult`, indicating the success or failure of the operation.
	#[precompile::public("withdrawRewards()")]
	fn withdraw_rewards(handle: &mut impl PrecompileHandle) -> EvmResult {
		// Convert caller's Ethereum address to Substrate account ID
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		// Create the call to the Jobs module's withdraw_rewards function
		let call = JobsCall::<Runtime>::withdraw_rewards {};

		// Dispatch the call using the RuntimeHelper
		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Sets a new permitted caller for a specific job type identified by the given key and job ID.
	///
	/// # Parameters
	///
	/// - `handle`: A mutable reference to the `PrecompileHandle` implementation.
	/// - `job_key`: An identifier specifying the type of job to update the permitted caller for
	///   (u8).
	/// - `job_id`: The unique identifier of the job for which the permitted caller is being updated
	///   (u32).
	/// - `new_permitted_caller`: The Ethereum address of the new permitted caller.
	///
	/// # Returns
	///
	/// Returns an `EvmResult`, indicating the success or failure of the operation.
	#[precompile::public("setPermittedCaller(uint8,uint32,address)")]
	fn set_permitted_caller(
		handle: &mut impl PrecompileHandle,
		job_key: u8,
		job_id: u32,
		new_permitted_caller: Address,
	) -> EvmResult {
		// Convert job_key to JobKey
		let job_key = match job_key {
			0 => Some(JobKey::DKG),
			1 => Some(JobKey::DKGSignature),
			2 => Some(JobKey::ZkSaasPhaseOne),
			3 => Some(JobKey::ZkSaasPhaseTwo),
			_ => None,
		};

		// Check if job key is valid, otherwise return an error
		if job_key.is_none() {
			return Err(PrecompileFailure::Revert {
				exit_status: ExitRevert::Reverted,
				output: revert_as_bytes("Invalid job key!"),
			})
		}

		// Convert Ethereum address to Substrate account ID
		let new_permitted_caller = Runtime::AddressMapping::into_account_id(new_permitted_caller.0);

		// Convert caller's Ethereum address to Substrate account ID
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		// Create the call to the Jobs module's set_permitted_caller function
		let call = JobsCall::<Runtime>::set_permitted_caller {
			job_key: job_key.unwrap(),
			job_id,
			new_permitted_caller,
		};

		// Dispatch the call using the RuntimeHelper
		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}
}
