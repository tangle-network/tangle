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
use super::*;
use crate::types::BalanceOf;
use frame_support::{ensure, pallet_prelude::DispatchResult, sp_runtime::Saturating};
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::Encode;
use sp_core::{ecdsa, sr25519};
use sp_io::{crypto::sr25519_verify, hashing::keccak_256, EcdsaVerifyError};
use sp_std::{default::Default, vec::Vec};
use tangle_primitives::jobs::*;

/// Expected signature length
pub const SIGNATURE_LENGTH: usize = 65;
/// Expected key length for ecdsa
const ECDSA_KEY_LENGTH: usize = 33;
/// Expected key length for sr25519
const SCHNORR_KEY_LENGTH: usize = 32;

impl<T: Config> Pallet<T> {
	/// Calculates the fee for a given job submission based on the provided fee information.
	///
	/// The fee calculation considers both the base fee and an additional fee per validator,
	/// depending on the job type.
	///
	/// # Arguments
	///
	/// * `job` - A reference to the job submission containing information about the account, job
	///   type, and block number.
	///
	/// # Returns
	///
	/// Returns the calculated fee as a `BalanceOf<T>` type.
	pub fn job_to_fee(job: &JobSubmission<T::AccountId, BlockNumberFor<T>>) -> BalanceOf<T> {
		let fee_info = FeeInfo::<T>::get();
		// charge the base fee + per validator fee
		if job.job_type.is_phase_one() {
			let validator_count =
				job.job_type.clone().get_participants().expect("checked_above").len();
			let validator_fee = fee_info.dkg_validator_fee * (validator_count as u32).into();
			validator_fee.saturating_add(fee_info.base_fee)
		} else {
			fee_info.base_fee.saturating_add(fee_info.sig_validator_fee)
		}
	}

	/// Verifies a given job verification information and dispatches to specific verification logic
	/// based on the job type.
	///
	/// # Arguments
	///
	/// * `data` - The job verification information, which could be of different types such as DKG
	///   or others.
	///
	/// # Returns
	///
	/// Returns a `DispatchResult` indicating whether the verification was successful or encountered
	/// an error.
	pub fn verify(data: JobResult) -> DispatchResult {
		match data {
			JobResult::DKGPhaseOne(info) => Self::verify_generated_dkg_key(info),
			JobResult::DKGPhaseTwo(info) => Self::verify_dkg_signature(info),
			_ => Err(Error::<T>::InvalidJobType.into()), // this should never happen
		}
	}

	/// Verifies a generated DKG (Distributed Key Generation) key based on the provided DKG result.
	///
	/// The verification process depends on the key type specified in the DKG result.
	/// It dispatches the verification to the appropriate function for the specified key type (ECDSA
	/// or Schnorr).
	///
	/// # Arguments
	///
	/// * `data` - The DKG result containing participants, keys, and signatures.
	///
	/// # Returns
	///
	/// Returns a `DispatchResult` indicating whether the DKG key verification was successful
	/// or encountered an error.
	fn verify_generated_dkg_key(data: DKGTSSKeySubmissionResult) -> DispatchResult {
		match data.signature_type {
			DigitalSignatureType::Ecdsa => Self::verify_generated_dkg_key_ecdsa(data),
			DigitalSignatureType::SchnorrSr25519 => Self::verify_generated_dkg_key_schnorr(data),
			_ => Err(Error::<T>::InvalidSignature.into()), // unimplemented
		}
	}

