use crate::{Config, Error};
use frame_support::dispatch::DispatchResult;
use sp_core::keccak_256;
use tangle_primitives::jobs::DigitalSignatureScheme;
use wsts::common::Signature;
use wsts::{Point, Scalar};

/// Verifies a Distributed Key Generation (DKG) signature using the Schnorr signature scheme.
///
/// Utilizes the Schnorr signature algorithm to validate the authenticity of a signature
/// by comparing it against the original message data and the corresponding public key.
/// Supports multiple signature schemes based on the role type.
///
/// # Arguments
///
/// * `scheme` - The type that determines the signature scheme to be used for verification.
/// * `msg` - The message data that was signed.
/// * `signature` - The Schnorr signature to be verified.
/// * `key` - The public key associated with the signature.
pub fn verify_dkg_signature_wsts_v2<T: Config>(
	_scheme: DigitalSignatureScheme,
	msg: &[u8],
	signature: &[u8],
	key: &[u8],
) -> DispatchResult {
	let (R, z) = bincode2::deserialize::<(Point, Scalar)>(signature)
		.map_err(|_| Error::<T>::InvalidSignatureDeserialization)?;
	let signature = Signature { R, z };
	let aggregated_public_key = bincode2::deserialize::<Point>(key)
		.map_err(|_| Error::<T>::InvalidVerifyingKeyDeserialization)?;
	// The signature is performed over the hash of the original message
	let msg = keccak_256(msg);
	if signature.verify(&aggregated_public_key, &msg) {
		Ok(())
	} else {
		Err(Error::<T>::InvalidSignature.into())
	}
}
