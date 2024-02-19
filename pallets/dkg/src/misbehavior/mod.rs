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
use signatures_schemes::ecdsa::recover_ecdsa_pub_key_compressed;
use tangle_primitives::{
	misbehavior::{
		DKGTSSJustification, MisbehaviorJustification, MisbehaviorSubmission, SignedRoundMessage,
	},
	roles::RoleType,
};

pub mod dfns_cggmp21;
pub mod zcash_frost;

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
			MisbehaviorJustification::DKGTSS(DKGTSSJustification::ZCashFrost(
				ref justification,
			)) => Self::verify_zcash_frost_misbehavior(&data, justification),
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
		let signer = recover_ecdsa_pub_key_compressed(&final_message, &signed_message.signature)
			.map_err(|_| Error::<T>::InvalidSignature)?;
		ensure!(signer == expected_signer, Error::<T>::NotSignedByOffender);
		Ok(())
	}
}
