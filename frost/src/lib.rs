// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.

//! # FROST no_std primitives
//!
//! A no_std copy of FROST primitives from https://github.com/LIT-Protocol/frost.
//! Needed in order to properly verify Schnorr threshold signatures based on FROST
//! protocol from this library, since the original library was not no_std compatible.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]
extern crate alloc;

pub mod challenge;
pub mod const_crc32;
pub mod error;
pub mod identifier;
pub mod keygen;
pub mod keys;
pub mod round1;
pub mod scalar_mul;
pub mod serialization;
pub mod signature;
pub mod signing_key;
pub mod traits;
pub mod util;
pub mod verifying_key;

use alloc::collections::{BTreeMap, BTreeSet};
use core::{fmt::Debug, marker::PhantomData};
use scalar_mul::VartimeMultiscalarMul;
use sp_std::{vec, vec::Vec};
use zeroize::Zeroize;

use challenge::Challenge;
use error::Error;
use identifier::Identifier;
use serde::{Deserialize, Serialize};
use traits::{Ciphersuite, Element, Field, Group, Scalar};
use verifying_key::VerifyingKey;

#[cfg(feature = "std")]
use hex::FromHex;

#[cfg(feature = "std")]
use rand_core::{CryptoRng, RngCore};

/// Generates a random nonzero scalar.
///
/// It assumes that the Scalar Eq/PartialEq implementation is constant-time.
#[cfg(feature = "std")]
pub fn random_nonzero<C: Ciphersuite, R: RngCore + CryptoRng>(rng: &mut R) -> Scalar<C> {
	loop {
		let scalar = <<C::Group as Group>::Field>::random(rng);

		if scalar != <<C::Group as Group>::Field>::zero() {
			return scalar
		}
	}
}

#[derive(Copy, Clone, Debug, Zeroize, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header<C: Ciphersuite> {
	/// Format version
	pub version: u8,
	/// Ciphersuite ID
	pub ciphersuite: (),
	#[serde(skip)]
	pub phantom: PhantomData<C>,
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
	pub fn serialize(&self) -> <<C::Group as Group>::Field as Field>::Serialization {
		<<C::Group as Group>::Field>::serialize(&self.0)
	}
}

impl<C> Debug for BindingFactor<C>
where
	C: Ciphersuite,
{
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		f.debug_tuple("BindingFactor").field(&hex::encode(self.serialize())).finish()
	}
}

/// A list of binding factors and their associated identifiers.
#[derive(Clone)]
pub struct BindingFactorList<C: Ciphersuite>(BTreeMap<Identifier<C>, BindingFactor<C>>);

impl<C> BindingFactorList<C>
where
	C: Ciphersuite,
{
	/// Create a new [`BindingFactorList`] from a map of identifiers to binding factors.
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
/// [`compute_binding_factors`]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-4.4
pub fn compute_binding_factor_list<C>(
	signing_package: &SigningPackage<C>,
	verifying_key: &VerifyingKey<C>,
	additional_prefix: &[u8],
) -> BindingFactorList<C>
where
	C: Ciphersuite,
{
	let preimages = signing_package.binding_factor_preimages(verifying_key, additional_prefix);

	BindingFactorList(
		preimages
			.iter()
			.map(|(identifier, preimage)| {
				let binding_factor = C::H1(preimage);
				(*identifier, BindingFactor(binding_factor))
			})
			.collect(),
	)
}

#[cfg(any(test, feature = "std"))]
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
) -> Result<Scalar<C>, Error> {
	if x_set.is_empty() {
		return Err(Error::IncorrectNumberOfIdentifiers)
	}
	let mut num = <<C::Group as Group>::Field>::one();
	let mut den = <<C::Group as Group>::Field>::one();

	let mut x_i_found = false;

	for x_j in x_set.iter() {
		if x_i == *x_j {
			x_i_found = true;
			continue
		}

		if let Some(x) = x {
			num *= x - *x_j;
			den *= x_i - *x_j;
		} else {
			// Both signs inverted just to avoid requiring Neg (-*xj)
			num *= *x_j;
			den *= *x_j - x_i;
		}
	}
	if !x_i_found {
		return Err(Error::UnknownIdentifier)
	}

	Ok(num * <<C::Group as Group>::Field>::invert(&den).map_err(|_| Error::DuplicatedIdentifier)?)
}

/// Generates the lagrange coefficient for the i'th participant (for `signer_id`).
///
/// Implements [`derive_interpolating_value()`] from the spec.
///
/// [`derive_interpolating_value()`]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#name-polynomials
pub fn derive_interpolating_value<C: Ciphersuite>(
	signer_id: &Identifier<C>,
	signing_package: &SigningPackage<C>,
) -> Result<Scalar<C>, Error> {
	compute_lagrange_coefficient(
		&signing_package.signing_commitments.keys().cloned().collect(),
		None,
		*signer_id,
	)
}

