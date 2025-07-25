//! FROST keys, keygen, key shares
#![allow(clippy::type_complexity)]

use core::iter;

use alloc::{
	collections::{BTreeMap, BTreeSet},
	fmt::{self, Debug},
	string::ToString,
	vec::Vec,
};

use derive_getters::Getters;
#[cfg(any(test, feature = "test-impl"))]
use hex::FromHex;

use rand_core::{CryptoRng, RngCore};
use zeroize::{DefaultIsZeroes, Zeroize};

use crate::{
	serialization::{SerializableElement, SerializableScalar},
	Ciphersuite, Element, Error, Field, Group, Header, Identifier, Scalar, SigningKey,
	VerifyingKey,
};

#[cfg(feature = "serialization")]
use crate::serialization::{Deserialize, Serialize};

use super::compute_lagrange_coefficient;

/// Sum the commitments from all participants in a distributed key generation
/// run into a single group commitment.
#[cfg_attr(feature = "internals", visibility::make(pub))]
pub(crate) fn sum_commitments<C: Ciphersuite>(
	commitments: &[&VerifiableSecretSharingCommitment<C>],
) -> Result<VerifiableSecretSharingCommitment<C>, Error<C>> {
	let mut group_commitment =
		vec![
			CoefficientCommitment::new(<C::Group>::identity());
			commitments.first().ok_or(Error::IncorrectNumberOfCommitments)?.0.len()
		];
	for commitment in commitments {
		for (i, c) in group_commitment.iter_mut().enumerate() {
			*c = CoefficientCommitment::new(
				c.value() + commitment.0.get(i).ok_or(Error::IncorrectNumberOfCommitments)?.value(),
			);
		}
	}
	Ok(VerifiableSecretSharingCommitment(group_commitment))
}

/// Return a vector of randomly generated polynomial coefficients ([`Scalar`]s).
pub(crate) fn generate_coefficients<C: Ciphersuite, R: RngCore + CryptoRng>(
	size: usize,
	rng: &mut R,
) -> Vec<Scalar<C>> {
	iter::repeat_with(|| <<C::Group as Group>::Field>::random(rng))
		.take(size)
		.collect()
}

/// Return a list of default identifiers (1 to max_signers, inclusive).
#[cfg_attr(feature = "internals", visibility::make(pub))]
pub(crate) fn default_identifiers<C: Ciphersuite>(max_signers: u16) -> Vec<Identifier<C>> {
	(1..=max_signers)
		.map(|i| Identifier::<C>::try_from(i).expect("nonzero"))
		.collect::<Vec<_>>()
}

/// A secret scalar value representing a signer's share of the group secret.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "C: Ciphersuite"))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct SigningShare<C: Ciphersuite>(pub(crate) SerializableScalar<C>);

impl<C> SigningShare<C>
where
	C: Ciphersuite,
{
	/// Create a new [`SigningShare`] from a scalar.
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
	pub(crate) fn new(scalar: Scalar<C>) -> Self {
		Self(SerializableScalar(scalar))
	}

	/// Get the inner scalar.
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
	pub(crate) fn to_scalar(&self) -> Scalar<C> {
		self.0 .0
	}

	/// Deserialize from bytes
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error<C>> {
		Ok(Self(SerializableScalar::deserialize(bytes)?))
	}

	/// Serialize to bytes
	pub fn serialize(&self) -> Vec<u8> {
		self.0.serialize()
	}

	/// Computes the signing share from a list of coefficients.
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	pub(crate) fn from_coefficients(coefficients: &[Scalar<C>], peer: Identifier<C>) -> Self {
		Self::new(evaluate_polynomial(peer, coefficients))
	}
}

impl<C> Debug for SigningShare<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_tuple("SigningShare").field(&"<redacted>").finish()
	}
}

impl<C> Default for SigningShare<C>
where
	C: Ciphersuite,
{
	fn default() -> Self {
		Self::new(<<C::Group as Group>::Field>::zero())
	}
}

// Implements [`Zeroize`] by overwriting a value with the [`Default::default()`] value
impl<C> DefaultIsZeroes for SigningShare<C> where C: Ciphersuite {}

#[cfg(any(test, feature = "test-impl"))]
impl<C> FromHex for SigningShare<C>
where
	C: Ciphersuite,
{
	type Error = &'static str;

	fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
		let v: Vec<u8> = FromHex::from_hex(hex).map_err(|_| "invalid hex")?;
		Self::deserialize(&v).map_err(|_| "malformed scalar")
	}
}

