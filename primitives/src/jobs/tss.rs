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

use crate::roles::ThresholdSignatureRoleType;
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;
use sp_runtime::traits::Get;

use super::JobId;

/// Represents the Distributed Key Generation (DKG) job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGTSSPhaseOneJobType<AccountId, MaxParticipants: Get<u32> + Clone> {
	/// List of participants' account IDs.
	pub participants: BoundedVec<AccountId, MaxParticipants>,

	/// The threshold value for the DKG.
	pub threshold: u8,

	/// The caller permitted to use this result later
	pub permitted_caller: Option<AccountId>,

	/// The role type to be used
	pub role_type: ThresholdSignatureRoleType,
}

/// Represents the DKG Signature job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGTSSPhaseTwoJobType<MaxSubmissionLen: Get<u32>> {
	/// The phase one ID.
	pub phase_one_id: JobId,

	/// The submission data as a vector of bytes.
	pub submission: BoundedVec<u8, MaxSubmissionLen>,

	/// The role type to be used
	pub role_type: ThresholdSignatureRoleType,
}

/// Represents the DKG Key Refresh job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGTSSPhaseThreeJobType {
	/// The phase one ID.
	pub phase_one_id: JobId,

	/// The role type to be used
	pub role_type: ThresholdSignatureRoleType,
}

/// Represents the DKG Key Rotation job type.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGTSSPhaseFourJobType {
	/// The phase one ID.
	pub phase_one_id: JobId,
	/// The new phase one ID.
	/// That will be used for the rotation.
	pub new_phase_one_id: JobId,
	/// The role type to be used
	pub role_type: ThresholdSignatureRoleType,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGTSSKeySubmissionResult<
	MaxKeyLen: Get<u32>,
	MaxParticipants: Get<u32>,
	MaxSignatureLen: Get<u32>,
> {
	/// Signature type of the DKG
	pub signature_type: DigitalSignatureType,

	/// Submitted key
	pub key: BoundedVec<u8, MaxKeyLen>,

	/// List of participants' public keys
	pub participants: BoundedVec<BoundedVec<u8, MaxKeyLen>, MaxParticipants>,

	/// List of participants' signatures
	pub signatures: BoundedVec<BoundedVec<u8, MaxSignatureLen>, MaxParticipants>,

	/// threshold needed to confirm the result
	pub threshold: u8,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGTSSSignatureResult<
	MaxDataLen: Get<u32>,
	MaxKeyLen: Get<u32>,
	MaxSignatureLen: Get<u32>,
> {
	/// Signature type to use for DKG
	pub signature_type: DigitalSignatureType,

	/// The input data
	pub data: BoundedVec<u8, MaxDataLen>,

	/// The signature to verify
	pub signature: BoundedVec<u8, MaxSignatureLen>,

	/// The expected key for the signature
	pub signing_key: BoundedVec<u8, MaxKeyLen>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGTSSKeyRefreshResult {
	/// Signature type to use for DKG
	pub signature_type: DigitalSignatureType,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGTSSKeyRotationResult<MaxKeyLen: Get<u32>, MaxSignatureLen: Get<u32>> {
	/// The phase one ID.
	pub phase_one_id: JobId,
	/// The new phase one ID.
	/// That will be used for the rotation.
	pub new_phase_one_id: JobId,
	/// Key from the new phase 1.
	pub new_key: BoundedVec<u8, MaxKeyLen>,
	/// Current key (from phase 1).
	pub key: BoundedVec<u8, MaxKeyLen>,
	/// The signature of signing the new key with the current key.
	pub signature: BoundedVec<u8, MaxSignatureLen>,
	/// Signature type of the DKG
	pub signature_type: DigitalSignatureType,
}

/// Possible key types for DKG
#[derive(Clone, RuntimeDebug, TypeInfo, PartialEq, Eq, Encode, Decode, Default, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DigitalSignatureType {
	/// Elliptic Curve Digital Signature Algorithm (ECDSA) key type.
	#[default]
	Ecdsa,

	/// Schnorr signature type over the P256 curve.
	SchnorrP256,

	/// Schnorr signature type of the P384 curve.
	SchnorrP384,

	/// Schnorr signature type over the Secp256k1 curve.
	SchnorrSecp256k1,

	/// Schnorr signature type of the Secp256k1 curve w/ Taproot compatibility.
	SchnorrSecp256k1Taproot,

	/// Schnorr signature type over the sr25519 curve.
	SchnorrSr25519,

	/// Schnorr signature type over the Ristretto255 curve / sr25519.
	SchnorrRistretto255,

	/// Schnorr signature type over the JubJub curve.
	SchnorrRedJubJub,

	/// Schnorr signature type over the Ed25519 curve.
	SchnorrEd25519,

	/// Schnorr signature type over the Ed448 curve.
	SchnorrEd448,

	/// BLS 381 signature type.
	Bls381,
}
