#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use crate::alloc::borrow::ToOwned;
use p256::{
	elliptic_curve::{
		hash2curve::{hash_to_field, ExpandMsgXmd},
		sec1::{FromEncodedPoint, ToEncodedPoint},
		PrimeField,
	},
	AffinePoint, ProjectivePoint, Scalar,
};

#[cfg(feature = "std")]
use rand_core::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};

pub mod types;
pub use types::*;

// Re-exports in our public API
pub use frost_core::{
	error::{FieldError, GroupError},
	traits::{Ciphersuite, Field, Group},
};

#[cfg(feature = "std")]
pub use rand_core;

/// An implementation of the FROST(P-256, SHA-256) ciphersuite scalar field.
#[derive(Clone, Copy)]
pub struct P256ScalarField;

impl Field for P256ScalarField {
	type Scalar = WrappedScalar;

	type Serialization = [u8; 32];

	fn zero() -> Self::Scalar {
		WrappedScalar(Scalar::ZERO)
	}

	fn one() -> Self::Scalar {
		WrappedScalar(Scalar::ONE)
	}

	fn invert(scalar: &Self::Scalar) -> Result<Self::Scalar, FieldError> {
		// [`p256::Scalar`]'s Eq/PartialEq does a constant-time comparison using
		// `ConstantTimeEq`
		if *scalar == <Self as Field>::zero() {
			Err(FieldError::InvalidZeroScalar)
		} else {
			Ok(WrappedScalar(scalar.0.invert().unwrap()))
		}
	}

	#[cfg(feature = "std")]
	fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self::Scalar {
		use p256::elliptic_curve::Field;

		WrappedScalar(Scalar::random(rng))
	}

	fn serialize(scalar: &Self::Scalar) -> Self::Serialization {
		scalar.0.to_bytes().into()
	}

	fn deserialize(buf: &Self::Serialization) -> Result<Self::Scalar, FieldError> {
		let field_bytes: &p256::FieldBytes = buf.into();
		match Scalar::from_repr(*field_bytes).into() {
			Some(s) => Ok(WrappedScalar(s)),
			None => Err(FieldError::MalformedScalar),
		}
	}

	fn little_endian_serialize(scalar: &Self::Scalar) -> Self::Serialization {
		let mut array = Self::serialize(scalar);
		array.reverse();
		array
	}
}

/// An implementation of the FROST(P-256, SHA-256) ciphersuite group.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct P256Group;

impl Group for P256Group {
	type Field = P256ScalarField;

	type Element = WrappedProjectivePoint;

	/// [SEC 1][1] serialization of a compressed point in P-256 takes 33 bytes
	/// (1-byte prefix and 32 bytes for the coordinate).
	///
	/// Note that, in the P-256 spec, the identity is encoded as a single null byte;
	/// but here we pad with zeroes. This is acceptable as the identity _should_ never
	/// be serialized in FROST, else we error.
	///
	/// [1]: https://secg.org/sec1-v2.pdf
	type Serialization = [u8; 33];

	fn cofactor() -> <Self::Field as Field>::Scalar {
		WrappedScalar(Scalar::ONE)
	}

	fn identity() -> Self::Element {
		WrappedProjectivePoint(ProjectivePoint::IDENTITY)
	}

	fn generator() -> Self::Element {
		WrappedProjectivePoint(ProjectivePoint::GENERATOR)
	}

	fn serialize(element: &Self::Element) -> Self::Serialization {
		let mut fixed_serialized = [0; 33];
		let serialized_point = element.0.to_encoded_point(true);
		let serialized = serialized_point.as_bytes();
		// Sanity check; either it takes all bytes or a single byte (identity).
		assert!(serialized.len() == fixed_serialized.len() || serialized.len() == 1);
		// Copy to the left of the buffer (i.e. pad the identity with zeroes).
		// Note that identity elements shouldn't be serialized in FROST, but we
		// do this padding so that this function doesn't have to return an error.
		// If this encodes the identity, it will fail when deserializing.
		{
			let (left, _right) = fixed_serialized.split_at_mut(serialized.len());
			left.copy_from_slice(serialized);
		}
		fixed_serialized
	}

