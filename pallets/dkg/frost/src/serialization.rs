//! Serialization support.

use super::traits::{Ciphersuite, Field, Group};
use alloc::string::String;
use sp_std::vec;

/// Helper struct to serialize a Scalar.
pub(crate) struct ScalarSerialization<C: Ciphersuite>(
	pub <<<C as Ciphersuite>::Group as Group>::Field as Field>::Serialization,
);

impl<C> serde::Serialize for ScalarSerialization<C>
where
	C: Ciphersuite,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serdect::array::serialize_hex_lower_or_bin(&self.0.as_ref(), serializer)
	}
}

impl<'de, C> serde::Deserialize<'de> for ScalarSerialization<C>
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
		Ok(Self(array))
	}
}

pub(crate) struct ElementSerialization<C: Ciphersuite>(
	pub(crate) <<C as Ciphersuite>::Group as Group>::Serialization,
);

impl<C> serde::Serialize for ElementSerialization<C>
where
	C: Ciphersuite,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serdect::array::serialize_hex_lower_or_bin(&self.0.as_ref(), serializer)
	}
}

impl<'de, C> serde::Deserialize<'de> for ElementSerialization<C>
where
	C: Ciphersuite,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		// Get size from the size of the generator
		let generator = <C::Group>::generator();
		let len = <C::Group>::serialize(&generator).as_ref().len();

		let mut bytes = vec![0u8; len];
		serdect::array::deserialize_hex_or_bin(&mut bytes[..], deserializer)?;
		let array =
			bytes.try_into().map_err(|_| serde::de::Error::custom("invalid byte length"))?;
		Ok(Self(array))
	}
}

// The short 4-byte ID. Derived as the CRC-32 of the UTF-8
// encoded ID in big endian format.
#[allow(unused)]
const fn short_id<C>() -> [u8; 4]
where
	C: Ciphersuite,
{
	super::const_crc32::crc32(C::ID.as_bytes()).to_be_bytes()
}

/// Serialize a placeholder ciphersuite field with the ciphersuite ID string.
#[allow(unused)]
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
#[allow(unused)]
pub(crate) fn ciphersuite_deserialize<'de, D, C>(deserializer: D) -> Result<(), D::Error>
where
	D: serde::Deserializer<'de>,
	C: Ciphersuite,
{
	if deserializer.is_human_readable() {
		let s: String = serde::de::Deserialize::deserialize(deserializer)?;
		if s != C::ID {
			Err(serde::de::Error::custom("wrong ciphersuite"))
		} else {
			Ok(())
		}
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
#[allow(unused)]
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
		Self: std::marker::Sized;
}

#[cfg(feature = "serialization")]
impl<T: serde::Serialize, C: Ciphersuite> Serialize<C> for T {
	fn serialize(&self) -> Result<Vec<u8>, Error<C>> {
		postcard::to_stdvec(self).map_err(|_| Error::SerializationError)
	}
}

#[cfg(feature = "serialization")]
impl<T: for<'de> serde::Deserialize<'de>, C: Ciphersuite> Deserialize<C> for T {
	fn deserialize(bytes: &[u8]) -> Result<Self, Error<C>> {
		postcard::from_bytes(bytes).map_err(|_| Error::DeserializationError)
	}
}
