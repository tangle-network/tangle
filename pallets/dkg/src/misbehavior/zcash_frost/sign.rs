use crate::misbehavior::zcash_frost::convert_error;
use alloc::collections::BTreeMap;
use core::fmt::Debug;
use frame_support::{ensure, pallet_prelude::DispatchResult};
use frost_core::{
	compute_binding_factor_list, compute_group_commitment, derive_interpolating_value,
	identifier::Identifier,
	keys::VerifyingShare,
	round1::SigningCommitments,
	signature::{Signature, SignatureShare},
	traits::{Ciphersuite, Field, Group},
	verifying_key::VerifyingKey,
	BindingFactorList, SigningPackage,
};
use frost_ed25519::Ed25519Sha512;
use frost_ed448::Ed448Shake256;
use frost_p256::P256Sha256;
use frost_p384::P384Sha384;
use frost_ristretto255::Ristretto255Sha512;
use frost_secp256k1::Secp256K1Sha256;
use sp_std::vec::Vec;
use tangle_primitives::{
	misbehavior::{MisbehaviorSubmission, SignedRoundMessage},
	roles::ThresholdSignatureRoleType,
};

use crate::{Config, Error, Pallet};

/// Message from round 1
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, udigest::Digestable)]
#[serde(bound = "")]
#[udigest(bound = "")]
#[udigest(tag = "zcash.frost.sign.threshold.round1")]
pub struct MsgRound1 {
	pub msg: Vec<u8>,
}

/// Message from round 2
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, udigest::Digestable)]
#[serde(bound = "")]
#[udigest(bound = "")]
#[udigest(tag = "zcash.frost.sign.threshold.round2")]
pub struct MsgRound2 {
	pub msg: Vec<u8>,
}

pub fn verify_invalid_signature_share<T: Config, C: Ciphersuite>(
	sender: u16,
	round1_msgs: &[MsgRound1],
	round2_msgs: &[MsgRound2],
	offender_pubkey: &[u8],
	group_pubkey: &[u8],
	msg_to_sign: &[u8],
) -> DispatchResult {
	// Identifier in FROST protocol who is the offender
	let offender_identifier: Identifier<C> =
		Identifier::try_from(sender).map_err(|_| Error::<T>::InvalidIdentifierDeserialization)?;

	// The verifying key of the group (the group's joint public key)
	let ser = <C::Group as Group>::Serialization::try_from(group_pubkey.to_vec())
		.map_err(|_| Error::<T>::InvalidFrostMessageDeserialization)?;
	let verifying_key = VerifyingKey::deserialize(ser)
		.map_err(|_| Error::<T>::InvalidFrostMessageDeserialization)?;

	// Deserialize the round 1 signing commitments. Assumes the round 1 messages are in order.
	// TODO: Check if the signed messages should be parsed without assuming order.
	let round1_signing_commitments: BTreeMap<Identifier<C>, SigningCommitments<C>> = round1_msgs
		.iter()
		.enumerate()
		.map(|(party_inx, msg)| {
			let participant_identifier = Identifier::<C>::try_from((party_inx + 1) as u16)
				.expect("Failed to convert party index to identifier");
			let msg = SigningCommitments::<C>::deserialize(&msg.msg)
				.unwrap_or_else(|_| panic!("Failed to deserialize round 1 signing commitments"));
			(participant_identifier, msg)
		})
		.collect();

	// Create the signing package from the round 1 signing commitments and the message to sign.
	let signing_package = SigningPackage::<C>::new(round1_signing_commitments.clone(), msg_to_sign);

	// Encodes the signing commitment list produced in round one as part of generating
	// [`BindingFactor`], the binding factor.
	let binding_factor_list: BindingFactorList<C> =
		compute_binding_factor_list(&signing_package, &verifying_key, &[]);

	// Compute the group commitment from signing commitments produced in round one.
	let group_commitment = compute_group_commitment(&signing_package, &binding_factor_list)
		.map_err(convert_error::<T>)?;

	// TODO: Check if the signed messages should be parsed without assuming order.
	let signature_shares: BTreeMap<Identifier<C>, SignatureShare<C>> = round2_msgs
		.iter()
		.enumerate()
		.map(|(party_inx, msg)| {
			let participant_identifier = Identifier::<C>::try_from((party_inx + 1) as u16)
				.expect("Failed to convert party index to identifier");
			let ser =
				<<C::Group as Group>::Field as Field>::Serialization::try_from(msg.msg.clone())
					.unwrap_or_else(|_| panic!("Failed to deserialize round 2 signature share"));
			let sig_share = SignatureShare::<C>::deserialize(ser)
				.unwrap_or_else(|_| panic!("Failed to deserialize round 2 signature share"));
			(participant_identifier, sig_share)
		})
		.collect();

	// The verifying share of the offender.
	let ser = <C::Group as Group>::Serialization::try_from(offender_pubkey.to_vec())
		.map_err(|_| Error::<T>::InvalidFrostMessageDeserialization)?;
	let verifying_share = VerifyingShare::deserialize(ser)
		.map_err(|_| Error::<T>::InvalidFrostMessageDeserialization)?;

	// The aggregation of the signature shares by summing them up, resulting in
	// a plain Schnorr signature.
	//
	// Implements [`aggregate`] from the spec.
	//
	// [`aggregate`]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-5.3
	let mut z = <<C::Group as Group>::Field>::zero();

	for signature_share in signature_shares.values() {
		z = z + signature_share.share;
	}

	let signature = Signature { R: group_commitment.0, z };

	// Verify the aggregate signature
	let verification_result = verifying_key.verify(&signing_package.message, &signature);

	ensure!(verification_result.is_ok(), Error::<T>::ValidFrostSignature);

	// Compute the per-message challenge.
	let challenge = frost_core::challenge::<C>(&group_commitment.0, &verifying_key, msg_to_sign);

	// Compute Lagrange coefficient.
	let lambda_i = derive_interpolating_value(&offender_identifier, &signing_package)
		.map_err(convert_error::<T>)?;

	let binding_factor = binding_factor_list
		.get(&offender_identifier)
		.ok_or(Error::<T>::UnknownIdentifier)?;

	let R_share = round1_signing_commitments
		.get(&offender_identifier)
		.ok_or(Error::<T>::InvalidParticipantPublicKey)?
		.to_group_commitment_share(binding_factor);

	let offending_signature_share: &SignatureShare<C> = signature_shares
		.get(&offender_identifier)
		.ok_or(Error::<T>::UnknownIdentifier)?;

	// Verify that the offending signature share is invalid
	if offending_signature_share
		.verify(offender_identifier, &R_share, &verifying_share, lambda_i, &challenge)
		.is_ok()
	{
		Err(Error::<T>::ValidFrostSignatureShare)?
	}

	Ok(())
}

