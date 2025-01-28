#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]

use sp_std::{borrow::Cow, collections::btree_map::BTreeMap, vec, vec::Vec};

use k256::{
	elliptic_curve::{
		bigint::U256,
		group::prime::PrimeCurveAffine,
		hash2curve::{hash_to_field, ExpandMsgXmd},
		ops::Reduce,
		point::AffineCoordinates,
		sec1::{FromEncodedPoint, ToEncodedPoint},
		Field as FFField, PrimeField,
	},
	AffinePoint, ProjectivePoint, Scalar,
};
use rand_core::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};

use tg_frost_core as frost_core;

use frost_core::{self as frost, random_nonzero};

use keys::{EvenY, Tweak};

// Re-exports in our public API
pub use frost_core::{
	Challenge, Ciphersuite, Element, Field, FieldError, Group, GroupCommitment, GroupError,
};
pub use rand_core;

/// An error.
pub type Error = frost_core::Error<Secp256K1Sha256TR>;

/// An implementation of the FROST(secp256k1, SHA-256) ciphersuite scalar field.
#[derive(Clone, Copy)]
pub struct Secp256K1ScalarField;

impl Field for Secp256K1ScalarField {
	type Scalar = Scalar;

	type Serialization = [u8; 32];

	fn zero() -> Self::Scalar {
		Scalar::ZERO
	}

	fn one() -> Self::Scalar {
		Scalar::ONE
	}

	fn invert(scalar: &Self::Scalar) -> Result<Self::Scalar, FieldError> {
		// [`Scalar`]'s Eq/PartialEq does a constant-time comparison
		if *scalar == <Self as Field>::zero() {
			Err(FieldError::InvalidZeroScalar)
		} else {
			Ok(scalar.invert().unwrap())
		}
	}

	fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self::Scalar {
		Scalar::random(rng)
	}

	fn serialize(scalar: &Self::Scalar) -> Self::Serialization {
		scalar.to_bytes().into()
	}

	fn deserialize(buf: &Self::Serialization) -> Result<Self::Scalar, FieldError> {
		let field_bytes: &k256::FieldBytes = buf.into();
		match Scalar::from_repr(*field_bytes).into() {
			Some(s) => Ok(s),
			None => Err(FieldError::MalformedScalar),
		}
	}

	fn little_endian_serialize(scalar: &Self::Scalar) -> Self::Serialization {
		let mut array = Self::serialize(scalar);
		array.reverse();
		array
	}
}

/// An implementation of the FROST(secp256k1, SHA-256) ciphersuite group.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Secp256K1Group;

impl Group for Secp256K1Group {
	type Field = Secp256K1ScalarField;

	type Element = ProjectivePoint;

	/// [SEC 1][1] serialization of a compressed point in secp256k1 takes 33 bytes
	/// (1-byte prefix and 32 bytes for the coordinate).
	///
	/// Note that, in the SEC 1 spec, the identity is encoded as a single null byte;
	/// but here we pad with zeroes. This is acceptable as the identity _should_ never
	/// be serialized in FROST, else we error.
	///
	/// [1]: https://secg.org/sec1-v2.pdf
	type Serialization = [u8; 33];

	fn cofactor() -> <Self::Field as Field>::Scalar {
		Scalar::ONE
	}

	fn identity() -> Self::Element {
		ProjectivePoint::IDENTITY
	}

	fn generator() -> Self::Element {
		ProjectivePoint::GENERATOR
	}

	fn serialize(element: &Self::Element) -> Result<Self::Serialization, GroupError> {
		if *element == Self::identity() {
			return Err(GroupError::InvalidIdentityElement);
		}
		let mut fixed_serialized = [0; 33];
		let serialized_point = element.to_affine().to_encoded_point(true);
		let serialized = serialized_point.as_bytes();
		fixed_serialized.copy_from_slice(serialized);
		Ok(fixed_serialized)
	}

