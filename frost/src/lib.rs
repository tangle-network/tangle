#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]
#[macro_use]
extern crate alloc;

use core::marker::PhantomData;

use alloc::{
	collections::{BTreeMap, BTreeSet},
	fmt::{self, Debug},
	vec::Vec,
};

use derive_getters::Getters;
#[cfg(any(test, feature = "test-impl"))]
use hex::FromHex;
use keys::PublicKeyPackage;
use rand_core::{CryptoRng, RngCore};
use serialization::SerializableScalar;
use zeroize::Zeroize;

pub mod batch;
#[cfg(any(test, feature = "test-impl"))]
pub mod benches;
mod error;
mod identifier;
pub mod keys;
pub mod round1;
pub mod round2;
mod scalar_mul;
// We'd like to make this conditionally pub but the attribute below does
// not work yet (https://github.com/rust-lang/rust/issues/54727)
// #[cfg_attr(feature = "internals", visibility::make(pub))]
pub mod serialization;
mod signature;
mod signing_key;
#[cfg(any(test, feature = "test-impl"))]
pub mod tests;
mod traits;
mod verifying_key;

pub use error::{Error, FieldError, GroupError};
pub use identifier::Identifier;
use scalar_mul::VartimeMultiscalarMul;
// Re-export serde
#[cfg(feature = "serde")]
pub use serde;
pub use signature::Signature;
pub use signing_key::SigningKey;
pub use traits::{Ciphersuite, Element, Field, Group, Scalar};
pub use verifying_key::VerifyingKey;

/// A type refinement for the scalar field element representing the per-message _[challenge]_.
///
/// [challenge]: https://datatracker.ietf.org/doc/html/rfc9591#name-signature-challenge-computa
#[derive(Copy, Clone)]
pub struct Challenge<C: Ciphersuite>(pub(crate) <<C::Group as Group>::Field as Field>::Scalar);

impl<C> Challenge<C>
where
	C: Ciphersuite,
{
	/// Creates a challenge from a scalar.
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
	#[allow(dead_code)]
	pub(crate) fn from_scalar(
		scalar: <<<C as Ciphersuite>::Group as Group>::Field as Field>::Scalar,
	) -> Self {
		Self(scalar)
	}

	/// Return the underlying scalar.
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
	pub(crate) fn to_scalar(
		self,
	) -> <<<C as Ciphersuite>::Group as Group>::Field as Field>::Scalar {
		self.0
	}
}

impl<C> Debug for Challenge<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
/// [FROST]: https://datatracker.ietf.org/doc/html/rfc9591#name-signature-challenge-computa
/// [RFC]: https://datatracker.ietf.org/doc/html/rfc9591#name-cryptographic-hash-function
#[cfg_attr(feature = "internals", visibility::make(pub))]
#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
fn challenge<C>(
	R: &Element<C>,
	verifying_key: &VerifyingKey<C>,
	msg: &[u8],
) -> Result<Challenge<C>, Error<C>>
where
	C: Ciphersuite,
{
	let mut preimage = Vec::new();

	preimage.extend_from_slice(<C::Group>::serialize(R)?.as_ref());
	preimage.extend_from_slice(<C::Group>::serialize(&verifying_key.to_element())?.as_ref());
	preimage.extend_from_slice(msg);

	Ok(Challenge(C::H2(&preimage[..])))
}

/// Generates a random nonzero scalar.
///
/// It assumes that the Scalar Eq/PartialEq implementation is constant-time.
#[cfg_attr(feature = "internals", visibility::make(pub))]
#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
pub(crate) fn random_nonzero<C: Ciphersuite, R: RngCore + CryptoRng>(rng: &mut R) -> Scalar<C> {
	loop {
		let scalar = <<C::Group as Group>::Field>::random(rng);

		if scalar != <<C::Group as Group>::Field>::zero() {
			return scalar;
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Zeroize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
struct Header<C: Ciphersuite> {
	/// Format version
	#[cfg_attr(
		feature = "serde",
		serde(deserialize_with = "crate::serialization::version_deserialize::<_>")
	)]
	version: u8,
	/// Ciphersuite ID
	#[cfg_attr(
		feature = "serde",
		serde(serialize_with = "crate::serialization::ciphersuite_serialize::<_, C>")
	)]
	#[cfg_attr(
		feature = "serde",
		serde(deserialize_with = "crate::serialization::ciphersuite_deserialize::<_, C>")
	)]
	ciphersuite: (),
	#[cfg_attr(feature = "serde", serde(skip))]
	phantom: PhantomData<C>,
}

