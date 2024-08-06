use core::ops::{Add, Mul, Neg, Sub};

use group::{ff::Field as FFField, GroupEncoding};
use jubjub::{Scalar, SubgroupPoint};
use parity_scale_codec::{Decode, Encode};
use subtle::{Choice, ConditionallyNegatable, ConditionallySelectable};

/// A wrapper around a [`ed448_goldilocks::Scalar`] to implement the [`Encode`,`Decode`]
/// traits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WrappedScalar(pub Scalar);

impl Encode for WrappedScalar {
	fn size_hint(&self) -> usize {
		32
	}

	fn encode_to<W: parity_scale_codec::Output + ?Sized>(&self, dest: &mut W) {
		dest.write(self.0.to_bytes().as_ref());
	}
}

impl Decode for WrappedScalar {
	fn decode<I: parity_scale_codec::Input>(
		input: &mut I,
	) -> Result<Self, parity_scale_codec::Error> {
		let mut bytes = [0u8; 32];
		input.read(&mut bytes)?;
		Ok(WrappedScalar(Scalar::from_bytes(&bytes).unwrap_or(Scalar::ZERO)))
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
pub struct WrappedSubgroupPoint(pub SubgroupPoint);

impl Encode for WrappedSubgroupPoint {
	fn size_hint(&self) -> usize {
		32
	}

	fn encode_to<W: parity_scale_codec::Output + ?Sized>(&self, dest: &mut W) {
		dest.write(self.0.to_bytes().as_ref());
	}
}

impl Decode for WrappedSubgroupPoint {
	fn decode<I: parity_scale_codec::Input>(
		input: &mut I,
	) -> Result<Self, parity_scale_codec::Error> {
		let mut bytes = [0u8; 32];
		input.read(&mut bytes)?;
		Ok(WrappedSubgroupPoint(
			SubgroupPoint::from_bytes(&bytes).unwrap_or(SubgroupPoint::default()),
		))
	}
}

impl Sub for WrappedSubgroupPoint {
	type Output = WrappedSubgroupPoint;

	fn sub(self, rhs: WrappedSubgroupPoint) -> WrappedSubgroupPoint {
		WrappedSubgroupPoint(self.0 - rhs.0)
	}
}

impl Add for WrappedSubgroupPoint {
	type Output = WrappedSubgroupPoint;

	fn add(self, rhs: WrappedSubgroupPoint) -> WrappedSubgroupPoint {
		WrappedSubgroupPoint(self.0 + rhs.0)
	}
}

impl Mul<WrappedScalar> for WrappedSubgroupPoint {
	type Output = WrappedSubgroupPoint;

	fn mul(self, rhs: WrappedScalar) -> WrappedSubgroupPoint {
		WrappedSubgroupPoint(self.0 * rhs.0)
	}
}

impl Neg for WrappedSubgroupPoint {
	type Output = WrappedSubgroupPoint;

	fn neg(self) -> WrappedSubgroupPoint {
		WrappedSubgroupPoint(-self.0)
	}
}

impl ConditionallySelectable for WrappedSubgroupPoint {
	fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
		WrappedSubgroupPoint(SubgroupPoint::conditional_select(&a.0, &b.0, choice))
	}
}

impl ConditionallyNegatable for WrappedSubgroupPoint {
	fn conditional_negate(&mut self, choice: Choice) {
		self.0.conditional_negate(choice);
	}
}
