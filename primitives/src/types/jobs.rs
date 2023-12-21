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

/// Enum representing different types of circuits and snark schemes.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ZkSaaSSystem {
	Groth16(Groth16System),
}

/// Represents the Groth16 system for zk-SNARKs.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Groth16System {
	/// R1CS circuit file.
	pub circuit: HyperData,
	/// Number of inputs
	pub num_inputs: u64,
	/// Number of constraints
	pub num_constraints: u64,
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
pub enum ZkSaaSPhaseTwoRequest {
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
	pub ax_shares: Vec<HyperData>,
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

/// Represents the Distributed Key Generation (DKG) job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGTSSPhaseOneJobType<AccountId> {
	/// List of participants' account IDs.
	pub participants: Vec<AccountId>,

	/// The threshold value for the DKG.
	pub threshold: u8,

	/// the caller permitted to use this result later
	pub permitted_caller: Option<AccountId>,

	/// the key type to be used
	pub key_type: DkgKeyType,
}

/// Represents the DKG Signature job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGTSSPhaseTwoJobType {
	/// The phase one ID.
	pub phase_one_id: u32,

	/// The submission data as a vector of bytes.
	pub submission: Vec<u8>,
}

/// Represents the (zk-SNARK) Phase One job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ZkSaaSPhaseOneJobType<AccountId> {
	/// List of participants' account IDs.
	pub participants: Vec<AccountId>,
	/// the caller permitted to use this result later
	pub permitted_caller: Option<AccountId>,
	/// ZK-SNARK Proving system
	pub system: ZkSaaSSystem,
}

/// Represents the (zk-SNARK) Phase Two job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ZkSaaSPhaseTwoJobType {
	/// The phase one ID.
	pub phase_one_id: u32,

	/// ZK-SNARK Proving request
	pub request: ZkSaaSPhaseTwoRequest,
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
	DKGPhaseOne(DKGResult),

	DKGPhaseTwo(DKGSignatureResult),

	ZkSaaSPhaseOne(ZkSaaSCircuitResult),

	ZkSaaSPhaseTwo(ZkSaaSProofResult),
}

pub type Signatures = Vec<Vec<u8>>;

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGResult {
	/// Key type to use for DKG
	pub key_type: DkgKeyType,

	/// Submitted key
	pub key: Vec<u8>,

	/// List of participants' public keys
	pub participants: Vec<Vec<u8>>,

	/// List of participants' signatures
	pub signatures: Signatures,

	/// threshold needed to confirm the result
	pub threshold: u8,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGSignatureResult {
	/// Key type to use for DKG
	pub key_type: DkgKeyType,

	/// The input data
	pub data: Vec<u8>,

	/// The signature to verify
	pub signature: Vec<u8>,

	/// The expected key for the signature
	pub signing_key: Vec<u8>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ZkSaaSCircuitResult {
	/// The job id of the job (circuit)
	pub job_id: JobId,

	/// List of participants' public keys
	pub participants: Vec<ecdsa::Public>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ZkSaaSProofResult {
	Arkworks(ArkworksProofResult),
	Circom(CircomProofResult),
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CircomProofResult {
	pub proof: Vec<u8>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ArkworksProofResult {
	pub proof: Vec<u8>,
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

/// Possible key types for DKG
#[derive(Clone, RuntimeDebug, TypeInfo, PartialEq, Eq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DkgKeyType {
	/// Elliptic Curve Digital Signature Algorithm (ECDSA) key type.
	#[default]
	Ecdsa,

	/// Schnorr signature key type.
	Schnorr,
}