// Generated by the coordinator of the signing operation and distributed to
/// each signing party
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(bound = "C: Ciphersuite")]
#[serde(deny_unknown_fields)]
pub struct SigningPackage<C: Ciphersuite> {
	/// Serialization header
	pub header: Header<C>,
	/// The set of commitments participants published in the first round of the
	/// protocol.
	pub signing_commitments: BTreeMap<Identifier<C>, round1::SigningCommitments<C>>,
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
	pub message: Vec<u8>,
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
	pub fn binding_factor_preimages(
		&self,
		verifying_key: &VerifyingKey<C>,
		additional_prefix: &[u8],
	) -> Vec<(Identifier<C>, Vec<u8>)> {
		let mut binding_factor_input_prefix = vec![];

		// The length of a serialized verifying key of the same cipersuite does
		// not change between runs of the protocol, so we don't need to hash to
		// get a fixed length.
		binding_factor_input_prefix.extend_from_slice(verifying_key.serialize().as_ref());

		// The message is hashed with H4 to force the variable-length message
		// into a fixed-length byte string, same for hashing the variable-sized
		// (between runs of the protocol) set of group commitments, but with H5.
		binding_factor_input_prefix.extend_from_slice(C::H4(self.message.as_slice()).as_ref());
		binding_factor_input_prefix.extend_from_slice(
			C::H5(&round1::encode_group_commitments(&self.signing_commitments)[..]).as_ref(),
		);
		binding_factor_input_prefix.extend_from_slice(additional_prefix);

		self.signing_commitments
			.keys()
			.map(|identifier| {
				let mut binding_factor_input = vec![];

				binding_factor_input.extend_from_slice(&binding_factor_input_prefix);
				binding_factor_input.extend_from_slice(identifier.serialize().as_ref());
				(*identifier, binding_factor_input)
			})
			.collect()
	}

	/// Check if the signing package is valid.
	pub fn is_valid(&self) -> bool {
		self.signing_commitments.iter().all(|(i, c)| i.is_valid() && c.is_valid())
	}
}

impl<C> SigningPackage<C>
where
	C: Ciphersuite,
{
	/// Serialize the struct into a Vec.
	pub fn serialize(&self) -> Result<Vec<u8>, Error> {
		serialization::Serialize::serialize(&self)
	}

	/// Deserialize the struct from a slice of bytes.
	pub fn deserialize(bytes: &[u8]) -> Result<Self, Error> {
		serialization::Deserialize::deserialize(bytes)
	}
}

/// The product of all signers' individual commitments, published as part of the
/// final signature.
#[derive(Clone, PartialEq, Eq)]
pub struct GroupCommitment<C: Ciphersuite>(pub Element<C>);

impl<C> GroupCommitment<C>
where
	C: Ciphersuite,
{
	/// Return the underlying element.
	pub fn to_element(self) -> <C::Group as Group>::Element {
		self.0
	}
}

/// Generates the group commitment which is published as part of the joint
/// Schnorr signature.
///
/// Implements [`compute_group_commitment`] from the spec.
///
/// [`compute_group_commitment`]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-4.5
pub fn compute_group_commitment<C>(
	signing_package: &SigningPackage<C>,
	binding_factor_list: &BindingFactorList<C>,
) -> Result<GroupCommitment<C>, Error>
where
	C: Ciphersuite,
{
	let identity = <C::Group as Group>::identity();

	let mut group_commitment = <C::Group as Group>::identity();

	// Number of signing participants we are iterating over.
	let n = signing_package.signing_commitments.len();

	let mut binding_scalars = Vec::with_capacity(n);

	let mut binding_elements = Vec::with_capacity(n);

	for (commitment_identifier, commitment) in &signing_package.signing_commitments {
		// The following check prevents a party from accidentally revealing their share.
		// Note that the '&&' operator would be sufficient.
		if identity == commitment.binding.0 || identity == commitment.hiding.0 {
			return Err(Error::IdentityCommitment)
		}

		let binding_factor =
			binding_factor_list.get(commitment_identifier).ok_or(Error::UnknownIdentifier)?;

		// Collect the binding commitments and their binding factors for one big
		// multiscalar multiplication at the end.
		binding_elements.push(commitment.binding.0);
		binding_scalars.push(binding_factor.0);

		group_commitment = group_commitment + commitment.hiding.0;
	}

	let accumulated_binding_commitment: Element<C> =
		VartimeMultiscalarMul::<C>::vartime_multiscalar_mul(binding_scalars, binding_elements);

	group_commitment = group_commitment + accumulated_binding_commitment;

	Ok(GroupCommitment(group_commitment))
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

/// Generates the challenge for the proof of knowledge to a secret for the DKG.
pub fn pok_challenge<C>(
	identifier: Identifier<C>,
	verifying_key: &VerifyingKey<C>,
	R: &Element<C>,
) -> Option<Challenge<C>>
where
	C: Ciphersuite,
{
	let mut preimage = vec![];

	preimage.extend_from_slice(identifier.serialize().as_ref());
	preimage.extend_from_slice(<C::Group>::serialize(&verifying_key.element).as_ref());
	preimage.extend_from_slice(<C::Group>::serialize(R).as_ref());

	Some(Challenge(C::HDKG(&preimage[..])?))
}
