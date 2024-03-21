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
use crate::{signatures_schemes::to_slice_33, Config, Error};
use ecdsa_core::signature::hazmat::PrehashVerifier;
use frame_support::{ensure, pallet_prelude::DispatchResult};
use generic_ec::{coords::HasAffineX, curves::Stark, Point, Scalar};
use sp_core::ecdsa;
use sp_io::{hashing::keccak_256, EcdsaVerifyError};
use sp_runtime::BoundedVec;
use sp_std::vec::Vec;
use tangle_primitives::jobs::DKGTSSKeySubmissionResult;

pub const ECDSA_SIGNATURE_LENGTH: usize = 65;

/// Verifies the Secp256k1 DKG signature result by recovering the ECDSA public key from the provided
/// data and signature.
///
/// This function checks whether the recovered public key matches the expected signing key,
/// ensuring the validity of the signature.
///
/// # Arguments
///
/// * `data` - The DKG signature result containing the message data and ECDSA signature.
/// * `signature` - The ECDSA signature to be verified.
/// * `expected_key` - The expected ECDSA public key.
pub fn verify_secp256k1_ecdsa_signature<T: Config>(
	msg: &[u8],
	signature: &[u8],
	expected_key: &[u8],
	derivation_path: Option<BoundedVec<u8, T::MaxAdditionalParamsLen>>,
) -> DispatchResult {
	let public_key =
		secp256k1::PublicKey::from_slice(expected_key).map_err(|_| Error::<T>::InvalidPublicKey)?;
	let message =
		secp256k1::Message::from_digest_slice(msg).map_err(|_| Error::<T>::InvalidMessage)?;
	let signature = secp256k1::ecdsa::Signature::from_compact(&signature)
		.map_err(|_| Error::<T>::InvalidSignature)?;
	ensure!(signature.verify(&message, &public_key).is_ok(), Error::<T>::InvalidSignature);
	Ok(())
}

/// Verify the Secp256r1 DKG signature result by recovering the ECDSA public key from the provided
/// data and signature.
///
/// This function checks whether the recovered public key matches the expected signing key,
/// ensuring the validity of the signature.
///
/// # Arguments
///
/// * `data` - The DKG signature result containing the message data and ECDSA signature.
/// * `signature` - The ECDSA signature to be verified.
/// * `expected_key` - The expected ECDSA public key.
pub fn verify_secp256r1_ecdsa_signature<T: Config>(
	msg: &[u8],
	signature: &[u8],
	expected_key: &[u8],
	derivation_path: Option<BoundedVec<u8, T::MaxAdditionalParamsLen>>,
) -> DispatchResult {
	use p256::elliptic_curve::group::GroupEncoding;
	let maybe_affine_point = p256::AffinePoint::from_bytes(expected_key.into());
	if maybe_affine_point.is_none().into() {
		Err(Error::<T>::InvalidPublicKey)?;
	}
	let verifying_key = p256::ecdsa::VerifyingKey::from_affine(maybe_affine_point.unwrap())
		.map_err(|_| Error::<T>::InvalidPublicKey)?;
	let signature =
		p256::ecdsa::Signature::from_slice(&signature).map_err(|_| Error::<T>::InvalidSignature)?;

	ensure!(
		verifying_key.verify_prehash(msg, &signature).map(|_| signature).is_ok(),
		Error::<T>::InvalidSignature
	);
	Ok(())
}

