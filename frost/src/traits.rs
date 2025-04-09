//! Traits used to abstract Ciphersuites.

use core::{
	fmt::Debug,
	ops::{Add, Mul, Sub},
};

use alloc::{borrow::Cow, collections::BTreeMap, vec::Vec};
use rand_core::{CryptoRng, RngCore};

use crate::{
	BindingFactor, Challenge, Error, FieldError, GroupCommitment, GroupError, Identifier,
	Signature, SigningKey, SigningPackage, VerifyingKey, challenge,
	keys::{KeyPackage, PublicKeyPackage, VerifyingShare},
	random_nonzero,
	round1::{self},
	round2::{self, SignatureShare},
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
		+ Copy
		+ Clone
		+ Eq
		+ Mul<Output = Self::Scalar>
		+ PartialEq
		+ Sub<Output = Self::Scalar>;

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
	/// <https://datatracker.ietf.org/doc/html/rfc9591#section-3.1-4.6>
	fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self::Scalar;

	/// A member function of a [`Field`] that maps a [`Scalar`] to a unique byte array buf of
	/// fixed length Ne.
	///
	/// <https://datatracker.ietf.org/doc/html/rfc9591#section-3.1-4.16>
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
	/// <https://datatracker.ietf.org/doc/html/rfc9591#section-3.1-4.18>
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
		+ Copy
		+ Clone
		+ Eq
		+ Mul<<Self::Field as Field>::Scalar, Output = Self::Element>
		+ PartialEq
		+ Sub<Output = Self::Element>;

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
	/// [identity]: https://datatracker.ietf.org/doc/html/rfc9591#section-3.1-4.4
	fn identity() -> Self::Element;

	/// The fixed generator element of the prime order group.
	///
	/// The 'base' of ['ScalarBaseMult()'] from the spec.
	///
	/// [`ScalarBaseMult()`]: https://datatracker.ietf.org/doc/html/rfc9591#section-3.1-4.10
	fn generator() -> Self::Element;

	/// A member function of a group _G_ that maps an [`Element`] to a unique
	/// byte array buf of fixed length Ne. This function raises an error if the
	/// element is the identity element of the group.
	///
	/// <https://datatracker.ietf.org/doc/html/rfc9591#section-3.1-4.12>
	fn serialize(element: &Self::Element) -> Result<Self::Serialization, GroupError>;

	/// A member function of a [`Group`] that attempts to map a byte array `buf` to an [`Element`].
	///
	/// Fails if the input is not a valid byte representation of an [`Element`] of the
	/// [`Group`]. This function can raise an [`Error`] if deserialization fails or if the
	/// resulting [`Element`] is the identity element of the group
	///
	/// <https://datatracker.ietf.org/doc/html/rfc9591#section-3.1-4.14>
	fn deserialize(buf: &Self::Serialization) -> Result<Self::Element, GroupError>;
}

/// An element of the [`Ciphersuite`] `C`'s [`Group`].
pub type Element<C> = <<C as Ciphersuite>::Group as Group>::Element;

