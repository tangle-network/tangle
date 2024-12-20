//! FROST Round 2 functionality and types, for signature share generation

use core::fmt::{self, Debug};

use crate as frost;
use crate::{round1, Challenge, Ciphersuite, Error, Field, Group, *};

/// A participant's signature share, which the coordinator will aggregate with all other signer's
/// shares into the joint signature.
#[derive(Clone, Copy, Eq, PartialEq, Getters)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "C: Ciphersuite"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct SignatureShare<C: Ciphersuite> {
	/// Serialization header
	#[getter(skip)]
	pub(crate) header: Header<C>,
	/// This participant's signature over the message.
	pub(crate) share: SerializableScalar<C>,
}

impl<C> SignatureShare<C>
where
	C: Ciphersuite,
{
	pub(crate) fn new(
		scalar: <<<C as Ciphersuite>::Group as Group>::Field as Field>::Scalar,
	) -> Self {
		Self { header: Header::default(), share: SerializableScalar(scalar) }
	}

	pub(crate) fn to_scalar(
		self,
	) -> <<<C as Ciphersuite>::Group as Group>::Field as Field>::Scalar {
		self.share.0
	}

	/// Deserialize [`SignatureShare`] from bytes
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error<C>> {
		Ok(Self { header: Header::default(), share: SerializableScalar::deserialize(bytes)? })
	}

	/// Serialize [`SignatureShare`] to bytes
	pub fn serialize(&self) -> Vec<u8> {
		self.share.serialize()
	}

	/// Tests if a signature share issued by a participant is valid before
	/// aggregating it into a final joint signature to publish.
	///
	/// This is the final step of [`verify_signature_share`] from the spec.
	///
	/// [`verify_signature_share`]: https://datatracker.ietf.org/doc/html/rfc9591#name-signature-share-aggregation
	#[cfg(any(feature = "cheater-detection", feature = "internals"))]
	#[cfg_attr(feature = "internals", visibility::make(pub))]
	#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
	pub(crate) fn verify(
		&self,
		identifier: Identifier<C>,
		group_commitment_share: &round1::GroupCommitmentShare<C>,
		verifying_share: &frost::keys::VerifyingShare<C>,
		lambda_i: Scalar<C>,
		challenge: &Challenge<C>,
	) -> Result<(), Error<C>> {
		if (<C::Group>::generator() * self.to_scalar())
			!= (group_commitment_share.to_element()
				+ (verifying_share.to_element() * challenge.0 * lambda_i))
		{
			return Err(Error::InvalidSignatureShare { culprit: identifier });
		}

		Ok(())
	}
}

impl<C> Debug for SignatureShare<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("SignatureShare")
			.field("share", &hex::encode(self.serialize()))
			.finish()
	}
}

/// Compute the signature share for a signing operation.
#[cfg_attr(feature = "internals", visibility::make(pub))]
#[cfg_attr(docsrs, doc(cfg(feature = "internals")))]
pub(super) fn compute_signature_share<C: Ciphersuite>(
	signer_nonces: &round1::SigningNonces<C>,
	binding_factor: BindingFactor<C>,
	lambda_i: <<<C as Ciphersuite>::Group as Group>::Field as Field>::Scalar,
	key_package: &keys::KeyPackage<C>,
	challenge: Challenge<C>,
) -> SignatureShare<C> {
	let z_share: <<C::Group as Group>::Field as Field>::Scalar = signer_nonces.hiding.to_scalar()
		+ (signer_nonces.binding.to_scalar() * binding_factor.0)
		+ (lambda_i * key_package.signing_share.to_scalar() * challenge.to_scalar());

	SignatureShare::<C>::new(z_share)
}

/// Performed once by each participant selected for the signing operation.
///
/// Implements [`sign`] from the spec.
///
/// Receives the message to be signed and a set of signing commitments and a set
/// of randomizing commitments to be used in that signing operation, including
/// that for this participant.
///
/// Assumes the participant has already determined which nonce corresponds with
/// the commitment that was assigned by the coordinator in the SigningPackage.
///
/// [`sign`]: https://datatracker.ietf.org/doc/html/rfc9591#name-round-two-signature-share-g
pub fn sign<C: Ciphersuite>(
	signing_package: &SigningPackage<C>,
	signer_nonces: &round1::SigningNonces<C>,
	key_package: &frost::keys::KeyPackage<C>,
) -> Result<SignatureShare<C>, Error<C>> {
	if signing_package.signing_commitments().len() < key_package.min_signers as usize {
		return Err(Error::IncorrectNumberOfCommitments);
	}

	// Validate the signer's commitment is present in the signing package
	let commitment = signing_package
		.signing_commitments
		.get(&key_package.identifier)
		.ok_or(Error::MissingCommitment)?;

	// Validate if the signer's commitment exists
	if &signer_nonces.commitments != commitment {
		return Err(Error::IncorrectCommitment);
	}

	let (signing_package, signer_nonces, key_package) =
		<C>::pre_sign(signing_package, signer_nonces, key_package)?;

	// Encodes the signing commitment list produced in round one as part of generating
	// [`BindingFactor`], the binding factor.
	let binding_factor_list: BindingFactorList<C> =
		compute_binding_factor_list(&signing_package, &key_package.verifying_key, &[])?;
	let binding_factor: frost::BindingFactor<C> = binding_factor_list
		.get(&key_package.identifier)
		.ok_or(Error::UnknownIdentifier)?
		.clone();

	// Compute the group commitment from signing commitments produced in round one.
	let group_commitment = compute_group_commitment(&signing_package, &binding_factor_list)?;

	// Compute Lagrange coefficient.
	let lambda_i = frost::derive_interpolating_value(key_package.identifier(), &signing_package)?;

	// Compute the per-message challenge.
	let challenge =
		<C>::challenge(&group_commitment.0, &key_package.verifying_key, signing_package.message())?;

	// Compute the Schnorr signature share.
	let signature_share = <C>::compute_signature_share(
		&group_commitment,
		&signer_nonces,
		binding_factor,
		lambda_i,
		&key_package,
		challenge,
	);

	Ok(signature_share)
}
