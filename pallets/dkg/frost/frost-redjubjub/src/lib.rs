#![cfg_attr(not(feature = "std"), no_std)]

mod constants;
mod hash;
pub mod types;
pub use types::*;

use group::{
	cofactor::CofactorGroup,
	ff::{Field as FFField, PrimeField},
	Group as GGroup, GroupEncoding,
};
use jubjub::{ExtendedPoint, SubgroupPoint};

// Re-exports in our public API
pub use frost_core::{
	error::{FieldError, GroupError},
	traits::{Ciphersuite, Field, Group},
};

#[cfg(feature = "std")]
use rand_core::{CryptoRng, RngCore};

use crate::hash::HStar;

/// The context string for FROST(Jubjub, BLAKE2b-512).
/// TODO: this hasn't been formalized yet, so it's subject to change.
const CONTEXT_STRING: &str = "FROST-RedJubjub-BLAKE2b-512-v1";

fn hash_to_array(inputs: &[&[u8]]) -> [u8; 64] {
	let mut state = HStar::default();
	for i in &inputs[1..] {
		state.update(i);
	}
	*state.state.finalize().as_array()
}
fn hash_to_scalar(domain: &[u8], msg: &[u8]) -> jubjub::Scalar {
	HStar::default().update(domain).update(msg).finalize()
}

/// An implementation of the FROST(Jubjub, BLAKE2b-512) ciphersuite scalar field.
#[derive(Clone, Copy)]
pub struct JubjubScalarField;

impl Field for JubjubScalarField {
	type Scalar = WrappedScalar;

	type Serialization = [u8; 32];

	fn zero() -> Self::Scalar {
		WrappedScalar(jubjub::Scalar::zero())
	}

	fn one() -> Self::Scalar {
		WrappedScalar(jubjub::Scalar::one())
	}

	fn invert(scalar: &Self::Scalar) -> Result<Self::Scalar, FieldError> {
		// [`Jubjub::Scalar`]'s Eq/PartialEq does a constant-time comparison using
		// `ConstantTimeEq`
		if *scalar == <Self as Field>::zero() {
			Err(FieldError::InvalidZeroScalar)
		} else {
			Ok(WrappedScalar(jubjub::Scalar::invert(&scalar.0).unwrap()))
		}
	}

	#[cfg(feature = "std")]
	fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self::Scalar {
		WrappedScalar(jubjub::Scalar::random(rng))
	}

	fn serialize(scalar: &Self::Scalar) -> Self::Serialization {
		scalar.0.to_bytes()
	}

	fn little_endian_serialize(scalar: &Self::Scalar) -> Self::Serialization {
		Self::serialize(scalar)
	}

	fn deserialize(buf: &Self::Serialization) -> Result<Self::Scalar, FieldError> {
		match jubjub::Scalar::from_repr(*buf).into() {
			Some(s) => Ok(WrappedScalar(s)),
			None => Err(FieldError::MalformedScalar),
		}
	}
}

/// An implementation of the FROST(Jubjub, BLAKE2b-512) ciphersuite group.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct JubjubGroup;

impl Group for JubjubGroup {
	type Field = JubjubScalarField;

	type Element = WrappedSubgroupPoint;

	type Serialization = [u8; 32];

	fn cofactor() -> <Self::Field as Field>::Scalar {
		Self::Field::one()
	}

	fn identity() -> Self::Element {
		WrappedSubgroupPoint(SubgroupPoint::identity())
	}

	fn generator() -> Self::Element {
		let pt: ExtendedPoint =
			jubjub::AffinePoint::from_bytes(&constants::SPENDAUTHSIG_BASEPOINT_BYTES)
				.unwrap()
				.into();
		WrappedSubgroupPoint(pt.into_subgroup().unwrap())
	}

	fn serialize(element: &Self::Element) -> Self::Serialization {
		element.0.to_bytes()
	}

	fn deserialize(buf: &Self::Serialization) -> Result<Self::Element, GroupError> {
		let point = SubgroupPoint::from_bytes(buf);

		match Option::<SubgroupPoint>::from(point) {
			Some(point) =>
				if point == SubgroupPoint::identity() {
					Err(GroupError::InvalidIdentityElement)
				} else {
					Ok(WrappedSubgroupPoint(point))
				},
			None => Err(GroupError::MalformedElement),
		}
	}
}

/// An implementation of the FROST(Jubjub, BLAKE2b-512) ciphersuite.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct JubjubBlake2b512;

impl Ciphersuite for JubjubBlake2b512 {
	const ID: &'static str = CONTEXT_STRING;

	type Group = JubjubGroup;

	type HashOutput = [u8; 64];

	type SignatureSerialization = [u8; 64];

	/// H1 for FROST(Jubjub, BLAKE2b-512)
	fn H1(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		WrappedScalar(hash_to_scalar((CONTEXT_STRING.to_owned() + "rho").as_bytes(), m))
	}

	/// H2 for FROST(Jubjub, BLAKE2b-512)
	fn H2(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		WrappedScalar(HStar::default().update(m).finalize())
	}

	/// H3 for FROST(Jubjub, BLAKE2b-512)
	fn H3(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		WrappedScalar(hash_to_scalar((CONTEXT_STRING.to_owned() + "nonce").as_bytes(), m))
	}

	/// H4 for FROST(Jubjub, BLAKE2b-512)
	fn H4(m: &[u8]) -> Self::HashOutput {
		hash_to_array(&[CONTEXT_STRING.as_bytes(), b"msg", m])
	}

	/// H5 for FROST(Jubjub, BLAKE2b-512)
	fn H5(m: &[u8]) -> Self::HashOutput {
		hash_to_array(&[CONTEXT_STRING.as_bytes(), b"com", m])
	}

	/// HDKG for FROST(Jubjub, BLAKE2b-512)
	fn HDKG(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
		Some(WrappedScalar(hash_to_scalar((CONTEXT_STRING.to_owned() + "dkg").as_bytes(), m)))
	}

	/// HID for FROST(Jubjub, BLAKE2b-512)
	fn HID(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
		Some(WrappedScalar(hash_to_scalar((CONTEXT_STRING.to_owned() + "id").as_bytes(), m)))
	}
}