	fn deserialize(buf: &Self::Serialization) -> Result<Self::Element, GroupError> {
		let encoded_point =
			k256::EncodedPoint::from_bytes(buf).map_err(|_| GroupError::MalformedElement)?;

		match Option::<AffinePoint>::from(AffinePoint::from_encoded_point(&encoded_point)) {
			Some(point) => {
				if point.is_identity().into() {
					// This is actually impossible since the identity is encoded a a single byte
					// which will never happen since we receive a 33-byte buffer.
					// We leave the check for consistency.
					Err(GroupError::InvalidIdentityElement)
				} else {
					Ok(ProjectivePoint::from(point))
				}
			},
			None => Err(GroupError::MalformedElement),
		}
	}
}

fn hash_to_array(inputs: &[&[u8]]) -> [u8; 32] {
	let mut h = Sha256::new();
	for i in inputs {
		h.update(i);
	}
	let mut output = [0u8; 32];
	output.copy_from_slice(h.finalize().as_slice());
	output
}

fn hash_to_scalar(domain: &[&[u8]], msg: &[u8]) -> Scalar {
	let mut u = [Secp256K1ScalarField::zero()];
	hash_to_field::<ExpandMsgXmd<Sha256>, Scalar>(&[msg], domain, &mut u)
		.expect("should never return error according to error cases described in ExpandMsgXmd");
	u[0]
}

/// Context string from the ciphersuite in the [spec].
///
/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.5-1
const CONTEXT_STRING: &str = "FROST-secp256k1-SHA256-TR-v1";

/// An implementation of the FROST(secp256k1, SHA-256) ciphersuite.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Secp256K1Sha256TR;

/// Digest the hasher to a Scalar
fn hasher_to_scalar(hasher: Sha256) -> Scalar {
	// This is acceptable because secp256k1 curve order is close to 2^256,
	// and the input is uniformly random since it is a hash output, therefore
	// the bias is negligibly small.
	Scalar::reduce(U256::from_be_slice(&hasher.finalize()))
}

/// Create a BIP340 compliant tagged hash
fn tagged_hash(tag: &str) -> Sha256 {
	let mut hasher = Sha256::new();
	let mut tag_hasher = Sha256::new();
	tag_hasher.update(tag.as_bytes());
	let tag_hash = tag_hasher.finalize();
	hasher.update(tag_hash);
	hasher.update(tag_hash);
	hasher
}

/// Create a BIP341 compliant taproot tweak
fn tweak<T: AsRef<[u8]>>(
	public_key: &<<Secp256K1Sha256TR as Ciphersuite>::Group as Group>::Element,
	merkle_root: Option<T>,
) -> Scalar {
	match merkle_root {
		None => Secp256K1ScalarField::zero(),
		Some(root) => {
			let mut hasher = tagged_hash("TapTweak");
			hasher.update(public_key.to_affine().x());
			hasher.update(root.as_ref());
			hasher_to_scalar(hasher)
		},
	}
}

// Negate a Nonce
fn negate_nonce(nonce: &frost_core::round1::Nonce<S>) -> frost_core::round1::Nonce<S> {
	frost_core::round1::Nonce::<S>::from_scalar(-nonce.to_scalar())
}

// Negate a SigningNonces
fn negate_nonces(signing_nonces: &round1::SigningNonces) -> round1::SigningNonces {
	// Create an `internals` SigningNonces::from_nonces_and_commitments or
	// something similar.
	round1::SigningNonces::from_nonces(
		negate_nonce(signing_nonces.hiding()),
		negate_nonce(signing_nonces.binding()),
	)
}

impl Ciphersuite for Secp256K1Sha256TR {
	const ID: &'static str = CONTEXT_STRING;

	type Group = Secp256K1Group;

	type HashOutput = [u8; 32];

	type SignatureSerialization = [u8; 64];

