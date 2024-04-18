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

use crate::roles::RoleType;
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;
use sp_runtime::traits::Get;
use sp_std::vec::Vec;

pub type JobId = u64;

pub mod traits;
pub mod tss;
/// Temporary module to design the v2 SPEC, once done it will be moved here.
pub mod v2;
pub mod zksaas;

pub use tss::*;
pub use zksaas::*;

/// Represents a job submission with specified `AccountId` and `BlockNumber`.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
pub struct JobSubmission<
	AccountId,
	BlockNumber,
	MaxParticipants: Get<u32> + Clone,
	MaxSubmissionLen: Get<u32>,
	MaxAdditionalParamsLen: Get<u32>,
> {
	/// Represents the maximum allowed submission time for a job result.
	/// Once this time has passed, the result cannot be submitted.
	pub expiry: BlockNumber,

	/// The time-to-live (TTL) for the job, which determines the maximum allowed time for this job
	/// to be available. After the TTL expires, the job can no longer be used.
	pub ttl: BlockNumber,

	/// The type of the job submission.
	pub job_type: JobType<AccountId, MaxParticipants, MaxSubmissionLen, MaxAdditionalParamsLen>,

	/// The fallback option selected by user
	pub fallback: FallbackOptions,
}

/// Represents a job info
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
pub struct JobInfo<
	AccountId,
	BlockNumber,
	Balance,
	MaxParticipants: Get<u32> + Clone,
	MaxSubmissionLen: Get<u32>,
	MaxAdditionalParamsLen: Get<u32>,
> {
	/// The caller that requested the job
	pub owner: AccountId,

	/// Represents the maximum allowed submission time for a job result.
	/// Once this time has passed, the result cannot be submitted.
	pub expiry: BlockNumber,

	/// The time-to-live (TTL) for the job, which determines the maximum allowed time for this job
	/// to be available. After the TTL expires, the job can no longer be used.
	pub ttl: BlockNumber,

	/// The type of the job submission.
	pub job_type: JobType<AccountId, MaxParticipants, MaxSubmissionLen, MaxAdditionalParamsLen>,

	/// The fee taken for the job
	pub fee: Balance,

	/// The fallback option selected by user
	pub fallback: FallbackOptions,
}

/// Represents a job with its result.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
pub struct JobWithResult<
	AccountId,
	MaxParticipants: Get<u32> + Clone,
	MaxSubmissionLen: Get<u32>,
	MaxKeyLen: Get<u32>,
	MaxDataLen: Get<u32>,
	MaxSignatureLen: Get<u32>,
	MaxProofLen: Get<u32>,
	MaxAdditionalParamsLen: Get<u32>,
> {
	/// Current Job type
	pub job_type: JobType<AccountId, MaxParticipants, MaxSubmissionLen, MaxAdditionalParamsLen>,
	/// Phase one job type if any.
	///
	/// None if this job is a phase one job.
	pub phase_one_job_type:
		Option<JobType<AccountId, MaxParticipants, MaxSubmissionLen, MaxAdditionalParamsLen>>,
	/// Current job result
	pub result: JobResult<
		MaxParticipants,
		MaxKeyLen,
		MaxSignatureLen,
		MaxDataLen,
		MaxProofLen,
		MaxAdditionalParamsLen,
	>,
}

/// Enum representing different types of jobs.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum JobType<
	AccountId,
	MaxParticipants: Get<u32> + Clone,
	MaxSubmissionLen: Get<u32>,
	MaxAdditionalParamsLen: Get<u32>,
> {
	/// Distributed Key Generation (DKG) job type.
	DKGTSSPhaseOne(DKGTSSPhaseOneJobType<AccountId, MaxParticipants>),
	/// DKG Signature job type.
	DKGTSSPhaseTwo(DKGTSSPhaseTwoJobType<MaxSubmissionLen, MaxAdditionalParamsLen>),
	/// DKG Key Refresh job type.
	DKGTSSPhaseThree(DKGTSSPhaseThreeJobType),
	/// DKG Key Rotation job type.
	DKGTSSPhaseFour(DKGTSSPhaseFourJobType),
	/// (zk-SNARK) Create Circuit job type.
	ZkSaaSPhaseOne(ZkSaaSPhaseOneJobType<AccountId, MaxParticipants, MaxSubmissionLen>),
	/// (zk-SNARK) Create Proof job type.
	ZkSaaSPhaseTwo(ZkSaaSPhaseTwoJobType<MaxSubmissionLen>),
}

