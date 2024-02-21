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

use crate::*;

use frost_ed25519::Ed25519Sha512;
use frost_ed448::Ed448Shake256;
use frost_p256::P256Sha256;
use frost_p384::P384Sha384;
use frost_ristretto255::Ristretto255Sha512;
use frost_secp256k1::Secp256K1Sha256;
use sp_runtime::DispatchResult;
use sp_std::vec::Vec;

use tangle_primitives::{
	misbehavior::{MisbehaviorSubmission, SignedRoundMessage},
	roles::ThresholdSignatureRoleType,
};

use frost_core::{
	identifier::Identifier,
	keygen::{round1, round2},
	keys::{SecretShare, VerifiableSecretSharingCommitment},
	pok_challenge,
	signature::Signature,
	traits::{Ciphersuite, Group},
	Header,
};

use super::convert_error;

/// Message from round 1
#[derive(Clone, serde::Serialize, serde::Deserialize, udigest::Digestable)]
#[serde(bound = "")]
#[udigest(bound = "")]
#[udigest(tag = "zcash.frost.keygen.threshold.round1")]
pub struct MsgRound1 {
	pub msg: Vec<u8>,
}
/// Message from round 2
#[derive(Clone, serde::Serialize, serde::Deserialize, udigest::Digestable)]
#[serde(bound = "")]
#[udigest(bound = "")]
#[udigest(tag = "zcash.frost.keygen.threshold.round2")]
pub struct MsgRound2 {
	pub msg: Vec<u8>,
}

pub fn verify_invalid_schnorr_proof<T: Config, C: Ciphersuite>(
	offender: u16,
	round1: &MsgRound1,
) -> DispatchResult {
	// Identifiers are indexed from 1 in the FROST protocol and
	// the offender here is indexed from 0 out of the participants.
	let identifier: Identifier<C> = Identifier::try_from(offender + 1)
		.map_err(|_| Error::<T>::InvalidIdentifierDeserialization)?;

	let round1_package: round1::Package<C> =
		postcard::from_bytes(&round1.msg).map_err(|_| Error::<T>::MalformedRoundMessage)?;

	if (verify_invalid_proof_of_knowledge::<T, C>(
		identifier,
		&round1_package.commitment,
		round1_package.proof_of_knowledge,
	)?)
	.is_some()
	{
		Ok(())
	} else {
		Err(Error::<T>::ValidSchnorrProof)?
	}
}

/// Verifies the proof of knowledge of the secret coefficients used to generate the
/// public secret sharing commitment.
pub fn verify_invalid_proof_of_knowledge<T: Config, C: Ciphersuite>(
	identifier: Identifier<C>,
	commitment: &VerifiableSecretSharingCommitment<C>,
	proof_of_knowledge: Signature<C>,
) -> Result<Option<Identifier<C>>, Error<T>> {
	// Round 1, Step 5
	//
	// > Upon receiving C⃗_ℓ, σ_ℓ from participants 1 ≤ ℓ ≤ n, ℓ ≠ i, participant
	// > P_i verifies σ_ℓ = (R_ℓ, μ_ℓ), aborting on failure, by checking
	// > R_ℓ ? ≟ g^{μ_ℓ} · φ^{-c_ℓ}_{ℓ0}, where c_ℓ = H(ℓ, Φ, φ_{ℓ0}, R_ℓ).
	let ell = identifier;
	let R_ell = proof_of_knowledge.R;
	let mu_ell = proof_of_knowledge.z;
	let phi_ell0 = commitment.verifying_key().map_err(|_| Error::<T>::MissingFrostCommitment)?;
	let c_ell = pok_challenge::<C>(ell, &phi_ell0, &R_ell)
		.ok_or(Error::<T>::InvalidFrostSignatureScheme)?;
	// Check if the proof is valid, otherwise return the offender
	if R_ell != <C::Group>::generator() * mu_ell - phi_ell0.element * c_ell.0 {
		Ok(Some(ell))
	} else {
		// Valid schnorr proof should return `None`
		Ok(None)
	}
}