	/// H1 for FROST(secp256k1, SHA-256)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.5-2.2.2.1
	fn H1(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"rho"], m)
	}

	/// H2 for FROST(secp256k1, SHA-256)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.5-2.2.2.2
	fn H2(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		let mut hasher = tagged_hash("BIP0340/challenge");
		hasher.update(m);
		hasher_to_scalar(hasher)
	}

	/// H3 for FROST(secp256k1, SHA-256)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.5-2.2.2.3
	fn H3(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
		hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"nonce"], m)
	}

	/// H4 for FROST(secp256k1, SHA-256)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.5-2.2.2.4
	fn H4(m: &[u8]) -> Self::HashOutput {
		hash_to_array(&[CONTEXT_STRING.as_bytes(), b"msg", m])
	}

	/// H5 for FROST(secp256k1, SHA-256)
	///
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.5-2.2.2.5
	fn H5(m: &[u8]) -> Self::HashOutput {
		hash_to_array(&[CONTEXT_STRING.as_bytes(), b"com", m])
	}

	/// HDKG for FROST(secp256k1, SHA-256)
	fn HDKG(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
		Some(hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"dkg"], m))
	}

	/// HID for FROST(secp256k1, SHA-256)
	fn HID(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
		Some(hash_to_scalar(&[CONTEXT_STRING.as_bytes(), b"id"], m))
	}

	// Sign, negating the key if required by BIP-340.
	fn single_sign<R: RngCore + CryptoRng>(
		signing_key: &SigningKey,
		rng: R,
		message: &[u8],
	) -> Signature {
		let signing_key = signing_key.into_even_y(None);
		signing_key.default_sign(rng, message)
	}

	// Preprocess sign inputs, negating the keys in the KeyPackage if required
	// by BIP-340.
	fn pre_sign<'a>(
		signing_package: &'a SigningPackage,
		signer_nonces: &'a round1::SigningNonces,
		key_package: &'a keys::KeyPackage,
	) -> Result<
		(Cow<'a, SigningPackage>, Cow<'a, round1::SigningNonces>, Cow<'a, keys::KeyPackage>),
		Error,
	> {
		Ok((
			Cow::Borrowed(signing_package),
			Cow::Borrowed(signer_nonces),
			Cow::Owned(key_package.clone().into_even_y(None)),
		))
	}

	// Preprocess sign inputs, negating the keys in the PublicKeyPackage if
	// required by BIP-340.
	fn pre_aggregate<'a>(
		signing_package: &'a SigningPackage,
		signature_shares: &'a BTreeMap<Identifier, round2::SignatureShare>,
		public_key_package: &'a keys::PublicKeyPackage,
	) -> Result<
		(
			Cow<'a, SigningPackage>,
			Cow<'a, BTreeMap<Identifier, round2::SignatureShare>>,
			Cow<'a, keys::PublicKeyPackage>,
		),
		Error,
	> {
		Ok((
			Cow::Borrowed(signing_package),
			Cow::Borrowed(signature_shares),
			Cow::Owned(public_key_package.clone().into_even_y(None)),
		))
	}

	// Preprocess verify inputs, negating the VerifyingKey and `signature.R` if required by
	// BIP-340.
	fn pre_verify<'a>(
		message: &'a [u8],
		signature: &'a Signature,
		public_key: &'a VerifyingKey,
	) -> Result<(Cow<'a, [u8]>, Cow<'a, Signature>, Cow<'a, VerifyingKey>), Error> {
		let public_key = public_key.into_even_y(None);
		let signature = signature.into_even_y(None);
		Ok((Cow::Borrowed(message), Cow::Owned(signature), Cow::Owned(public_key)))
	}

	// Generate a nonce, negating it if required by BIP-340.
	fn generate_nonce<R: RngCore + CryptoRng>(
		rng: &mut R,
	) -> (<<Self::Group as Group>::Field as Field>::Scalar, <Self::Group as Group>::Element) {
		let k = random_nonzero::<Self, R>(rng);
		let R = <Self::Group>::generator() * k;
		if R.to_affine().y_is_odd().into() {
			(-k, -R)
		} else {
			(k, R)
		}
	}

	// Compute the challenge. Per BIP-340, only the X coordinate of R and
	// verifying_key are hashed, unlike vanilla FROST.
	fn challenge(
		R: &Element<S>,
		verifying_key: &VerifyingKey,
		message: &[u8],
	) -> Result<Challenge<S>, Error> {
		let mut preimage = vec![];
		preimage.extend_from_slice(&R.to_affine().x());
		preimage.extend_from_slice(&verifying_key.to_element().to_affine().x());
		preimage.extend_from_slice(message);
		Ok(Challenge::from_scalar(S::H2(&preimage[..])))
	}

	/// Compute a signature share, negating the nonces if required by BIP-340.
	fn compute_signature_share(
		group_commitment: &GroupCommitment<S>,
		signer_nonces: &round1::SigningNonces,
		binding_factor: frost::BindingFactor<S>,
		lambda_i: <<Self::Group as Group>::Field as Field>::Scalar,
		key_package: &frost::keys::KeyPackage<S>,
		challenge: Challenge<S>,
	) -> round2::SignatureShare {
		let signer_nonces = if !group_commitment.has_even_y() {
			negate_nonces(signer_nonces)
		} else {
			signer_nonces.clone()
		};

		frost::round2::compute_signature_share(
			&signer_nonces,
			binding_factor,
			lambda_i,
			key_package,
			challenge,
		)
	}

	/// Verify a signature share, negating the group commitment share if
	/// required by BIP-340.
	fn verify_share(
		group_commitment: &GroupCommitment<S>,
		signature_share: &frost_core::round2::SignatureShare<S>,
		identifier: Identifier,
		group_commitment_share: &frost_core::round1::GroupCommitmentShare<S>,
		verifying_share: &frost_core::keys::VerifyingShare<S>,
		lambda_i: Scalar,
		challenge: &Challenge<S>,
	) -> Result<(), Error> {
		let group_commitment_share = if !group_commitment.has_even_y() {
			frost_core::round1::GroupCommitmentShare::from_element(
				-group_commitment_share.to_element(),
			)
		} else {
			*group_commitment_share
		};
		signature_share.verify(
			identifier,
			&group_commitment_share,
			verifying_share,
			lambda_i,
			challenge,
		)
	}

	/// Serialize a signature in compact BIP340 format, with an x-only R point.
	fn serialize_signature(signature: &Signature) -> Result<Vec<u8>, Error> {
		let R_bytes = Self::Group::serialize(signature.R())?;
		let z_bytes = <Self::Group as Group>::Field::serialize(signature.z());

		let mut bytes = vec![0u8; 64];
		bytes[..32].copy_from_slice(&R_bytes[1..]);
		bytes[32..].copy_from_slice(&z_bytes);
		Ok(bytes)
	}

	/// Deserialize a signature in compact BIP340 format, with an x-only R point.
	fn deserialize_signature(bytes: &[u8]) -> Result<Signature, Error> {
		if bytes.len() != 64 {
			return Err(Error::MalformedSignature);
		}

		let mut R_bytes = [0u8; 33];
		R_bytes[0] = 0x02; // taproot signatures always have an even R point
		R_bytes[1..].copy_from_slice(&bytes[..32]);

		let mut z_bytes = [0u8; 32];
		z_bytes.copy_from_slice(&bytes[32..]);

		let R = Self::Group::deserialize(&R_bytes)?;
		let z = <Self::Group as Group>::Field::deserialize(&z_bytes)?;

		Ok(Signature::new(R, z))
	}

	/// Post-process the DKG output. We add an unusable taproot tweak to the
	/// group key computed by a DKG run, to prevent peers from inserting rogue
	/// tapscript tweaks into the group's joint public key.
	fn post_dkg(
		key_package: keys::KeyPackage,
		public_key_package: keys::PublicKeyPackage,
	) -> Result<(keys::KeyPackage, keys::PublicKeyPackage), Error> {
		// From BIP-341:
		// > If the spending conditions do not require a script path, the output
		// > key should commit to an unspendable script path instead of having
		// > no script path. This can be achieved by computing the output key
		// > point as Q = P + int(hashTapTweak(bytes(P)))G.
		let merkle_root = [0u8; 0];
		Ok((key_package.tweak(Some(&merkle_root)), public_key_package.tweak(Some(&merkle_root))))
	}
}

