use super::traits::{Ciphersuite, Field, Group};
use core::fmt::Debug;

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
