use core::fmt::Debug;

use crate::{
	error::Error,
	identifier::Identifier,
	serialization::{Deserialize, ElementSerialization, ScalarSerialization, Serialize},
	traits::{Ciphersuite, Element, Field, Group, Scalar},
	util::{element_is_valid, scalar_is_valid},
	verifying_key::VerifyingKey,
	Header,
};
use alloc::collections::BTreeMap;
use sp_std::vec::Vec;
use zeroize::{DefaultIsZeroes, Zeroize};

#[cfg(feature = "std")]
use hex::FromHex;

/// A secret scalar value representing a signer's share of the group secret.
#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
#[serde(try_from = "ScalarSerialization<C>")]
#[serde(into = "ScalarSerialization<C>")]
pub struct SigningShare<C: Ciphersuite>(pub Scalar<C>);

impl<C> SigningShare<C>
where
	C: Ciphersuite,
{
	/// Create a new [`SigningShare`] from a scalar.
	pub fn new(scalar: Scalar<C>) -> Self {
		Self(scalar)
	}

	/// Get the inner scalar.
	pub fn to_scalar(&self) -> Scalar<C> {
		self.0
	}

	/// Deserialize from bytes
	pub fn deserialize(
		bytes: <<C::Group as Group>::Field as Field>::Serialization,
	) -> Result<Self, Error> {
		<<C::Group as Group>::Field>::deserialize(&bytes)
			.map(|scalar| Self(scalar))
			.map_err(|e| e.into())
	}

	/// Serialize to bytes
	pub fn serialize(&self) -> <<C::Group as Group>::Field as Field>::Serialization {
		<<C::Group as Group>::Field>::serialize(&self.0)
	}

	/// Computes the signing share from a list of coefficients.
	#[allow(dead_code)]
	pub fn from_coefficients(coefficients: &[Scalar<C>], peer: Identifier<C>) -> Self {
		Self(evaluate_polynomial(peer, coefficients))
	}

	/// Verifies that a signing share is valid aka not zero
	pub fn is_valid(&self) -> bool {
		scalar_is_valid::<C>(&self.0)
	}
}

impl<C> Debug for SigningShare<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_tuple("SigningShare").field(&"<redacted>").finish()
	}
}

impl<C> Default for SigningShare<C>
where
	C: Ciphersuite,
{
	fn default() -> Self {
		Self(<<C::Group as Group>::Field>::zero())
	}
}

// Implements [`Zeroize`] by overwriting a value with the [`Default::default()`] value
impl<C> DefaultIsZeroes for SigningShare<C> where C: Ciphersuite {}

#[cfg(any(test, feature = "std"))]
impl<C> FromHex for SigningShare<C>
where
	C: Ciphersuite,
{
	type Error = &'static str;

	fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
		let v: Vec<u8> = FromHex::from_hex(hex).map_err(|_| "invalid hex")?;
		match v.try_into() {
			Ok(bytes) => Self::deserialize(bytes).map_err(|_| "malformed secret encoding"),
			Err(_) => Err("malformed secret encoding"),
		}
	}
}

impl<C> TryFrom<ScalarSerialization<C>> for SigningShare<C>
where
	C: Ciphersuite,
{
	type Error = Error;

	fn try_from(value: ScalarSerialization<C>) -> Result<Self, Self::Error> {
		Self::deserialize(value.0)
	}
}

impl<C> From<SigningShare<C>> for ScalarSerialization<C>
where
	C: Ciphersuite,
{
	fn from(value: SigningShare<C>) -> Self {
		Self(value.serialize())
	}
}

/// A public group element that represents a single signer's public verification share.
#[derive(Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
#[serde(try_from = "ElementSerialization<C>")]
#[serde(into = "ElementSerialization<C>")]
pub struct VerifyingShare<C>(pub Element<C>)
where
	C: Ciphersuite;

impl<C> VerifyingShare<C>
where
	C: Ciphersuite,
{
	/// Create a new [`VerifyingShare`] from an element.
	pub fn new(element: Element<C>) -> Self {
		Self(element)
	}

	/// Get the inner element.
	pub fn to_element(&self) -> Element<C> {
		self.0
	}

	/// Deserialize from bytes
	pub fn deserialize(bytes: <C::Group as Group>::Serialization) -> Result<Self, Error> {
		<C::Group as Group>::deserialize(&bytes)
			.map(|element| Self(element))
			.map_err(|e| e.into())
	}

	/// Serialize to bytes
	pub fn serialize(&self) -> <C::Group as Group>::Serialization {
		<C::Group as Group>::serialize(&self.0)
	}

	/// Verifies that a verifying share is valid aka not zero or the base point
	pub fn is_valid(&self) -> bool {
		element_is_valid::<C>(&self.0)
	}
}

impl<C> Debug for VerifyingShare<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		f.debug_tuple("VerifyingShare").field(&hex::encode(self.serialize())).finish()
	}
}

