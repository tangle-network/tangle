use core::ops::{Add, Mul, Neg, Sub};

use p384::{
	elliptic_curve::{
		sec1::{FromEncodedPoint, ToEncodedPoint},
		PrimeField,
	},
	EncodedPoint, FieldBytes, ProjectivePoint, Scalar,
};
use parity_scale_codec::{Decode, Encode};
use subtle::{Choice, ConditionallyNegatable, ConditionallySelectable};

/// A wrapper around a [`p384::Scalar`] to implement the [`Encode`,`Decode`]
/// traits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WrappedScalar(pub Scalar);

impl Encode for WrappedScalar {
	fn size_hint(&self) -> usize {
		48
	}

	fn encode_to<W: parity_scale_codec::Output + ?Sized>(&self, dest: &mut W) {
		dest.write(self.0.to_repr().encode().as_ref());
	}
}

impl Decode for WrappedScalar {
	fn decode<I: parity_scale_codec::Input>(
		input: &mut I,
	) -> Result<Self, parity_scale_codec::Error> {
		let mut bytes = [0u8; 32];
		input.read(&mut bytes)?;
		Ok(WrappedScalar(
			Scalar::from_repr(*FieldBytes::from_slice(&bytes)).unwrap_or(Scalar::ZERO),
		))
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

/// A wrapper around a [`p384::ProjectivePoint`] to implement the
/// [`Encode`,`Decode`] traits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WrappedProjectivePoint(pub ProjectivePoint);

impl Encode for WrappedProjectivePoint {
	fn size_hint(&self) -> usize {
		49
	}

	fn encode_to<W: parity_scale_codec::Output + ?Sized>(&self, dest: &mut W) {
		dest.write(self.0.to_encoded_point(true).as_bytes());
	}
}

impl Decode for WrappedProjectivePoint {
	fn decode<I: parity_scale_codec::Input>(
		input: &mut I,
	) -> Result<Self, parity_scale_codec::Error> {
		let mut bytes = [0u8; 32];
		input.read(&mut bytes)?;
		let pt = ProjectivePoint::from_encoded_point(
			&EncodedPoint::from_bytes(bytes).unwrap_or_default(),
		)
		.unwrap_or(ProjectivePoint::default());
		Ok(WrappedProjectivePoint(pt))
	}
}

impl Sub for WrappedProjectivePoint {
	type Output = WrappedProjectivePoint;

	fn sub(self, rhs: WrappedProjectivePoint) -> WrappedProjectivePoint {
		WrappedProjectivePoint(self.0 - rhs.0)
	}
}

impl Add for WrappedProjectivePoint {
	type Output = WrappedProjectivePoint;

	fn add(self, rhs: WrappedProjectivePoint) -> WrappedProjectivePoint {
		WrappedProjectivePoint(self.0 + rhs.0)
	}
}

impl Mul<WrappedScalar> for WrappedProjectivePoint {
	type Output = WrappedProjectivePoint;

	fn mul(self, rhs: WrappedScalar) -> WrappedProjectivePoint {
		WrappedProjectivePoint(self.0 * rhs.0)
	}
}

impl Neg for WrappedProjectivePoint {
	type Output = WrappedProjectivePoint;

	fn neg(self) -> WrappedProjectivePoint {
		WrappedProjectivePoint(-self.0)
	}
}

impl ConditionallySelectable for WrappedProjectivePoint {
	fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
		WrappedProjectivePoint(ProjectivePoint::conditional_select(&a.0, &b.0, choice))
	}
}

impl ConditionallyNegatable for WrappedProjectivePoint {
	fn conditional_negate(&mut self, choice: Choice) {
		self.0.conditional_negate(choice);
	}
}
