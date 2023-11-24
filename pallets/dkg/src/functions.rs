use super::*;
use crate::types::{BalanceOf, FeeInfoOf};
use frame_support::{pallet_prelude::DispatchResult, sp_runtime::Saturating, traits::Get};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_std::default::Default;
use tangle_primitives::{jobs::*, roles::RoleType};

impl<T: Config> Pallet<T> {
	fn job_to_fee(job: &JobSubmission<T::AccountId, BlockNumberFor<T>>) -> BalanceOf<T> {
		let fee_info = FeeInfo::<T>::get();
		// charge the base fee + per validator fee
		if job.job_type.is_phase_one() {
			let validator_count =
				job.job_type.clone().get_participants().expect("checked_above").len();
			let validator_fee = fee_info.dkg_validator_fee * (validator_count as u32).into();
			return validator_fee.saturating_add(fee_info.base_fee)
		} else {
			return fee_info.base_fee.saturating_add(fee_info.dkg_validator_fee)
		}
	}

	fn verify(
		job: &JobInfo<T::AccountId, BlockNumberFor<T>, tangle_primitives::Balance>,
		phase_one_data: Option<PhaseOneResult<T::AccountId, BlockNumberFor<T>>>,
		result: Vec<u8>,
		signatures: Vec<Vec<u8>>
	) -> DispatchResult {

		if job.job_type.is_phase_one() {
			return Self::verify_generated_dkg_key(
				result,
				signatures,
				job.job_type.clone().get_participants().expect("already verified above"),
				job.job_type.clone().get_threshold().expect("already verified above"),
			)
		}

		Ok(())
	}

	fn verify_generated_dkg_key(
		result: Vec<u8>,
		signatures: Vec<Vec<u8>>,
		authorities: Vec<T::AccountId>,
		threshold: u8,
	) -> DispatchResult {

		// try to decode the result
		

		// 


		Ok(())
	}
}
