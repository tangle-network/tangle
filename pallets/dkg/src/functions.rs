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
use crate::types::BalanceOf;
use frame_support::{pallet_prelude::DispatchResult, sp_runtime::Saturating};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_core::Get;
use tangle_primitives::jobs::*;

use self::signatures_schemes::{
	ecdsa::{verify_dkg_signature_ecdsa, verify_generated_dkg_key_ecdsa},
	schnorr_frost::verify_dkg_signature_schnorr_frost,
	schnorr_sr25519::{
		verify_dkg_signature_schnorr_sr25519, verify_generated_dkg_key_schnorr_sr25519,
	},
};

impl<T: Config> Pallet<T> {
	/// Calculates the fee for a given job submission based on the provided fee information.
	///
	/// The fee calculation considers both the base fee and an additional fee per validator,
	/// depending on the job type.
	///
	/// # Arguments
	///
	/// * `job` - A reference to the job submission containing information about the account, job
	///   type, and block number.
	///
	/// # Returns
	///
	/// Returns the calculated fee as a `BalanceOf<T>` type.
	pub fn job_to_fee(
		job: &JobSubmission<
			T::AccountId,
			BlockNumberFor<T>,
			T::MaxParticipants,
			T::MaxSubmissionLen,
		>,
	) -> BalanceOf<T> {
		let fee_info = FeeInfo::<T>::get();
		// charge the base fee + per validator fee
		if job.job_type.is_phase_one() {
			let validator_count =
				job.job_type.clone().get_participants().expect("checked_above").len();
			let validator_fee = fee_info.dkg_validator_fee * (validator_count as u32).into();
			let storage_fee = fee_info.storage_fee_per_byte * T::MaxKeyLen::get().into();
			validator_fee.saturating_add(fee_info.base_fee).saturating_add(storage_fee)
		} else {
			let storage_fee = fee_info.storage_fee_per_byte * T::MaxSignatureLen::get().into();
			fee_info
				.base_fee
				.saturating_add(fee_info.sig_validator_fee)
				.saturating_add(storage_fee)
		}
	}

	/// Verifies a given job verification information and dispatches to specific verification logic
	/// based on the job type.
	///
	/// # Arguments
	///
	/// * `data` - The job verification information, which could be of different types such as DKG
	///   or others.
	///
	/// # Returns
	///
	/// Returns a `DispatchResult` indicating whether the verification was successful or encountered
	/// an error.
	#[allow(clippy::type_complexity)]
	pub fn verify(
		data: JobResult<
			T::MaxParticipants,
			T::MaxKeyLen,
			T::MaxSignatureLen,
			T::MaxDataLen,
			T::MaxProofLen,
		>,
	) -> DispatchResult {
		match data {
			JobResult::DKGPhaseOne(info) => Self::verify_generated_dkg_key(info),
			JobResult::DKGPhaseTwo(info) => Self::verify_dkg_signature(info),
			JobResult::DKGPhaseThree(_) => Ok(()),
			JobResult::DKGPhaseFour(info) => Self::verify_dkg_key_rotation(info),
			_ => Err(Error::<T>::InvalidJobType.into()), // this should never happen
		}
	}

	/// Verifies a generated DKG (Distributed Key Generation) key based on the provided DKG result.
	///
	/// The verification process depends on the key type specified in the DKG result.
	/// It dispatches the verification to the appropriate function for the specified key type (ECDSA
	/// or Schnorr).
	///
	/// # Arguments
	///
	/// * `data` - The DKG result containing participants, keys, and signatures.
	///
	/// # Returns
	///
	/// Returns a `DispatchResult` indicating whether the DKG key verification was successful
	/// or encountered an error.
	fn verify_generated_dkg_key(
		data: DKGTSSKeySubmissionResult<T::MaxKeyLen, T::MaxParticipants, T::MaxSignatureLen>,
	) -> DispatchResult {
		match data.signature_scheme {
			DigitalSignatureScheme::Ecdsa => verify_generated_dkg_key_ecdsa::<T>(data),
			DigitalSignatureScheme::SchnorrSr25519 =>
				verify_generated_dkg_key_schnorr_sr25519::<T>(data),
            DigitalSignatureScheme::Bls381 => Self::verify_generated_dkg_key_bls(data),
			_ => Err(Error::<T>::InvalidSignature.into()),
		}
	}

    /// Verifies the generated DKG key for BLS signatures.
	///
	/// This function includes generating required signers, validating signatures, and ensuring a
	/// sufficient number of unique signers are present.
	///
	/// # Arguments
	///
	/// * `data` - The DKG verification information containing participants, keys, and signatures.
	///
	/// # Returns
	///
	/// Returns a `DispatchResult` indicating whether the DKG key verification was successful or
	/// encountered an error.
	fn verify_generated_dkg_key_bls(
		data: DKGTSSKeySubmissionResult<T::MaxKeyLen, T::MaxParticipants, T::MaxSignatureLen>,
	) -> DispatchResult {
		// The BLS public key is signed using an ECDSA signature, therefore, validate the ECDSA
		// signature only
		Self::verify_generated_dkg_key_ecdsa(data)
	}

