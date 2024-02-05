#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use curve25519_dalek::{
	constants::ED25519_BASEPOINT_POINT,
	edwards::{CompressedEdwardsY, EdwardsPoint},
	scalar::Scalar,
	traits::Identity,
};

#[cfg(feature = "std")]
use rand_core::{CryptoRng, RngCore};
use sha2::{Digest, Sha512};

pub mod types;
pub use types::*;

// Re-exports in our public API
pub use frost_core::{
	error::{FieldError, GroupError},
	traits::{Ciphersuite, Field, Group},
};

#[cfg(feature = "std")]
pub use rand_core;

/// An implementation of the FROST(Ed25519, SHA-512) ciphersuite scalar field.
#[derive(Clone, Copy)]
pub struct Ed25519ScalarField;

impl Field for Ed25519ScalarField {
	type Scalar = WrappedScalar;

	type Serialization = [u8; 32];

	fn zero() -> Self::Scalar {
		WrappedScalar(Scalar::ZERO)
	}

	fn one() -> Self::Scalar {
		WrappedScalar(Scalar::ONE)
	}

	fn invert(scalar: &Self::Scalar) -> Result<Self::Scalar, FieldError> {
		// [`curve25519_dalek::scalar::Scalar`]'s Eq/PartialEq does a constant-time comparison using
		// `ConstantTimeEq`
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
		scalar.0.to_bytes()
	}

	fn deserialize(buf: &Self::Serialization) -> Result<Self::Scalar, FieldError> {
		match Scalar::from_canonical_bytes(*buf).into() {
			Some(s) => Ok(WrappedScalar(s)),
			None => Err(FieldError::MalformedScalar),
		}
	}

	fn little_endian_serialize(scalar: &Self::Scalar) -> Self::Serialization {
		Self::serialize(scalar)
	}
}

/// An implementation of the FROST(Ed25519, SHA-512) ciphersuite group.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Ed25519Group;

impl Group for Ed25519Group {
	type Field = Ed25519ScalarField;

	type Element = WrappedEdwardsPoint;

	type Serialization = [u8; 32];

	fn cofactor() -> <Self::Field as Field>::Scalar {
		WrappedScalar(Scalar::ONE)
	}

	fn identity() -> Self::Element {
		WrappedEdwardsPoint(EdwardsPoint::identity())
	}

	fn generator() -> Self::Element {
		WrappedEdwardsPoint(ED25519_BASEPOINT_POINT)
	}

	fn serialize(element: &Self::Element) -> Self::Serialization {
		element.0.compress().to_bytes()
	}

	fn deserialize(buf: &Self::Serialization) -> Result<Self::Element, GroupError> {
		match CompressedEdwardsY::from_slice(buf.as_ref())
			.map_err(|_| GroupError::MalformedElement)?
			.decompress()
		{
			Some(point) => {
				if WrappedEdwardsPoint(point) == Self::identity() {
                    Err(GroupError::InvalidIdentityElement)
                } else if point.is_torsion_free() {
                    // At this point we should reject points which were not
                    // encoded canonically (i.e. Y coordinate >= p).
                    // However, we don't allow non-prime order elements,
                    // and that suffices to also reject non-canonical encodings
                    // per https://eprint.iacr.org/2020/1244.pdf:
                    //
                    // > There are 19 elliptic curve points that can be encoded in a non-canonical form.
                    // > (...) Among these points there are 2 points of small order and from the
                    // > remaining 17 y-coordinates only 10 decode to valid curve points all of mixed order.
                    Ok(WrappedEdwardsPoint(point))
                } else {
                    Err(GroupError::InvalidNonPrimeOrderElement)
                }
			},
			None => Err(GroupError::MalformedElement),
		}
	}
}

fn hash_to_array(inputs: &[&[u8]]) -> [u8; 64] {
	let mut h = Sha512::new();
	for i in inputs {
		h.update(i);
	}
	let mut output = [0u8; 64];
	output.copy_from_slice(h.finalize().as_slice());
	output
}

fn hash_to_scalar(inputs: &[&[u8]]) -> Scalar {
	let output = hash_to_array(inputs);
	Scalar::from_bytes_mod_order_wide(&output)
}

/// Context string from the ciphersuite in the [spec]
///
/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.1-1
const CONTEXT_STRING: &str = "FROST-ED25519-SHA512-v1";

/// An implementation of the FROST(Ed25519, SHA-512) ciphersuite.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ed25519Sha512;

impl Ciphersuite for Ed25519Sha512 {
	const ID: &'static str = CONTEXT_STRING;

	type Group = Ed25519Group;

	type HashOutput = [u8; 64];

	type SignatureSerialization = [u8; 64];

	/// H1 for FROST(Ed25519, SHA-512)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.1-2.2.2.1
	fn H1(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		WrappedScalar(hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"rho", m]))
	}

	/// H2 for FROST(Ed25519, SHA-512)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.1-2.2.2.2
	fn H2(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		WrappedScalar(hash_to_scalar(&[m]))
	}

	/// H3 for FROST(Ed25519, SHA-512)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.1-2.2.2.3
	fn H3(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		WrappedScalar(hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"nonce", m]))
	}

	/// H4 for FROST(Ed25519, SHA-512)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.1-2.2.2.4
	fn H4(m: &[u8]) -> Self::HashOutput {
		hash_to_array(&[CONTEXT_STRING.as_bytes(), b"msg", m])
	}

	/// H5 for FROST(Ed25519, SHA-512)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.1-2.2.2.5
	fn H5(m: &[u8]) -> Self::HashOutput {
		hash_to_array(&[CONTEXT_STRING.as_bytes(), b"com", m])
	}

	/// HDKG for FROST(Ed25519, SHA-512)
	fn HDKG(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
		Some(WrappedScalar(hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"dkg", m])))
	}

	/// HID for FROST(Ed25519, SHA-512)
	fn HID(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
		Some(WrappedScalar(hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"id", m])))
	}
}
