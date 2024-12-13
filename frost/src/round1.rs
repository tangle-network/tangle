use core::fmt::Debug;

use crate::{
	error::Error,
	identifier::Identifier,
	serialization::{Deserialize, ElementSerialization, Serialize},
	traits::{Ciphersuite, Element, Group},
	util::element_is_valid,
	BindingFactor, Header,
};
use alloc::collections::BTreeMap;
use sp_std::{vec, vec::Vec};

/// A group element that is a commitment to a signing nonce share.
#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
#[serde(try_from = "ElementSerialization<C>")]
#[serde(into = "ElementSerialization<C>")]
pub struct NonceCommitment<C: Ciphersuite>(pub(super) Element<C>);

impl<C> NonceCommitment<C>
where
	C: Ciphersuite,
{
	/// Deserialize [`NonceCommitment`] from bytes
	pub fn deserialize(bytes: <C::Group as Group>::Serialization) -> Result<Self, Error> {
		<C::Group>::deserialize(&bytes)
			.map(|element| Self(element))
			.map_err(|e| e.into())
	}

	/// Serialize [`NonceCommitment`] to bytes
	pub fn serialize(&self) -> <C::Group as Group>::Serialization {
		<C::Group>::serialize(&self.0)
	}

	/// Checks if the commitment is valid.
	pub fn is_valid(&self) -> bool {
		element_is_valid::<C>(&self.0)
	}
}

impl<C> TryFrom<ElementSerialization<C>> for NonceCommitment<C>
where
	C: Ciphersuite,
{
	type Error = Error;

	fn try_from(value: ElementSerialization<C>) -> Result<Self, Self::Error> {
		Self::deserialize(value.0)
	}
}

impl<C> From<NonceCommitment<C>> for ElementSerialization<C>
where
	C: Ciphersuite,
{
	fn from(value: NonceCommitment<C>) -> Self {
		Self(value.serialize())
	}
}

impl<C> Debug for NonceCommitment<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		f.debug_tuple("NonceCommitment").field(&hex::encode(self.serialize())).finish()
	}
}

/// Published by each participant in the first round of the signing protocol.
///
/// This step can be batched if desired by the implementation. Each
/// SigningCommitment can be used for exactly *one* signature.
#[derive(Copy, Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
#[serde(deny_unknown_fields)]
pub struct SigningCommitments<C: Ciphersuite> {
	/// Serialization header
	pub(crate) header: Header<C>,
	/// Commitment to the hiding [`Nonce`].
	pub(crate) hiding: NonceCommitment<C>,
	/// Commitment to the binding [`Nonce`].
	pub(crate) binding: NonceCommitment<C>,
}

impl<C> SigningCommitments<C>
where
	C: Ciphersuite,
{
	/// Create new SigningCommitments
	pub fn new(hiding: NonceCommitment<C>, binding: NonceCommitment<C>) -> Self {
		Self { header: Header::default(), hiding, binding }
	}

	/// Computes the [signature commitment share] from these round one signing commitments.
	///
	/// [signature commitment share]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#name-signature-share-verificatio
	pub fn to_group_commitment_share(
		self,
		binding_factor: &BindingFactor<C>,
	) -> GroupCommitmentShare<C> {
		GroupCommitmentShare::<C>(self.hiding.0 + (self.binding.0 * binding_factor.0))
	}

	/// Checks if the commitments are valid.
	pub fn is_valid(&self) -> bool {
		element_is_valid::<C>(&self.hiding.0)
			&& element_is_valid::<C>(&self.binding.0)
			&& self.hiding.0 != self.binding.0
	}
}

impl<C> SigningCommitments<C>
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

/// One signer's share of the group commitment, derived from their individual signing commitments
/// and the binding factor _rho_.
#[derive(Clone, Copy, PartialEq)]
pub struct GroupCommitmentShare<C: Ciphersuite>(pub(super) Element<C>);

/// Encode the list of group signing commitments.
///
/// Implements [`encode_group_commitment_list()`] from the spec.
///
/// `signing_commitments` must contain the sorted map of participants
/// identifiers to the signing commitments they issued.
///
/// Returns a byte string containing the serialized representation of the
/// commitment list.
///
/// [`encode_group_commitment_list()`]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#name-list-operations
pub(super) fn encode_group_commitments<C: Ciphersuite>(
	signing_commitments: &BTreeMap<Identifier<C>, SigningCommitments<C>>,
) -> Vec<u8> {
	let mut bytes = vec![];

	for (item_identifier, item) in signing_commitments {
		bytes.extend_from_slice(item_identifier.serialize().as_ref());
		bytes.extend_from_slice(<C::Group>::serialize(&item.hiding.0).as_ref());
		bytes.extend_from_slice(<C::Group>::serialize(&item.binding.0).as_ref());
	}

	bytes
}