impl<C> Default for Header<C>
where
	C: Ciphersuite,
{
	fn default() -> Self {
		Self {
			version: Default::default(),
			ciphersuite: Default::default(),
			phantom: Default::default(),
		}
	}
}

/// The binding factor, also known as _rho_ (ρ)
///
/// Ensures each signature share is strongly bound to a signing set, specific set
/// of commitments, and a specific message.
///
/// <https://github.com/cfrg/draft-irtf-cfrg-frost/blob/master/draft-irtf-cfrg-frost.md>
#[derive(Clone, PartialEq, Eq)]
pub struct BindingFactor<C: Ciphersuite>(Scalar<C>);

impl<C> BindingFactor<C>
where
	C: Ciphersuite,
{
	/// Serializes [`BindingFactor`] to bytes.
	pub fn serialize(&self) -> Vec<u8> {
		SerializableScalar::<C>(self.0).serialize()
	}
}

impl<C> Debug for BindingFactor<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_tuple("BindingFactor").field(&hex::encode(self.serialize())).finish()
	}
}

/// A list of binding factors and their associated identifiers.
#[derive(Clone)]
#[cfg_attr(feature = "internals", visibility::make(pub))]
#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
pub(crate) struct BindingFactorList<C: Ciphersuite>(BTreeMap<Identifier<C>, BindingFactor<C>>);

impl<C> BindingFactorList<C>
where
	C: Ciphersuite,
{
	/// Create a new [`BindingFactorList`] from a map of identifiers to binding factors.
	#[cfg(feature = "internals")]
	pub fn new(binding_factors: BTreeMap<Identifier<C>, BindingFactor<C>>) -> Self {
		Self(binding_factors)
	}

	/// Get the [`BindingFactor`] for the given identifier, or None if not found.
	pub fn get(&self, key: &Identifier<C>) -> Option<&BindingFactor<C>> {
		self.0.get(key)
	}
}

/// [`compute_binding_factors`] in the spec
///
/// [`compute_binding_factors`]: https://datatracker.ietf.org/doc/html/rfc9591#name-binding-factors-computation
#[cfg_attr(feature = "internals", visibility::make(pub))]
#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
pub(crate) fn compute_binding_factor_list<C>(
	signing_package: &SigningPackage<C>,
	verifying_key: &VerifyingKey<C>,
	additional_prefix: &[u8],
) -> Result<BindingFactorList<C>, Error<C>>
where
	C: Ciphersuite,
{
	let preimages = signing_package.binding_factor_preimages(verifying_key, additional_prefix)?;

	Ok(BindingFactorList(
		preimages
			.iter()
			.map(|(identifier, preimage)| {
				let binding_factor = C::H1(preimage);
				(*identifier, BindingFactor(binding_factor))
			})
			.collect(),
	))
}

#[cfg(any(test, feature = "test-impl"))]
impl<C> FromHex for BindingFactor<C>
where
	C: Ciphersuite,
{
	type Error = &'static str;

	fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
		let v: Vec<u8> = FromHex::from_hex(hex).map_err(|_| "invalid hex")?;

		match v.try_into() {
			Ok(bytes) => <<C::Group as Group>::Field>::deserialize(&bytes)
				.map(|scalar| Self(scalar))
				.map_err(|_| "malformed scalar encoding"),
			Err(_) => Err("malformed scalar encoding"),
		}
	}
}