/// Verifies the Stark curve DKG signature result by recovering the ECDSA public key from the
/// provided data and signature.
///
/// This function checks whether the recovered public key matches the expected signing key,
/// ensuring the validity of the signature.
///
/// # Arguments
///
/// * `data` - The DKG signature result containing the message data and ECDSA signature.
/// * `signature` - The ECDSA signature to be verified.
/// * `expected_key` - The expected ECDSA public key.
pub fn verify_stark_ecdsa_signature<T: Config>(
	msg: &[u8],
	signature: &[u8],
	expected_key: &[u8],
	derivation_path: Option<BoundedVec<u8, T::MaxAdditionalParamsLen>>,
) -> DispatchResult {
	// The message should be pre-hashed uisng a 32-byte digest
	if msg.len() != 32 {
		Err(Error::<T>::InvalidMessage)?;
	}

	// The signature should be a 64-byte r and s pair
	if signature.len() != 64 {
		Err(Error::<T>::MalformedStarkSignature)?;
	}

	let parse_signature = |inp: &[u8]| -> Result<(Scalar<Stark>, Scalar<Stark>), Error<T>> {
		let r_bytes = &inp[0..inp.len() / 2];
		let s_bytes = &inp[inp.len() / 2..];
		let r = Scalar::from_be_bytes(r_bytes).map_err(|_| Error::<T>::FieldElementError)?;
		let s = Scalar::from_be_bytes(s_bytes).map_err(|_| Error::<T>::FieldElementError)?;

		Ok((r, s))
	};

	let (r, s) = parse_signature(signature)?;
	let public_key_x: Scalar<Stark> = Point::from_bytes(expected_key)
		.map_err(|_| Error::<T>::InvalidPublicKey)?
		.x()
		.ok_or(Error::<T>::FieldElementError)?
		.to_scalar();

	let public_key = convert_stark_scalar::<T>(&public_key_x)?;
	let message = convert_stark_scalar::<T>(&Scalar::<Stark>::from_be_bytes_mod_order(msg))?;
	let r = convert_stark_scalar::<T>(&r)?;
	let s = convert_stark_scalar::<T>(&s)?;

	let result = starknet_crypto::verify(&public_key, &message, &r, &s)
		.map_err(|_| Error::<T>::InvalidSignature)?;

	ensure!(result, Error::<T>::InvalidSignature);
	Ok(())
}

pub fn convert_stark_scalar<T: Config>(
	x: &Scalar<Stark>,
) -> Result<starknet_crypto::FieldElement, Error<T>> {
	let bytes = x.to_be_bytes();
	debug_assert_eq!(bytes.len(), 32);
	let mut buffer = [0u8; 32];
	buffer.copy_from_slice(bytes.as_bytes());
	starknet_crypto::FieldElement::from_bytes_be(&buffer).map_err(|_| Error::<T>::FieldElementError)
}

/// Recovers the ECDSA public key from a given message and signature.
///
/// # Arguments
///
/// * `data` - The message for which the signature is being verified.
/// * `signature` - The ECDSA signature to be verified.
///
/// # Returns
///
/// Returns a `Result` containing the recovered ECDSA public key as a `Vec<u8>` or an
/// `EcdsaVerifyError` if verification fails.
pub fn recover_ecdsa_pub_key(data: &[u8], signature: &[u8]) -> Result<Vec<u8>, EcdsaVerifyError> {
	if signature.len() == ECDSA_SIGNATURE_LENGTH {
		let mut sig = [0u8; ECDSA_SIGNATURE_LENGTH];
		sig[..ECDSA_SIGNATURE_LENGTH].copy_from_slice(signature);

		let hash = keccak_256(data);

		let pub_key = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &hash)?;
		return Ok(pub_key.to_vec());
	}
	Err(EcdsaVerifyError::BadSignature)
}

/// Recovers the ECDSA public key from a given message and signature.
///
/// # Arguments
///
/// * `data` - The message for which the signature is being verified.
/// * `signature` - The ECDSA signature to be verified.
///
/// # Returns
///
/// Returns a `Result` containing the recovered ECDSA public key as a `Vec<u8>` or an
/// `EcdsaVerifyError` if verification fails.
pub fn recover_ecdsa_pub_key_compressed(
	data: &[u8],
	signature: &[u8],
) -> Result<Vec<u8>, EcdsaVerifyError> {
	if signature.len() == ECDSA_SIGNATURE_LENGTH {
		let mut sig = [0u8; ECDSA_SIGNATURE_LENGTH];
		sig[..ECDSA_SIGNATURE_LENGTH].copy_from_slice(signature);

		let hash = keccak_256(data);

		let pub_key = sp_io::crypto::secp256k1_ecdsa_recover_compressed(&sig, &hash)?;
		return Ok(pub_key.to_vec());
	}
	Err(EcdsaVerifyError::BadSignature)
}