	fn deserialize(buf: &Self::Serialization) -> Result<Self::Element, GroupError> {
		let encoded_point =
			p256::EncodedPoint::from_bytes(buf).map_err(|_| GroupError::MalformedElement)?;

		match Option::<AffinePoint>::from(AffinePoint::from_encoded_point(&encoded_point)) {
			Some(point) => {
				if point.is_identity().into() {
					// This is actually impossible since the identity is encoded in a single byte
					// which will never happen since we receive a 33-byte buffer.
					// We leave the check for consistency.
					Err(GroupError::InvalidIdentityElement)
				} else {
					Ok(WrappedProjectivePoint(ProjectivePoint::from(point)))
				}
			},
			None => Err(GroupError::MalformedElement),
		}
	}
}

fn hash_to_array(inputs: &[&[u8]]) -> [u8; 32] {
	let mut h = Sha256::new();
	for i in inputs {
		h.update(i);
	}
	let mut output = [0u8; 32];
	output.copy_from_slice(h.finalize().as_slice());
	output
}

fn hash_to_scalar(domain: &[u8], msg: &[u8]) -> Scalar {
	let mut u = [P256ScalarField::zero().0];
	hash_to_field::<ExpandMsgXmd<Sha256>, Scalar>(&[msg], &[domain], &mut u)
		.expect("should never return error according to error cases described in ExpandMsgXmd");
	u[0]
}

/// Context string from the ciphersuite in the [spec]
///
/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.4-1
const CONTEXT_STRING: &str = "FROST-P256-SHA256-v1";

/// An implementation of the FROST(P-256, SHA-256) ciphersuite.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct P256Sha256;

impl Ciphersuite for P256Sha256 {
	const ID: &'static str = CONTEXT_STRING;

	type Group = P256Group;

	type HashOutput = [u8; 32];

	type SignatureSerialization = [u8; 65];

	/// H1 for FROST(P-256, SHA-256)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.4-2.2.2.1
	fn H1(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		WrappedScalar(hash_to_scalar((CONTEXT_STRING.to_owned() + "rho").as_bytes(), m))
	}

	/// H2 for FROST(P-256, SHA-256)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.4-2.2.2.2
	fn H2(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		WrappedScalar(hash_to_scalar((CONTEXT_STRING.to_owned() + "chal").as_bytes(), m))
	}

	/// H3 for FROST(P-256, SHA-256)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.4-2.2.2.3
	fn H3(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		WrappedScalar(hash_to_scalar((CONTEXT_STRING.to_owned() + "nonce").as_bytes(), m))
	}

	/// H4 for FROST(P-256, SHA-256)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.4-2.2.2.4
	fn H4(m: &[u8]) -> Self::HashOutput {
		hash_to_array(&[CONTEXT_STRING.as_bytes(), b"msg", m])
	}

	/// H5 for FROST(P-256, SHA-256)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.4-2.2.2.5
	fn H5(m: &[u8]) -> Self::HashOutput {
		hash_to_array(&[CONTEXT_STRING.as_bytes(), b"com", m])
	}

	/// HDKG for FROST(P-256, SHA-256)
	fn HDKG(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
		Some(WrappedScalar(hash_to_scalar((CONTEXT_STRING.to_owned() + "dkg").as_bytes(), m)))
	}

	/// HID for FROST(P-256, SHA-256)
	fn HID(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
		Some(WrappedScalar(hash_to_scalar((CONTEXT_STRING.to_owned() + "id").as_bytes(), m)))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frost_core::{signing_key::SigningKey, verifying_key::VerifyingKey};

	#[test]
	fn test_sign_and_verify() {
		let mut rng = rand_core::OsRng;

		let sk = SigningKey::<P256Sha256>::new(&mut rng);
		let vk = VerifyingKey::<P256Sha256>::from(sk);

		let msg = b"Hello, world!";
		let signature = sk.sign(rng, msg);
		assert!(vk.verify(msg, &signature).is_ok());
	}
}
