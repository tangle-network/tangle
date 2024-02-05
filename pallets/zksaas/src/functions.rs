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
use tangle_primitives::{jobs::*, verifier::*};

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
			let validator_fee = fee_info.circuit_fee * (validator_count as u32).into();
			validator_fee.saturating_add(fee_info.base_fee)
		} else {
			fee_info.base_fee.saturating_add(fee_info.get_prove_fee())
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
		data: JobWithResult<
			T::AccountId,
			T::MaxParticipants,
			T::MaxSubmissionLen,
			T::MaxKeyLen,
			T::MaxDataLen,
			T::MaxSignatureLen,
			T::MaxProofLen,
		>,
	) -> DispatchResult {
		match (data.phase_one_job_type, data.job_type, data.result) {
			(None, _, JobResult::ZkSaaSPhaseOne(_)) => Ok(()),
			(
				Some(JobType::ZkSaaSPhaseOne(circuit)),
				JobType::ZkSaaSPhaseTwo(req),
				JobResult::ZkSaaSPhaseTwo(res),
			) => Self::verify_proof(circuit, req, res),
			_ => Err(Error::<T>::InvalidJobType.into()), // this should never happen
		}
	}

	/// Verifies a given proof submission.
	pub fn verify_proof(
		ZkSaaSPhaseOneJobType { system, .. }: ZkSaaSPhaseOneJobType<
			T::AccountId,
			T::MaxParticipants,
			T::MaxSubmissionLen,
		>,
		ZkSaaSPhaseTwoJobType { request, .. }: ZkSaaSPhaseTwoJobType<T::MaxSubmissionLen>,
		res: ZkSaaSProofResult<T::MaxProofLen>,
	) -> DispatchResult {
		match (system, request, res) {
			(
				ZkSaaSSystem::Groth16(sys),
				ZkSaaSPhaseTwoRequest::Groth16(req),
				ZkSaaSProofResult::Circom(res),
			) => Self::verify_circom_proof(sys, req, res),
			(
				ZkSaaSSystem::Groth16(sys),
				ZkSaaSPhaseTwoRequest::Groth16(req),
				ZkSaaSProofResult::Arkworks(res),
			) => Self::verify_arkworks_proof(sys, req, res),
		}
	}

	/// Verifies a given circom proof submission.
	pub fn verify_circom_proof(
		system: Groth16System<T::MaxSubmissionLen>,
		req: Groth16ProveRequest<T::MaxSubmissionLen>,
		res: CircomProofResult<T::MaxProofLen>,
	) -> DispatchResult {
		let maybe_verified =
			T::Verifier::verify(&req.public_input, &res.proof, &system.verifying_key);
		match maybe_verified {
			Ok(true) => Ok(()),
			Ok(false) => Err(Error::<T>::InvalidProof.into()),
			Err(e) => {
				log::warn!(target: "zksaas::verify_circom_proof", "Invalid Circom Proof: {}", e);
				Err(Error::<T>::MalformedProof.into())
			},
		}
	}

	/// Verifies a given arkworks proof submission.
	pub fn verify_arkworks_proof(
		system: Groth16System<T::MaxSubmissionLen>,
		req: Groth16ProveRequest<T::MaxSubmissionLen>,
		res: ArkworksProofResult<T::MaxProofLen>,
	) -> DispatchResult {
		let maybe_verified =
			T::Verifier::verify(&req.public_input, &res.proof, &system.verifying_key);
		match maybe_verified {
			Ok(true) => Ok(()),
			Ok(false) => Err(Error::<T>::InvalidProof.into()),
			Err(e) => {
				log::warn!(target: "zksaas::verify_arkworks_proof", "Invalid Arkworks Proof: {}", e);
				Err(Error::<T>::MalformedProof.into())
			},
		}
	}
}