/// A public group element that represents a single signer's public verification share.
#[derive(Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "C: Ciphersuite"))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct VerifyingShare<C>(pub(super) SerializableElement<C>)
where
	C: Ciphersuite;

impl<C> VerifyingShare<C>
where
	C: Ciphersuite,
{
	/// Create a new [`VerifyingShare`] from a element.
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
	pub(crate) fn new(element: Element<C>) -> Self {
		Self(SerializableElement(element))
	}

	/// Get the inner element.
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
	#[allow(dead_code)]
	pub(crate) fn to_element(&self) -> Element<C> {
		self.0 .0
	}

	/// Deserialize from bytes
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error<C>> {
		Ok(Self(SerializableElement::deserialize(bytes)?))
	}

	/// Serialize to bytes
	pub fn serialize(&self) -> Result<Vec<u8>, Error<C>> {
		self.0.serialize()
	}

	/// Computes a verifying share for a peer given the group commitment.
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	pub(crate) fn from_commitment(
		identifier: Identifier<C>,
		commitment: &VerifiableSecretSharingCommitment<C>,
	) -> VerifyingShare<C> {
		// DKG Round 2, Step 4
		//
		// > Any participant can compute the public verification share of any
		// > other participant by calculating
		// > Y_i = ∏_{j=1}^n ∏_{k=0}^{t−1} φ_{jk}^{i^k mod q}.
		//
		// Rewriting the equation by moving the product over j to further inside
		// the equation:
		// Y_i = ∏_{k=0}^{t−1} (∏_{j=1}^n φ_{jk})^{i^k mod q}
		// i.e. we can operate on the sum of all φ_j commitments, which is
		// what is passed to the functions.
		VerifyingShare::new(evaluate_vss(identifier, commitment))
	}
}

impl<C> Debug for VerifyingShare<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_tuple("VerifyingShare")
			.field(&self.serialize().map(hex::encode).unwrap_or("<invalid>".to_string()))
			.finish()
	}
}

impl<C> From<SigningShare<C>> for VerifyingShare<C>
where
	C: Ciphersuite,
{
	fn from(secret: SigningShare<C>) -> VerifyingShare<C> {
		VerifyingShare::new(<C::Group>::generator() * secret.to_scalar())
	}
}

/// A [`Group::Element`] newtype that is a commitment to one coefficient of our secret polynomial.
///
/// This is a (public) commitment to one coefficient of a secret polynomial used for performing
/// verifiable secret sharing for a Shamir secret share.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "C: Ciphersuite"))]
pub struct CoefficientCommitment<C: Ciphersuite>(pub(crate) SerializableElement<C>);

impl<C> CoefficientCommitment<C>
where
	C: Ciphersuite,
{
	/// Create a new CoefficientCommitment.
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	pub(crate) fn new(value: Element<C>) -> Self {
		Self(SerializableElement(value))
	}

	/// Deserialize from bytes
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error<C>> {
		Ok(Self(SerializableElement::deserialize(bytes)?))
	}

	/// Serialize to bytes
	pub fn serialize(&self) -> Result<Vec<u8>, Error<C>> {
		self.0.serialize()
	}

	/// Returns inner element value
	pub fn value(&self) -> Element<C> {
		self.0 .0
	}
}

impl<C> Debug for CoefficientCommitment<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_tuple("CoefficientCommitment")
			.field(&self.serialize().map(hex::encode).unwrap_or("<invalid>".to_string()))
			.finish()
	}
}

/// Contains the commitments to the coefficients for our secret polynomial _f_,
/// used to generate participants' key shares.
///
/// [`VerifiableSecretSharingCommitment`] contains a set of commitments to the coefficients (which
/// themselves are scalars) for a secret polynomial f, where f is used to
/// generate each ith participant's key share f(i). Participants use this set of
/// commitments to perform verifiable secret sharing.
///
/// Note that participants MUST be assured that they have the *same*
/// [`VerifiableSecretSharingCommitment`], either by performing pairwise comparison, or by using
/// some agreed-upon public location for publication, where each participant can
/// ensure that they received the correct (and same) value.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "C: Ciphersuite"))]
pub struct VerifiableSecretSharingCommitment<C: Ciphersuite>(
	pub(crate) Vec<CoefficientCommitment<C>>,
);