/// Generates a lagrange coefficient.
///
/// The Lagrange polynomial for a set of points (x_j, y_j) for 0 <= j <= k
/// is ∑_{i=0}^k y_i.ℓ_i(x), where ℓ_i(x) is the Lagrange basis polynomial:
///
/// ℓ_i(x) = ∏_{0≤j≤k; j≠i} (x - x_j) / (x_i - x_j).
///
/// This computes ℓ_j(x) for the set of points `xs` and for the j corresponding
/// to the given xj.
///
/// If `x` is None, it uses 0 for it (since Identifiers can't be 0)
#[cfg_attr(feature = "internals", visibility::make(pub))]
#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
fn compute_lagrange_coefficient<C: Ciphersuite>(
	x_set: &BTreeSet<Identifier<C>>,
	x: Option<Identifier<C>>,
	x_i: Identifier<C>,
) -> Result<Scalar<C>, Error<C>> {
	if x_set.is_empty() {
		return Err(Error::IncorrectNumberOfIdentifiers);
	}
	let mut num = <<C::Group as Group>::Field>::one();
	let mut den = <<C::Group as Group>::Field>::one();

	let mut x_i_found = false;

	for x_j in x_set.iter() {
		if x_i == *x_j {
			x_i_found = true;
			continue;
		}

		if let Some(x) = x {
			num = num * (x.to_scalar() - x_j.to_scalar());
			den = den * (x_i.to_scalar() - x_j.to_scalar());
		} else {
			// Both signs inverted just to avoid requiring Neg (-*xj)
			num = num * x_j.to_scalar();
			den = den * (x_j.to_scalar() - x_i.to_scalar());
		}
	}
	if !x_i_found {
		return Err(Error::UnknownIdentifier);
	}

	Ok(num * <<C::Group as Group>::Field>::invert(&den).map_err(|_| Error::DuplicatedIdentifier)?)
}

/// Generates the lagrange coefficient for the i'th participant (for `signer_id`).
///
/// Implements [`derive_interpolating_value()`] from the spec.
///
/// [`derive_interpolating_value()`]: https://datatracker.ietf.org/doc/html/rfc9591#name-polynomials
#[cfg_attr(feature = "internals", visibility::make(pub))]
#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
fn derive_interpolating_value<C: Ciphersuite>(
	signer_id: &Identifier<C>,
	signing_package: &SigningPackage<C>,
) -> Result<Scalar<C>, Error<C>> {
	compute_lagrange_coefficient(
		&signing_package.signing_commitments().keys().cloned().collect(),
		None,
		*signer_id,
	)
}

/// Generated by the coordinator of the signing operation and distributed to
/// each signing party
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "C: Ciphersuite"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct SigningPackage<C: Ciphersuite> {
	/// Serialization header
	#[getter(skip)]
	pub(crate) header: Header<C>,
	/// The set of commitments participants published in the first round of the
	/// protocol.
	signing_commitments: BTreeMap<Identifier<C>, round1::SigningCommitments<C>>,
	/// Message which each participant will sign.
	///
	/// Each signer should perform protocol-specific verification on the
	/// message.
	#[cfg_attr(
		feature = "serde",
		serde(
			serialize_with = "serdect::slice::serialize_hex_lower_or_bin",
			deserialize_with = "serdect::slice::deserialize_hex_or_bin_vec"
		)
	)]
	message: Vec<u8>,
}