type S = Secp256K1Sha256TR;

/// A FROST(secp256k1, SHA-256) participant identifier.
pub type Identifier = frost::Identifier<S>;

/// FROST(secp256k1, SHA-256) keys, key generation, key shares.
pub mod keys {
	use super::*;

	/// The identifier list to use when generating key shares.
	pub type IdentifierList<'a> = frost::keys::IdentifierList<'a, S>;

	/// Allows all participants' keys to be generated using a central, trusted
	/// dealer.
	pub fn generate_with_dealer<RNG: RngCore + CryptoRng>(
		max_signers: u16,
		min_signers: u16,
		identifiers: IdentifierList,
		mut rng: RNG,
	) -> Result<(BTreeMap<Identifier, SecretShare>, PublicKeyPackage), Error> {
		frost::keys::generate_with_dealer(max_signers, min_signers, identifiers, &mut rng)
	}

	/// Splits an existing key into FROST shares.
	///
	/// This is identical to [`generate_with_dealer`] but receives an existing key
	/// instead of generating a fresh one. This is useful in scenarios where
	/// the key needs to be generated externally or must be derived from e.g. a
	/// seed phrase.
	pub fn split<R: RngCore + CryptoRng>(
		secret: &SigningKey,
		max_signers: u16,
		min_signers: u16,
		identifiers: IdentifierList,
		rng: &mut R,
	) -> Result<(BTreeMap<Identifier, SecretShare>, PublicKeyPackage), Error> {
		frost::keys::split(secret, max_signers, min_signers, identifiers, rng)
	}

