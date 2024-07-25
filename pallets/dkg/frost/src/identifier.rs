use core::{
	fmt::Debug,
	hash::{Hash, Hasher},
};

use crate::{
	error::{Error, FieldError},
	serialization::ScalarSerialization,
	traits::{Ciphersuite, Field, Group, Scalar},
	util::scalar_is_valid,
};

#[derive(Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
#[serde(try_from = "ScalarSerialization<C>")]
#[serde(into = "ScalarSerialization<C>")]
pub struct Identifier<C: Ciphersuite>(Scalar<C>);

impl<C> Identifier<C>
where
	C: Ciphersuite,
{
	/// Create a new Identifier from a scalar. For internal use only.
	fn new(scalar: Scalar<C>) -> Result<Self, Error> {
		if scalar == <<C::Group as Group>::Field>::zero() {
			Err(FieldError::InvalidZeroScalar.into())
		} else {
			Ok(Self(scalar))
		}
	}

	/// Derive an Identifier from an arbitrary byte string.
	///
	/// This feature is not part of the specification and is just a convenient
	/// way of creating identifiers.
	///
	/// Each possible byte string will map to an uniformly random identifier.
	/// Returns an error if the ciphersuite does not support identifier derivation,
	/// or if the mapped identifier is zero (which is unpredictable, but should happen
	/// with negligible probability).
	pub fn derive(s: &[u8]) -> Result<Self, Error> {
		let scalar = C::HID(s).ok_or(Error::IdentifierDerivationNotSupported)?;
		Self::new(scalar)
	}

	/// Serialize the identifier using the ciphersuite encoding.
	pub fn serialize(&self) -> <<C::Group as Group>::Field as Field>::Serialization {
		<<C::Group as Group>::Field>::serialize(&self.0)
	}

	/// Deserialize an Identifier from a serialized buffer.
	/// Returns an error if it attempts to deserialize zero.
	pub fn deserialize(
		buf: &<<C::Group as Group>::Field as Field>::Serialization,
	) -> Result<Self, Error> {
		let scalar = <<C::Group as Group>::Field>::deserialize(buf)?;
		Self::new(scalar)
	}

	/// Check if the identifier is valid aka not zero
	pub fn is_valid(&self) -> bool {
		scalar_is_valid::<C>(&self.0)
	}
}

impl<C> TryFrom<ScalarSerialization<C>> for Identifier<C>
where
	C: Ciphersuite,
{
	type Error = Error;

	fn try_from(value: ScalarSerialization<C>) -> Result<Self, Self::Error> {
		Self::deserialize(&value.0)
	}
}

impl<C> From<Identifier<C>> for ScalarSerialization<C>
where
	C: Ciphersuite,
{
	fn from(value: Identifier<C>) -> Self {
		Self(value.serialize())
	}
}

impl<C> Eq for Identifier<C> where C: Ciphersuite {}

impl<C> Debug for Identifier<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		f.debug_tuple("Identifier")
			.field(&hex::encode(<<C::Group as Group>::Field>::serialize(&self.0).as_ref()))
			.finish()
	}
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl<C> Hash for Identifier<C>
where
	C: Ciphersuite,
{
	fn hash<H: Hasher>(&self, state: &mut H) {
		<<C::Group as Group>::Field>::serialize(&self.0).as_ref().hash(state)
	}
}

impl<C> Ord for Identifier<C>
where
	C: Ciphersuite,
{
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		let serialized_self = <<C::Group as Group>::Field>::little_endian_serialize(&self.0);
		let serialized_other = <<C::Group as Group>::Field>::little_endian_serialize(&other.0);
		// The default cmp uses lexicographic order; so we need the elements in big endian
		serialized_self
			.as_ref()
			.iter()
			.rev()
			.cmp(serialized_other.as_ref().iter().rev())
	}
}

impl<C> PartialOrd for Identifier<C>
where
	C: Ciphersuite,
{
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<C> core::ops::Mul<Scalar<C>> for Identifier<C>
where
	C: Ciphersuite,
{
	type Output = Scalar<C>;

	fn mul(self, scalar: Scalar<C>) -> Scalar<C> {
		self.0 * scalar
	}
}

#[allow(uncovered_param_in_projection)]
impl<C> core::ops::MulAssign<Identifier<C>> for Scalar<C>
where
	C: Ciphersuite,
{
	fn mul_assign(&mut self, identifier: Identifier<C>) {
		*self = *self * identifier.0
	}
}

impl<C> core::ops::Sub for Identifier<C>
where
	C: Ciphersuite,
{
	type Output = Self;

	fn sub(self, rhs: Identifier<C>) -> Self::Output {
		Self(self.0 - rhs.0)
	}
}

impl<C> TryFrom<u16> for Identifier<C>
where
	C: Ciphersuite,
{
	type Error = Error;

	fn try_from(n: u16) -> Result<Identifier<C>, Self::Error> {
		if n == 0 {
			Err(FieldError::InvalidZeroScalar.into())
		} else {
			// Classic left-to-right double-and-add algorithm that skips the first bit 1 (since
			// identifiers are never zero, there is always a bit 1), thus `sum` starts with 1 too.
			let one = <<C::Group as Group>::Field>::one();
			let mut sum = <<C::Group as Group>::Field>::one();

			let bits = (n.to_be_bytes().len() as u32) * 8;
			for i in (0..(bits - n.leading_zeros() - 1)).rev() {
				sum = sum + sum;
				if n & (1 << i) != 0 {
					sum = sum + one;
				}
			}
			Ok(Self(sum))
		}
	}
}
