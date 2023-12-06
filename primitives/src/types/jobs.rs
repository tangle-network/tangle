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
	DKG(DKGJobType<AccountId>),
	/// DKG Signature job type.
	DKGSignature(DKGSignatureJobType),
	/// (zk-SNARK) Create Circuit job type.
	ZkSaasCircuit(ZkSaasCircuitJobType<AccountId>),
	/// (zk-SNARK) Create Proof job type.
	ZkSaasProve(ZkSaasProveJobType),
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

/// Enum representing different types of circuits and snark schemes.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ZkSaasSystem {
	Groth16(Groth16System),
}

/// Represents the Groth16 system for zk-SNARKs.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Groth16System {
	/// R1CS circuit file.
	pub circuit: HyperData,
	/// Proving key file.
	pub proving_key: HyperData,
	/// Verifying key bytes
	pub verifying_key: Vec<u8>,
	/// Circom WASM file.
	pub wasm: HyperData,
}

/// Represents ZK-SNARK proving request
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ZkSaasProveRequest {
	/// Groth16 proving request
	Groth16(Groth16ProveRequest),
}

/// Represents Groth16 proving request
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Groth16ProveRequest {
	/// Public input that are used during the verification
	pub public_input: Vec<u8>,
	/// `a` is the full assignment (full_assginment[0] is 1)
	/// a = full_assginment[1..]
	/// Each element contains a PSS of the witness
	pub a_shares: Vec<HyperData>,
	/// `ax` is the auxiliary input
	/// ax = full_assginment[num_inputs..]
	/// Each element contains a PSS of the auxiliary input
	pub ax: Vec<HyperData>,
	/// PSS of the QAP polynomials
	pub qap_shares: Vec<QAPShare>,
}

/// Represents QAP share
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct QAPShare {
	pub a: HyperData,
	pub b: HyperData,
	pub c: HyperData,
}

impl<AccountId> JobType<AccountId> {
	/// Checks if the job type is a phase one job.
	pub fn is_phase_one(&self) -> bool {
		use crate::jobs::JobType::*;
		if matches!(self, DKG(_) | ZkSaasCircuit(_)) {
			return true
		}
		false
	}

	/// Gets the participants for the job type, if applicable.
	pub fn get_participants(self) -> Option<Vec<AccountId>> {
		use crate::jobs::JobType::*;
		match self {
			DKG(info) => Some(info.participants),
			ZkSaasCircuit(info) => Some(info.participants),
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
			JobType::ZkSaasCircuit(_) => JobKey::ZkSaasCircuit,
			JobType::DKGSignature(_) => JobKey::DKGSignature,
			JobType::ZkSaasProve(_) => JobKey::ZkSaasProve,
		}
	}

	/// Gets the job key associated with the previous phase job type.
	pub fn get_previous_phase_job_key(&self) -> Option<JobKey> {
		match self {
			JobType::DKGSignature(_) => Some(JobKey::DKG),
			JobType::ZkSaasProve(_) => Some(JobKey::ZkSaasCircuit),
			_ => None,
		}
	}

	/// Performs a basic sanity check on the job type.
	///
	/// This function is intended for simple checks and may need improvement in the future.
	pub fn sanity_check(&self) -> bool {
		match self {
			JobType::DKG(info) => info.participants.len() > info.threshold.into(),
			JobType::ZkSaasCircuit(info) => !info.participants.is_empty(),
			_ => true,
		}
	}

	/// Gets the phase one ID for phase two jobs, if applicable.
	pub fn get_phase_one_id(&self) -> Option<u32> {
		use crate::jobs::JobType::*;
		match self {
			DKGSignature(info) => Some(info.phase_one_id),
			ZkSaasProve(info) => Some(info.phase_one_id),
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
pub struct ZkSaasCircuitJobType<AccountId> {
	/// List of participants' account IDs.
	pub participants: Vec<AccountId>,
	/// ZK-SNARK Proving system
	pub system: ZkSaasSystem,
}

/// Represents the (zk-SNARK) Phase Two job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ZkSaasProveJobType {
	/// The phase one ID.
	pub phase_one_id: u32,

	/// ZK-SNARK Proving request
	pub request: ZkSaasProveRequest,
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
	ZkSaasCircuit,
	/// (zk-SNARK) Create Proof job type.
	ZkSaasProve,
}

impl JobKey {
	/// Returns role assigned with the job.
	pub fn get_role_type(&self) -> RoleType {
		match self {
			JobKey::DKG => RoleType::Tss,
			JobKey::DKGSignature => RoleType::Tss,
			JobKey::ZkSaasCircuit => RoleType::ZkSaas,
			JobKey::ZkSaasProve => RoleType::ZkSaas,
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
	/// The type of the job submission.
	pub job_type: JobType<AccountId>,
}

impl<AccountId, BlockNumber> PhaseOneResult<AccountId, BlockNumber>
where
	AccountId: Clone,
{
	pub fn participants(&self) -> Option<Vec<AccountId>> {
		match &self.job_type {
			JobType::DKG(x) => Some(x.participants.clone()),
			JobType::ZkSaasCircuit(x) => Some(x.participants.clone()),
			_ => None,
		}
	}

	pub fn threshold(&self) -> Option<u8> {
		match &self.job_type {
			JobType::DKG(x) => Some(x.threshold),
			_ => None,
		}
	}
}

/// Represents different types of validator offences.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum ValidatorOffence {
	/// The validator has been inactive.
	Inactivity,

	/// The validator has committed duplicate signing.
	Equivocation,
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

	ZkSaasCircuit(ZkSaasCircuitResult),

	ZkSaasProve(ZkSaasProofResult),
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
pub struct ZkSaasCircuitResult {
	/// The job id of the job (circuit)
	pub job_id: JobId,

	/// List of participants' public keys
	pub participants: Vec<ecdsa::Public>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ZkSaasProofResult {
	Circom(CircomProofResult),
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CircomProofResult {
	pub proof: Vec<u8>,
}
