use core::fmt::Debug;

use super::{
	challenge::Challenge,
	error::Error,
	serialization::ElementSerialization,
	signature::Signature,
	traits::{Ciphersuite, Element, Group},
	util::element_is_valid,
};

use hex::FromHex;
use parity_scale_codec::{Decode, Encode};
use sp_std::vec::Vec;

/// A valid verifying key for Schnorr signatures over a FROST [`Ciphersuite::Group`].
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
#[serde(try_from = "ElementSerialization<C>")]
#[serde(into = "ElementSerialization<C>")]
pub struct VerifyingKey<C>
where
	C: Ciphersuite,
{
	pub element: Element<C>,
}

impl<C> VerifyingKey<C>
where
	C: Ciphersuite,
{
	/// Create a new VerifyingKey from the given element.
	#[allow(unused)]
	pub fn new(element: <C::Group as Group>::Element) -> Self {
		Self { element }
	}

	/// Return the underlying element.
	pub fn to_element(self) -> <C::Group as Group>::Element {
		self.element
	}

	/// Deserialize from bytes
	pub fn deserialize(
		bytes: <C::Group as Group>::Serialization,
	) -> Result<VerifyingKey<C>, Error> {
		<C::Group>::deserialize(&bytes)
			.map(|element| VerifyingKey { element })
			.map_err(Error::Group)
	}

	/// Serialize `VerifyingKey` to bytes
	pub fn serialize(&self) -> <C::Group as Group>::Serialization {
		<C::Group>::serialize(&self.element)
	}

	/// Verify a purported `signature` with a pre-hashed [`Challenge`] made by this verification
	/// key.
	pub fn verify_prehashed(
		&self,
		challenge: Challenge<C>,
		signature: &Signature<C>,
	) -> Result<(), Error> {
		// Verify check is h * ( - z * B + R  + c * A) == 0
		//                 h * ( z * B - c * A - R) == 0
		//
		// where h is the cofactor
		let zB = C::Group::generator() * signature.z;
		let cA = self.element * challenge.0;
		let check = (zB - cA - signature.R) * C::Group::cofactor();
		if check == C::Group::identity() {
			Ok(())
		} else {
			Err(Error::MalformedSignature)
		}
	}

	/// Verify a purported `signature` over `msg` made by this verification key.
	pub fn verify(&self, msg: &[u8], signature: &Signature<C>) -> Result<(), Error> {
		C::verify_signature(msg, signature, self)
	}

	/// Check if the verifying key is valid.
	pub fn is_valid(&self) -> bool {
		element_is_valid::<C>(&self.element)
	}
}

impl<C> Debug for VerifyingKey<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		f.debug_tuple("VerifyingKey").field(&hex::encode(self.serialize())).finish()
	}
}

impl<C> FromHex for VerifyingKey<C>
where
	C: Ciphersuite,
{
	type Error = &'static str;

	fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
		let v: Vec<u8> = FromHex::from_hex(hex).map_err(|_| "invalid hex")?;
		match v.try_into() {
			Ok(bytes) => Self::deserialize(bytes).map_err(|_| "malformed verifying key encoding"),
			Err(_) => Err("malformed verifying key encoding"),
		}
	}
}

impl<C> TryFrom<ElementSerialization<C>> for VerifyingKey<C>
where
	C: Ciphersuite,
{
	type Error = Error;

	fn try_from(value: ElementSerialization<C>) -> Result<Self, Self::Error> {
		Self::deserialize(value.0)
	}
}

impl<C> From<VerifyingKey<C>> for ElementSerialization<C>
where
	C: Ciphersuite,
{
	fn from(value: VerifyingKey<C>) -> Self {
		Self(value.serialize())
	}
}