impl<C> SigningPackage<C>
where
	C: Ciphersuite,
{
	/// Create a new `SigningPackage`
	///
	/// The `signing_commitments` are sorted by participant `identifier`.
	pub fn new(
		signing_commitments: BTreeMap<Identifier<C>, round1::SigningCommitments<C>>,
		message: &[u8],
	) -> SigningPackage<C> {
		SigningPackage { header: Header::default(), signing_commitments, message: message.to_vec() }
	}

	/// Get a signing commitment by its participant identifier, or None if not found.
	pub fn signing_commitment(
		&self,
		identifier: &Identifier<C>,
	) -> Option<round1::SigningCommitments<C>> {
		self.signing_commitments.get(identifier).copied()
	}

	/// Compute the preimages to H1 to compute the per-signer binding factors
	// We separate this out into its own method so it can be tested
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
	#[allow(clippy::type_complexity)]
	pub fn binding_factor_preimages(
		&self,
		verifying_key: &VerifyingKey<C>,
		additional_prefix: &[u8],
	) -> Result<Vec<(Identifier<C>, Vec<u8>)>, Error<C>> {
		let mut binding_factor_input_prefix = Vec::new();

		// The length of a serialized verifying key of the same cipersuite does
		// not change between runs of the protocol, so we don't need to hash to
		// get a fixed length.
		binding_factor_input_prefix.extend_from_slice(verifying_key.serialize()?.as_ref());

		// The message is hashed with H4 to force the variable-length message
		// into a fixed-length byte string, same for hashing the variable-sized
		// (between runs of the protocol) set of group commitments, but with H5.
		binding_factor_input_prefix.extend_from_slice(C::H4(self.message.as_slice()).as_ref());
		binding_factor_input_prefix.extend_from_slice(
			C::H5(&round1::encode_group_commitments(self.signing_commitments())?[..]).as_ref(),
		);
		binding_factor_input_prefix.extend_from_slice(additional_prefix);

		Ok(self
			.signing_commitments()
			.keys()
			.map(|identifier| {
				let mut binding_factor_input = Vec::new();

				binding_factor_input.extend_from_slice(&binding_factor_input_prefix);
				binding_factor_input.extend_from_slice(identifier.serialize().as_ref());
				(*identifier, binding_factor_input)
			})
			.collect())
	}
}

#[cfg(feature = "serialization")]
impl<C> SigningPackage<C>
where
	C: Ciphersuite,
{
	/// Serialize the struct into a Vec.
	pub fn serialize(&self) -> Result<Vec<u8>, Error<C>> {
		serialization::Serialize::serialize(&self)
	}

	/// Deserialize the struct from a slice of bytes.
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error<C>> {
		serialization::Deserialize::deserialize(bytes)
	}
}

/// The product of all signers' individual commitments, published as part of the
/// final signature.
#[derive(Clone, PartialEq, Eq)]
pub struct GroupCommitment<C: Ciphersuite>(pub(crate) Element<C>);

impl<C> GroupCommitment<C>
where
	C: Ciphersuite,
{
	/// Return the underlying element.
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
	pub(crate) fn to_element(self) -> <C::Group as Group>::Element {
		self.0
	}

	/// Return the underlying element.
	#[cfg(feature = "internals")]
	pub fn from_element(element: Element<C>) -> Self {
		Self(element)
	}
}

/// Generates the group commitment which is published as part of the joint
/// Schnorr signature.
///
/// Implements [`compute_group_commitment`] from the spec.
///
/// [`compute_group_commitment`]: https://datatracker.ietf.org/doc/html/rfc9591#name-group-commitment-computatio
#[cfg_attr(feature = "internals", visibility::make(pub))]
#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
fn compute_group_commitment<C>(
	signing_package: &SigningPackage<C>,
	binding_factor_list: &BindingFactorList<C>,
) -> Result<GroupCommitment<C>, Error<C>>
where
	C: Ciphersuite,
{
	let identity = <C::Group as Group>::identity();

	let mut group_commitment = <C::Group as Group>::identity();

	// Number of signing participants we are iterating over.
	let n = signing_package.signing_commitments().len();

	let mut binding_scalars = Vec::with_capacity(n);

	let mut binding_elements = Vec::with_capacity(n);

	for (commitment_identifier, commitment) in signing_package.signing_commitments() {
		// The following check prevents a party from accidentally revealing their share.
		// Note that the '&&' operator would be sufficient.
		if identity == commitment.binding.value() || identity == commitment.hiding.value() {
			return Err(Error::IdentityCommitment);
		}

		let binding_factor =
			binding_factor_list.get(commitment_identifier).ok_or(Error::UnknownIdentifier)?;

		// Collect the binding commitments and their binding factors for one big
		// multiscalar multiplication at the end.
		binding_elements.push(commitment.binding.value());
		binding_scalars.push(binding_factor.0);

		group_commitment = group_commitment + commitment.hiding.value();
	}

	let accumulated_binding_commitment: Element<C> =
		VartimeMultiscalarMul::<C>::vartime_multiscalar_mul(binding_scalars, binding_elements);

	group_commitment = group_commitment + accumulated_binding_commitment;

	Ok(GroupCommitment(group_commitment))
}