/// A [FROST ciphersuite] specifies the underlying prime-order group details and cryptographic hash
/// function.
///
/// [FROST ciphersuite]: https://datatracker.ietf.org/doc/html/rfc9591#name-ciphersuites
// See https://github.com/ZcashFoundation/frost/issues/693 for reasoning about the 'static bound.
pub trait Ciphersuite: Copy + Clone + PartialEq + Debug + 'static {
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
	/// [H1]: https://datatracker.ietf.org/doc/html/rfc9591#name-cryptographic-hash-function
	fn H1(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar;

	/// [H2] for a FROST ciphersuite.
	///
	/// Maps arbitrary inputs to `Self::Scalar` elements of the prime-order group scalar field.
	///
	/// [H2]: https://datatracker.ietf.org/doc/html/rfc9591#name-cryptographic-hash-function
	fn H2(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar;

	/// [H3] for a FROST ciphersuite.
	///
	/// Maps arbitrary inputs to `Self::Scalar` elements of the prime-order group scalar field.
	///
	/// [H3]: https://datatracker.ietf.org/doc/html/rfc9591#name-cryptographic-hash-function
	fn H3(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar;

	/// [H4] for a FROST ciphersuite.
	///
	/// Usually an an alias for the ciphersuite hash function _H_ with domain separation applied.
	///
	/// [H4]: https://datatracker.ietf.org/doc/html/rfc9591#name-cryptographic-hash-function
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

	// The following are optional methods that allow customizing steps of the
	// protocol if required.

	/// Optional. Do regular (non-FROST) signing with a [`SigningKey`]. Called
	/// by [`SigningKey::sign()`]. This is not used by FROST. Can be overriden
	/// if required which is useful if FROST signing has been changed by the
	/// other Ciphersuite trait methods and regular signing should be changed
	/// accordingly to match.
	fn single_sign<R: RngCore + CryptoRng>(
		signing_key: &SigningKey<Self>,
		rng: R,
		message: &[u8],
	) -> Signature<Self> {
		signing_key.sign(rng, message)
	}

	/// Optional. Verify a signature for this ciphersuite. Called by
	/// [`VerifyingKey::verify()`]. The default implementation uses the
	/// "cofactored" equation (it multiplies by the cofactor returned by
	/// [`Group::cofactor()`]).
	///
	/// # Cryptographic Safety
	///
	/// You may override this to provide a tailored implementation, but if the
	/// ciphersuite defines it, it must also multiply by the cofactor to comply
	/// with the RFC. Note that batch verification (see
	/// [`crate::batch::Verifier`]) also uses the default implementation
	/// regardless whether a tailored implementation was provided.
	fn verify_signature(
		message: &[u8],
		signature: &Signature<Self>,
		public_key: &VerifyingKey<Self>,
	) -> Result<(), Error<Self>> {
		let (message, signature, public_key) = <Self>::pre_verify(message, signature, public_key)?;

		let c = <Self>::challenge(&signature.R, &public_key, &message)?;

		public_key.verify_prehashed(c, &signature)
	}

	/// Optional. Pre-process [`round2::sign()`] inputs. The default
	/// implementation returns them as-is. [`Cow`] is used so implementations
	/// can choose to return the same passed reference or a modified clone.
	#[allow(clippy::type_complexity)]
	fn pre_sign<'a>(
		signing_package: &'a SigningPackage<Self>,
		signer_nonces: &'a round1::SigningNonces<Self>,
		key_package: &'a KeyPackage<Self>,
	) -> Result<
		(
			Cow<'a, SigningPackage<Self>>,
			Cow<'a, round1::SigningNonces<Self>>,
			Cow<'a, KeyPackage<Self>>,
		),
		Error<Self>,
	> {
		Ok((
			Cow::Borrowed(signing_package),
			Cow::Borrowed(signer_nonces),
			Cow::Borrowed(key_package),
		))
	}

	/// Optional. Pre-process [`crate::aggregate()`] and
	/// [`crate::verify_signature_share()`] inputs. In the latter case, "dummy"
	/// container BTreeMap and PublicKeyPackage are passed with the relevant
	/// values. The default implementation returns them as-is. [`Cow`] is used
	/// so implementations can choose to return the same passed reference or a
	/// modified clone.
	#[allow(clippy::type_complexity)]
	fn pre_aggregate<'a>(
		signing_package: &'a SigningPackage<Self>,
		signature_shares: &'a BTreeMap<Identifier<Self>, round2::SignatureShare<Self>>,
		public_key_package: &'a PublicKeyPackage<Self>,
	) -> Result<
		(
			Cow<'a, SigningPackage<Self>>,
			Cow<'a, BTreeMap<Identifier<Self>, round2::SignatureShare<Self>>>,
			Cow<'a, PublicKeyPackage<Self>>,
		),
		Error<Self>,
	> {
		Ok((
			Cow::Borrowed(signing_package),
			Cow::Borrowed(signature_shares),
			Cow::Borrowed(public_key_package),
		))
	}

	/// Optional. Pre-process [`VerifyingKey::verify()`] inputs. The default
	/// implementation returns them as-is. [`Cow`] is used so implementations
	/// can choose to return the same passed reference or a modified clone.
	#[allow(clippy::type_complexity)]
	fn pre_verify<'a>(
		msg: &'a [u8],
		signature: &'a Signature<Self>,
		public_key: &'a VerifyingKey<Self>,
	) -> Result<(Cow<'a, [u8]>, Cow<'a, Signature<Self>>, Cow<'a, VerifyingKey<Self>>), Error<Self>>
	{
		Ok((Cow::Borrowed(msg), Cow::Borrowed(signature), Cow::Borrowed(public_key)))
	}

	/// Optional. Generate a nonce and a commitment to it. Used by
	/// [`SigningKey`] for regular (non-FROST) signing and internally by the DKG
	/// to generate proof-of-knowledge signatures.
	fn generate_nonce<R: RngCore + CryptoRng>(
		rng: &mut R,
	) -> (<<Self::Group as Group>::Field as Field>::Scalar, <Self::Group as Group>::Element) {
		let k = random_nonzero::<Self, R>(rng);
		let R = <Self::Group>::generator() * k;
		(k, R)
	}

	/// Optional. Generates the challenge as is required for Schnorr signatures.
	/// Called by [`round2::sign()`] and [`crate::aggregate()`].
	fn challenge(
		R: &Element<Self>,
		verifying_key: &VerifyingKey<Self>,
		message: &[u8],
	) -> Result<Challenge<Self>, Error<Self>> {
		challenge(R, verifying_key, message)
	}

	/// Optional. Compute the signature share for a particular signer on a given
	/// challenge. Called by [`round2::sign()`].
	fn compute_signature_share(
		_group_commitment: &GroupCommitment<Self>,
		signer_nonces: &round1::SigningNonces<Self>,
		binding_factor: BindingFactor<Self>,
		lambda_i: <<Self::Group as Group>::Field as Field>::Scalar,
		key_package: &KeyPackage<Self>,
		challenge: Challenge<Self>,
	) -> round2::SignatureShare<Self> {
		round2::compute_signature_share(
			signer_nonces,
			binding_factor,
			lambda_i,
			key_package,
			challenge,
		)
	}

	/// Optional. Verify a signing share. Called by [`crate::aggregate()`] if
	/// cheater detection is enabled.
	fn verify_share(
		_group_commitment: &GroupCommitment<Self>,
		signature_share: &SignatureShare<Self>,
		identifier: Identifier<Self>,
		group_commitment_share: &round1::GroupCommitmentShare<Self>,
		verifying_share: &VerifyingShare<Self>,
		lambda_i: Scalar<Self>,
		challenge: &Challenge<Self>,
	) -> Result<(), Error<Self>> {
		signature_share.verify(
			identifier,
			group_commitment_share,
			verifying_share,
			lambda_i,
			challenge,
		)
	}

	/// Optional. Converts a signature to its
	/// [`Ciphersuite::SignatureSerialization`] in bytes.
	///
	/// The default implementation serializes a signature by serializing its `R`
	/// point and `z` component independently, and then concatenating them.
	fn serialize_signature(signature: &Signature<Self>) -> Result<Vec<u8>, Error<Self>> {
		signature.serialize()
	}

	/// Optional. Converts bytes as [`Ciphersuite::SignatureSerialization`] into
	/// a `Signature<C>`.
	///
	/// The default implementation assumes the serialization is a serialized `R`
	/// point followed by a serialized `z` component with no padding or extra
	/// fields.
	fn deserialize_signature(bytes: &[u8]) -> Result<Signature<Self>, Error<Self>> {
		Signature::<Self>::deserialize(bytes)
	}

	/// Post-process the output of the DKG for a given participant.
	fn post_dkg(
		key_package: KeyPackage<Self>,
		public_key_package: PublicKeyPackage<Self>,
	) -> Result<(KeyPackage<Self>, PublicKeyPackage<Self>), Error<Self>> {
		Ok((key_package, public_key_package))
	}
}