impl<C> VerifiableSecretSharingCommitment<C>
where
	C: Ciphersuite,
{
	/// Create a new VerifiableSecretSharingCommitment.
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	pub(crate) fn new(coefficients: Vec<CoefficientCommitment<C>>) -> Self {
		Self(coefficients)
	}

	/// Returns serialized coefficent commitments
	pub fn serialize(&self) -> Result<Vec<Vec<u8>>, Error<C>> {
		self.0.iter().map(|cc| cc.serialize()).collect::<Result<_, Error<C>>>()
	}

	/// Returns VerifiableSecretSharingCommitment from an iterator of serialized
	/// CoefficientCommitments (e.g. a [`Vec<Vec<u8>>`]).
	pub fn deserialize<I, V>(serialized_coefficient_commitments: I) -> Result<Self, Error<C>>
	where
		I: IntoIterator<Item = V>,
		V: AsRef<[u8]>,
	{
		let mut coefficient_commitments = Vec::new();
		for cc in serialized_coefficient_commitments.into_iter() {
			coefficient_commitments.push(CoefficientCommitment::<C>::deserialize(cc.as_ref())?);
		}

		Ok(Self::new(coefficient_commitments))
	}

	/// Get the VerifyingKey matching this commitment vector (which is the first
	/// element in the vector), or an error if the vector is empty.
	pub(crate) fn verifying_key(&self) -> Result<VerifyingKey<C>, Error<C>> {
		Ok(VerifyingKey::new(self.0.first().ok_or(Error::MissingCommitment)?.0 .0))
	}

	/// Returns the coefficient commitments.
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	pub(crate) fn coefficients(&self) -> &[CoefficientCommitment<C>] {
		&self.0
	}
}

/// A secret share generated by performing a (t-out-of-n) secret sharing scheme,
/// generated by a dealer performing [`generate_with_dealer`].
///
/// `n` is the total number of shares and `t` is the threshold required to reconstruct the secret;
/// in this case we use Shamir's secret sharing.
///
/// As a solution to the secret polynomial _f_ (a 'point'), the `identifier` is the x-coordinate,
/// and the `value` is the y-coordinate.
///
/// To derive a FROST keypair, the receiver of the [`SecretShare`] *must* call
/// .into(), which under the hood also performs validation.
#[derive(Clone, Debug, Zeroize, PartialEq, Eq, Getters)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "C: Ciphersuite"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct SecretShare<C: Ciphersuite> {
	/// Serialization header
	#[getter(skip)]
	pub(crate) header: Header<C>,
	/// The participant identifier of this [`SecretShare`].
	#[zeroize(skip)]
	pub(crate) identifier: Identifier<C>,
	/// Secret Key.
	pub(crate) signing_share: SigningShare<C>,
	#[zeroize(skip)]
	/// The commitments to be distributed among signers.
	pub(crate) commitment: VerifiableSecretSharingCommitment<C>,
}

impl<C> SecretShare<C>
where
	C: Ciphersuite,
{
	/// Create a new [`SecretShare`] instance.
	pub fn new(
		identifier: Identifier<C>,
		signing_share: SigningShare<C>,
		commitment: VerifiableSecretSharingCommitment<C>,
	) -> Self {
		SecretShare { header: Header::default(), identifier, signing_share, commitment }
	}

	/// Verifies that a secret share is consistent with a verifiable secret sharing commitment,
	/// and returns the derived group info for the participant (their public verification share,
	/// and the group public key) if successful.
	///
	/// This ensures that this participant's share has been generated using the same
	/// mechanism as all other signing participants. Note that participants *MUST*
	/// ensure that they have the same view as all other participants of the
	/// commitment!
	///
	/// An implementation of `vss_verify()` from the [spec].
	/// This also implements `derive_group_info()` from the [spec] (which is very similar),
	/// but only for this participant.
	///
	/// [spec]: https://datatracker.ietf.org/doc/html/rfc9591#appendix-C.2-3
	pub fn verify(&self) -> Result<(VerifyingShare<C>, VerifyingKey<C>), Error<C>> {
		let f_result = <C::Group>::generator() * self.signing_share.to_scalar();
		let result = evaluate_vss(self.identifier, &self.commitment);

		if !(f_result == result) {
			// The culprit needs to be identified by the caller if needed,
			// because this function is called in two different contexts:
			// - after trusted dealer key generation, by the participant who receives the
			//   SecretShare. In that case it does not make sense to identify themselves as the
			//   culprit, since the issue was with the Coordinator or in the communication.
			// - during DKG, where a "fake" SecretShare is built just to reuse the verification
			//   logic and it does make sense to identify the culprit. Note that in this case,
			//   self.identifier is the caller's identifier and not the culprit's, so we couldn't
			//   identify the culprit inside this function anyway.
			return Err(Error::InvalidSecretShare { culprit: None });
		}

		Ok((VerifyingShare::new(result), self.commitment.verifying_key()?))
	}
}