	/// Verifies the generated DKG key for ECDSA signatures.
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
	fn verify_generated_dkg_key_ecdsa(data: DKGTSSKeySubmissionResult) -> DispatchResult {
		// Ensure participants and signatures are not empty
		ensure!(!data.participants.is_empty(), Error::<T>::NoParticipantsFound);
		ensure!(!data.signatures.is_empty(), Error::<T>::NoSignaturesFound);

		// Generate the required ECDSA signers
		let maybe_signers = data
			.participants
			.iter()
			.map(|x| {
				ecdsa::Public(
					Self::to_slice_33(x)
						.unwrap_or_else(|| panic!("Failed to convert input to ecdsa public key")),
				)
			})
			.collect::<Vec<ecdsa::Public>>();

		ensure!(!maybe_signers.is_empty(), Error::<T>::NoParticipantsFound);

		let mut known_signers: Vec<ecdsa::Public> = Default::default();

		for signature in data.signatures {
			// Ensure the required signer signature exists
			let (maybe_authority, success) =
				Self::verify_signer_from_set_ecdsa(maybe_signers.clone(), &data.key, &signature);

			if success {
				let authority = maybe_authority.ok_or(Error::<T>::CannotRetreiveSigner)?;

				// Ensure no duplicate signatures
				ensure!(!known_signers.contains(&authority), Error::<T>::DuplicateSignature);

				known_signers.push(authority);
			}
		}

		// Ensure a sufficient number of unique signers are present
		ensure!(known_signers.len() > data.threshold.into(), Error::<T>::NotEnoughSigners);

		Ok(())
	}

	/// Verifies the generated DKG key for Schnorr signatures.
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
	fn verify_generated_dkg_key_schnorr(data: DKGTSSKeySubmissionResult) -> DispatchResult {
		// Ensure participants and signatures are not empty
		ensure!(!data.participants.is_empty(), Error::<T>::NoParticipantsFound);
		ensure!(!data.signatures.is_empty(), Error::<T>::NoSignaturesFound);

		// Generate the required Schnorr signers
		let maybe_signers = data
			.participants
			.iter()
			.map(|x| {
				sr25519::Public(
					Self::to_slice_32(x)
						.unwrap_or_else(|| panic!("Failed to convert input to sr25519 public key")),
				)
			})
			.collect::<Vec<sr25519::Public>>();

		ensure!(!maybe_signers.is_empty(), Error::<T>::NoParticipantsFound);

		let mut known_signers: Vec<sr25519::Public> = Default::default();

		for signature in data.signatures {
			// Convert the signature from bytes to sr25519::Signature
			let signature: sr25519::Signature =
				signature.as_slice().try_into().map_err(|_| Error::<T>::CannotRetreiveSigner)?;

			let msg = data.key.encode();
			let hash = keccak_256(&msg);

			for signer in maybe_signers.clone() {
				// Verify the Schnorr signature
				if sr25519_verify(&signature, &hash, &signer) {
					ensure!(!known_signers.contains(&signer), Error::<T>::DuplicateSignature);

					known_signers.push(signer);
				}
			}
		}

		// Ensure a sufficient number of unique signers are present
		ensure!(known_signers.len() > data.threshold.into(), Error::<T>::NotEnoughSigners);

		Ok(())
	}

	/// Verifies a DKG (Distributed Key Generation) signature based on the provided DKG signature
	/// result.
	///
	/// The verification process depends on the key type specified in the DKG signature result.
	/// It dispatches the verification to the appropriate function for the specified key type (ECDSA
	/// or Schnorr).
	///
	/// # Arguments
	///
	/// * `data` - The DKG signature result containing the message data, signature, signing key, and
	///   key type.
	fn verify_dkg_signature(data: DKGTSSSignatureResult) -> DispatchResult {
		match data.signature_type {
			DigitalSignatureType::Ecdsa => Self::verify_dkg_signature_ecdsa(data),
			DigitalSignatureType::SchnorrSr25519 => Self::verify_dkg_signature_schnorr(data),
			_ => Err(Error::<T>::InvalidSignature.into()), // unimplemented
		}
	}