/// Verifies the signer of a given message using a set of Secp256k1 ECDSA public keys.
///
/// Given a vector of ECDSA public keys (`maybe_signers`), a message (`msg`), and an ECDSA
/// signature (`signature`), this function checks if any of the public keys in the set can be a
/// valid signer for the provided message and signature.
///
/// # Arguments
///
/// * `maybe_signers` - A vector of ECDSA public keys that may represent the potential signers.
/// * `msg` - The message for which the signature is being verified.
/// * `signature` - The ECDSA signature to be verified.
///
/// # Returns
///
/// Returns a tuple containing:
/// * An optional ECDSA public key (`Option<ecdsa::Public>`) representing the verified signer. It is
///   `None` if no valid signer is found.
/// * A boolean value (`bool`) indicating whether the verification was successful (`true`) or not
///   (`false`).
pub fn verify_signer_from_set_ecdsa(
	maybe_signers: Vec<ecdsa::Public>,
	msg: &[u8],
	signature: &[u8],
) -> (Option<ecdsa::Public>, bool) {
	let mut signer = None;
	let recovered_result = recover_ecdsa_pub_key(msg, signature);
	let res = if let Ok(data) = recovered_result {
		let recovered = &data[..32];
		maybe_signers.iter().any(|x| {
			if x.0[1..].to_vec() == recovered.to_vec() {
				signer = Some(*x);
				true
			} else {
				false
			}
		})
	} else {
		false
	};

	(signer, res)
}

/// Verifies the generated DKG key from a set of participants' Secp256k1 ECDSA keys and signatures.
///
/// This function includes generating required signers, validating signatures, and ensuring a
/// sufficient number of unique signers are present.
///
/// # Arguments
///
/// * `data` - The DKG verification information containing participants, keys, and signatures.
///
/// # Returns
///
/// Returns a `DispatchResult` indicating whether the DKG key verification was successful or
/// encountered an error.
pub fn verify_generated_dkg_key_ecdsa<T: Config>(
	data: DKGTSSKeySubmissionResult<T::MaxKeyLen, T::MaxParticipants, T::MaxSignatureLen>,
) -> DispatchResult {
	// Ensure participants and signatures are not empty
	ensure!(!data.participants.is_empty(), Error::<T>::NoParticipantsFound);
	ensure!(!data.signatures.is_empty(), Error::<T>::NoSignaturesFound);

	// Generate the required ECDSA signers
	let maybe_signers = data
		.participants
		.iter()
		.map(|x| {
			ecdsa::Public(
				to_slice_33(x)
					.unwrap_or_else(|| panic!("Failed to convert input to ecdsa public key")),
			)
		})
		.collect::<Vec<ecdsa::Public>>();

	ensure!(!maybe_signers.is_empty(), Error::<T>::NoParticipantsFound);

	let mut known_signers: Vec<ecdsa::Public> = Default::default();

	for signature in data.signatures {
		// Ensure the required signer signature exists
		let (maybe_authority, success) =
			verify_signer_from_set_ecdsa(maybe_signers.clone(), &data.key, &signature);

		if success {
			let authority = maybe_authority.ok_or(Error::<T>::CannotRetreiveSigner)?;

			// Ensure no duplicate signatures
			ensure!(!known_signers.contains(&authority), Error::<T>::DuplicateSignature);

			known_signers.push(authority);
		}
	}

	// Ensure a sufficient number of unique signers are present
	ensure!(known_signers.len() >= usize::from(data.threshold), Error::<T>::NotEnoughSigners);

	Ok(())
}
