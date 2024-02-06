#![cfg_attr(not(feature = "std"), no_std)]

use curve25519_dalek::{
	constants::RISTRETTO_BASEPOINT_POINT,
	ristretto::{CompressedRistretto, RistrettoPoint},
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

/// An implementation of the FROST(ristretto255, SHA-512) ciphersuite scalar field.
#[derive(Clone, Copy)]
pub struct RistrettoScalarField;

impl Field for RistrettoScalarField {
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

/// An implementation of the FROST(ristretto255, SHA-512) ciphersuite group.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RistrettoGroup;

impl Group for RistrettoGroup {
	type Field = RistrettoScalarField;

	type Element = WrappedRistrettoPoint;

	type Serialization = [u8; 32];

	fn cofactor() -> <Self::Field as Field>::Scalar {
		WrappedScalar(Scalar::ONE)
	}

	fn identity() -> Self::Element {
		WrappedRistrettoPoint(RistrettoPoint::identity())
	}

	fn generator() -> Self::Element {
		WrappedRistrettoPoint(RISTRETTO_BASEPOINT_POINT)
	}

	fn serialize(element: &Self::Element) -> Self::Serialization {
		element.0.compress().to_bytes()
	}

	fn deserialize(buf: &Self::Serialization) -> Result<Self::Element, GroupError> {
		match CompressedRistretto::from_slice(buf.as_ref())
			.map_err(|_| GroupError::MalformedElement)?
			.decompress()
		{
			Some(point) =>
				if point == RistrettoPoint::identity() {
					Err(GroupError::InvalidIdentityElement)
				} else {
					Ok(WrappedRistrettoPoint(point))
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
const CONTEXT_STRING: &str = "FROST-RISTRETTO255-SHA512-v1";

/// An implementation of the FROST(ristretto255, SHA-512) ciphersuite.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ristretto255Sha512;

impl Ciphersuite for Ristretto255Sha512 {
	const ID: &'static str = CONTEXT_STRING;

	type Group = RistrettoGroup;

	type HashOutput = [u8; 64];

	type SignatureSerialization = [u8; 64];

	/// H1 for FROST(ristretto255, SHA-512)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.2-2.2.2.1
	fn H1(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		WrappedScalar(hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"rho", m]))
	}

	/// H2 for FROST(ristretto255, SHA-512)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.2-2.2.2.2
	fn H2(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		WrappedScalar(hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"chal", m]))
	}

	/// H3 for FROST(ristretto255, SHA-512)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.2-2.2.2.3
	fn H3(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		WrappedScalar(hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"nonce", m]))
	}

	/// H4 for FROST(ristretto255, SHA-512)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.2-2.2.2.4
	fn H4(m: &[u8]) -> Self::HashOutput {
		hash_to_array(&[CONTEXT_STRING.as_bytes(), b"msg", m])
	}

	/// H5 for FROST(ristretto255, SHA-512)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.2-2.2.2.5
	fn H5(m: &[u8]) -> Self::HashOutput {
		hash_to_array(&[CONTEXT_STRING.as_bytes(), b"com", m])
	}

	/// HDKG for FROST(ristretto255, SHA-512)
	fn HDKG(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
		Some(WrappedScalar(hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"dkg", m])))
	}

	/// HID for FROST(ristretto255, SHA-512)
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

		let sk = SigningKey::<Ristretto255Sha512>::new(&mut rng);
		let vk = VerifyingKey::<Ristretto255Sha512>::from(sk);

		let msg = b"Hello, world!";
		let signature = sk.sign(&mut rng, msg);
		assert!(vk.verify(msg, &signature).is_ok());
	}
}