	/// Recompute the secret from t-of-n secret shares using Lagrange interpolation.
	///
	/// This can be used if for some reason the original key must be restored; e.g.
	/// if threshold signing is not required anymore.
	///
	/// This is NOT required to sign with FROST; the whole point of FROST is being
	/// able to generate signatures only using the shares, without having to
	/// reconstruct the original key.
	///
	/// The caller is responsible for providing at least `min_signers` shares;
	/// if less than that is provided, a different key will be returned.
	pub fn reconstruct(secret_shares: &[KeyPackage]) -> Result<SigningKey, Error> {
		frost::keys::reconstruct(secret_shares)
	}

	/// Secret and public key material generated by a dealer performing
	/// [`generate_with_dealer`].
	///
	/// # Security
	///
	/// To derive a FROST(secp256k1, SHA-256) keypair, the receiver of the [`SecretShare`] *must*
	/// call .into(), which under the hood also performs validation.
	pub type SecretShare = frost::keys::SecretShare<S>;

	/// A secret scalar value representing a signer's share of the group secret.
	pub type SigningShare = frost::keys::SigningShare<S>;

	/// A public group element that represents a single signer's public verification share.
	pub type VerifyingShare = frost::keys::VerifyingShare<S>;

	/// A FROST(secp256k1, SHA-256) keypair, which can be generated either by a trusted dealer or
	/// using a DKG.
	///
	/// When using a central dealer, [`SecretShare`]s are distributed to
	/// participants, who then perform verification, before deriving
	/// [`KeyPackage`]s, which they store to later use during signing.
	pub type KeyPackage = frost::keys::KeyPackage<S>;

	/// Public data that contains all the signers' public keys as well as the
	/// group public key.
	///
	/// Used for verification purposes before publishing a signature.
	pub type PublicKeyPackage = frost::keys::PublicKeyPackage<S>;

	/// Contains the commitments to the coefficients for our secret polynomial _f_,
	/// used to generate participants' key shares.
	///
	/// [`VerifiableSecretSharingCommitment`] contains a set of commitments to the coefficients
	/// (which themselves are scalars) for a secret polynomial f, where f is used to
	/// generate each ith participant's key share f(i). Participants use this set of
	/// commitments to perform verifiable secret sharing.
	///
	/// Note that participants MUST be assured that they have the *same*
	/// [`VerifiableSecretSharingCommitment`], either by performing pairwise comparison, or by using
	/// some agreed-upon public location for publication, where each participant can
	/// ensure that they received the correct (and same) value.
	pub type VerifiableSecretSharingCommitment = frost::keys::VerifiableSecretSharingCommitment<S>;

