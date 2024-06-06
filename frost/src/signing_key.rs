//! Schnorr signature signing keys

use crate::{
	error::Error,
	traits::{Ciphersuite, Field, Group, Scalar},
	util::scalar_is_valid,
	verifying_key::VerifyingKey,
};

#[cfg(feature = "std")]
use crate::{challenge, signature::Signature};

#[cfg(feature = "std")]
use crate::random_nonzero;
#[cfg(feature = "std")]
use rand_core::{CryptoRng, RngCore};

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
	#[cfg(feature = "std")]
	pub fn new<R: RngCore + CryptoRng>(rng: &mut R) -> SigningKey<C> {
		let scalar = random_nonzero::<C, R>(rng);

		SigningKey { scalar }
	}

	/// Deserialize from bytes
	pub fn deserialize(
		bytes: <<C::Group as Group>::Field as Field>::Serialization,
	) -> Result<SigningKey<C>, Error> {
		let scalar =
			<<C::Group as Group>::Field as Field>::deserialize(&bytes).map_err(Error::from)?;

		if scalar == <<C::Group as Group>::Field as Field>::zero() {
			return Err(Error::MalformedSigningKey);
		}

		Ok(Self { scalar })
	}

	/// Serialize `SigningKey` to bytes
	pub fn serialize(&self) -> <<C::Group as Group>::Field as Field>::Serialization {
		<<C::Group as Group>::Field as Field>::serialize(&self.scalar)
	}

	/// Create a signature `msg` using this `SigningKey`.
	#[cfg(feature = "std")]
	pub fn sign<R: RngCore + CryptoRng>(&self, mut rng: R, msg: &[u8]) -> Signature<C> {
		let k = random_nonzero::<C, R>(&mut rng);

		let R = <C::Group>::generator() * k;

		// Generate Schnorr challenge
		let c = challenge::<C>(&R, &VerifyingKey::<C>::from(*self), msg);

		let z = k + (c.0 * self.scalar);

		Signature { R, z }
	}

	/// Creates a SigningKey from a scalar.
	pub fn from_scalar(
		scalar: <<<C as Ciphersuite>::Group as Group>::Field as Field>::Scalar,
	) -> Self {
		Self { scalar }
	}

	/// Return the underlying scalar.
	pub fn to_scalar(self) -> <<<C as Ciphersuite>::Group as Group>::Field as Field>::Scalar {
		self.scalar
	}

	/// Check if the signing key is valid.
	pub fn is_valid(&self) -> bool {
		scalar_is_valid::<C>(&self.scalar)
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
		VerifyingKey { element: C::Group::generator() * signing_key.scalar }
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