#[cfg(feature = "serialization")]
impl<C> SecretShare<C>
where
	C: Ciphersuite,
{
	/// Serialize the struct into a Vec.
	pub fn serialize(&self) -> Result<Vec<u8>, Error<C>> {
		Serialize::serialize(&self)
	}

	/// Deserialize the struct from a slice of bytes.
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error<C>> {
		Deserialize::deserialize(bytes)
	}
}

/// The identifier list to use when generating key shares.
pub enum IdentifierList<'a, C: Ciphersuite> {
	/// Use the default values (1 to max_signers, inclusive).
	Default,
	/// A user-provided list of identifiers.
	Custom(&'a [Identifier<C>]),
}

/// Allows all participants' keys to be generated using a central, trusted
/// dealer.
///
/// Under the hood, this performs verifiable secret sharing, which itself uses
/// Shamir secret sharing, from which each share becomes a participant's secret
/// key. The output from this function is a set of shares along with one single
/// commitment that participants use to verify the integrity of the share.
///
/// Implements [`trusted_dealer_keygen`] from the spec.
///
/// [`trusted_dealer_keygen`]: https://datatracker.ietf.org/doc/html/rfc9591#appendix-C
pub fn generate_with_dealer<C: Ciphersuite, R: RngCore + CryptoRng>(
	max_signers: u16,
	min_signers: u16,
	identifiers: IdentifierList<C>,
	rng: &mut R,
) -> Result<(BTreeMap<Identifier<C>, SecretShare<C>>, PublicKeyPackage<C>), Error<C>> {
	let mut bytes = [0; 64];
	rng.fill_bytes(&mut bytes);

	let key = SigningKey::new(rng);

	split(&key, max_signers, min_signers, identifiers, rng)
}

/// Splits an existing key into FROST shares.
///
/// This is identical to [`generate_with_dealer`] but receives an existing key
/// instead of generating a fresh one. This is useful in scenarios where
/// the key needs to be generated externally or must be derived from e.g. a
/// seed phrase.
pub fn split<C: Ciphersuite, R: RngCore + CryptoRng>(
	key: &SigningKey<C>,
	max_signers: u16,
	min_signers: u16,
	identifiers: IdentifierList<C>,
	rng: &mut R,
) -> Result<(BTreeMap<Identifier<C>, SecretShare<C>>, PublicKeyPackage<C>), Error<C>> {
	validate_num_of_signers(min_signers, max_signers)?;

	if let IdentifierList::Custom(identifiers) = &identifiers {
		if identifiers.len() != max_signers as usize {
			return Err(Error::IncorrectNumberOfIdentifiers);
		}
	}

	let verifying_key = VerifyingKey::from(key);

	let coefficients = generate_coefficients::<C, R>(min_signers as usize - 1, rng);

	let secret_shares = match identifiers {
		IdentifierList::Default => {
			let identifiers = default_identifiers(max_signers);
			generate_secret_shares(key, max_signers, min_signers, coefficients, &identifiers)?
		},
		IdentifierList::Custom(identifiers) =>
			generate_secret_shares(key, max_signers, min_signers, coefficients, identifiers)?,
	};
	let mut verifying_shares: BTreeMap<Identifier<C>, VerifyingShare<C>> = BTreeMap::new();

	let mut secret_shares_by_id: BTreeMap<Identifier<C>, SecretShare<C>> = BTreeMap::new();

	for secret_share in secret_shares {
		let signer_public = secret_share.signing_share.into();
		verifying_shares.insert(secret_share.identifier, signer_public);

		secret_shares_by_id.insert(secret_share.identifier, secret_share);
	}

	Ok((
		secret_shares_by_id,
		PublicKeyPackage { header: Header::default(), verifying_shares, verifying_key },
	))
}