	/// Trait for ensuring the group public key has an even Y coordinate.
	///
	/// In BIP-320, public keys are encoded with only the X coordinate, which
	/// means that two Y coordinates are possible. The specification says that
	/// the coordinate which is even must be used. Alternatively, something
	/// equivalent can be accomplished by simply converting any existing
	/// (non-encoded) public key to have an even Y coordinate.
	///
	/// This trait is used to enable this procedure, by changing the private and
	/// public keys to ensure that the public key has a even Y coordinate. This
	/// is done by simply negating both keys if Y is even (in a field, negating
	/// is equivalent to computing p - x where p is the prime modulus. Since p
	/// is odd, if x is odd then the result will be even). Fortunately this
	/// works even after Shamir secret sharing, in the individual signing and
	/// verifying shares, since it's linear.
	pub trait EvenY {
		/// Return if the given type has a group public key with an even Y
		/// coordinate.
		fn has_even_y(&self) -> bool;

		/// Convert the given type to make sure the group public key has an even
		/// Y coordinate. `is_even` can be specified if evenness was already
		/// determined beforehand.
		fn into_even_y(self, is_even: Option<bool>) -> Self;
	}

	impl EvenY for PublicKeyPackage {
		fn has_even_y(&self) -> bool {
			let verifying_key = self.verifying_key();
			(!verifying_key.to_element().to_affine().y_is_odd()).into()
		}

		fn into_even_y(self, is_even: Option<bool>) -> Self {
			let is_even = is_even.unwrap_or_else(|| self.has_even_y());
			if !is_even {
				// Negate verifying key
				let verifying_key = VerifyingKey::new(-self.verifying_key().to_element());
				// Recreate verifying share map with negated VerifyingShares
				// values.
				let verifying_shares: BTreeMap<_, _> = self
					.verifying_shares()
					.iter()
					.map(|(i, vs)| {
						let vs = VerifyingShare::new(-vs.to_element());
						(*i, vs)
					})
					.collect();
				PublicKeyPackage::new(verifying_shares, verifying_key)
			} else {
				self
			}
		}
	}

	impl EvenY for KeyPackage {
		fn has_even_y(&self) -> bool {
			let verifying_key = self.verifying_key();
			(!verifying_key.to_element().to_affine().y_is_odd()).into()
		}

		fn into_even_y(self, is_even: Option<bool>) -> Self {
			let is_even = is_even.unwrap_or_else(|| self.has_even_y());
			if !is_even {
				// Negate all components
				let verifying_key = VerifyingKey::new(-self.verifying_key().to_element());
				let signing_share = SigningShare::new(-self.signing_share().to_scalar());
				let verifying_share = VerifyingShare::new(-self.verifying_share().to_element());
				KeyPackage::new(
					*self.identifier(),
					signing_share,
					verifying_share,
					verifying_key,
					*self.min_signers(),
				)
			} else {
				self
			}
		}
	}

	impl EvenY for VerifyingKey {
		fn has_even_y(&self) -> bool {
			(!self.to_element().to_affine().y_is_odd()).into()
		}

		fn into_even_y(self, is_even: Option<bool>) -> Self {
			let is_even = is_even.unwrap_or_else(|| self.has_even_y());
			if !is_even {
				VerifyingKey::new(-self.to_element())
			} else {
				self
			}
		}
	}

	impl EvenY for GroupCommitment<S> {
		fn has_even_y(&self) -> bool {
			(!self.clone().to_element().to_affine().y_is_odd()).into()
		}

		fn into_even_y(self, is_even: Option<bool>) -> Self {
			let is_even = is_even.unwrap_or_else(|| self.has_even_y());
			if !is_even {
				Self::from_element(-self.to_element())
			} else {
				self
			}
		}
	}

