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
use sp_core::{ecdsa, RuntimeDebug};
use sp_std::vec::Vec;

pub type JobId = u32;

pub mod traits;
pub mod tss;
pub mod zksaas;

pub use tss::*;
pub use zksaas::*;

/// Represents a job submission with specified `AccountId` and `BlockNumber`.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct JobSubmission<AccountId, BlockNumber> {
	/// The expiry block number.
	pub expiry: BlockNumber,

	/// The type of the job submission.
	pub job_type: JobType<AccountId>,
}

/// Represents a job info
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct JobInfo<AccountId, BlockNumber, Balance> {
	/// The caller that requested the job
	pub owner: AccountId,

	/// The expiry block number.
	pub expiry: BlockNumber,

	/// The type of the job submission.
	pub job_type: JobType<AccountId>,

	/// The fee taken for the job
	pub fee: Balance,
}

/// Represents a job with its result.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct JobWithResult<AccountId> {
	/// Current Job type
	pub job_type: JobType<AccountId>,
	/// Phase one job type if any.
	///
	/// None if this job is a phase one job.
	pub phase_one_job_type: Option<JobType<AccountId>>,
	/// Current job result
	pub result: JobResult,
}

/// Enum representing different types of jobs.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum JobType<AccountId> {
	/// Distributed Key Generation (DKG) job type.
	DKGTSSPhaseOne(DKGTSSPhaseOneJobType<AccountId>),
	/// DKG Signature job type.
	DKGTSSPhaseTwo(DKGTSSPhaseTwoJobType),
	/// (zk-SNARK) Create Circuit job type.
	ZkSaaSPhaseOne(ZkSaaSPhaseOneJobType<AccountId>),
	/// (zk-SNARK) Create Proof job type.
	ZkSaaSPhaseTwo(ZkSaaSPhaseTwoJobType),
}

/// Enum representing different types of data sources.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum HyperData {
	/// Raw data, stored on-chain.
	///
	/// Only use this for small files.
	Raw(Vec<u8>),
	/// IPFS CID. The CID is stored on-chain.
	/// The actual data is stored off-chain.
	IPFS(Vec<u8>),
	/// HTTP URL. The URL is stored on-chain.
	/// The actual data is stored off-chain.
	/// The URL is expected to be accessible via HTTP GET.
	HTTP(Vec<u8>),
}

impl<AccountId> JobType<AccountId> {
	/// Checks if the job type is a phase one job.
	pub fn is_phase_one(&self) -> bool {
		use crate::jobs::JobType::*;
		if matches!(self, DKGTSSPhaseOne(_) | ZkSaaSPhaseOne(_)) {
			return true
		}
		false
	}

	/// Gets the participants for the job type, if applicable.
	pub fn get_participants(self) -> Option<Vec<AccountId>> {
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

	/// Gets the job key associated with the job type.
	pub fn get_job_key(&self) -> JobKey {
		match self {
			JobType::DKGTSSPhaseOne(_) => JobKey::DKG,
			JobType::ZkSaaSPhaseOne(_) => JobKey::ZkSaaSCircuit,
			JobType::DKGTSSPhaseTwo(_) => JobKey::DKGSignature,
			JobType::ZkSaaSPhaseTwo(_) => JobKey::ZkSaaSProve,
		}
	}

	/// Gets the job key associated with the previous phase job type.
	pub fn get_previous_phase_job_key(&self) -> Option<JobKey> {
		match self {
			JobType::DKGTSSPhaseTwo(_) => Some(JobKey::DKG),
			JobType::ZkSaaSPhaseTwo(_) => Some(JobKey::ZkSaaSCircuit),
			_ => None,
		}
	}

	/// Performs a basic sanity check on the job type.
	///
	/// This function is intended for simple checks and may need improvement in the future.
	pub fn sanity_check(&self) -> bool {
		match self {
			JobType::DKGTSSPhaseOne(info) => info.participants.len() > info.threshold.into(),
			JobType::ZkSaaSPhaseOne(info) => !info.participants.is_empty(),
			_ => true,
		}
	}

	/// Gets the phase one ID for phase two jobs, if applicable.
	pub fn get_phase_one_id(&self) -> Option<u32> {
		use crate::jobs::JobType::*;
		match self {
			DKGTSSPhaseTwo(info) => Some(info.phase_one_id),
			ZkSaaSPhaseTwo(info) => Some(info.phase_one_id),
			_ => None,
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

/// Enum representing different types of job keys.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, Copy)]
pub enum JobKey {
	/// Distributed Key Generation (DKG) job type.
	DKG,
	/// DKG Signature job type.
	DKGSignature,
	/// (zk-SNARK) Create Circuit job type.
	ZkSaaSCircuit,
	/// (zk-SNARK) Create Proof job type.
	ZkSaaSProve,
}

impl JobKey {
	/// Returns role assigned with the job.
	pub fn get_role_type(&self) -> RoleType {
		match self {
			JobKey::DKG => RoleType::Tss,
			JobKey::DKGSignature => RoleType::Tss,
			JobKey::ZkSaaSCircuit => RoleType::ZkSaaS,
			JobKey::ZkSaaSProve => RoleType::ZkSaaS,
		}
	}
}

/// Represents a job submission with specified `AccountId` and `BlockNumber`.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct PhaseOneResult<AccountId, BlockNumber> {
	/// The owner's account ID.
	pub owner: AccountId,
	/// The expiry block number.
	pub expiry: BlockNumber,
	/// The type of the job submission.
	pub result: Vec<u8>,
	/// permitted caller to use this result
	pub permitted_caller: Option<AccountId>,
	/// Key type if applicable
	pub key_type: Option<DkgKeyType>,
	/// The type of the job submission.
	pub job_type: JobType<AccountId>,
}

impl<AccountId, BlockNumber> PhaseOneResult<AccountId, BlockNumber>
where
	AccountId: Clone,
{
	pub fn participants(&self) -> Option<Vec<AccountId>> {
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
pub struct RpcResponseJobsData<AccountId> {
	/// The job id of the job
	pub job_id: JobId,

	/// The type of the job submission.
	pub job_type: JobType<AccountId>,

	/// (Optional) List of participants' account IDs.
	pub participants: Option<Vec<AccountId>>,

	/// threshold if any for the original set
	pub threshold: Option<u8>,

	/// previous phase key if any
	pub key: Option<Vec<u8>>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct RpcResponsePhaseOneResult<AccountId> {
	/// The owner's account ID.
	pub owner: AccountId,
	/// The type of the job result.
	pub result: Vec<u8>,
	/// permitted caller to use this result
	pub permitted_caller: Option<AccountId>,
	/// Key type if applicable
	pub key_type: Option<DkgKeyType>,
	/// The type of the job submission.
	pub job_type: JobType<AccountId>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum JobResult {
	DKGPhaseOne(DKGTSSResult),

	DKGPhaseTwo(DKGTSSSignatureResult),

	ZkSaaSPhaseOne(ZkSaaSCircuitResult),

	ZkSaaSPhaseTwo(ZkSaaSProofResult),
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
pub struct ReportValidatorOffence<Offender> {
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