/// Evaluate the polynomial with the given coefficients (constant term first)
/// at the point x=identifier using Horner's method.
///
/// Implements [`polynomial_evaluate`] from the spec.
///
/// [`polynomial_evaluate`]: https://datatracker.ietf.org/doc/html/rfc9591#name-additional-polynomial-opera
fn evaluate_polynomial<C: Ciphersuite>(
	identifier: Identifier<C>,
	coefficients: &[Scalar<C>],
) -> Scalar<C> {
	let mut value = <<C::Group as Group>::Field>::zero();

	let ell = identifier;
	for coeff in coefficients.iter().skip(1).rev() {
		value = value + *coeff;
		value = value * ell.to_scalar();
	}
	value = value + *coefficients.first().expect("coefficients must have at least one element");
	value
}

/// Evaluates the right-hand side of the VSS verification equation, namely
/// ∏^{t−1}_{k=0} φ^{i^k mod q}_{ℓk} (multiplicative notation) using
/// `identifier` as `i` and the `commitment` as the commitment vector φ_ℓ.
///
/// This is also used in Round 2, Step 4 of the DKG.
fn evaluate_vss<C: Ciphersuite>(
	identifier: Identifier<C>,
	commitment: &VerifiableSecretSharingCommitment<C>,
) -> Element<C> {
	let i = identifier.to_scalar();

	let (_, result) = commitment.0.iter().fold(
		(<<C::Group as Group>::Field>::one(), <C::Group>::identity()),
		|(i_to_the_k, sum_so_far), comm_k| {
			(i * i_to_the_k, sum_so_far + comm_k.value() * i_to_the_k)
		},
	);
	result
}

/// A FROST keypair, which can be generated either by a trusted dealer or using
/// a DKG.
///
/// When using a central dealer, [`SecretShare`]s are distributed to
/// participants, who then perform verification, before deriving
/// [`KeyPackage`]s, which they store to later use during signing.
#[derive(Clone, Debug, PartialEq, Eq, Getters, Zeroize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "C: Ciphersuite"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct KeyPackage<C: Ciphersuite> {
	/// Serialization header
	#[getter(skip)]
	pub(crate) header: Header<C>,
	/// Denotes the participant identifier each secret share key package is owned by.
	#[zeroize(skip)]
	pub(crate) identifier: Identifier<C>,
	/// This participant's signing share. This is secret.
	pub(crate) signing_share: SigningShare<C>,
	/// This participant's public key.
	#[zeroize(skip)]
	pub(crate) verifying_share: VerifyingShare<C>,
	/// The public verifying key that represents the entire group.
	#[zeroize(skip)]
	pub(crate) verifying_key: VerifyingKey<C>,
	pub(crate) min_signers: u16,
}

impl<C> KeyPackage<C>
where
	C: Ciphersuite,
{
	/// Create a new [`KeyPackage`] instance.
	pub fn new(
		identifier: Identifier<C>,
		signing_share: SigningShare<C>,
		verifying_share: VerifyingShare<C>,
		verifying_key: VerifyingKey<C>,
		min_signers: u16,
	) -> Self {
		Self {
			header: Header::default(),
			identifier,
			signing_share,
			verifying_share,
			verifying_key,
			min_signers,
		}
	}
}

#[cfg(feature = "serialization")]
impl<C> KeyPackage<C>
where
	C: Ciphersuite,
{
	/// Serialize the struct into a Vec.
	pub fn serialize(&self) -> Result<Vec<u8>, Error<C>> {
		Serialize::serialize(&self)
	}

	/// Deserialize the struct from a slice of bytes.
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error<C>> {
		Deserialize::deserialize(bytes)
	}
}

impl<C> TryFrom<SecretShare<C>> for KeyPackage<C>
where
	C: Ciphersuite,
{
	type Error = Error<C>;

	/// Tries to verify a share and construct a [`KeyPackage`] from it.
	///
	/// When participants receive a [`SecretShare`] from the dealer, they
	/// *MUST* verify the integrity of the share before continuing on to
	/// transform it into a signing/verification keypair. Here, we assume that
	/// every participant has the same view of the commitment issued by the
	/// dealer, but implementations *MUST* make sure that all participants have
	/// a consistent view of this commitment in practice.
	fn try_from(secret_share: SecretShare<C>) -> Result<Self, Error<C>> {
		let (verifying_share, verifying_key) = secret_share.verify()?;

		Ok(KeyPackage {
			header: Header::default(),
			identifier: secret_share.identifier,
			signing_share: secret_share.signing_share,
			verifying_share,
			verifying_key,
			min_signers: secret_share.commitment.0.len() as u16,
		})
	}
}