pub fn schnorr_proof<T: Config>(
	role: ThresholdSignatureRoleType,
	data: &MisbehaviorSubmission,
	parties_including_offender: &[[u8; 33]],
	round1: &SignedRoundMessage,
) -> DispatchResult {
	let offender = data.offender;
	let index: u16 = parties_including_offender
		.iter()
		.position(|&p| p == offender)
		.ok_or(Error::<T>::UnknownIdentifier)? as u16;
	Pallet::<T>::ensure_signed_by_offender(round1, data.offender)?;

	let round1_msg: MsgRound1 = postcard::from_bytes::<MsgRound1>(&round1.message)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;

	match role {
		ThresholdSignatureRoleType::ZcashFrostP256 =>
			verify_invalid_schnorr_proof::<T, P256Sha256>(index, &round1_msg)?,
		ThresholdSignatureRoleType::ZcashFrostP384 =>
			verify_invalid_schnorr_proof::<T, P384Sha384>(index, &round1_msg)?,
		ThresholdSignatureRoleType::ZcashFrostSecp256k1 =>
			verify_invalid_schnorr_proof::<T, Secp256K1Sha256>(index, &round1_msg)?,
		ThresholdSignatureRoleType::ZcashFrostRistretto255 =>
			verify_invalid_schnorr_proof::<T, Ristretto255Sha512>(index, &round1_msg)?,
		ThresholdSignatureRoleType::ZcashFrostEd25519 =>
			verify_invalid_schnorr_proof::<T, Ed25519Sha512>(index, &round1_msg)?,
		ThresholdSignatureRoleType::ZcashFrostEd448 =>
			verify_invalid_schnorr_proof::<T, Ed448Shake256>(index, &round1_msg)?,
		_ => Err(Error::<T>::InvalidFrostSignatureScheme)?,
	};

	// TODO: add slashing logic
	// Slash the offender!
	Ok(())
}

pub fn verify_invalid_secret_share<T: Config, C: Ciphersuite>(
	offender: u16,
	round1: &MsgRound1,
	round2: &MsgRound2,
) -> DispatchResult {
	// Identifiers are indexed from 1 in the FROST protocol and
	// the offender here is indexed from 0 out of the participants.
	let identifier: Identifier<C> = Identifier::try_from(offender + 1)
		.map_err(|_| Error::<T>::InvalidIdentifierDeserialization)?;

	let round1_package: round1::Package<C> =
		postcard::from_bytes(&round1.msg).map_err(|_| Error::<T>::MalformedRoundMessage)?;

	let round2_package: round2::Package<C> =
		postcard::from_bytes(&round2.msg).map_err(|_| Error::<T>::MalformedRoundMessage)?;

	let commitment = round1_package.commitment;
	let f_ell_i = round2_package.signing_share;

	// The verification is exactly the same as the regular SecretShare verification;
	// however the required components are in different places.
	// Build a temporary SecretShare so what we can call verify().
	let secret_share = SecretShare {
		header: Header::default(),
		identifier,
		signing_share: f_ell_i,
		commitment: commitment.clone(),
	};

	// Verify the share. We don't need the result.
	let _ = secret_share.verify().map_err(convert_error::<T>)?;

	Ok(())
}

pub fn invalid_secret_share<T: Config>(
	role: ThresholdSignatureRoleType,
	data: &MisbehaviorSubmission,
	parties_including_offender: &[[u8; 33]],
	round1: &SignedRoundMessage,
	round2: &SignedRoundMessage,
) -> DispatchResult {
	let offender = data.offender;
	let index: u16 = parties_including_offender
		.iter()
		.position(|&p| p == offender)
		.ok_or(Error::<T>::UnknownIdentifier)? as u16;
	Pallet::<T>::ensure_signed_by_offender(round1, data.offender)?;

	let round1_msg: MsgRound1 = postcard::from_bytes::<MsgRound1>(&round1.message)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;

	let round2_msg: MsgRound2 = postcard::from_bytes::<MsgRound2>(&round2.message)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;

	match role {
		ThresholdSignatureRoleType::ZcashFrostP256 =>
			verify_invalid_secret_share::<T, P256Sha256>(index, &round1_msg, &round2_msg)?,
		ThresholdSignatureRoleType::ZcashFrostP384 =>
			verify_invalid_secret_share::<T, P384Sha384>(index, &round1_msg, &round2_msg)?,
		ThresholdSignatureRoleType::ZcashFrostSecp256k1 =>
			verify_invalid_secret_share::<T, Secp256K1Sha256>(index, &round1_msg, &round2_msg)?,
		ThresholdSignatureRoleType::ZcashFrostRistretto255 =>
			verify_invalid_secret_share::<T, Ristretto255Sha512>(index, &round1_msg, &round2_msg)?,
		ThresholdSignatureRoleType::ZcashFrostEd25519 =>
			verify_invalid_secret_share::<T, Ed25519Sha512>(index, &round1_msg, &round2_msg)?,
		ThresholdSignatureRoleType::ZcashFrostEd448 =>
			verify_invalid_secret_share::<T, Ed448Shake256>(index, &round1_msg, &round2_msg)?,
		_ => Err(Error::<T>::InvalidFrostSignatureScheme)?,
	};

	// TODO: add slashing logic
	// Slash the offender!
	Ok(())
}