pub fn invalid_signature_share<T: Config>(
	role: ThresholdSignatureRoleType,
	data: &MisbehaviorSubmission,
	participants: &[[u8; 33]],
	round1: &[SignedRoundMessage],
	round2: &[SignedRoundMessage],
	group_pubkey: &[u8],
	msg_to_sign: &[u8],
) -> DispatchResult {
	let offender = data.offender;
	let index = participants
		.iter()
		.position(|&p| p == offender)
		.ok_or(Error::<T>::UnknownIdentifier)?;
	Pallet::<T>::ensure_signed_by_offender(&round1[index], data.offender)?;
	Pallet::<T>::ensure_signed_by_offender(&round2[index], data.offender)?;

	let round1_msgs: Vec<MsgRound1> = round1
		.iter()
		.map(|msg| {
			postcard::from_bytes::<MsgRound1>(&msg.message)
				.map_err(|_| Error::<T>::MalformedRoundMessage)
		})
		.collect::<Result<Vec<MsgRound1>, Error<T>>>()?;

	let round2_msgs: Vec<MsgRound2> = round2
		.iter()
		.map(|msg| {
			postcard::from_bytes::<MsgRound2>(&msg.message)
				.map_err(|_| Error::<T>::MalformedRoundMessage)
		})
		.collect::<Result<Vec<MsgRound2>, Error<T>>>()?;

	match role {
		ThresholdSignatureRoleType::ZcashFrostP256 => {
			verify_invalid_signature_share::<T, P256Sha256>(
				index as u16,
				&round1_msgs,
				&round2_msgs,
				&offender,
				group_pubkey,
				msg_to_sign,
			)?
		},
		ThresholdSignatureRoleType::ZcashFrostP384 => {
			verify_invalid_signature_share::<T, P384Sha384>(
				index as u16,
				&round1_msgs,
				&round2_msgs,
				&offender,
				group_pubkey,
				msg_to_sign,
			)?
		},
		ThresholdSignatureRoleType::ZcashFrostSecp256k1 => {
			verify_invalid_signature_share::<T, Secp256K1Sha256>(
				index as u16,
				&round1_msgs,
				&round2_msgs,
				&offender,
				group_pubkey,
				msg_to_sign,
			)?
		},
		ThresholdSignatureRoleType::ZcashFrostRistretto255 => {
			verify_invalid_signature_share::<T, Ristretto255Sha512>(
				index as u16,
				&round1_msgs,
				&round2_msgs,
				&offender,
				group_pubkey,
				msg_to_sign,
			)?
		},
		ThresholdSignatureRoleType::ZcashFrostEd25519 => {
			verify_invalid_signature_share::<T, Ed25519Sha512>(
				index as u16,
				&round1_msgs,
				&round2_msgs,
				&offender,
				group_pubkey,
				msg_to_sign,
			)?
		},
		ThresholdSignatureRoleType::ZcashFrostEd448 => {
			verify_invalid_signature_share::<T, Ed448Shake256>(
				index as u16,
				&round1_msgs,
				&round2_msgs,
				&offender,
				group_pubkey,
				msg_to_sign,
			)?
		},
		_ => Err(Error::<T>::InvalidFrostSignatureScheme)?,
	};

	// Slash the offender!
	// TODO: add slashing logic
	Ok(())
}
