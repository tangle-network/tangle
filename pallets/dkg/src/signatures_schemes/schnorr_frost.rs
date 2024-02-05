use frame_support::pallet_prelude::DispatchResult;
use frost_core::{signature::Signature, verifying_key::VerifyingKey};
use frost_ed25519::Ed25519Sha512;
use frost_ed448::Ed448Shake256;
use frost_p256::P256Sha256;
use frost_p384::P384Sha384;
use frost_ristretto255::Ristretto255Sha512;
use frost_secp256k1::Secp256K1Sha256;
use frost_taproot::Secp256K1Taproot;
use tangle_primitives::jobs::DigitalSignatureScheme;

use crate::{Config, Error};

/// Macro to verify a Schnorr signature using the specified signature scheme.
macro_rules! verify_signature {
	($impl_type:ty, $key:expr, $signature:expr, $msg:expr, $key_default:expr, $sig_default:expr) => {{
		let verifying_key: VerifyingKey<$impl_type> =
			VerifyingKey::deserialize($key.try_into().unwrap_or($key_default))
				.map_err(|_| Error::<T>::InvalidVerifyingKey)?;
		let sig: Signature<$impl_type> =
			Signature::deserialize($signature.try_into().unwrap_or($sig_default))
				.map_err(|_| Error::<T>::InvalidSignature)?;
		verifying_key.verify($msg, &sig).map_err(|_| Error::<T>::InvalidSignature)?
	}};
}

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
pub fn verify_dkg_signature_schnorr_frost<T: Config>(
	scheme: DigitalSignatureScheme,
	msg: &[u8],
	signature: &[u8],
	key: &[u8],
) -> DispatchResult {
	match scheme {
		DigitalSignatureScheme::SchnorrEd25519 => {
			verify_signature!(Ed25519Sha512, key, signature, msg, [0u8; 32], [0u8; 64]);
		},
		DigitalSignatureScheme::SchnorrEd448 => {
			verify_signature!(Ed448Shake256, key, signature, msg, [0u8; 57], [0u8; 114]);
		},
		DigitalSignatureScheme::SchnorrP256 => {
			verify_signature!(P256Sha256, key, signature, msg, [0u8; 33], [0u8; 65]);
		},
		DigitalSignatureScheme::SchnorrP384 => {
			verify_signature!(P384Sha384, key, signature, msg, [0u8; 49], [0u8; 97]);
		},
		DigitalSignatureScheme::SchnorrRistretto255 => {
			verify_signature!(Ristretto255Sha512, key, signature, msg, [0u8; 32], [0u8; 64]);
		},
		DigitalSignatureScheme::SchnorrSecp256k1 => {
			verify_signature!(Secp256K1Sha256, key, signature, msg, [0u8; 33], [0u8; 65]);
		},
		DigitalSignatureScheme::SchnorrSecp256k1Taproot => {
			verify_signature!(Secp256K1Taproot, key, signature, msg, [0u8; 33], [0u8; 65]);
		},
		_ => return Err(Error::<T>::InvalidSignature.into()),
	};

	Ok(())
}