////////////////////////////////////////////////////////////////////////////////
// Aggregation
////////////////////////////////////////////////////////////////////////////////

/// Aggregates the signature shares to produce a final signature that
/// can be verified with the group public key.
///
/// `signature_shares` maps the identifier of each participant to the
/// [`round2::SignatureShare`] they sent. These identifiers must come from whatever mapping
/// the coordinator has between communication channels and participants, i.e.
/// they must have assurance that the [`round2::SignatureShare`] came from
/// the participant with that identifier.
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
pub fn aggregate<C>(
	signing_package: &SigningPackage<C>,
	signature_shares: &BTreeMap<Identifier<C>, round2::SignatureShare<C>>,
	pubkeys: &keys::PublicKeyPackage<C>,
) -> Result<Signature<C>, Error<C>>
where
	C: Ciphersuite,
{
	// Check if signing_package.signing_commitments and signature_shares have
	// the same set of identifiers, and if they are all in pubkeys.verifying_shares.
	if signing_package.signing_commitments().len() != signature_shares.len() {
		return Err(Error::UnknownIdentifier);
	}

	if !signing_package.signing_commitments().keys().all(|id| {
		#[cfg(feature = "cheater-detection")]
		return signature_shares.contains_key(id) && pubkeys.verifying_shares().contains_key(id);
		#[cfg(not(feature = "cheater-detection"))]
		return signature_shares.contains_key(id);
	}) {
		return Err(Error::UnknownIdentifier);
	}

	let (signing_package, signature_shares, pubkeys) =
		<C>::pre_aggregate(signing_package, signature_shares, pubkeys)?;

	// Encodes the signing commitment list produced in round one as part of generating [`BindingFactor`], the
	// binding factor.
	let binding_factor_list: BindingFactorList<C> =
		compute_binding_factor_list(&signing_package, &pubkeys.verifying_key, &[])?;
	// Compute the group commitment from signing commitments produced in round one.
	let group_commitment = compute_group_commitment(&signing_package, &binding_factor_list)?;

	// The aggregation of the signature shares by summing them up, resulting in
	// a plain Schnorr signature.
	//
	// Implements [`aggregate`] from the spec.
	//
	// [`aggregate`]: https://datatracker.ietf.org/doc/html/rfc9591#name-signature-share-aggregation
	let mut z = <<C::Group as Group>::Field>::zero();

	for signature_share in signature_shares.values() {
		z = z + signature_share.to_scalar();
	}

	let signature = Signature { R: group_commitment.0, z };

	// Verify the aggregate signature
	let verification_result = pubkeys.verifying_key.verify(signing_package.message(), &signature);

	// Only if the verification of the aggregate signature failed; verify each share to find the cheater.
	// This approach is more efficient since we don't need to verify all shares
	// if the aggregate signature is valid (which should be the common case).
	#[cfg(feature = "cheater-detection")]
	if verification_result.is_err() {
		detect_cheater(
			&group_commitment,
			&pubkeys,
			&signing_package,
			&signature_shares,
			&binding_factor_list,
		)?;
	}

	#[cfg(not(feature = "cheater-detection"))]
	verification_result?;

	Ok(signature)
}

/// Optional cheater detection feature
/// Each share is verified to find the cheater
#[cfg(feature = "cheater-detection")]
fn detect_cheater<C: Ciphersuite>(
	group_commitment: &GroupCommitment<C>,
	pubkeys: &keys::PublicKeyPackage<C>,
	signing_package: &SigningPackage<C>,
	signature_shares: &BTreeMap<Identifier<C>, round2::SignatureShare<C>>,
	binding_factor_list: &BindingFactorList<C>,
) -> Result<(), Error<C>> {
	// Compute the per-message challenge.
	let challenge =
		<C>::challenge(&group_commitment.0, &pubkeys.verifying_key, signing_package.message())?;

	// Verify the signature shares.
	for (identifier, signature_share) in signature_shares {
		// Look up the public key for this signer, where `signer_pubkey` = _G.ScalarBaseMult(s[i])_,
		// and where s[i] is a secret share of the constant term of _f_, the secret polynomial.
		let verifying_share =
			pubkeys.verifying_shares.get(identifier).ok_or(Error::UnknownIdentifier)?;

		verify_signature_share_precomputed(
			*identifier,
			signing_package,
			binding_factor_list,
			group_commitment,
			signature_share,
			verifying_share,
			challenge,
		)?;
	}

	// We should never reach here; but we return an error to be safe.
	Err(Error::InvalidSignature)
}