	/// Verifies the DKG signature result by recovering the ECDSA public key from the provided data
	/// and signature.
	///
	/// This function checks whether the recovered public key matches the expected signing key,
	/// ensuring the validity of the signature.
	///
	/// # Arguments
	///
	/// * `data` - The DKG signature result containing the message data and ECDSA signature.
	fn verify_dkg_signature_ecdsa(data: DKGTSSSignatureResult) -> DispatchResult {
		// Recover the ECDSA public key from the provided data and signature
		let recovered_key = Self::recover_ecdsa_pub_key(&data.data, &data.signature)
			.map_err(|_| Error::<T>::InvalidSignature)?;

		// Extract the expected key from the provided signing key
		let expected_key: Vec<_> = data.signing_key.iter().skip(1).cloned().collect();
		// The recovered key is 64 bytes uncompressed. The first 32 bytes represent the compressed
		// portion of the key.
		let signer = &recovered_key[..32];

		// Ensure that the recovered key matches the expected signing key
		ensure!(expected_key == signer, Error::<T>::SigningKeyMismatch);

		Ok(())
	}

	/// Verifies the DKG signature result for Schnorr signatures.
	///
	/// This function uses the Schnorr signature algorithm to verify the provided signature
	/// based on the message data, signature, and signing key in the DKG signature result.
	///
	/// # Arguments
	///
	/// * `data` - The DKG signature result containing the message data, Schnorr signature, and
	///   signing key.
	fn verify_dkg_signature_schnorr(data: DKGTSSSignatureResult) -> DispatchResult {
		// Convert the signature from bytes to sr25519::Signature
		let signature: sr25519::Signature = data
			.signature
			.as_slice()
			.try_into()
			.map_err(|_| Error::<T>::CannotRetreiveSigner)?;

		// Encode the message data and compute its keccak256 hash
		let msg = data.data.encode();
		let hash = keccak_256(&msg);

		// Verify the Schnorr signature using sr25519_verify
		if !sr25519_verify(
			&signature,
			&hash,
			&sr25519::Public(
				Self::to_slice_32(&data.signing_key)
					.unwrap_or_else(|| panic!("Failed to convert input to sr25519 public key")),
			),
		) {
			return Err(Error::<T>::InvalidSignature.into())
		}

		Ok(())
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
	pub fn recover_ecdsa_pub_key(
		data: &[u8],
		signature: &[u8],
	) -> Result<Vec<u8>, EcdsaVerifyError> {
		if signature.len() == SIGNATURE_LENGTH {
			let mut sig = [0u8; SIGNATURE_LENGTH];
			sig[..SIGNATURE_LENGTH].copy_from_slice(signature);

			let hash = keccak_256(data);

			let pub_key = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &hash)?;
			return Ok(pub_key.to_vec())
		}
		Err(EcdsaVerifyError::BadSignature)
	}

	/// Verifies the signer of a given message using a set of ECDSA public keys.
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
	/// * An optional ECDSA public key (`Option<ecdsa::Public>`) representing the verified signer.
	///   It is `None` if no valid signer is found.
	/// * A boolean value (`bool`) indicating whether the verification was successful (`true`) or
	///   not (`false`).
	pub fn verify_signer_from_set_ecdsa(
		maybe_signers: Vec<ecdsa::Public>,
		msg: &[u8],
		signature: &[u8],
	) -> (Option<ecdsa::Public>, bool) {
		let mut signer = None;
		let res = maybe_signers.iter().any(|x| {
			if let Ok(data) = Self::recover_ecdsa_pub_key(msg, signature) {
				let recovered = &data[..32];
				if x.0[1..].to_vec() == recovered.to_vec() {
					signer = Some(*x);
					true
				} else {
					false
				}
			} else {
				false
			}
		});

		(signer, res)
	}

	/// Utility function to create slice of fixed size
	pub fn to_slice_33(val: &[u8]) -> Option<[u8; 33]> {
		if val.len() == ECDSA_KEY_LENGTH {
			let mut key = [0u8; ECDSA_KEY_LENGTH];
			key[..ECDSA_KEY_LENGTH].copy_from_slice(val);

			return Some(key)
		}
		None
	}

	/// Utility function to create slice of fixed size
	pub fn to_slice_32(val: &[u8]) -> Option<[u8; 32]> {
		if val.len() == SCHNORR_KEY_LENGTH {
			let mut key = [0u8; SCHNORR_KEY_LENGTH];
			key[..SCHNORR_KEY_LENGTH].copy_from_slice(val);

			return Some(key)
		}
		None
	}
}
