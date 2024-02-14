use core::fmt::Debug;

use crate::{
	error::Error,
	identifier::Identifier,
	serialization::{Deserialize, ElementSerialization, Serialize},
	traits::{Ciphersuite, Element, Group},
	util::element_is_valid,
	verifying_key::VerifyingKey,
	Header,
};
use alloc::collections::BTreeMap;
use sp_std::vec::Vec;

/// A public group element that represents a single signer's public verification share.
#[derive(Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
#[serde(try_from = "ElementSerialization<C>")]
#[serde(into = "ElementSerialization<C>")]
pub struct VerifyingShare<C>(pub(super) Element<C>)
where
	C: Ciphersuite;

impl<C> VerifyingShare<C>
where
	C: Ciphersuite,
{
	/// Create a new [`VerifyingShare`] from a element.
	pub fn new(element: Element<C>) -> Self {
		Self(element)
	}

	/// Get the inner element.
	#[cfg(feature = "internals")]
	pub fn to_element(&self) -> Element<C> {
		self.0
	}

	/// Deserialize from bytes
	pub fn deserialize(bytes: <C::Group as Group>::Serialization) -> Result<Self, Error> {
		<C::Group as Group>::deserialize(&bytes)
			.map(|element| Self(element))
			.map_err(|e| e.into())
	}

	/// Serialize to bytes
	pub fn serialize(&self) -> <C::Group as Group>::Serialization {
		<C::Group as Group>::serialize(&self.0)
	}

	/// Verifies that a verifying share is valid aka not zero or the base point
	pub fn is_valid(&self) -> bool {
		element_is_valid::<C>(&self.0)
	}
}

impl<C> Debug for VerifyingShare<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		f.debug_tuple("VerifyingShare").field(&hex::encode(self.serialize())).finish()
	}
}

impl<C> TryFrom<ElementSerialization<C>> for VerifyingShare<C>
where
	C: Ciphersuite,
{
	type Error = Error;

	fn try_from(value: ElementSerialization<C>) -> Result<Self, Self::Error> {
		Self::deserialize(value.0)
	}
}

impl<C> From<VerifyingShare<C>> for ElementSerialization<C>
where
	C: Ciphersuite,
{
	fn from(value: VerifyingShare<C>) -> Self {
		Self(value.serialize())
	}
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
#[serde(deny_unknown_fields)]
pub struct PublicKeyPackage<C: Ciphersuite> {
	/// Serialization header
	pub header: Header<C>,
	/// The verifying shares for all participants. Used to validate signature
	/// shares they generate.
	pub verifying_shares: BTreeMap<Identifier<C>, VerifyingShare<C>>,
	/// The joint public key for the entire group.
	pub verifying_key: VerifyingKey<C>,
}

impl<C> PublicKeyPackage<C>
where
	C: Ciphersuite,
{
	/// Serialize the struct into a Vec.
	pub fn serialize(&self) -> Result<Vec<u8>, Error> {
		Serialize::serialize(&self)
	}

	/// Deserialize the struct from a slice of bytes.
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error> {
		Deserialize::deserialize(bytes)
	}
}

// Default byte-oriented serialization for structs that need to be communicated.
//
// Note that we still manually implement these methods in each applicable type,
// instead of making these traits `pub` and asking users to import the traits.
// The reason is that ciphersuite traits would need to re-export these traits,
// parametrized with the ciphersuite, but trait aliases are not currently
// supported: <https://github.com/rust-lang/rust/issues/41517>

#[cfg(feature = "serialization")]
pub(crate) trait Serialize<C: Ciphersuite> {
	/// Serialize the struct into a Vec.
	fn serialize(&self) -> Result<Vec<u8>, Error>;
}

#[cfg(feature = "serialization")]
pub(crate) trait Deserialize<C: Ciphersuite> {
	/// Deserialize the struct from a slice of bytes.
	fn deserialize(bytes: &[u8]) -> Result<Self, Error>
	where
		Self: std::marker::Sized;
}

#[cfg(feature = "serialization")]
impl<T: serde::Serialize, C: Ciphersuite> Serialize<C> for T {
	fn serialize(&self) -> Result<Vec<u8>, Error> {
		postcard::to_stdvec(self).map_err(|_| Error::SerializationError)
	}
}

#[cfg(feature = "serialization")]
impl<T: for<'de> serde::Deserialize<'de>, C: Ciphersuite> Deserialize<C> for T {
	fn deserialize(bytes: &[u8]) -> Result<Self, Error> {
		postcard::from_bytes(bytes).map_err(|_| Error::DeserializationError)
	}
}
