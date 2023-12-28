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

pub type Signatures = Vec<Vec<u8>>;

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DKGTSSResult {
	/// Signature type of the DKG
	pub signature_type: DigitalSignatureType,

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
pub struct DKGTSSSignatureResult {
	/// Signature type to use for DKG
	pub signature_type: DigitalSignatureType,

	/// The input data
	pub data: Vec<u8>,

	/// The signature to verify
	pub signature: Vec<u8>,

	/// The expected key for the signature
	pub signing_key: Vec<u8>,
}

/// Possible key types for DKG
#[derive(Clone, RuntimeDebug, TypeInfo, PartialEq, Eq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DigitalSignatureType {
	/// Elliptic Curve Digital Signature Algorithm (ECDSA) key type.
	#[default]
	Ecdsa,

	/// Schnorr signature type over the P256 curve.
	SchnorrP256,

    /// Schnorr signature type over the Secp256k1 curve.
    SchnorrSecp256k1,

    /// Schnorr signature type over the Ristretto255 curve.
    SchnorrRistretto255,

    /// Schnorr signature type over the BabyJubJub curve.
    SchnorrBabyJubJub,

    /// Schnorr signature type over the Ed25519 curve.
    SchnorrEd25519,

    /// Edwards Digital Signature Algorithm (EdDSA) key type over the BabyJubJub curve.
    EdDSABabyJubJub,

    /// BLS 381 signature type.
    Bls381
}