/// Verify a signature share for the given participant `identifier`,
/// `verifying_share` and `signature_share`; with the `signing_package`
/// for which the signature share was produced and with the group's
/// `verifying_key`.
///
/// This is not required for regular FROST usage but might useful in certain
/// situations where it is desired to verify each individual signature share
/// before aggregating the signature.
pub fn verify_signature_share<C: Ciphersuite>(
	identifier: Identifier<C>,
	verifying_share: &keys::VerifyingShare<C>,
	signature_share: &round2::SignatureShare<C>,
	signing_package: &SigningPackage<C>,
	verifying_key: &VerifyingKey<C>,
) -> Result<(), Error<C>> {
	// In order to reuse `pre_aggregate()`, we need to create some "dummy" containers
	let signature_shares = BTreeMap::from([(identifier, *signature_share)]);
	let verifying_shares = BTreeMap::from([(identifier, *verifying_share)]);
	let public_key_package = PublicKeyPackage::new(verifying_shares, *verifying_key);

	let (signing_package, signature_shares, pubkeys) =
		<C>::pre_aggregate(signing_package, &signature_shares, &public_key_package)?;

	// Extract the processed values back from the "dummy" containers
	let verifying_share = pubkeys
		.verifying_shares()
		.get(&identifier)
		.expect("pre_aggregate() must keep the identifiers");
	let verifying_key = pubkeys.verifying_key();
	let signature_share = signature_shares
		.get(&identifier)
		.expect("pre_aggregate() must keep the identifiers");

	// Encodes the signing commitment list produced in round one as part of generating [`BindingFactor`], the
	// binding factor.
	let binding_factor_list: BindingFactorList<C> =
		compute_binding_factor_list(&signing_package, verifying_key, &[])?;

	// Compute the group commitment from signing commitments produced in round one.
	let group_commitment = compute_group_commitment(&signing_package, &binding_factor_list)?;

	// Compute the per-message challenge.
	let challenge = <C>::challenge(
		&group_commitment.clone().to_element(),
		verifying_key,
		signing_package.message().as_slice(),
	)?;

	verify_signature_share_precomputed(
		identifier,
		&signing_package,
		&binding_factor_list,
		&group_commitment,
		signature_share,
		verifying_share,
		challenge,
	)
}

/// Similar to [`verify_signature_share()`] but using a precomputed
/// `binding_factor_list` and `challenge`.
#[cfg_attr(feature = "internals", visibility::make(pub))]
#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
fn verify_signature_share_precomputed<C: Ciphersuite>(
	signature_share_identifier: Identifier<C>,
	signing_package: &SigningPackage<C>,
	binding_factor_list: &BindingFactorList<C>,
	group_commitment: &GroupCommitment<C>,
	signature_share: &round2::SignatureShare<C>,
	verifying_share: &keys::VerifyingShare<C>,
	challenge: Challenge<C>,
) -> Result<(), Error<C>> {
	let lambda_i = derive_interpolating_value(&signature_share_identifier, signing_package)?;

	let binding_factor = binding_factor_list
		.get(&signature_share_identifier)
		.ok_or(Error::UnknownIdentifier)?;

	let R_share = signing_package
		.signing_commitment(&signature_share_identifier)
		.ok_or(Error::UnknownIdentifier)?
		.to_group_commitment_share(binding_factor);

	// Compute relation values to verify this signature share.
	<C>::verify_share(
		group_commitment,
		signature_share,
		signature_share_identifier,
		&R_share,
		verifying_share,
		lambda_i,
		&challenge,
	)?;

	Ok(())
}