/// Public data that contains all the signers' verifying shares as well as the
/// group verifying key.
///
/// Used for verification purposes before publishing a signature.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "C: Ciphersuite"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct PublicKeyPackage<C: Ciphersuite> {
	/// Serialization header
	#[getter(skip)]
	pub(crate) header: Header<C>,
	/// The verifying shares for all participants. Used to validate signature
	/// shares they generate.
	pub(crate) verifying_shares: BTreeMap<Identifier<C>, VerifyingShare<C>>,
	/// The joint public key for the entire group.
	pub(crate) verifying_key: VerifyingKey<C>,
}

impl<C> PublicKeyPackage<C>
where
	C: Ciphersuite,
{
	/// Create a new [`PublicKeyPackage`] instance.
	pub fn new(
		verifying_shares: BTreeMap<Identifier<C>, VerifyingShare<C>>,
		verifying_key: VerifyingKey<C>,
	) -> Self {
		Self { header: Header::default(), verifying_shares, verifying_key }
	}

	/// Computes the public key package given a list of participant identifiers
	/// and a [`VerifiableSecretSharingCommitment`]. This is useful in scenarios
	/// where the commitments are published somewhere and it's desirable to
	/// recreate the public key package from them.
	pub fn from_commitment(
		identifiers: &BTreeSet<Identifier<C>>,
		commitment: &VerifiableSecretSharingCommitment<C>,
	) -> Result<PublicKeyPackage<C>, Error<C>> {
		let verifying_keys: BTreeMap<_, _> = identifiers
			.iter()
			.map(|id| (*id, VerifyingShare::from_commitment(*id, commitment)))
			.collect();
		Ok(PublicKeyPackage::new(verifying_keys, VerifyingKey::from_commitment(commitment)?))
	}

	/// Computes the public key package given a map of participant identifiers
	/// and their [`VerifiableSecretSharingCommitment`] from a distributed key
	/// generation process. This is useful in scenarios where the commitments
	/// are published somewhere and it's desirable to recreate the public key
	/// package from them.
	pub fn from_dkg_commitments(
		commitments: &BTreeMap<Identifier<C>, &VerifiableSecretSharingCommitment<C>>,
	) -> Result<PublicKeyPackage<C>, Error<C>> {
		let identifiers: BTreeSet<_> = commitments.keys().copied().collect();
		let commitments: Vec<_> = commitments.values().copied().collect();
		let group_commitment = sum_commitments(&commitments)?;
		Self::from_commitment(&identifiers, &group_commitment)
	}
}

#[cfg(feature = "serialization")]
impl<C> PublicKeyPackage<C>
where
	C: Ciphersuite,
{
	/// Serialize the struct into a Vec.
	pub fn serialize(&self) -> Result<Vec<u8>, Error<C>> {
		Serialize::serialize(&self)
	}

	/// Deserialize the struct from a slice of bytes.
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error<C>> {
		Deserialize::deserialize(bytes)
	}
}

/// Validates the number of signers.
#[cfg_attr(feature = "internals", visibility::make(pub))]
fn validate_num_of_signers<C: Ciphersuite>(
	min_signers: u16,
	max_signers: u16,
) -> Result<(), Error<C>> {
	if min_signers < 2 {
		return Err(Error::InvalidMinSigners);
	}

	if max_signers < 2 {
		return Err(Error::InvalidMaxSigners);
	}

	if min_signers > max_signers {
		return Err(Error::InvalidMinSigners);
	}

	Ok(())
}

/// Generate a secret polynomial to use in secret sharing, for the given
/// secret value. Also validates the given parameters.
///
/// Returns the full vector of coefficients in little-endian order (including the
/// given secret, which is the first element) and a [`VerifiableSecretSharingCommitment`]
/// which contains commitments to those coefficients.
///
/// Returns an error if the parameters (max_signers, min_signers) are inconsistent.
pub(crate) fn generate_secret_polynomial<C: Ciphersuite>(
	secret: &SigningKey<C>,
	max_signers: u16,
	min_signers: u16,
	mut coefficients: Vec<Scalar<C>>,
) -> Result<(Vec<Scalar<C>>, VerifiableSecretSharingCommitment<C>), Error<C>> {
	validate_num_of_signers(min_signers, max_signers)?;

	if coefficients.len() != min_signers as usize - 1 {
		return Err(Error::InvalidCoefficients);
	}

	// Prepend the secret, which is the 0th coefficient
	coefficients.insert(0, secret.scalar);

	// Create the vector of commitments
	let commitment: Vec<_> = coefficients
		.iter()
		.map(|c| CoefficientCommitment::new(<C::Group as Group>::generator() * *c))
		.collect();
	let commitment: VerifiableSecretSharingCommitment<C> =
		VerifiableSecretSharingCommitment(commitment);

	Ok((coefficients, commitment))
}

