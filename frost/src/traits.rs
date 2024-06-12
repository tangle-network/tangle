//! Traits used to abstract Ciphersuites.

use core::{
	fmt::Debug,
	ops::{Add, Mul, Sub},
};

use parity_scale_codec::{Decode, Encode};
use sp_std::vec::Vec;

#[cfg(feature = "std")]
use rand_core::{CryptoRng, RngCore};

use subtle::ConditionallyNegatable;

use super::{
	error::{Error, FieldError, GroupError},
	signature::Signature,
	verifying_key::VerifyingKey,
};

/// A prime order finite field GF(q) over which all scalar values for our prime order group can be
/// multiplied are defined.
///
/// This trait does not have to be implemented for a finite field scalar itself, it can be a
/// pass-through, implemented for a type just for the ciphersuite, and calls through to another
/// implementation underneath, so that this trait does not have to be implemented for types you
/// don't own.
pub trait Field: Copy + Clone {
	/// An element of the scalar field GF(p).
	/// The Eq/PartialEq implementation MUST be constant-time.
	type Scalar: Add<Output = Self::Scalar>
		+ ConditionallyNegatable
		+ Copy
		+ Clone
		+ Eq
		+ Mul<Output = Self::Scalar>
		+ PartialEq
		+ Sub<Output = Self::Scalar>
		+ Encode
		+ Decode;

	/// A unique byte array buf of fixed length N.
	type Serialization: AsRef<[u8]> + Debug + TryFrom<Vec<u8>>;

	/// Returns the zero element of the field, the additive identity.
	fn zero() -> Self::Scalar;

	/// Returns the one element of the field, the multiplicative identity.
	fn one() -> Self::Scalar;

	/// Computes the multiplicative inverse of an element of the scalar field, failing if the
	/// element is zero.
	fn invert(scalar: &Self::Scalar) -> Result<Self::Scalar, FieldError>;

	/// Generate a random scalar from the entire space [0, l-1]
	///
	/// <https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-3.1-3.3>
	#[cfg(feature = "std")]
	fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self::Scalar;

	/// A member function of a [`Field`] that maps a [`Scalar`] to a unique byte array buf of
	/// fixed length Ne.
	///
	/// <https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-3.1-3.8>
	fn serialize(scalar: &Self::Scalar) -> Self::Serialization;

	/// A member function of a [`Field`] that maps a [`Scalar`] to a unique byte array buf of
	/// fixed length Ne, in little-endian order.
	///
	/// This is used internally.
	fn little_endian_serialize(scalar: &Self::Scalar) -> Self::Serialization;

	/// A member function of a [`Field`] that attempts to map a byte array `buf` to a [`Scalar`].
	///
	/// Fails if the input is not a valid byte representation of an [`Scalar`] of the
	/// [`Field`]. This function can raise an [`Error`] if deserialization fails.
	///
	/// <https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-3.1-3.9>
	fn deserialize(buf: &Self::Serialization) -> Result<Self::Scalar, FieldError>;
}

/// An element of the [`Ciphersuite`] `C`'s [`Group`]'s scalar [`Field`].
pub type Scalar<C> = <<<C as Ciphersuite>::Group as Group>::Field as Field>::Scalar;

/// A prime-order group (or subgroup) that provides everything we need to create and verify Schnorr
/// signatures.
///
/// This trait does not have to be implemented for the curve/element/point itself, it can be a
/// pass-through, implemented for a type just for the ciphersuite, and calls through to another
/// implementation underneath, so that this trait does not have to be implemented for types you
/// don't own.
pub trait Group: Copy + Clone + PartialEq {
	/// A prime order finite field GF(q) over which all scalar values for our prime order group can
	/// be multiplied are defined.
	type Field: Field;

	/// An element of our group that we will be computing over.
	type Element: Add<Output = Self::Element>
		+ ConditionallyNegatable
		+ Copy
		+ Clone
		+ Eq
		+ Mul<<Self::Field as Field>::Scalar, Output = Self::Element>
		+ PartialEq
		+ Sub<Output = Self::Element>
		+ Encode
		+ Decode;

	/// A unique byte array buf of fixed length N.
	///
	/// Little-endian!
	type Serialization: AsRef<[u8]> + Debug + TryFrom<Vec<u8>>;

	/// The order of the the quotient group when the prime order subgroup divides the order of the
	/// full curve group.
	///
	/// If using a prime order elliptic curve, the cofactor should be 1 in the scalar field.
	fn cofactor() -> <Self::Field as Field>::Scalar;

	/// Additive [identity] of the prime order group.
	///
	/// [identity]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-3.1-3.2
	fn identity() -> Self::Element;

	/// The fixed generator element of the prime order group.
	///
	/// The 'base' of ['ScalarBaseMult()'] from the spec.
	///
	/// [`ScalarBaseMult()`]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-3.1-3.5
	fn generator() -> Self::Element;

	/// A member function of a group _G_ that maps an [`Element`] to a unique byte array buf of
	/// fixed length Ne.
	///
	/// <https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-3.1-3.6>
	fn serialize(element: &Self::Element) -> Self::Serialization;