	impl EvenY for Signature {
		fn has_even_y(&self) -> bool {
			(!self.R().to_affine().y_is_odd()).into()
		}

		fn into_even_y(self, is_even: Option<bool>) -> Self {
			let is_even = is_even.unwrap_or_else(|| self.has_even_y());
			if !is_even {
				Self::new(-*self.R(), *self.z())
			} else {
				self
			}
		}
	}

	impl EvenY for SigningKey {
		fn has_even_y(&self) -> bool {
			(!Into::<VerifyingKey>::into(self).to_element().to_affine().y_is_odd()).into()
		}

		fn into_even_y(self, is_even: Option<bool>) -> Self {
			let is_even = is_even.unwrap_or_else(|| self.has_even_y());
			if !is_even {
				SigningKey::from_scalar(-self.to_scalar())
					.expect("the original SigningKey must be nonzero")
			} else {
				self
			}
		}
	}

	/// Trait for tweaking a key component following BIP-341
	pub trait Tweak: EvenY {
		/// Convert the given type to add a tweak.
		fn tweak<T: AsRef<[u8]>>(self, merkle_root: Option<T>) -> Self;
	}

	impl Tweak for PublicKeyPackage {
		fn tweak<T: AsRef<[u8]>>(self, merkle_root: Option<T>) -> Self {
			let t = tweak(&self.verifying_key().to_element(), merkle_root);
			let tp = ProjectivePoint::GENERATOR * t;
			let public_key_package = self.into_even_y(None);
			let verifying_key =
				VerifyingKey::new(public_key_package.verifying_key().to_element() + tp);
			// Recreate verifying share map with negated VerifyingShares
			// values.
			let verifying_shares: BTreeMap<_, _> = public_key_package
				.verifying_shares()
				.iter()
				.map(|(i, vs)| {
					let vs = VerifyingShare::new(vs.to_element() + tp);
					(*i, vs)
				})
				.collect();
			PublicKeyPackage::new(verifying_shares, verifying_key)
		}
	}

	impl Tweak for KeyPackage {
		fn tweak<T: AsRef<[u8]>>(self, merkle_root: Option<T>) -> Self {
			let t = tweak(&self.verifying_key().to_element(), merkle_root);
			let tp = ProjectivePoint::GENERATOR * t;
			let key_package = self.into_even_y(None);
			let verifying_key = VerifyingKey::new(key_package.verifying_key().to_element() + tp);
			let signing_share = SigningShare::new(key_package.signing_share().to_scalar() + t);
			let verifying_share =
				VerifyingShare::new(key_package.verifying_share().to_element() + tp);
			KeyPackage::new(
				*key_package.identifier(),
				signing_share,
				verifying_share,
				verifying_key,
				*key_package.min_signers(),
			)
		}
	}
}

/// FROST(secp256k1, SHA-256) Round 1 functionality and types.
pub mod round1 {
	use crate::keys::SigningShare;

	use super::*;

	/// Comprised of FROST(secp256k1, SHA-256) hiding and binding nonces.
	///
	/// Note that [`SigningNonces`] must be used *only once* for a signing
	/// operation; re-using nonces will result in leakage of a signer's long-lived
	/// signing key.
	pub type SigningNonces = frost::round1::SigningNonces<S>;

	/// Published by each participant in the first round of the signing protocol.
	///
	/// This step can be batched if desired by the implementation. Each
	/// SigningCommitment can be used for exactly *one* signature.
	pub type SigningCommitments = frost::round1::SigningCommitments<S>;

	/// A commitment to a signing nonce share.
	pub type NonceCommitment = frost::round1::NonceCommitment<S>;

	/// Performed once by each participant selected for the signing operation.
	///
	/// Generates the signing nonces and commitments to be used in the signing
	/// operation.
	pub fn commit<RNG>(secret: &SigningShare, rng: &mut RNG) -> (SigningNonces, SigningCommitments)
	where
		RNG: CryptoRng + RngCore,
	{
		frost::round1::commit::<S, RNG>(secret, rng)
	}
}

