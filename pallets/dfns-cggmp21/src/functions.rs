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
use dfns_cggmp21::{keygen, security_level::SecurityLevel128};
use frame_support::{ensure, pallet_prelude::DispatchResult};
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

use dfns_cggmp21::supported_curves::Secp256k1;
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
			DfnsCGGMP21Justification::Keygen { n, t, reason } =>
				Self::verify_keygen_misbehavior(data, *n, *t, reason),
			DfnsCGGMP21Justification::Signing { n, t, reason } =>
				Self::verify_signing_misbehavior(data, *n, *t, reason),
		}
	}

	/// given a keygen misbehavior justification, verify the misbehavior and return a dispatch
	/// result
	pub fn verify_keygen_misbehavior(
		data: &MisbehaviorSubmission,
		n: u16,
		t: u16,
		reason: &KeygenAborted,
	) -> DispatchResult {
		match reason {
			KeygenAborted::InvalidDecommitment { round1, round2a } =>
				Self::verify_keygen_invalid_decommitment(data, round1, round2a),
			KeygenAborted::InvalidDataSize { round2a } =>
				Self::verify_keygen_invalid_data_size(data, t, round2a),
			_ => unimplemented!(),
		}
	}

	/// given a signing misbehavior justification, verify the misbehavior and return a dispatch
	/// result
	pub fn verify_signing_misbehavior(
		data: &MisbehaviorSubmission,
		n: u16,
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
		if round1_msg.commitment == hash_commit {
			Err(Error::<T>::ValidDecommitment.into())
		} else {
			// Slash the offender!
			// TODO: add slashing logic

			Ok(())
		}
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
		if round2a_msg.F.degree() + 1 == usize::from(t) {
			Err(Error::<T>::ValidDataSize.into())
		} else {
			// Slash the offender!
			// TODO: add slashing logic

			Ok(())
		}
	}
	/// Given a [`SignedMessage`] ensure that the message is signed by the given offender.
	pub fn ensure_signed_by_offender(
		signed_message: &SignedRoundMessage,
		offender: [u8; 33],
	) -> DispatchResult {
		let final_message =
			[&signed_message.sender.to_be_bytes()[..], signed_message.message.as_slice()].concat();
		let signer = Self::recover_ecdsa_pub_key(&final_message, &signed_message.signature)
			.map_err(|_| Error::<T>::InvalidSignature)?;
		ensure!(signer == offender, Error::<T>::NotSignedByOffender);
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
}
