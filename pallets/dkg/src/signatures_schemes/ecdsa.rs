use frame_support::{ensure, pallet_prelude::DispatchResult};
use sp_core::ecdsa;
use sp_io::{hashing::keccak_256, EcdsaVerifyError};
use sp_std::vec::Vec;
use tangle_primitives::jobs::DKGTSSKeySubmissionResult;

use crate::{signatures_schemes::to_slice_33, Config, Error};

pub const ECDSA_SIGNATURE_LENGTH: usize = 65;

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
		return Ok(pub_key.to_vec())
	}
	Err(EcdsaVerifyError::BadSignature)
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
pub fn verify_dkg_signature_ecdsa<T: Config>(
	msg: &[u8],
	signature: &[u8],
	expected_key: &[u8],
) -> DispatchResult {
	// Recover the ECDSA public key from the provided data and signature
	let recovered_key =
		recover_ecdsa_pub_key(msg, signature).map_err(|_| Error::<T>::InvalidSignature)?;

	// Extract the expected key from the provided signing key
	let expected_key: Vec<_> = expected_key.iter().skip(1).cloned().collect();
	// The recovered key is 64 bytes uncompressed. The first 32 bytes represent the compressed
	// portion of the key.
	let signer = &recovered_key[..32];

	// Ensure that the recovered key matches the expected signing key
	ensure!(expected_key == signer, Error::<T>::SigningKeyMismatch);

	Ok(())
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

/// Verifies the generated DKG key from a set of participants' ECDSA keys and signatures.
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
	data: DKGTSSKeySubmissionResult,
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
	ensure!(known_signers.len() > data.threshold.into(), Error::<T>::NotEnoughSigners);

	Ok(())
}
