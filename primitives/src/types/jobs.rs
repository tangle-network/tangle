// This file is part of Tangle.
// Copyright (C) 2022-2023 Webb Technologies Inc.
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
use frame_support::{dispatch::Vec, pallet_prelude::*, RuntimeDebug};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::ecdsa;

pub type JobId = u32;

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

/// Enum representing different types of jobs.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum JobType<AccountId> {
	/// Distributed Key Generation (DKG) job type.
	DKG(DKGJobType<AccountId>),
	/// DKG Signature job type.
	DKGSignature(DKGSignatureJobType),
	/// (zk-SNARK) Phase One job type.
	ZkSaasPhaseOne(ZkSaasPhaseOneJobType<AccountId>),
	/// (zk-SNARK) Phase Two job type.
	ZkSaasPhaseTwo(ZkSaasPhaseTwoJobType),
}

impl<AccountId> JobType<AccountId> {
	/// Checks if the job type is a phase one job.
	pub fn is_phase_one(&self) -> bool {
		use crate::jobs::JobType::*;
		if matches!(self, DKG(_) | ZkSaasPhaseOne(_)) {
			return true
		}
		false
	}

	/// Gets the participants for the job type, if applicable.
	pub fn get_participants(self) -> Option<Vec<AccountId>> {
		use crate::jobs::JobType::*;
		match self {
			DKG(info) => Some(info.participants),
			ZkSaasPhaseOne(info) => Some(info.participants),
			_ => None,
		}
	}

	/// Gets the threshold value for the job type, if applicable.
	pub fn get_threshold(self) -> Option<u8> {
		use crate::jobs::JobType::*;
		match self {
			DKG(info) => Some(info.threshold),
			_ => None,
		}
	}

	/// Gets the job key associated with the job type.
	pub fn get_job_key(&self) -> JobKey {
		match self {
			JobType::DKG(_) => JobKey::DKG,
			JobType::ZkSaasPhaseOne(_) => JobKey::ZkSaasPhaseOne,
			JobType::DKGSignature(_) => JobKey::DKGSignature,
			JobType::ZkSaasPhaseTwo(_) => JobKey::ZkSaasPhaseTwo,
		}
	}

	/// Gets the job key associated with the previous phase job type.
	pub fn get_previous_phase_job_key(&self) -> Option<JobKey> {
		match self {
			JobType::DKGSignature(_) => Some(JobKey::DKG),
			JobType::ZkSaasPhaseTwo(_) => Some(JobKey::ZkSaasPhaseOne),
			_ => None,
		}
	}

	/// Performs a basic sanity check on the job type.
	///
	/// This function is intended for simple checks and may need improvement in the future.
	pub fn sanity_check(&self) -> bool {
		match self {
			JobType::DKG(info) =>
				if info.participants.len() > info.threshold.into() {
					return true
				},
			_ => return true,
		}

		false
	}

	/// Gets the phase one ID for phase two jobs, if applicable.
	pub fn get_phase_one_id(self) -> Option<u32> {
		use crate::jobs::JobType::*;
		match self {
			DKGSignature(info) => Some(info.phase_one_id),
			ZkSaasPhaseTwo(info) => Some(info.phase_one_id),
			_ => None,
		}
	}

	pub fn get_permitted_caller(self) -> Option<AccountId> {
		use crate::jobs::JobType::*;
		match self {
			DKG(info) => info.permitted_caller,
			ZkSaasPhaseOne(info) => info.permitted_caller,
			_ => None,
		}
	}
}

/// Represents the Distributed Key Generation (DKG) job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGJobType<AccountId> {
	/// List of participants' account IDs.
	pub participants: Vec<AccountId>,

	/// The threshold value for the DKG.
	pub threshold: u8,

	/// the caller permitted to use this result later
	pub permitted_caller: Option<AccountId>,
}

/// Represents the DKG Signature job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGSignatureJobType {
	/// The phase one ID.
	pub phase_one_id: u32,

	/// The submission data as a vector of bytes.
	pub submission: Vec<u8>,
}

/// Represents the (zk-SNARK) Phase One job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ZkSaasPhaseOneJobType<AccountId> {
	/// List of participants' account IDs.
	pub participants: Vec<AccountId>,
	/// the caller permitted to use this result later
	pub permitted_caller: Option<AccountId>,
}

/// Represents the (zk-SNARK) Phase Two job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ZkSaasPhaseTwoJobType {
	/// The phase one ID.
	pub phase_one_id: u32,

	/// The submission data as a vector of bytes.
	pub submission: Vec<u8>,
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
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum JobKey {
	/// Distributed Key Generation (DKG) job type.
	DKG,
	/// DKG Signature job type.
	DKGSignature,
	/// (zk-SNARK) Phase One job type.
	ZkSaasPhaseOne,
	/// (zk-SNARK) Phase Two job type.
	ZkSaasPhaseTwo,
}

impl JobKey {
	/// Returns role assigned with the job.
	pub fn get_role_type(&self) -> RoleType {
		match self {
			JobKey::DKG => RoleType::Tss,
			JobKey::DKGSignature => RoleType::Tss,
			JobKey::ZkSaasPhaseOne => RoleType::ZkSaas,
			JobKey::ZkSaasPhaseTwo => RoleType::ZkSaas,
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

	/// List of participants' account IDs.
	pub participants: Vec<AccountId>,

	/// threshold if any for the original set
	pub threshold: Option<u8>,

	/// permitted caller to use this result
	pub permitted_caller: Option<AccountId>,
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
pub enum JobResult {
	DKG(DKGResult),

	DKGSignature(DKGSignatureResult),

	ZkSaasPhaseOne(ZkSaasPhaseOneResult),

	ZkSaasPhaseTwo(ZkSaasPhaseTwoResult),
}

pub type KeysAndSignatures = Vec<(Vec<u8>, Vec<u8>)>;

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGResult {
	/// Submitted key
	pub key: Vec<u8>,

	/// List of participants' public keys
	pub participants: Vec<ecdsa::Public>,

	/// List of participants' keys and signatures
	pub keys_and_signatures: KeysAndSignatures,

	/// threshold needed to confirm the result
	pub threshold: u8,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGSignatureResult {
	/// The input data
	pub data: Vec<u8>,

	/// The signature to verify
	pub signature: Vec<u8>,

	/// The expected key for the signature
	pub signing_key: Vec<u8>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ZkSaasPhaseOneResult {
	/// The job id of the job
	pub job_id: JobId,

	/// List of participants' public keys
	pub participants: Vec<Vec<u8>>,

	/// The data to verify
	pub data: Vec<u8>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ZkSaasPhaseTwoResult {
	/// The data to verify
	pub data: Vec<u8>,

	/// The expected key for the signature
	pub signing_key: Vec<u8>,
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
