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

use tangle_primitives::{
	misbehavior::{
		zcash_frost::{KeygenAborted, SigningAborted, ZCashFrostJustification},
		MisbehaviorSubmission,
	},
	roles::{RoleType, ThresholdSignatureRoleType},
};

use crate::{Config, Error, Pallet};

pub mod keygen;
pub mod sign;

impl<T: Config> Pallet<T> {
	/// Verifies a given a misbehavior justification and dispatches to specific verification logic
	pub fn verify_zcash_frost_misbehavior(
		data: &MisbehaviorSubmission,
		justification: &ZCashFrostJustification,
	) -> DispatchResult {
		let role = validate_frost_role::<T>(data.role_type)?;
		match justification {
			ZCashFrostJustification::Keygen { participants, t, reason } =>
				Self::verify_zcash_frost_keygen_misbehavior(role, data, participants, *t, reason),
			ZCashFrostJustification::Signing { participants, t, reason } =>
				Self::verify_zcash_frost_signing_misbehavior(role, data, participants, *t, reason),
		}
	}

	/// given a keygen misbehavior justification, verify the misbehavior and return a dispatch
	/// result
	pub fn verify_zcash_frost_keygen_misbehavior(
		role: ThresholdSignatureRoleType,
		data: &MisbehaviorSubmission,
		participants: &[[u8; 33]],
		_t: u16,
		reason: &KeygenAborted,
	) -> DispatchResult {
		match reason {
			KeygenAborted::InvalidProofOfKnowledge { round } =>
				keygen::schnorr_proof::<T>(role, data, participants, round),
		}
	}

	/// given a signing misbehavior justification, verify the misbehavior and return a dispatch
	/// result
	pub fn verify_zcash_frost_signing_misbehavior(
		role: ThresholdSignatureRoleType,
		data: &MisbehaviorSubmission,
		participants: &[[u8; 33]],
		_t: u16,
		reason: &SigningAborted,
	) -> DispatchResult {
		// TODO: Fetch the phase one job result. It must exist for a valid
		// TODO: signing misbehavior to have occurred (chicken before the egg).
		let msg_to_sign = &[];
		let group_pubkey = &[];

		match reason {
			SigningAborted::InvalidSignatureShare { round1, round2 } =>
				sign::invalid_signature_share::<T>(
					role,
					data,
					participants,
					round1,
					round2,
					group_pubkey,
					msg_to_sign,
				),
		}
	}
}

pub fn validate_frost_role<T: Config>(
	role: RoleType,
) -> Result<ThresholdSignatureRoleType, Error<T>> {
	match role {
		RoleType::Tss(inner) => match inner {
			ThresholdSignatureRoleType::ZcashFrostEd25519 |
			ThresholdSignatureRoleType::ZcashFrostEd448 |
			ThresholdSignatureRoleType::ZcashFrostP256 |
			ThresholdSignatureRoleType::ZcashFrostP384 |
			ThresholdSignatureRoleType::ZcashFrostRistretto255 |
			ThresholdSignatureRoleType::ZcashFrostSecp256k1 => Ok(inner),
			_ => Err(Error::<T>::InvalidRoleType),
		},
		_ => Err(Error::<T>::InvalidRoleType),
	}
}

pub fn convert_error<T: Config>(err: frost_core::error::Error) -> Error<T> {
	match err {
		frost_core::error::Error::Field(_field_err) => Error::<T>::FrostFieldError,
		frost_core::error::Error::Group(_group_err) => Error::<T>::FrostGroupError,
		frost_core::error::Error::SerializationError =>
			Error::<T>::InvalidFrostMessageSerialization,
		frost_core::error::Error::DeserializationError =>
			Error::<T>::InvalidFrostMessageDeserialization,
		frost_core::error::Error::IdentifierDerivationNotSupported =>
			Error::<T>::IdentifierDerivationNotSupported,
		frost_core::error::Error::MalformedSignature => Error::<T>::MalformedFrostSignature,
		frost_core::error::Error::InvalidSignature => Error::<T>::InvalidFrostSignature,
		frost_core::error::Error::MalformedVerifyingKey => Error::<T>::MalformedFrostVerifyingKey,
		frost_core::error::Error::MalformedSigningKey => Error::<T>::MalformedFrostSigningKey,
		frost_core::error::Error::MissingCommitment => Error::<T>::MissingFrostCommitment,
		frost_core::error::Error::InvalidSignatureShare => Error::<T>::InvalidFrostSignatureShare,
		frost_core::error::Error::DuplicatedIdentifier => Error::<T>::DuplicateIdentifier,
		frost_core::error::Error::UnknownIdentifier => Error::<T>::UnknownIdentifier,
		frost_core::error::Error::IncorrectNumberOfIdentifiers =>
			Error::<T>::IncorrectNumberOfIdentifiers,
		frost_core::error::Error::IdentityCommitment => Error::<T>::IdentityCommitment,
		_ => Error::<T>::InvalidSignatureData,
	}
}
