// This file is part of Tangle.
// Copyright (C) 2022-2023 Webb Technologies Inc.
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
use frame_support::{ensure, pallet_prelude::DispatchResult, sp_runtime::Saturating};
use frame_system::pallet_prelude::BlockNumberFor;
use tangle_primitives::jobs::*;

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
	pub fn job_to_fee(job: &JobSubmission<T::AccountId, BlockNumberFor<T>>) -> BalanceOf<T> {
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
	pub fn verify(data: JobWithResult<T::AccountId>) -> DispatchResult {
		match (data.result, data.phase_one_job_type) {
			(JobResult::ZkSaasCircuit(_), None) => Ok(()),
			(JobResult::ZkSaasProve(info), Some(JobType::ZkSaasCircuit(circuit))) =>
				Self::verify_proof(circuit, info),
			_ => Err(Error::<T>::InvalidJobType.into()), // this should never happen
		}
	}

	/// Verifies a given proof submission.
	pub fn verify_proof(
		ZkSaasCircuitJobType { system, .. }: ZkSaasCircuitJobType<T::AccountId>,
		proof: ZkSaasProofResult,
	) -> DispatchResult {
		match proof {
			ZkSaasProofResult::Circom(info) => Self::verify_circom_proof(system, info),
		}
	}

	/// Verifies a given circom proof submission.
	pub fn verify_circom_proof(system: ZkSaasSystem, proof: CircomProofResult) -> DispatchResult {
		Ok(())
	}
}
