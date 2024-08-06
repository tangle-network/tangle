//! Schnorr signatures over prime order groups (or subgroups)

use core::fmt::Debug;

use crate::{
	challenge::Challenge, identifier::Identifier, keys::VerifyingShare,
	round1::GroupCommitmentShare, serialization::ScalarSerialization, Header,
};

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
	pub R: Element<C>,
	/// The response `z` to the challenge computed from the commitment `R`, the verifying key, and
	/// the message.
	pub z: Scalar<C>,
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

// Used to help encoding a SignatureShare. Since it has a Scalar<C> it can't
// be directly encoded with serde, so we use this struct to wrap the scalar.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
#[serde(try_from = "ScalarSerialization<C>")]
#[serde(into = "ScalarSerialization<C>")]
struct SignatureShareHelper<C: Ciphersuite>(Scalar<C>);

impl<C> TryFrom<ScalarSerialization<C>> for SignatureShareHelper<C>
where
	C: Ciphersuite,
{
	type Error = Error;

	fn try_from(value: ScalarSerialization<C>) -> Result<Self, Self::Error> {
		<<C::Group as Group>::Field>::deserialize(&value.0)
			.map(|scalar| Self(scalar))
			.map_err(|e| e.into())
	}
}

impl<C> From<SignatureShareHelper<C>> for ScalarSerialization<C>
where
	C: Ciphersuite,
{
	fn from(value: SignatureShareHelper<C>) -> Self {
		Self(<<C::Group as Group>::Field>::serialize(&value.0))
	}
}

/// A participant's signature share, which the coordinator will aggregate with all other signer's
/// shares into the joint signature.
#[derive(Clone, Copy, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
#[serde(deny_unknown_fields)]
#[serde(try_from = "SignatureShareSerialization<C>")]
#[serde(into = "SignatureShareSerialization<C>")]
pub struct SignatureShare<C: Ciphersuite> {
	/// This participant's signature over the message.
	pub share: Scalar<C>,
}

impl<C> SignatureShare<C>
where
	C: Ciphersuite,
{
	/// Deserialize [`SignatureShare`] from bytes
	pub fn deserialize(
		bytes: <<C::Group as Group>::Field as Field>::Serialization,
	) -> Result<Self, Error> {
		<<C::Group as Group>::Field>::deserialize(&bytes)
			.map(|scalar| Self { share: scalar })
			.map_err(|e| e.into())
	}

	/// Serialize [`SignatureShare`] to bytes
	pub fn serialize(&self) -> <<C::Group as Group>::Field as Field>::Serialization {
		<<C::Group as Group>::Field>::serialize(&self.share)
	}

	/// Tests if a signature share issued by a participant is valid before
	/// aggregating it into a final joint signature to publish.
	///
	/// This is the final step of [`verify_signature_share`] from the spec.
	///
	/// [`verify_signature_share`]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#name-signature-share-verificatio
	pub fn verify(
		&self,
		_identifier: Identifier<C>,
		group_commitment_share: &GroupCommitmentShare<C>,
		verifying_share: &VerifyingShare<C>,
		lambda_i: Scalar<C>,
		challenge: &Challenge<C>,
	) -> Result<(), Error> {
		if (<C::Group>::generator() * self.share)
			!= (group_commitment_share.0 + (verifying_share.0 * challenge.0 * lambda_i))
		{
			return Err(Error::InvalidSignatureShare);
		}

		Ok(())
	}

	/// Tests if the signature share is valid
	pub fn is_valid(&self) -> bool {
		scalar_is_valid::<C>(&self.share)
	}
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
#[serde(deny_unknown_fields)]
struct SignatureShareSerialization<C: Ciphersuite> {
	/// Serialization header
	pub(crate) header: Header<C>,
	share: SignatureShareHelper<C>,
}

impl<C> From<SignatureShareSerialization<C>> for SignatureShare<C>
where
	C: Ciphersuite,
{
	fn from(value: SignatureShareSerialization<C>) -> Self {
		Self { share: value.share.0 }
	}
}

impl<C> From<SignatureShare<C>> for SignatureShareSerialization<C>
where
	C: Ciphersuite,
{
	fn from(value: SignatureShare<C>) -> Self {
		Self { header: Header::default(), share: SignatureShareHelper(value.share) }
	}
}

impl<C> Debug for SignatureShare<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		f.debug_struct("SignatureShare")
			.field("share", &hex::encode(self.serialize()))
			.finish()
	}
}
