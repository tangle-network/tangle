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
use frame_support::pallet_prelude::DispatchResult;
use frost_core::{signature::Signature, verifying_key::VerifyingKey};
use frost_ed25519::Ed25519Sha512;
use frost_ed448::Ed448Shake256;
use frost_p256::P256Sha256;
use frost_p384::P384Sha384;
use frost_ristretto255::Ristretto255Sha512;
use frost_secp256k1::Secp256K1Sha256;
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
		_ => return Err(Error::<T>::InvalidSignature.into()),
	};

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::Runtime;
	use frame_support::assert_ok;
	use frost_core::{signing_key::SigningKey, verifying_key::VerifyingKey};

	const MESSAGE: &[u8] = b"test message";

	macro_rules! test_verify_dkg_signature {
		($sig_name:ident, $scheme:expr, $impl_type:ty, $msg:expr) => {
			paste::item! {
				#[test]
				fn [< test_verify_signature_ $sig_name >] () {
					let mut rng = rand_core::OsRng;

					let sk = SigningKey::<$impl_type>::new(&mut rng);
					let vk = VerifyingKey::<$impl_type>::from(sk);

					// Generate the signature
					let signature = sk.sign(&mut rng, $msg);

					// Verify the signature
					assert!(vk.verify($msg, &signature).is_ok());

					// Verify using the DKG signature verification function
					assert_ok!(verify_dkg_signature_schnorr_frost::<Runtime>(
						$scheme,
						$msg,
						&signature.serialize(),
						&vk.serialize()
					));
				}
			}
		};
	}

	test_verify_dkg_signature!(
		secp256k1,
		DigitalSignatureScheme::SchnorrSecp256k1,
		Secp256K1Sha256,
		MESSAGE
	);

	test_verify_dkg_signature!(
		ed25519,
		DigitalSignatureScheme::SchnorrEd25519,
		Ed25519Sha512,
		MESSAGE
	);

	test_verify_dkg_signature!(ed448, DigitalSignatureScheme::SchnorrEd448, Ed448Shake256, MESSAGE);

	test_verify_dkg_signature!(p256, DigitalSignatureScheme::SchnorrP256, P256Sha256, MESSAGE);

	test_verify_dkg_signature!(p384, DigitalSignatureScheme::SchnorrP384, P384Sha384, MESSAGE);

	test_verify_dkg_signature!(
		ristretto255,
		DigitalSignatureScheme::SchnorrRistretto255,
		Ristretto255Sha512,
		MESSAGE
	);
}