impl<
		AccountId,
		MaxParticipants: Get<u32> + Clone,
		MaxSubmissionLen: Get<u32>,
		MaxAdditionalParamsLen: Get<u32>,
	> JobType<AccountId, MaxParticipants, MaxSubmissionLen, MaxAdditionalParamsLen>
{
	/// Checks if the job type is a phase one job.
	pub fn is_phase_one(&self) -> bool {
		use crate::jobs::JobType::*;
		if matches!(self, DKGTSSPhaseOne(_) | ZkSaaSPhaseOne(_)) {
			return true;
		}
		false
	}

	/// Gets the participants for the job type, if applicable.
	pub fn get_participants(self) -> Option<BoundedVec<AccountId, MaxParticipants>> {
		use crate::jobs::JobType::*;
		match self {
			DKGTSSPhaseOne(info) => Some(info.participants),
			ZkSaaSPhaseOne(info) => Some(info.participants),
			_ => None,
		}
	}

	/// Gets the threshold value for the job type, if applicable.
	pub fn get_threshold(self) -> Option<u8> {
		use crate::jobs::JobType::*;
		match self {
			DKGTSSPhaseOne(info) => Some(info.threshold),
			_ => None,
		}
	}

	/// Gets the role associated with the job type.
	pub fn get_role_type(&self) -> RoleType {
		match self {
			JobType::DKGTSSPhaseOne(job) => RoleType::Tss(job.role_type),
			JobType::ZkSaaSPhaseOne(job) => RoleType::ZkSaaS(job.role_type),
			JobType::DKGTSSPhaseTwo(job) => RoleType::Tss(job.role_type),
			JobType::ZkSaaSPhaseTwo(job) => RoleType::ZkSaaS(job.role_type),
			JobType::DKGTSSPhaseThree(job) => RoleType::Tss(job.role_type),
			JobType::DKGTSSPhaseFour(job) => RoleType::Tss(job.role_type),
		}
	}

	/// Performs a basic sanity check on the job type.
	///
	/// This function is intended for simple checks and may need improvement in the future.
	pub fn sanity_check(&self) -> bool {
		match self {
			JobType::DKGTSSPhaseOne(info) => info.participants.len() >= info.threshold.into(),
			JobType::ZkSaaSPhaseOne(info) => !info.participants.is_empty(),
			_ => true,
		}
	}

	/// Gets the phase one ID for phase two jobs, if applicable.
	pub fn get_phase_one_id(&self) -> Option<JobId> {
		use crate::jobs::JobType::*;
		match self {
			DKGTSSPhaseTwo(info) => Some(info.phase_one_id),
			DKGTSSPhaseThree(info) => Some(info.phase_one_id),
			DKGTSSPhaseFour(info) => Some(info.phase_one_id),
			ZkSaaSPhaseTwo(info) => Some(info.phase_one_id),
			_ => None,
		}
	}

	/// Updates the phase one ID for phase two jobs, if applicable.
	pub fn update_phase_one_id(&mut self, new_phase_one_id: u64) {
		use crate::jobs::JobType::*;
		match self {
			DKGTSSPhaseTwo(info) => info.phase_one_id = new_phase_one_id,
			DKGTSSPhaseThree(info) => info.phase_one_id = new_phase_one_id,
			DKGTSSPhaseFour(info) => info.phase_one_id = new_phase_one_id,
			ZkSaaSPhaseTwo(info) => info.phase_one_id = new_phase_one_id,
			_ => {},
		}
	}

	pub fn get_permitted_caller(self) -> Option<AccountId> {
		use crate::jobs::JobType::*;
		match self {
			DKGTSSPhaseOne(info) => info.permitted_caller,
			ZkSaaSPhaseOne(info) => info.permitted_caller,
			_ => None,
		}
	}
}

/// Enum representing different states of a job.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum JobState {
	/// The job is active.
	Active,
	/// The job is pending.
	Pending,
	/// The job has been terminated.
	Terminated,
}

/// Enum representing fallback options that the job creator has
#[derive(Copy, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum FallbackOptions {
	/// The job should be destroyed and caller refunded
	Destroy,
	/// The job can be regenerated with a lower threshold
	/// The user supplies the lower threshold
	RegenerateWithThreshold(u8),
}

/// Represents a job submission with specified `AccountId` and `BlockNumber`.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PhaseResult<
	AccountId,
	BlockNumber,
	MaxParticipants: Get<u32> + Clone,
	MaxKeyLen: Get<u32>,
	MaxDataLen: Get<u32>,
	MaxSignatureLen: Get<u32>,
	MaxSubmissionLen: Get<u32>,
	MaxProofLen: Get<u32>,
	MaxAdditionalParamsLen: Get<u32>,