impl<C> TryFrom<ElementSerialization<C>> for VerifyingShare<C>
where
	C: Ciphersuite,
{
	type Error = Error;

	fn try_from(value: ElementSerialization<C>) -> Result<Self, Self::Error> {
		Self::deserialize(value.0)
	}
}

impl<C> From<VerifyingShare<C>> for ElementSerialization<C>
where
	C: Ciphersuite,
{
	fn from(value: VerifyingShare<C>) -> Self {
		Self(value.serialize())
	}
}

/// A [`Group::Element`] newtype that is a commitment to one coefficient of our secret polynomial.
///
/// This is a (public) commitment to one coefficient of a secret polynomial used for performing
/// verifiable secret sharing for a Shamir secret share.
#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
#[serde(try_from = "ElementSerialization<C>")]
#[serde(into = "ElementSerialization<C>")]
pub struct CoefficientCommitment<C: Ciphersuite>(pub Element<C>);

impl<C> CoefficientCommitment<C>
where
	C: Ciphersuite,
{
	/// Create a new CoefficientCommitment.
	pub fn new(value: Element<C>) -> Self {
		Self(value)
	}

	/// returns serialized element
	pub fn serialize(&self) -> <C::Group as Group>::Serialization {
		<C::Group>::serialize(&self.0)
	}

	/// Creates a new commitment from a coefficient input
	pub fn deserialize(
		coefficient: <C::Group as Group>::Serialization,
	) -> Result<CoefficientCommitment<C>, Error> {
		Ok(Self::new(<C::Group as Group>::deserialize(&coefficient)?))
	}

	/// Returns inner element value
	pub fn value(&self) -> Element<C> {
		self.0
	}

	/// Verifies that a coefficient commitment is valid aka not zero or the base point
	pub fn is_valid(&self) -> bool {
		element_is_valid::<C>(&self.0)
	}
}

impl<C> Debug for CoefficientCommitment<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_tuple("CoefficientCommitment")
			.field(&hex::encode(self.serialize()))
			.finish()
	}
}

impl<C> TryFrom<ElementSerialization<C>> for CoefficientCommitment<C>
where
	C: Ciphersuite,
{
	type Error = Error;

	fn try_from(value: ElementSerialization<C>) -> Result<Self, Self::Error> {
		Self::deserialize(value.0)
	}
}