	/// A member function of a [`Group`] that attempts to map a byte array `buf` to an [`Element`].
	///
	/// Fails if the input is not a valid byte representation of an [`Element`] of the
	/// [`Group`]. This function can raise an [`Error`] if deserialization fails or if the
	/// resulting [`Element`] is the identity element of the group
	///
	/// <https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-3.1-3.7>
	fn deserialize(buf: &Self::Serialization) -> Result<Self::Element, GroupError>;

	/// The challenge bytes for a FROST ciphersuite. These may or may not match
	/// the output from serialization like in the case of Taproot
	fn challenge_bytes(element: &Self::Element) -> Vec<u8> {
		Self::serialize(element).as_ref().to_vec()
	}

	/// Determine if the elements y is odd or not. For now only applies
	/// for bitcoin taproot
	fn y_is_odd(_element: &Self::Element) -> subtle::Choice {
		subtle::Choice::from(0u8)
	}
}

/// An element of the [`Ciphersuite`] `C`'s [`Group`].
pub type Element<C> = <<C as Ciphersuite>::Group as Group>::Element;

/// A [FROST ciphersuite] specifies the underlying prime-order group details and cryptographic hash
/// function.
///
/// [FROST ciphersuite]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#name-ciphersuites
pub trait Ciphersuite: Copy + Clone + PartialEq + Debug + Default {
	/// The ciphersuite ID string. It should be equal to the contextString in
	/// the spec. For new ciphersuites, this should be a string that identifies
	/// the ciphersuite; it's recommended to use a similar format to the
	/// ciphersuites in the FROST spec, e.g. "FROST-RISTRETTO255-SHA512-v1".
	const ID: &'static str;

	/// The prime order group (or subgroup) that this ciphersuite operates over.
	type Group: Group;

	/// A unique byte array of fixed length.
	type HashOutput: AsRef<[u8]>;

	/// A unique byte array of fixed length that is the `Group::ElementSerialization` +
	/// `Group::ScalarSerialization`
	type SignatureSerialization: AsRef<[u8]> + TryFrom<Vec<u8>>;

	/// [H1] for a FROST ciphersuite.
	///
	/// Maps arbitrary inputs to `Self::Scalar` elements of the prime-order group scalar field.
	///
	/// [H1]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#name-cryptographic-hash-function
	fn H1(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar;

	/// [H2] for a FROST ciphersuite.
	///
	/// Maps arbitrary inputs to `Self::Scalar` elements of the prime-order group scalar field.
	///
	/// [H2]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#name-cryptographic-hash-function
	fn H2(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar;

	/// [H3] for a FROST ciphersuite.
	///
	/// Maps arbitrary inputs to `Self::Scalar` elements of the prime-order group scalar field.
	///
	/// [H3]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#name-cryptographic-hash-function
	fn H3(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar;

	/// [H4] for a FROST ciphersuite.
	///
	/// Usually an an alias for the ciphersuite hash function _H_ with domain separation applied.
	///
	/// [H4]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#name-cryptographic-hash-function
	fn H4(m: &[u8]) -> Self::HashOutput;

	/// [H5] for a FROST ciphersuite.
	///
	/// Usually an an alias for the ciphersuite hash function _H_ with domain separation applied.
	///
	/// [H5]: https://github.com/cfrg/draft-irtf-cfrg-frost/blob/master/draft-irtf-cfrg-frost.md#cryptographic-hash
	fn H5(m: &[u8]) -> Self::HashOutput;

	/// Hash function for a FROST ciphersuite, used for the DKG.
	///
	/// The DKG it not part of the specification, thus this is optional.
	/// It can return None if DKG is not supported by the Ciphersuite. This is
	/// the default implementation.
	///
	/// Maps arbitrary inputs to non-zero `Self::Scalar` elements of the prime-order group scalar
	/// field.
	fn HDKG(_m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
		None
	}

	/// Hash function for a FROST ciphersuite, used for deriving identifiers from strings.
	///
	/// This feature is not part of the specification and is just a convenient
	/// way of creating identifiers. Therefore it can return None if this is not supported by the
	/// Ciphersuite. This is the default implementation.
	///
	/// Maps arbitrary inputs to non-zero `Self::Scalar` elements of the prime-order group scalar
	/// field.
	fn HID(_m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
		None
	}

	/// Verify a signature for this ciphersuite. The default implementation uses the "cofactored"
	/// equation (it multiplies by the cofactor returned by [`Group::cofactor()`]).
	///
	/// # Cryptographic Safety
	///
	/// You may override this to provide a tailored implementation, but if the ciphersuite defines
	/// it, it must also multiply by the cofactor to comply with the RFC. Note that batch
	/// verification (see [`crate::batch::Verifier`]) also uses the default implementation
	/// regardless whether a tailored implementation was provided.
	fn verify_signature(
		msg: &[u8],
		signature: &Signature<Self>,
		public_key: &VerifyingKey<Self>,
	) -> Result<(), Error> {
		let c = super::challenge::<Self>(&signature.R, public_key, msg);

		public_key.verify_prehashed(c, signature)
	}
}
