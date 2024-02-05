use frame_support::{ensure, pallet_prelude::DispatchResult};
use parity_scale_codec::Encode;
use sp_core::sr25519;
use sp_io::{crypto::sr25519_verify, hashing::keccak_256};
use sp_std::vec::Vec;
use tangle_primitives::jobs::DKGTSSKeySubmissionResult;

use crate::{Config, Error};

use super::to_slice_32;

/// Verifies the DKG signature result for Schnorr signatures over sr25519.
///
/// This function uses the Schnorr signature algorithm to verify the provided signature
/// based on the message data, signature, and signing key in the DKG signature result.
///
/// # Arguments
///
/// * `msg` - The message data that was signed.
/// * `signature` - The Schnorr signature to be verified.
/// * `key` - The public key associated with the signature.
pub fn verify_dkg_signature_schnorr_sr25519<T: Config>(
	msg: &[u8],
	signature: &[u8],
	key: &[u8],
) -> DispatchResult {
	// Convert the signature from bytes to sr25519::Signature
	let signature: sr25519::Signature =
		signature.try_into().map_err(|_| Error::<T>::CannotRetreiveSigner)?;

	// Encode the message data and compute its keccak256 hash
	let msg = msg.encode();
	let hash = keccak_256(&msg);

	// Verify the Schnorr signature using sr25519_verify
	if !sr25519_verify(
		&signature,
		&hash,
		&sr25519::Public(
			to_slice_32(key)
				.unwrap_or_else(|| panic!("Failed to convert input to sr25519 public key")),
		),
	) {
		return Err(Error::<T>::InvalidSignature.into())
	}

	Ok(())
}

/// Verifies the generated DKG key from a set of participants' Schnorr Sr25519 keys and signatures.
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
pub fn verify_generated_dkg_key_schnorr_sr25519<T: Config>(
	data: DKGTSSKeySubmissionResult<T::MaxKeyLen, T::MaxParticipants, T::MaxSignatureLen>,
) -> DispatchResult {
	// Ensure participants and signatures are not empty
	ensure!(!data.participants.is_empty(), Error::<T>::NoParticipantsFound);
	ensure!(!data.signatures.is_empty(), Error::<T>::NoSignaturesFound);

	// Generate the required Schnorr signers
	let maybe_signers = data
		.participants
		.iter()
		.map(|x| {
			sr25519::Public(
				to_slice_32(x)
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
