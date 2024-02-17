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

use sp_runtime::DispatchResult;
use tangle_primitives::misbehavior::{
	zcash_frost::{KeygenAborted, SigningAborted, ZCashFrostJustification}, MisbehaviorSubmission
};

use crate::{Config, Pallet};

pub mod keygen;

impl<T: Config> Pallet<T> {
	/// Verifies a given a misbehavior justification and dispatches to specific verification logic
	pub fn verify_zcash_frost_misbehavior(
		data: &MisbehaviorSubmission,
		justification: &ZCashFrostJustification,
	) -> DispatchResult {
		match justification {
			ZCashFrostJustification::Keygen { participants, t, reason } =>
				Self::verify_zcash_frost_keygen_misbehavior(data, participants, *t, reason),
			ZCashFrostJustification::Signing { participants, t, reason } =>
				Self::verify_zcash_frost_signing_misbehavior(data, participants, *t, reason),
		}
	}

	/// given a keygen misbehavior justification, verify the misbehavior and return a dispatch
	/// result
	pub fn verify_zcash_frost_keygen_misbehavior(
		data: &MisbehaviorSubmission,
		participants: &[[u8; 33]],
		t: u16,
		reason: &KeygenAborted,
	) -> DispatchResult {
		match reason {
			KeygenAborted::InvalidProofOfKnowledge { round1, round2 } =>
				keygen::schnorr_proof::<T>(data, round1, round2),
		}
	}

	/// given a signing misbehavior justification, verify the misbehavior and return a dispatch
	/// result
	pub fn verify_zcash_frost_signing_misbehavior(
		_data: &MisbehaviorSubmission,
		_participants: &[[u8; 33]],
		_t: u16,
		_reason: &SigningAborted,
	) -> DispatchResult {
		unimplemented!()
	}
}
