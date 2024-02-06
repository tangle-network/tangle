//! Schnorr signatures over prime order groups (or subgroups)

use super::{
	error::Error,
	traits::{Ciphersuite, Element, Field, Group, Scalar},
	util::{element_is_valid, scalar_is_valid},
};
use alloc::format;
use debugless_unwrap::DebuglessUnwrap;
use sp_std::{vec, vec::Vec};

/// A Schnorr signature over some prime order group (or subgroup).
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Signature<C: Ciphersuite> {
	/// The commitment `R` to the signature nonce.
	pub(crate) R: Element<C>,
	/// The response `z` to the challenge computed from the commitment `R`, the verifying key, and
	/// the message.
	pub(crate) z: Scalar<C>,
}

impl<C> Signature<C>
where
	C: Ciphersuite,
	C::Group: Group,
	<C::Group as Group>::Field: Field,
{
	/// Create a new Signature.
	pub fn new(
		R: <C::Group as Group>::Element,
		z: <<C::Group as Group>::Field as Field>::Scalar,
	) -> Self {
		Self { R, z }
	}

	/// Converts bytes as [`Ciphersuite::SignatureSerialization`] into a `Signature<C>`.
	pub fn deserialize(bytes: C::SignatureSerialization) -> Result<Self, Error> {
		// To compute the expected length of the encoded point, encode the generator
		// and get its length. Note that we can't use the identity because it can be encoded
		// shorter in some cases (e.g. P-256, which uses SEC1 encoding).
		let generator = <C::Group>::generator();
		let mut R_bytes = Vec::from(<C::Group>::serialize(&generator).as_ref());

		let R_bytes_len = R_bytes.len();

		R_bytes[..]
			.copy_from_slice(bytes.as_ref().get(0..R_bytes_len).ok_or(Error::MalformedSignature)?);

		let R_serialization = &R_bytes.try_into().map_err(|_| Error::MalformedSignature)?;

		let one = <<C::Group as Group>::Field as Field>::zero();
		let mut z_bytes =
			Vec::from(<<C::Group as Group>::Field as Field>::serialize(&one).as_ref());

		let z_bytes_len = z_bytes.len();

		// We extract the exact length of bytes we expect, not just the remaining bytes with
		// `bytes[R_bytes_len..]`
		z_bytes[..].copy_from_slice(
			bytes
				.as_ref()
				.get(R_bytes_len..R_bytes_len + z_bytes_len)
				.ok_or(Error::MalformedSignature)?,
		);

		let z_serialization = &z_bytes.try_into().map_err(|_| Error::MalformedSignature)?;

		Ok(Self {
			R: <C::Group>::deserialize(R_serialization).map_err(Error::Group)?,
			z: <<C::Group as Group>::Field>::deserialize(z_serialization).map_err(Error::Field)?,
		})
	}

	/// Converts this signature to its [`Ciphersuite::SignatureSerialization`] in bytes.
	pub fn serialize(&self) -> C::SignatureSerialization {
		let mut bytes = vec![];

		bytes.extend(<C::Group>::serialize(&self.R).as_ref());
		bytes.extend(<<C::Group as Group>::Field>::serialize(&self.z).as_ref());

		bytes.try_into().debugless_unwrap()
	}

	/// Check if the signature as valid values.
	pub fn is_valid(&self) -> bool {
		element_is_valid::<C>(&self.R) && scalar_is_valid::<C>(&self.z)
	}
}

impl<C> serde::Serialize for Signature<C>
where
	C: Ciphersuite,
	C::Group: Group,
	<C::Group as Group>::Field: Field,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serdect::slice::serialize_hex_lower_or_bin(&self.serialize().as_ref(), serializer)
	}
}

impl<'de, C> serde::Deserialize<'de> for Signature<C>
where
	C: Ciphersuite,
	C::Group: Group,
	<C::Group as Group>::Field: Field,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let bytes = serdect::slice::deserialize_hex_or_bin_vec(deserializer)?;
		let array =
			bytes.try_into().map_err(|_| serde::de::Error::custom("invalid byte length"))?;
		let identifier = Signature::deserialize(array)
			.map_err(|err| serde::de::Error::custom(format!("{err}")))?;
		Ok(identifier)
	}
}

impl<C: Ciphersuite> sp_std::fmt::Debug for Signature<C> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		f.debug_struct("Signature")
			.field("R", &hex::encode(<C::Group>::serialize(&self.R).as_ref()))
			.field("z", &hex::encode(<<C::Group as Group>::Field>::serialize(&self.z).as_ref()))
			.finish()
	}
}
