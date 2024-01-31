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
use sp_core::RuntimeDebug;
use sp_std::vec::Vec;

/// Represents a Signed Round Message by the offender.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub struct SignedRoundMessage {
	pub message: Vec<u8>,
	pub signature: Vec<u8>,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum DfnsCGGMP21Justification {
	Keygen(KeyAborted),
	Signing(SigningAborted),
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum KeyAborted {
	/// party decommitment doesn't match commitment.
	InvalidDecommitment { round1: SignedRoundMessage, round2: SignedRoundMessage },
	/// party provided invalid schnorr proof.
	InvalidSchnorrProof,
	/// party secret share is not consistent.
	FeldmanVerificationFailed,
	/// party data size is not suitable for threshold parameters.
	InvalidDataSize,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone)]
pub enum SigningAborted {
	/// `pi_enc::verify(K)` failed.
	EncProofOfK,
	/// ψ, ψˆ, or ψ' proofs are invalid
	InvalidPsi,
	/// ψ'' proof is invalid.
	InvalidPsiPrimePrime,
	/// Delta != G * delta
	MismatchedDelta,
}
