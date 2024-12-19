//! Schnorr signatures over prime order groups (or subgroups)

use alloc::{string::ToString, vec::Vec};
use derive_getters::Getters;

use crate::{Ciphersuite, Element, Error, Field, Group, Scalar};

/// A Schnorr signature over some prime order group (or subgroup).
#[derive(Copy, Clone, Eq, PartialEq, Getters)]
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
	#[cfg(feature = "internals")]
	pub fn new(
		R: <C::Group as Group>::Element,
		z: <<C::Group as Group>::Field as Field>::Scalar,
	) -> Self {
		Self { R, z }
	}

	/// Converts default-encoded bytes as
	/// [`Ciphersuite::SignatureSerialization`] into a `Signature<C>`.
	#[cfg(feature = "internals")]
	pub fn default_deserialize(bytes: &[u8]) -> Result<Self, Error<C>> {
		// To compute the expected length of the encoded point, encode the generator
		// and get its length. Note that we can't use the identity because it can be encoded
		// shorter in some cases (e.g. P-256, which uses SEC1 encoding).
		let generator = <C::Group>::generator();
		let mut R_bytes = Vec::from(<C::Group>::serialize(&generator)?.as_ref());
		let R_bytes_len = R_bytes.len();

		let one = <<C::Group as Group>::Field as Field>::zero();
		let mut z_bytes =
			Vec::from(<<C::Group as Group>::Field as Field>::serialize(&one).as_ref());
		let z_bytes_len = z_bytes.len();

		if bytes.len() != R_bytes_len + z_bytes_len {
			return Err(Error::MalformedSignature);
		}

		R_bytes[..].copy_from_slice(bytes.get(0..R_bytes_len).ok_or(Error::MalformedSignature)?);

		let R_serialization = &R_bytes.try_into().map_err(|_| Error::MalformedSignature)?;

		// We extract the exact length of bytes we expect, not just the remaining bytes with
		// `bytes[R_bytes_len..]`
		z_bytes[..].copy_from_slice(
			bytes
				.get(R_bytes_len..R_bytes_len + z_bytes_len)
				.ok_or(Error::MalformedSignature)?,
		);

		let z_serialization = &z_bytes.try_into().map_err(|_| Error::MalformedSignature)?;

		Ok(Self {
			R: <C::Group>::deserialize(R_serialization)?,
			z: <<C::Group as Group>::Field>::deserialize(z_serialization)?,
		})
	}

	/// Converts bytes as [`Ciphersuite::SignatureSerialization`] into a `Signature<C>`.
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error<C>> {
		C::deserialize_signature(bytes)
	}

	/// Converts this signature to its default byte serialization.
	#[cfg(feature = "internals")]
	pub fn default_serialize(&self) -> Result<Vec<u8>, Error<C>> {
		let mut bytes = Vec::<u8>::new();

		bytes.extend(<C::Group>::serialize(&self.R)?.as_ref());
		bytes.extend(<<C::Group as Group>::Field>::serialize(&self.z).as_ref());

		Ok(bytes)
	}

	/// Converts this signature to its byte serialization.
	pub fn serialize(&self) -> Result<Vec<u8>, Error<C>> {
		<C>::serialize_signature(self)
	}
}

#[cfg(feature = "serde")]
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
		serdect::slice::serialize_hex_lower_or_bin(
			&self.serialize().map_err(serde::ser::Error::custom)?,
			serializer,
		)
	}
}

#[cfg(feature = "serde")]
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
		let signature = Signature::deserialize(&bytes)
			.map_err(|err| serde::de::Error::custom(format!("{err}")))?;
		Ok(signature)
	}
}

impl<C: Ciphersuite> core::fmt::Debug for Signature<C> {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		f.debug_struct("Signature")
			.field(
				"R",
				&<C::Group>::serialize(&self.R)
					.map(|s| hex::encode(s.as_ref()))
					.unwrap_or("<invalid>".to_string()),
			)
			.field("z", &hex::encode(<<C::Group as Group>::Field>::serialize(&self.z).as_ref()))
			.finish()
	}
}
