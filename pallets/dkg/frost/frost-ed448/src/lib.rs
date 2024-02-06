#![cfg_attr(not(feature = "std"), no_std)]

use ed448_goldilocks::{
	elliptic_curve::generic_array::{typenum::U114, GenericArray},
	CompressedEdwardsY, EdwardsPoint, Scalar, ScalarBytes,
};

#[cfg(feature = "std")]
use rand_core::{CryptoRng, RngCore};

use sha3::{
	digest::{ExtendableOutput, Update, XofReader},
	Shake256,
};

pub mod types;
pub use types::*;

// Re-exports in our public API
pub use frost_core::{
	error::{FieldError, GroupError},
	traits::{Ciphersuite, Field, Group},
};

#[cfg(feature = "std")]
pub use rand_core;

/// An implementation of the FROST(Ed448, SHAKE256) ciphersuite scalar field.
#[derive(Clone, Copy)]
pub struct Ed448ScalarField;

impl Field for Ed448ScalarField {
	type Scalar = WrappedScalar;

	type Serialization = [u8; 57];

	fn zero() -> Self::Scalar {
		WrappedScalar(Scalar::ZERO)
	}

	fn one() -> Self::Scalar {
		WrappedScalar(Scalar::ONE)
	}

	fn invert(scalar: &Self::Scalar) -> Result<Self::Scalar, FieldError> {
		if *scalar == <Self as Field>::zero() {
			Err(FieldError::InvalidZeroScalar)
		} else {
			Ok(WrappedScalar(scalar.0.invert()))
		}
	}

	#[cfg(feature = "std")]
	fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self::Scalar {
		WrappedScalar(Scalar::random(rng))
	}

	fn serialize(scalar: &Self::Scalar) -> Self::Serialization {
		scalar.0.to_bytes_rfc_8032().into()
	}

	fn deserialize(buf: &Self::Serialization) -> Result<Self::Scalar, FieldError> {
		let buffer = ScalarBytes::clone_from_slice(buf);
		match Scalar::from_canonical_bytes(&buffer).into() {
			Some(s) => Ok(WrappedScalar(s)),
			None => Err(FieldError::MalformedScalar),
		}
	}

	fn little_endian_serialize(scalar: &Self::Scalar) -> Self::Serialization {
		Self::serialize(scalar)
	}
}

/// An implementation of the FROST(Ed448, SHAKE256) ciphersuite group.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Ed448Group;

impl Group for Ed448Group {
	type Field = Ed448ScalarField;

	type Element = WrappedEdwardsPoint;

	type Serialization = [u8; 57];

	fn cofactor() -> <Self::Field as Field>::Scalar {
		WrappedScalar(Scalar::ONE)
	}

	fn identity() -> Self::Element {
		WrappedEdwardsPoint(EdwardsPoint::IDENTITY)
	}

	fn generator() -> Self::Element {
		WrappedEdwardsPoint(EdwardsPoint::GENERATOR)
	}

	fn serialize(element: &Self::Element) -> Self::Serialization {
		element.0.compress().0
	}

	fn deserialize(buf: &Self::Serialization) -> Result<Self::Element, GroupError> {
		let compressed = CompressedEdwardsY(*buf);
		match Option::<EdwardsPoint>::from(compressed.decompress()) {
			Some(point) => {
				if point == EdwardsPoint::IDENTITY {
					Err(GroupError::InvalidIdentityElement)
				} else if point.is_torsion_free().into() {
					// decompress() does not check for canonicality, so we
					// check by recompressing and comparing
					if point.compress().0 != compressed.0 {
						Err(GroupError::MalformedElement)
					} else {
						Ok(WrappedEdwardsPoint(point))
					}
				} else {
					Err(GroupError::InvalidNonPrimeOrderElement)
				}
			},
			None => Err(GroupError::MalformedElement),
		}
	}
}

fn hash_to_array(inputs: &[&[u8]]) -> [u8; 114] {
	let mut h = Shake256::default();
	for i in inputs {
		h.update(i);
	}
	let mut reader = h.finalize_xof();
	let mut output = [0u8; 114];
	reader.read(&mut output);
	output
}

fn hash_to_scalar(inputs: &[&[u8]]) -> Scalar {
	let output = GenericArray::<u8, U114>::clone_from_slice(&hash_to_array(inputs));
	Scalar::from_bytes_mod_order_wide(&output)
}

/// Context string from the ciphersuite in the [spec]
///
/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.3-1
const CONTEXT_STRING: &str = "FROST-ED448-SHAKE256-v1";

/// An implementation of the FROST(Ed448, SHAKE256) ciphersuite.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ed448Shake256;

impl Ciphersuite for Ed448Shake256 {
	const ID: &'static str = CONTEXT_STRING;

	type Group = Ed448Group;

	type HashOutput = [u8; 114];

	type SignatureSerialization = [u8; 114];

	/// H1 for FROST(Ed448, SHAKE256)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.3-2.2.2.1
	fn H1(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		WrappedScalar(hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"rho", m]))
	}

	/// H2 for FROST(Ed448, SHAKE256)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.3-2.2.2.2
	fn H2(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		WrappedScalar(hash_to_scalar(&[b"SigEd448\0\0", m]))
	}

	/// H3 for FROST(Ed448, SHAKE256)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.3-2.2.2.3
	fn H3(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		WrappedScalar(hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"nonce", m]))
	}

	/// H4 for FROST(Ed448, SHAKE256)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.3-2.2.2.4
	fn H4(m: &[u8]) -> Self::HashOutput {
		hash_to_array(&[CONTEXT_STRING.as_bytes(), b"msg", m])
	}

	/// H5 for FROST(Ed448, SHAKE256)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.3-2.2.2.5
	fn H5(m: &[u8]) -> Self::HashOutput {
		hash_to_array(&[CONTEXT_STRING.as_bytes(), b"com", m])
	}

	/// HDKG for FROST(Ed448, SHAKE256)
	fn HDKG(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
		Some(WrappedScalar(hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"dkg", m])))
	}

	/// HID for FROST(Ed448, SHAKE256)
	fn HID(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
		Some(WrappedScalar(hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"id", m])))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frost_core::{signing_key::SigningKey, verifying_key::VerifyingKey};

	#[test]
	fn test_sign_and_verify() {
		let mut rng = rand_core::OsRng;

		let sk = SigningKey::<Ed448Shake256>::new(&mut rng);
		let vk = VerifyingKey::<Ed448Shake256>::from(sk);

		let msg = b"Hello, world!";
		let signature = sk.sign(&mut rng, msg);
		assert!(vk.verify(msg, &signature).is_ok());
	}
}