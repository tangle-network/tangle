// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
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
use super::*;
use dfns_cggmp21::{
	keygen,
	security_level::{SecurityLevel, SecurityLevel128},
};
use frame_support::{ensure, pallet_prelude::DispatchResult};
use generic_ec::{Point, Scalar};
use generic_ec_zkp::{polynomial::Polynomial, schnorr_pok};
use sha2::Digest;
use sp_io::{hashing::keccak_256, EcdsaVerifyError};
use tangle_primitives::{
	misbehavior::{
		dfns_cggmp21::{
			DfnsCGGMP21Justification, KeygenAborted, SignedRoundMessage, SigningAborted,
		},
		DKGTSSJustification, MisbehaviorJustification, MisbehaviorSubmission,
	},
	roles::{RoleType, ThresholdSignatureRoleType},
};

use dfns_cggmp21::{generic_ec, supported_curves::Secp256k1};
use types::{DefaultDigest, Tag};

/// Expected signature length
pub const SIGNATURE_LENGTH: usize = 65;
/// Expected key length for ecdsa
const ECDSA_KEY_LENGTH: usize = 33;
/// Expected key length for sr25519
const SCHNORR_KEY_LENGTH: usize = 32;

impl<T: Config> Pallet<T> {
	/// Verifies a given a misbehavior report and dispatches to specific verification logic
	/// based on the round.
	///
	/// # Arguments
	///
	/// * `data` - The misbehavior report, which could be of different types depending on the
	/// round.
	///
	/// # Returns
	///
	/// Returns a `DispatchResult` indicating whether the verification was successful or encountered
	/// an error.
	pub fn verify(data: MisbehaviorSubmission) -> DispatchResult {
		ensure!(
			matches!(
				data.role_type,
				RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1) |
					RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256r1) |
					RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Stark),
			),
			Error::<T>::InvalidRoleType
		);

		ensure!(
			matches!(
				data.justification,
				MisbehaviorJustification::DKGTSS(DKGTSSJustification::DfnsCGGMP21(_))
			),
			Error::<T>::InvalidJustification,
		);

		match data.justification {
			MisbehaviorJustification::DKGTSS(DKGTSSJustification::DfnsCGGMP21(
				ref justification,
			)) => Self::verify_misbehavior(&data, justification),
			MisbehaviorJustification::ZkSaaS(_) => Err(Error::<T>::InvalidJustification.into()),
		}
	}

	/// Verifies a given a misbehavior justification and dispatches to specific verification logic
	pub fn verify_misbehavior(
		data: &MisbehaviorSubmission,
		justification: &DfnsCGGMP21Justification,
	) -> DispatchResult {
		match justification {
			DfnsCGGMP21Justification::Keygen { participants, t, reason } =>
				Self::verify_keygen_misbehavior(data, participants, *t, reason),
			DfnsCGGMP21Justification::Signing { participants, t, reason } =>
				Self::verify_signing_misbehavior(data, participants, *t, reason),
		}
	}

	/// given a keygen misbehavior justification, verify the misbehavior and return a dispatch
	/// result
	pub fn verify_keygen_misbehavior(
		data: &MisbehaviorSubmission,
		participants: &[[u8; 33]],
		t: u16,
		reason: &KeygenAborted,
	) -> DispatchResult {
		match reason {
			KeygenAborted::InvalidDecommitment { round1, round2a } =>
				Self::verify_keygen_invalid_decommitment(data, round1, round2a),
			KeygenAborted::InvalidDataSize { round2a } =>
				Self::verify_keygen_invalid_data_size(data, t, round2a),
			KeygenAborted::FeldmanVerificationFailed { round2a, round2b } =>
				Self::verify_keygen_feldman(data, round2a, round2b),
			KeygenAborted::InvalidSchnorrProof { round2a, round3 } =>
				Self::verify_schnorr_proof(data, participants, round2a, round3),
		}
	}

	/// given a signing misbehavior justification, verify the misbehavior and return a dispatch
	/// result
	pub fn verify_signing_misbehavior(
		data: &MisbehaviorSubmission,
		participants: &[[u8; 33]],
		t: u16,
		reason: &SigningAborted,
	) -> DispatchResult {
		unimplemented!()
	}

	/// Given a Keygen Round1 and Round2a messages, verify the misbehavior and return the result.
	pub fn verify_keygen_invalid_decommitment(
		data: &MisbehaviorSubmission,
		round1: &SignedRoundMessage,
		round2a: &SignedRoundMessage,
	) -> DispatchResult {
		Self::ensure_signed_by_offender(round1, data.offender)?;
		Self::ensure_signed_by_offender(round2a, data.offender)?;
		ensure!(round1.sender == round2a.sender, Error::<T>::InvalidJustification);

		let job_id_bytes = data.job_id.to_be_bytes();
		let mix = keccak_256(b"dnfs-cggmp21-keygen");
		let eid_bytes = [&job_id_bytes[..], &mix[..]].concat();
		let tag = udigest::Tag::<DefaultDigest>::new_structured(Tag::Indexed {
			party_index: round1.sender,
			sid: &eid_bytes[..],
		});

		let round1_msg = bincode2::deserialize::<keygen::msg::threshold::MsgRound1<DefaultDigest>>(
			&round1.message,
		)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;

		let round2_msg = bincode2::deserialize::<
			keygen::msg::threshold::MsgRound2Broad<Secp256k1, SecurityLevel128>,
		>(&round2a.message)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;
		let hash_commit = tag.digest(round2_msg);

		ensure!(round1_msg.commitment != hash_commit, Error::<T>::ValidDecommitment);
		// Slash the offender!
		// TODO: add slashing logic
		Ok(())
	}

	/// Given a Keygen t and Round2a messages, verify the misbehavior and return the result.
	pub fn verify_keygen_invalid_data_size(
		data: &MisbehaviorSubmission,
		t: u16,
		round2a: &SignedRoundMessage,
	) -> DispatchResult {
		Self::ensure_signed_by_offender(round2a, data.offender)?;

		let round2a_msg = bincode2::deserialize::<
			keygen::msg::threshold::MsgRound2Broad<Secp256k1, SecurityLevel128>,
		>(&round2a.message)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;

		ensure!(round2a_msg.F.degree() + 1 != usize::from(t), Error::<T>::ValidDataSize);
		// Slash the offender!
		// TODO: add slashing logic
		Ok(())
	}

	/// Given a Keygen Round2a and Round2b messages, verify the misbehavior and return the result.
	pub fn verify_keygen_feldman(
		data: &MisbehaviorSubmission,
		round2a: &SignedRoundMessage,
		round2b: &SignedRoundMessage,
	) -> DispatchResult {
		Self::ensure_signed_by_offender(round2a, data.offender)?;
		Self::ensure_signed_by_offender(round2b, data.offender)?;
		ensure!(round2a.sender == round2b.sender, Error::<T>::InvalidJustification);
		let i = round2a.sender;

		let round2a_msg = bincode2::deserialize::<
			keygen::msg::threshold::MsgRound2Broad<Secp256k1, SecurityLevel128>,
		>(&round2a.message)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;

		let round2b_msg = bincode2::deserialize::<keygen::msg::threshold::MsgRound2Uni<Secp256k1>>(
			&round2b.message,
		)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;

		let lhs = round2a_msg.F.value::<_, generic_ec::Point<_>>(&Scalar::from(i + 1));
		let rhs = generic_ec::Point::generator() * round2b_msg.sigma;
		let feldman_verification = lhs != rhs;
		ensure!(feldman_verification, Error::<T>::ValidFeldmanVerification);
		// Slash the offender!
		// TODO: add slashing logic
		Ok(())
	}

	pub fn verify_schnorr_proof(
		data: &MisbehaviorSubmission,
		parties_including_offender: &[[u8; 33]],
		round2a: &[SignedRoundMessage],
		round3: &SignedRoundMessage,
	) -> DispatchResult {
		let i = round3.sender;
		let n = parties_including_offender.len() as u16;
		Self::ensure_signed_by_offender(round3, data.offender)?;
		ensure!(round2a.len() == usize::from(n), Error::<T>::InvalidJustification);
		round2a
			.iter()
			.zip(parties_including_offender)
			.try_for_each(|(r, p)| Self::ensure_signed_by(r, *p))?;

		let decomm = round2a.get(usize::from(i)).ok_or(Error::<T>::InvalidJustification)?;
		// double-check
		Self::ensure_signed_by_offender(decomm, data.offender)?;

		let job_id_bytes = data.job_id.to_be_bytes();
		let mix = keccak_256(b"dnfs-cggmp21-keygen");
		let eid_bytes = [&job_id_bytes[..], &mix[..]].concat();

		let round3_msg =
			bincode2::deserialize::<keygen::msg::threshold::MsgRound3<Secp256k1>>(&round3.message)
				.map_err(|_| Error::<T>::MalformedRoundMessage)?;

		let round2a_msgs = round2a
			.iter()
			.map(|r| {
				bincode2::deserialize::<
					keygen::msg::threshold::MsgRound2Broad<Secp256k1, SecurityLevel128>,
				>(&r.message)
				.map_err(|_| Error::<T>::MalformedRoundMessage)
			})
			.collect::<Result<Vec<_>, _>>()?;
		let round2a_msg =
			round2a_msgs.get(usize::from(i)).ok_or(Error::<T>::InvalidJustification)?;

		let rid = round2a_msgs
			.iter()
			.map(|d| &d.rid)
			.fold(<SecurityLevel128 as SecurityLevel>::Rid::default(), Self::xor_array);

		let polynomial_sum =
			round2a_msgs.iter().map(|d| &d.F).sum::<Polynomial<Point<Secp256k1>>>();

		let ys = (0..n)
			.map(|l| polynomial_sum.value(&Scalar::from(l + 1)))
			.collect::<Vec<Point<Secp256k1>>>();

		let challenge = {
			let hash = |d: DefaultDigest| {
				d.chain_update(&eid_bytes)
					.chain_update(i.to_be_bytes())
					.chain_update(rid.as_ref())
					.chain_update(ys[usize::from(i)].to_bytes(true)) // y_i
					.chain_update(round2a_msg.sch_commit.0.to_bytes(false)) // h
					.finalize()
			};
			let mut rng = paillier_zk::rng::HashRng::new(hash);
			Scalar::random(&mut rng)
		};
		let challenge = schnorr_pok::Challenge { nonce: challenge };

		let proof =
			round3_msg
				.sch_proof
				.verify(&round2a_msg.sch_commit, &challenge, &ys[usize::from(i)]);

		ensure!(proof.is_err(), Error::<T>::ValidSchnorrProof);

		// TODO: add slashing logic
		// Slash the offender!
		Ok(())
	}

	/// Given a [`SignedRoundMessage`] ensure that the message is signed by the given offender.
	pub fn ensure_signed_by_offender(
		signed_message: &SignedRoundMessage,
		offender: [u8; 33],
	) -> DispatchResult {
		Self::ensure_signed_by(signed_message, offender)
	}

	/// Given a [`SignedRoundMessage`] ensure that the message is signed by the given signer
	pub fn ensure_signed_by(
		signed_message: &SignedRoundMessage,
		expected_signer: [u8; 33],
	) -> DispatchResult {
		let final_message =
			[&signed_message.sender.to_be_bytes()[..], signed_message.message.as_slice()].concat();
		let signer = Self::recover_ecdsa_pub_key(&final_message, &signed_message.signature)
			.map_err(|_| Error::<T>::InvalidSignature)?;
		ensure!(signer == expected_signer, Error::<T>::NotSignedByOffender);
		Ok(())
	}
	/// Recovers the ECDSA public key from a given message and signature.
	///
	/// # Arguments
	///
	/// * `data` - The message for which the signature is being verified.
	/// * `signature` - The ECDSA signature to be verified.
	///
	/// # Returns
	///
	/// Returns a `Result` containing the recovered ECDSA public key as a `Vec<u8>` or an
	/// `EcdsaVerifyError` if verification fails.
	pub fn recover_ecdsa_pub_key(
		data: &[u8],
		signature: &[u8],
	) -> Result<[u8; 33], EcdsaVerifyError> {
		if signature.len() == SIGNATURE_LENGTH {
			let mut sig = [0u8; SIGNATURE_LENGTH];
			sig[..SIGNATURE_LENGTH].copy_from_slice(signature);

			let hash = keccak_256(data);

			let pub_key = sp_io::crypto::secp256k1_ecdsa_recover_compressed(&sig, &hash)?;
			return Ok(pub_key)
		}
		Err(EcdsaVerifyError::BadSignature)
	}

	/// Utility function to create slice of fixed size
	pub fn to_slice_33(val: &[u8]) -> Option<[u8; 33]> {
		if val.len() == ECDSA_KEY_LENGTH {
			let mut key = [0u8; ECDSA_KEY_LENGTH];
			key[..ECDSA_KEY_LENGTH].copy_from_slice(val);

			return Some(key)
		}
		None
	}

	/// Utility function to create slice of fixed size
	pub fn to_slice_32(val: &[u8]) -> Option<[u8; 32]> {
		if val.len() == SCHNORR_KEY_LENGTH {
			let mut key = [0u8; SCHNORR_KEY_LENGTH];
			key[..SCHNORR_KEY_LENGTH].copy_from_slice(val);

			return Some(key)
		}
		None
	}

	pub fn xor_array<A, B>(mut a: A, b: B) -> A
	where
		A: AsMut<[u8]>,
		B: AsRef<[u8]>,
	{
		a.as_mut().iter_mut().zip(b.as_ref()).for_each(|(a_i, b_i)| *a_i ^= *b_i);
		a
	}
}
