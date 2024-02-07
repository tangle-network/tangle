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
use frame_support::{ensure, pallet_prelude::DispatchResult};
use sp_io::{hashing::keccak_256, EcdsaVerifyError};
use tangle_primitives::{
	misbehavior::{
		dfns_cggmp21::SignedRoundMessage, DKGTSSJustification, MisbehaviorJustification,
		MisbehaviorSubmission,
	},
	roles::RoleType,
};

/// Expected signature length
pub const SIGNATURE_LENGTH: usize = 65;

pub mod dfns_cggmp21;

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
	pub fn verify_misbehavior(data: MisbehaviorSubmission) -> DispatchResult {
		ensure!(matches!(data.role_type, RoleType::Tss(_)), Error::<T>::InvalidRoleType);

		ensure!(
			matches!(data.justification, MisbehaviorJustification::DKGTSS(_)),
			Error::<T>::InvalidJustification,
		);

		match data.justification {
			MisbehaviorJustification::DKGTSS(DKGTSSJustification::DfnsCGGMP21(
				ref justification,
			)) => Self::verify_dfns_cggmp21_misbehavior(&data, justification),
			MisbehaviorJustification::ZkSaaS(_) => Err(Error::<T>::InvalidJustification.into()),
		}
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
		let signer =
			Self::recover_ecdsa_pub_key_compressed(&final_message, &signed_message.signature)
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
	pub fn recover_ecdsa_pub_key_compressed(
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
}
