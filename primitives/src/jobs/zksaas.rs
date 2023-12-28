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

use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{RuntimeDebug};
use sp_std::vec::Vec;

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