/// Creates secret shares for a given secret using the given coefficients.
///
/// This function accepts a secret from which shares are generated,
/// and a list of threshold-1 coefficients. While in FROST this secret
/// and coefficients should always be generated randomly, we allow them
/// to be specified for this internal function for testability.
///
/// Internally, [`generate_secret_shares`] performs verifiable secret sharing, which
/// generates shares via Shamir Secret Sharing, and then generates public
/// commitments to those shares.
///
/// More specifically, [`generate_secret_shares`]:
/// - Interpret [secret, `coefficients[0]`, ...] as a secret polynomial f
/// - For each participant i, their secret share is f(i)
/// - The commitment to the secret polynomial f is [g^secret, `g^coefficients[0]`, ...]
///
/// Implements [`secret_share_shard`] from the spec.
///
/// [`secret_share_shard`]: https://datatracker.ietf.org/doc/html/rfc9591#name-shamir-secret-sharing
pub(crate) fn generate_secret_shares<C: Ciphersuite>(
	secret: &SigningKey<C>,
	max_signers: u16,
	min_signers: u16,
	coefficients: Vec<Scalar<C>>,
	identifiers: &[Identifier<C>],
) -> Result<Vec<SecretShare<C>>, Error<C>> {
	let mut secret_shares: Vec<SecretShare<C>> = Vec::with_capacity(max_signers as usize);

	let (coefficients, commitment) =
		generate_secret_polynomial(secret, max_signers, min_signers, coefficients)?;

	let identifiers_set: BTreeSet<_> = identifiers.iter().collect();
	if identifiers_set.len() != identifiers.len() {
		return Err(Error::DuplicatedIdentifier);
	}

	for id in identifiers {
		let signing_share = SigningShare::from_coefficients(&coefficients, *id);

		secret_shares.push(SecretShare {
			header: Header::default(),
			identifier: *id,
			signing_share,
			commitment: commitment.clone(),
		});
	}

	Ok(secret_shares)
}

/// Recompute the secret from at least `min_signers` secret shares (inside
/// [`KeyPackage`]s) using Lagrange interpolation.
///
/// This can be used if for some reason the original key must be restored; e.g.
/// if threshold signing is not required anymore.
///
/// This is NOT required to sign with FROST; the point of FROST is being
/// able to generate signatures only using the shares, without having to
/// reconstruct the original key.
///
/// The caller is responsible for providing at least `min_signers` packages;
/// if less than that is provided, a different key will be returned.
pub fn reconstruct<C: Ciphersuite>(
	key_packages: &[KeyPackage<C>],
) -> Result<SigningKey<C>, Error<C>> {
	if key_packages.is_empty() {
		return Err(Error::IncorrectNumberOfShares);
	}
	// There is no obvious way to get `min_signers` in order to validate the
	// size of `secret_shares`. Since that is just a best-effort validation,
	// we don't need to worry too much about adversarial situations where people
	// lie about min_signers, so just get the minimum value out of all of them.
	let min_signers = key_packages
		.iter()
		.map(|k| k.min_signers)
		.min()
		.expect("should not be empty since that was just tested");
	if key_packages.len() < min_signers as usize {
		return Err(Error::IncorrectNumberOfShares);
	}

	let mut secret = <<C::Group as Group>::Field>::zero();

	let identifiers: BTreeSet<_> = key_packages.iter().map(|s| s.identifier()).cloned().collect();

	if identifiers.len() != key_packages.len() {
		return Err(Error::DuplicatedIdentifier);
	}

	// Compute the Lagrange coefficients
	for key_package in key_packages.iter() {
		let lagrange_coefficient =
			compute_lagrange_coefficient(&identifiers, None, key_package.identifier)?;

		// Compute y = f(0) via polynomial interpolation of these t-of-n solutions ('points) of f
		secret = secret + (lagrange_coefficient * key_package.signing_share().to_scalar());
	}

	Ok(SigningKey { scalar: secret })
}
