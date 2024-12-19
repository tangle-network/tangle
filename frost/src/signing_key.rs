//! Schnorr signature signing keys

use alloc::vec::Vec;

use rand_core::{CryptoRng, RngCore};

use crate::{
	random_nonzero, serialization::SerializableScalar, Challenge, Ciphersuite, Error, Field, Group,
	Scalar, Signature, VerifyingKey,
};

/// A signing key for a Schnorr signature on a FROST [`Ciphersuite::Group`].
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct SigningKey<C>
where
	C: Ciphersuite,
{
	pub(crate) scalar: Scalar<C>,
}

impl<C> SigningKey<C>
where
	C: Ciphersuite,
{
	/// Generate a new signing key.
	pub fn new<R: RngCore + CryptoRng>(rng: &mut R) -> SigningKey<C> {
		let scalar = random_nonzero::<C, R>(rng);

		SigningKey { scalar }
	}

	/// Deserialize from bytes
	pub fn deserialize(bytes: &[u8]) -> Result<SigningKey<C>, Error<C>> {
		Self::from_scalar(SerializableScalar::deserialize(bytes)?.0)
	}

	/// Serialize `SigningKey` to bytes
	pub fn serialize(&self) -> Vec<u8> {
		SerializableScalar::<C>(self.scalar).serialize()
	}

	/// Create a signature `msg` using this `SigningKey`.
	pub fn sign<R: RngCore + CryptoRng>(&self, rng: R, message: &[u8]) -> Signature<C> {
		<C>::single_sign(self, rng, message)
	}

	/// Create a signature `msg` using this `SigningKey` using the default
	/// signing.
	#[cfg(feature = "internals")]
	pub fn default_sign<R: RngCore + CryptoRng>(&self, mut rng: R, message: &[u8]) -> Signature<C> {
		let public = VerifyingKey::<C>::from(*self);

		let (k, R) = <C>::generate_nonce(&mut rng);

		// Generate Schnorr challenge
		let c: Challenge<C> = <C>::challenge(&R, &public, message).expect("should not return error since that happens only if one of the inputs is the identity. R is not since k is nonzero. The verifying_key is not because signing keys are not allowed to be zero.");

		let z = k + (c.0 * self.scalar);

		Signature { R, z }
	}

	/// Creates a SigningKey from a scalar. Returns an error if the scalar is zero.
	pub fn from_scalar(
		scalar: <<<C as Ciphersuite>::Group as Group>::Field as Field>::Scalar,
	) -> Result<Self, Error<C>> {
		if scalar == <<C::Group as Group>::Field as Field>::zero() {
			return Err(Error::MalformedSigningKey);
		}
		Ok(Self { scalar })
	}

	/// Return the underlying scalar.
	pub fn to_scalar(self) -> <<<C as Ciphersuite>::Group as Group>::Field as Field>::Scalar {
		self.scalar
	}
}

impl<C> core::fmt::Debug for SigningKey<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_tuple("SigningKey").field(&"<redacted>").finish()
	}
}

impl<C> From<&SigningKey<C>> for VerifyingKey<C>
where
	C: Ciphersuite,
{
	fn from(signing_key: &SigningKey<C>) -> Self {
		VerifyingKey::new(C::Group::generator() * signing_key.scalar)
	}
}

impl<C> From<SigningKey<C>> for VerifyingKey<C>
where
	C: Ciphersuite,
{
	fn from(signing_key: SigningKey<C>) -> Self {
		VerifyingKey::<C>::from(&signing_key)
	}
}
