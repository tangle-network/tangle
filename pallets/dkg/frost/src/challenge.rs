use super::{
	traits::{Ciphersuite, Element, Field, Group},
	verifying_key::VerifyingKey,
};
use core::fmt::Debug;
use sp_std::vec;

/// A type refinement for the scalar field element representing the per-message _[challenge]_.
///
/// [challenge]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#name-signature-challenge-computa
#[derive(Clone)]
pub struct Challenge<C: Ciphersuite>(pub <<C::Group as Group>::Field as Field>::Scalar);

impl<C> Challenge<C>
where
	C: Ciphersuite,
{
	/// Creates a challenge from a scalar.
	pub fn from_scalar(
		scalar: <<<C as Ciphersuite>::Group as Group>::Field as Field>::Scalar,
	) -> Self {
		Self(scalar)
	}

	/// Return the underlying scalar.
	pub fn to_scalar(self) -> <<<C as Ciphersuite>::Group as Group>::Field as Field>::Scalar {
		self.0
	}
}

impl<C> Debug for Challenge<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		f.debug_tuple("Secret")
			.field(&hex::encode(<<C::Group as Group>::Field>::serialize(&self.0)))
			.finish()
	}
}

/// Generates the challenge as is required for Schnorr signatures.
///
/// Deals in bytes, so that [FROST] and singleton signing and verification can use it with different
/// types.
///
/// This is the only invocation of the H2 hash function from the [RFC].
///
/// [FROST]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#name-signature-challenge-computa
/// [RFC]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-3.2
pub fn challenge<C>(R: &Element<C>, verifying_key: &VerifyingKey<C>, msg: &[u8]) -> Challenge<C>
where
	C: Ciphersuite,
{
	let mut preimage = vec![];

	preimage.extend_from_slice(<C::Group>::challenge_bytes(R).as_ref());
	preimage.extend_from_slice(<C::Group>::challenge_bytes(&verifying_key.element).as_ref());
	preimage.extend_from_slice(msg);

	Challenge(C::H2(&preimage[..]))
}
