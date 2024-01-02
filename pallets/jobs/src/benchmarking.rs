use crate::{
	BalanceOf, Call, Config, JobSubmissionOf, Pallet, ValidatorOffenceType, ValidatorRewards,
};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_runtime::traits::{Bounded, Zero};
use sp_std::vec;
use tangle_primitives::{
	jobs::{DKGJobType, JobId, JobType},
	roles::RoleType,
};

benchmarks! {
	// Benchmark submit_job function
	submit_job {
		let caller: T::AccountId = account("caller", 0, 0);
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
		let job =  JobSubmissionOf::<T> {
			expiry: 100u32.into(),
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType { participants: vec![caller.clone(), caller.clone()], threshold: 1, permitted_caller: None, key_type: Ecdsa }),
		};

	}: _(RawOrigin::Signed(caller.clone()), job.clone())

	// Benchmark submit_job_result function
	submit_job_result {
		let caller: T::AccountId = account("caller", 0, 0);
		let validator2: T::AccountId = account("caller", 0, 1);
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
		let job =  JobSubmissionOf::<T> {
			expiry: 100u32.into(),
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType { participants: vec![caller.clone(), validator2], threshold: 1, permitted_caller: None, key_type: Ecdsa }),
		};
		let _ = Pallet::<T>::submit_job(RawOrigin::Signed(caller.clone()).into(), job);
		let role_type: RoleType = RoleType::Tss(ThresholdSignatureRoleType::TssGG20);
		let job_id: JobId = 0;
	}: _(RawOrigin::Signed(caller.clone()), role_type, job_id.clone(), vec![])

	// Benchmark withdraw_rewards function
	withdraw_rewards {
		let caller: T::AccountId = account("caller", 0, 0);
		let pallet_account = Pallet::<T>::rewards_account_id();
		let _ = T::Currency::make_free_balance_be(&pallet_account, BalanceOf::<T>::max_value());
		let reward_amount: BalanceOf<T> = 100u32.into(); // Set a reward amount for testing
		ValidatorRewards::<T>::insert(caller.clone(), reward_amount);
	}: _(RawOrigin::Signed(caller.clone()))

	// Benchmark report_inactive_validator function
	report_inactive_validator {
		let caller: T::AccountId = account("caller", 0, 0);
		let validator2: T::AccountId = account("caller", 0, 1);
		let validator3: T::AccountId = account("caller", 0, 2);
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
		let job =  JobSubmissionOf::<T> {
			expiry: 100u32.into(),
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType { participants: vec![caller.clone(), validator2, validator3], threshold: 2, permitted_caller: None, key_type: Ecdsa }),
			};
		let _ = Pallet::<T>::submit_job(RawOrigin::Signed(caller.clone()).into(), job);
		let role_type: RoleType = RoleType::Tss(ThresholdSignatureRoleType::TssGG20);
		let job_id: JobId = 0;
	}: _(RawOrigin::Signed(caller.clone()), role_type, job_id.clone(), caller.clone(), ValidatorOffenceType::Inactivity, vec![])
}

// Define the module and associated types for the benchmarks
impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Runtime,);