impl<C> From<CoefficientCommitment<C>> for ElementSerialization<C>
where
	C: Ciphersuite,
{
	fn from(value: CoefficientCommitment<C>) -> Self {
		Self(value.serialize())
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
pub struct VerifiableSecretSharingCommitment<C: Ciphersuite>(pub Vec<CoefficientCommitment<C>>);

impl<C> VerifiableSecretSharingCommitment<C>
where
	C: Ciphersuite,
{
	/// Create a new VerifiableSecretSharingCommitment.
	pub fn new(coefficients: Vec<CoefficientCommitment<C>>) -> Self {
		Self(coefficients)
	}

	/// Returns serialized coefficent commitments
	pub fn serialize(&self) -> Vec<<C::Group as Group>::Serialization> {
		self.0
			.iter()
			.map(|cc| <<C as Ciphersuite>::Group as Group>::serialize(&cc.0))
			.collect()
	}

	/// Returns VerifiableSecretSharingCommitment from a vector of serialized CoefficientCommitments
	pub fn deserialize(
		serialized_coefficient_commitments: Vec<<C::Group as Group>::Serialization>,
	) -> Result<Self, Error> {
		let mut coefficient_commitments = Vec::new();
		for cc in serialized_coefficient_commitments {
			coefficient_commitments.push(CoefficientCommitment::<C>::deserialize(cc)?);
		}

		Ok(Self::new(coefficient_commitments))
	}

	/// Get the VerifyingKey matching this commitment vector (which is the first
	/// element in the vector), or an error if the vector is empty.
	pub fn verifying_key(&self) -> Result<VerifyingKey<C>, Error> {
		Ok(VerifyingKey::new(self.0.first().ok_or(Error::MissingCommitment)?.0))
	}

	/// Returns the coefficient commitments.
	pub fn coefficients(&self) -> &[CoefficientCommitment<C>] {
		&self.0
	}

	/// Verifies that all coefficients are valid aka not zero or the base point
	pub fn is_valid(&self) -> bool {
		self.0.iter().all(|cc| cc.is_valid())
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
#[derive(Clone, Debug, Zeroize, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
#[serde(deny_unknown_fields)]
pub struct SecretShare<C: Ciphersuite> {
	/// Serialization header
	pub header: Header<C>,
	/// The participant identifier of this [`SecretShare`].
	#[zeroize(skip)]
	pub identifier: Identifier<C>,
	/// Secret Key.
	pub signing_share: SigningShare<C>,
	#[zeroize(skip)]
	/// The commitments to be distributed among signers.
	pub commitment: VerifiableSecretSharingCommitment<C>,
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
	/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#appendix-C.2-4
	pub fn verify(&self) -> Result<(VerifyingShare<C>, VerifyingKey<C>), Error> {
		let f_result = <C::Group>::generator() * self.signing_share.0;
		let result = evaluate_vss(self.identifier, &self.commitment);

		if !(f_result == result) {
			return Err(Error::InvalidSecretShare);
		}

		Ok((VerifyingShare(result), self.commitment.verifying_key()?))
	}

	/// Checks that all values are valid aka no zero or base point values
	pub fn is_valid(&self) -> bool {
		self.signing_share.is_valid() && self.commitment.is_valid() && self.commitment.is_valid()
	}
}

#[cfg(feature = "serialization")]
impl<C> SecretShare<C>
where
	C: Ciphersuite,
{
	/// Serialize the struct into a Vec.
	pub fn serialize(&self) -> Result<Vec<u8>, Error> {
		Serialize::serialize(&self)
	}

	/// Deserialize the struct from a slice of bytes.
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error> {
		Deserialize::deserialize(bytes)
	}
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
#[serde(deny_unknown_fields)]
pub struct PublicKeyPackage<C: Ciphersuite> {
	/// Serialization header
	pub header: Header<C>,
	/// The verifying shares for all participants. Used to validate signature
	/// shares they generate.
	pub verifying_shares: BTreeMap<Identifier<C>, VerifyingShare<C>>,
	/// The joint public key for the entire group.
	pub verifying_key: VerifyingKey<C>,
}

impl<C> PublicKeyPackage<C>
where
	C: Ciphersuite,
{
	/// Serialize the struct into a Vec.
	pub fn serialize(&self) -> Result<Vec<u8>, Error> {
		Serialize::serialize(&self)
	}

	/// Deserialize the struct from a slice of bytes.
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error> {
		Deserialize::deserialize(bytes)
	}
}

// Default byte-oriented serialization for structs that need to be communicated.
//
// Note that we still manually implement these methods in each applicable type,
// instead of making these traits `pub` and asking users to import the traits.
// The reason is that ciphersuite traits would need to re-export these traits,
// parametrized with the ciphersuite, but trait aliases are not currently
// supported: <https://github.com/rust-lang/rust/issues/41517>

#[cfg(feature = "serialization")]
pub trait Serialize<C: Ciphersuite> {
	/// Serialize the struct into a Vec.
	fn serialize(&self) -> Result<Vec<u8>, Error>;
}

#[cfg(feature = "serialization")]
pub trait Deserialize<C: Ciphersuite> {
	/// Deserialize the struct from a slice of bytes.
	fn deserialize(bytes: &[u8]) -> Result<Self, Error>
	where
		Self: core::marker::Sized;
}

#[cfg(feature = "serialization")]
impl<T: serde::Serialize, C: Ciphersuite> Serialize<C> for T {
	fn serialize(&self) -> Result<Vec<u8>, Error> {
		postcard::to_allocvec(self).map_err(|_| Error::SerializationError)
	}
}

#[cfg(feature = "serialization")]
impl<T: for<'de> serde::Deserialize<'de>, C: Ciphersuite> Deserialize<C> for T {
	fn deserialize(bytes: &[u8]) -> Result<Self, Error> {
		postcard::from_bytes(bytes).map_err(|_| Error::DeserializationError)
	}
}

/// Evaluate the polynomial with the given coefficients (constant term first)
/// at the point x=identifier using Horner's method.
///
/// Implements [`polynomial_evaluate`] from the spec.
///
/// [`polynomial_evaluate`]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#name-evaluation-of-a-polynomial
pub fn evaluate_polynomial<C: Ciphersuite>(
	identifier: Identifier<C>,
	coefficients: &[Scalar<C>],
) -> Scalar<C> {
	let mut value = <<C::Group as Group>::Field>::zero();

	let ell_scalar = identifier;
	for coeff in coefficients.iter().skip(1).rev() {
		value = value + *coeff;
		value *= ell_scalar;
	}
	value = value + *coefficients.first().expect("coefficients must have at least one element");
	value
}

/// Evaluates the right-hand side of the VSS verification equation, namely
/// ∏^{t−1}_{k=0} φ^{i^k mod q}_{ℓk} (multiplicative notation) using
/// `identifier` as `i` and the `commitment` as the commitment vector φ_ℓ.
///
/// This is also used in Round 2, Step 4 of the DKG.
pub fn evaluate_vss<C: Ciphersuite>(
	identifier: Identifier<C>,
	commitment: &VerifiableSecretSharingCommitment<C>,
) -> Element<C> {
	let i = identifier;

	let (_, result) = commitment.0.iter().fold(
		(<<C::Group as Group>::Field>::one(), <C::Group>::identity()),
		|(i_to_the_k, sum_so_far), comm_k| (i * i_to_the_k, sum_so_far + comm_k.0 * i_to_the_k),
	);
	result
}