> {
	/// The owner's account ID.
	pub owner: AccountId,
	/// The type of the job submission.
	pub result: JobResult<
		MaxParticipants,
		MaxKeyLen,
		MaxSignatureLen,
		MaxDataLen,
		MaxProofLen,
		MaxAdditionalParamsLen,
	>,
	/// The time-to-live (TTL) for the job, which determines the maximum allowed time for this job
	/// to be available. After the TTL expires, the job can no longer be used.
	pub ttl: BlockNumber,
	/// permitted caller to use this result
	pub permitted_caller: Option<AccountId>,
	/// The type of the job submission.
	pub job_type: JobType<AccountId, MaxParticipants, MaxSubmissionLen, MaxAdditionalParamsLen>,
}

impl<
		AccountId,
		BlockNumber,
		MaxParticipants: Get<u32> + Clone,
		MaxKeyLen: Get<u32>,
		MaxDataLen: Get<u32>,
		MaxSignatureLen: Get<u32>,
		MaxSubmissionLen: Get<u32>,
		MaxProofLen: Get<u32>,
		MaxAdditionalParamsLen: Get<u32>,
	>
	PhaseResult<
		AccountId,
		BlockNumber,
		MaxParticipants,
		MaxKeyLen,
		MaxDataLen,
		MaxSignatureLen,
		MaxSubmissionLen,
		MaxProofLen,
		MaxAdditionalParamsLen,
	> where
	AccountId: Clone,
{
	pub fn participants(&self) -> Option<BoundedVec<AccountId, MaxParticipants>> {
		match &self.job_type {
			JobType::DKGTSSPhaseOne(x) => Some(x.participants.clone()),
			JobType::ZkSaaSPhaseOne(x) => Some(x.participants.clone()),
			_ => None,
		}
	}

	pub fn threshold(&self) -> Option<u8> {
		match &self.job_type {
			JobType::DKGTSSPhaseOne(x) => Some(x.threshold),
			_ => None,
		}
	}
}

/// Represents different types of validator offences.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum ValidatorOffence {
	/// The validator has been inactive.
	Inactivity,

	/// The validator has signed an invalid message.
	InvalidSignatureSubmitted,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct RpcResponseJobsData<
	AccountId,
	BlockNumber,
	MaxParticipants: Get<u32> + Clone,
	MaxSubmissionLen: Get<u32>,
	MaxAdditionalParamsLen: Get<u32>,
> {
	/// The job id of the job
	pub job_id: JobId,

	/// The type of the job submission.
	pub job_type: JobType<AccountId, MaxParticipants, MaxSubmissionLen, MaxAdditionalParamsLen>,

	/// Represents the maximum allowed submission time for a job result.
	/// Once this time has passed, the result cannot be submitted.
	pub expiry: BlockNumber,

	/// The time-to-live (TTL) for the job, which determines the maximum allowed time for this job
	/// to be available. After the TTL expires, the job can no longer be used.
	pub ttl: BlockNumber,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum JobResult<
	MaxParticipants: Get<u32>,
	MaxKeyLen: Get<u32>,
	MaxSignatureLen: Get<u32>,
	MaxDataLen: Get<u32>,
	MaxProofLen: Get<u32>,
	MaxAdditionalParamsLen: Get<u32>,
> {
	DKGPhaseOne(DKGTSSKeySubmissionResult<MaxKeyLen, MaxParticipants, MaxSignatureLen>),
	DKGPhaseTwo(
		DKGTSSSignatureResult<MaxDataLen, MaxKeyLen, MaxSignatureLen, MaxAdditionalParamsLen>,
	),
	DKGPhaseThree(DKGTSSKeyRefreshResult),
	DKGPhaseFour(DKGTSSKeyRotationResult<MaxKeyLen, MaxSignatureLen, MaxAdditionalParamsLen>),
	ZkSaaSPhaseOne(ZkSaaSCircuitResult<MaxParticipants>),
	ZkSaaSPhaseTwo(ZkSaaSProofResult<MaxProofLen>),
}

/// Represents different types of validator offences.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum ValidatorOffenceType {
	/// The validator has been inactive.
	Inactivity,
	/// Submitted invalid signature.
	InvalidSignatureSubmitted,
	/// Rejected valid action.
	RejectedValidAction,
	/// Approved invalid action.
	ApprovedInvalidAction,
}

/// An offence report that is filed if a validator misbehaves.
#[derive(Clone, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct ReportRestakerOffence<Offender> {
	/// The current session index in which offence is reported.
	pub session_index: u32,
	/// The size of the validator set in current session/era.
	pub validator_set_count: u32,
	/// The type of offence
	pub offence_type: ValidatorOffenceType,
	/// Role type against which offence is reported.
	pub role_type: RoleType,
	/// Offenders
	pub offenders: Vec<Offender>,
}