/// Generated by the coordinator of the signing operation and distributed to
/// each signing party.
pub type SigningPackage = frost::SigningPackage<S>;

/// FROST(secp256k1, SHA-256) Round 2 functionality and types, for signature share generation.
pub mod round2 {
	use keys::Tweak;

	use super::*;

	/// A FROST(secp256k1, SHA-256) participant's signature share, which the Coordinator will
	/// aggregate with all other signer's shares into the joint signature.
	pub type SignatureShare = frost::round2::SignatureShare<S>;

	/// Performed once by each participant selected for the signing operation.
	///
	/// Receives the message to be signed and a set of signing commitments and a set
	/// of randomizing commitments to be used in that signing operation, including
	/// that for this participant.
	///
	/// Assumes the participant has already determined which nonce corresponds with
	/// the commitment that was assigned by the coordinator in the SigningPackage.
	pub fn sign(
		signing_package: &SigningPackage,
		signer_nonces: &round1::SigningNonces,
		key_package: &keys::KeyPackage,
	) -> Result<SignatureShare, Error> {
		frost::round2::sign(signing_package, signer_nonces, key_package)
	}

	/// Same as [`sign()`], but using a Taproot tweak as specified in BIP-341.
	pub fn sign_with_tweak(
		signing_package: &SigningPackage,
		signer_nonces: &round1::SigningNonces,
		key_package: &keys::KeyPackage,
		merkle_root: Option<&[u8]>,
	) -> Result<SignatureShare, Error> {
		if merkle_root.is_some() {
			let key_package = key_package.clone().tweak(merkle_root);
			frost::round2::sign(signing_package, signer_nonces, &key_package)
		} else {
			frost::round2::sign(signing_package, signer_nonces, key_package)
		}
	}
}

/// A Schnorr signature on FROST(secp256k1, SHA-256).
pub type Signature = frost_core::Signature<S>;

/// Verifies each FROST(secp256k1, SHA-256) participant's signature share, and if all are valid,
/// aggregates the shares into a signature to publish.
///
/// Resulting signature is compatible with verification of a plain Schnorr
/// signature.
///
/// This operation is performed by a coordinator that can communicate with all
/// the signing participants before publishing the final signature. The
/// coordinator can be one of the participants or a semi-trusted third party
/// (who is trusted to not perform denial of service attacks, but does not learn
/// any secret information). Note that because the coordinator is trusted to
/// report misbehaving parties in order to avoid publishing an invalid
/// signature, if the coordinator themselves is a signer and misbehaves, they
/// can avoid that step. However, at worst, this results in a denial of
/// service attack due to publishing an invalid signature.
pub fn aggregate(
	signing_package: &SigningPackage,
	signature_shares: &BTreeMap<Identifier, round2::SignatureShare>,
	public_key_package: &keys::PublicKeyPackage,
) -> Result<Signature, Error> {
	frost::aggregate(signing_package, signature_shares, public_key_package)
}

/// Same as [`aggregate()`], but using a Taproot tweak as specified in BIP-341.
pub fn aggregate_with_tweak(
	signing_package: &SigningPackage,
	signature_shares: &BTreeMap<Identifier, round2::SignatureShare>,
	public_key_package: &keys::PublicKeyPackage,
	merkle_root: Option<&[u8]>,
) -> Result<Signature, Error> {
	if merkle_root.is_some() {
		let public_key_package = public_key_package.clone().tweak(merkle_root);
		frost::aggregate(signing_package, signature_shares, &public_key_package)
	} else {
		frost::aggregate(signing_package, signature_shares, public_key_package)
	}
}

/// A signing key for a Schnorr signature on FROST(secp256k1, SHA-256).
pub type SigningKey = frost_core::SigningKey<S>;

/// A valid verifying key for Schnorr signatures on FROST(secp256k1, SHA-256).
pub type VerifyingKey = frost_core::VerifyingKey<S>;
