use core::ops::{Add, Mul, Neg, Sub};

use ed448_goldilocks::{CompressedEdwardsY, EdwardsPoint, Scalar, ScalarBytes};
use parity_scale_codec::{Decode, Encode};
use subtle::{Choice, ConditionallyNegatable, ConditionallySelectable};

/// A wrapper around a [`ed448_goldilocks::Scalar`] to implement the [`Encode`,`Decode`]
/// traits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WrappedScalar(pub Scalar);

impl Encode for WrappedScalar {
	fn size_hint(&self) -> usize {
		56
	}

	fn encode_to<W: parity_scale_codec::Output + ?Sized>(&self, dest: &mut W) {
		dest.write(self.0.to_bytes().as_ref());
	}
}

impl Decode for WrappedScalar {
	fn decode<I: parity_scale_codec::Input>(
		input: &mut I,
	) -> Result<Self, parity_scale_codec::Error> {
		let mut bytes = [0u8; 56];
		input.read(&mut bytes)?;
		let buffer = ScalarBytes::from_slice(&bytes);
		WrappedScalar(Scalar::from_canonical_bytes(buffer))
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

/// A wrapper around a [`curve25519_dalek::edwards::EdwardsPoint`] to implement the
/// [`Encode`,`Decode`] traits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WrappedEdwardsPoint(pub EdwardsPoint);

impl Encode for WrappedEdwardsPoint {
	fn size_hint(&self) -> usize {
		57
	}

	fn encode_to<W: parity_scale_codec::Output + ?Sized>(&self, dest: &mut W) {
		dest.write(self.0.compress().as_bytes());
	}
}

impl Decode for WrappedEdwardsPoint {
	fn decode<I: parity_scale_codec::Input>(
		input: &mut I,
	) -> Result<Self, parity_scale_codec::Error> {
		let mut bytes = [0u8; 57];
		input.read(&mut bytes)?;
		Ok(WrappedEdwardsPoint(
			CompressedEdwardsY(bytes).decompress().unwrap_or(EdwardsPoint::default()),
		))
	}
}

impl Sub for WrappedEdwardsPoint {
	type Output = WrappedEdwardsPoint;

	fn sub(self, rhs: WrappedEdwardsPoint) -> WrappedEdwardsPoint {
		WrappedEdwardsPoint(self.0 - rhs.0)
	}
}

impl Add for WrappedEdwardsPoint {
	type Output = WrappedEdwardsPoint;

	fn add(self, rhs: WrappedEdwardsPoint) -> WrappedEdwardsPoint {
		WrappedEdwardsPoint(self.0 + rhs.0)
	}
}

impl Mul<WrappedScalar> for WrappedEdwardsPoint {
	type Output = WrappedEdwardsPoint;

	fn mul(self, rhs: WrappedScalar) -> WrappedEdwardsPoint {
		WrappedEdwardsPoint(self.0 * rhs.0)
	}
}

impl Neg for WrappedEdwardsPoint {
	type Output = WrappedEdwardsPoint;

	fn neg(self) -> WrappedEdwardsPoint {
		WrappedEdwardsPoint(-self.0)
	}
}

impl ConditionallySelectable for WrappedEdwardsPoint {
	fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
		WrappedEdwardsPoint(EdwardsPoint::conditional_select(&a.0, &b.0, choice))
	}
}

impl ConditionallyNegatable for WrappedEdwardsPoint {
	fn conditional_negate(&mut self, choice: Choice) {
		self.0.conditional_negate(choice);
	}
}
