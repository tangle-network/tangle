// This file is part of Webb.
// Copyright (C) 2022 Webb Technologies Inc.
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
use super::*;
use frame_support::pallet_prelude::*;
use frame_support::RuntimeDebug;

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

impl <AccountId> JobType<AccountId> {
    /// Checks if the job type is a phase one job.
    pub fn is_phase_one(&self) -> bool {
        use crate::jobs::JobType::*;
        match self {
            DKG(_) => true,
            ZkSaasPhaseOne(_) => true,
            _ => false
        }
    }

    /// Gets the participants for the job type, if applicable.
    pub fn get_participants(self) -> Option<Vec<AccountId>> {
        use crate::jobs::JobType::*;
        match self {
            DKG(info) => Some(info.participants),
            ZkSaasPhaseOne(info) => Some(info.participants),
            _ => None
        }
    }

    /// Gets the threshold value for the job type, if applicable.
    pub fn get_threshold(self) -> Option<u8> {
        use crate::jobs::JobType::*;
        match self {
            DKG(info) => Some(info.threshold),
            _ => None
        }
    }

    /// Gets the job key associated with the job type.
    pub fn get_job_key(&self) -> JobKey {
        match self {
            JobType::DKG(_) => JobKey::DKG,
            JobType::ZkSaasPhaseOne(_) => JobKey::ZkSaasPhaseOne,
            JobType::DKGSignature(_) => JobKey::DKGSignature,
            JobType::ZkSaasPhaseTwo(_) =>JobKey::ZkSaasPhaseTwo
        }
    }

        /// Gets the job key associated with the previous phase job type.
        pub fn get_previous_phase_job_key(&self) -> Option<JobKey> {
            match self {
                JobType::DKGSignature(_) => Some(JobKey::DKG),
                JobType::ZkSaasPhaseTwo(_) => Some(JobKey::ZkSaasPhaseOne),
                _ => None
            }
        }

    /// Performs a basic sanity check on the job type.
    ///
    /// This function is intended for simple checks and may need improvement in the future.
    pub fn sanity_check(&self) -> bool {
        match self {
            JobType::DKG(info) => {
                if info.participants.len() > info.threshold.into() {
                    return true;
                }
            },
            _ => { return true ;}
        }

        return false
    }

    /// Gets the phase one ID for phase two jobs, if applicable.
    pub fn get_phase_one_id(self) -> Option<u32> {
        use crate::jobs::JobType::*;
        match self {
            DKGSignature(info) => Some(info.phase_one_id),
            ZkSaasPhaseTwo(info) => Some(info.phase_one_id),
            _ => None
        }
    }
}


/// Represents the Distributed Key Generation (DKG) job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct DKGJobType<AccountId> {
    /// List of participants' account IDs.
    pub participants: Vec<AccountId>,

    /// The threshold value for the DKG.
    pub threshold: u8,
}

/// Represents the DKG Signature job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct DKGSignatureJobType {
    /// The phase one ID.
    pub phase_one_id: u32,

    /// The submission data as a vector of bytes.
    pub submission: Vec<u8>,
}

/// Represents the (zk-SNARK) Phase One job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct ZkSaasPhaseOneJobType<AccountId> {
    /// List of participants' account IDs.
    pub participants: Vec<AccountId>,
}

/// Represents the (zk-SNARK) Phase Two job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct ZkSaasPhaseTwoJobType {
    /// The phase one ID.
    pub phase_one_id: u32,

    /// The submission data as a vector of bytes.
    pub submission: Vec<u8>,
}

/// Enum representing different states of a job.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
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
    pub threshold: Option<u8>
}