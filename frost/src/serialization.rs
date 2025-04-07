//! Serialization support.

use alloc::vec::Vec;

use crate::{Ciphersuite, FieldError};

use crate::{Element, Error, Field, Group};

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "internals", visibility::make(pub))]
#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
/// Helper struct to serialize a Scalar.
pub(crate) struct SerializableScalar<C: Ciphersuite>(
	pub <<<C as Ciphersuite>::Group as Group>::Field as Field>::Scalar,
);

impl<C> SerializableScalar<C>
where
	C: Ciphersuite,
{
	/// Serialize a Scalar.
	pub fn serialize(&self) -> Vec<u8> {
		<<C::Group as Group>::Field>::serialize(&self.0).as_ref().to_vec()
	}

	/// Deserialize a Scalar from a serialized buffer.
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error<C>> {
		let serialized: <<C::Group as Group>::Field as Field>::Serialization =
			bytes.to_vec().try_into().map_err(|_| FieldError::MalformedScalar)?;
		let scalar = <<C::Group as Group>::Field>::deserialize(&serialized)?;
		Ok(Self(scalar))
	}
}

#[cfg(feature = "serde")]
impl<C> serde::Serialize for SerializableScalar<C>
where
	C: Ciphersuite,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let serialized = <<C as Ciphersuite>::Group as Group>::Field::serialize(&self.0);
		serdect::array::serialize_hex_lower_or_bin(&serialized.as_ref(), serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de, C> serde::Deserialize<'de> for SerializableScalar<C>
where
	C: Ciphersuite,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		// Get size from the size of the zero scalar
		let zero = <<C::Group as Group>::Field as Field>::zero();
		let len = <<C::Group as Group>::Field as Field>::serialize(&zero).as_ref().len();

		let mut bytes = vec![0u8; len];
		serdect::array::deserialize_hex_or_bin(&mut bytes[..], deserializer)?;
		let array =
			bytes.try_into().map_err(|_| serde::de::Error::custom("invalid byte length"))?;
		<<C as Ciphersuite>::Group as Group>::Field::deserialize(&array)
			.map(|scalar| Self(scalar))
			.map_err(serde::de::Error::custom)
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct SerializableElement<C: Ciphersuite>(pub(crate) Element<C>);

impl<C> SerializableElement<C>
where
	C: Ciphersuite,
{
	/// Serialize an Element. Returns an error if it's the identity.
	pub fn serialize(&self) -> Result<Vec<u8>, Error<C>> {
		Ok(<C::Group as Group>::serialize(&self.0)?.as_ref().to_vec())
	}

	/// Deserialize an Element. Returns an error if it's malformed or is the
	/// identity.
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error<C>> {
		let serialized: <C::Group as Group>::Serialization =
			bytes.to_vec().try_into().map_err(|_| FieldError::MalformedScalar)?;
		let scalar = <C::Group as Group>::deserialize(&serialized)?;
		Ok(Self(scalar))
	}
}

#[cfg(feature = "serde")]
impl<C> serde::Serialize for SerializableElement<C>
where
	C: Ciphersuite,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let serialized =
			<C::Group as Group>::serialize(&self.0).map_err(serde::ser::Error::custom)?;
		serdect::array::serialize_hex_lower_or_bin(&serialized.as_ref(), serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de, C> serde::Deserialize<'de> for SerializableElement<C>
where
	C: Ciphersuite,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		// Get size from the size of the generator
		let generator = <C::Group>::generator();
		let len = <C::Group>::serialize(&generator)
			.expect("serializing the generator always works")
			.as_ref()
			.len();

		let mut bytes = vec![0u8; len];
		serdect::array::deserialize_hex_or_bin(&mut bytes[..], deserializer)?;
		let array =
			bytes.try_into().map_err(|_| serde::de::Error::custom("invalid byte length"))?;
		<C::Group as Group>::deserialize(&array)
			.map(|element| Self(element))
			.map_err(serde::de::Error::custom)
	}
}

// The short 4-byte ID. Derived as the CRC-32 of the UTF-8
// encoded ID in big endian format.
#[cfg(feature = "serde")]
const fn short_id<C>() -> [u8; 4]
where
	C: Ciphersuite,
{
	const_crc32::crc32(C::ID.as_bytes()).to_be_bytes()
}

/// Serialize a placeholder ciphersuite field with the ciphersuite ID string.
#[cfg(feature = "serde")]
pub(crate) fn ciphersuite_serialize<S, C>(_: &(), s: S) -> Result<S::Ok, S::Error>
where
	S: serde::Serializer,
	C: Ciphersuite,
{
	use serde::Serialize;

	if s.is_human_readable() {
		C::ID.serialize(s)
	} else {
		serde::Serialize::serialize(&short_id::<C>(), s)
	}
}

/// Deserialize a placeholder ciphersuite field, checking if it's the ciphersuite ID string.
#[cfg(feature = "serde")]
pub(crate) fn ciphersuite_deserialize<'de, D, C>(deserializer: D) -> Result<(), D::Error>
where
	D: serde::Deserializer<'de>,
	C: Ciphersuite,
{
	if deserializer.is_human_readable() {
		let s: alloc::string::String = serde::de::Deserialize::deserialize(deserializer)?;
		if s != C::ID { Err(serde::de::Error::custom("wrong ciphersuite")) } else { Ok(()) }
	} else {
		let buffer: [u8; 4] = serde::de::Deserialize::deserialize(deserializer)?;
		if buffer != short_id::<C>() {
			Err(serde::de::Error::custom("wrong ciphersuite"))
		} else {
			Ok(())
		}
	}
}

/// Deserialize a version. For now, since there is a single version 0,
/// simply validate if it's 0.
#[cfg(feature = "serde")]
pub(crate) fn version_deserialize<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
	D: serde::Deserializer<'de>,
{
	let version: u8 = serde::de::Deserialize::deserialize(deserializer)?;
	if version != 0 {
		Err(serde::de::Error::custom("wrong format version, only 0 supported"))
	} else {
		Ok(version)
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
	fn serialize(&self) -> Result<Vec<u8>, Error<C>>;
}

#[cfg(feature = "serialization")]
pub(crate) trait Deserialize<C: Ciphersuite> {
	/// Deserialize the struct from a slice of bytes.
	fn deserialize(bytes: &[u8]) -> Result<Self, Error<C>>
	where
		Self: core::marker::Sized;
}

#[cfg(feature = "serialization")]
impl<T: serde::Serialize, C: Ciphersuite> Serialize<C> for T {
	fn serialize(&self) -> Result<Vec<u8>, Error<C>> {
		postcard::to_allocvec(self).map_err(|_| Error::SerializationError)
	}
}

#[cfg(feature = "serialization")]
impl<T: for<'de> serde::Deserialize<'de>, C: Ciphersuite> Deserialize<C> for T {
	fn deserialize(bytes: &[u8]) -> Result<Self, Error<C>> {
		postcard::from_bytes(bytes).map_err(|_| Error::DeserializationError)
	}
}
