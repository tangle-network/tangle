use core::ops::{Add, Mul, Neg, Sub};

use curve25519_dalek::{
	ristretto::{CompressedRistretto, RistrettoPoint},
	Scalar,
};
use parity_scale_codec::{Decode, Encode};
use subtle::{Choice, ConditionallyNegatable, ConditionallySelectable};

/// A wrapper around a [`curve25519_dalek::scalar::Scalar`] to implement the [`Encode`,`Decode`]
/// traits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WrappedScalar(pub Scalar);

impl Encode for WrappedScalar {
	fn size_hint(&self) -> usize {
		32
	}

	fn encode_to<W: parity_scale_codec::Output + ?Sized>(&self, dest: &mut W) {
		dest.write(self.0.as_bytes());
	}
}

impl Decode for WrappedScalar {
	fn decode<I: parity_scale_codec::Input>(
		input: &mut I,
	) -> Result<Self, parity_scale_codec::Error> {
		let mut bytes = [0u8; 32];
		input.read(&mut bytes)?;
		Ok(WrappedScalar(Scalar::from_canonical_bytes(bytes).unwrap_or(Scalar::ZERO)))
	}
}

impl Sub for WrappedScalar {
	type Output = WrappedScalar;

	fn sub(self, rhs: WrappedScalar) -> WrappedScalar {
		WrappedScalar(self.0 - rhs.0)
	}
}

impl Add for WrappedScalar {
	type Output = WrappedScalar;

	fn add(self, rhs: WrappedScalar) -> WrappedScalar {
		WrappedScalar(self.0 + rhs.0)
	}
}

impl Mul<WrappedScalar> for WrappedScalar {
	type Output = WrappedScalar;

	fn mul(self, rhs: WrappedScalar) -> WrappedScalar {
		WrappedScalar(self.0 * rhs.0)
	}
}

impl Neg for WrappedScalar {
	type Output = WrappedScalar;

	fn neg(self) -> WrappedScalar {
		WrappedScalar(-self.0)
	}
}

impl ConditionallySelectable for WrappedScalar {
	fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
		WrappedScalar(Scalar::conditional_select(&a.0, &b.0, choice))
	}
}

impl ConditionallyNegatable for WrappedScalar {
	fn conditional_negate(&mut self, choice: Choice) {
		self.0.conditional_negate(choice);
	}
}

/// A wrapper around a [`curve25519_dalek::edwards::RistrettoPoint`] to implement the
/// [`Encode`,`Decode`] traits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WrappedRistrettoPoint(pub RistrettoPoint);

impl Encode for WrappedRistrettoPoint {
	fn size_hint(&self) -> usize {
		32
	}

	fn encode_to<W: parity_scale_codec::Output + ?Sized>(&self, dest: &mut W) {
		dest.write(self.0.compress().as_bytes());
	}
}

impl Decode for WrappedRistrettoPoint {
	fn decode<I: parity_scale_codec::Input>(
		input: &mut I,
	) -> Result<Self, parity_scale_codec::Error> {
		let mut bytes = [0u8; 32];
		input.read(&mut bytes)?;
		Ok(WrappedRistrettoPoint(
			CompressedRistretto(bytes)
				.decompress()
				.ok_or(parity_scale_codec::Error::from("Invalid point"))?,
		))
	}
}

impl Sub for WrappedRistrettoPoint {
	type Output = WrappedRistrettoPoint;

	fn sub(self, rhs: WrappedRistrettoPoint) -> WrappedRistrettoPoint {
		WrappedRistrettoPoint(self.0 - rhs.0)
	}
}

impl Add for WrappedRistrettoPoint {
	type Output = WrappedRistrettoPoint;

	fn add(self, rhs: WrappedRistrettoPoint) -> WrappedRistrettoPoint {
		WrappedRistrettoPoint(self.0 + rhs.0)
	}
}

impl Mul<WrappedScalar> for WrappedRistrettoPoint {
	type Output = WrappedRistrettoPoint;

	fn mul(self, rhs: WrappedScalar) -> WrappedRistrettoPoint {
		WrappedRistrettoPoint(self.0 * rhs.0)
	}
}

impl Neg for WrappedRistrettoPoint {
	type Output = WrappedRistrettoPoint;

	fn neg(self) -> WrappedRistrettoPoint {
		WrappedRistrettoPoint(-self.0)
	}
}

impl ConditionallySelectable for WrappedRistrettoPoint {
	fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
		WrappedRistrettoPoint(RistrettoPoint::conditional_select(&a.0, &b.0, choice))
	}
}

impl ConditionallyNegatable for WrappedRistrettoPoint {
	fn conditional_negate(&mut self, choice: Choice) {
		self.0.conditional_negate(choice);
	}
}
