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

use crate::{jobs::JobId, roles::ZeroKnowledgeRoleType};
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{ecdsa, RuntimeDebug};
use sp_runtime::traits::Get;

/// Enum representing different types of data sources.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum HyperData<MaxSubmissionLen: Get<u32>> {
	/// Raw data, stored on-chain.
	///
	/// Only use this for small files.
	Raw(BoundedVec<u8, MaxSubmissionLen>),
	/// IPFS CID. The CID is stored on-chain.
	/// The actual data is stored off-chain.
	IPFS(BoundedVec<u8, MaxSubmissionLen>),
	/// HTTP URL. The URL is stored on-chain.
	/// The actual data is stored off-chain.
	/// The URL is expected to be accessible via HTTP GET.
	HTTP(BoundedVec<u8, MaxSubmissionLen>),
}

/// Enum representing different types of circuits and snark schemes.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ZkSaaSSystem<MaxSubmissionLen: Get<u32>> {
	Groth16(Groth16System<MaxSubmissionLen>),
}

/// Represents the Groth16 system for zk-SNARKs.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Groth16System<MaxSubmissionLen: Get<u32>> {
	/// R1CS circuit file.
	pub circuit: HyperData<MaxSubmissionLen>,
	/// Number of inputs
	pub num_inputs: u64,
	/// Number of constraints
	pub num_constraints: u64,
	/// Proving key file.
	pub proving_key: HyperData<MaxSubmissionLen>,
	/// Verifying key bytes
	pub verifying_key: BoundedVec<u8, MaxSubmissionLen>,
	/// Circom WASM file.
	pub wasm: HyperData<MaxSubmissionLen>,
}

/// Represents the (zk-SNARK) Phase One job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ZkSaaSPhaseOneJobType<
	AccountId,
	MaxParticipants: Get<u32> + Clone,
	MaxSubmissionLen: Get<u32>,
> {
	/// List of participants' account IDs.
	pub participants: BoundedVec<AccountId, MaxParticipants>,
	/// the caller permitted to use this result later
	pub permitted_caller: Option<AccountId>,
	/// ZK-SNARK Proving system
	pub system: ZkSaaSSystem<MaxSubmissionLen>,
	/// The role type of the job
	pub role_type: ZeroKnowledgeRoleType,
}

/// Represents the (zk-SNARK) Phase Two job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ZkSaaSPhaseTwoJobType<MaxSubmissionLen: Get<u32>> {
	/// The phase one ID.
	pub phase_one_id: JobId,
	/// ZK-SNARK Proving request
	pub request: ZkSaaSPhaseTwoRequest<MaxSubmissionLen>,
	/// The role type of the job
	pub role_type: ZeroKnowledgeRoleType,
}

/// Represents ZK-SNARK proving request
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ZkSaaSPhaseTwoRequest<MaxSubmissionLen: Get<u32>> {
	/// Groth16 proving request
	Groth16(Groth16ProveRequest<MaxSubmissionLen>),
}

/// Represents Groth16 proving request
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Groth16ProveRequest<MaxSubmissionLen: Get<u32>> {
	/// Public input that are used during the verification
	pub public_input: BoundedVec<u8, MaxSubmissionLen>,
	/// `a` is the full assignment (full_assginment[0] is 1)
	/// a = full_assginment[1..]
	/// Each element contains a PSS of the witness
	pub a_shares: BoundedVec<HyperData<MaxSubmissionLen>, MaxSubmissionLen>,
	/// `ax` is the auxiliary input
	/// ax = full_assginment[num_inputs..]
	/// Each element contains a PSS of the auxiliary input
	pub ax_shares: BoundedVec<HyperData<MaxSubmissionLen>, MaxSubmissionLen>,
	/// PSS of the QAP polynomials
	pub qap_shares: BoundedVec<QAPShare<MaxSubmissionLen>, MaxSubmissionLen>,
}

/// Represents QAP share
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct QAPShare<MaxSubmissionLen: Get<u32>> {
	pub a: HyperData<MaxSubmissionLen>,
	pub b: HyperData<MaxSubmissionLen>,
	pub c: HyperData<MaxSubmissionLen>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ZkSaaSCircuitResult<MaxParticipants: Get<u32>> {
	/// The job id of the job (circuit)
	pub job_id: JobId,

	/// List of participants' public keys
	pub participants: BoundedVec<ecdsa::Public, MaxParticipants>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ZkSaaSProofResult<MaxProofLen: Get<u32>> {
	Arkworks(ArkworksProofResult<MaxProofLen>),
	Circom(CircomProofResult<MaxProofLen>),
}

impl<MaxProofLen: Get<u32>> ZkSaaSProofResult<MaxProofLen> {
	pub fn proof(&self) -> BoundedVec<u8, MaxProofLen> {
		match self {
			ZkSaaSProofResult::Arkworks(x) => x.proof.clone(),
			ZkSaaSProofResult::Circom(x) => x.proof.clone(),
		}
	}
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CircomProofResult<MaxProofLen: Get<u32>> {
	pub proof: BoundedVec<u8, MaxProofLen>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ArkworksProofResult<MaxProofLen: Get<u32>> {
	pub proof: BoundedVec<u8, MaxProofLen>,
}
