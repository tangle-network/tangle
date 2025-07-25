//! FROST participant identifiers

use core::{
	fmt::{self, Debug},
	hash::{Hash, Hasher},
};

use alloc::vec::Vec;

use crate::{
	Ciphersuite, Error, Field, FieldError, Group, Scalar, serialization::SerializableScalar,
};

/// A FROST participant identifier.
///
/// The identifier is a field element in the scalar field that the secret polynomial is defined
/// over, corresponding to some x-coordinate for a polynomial f(x) = y.  MUST NOT be zero in the
/// field, as f(0) = the shared secret.
#[derive(Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "C: Ciphersuite"))]
// We use these to add a validation step since zero scalars should cause an
// error when deserializing.
#[cfg_attr(feature = "serde", serde(try_from = "SerializableScalar<C>"))]
#[cfg_attr(feature = "serde", serde(into = "SerializableScalar<C>"))]
pub struct Identifier<C: Ciphersuite>(SerializableScalar<C>);

impl<C> Identifier<C>
where
	C: Ciphersuite,
{
	/// Create a new Identifier from a scalar. For internal use only.
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
	pub(crate) fn new(scalar: Scalar<C>) -> Result<Self, Error<C>> {
		if scalar == <<C::Group as Group>::Field>::zero() {
			Err(FieldError::InvalidZeroScalar.into())
		} else {
			Ok(Self(SerializableScalar(scalar)))
		}
	}

	/// Get the inner scalar.
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
	pub(crate) fn to_scalar(&self) -> Scalar<C> {
		self.0.0
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
	pub fn derive(s: &[u8]) -> Result<Self, Error<C>> {
		let scalar = C::HID(s).ok_or(Error::IdentifierDerivationNotSupported)?;
		Self::new(scalar)
	}

	/// Serialize the identifier using the ciphersuite encoding.
	pub fn serialize(&self) -> Vec<u8> {
		self.0.serialize()
	}

	/// Deserialize an Identifier from a serialized buffer.
	/// Returns an error if it attempts to deserialize zero.
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error<C>> {
		Self::new(SerializableScalar::deserialize(bytes)?.0)
	}
}

#[cfg(feature = "serde")]
impl<C> TryFrom<SerializableScalar<C>> for Identifier<C>
where
	C: Ciphersuite,
{
	type Error = Error<C>;

	fn try_from(s: SerializableScalar<C>) -> Result<Self, Self::Error> {
		Self::new(s.0)
	}
}

#[cfg(feature = "serde")]
impl<C> From<Identifier<C>> for SerializableScalar<C>
where
	C: Ciphersuite,
{
	fn from(i: Identifier<C>) -> Self {
		i.0
	}
}

impl<C> Eq for Identifier<C> where C: Ciphersuite {}

impl<C> Debug for Identifier<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_tuple("Identifier").field(&hex::encode(self.serialize())).finish()
	}
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl<C> Hash for Identifier<C>
where
	C: Ciphersuite,
{
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.serialize().hash(state)
	}
}

impl<C> Ord for Identifier<C>
where
	C: Ciphersuite,
{
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		let serialized_self =
			<<C::Group as Group>::Field>::little_endian_serialize(&self.to_scalar());
		let serialized_other =
			<<C::Group as Group>::Field>::little_endian_serialize(&other.to_scalar());
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

impl<C> TryFrom<u16> for Identifier<C>
where
	C: Ciphersuite,
{
	type Error = Error<C>;

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
			Self::new(sum)
		}
	}
}