	/// Verifies a DKG (Distributed Key Generation) signature based on the provided DKG signature
	/// result.
	///
	/// The verification process depends on the key type specified in the DKG signature result.
	/// It dispatches the verification to the appropriate function for the specified key type (ECDSA
	/// or Schnorr).
	///
	/// # Arguments
	///
	/// * `data` - The DKG signature result containing the message data, signature, signing key, and
	///   key type.
	fn verify_dkg_signature(
		data: DKGTSSSignatureResult<T::MaxDataLen, T::MaxKeyLen, T::MaxSignatureLen>,
	) -> DispatchResult {
		match data.signature_scheme {
			DigitalSignatureScheme::Ecdsa =>
				verify_dkg_signature_ecdsa::<T>(&data.data, &data.signature, &data.signing_key),
			DigitalSignatureScheme::SchnorrSr25519 => verify_dkg_signature_schnorr_sr25519::<T>(
				&data.data,
				&data.signature,
				&data.signing_key,
			),
			DigitalSignatureScheme::SchnorrEd25519 |
			DigitalSignatureScheme::SchnorrEd448 |
			DigitalSignatureScheme::SchnorrP256 |
			DigitalSignatureScheme::SchnorrP384 |
			DigitalSignatureScheme::SchnorrSecp256k1 |
			DigitalSignatureScheme::SchnorrRistretto255 => verify_dkg_signature_schnorr_frost::<T>(
				data.signature_scheme,
				&data.data,
				&data.signature,
				&data.signing_key,
			),
			_ => Err(Error::<T>::InvalidSignature.into()),
		}
	}

	/// Verifies a DKG Key Rotation.
	///
	/// The verification process depends on the key type specified in the DKG result.
	/// It dispatches the verification to the appropriate function for the specified key type (ECDSA
	/// or Schnorr).
	///
	/// # Arguments
	///
	/// * `data` - The DKG result containing current key, new key and signature.
	///
	/// # Returns
	///
	/// Returns a `DispatchResult` indicating whether the key rotation verification was successful
	/// or encountered an error.
	fn verify_dkg_key_rotation(
		data: DKGTSSKeyRotationResult<T::MaxKeyLen, T::MaxSignatureLen>,
	) -> DispatchResult {
		let emit_event = |data: DKGTSSKeyRotationResult<T::MaxKeyLen, T::MaxSignatureLen>| {
			Self::deposit_event(Event::KeyRotated {
				from_job_id: data.phase_one_id,
				to_job_id: data.new_phase_one_id,
				signature: data.signature.to_vec(),
			});

			Ok(())
		};

		match data.signature_scheme {
			DigitalSignatureScheme::Ecdsa =>
				verify_dkg_signature_ecdsa::<T>(&data.new_key, &data.signature, &data.key)
					.map(|_| emit_event(data))?,
			DigitalSignatureScheme::SchnorrSr25519 =>
				verify_dkg_signature_schnorr_sr25519::<T>(&data.new_key, &data.signature, &data.key)
					.map(|_| emit_event(data))?,
            DigitalSignatureScheme::Bls381 => Self::verify_bls_signature(&data).map(|_| emit_event(data))?,
			_ => Err(Error::<T>::InvalidSignature.into()), // unimplemented
		}
	}


    /// Verifies the DKG signature result for BLS signatures.
    ///
    /// This function uses the BLS signature algorithm to verify the provided signature
    /// based on the message data, signature, and signing key in the DKG signature result.
    ///
    /// # Arguments
    ///
    /// * `data` - The DKG signature result containing the message data, BLS signature, and signing
    ///   key.
    fn verify_bls_signature(
        data: &DKGTSSSignatureResult<T::MaxDataLen, T::MaxKeyLen, T::MaxSignatureLen>,
    ) -> DispatchResult {
        let public_key = blst::min_pk::PublicKey::deserialize(&data.signing_key)
            .map_err(|_err| Error::<T>::InvalidBlsPublicKey)?;
        let signature = blst::min_pk::Signature::deserialize(&data.signature)
            .map_err(|_err| Error::<T>::InvalidSignatureData)?;
        let dst = &mut [0u8; 48];
        let signed_data = &data.data;

        if signature.verify(true, signed_data, dst, &[], &public_key, true) !=
            blst::BLST_ERROR::BLST_SUCCESS
        {
            return Err(Error::<T>::InvalidSignature.into())
        }

        Ok(())
    }
}
