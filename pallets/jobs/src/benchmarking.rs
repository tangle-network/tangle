use crate::{
	BalanceOf, Call, Config, JobSubmissionOf, Pallet, ValidatorOffenceType, ValidatorRewards,
};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_runtime::traits::{Bounded, Zero};
use sp_std::vec;
use tangle_primitives::{
	jobs::{
		DKGTSSKeySubmissionResult, DKGTSSPhaseOneJobType, DigitalSignatureScheme, FallbackOptions,
		JobId, JobResult, JobType,
	},
	roles::{RoleType, ThresholdSignatureRoleType},
};

benchmarks! {
	// Benchmark submit_job function
	submit_job {
		let caller: T::AccountId = account("caller", 0, 0);
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
		let job =  JobSubmissionOf::<T> {
			expiry: 100u32.into(),
			ttl: 100u32.into(),
			fallback: FallbackOptions::Destroy,
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType { participants: vec![caller.clone(), caller.clone()].try_into().unwrap(), threshold: 1, permitted_caller: None, role_type : Default::default()  }),
		};

	}: _(RawOrigin::Signed(caller.clone()), job.clone())

	// Benchmark submit_job_result function
	submit_job_result {
		let caller: T::AccountId = account("caller", 0, 0);
		let validator2: T::AccountId = account("caller", 0, 1);
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
		let job =  JobSubmissionOf::<T> {
			expiry: 100u32.into(),
			ttl: 100u32.into(),
			fallback: FallbackOptions::Destroy,
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType { participants: vec![caller.clone(), validator2].try_into().unwrap(), threshold: 1, permitted_caller: None, role_type : Default::default() }),
		};
		let _ = Pallet::<T>::submit_job(RawOrigin::Signed(caller.clone()).into(), job);
		let job_key: RoleType = RoleType::Tss(Default::default());
		let job_id: JobId = 0;
		let result = JobResult::DKGPhaseOne(DKGTSSKeySubmissionResult {
			signatures: vec![].try_into().unwrap(),
			threshold: 3,
			participants: vec![].try_into().unwrap(),
			key: vec![].try_into().unwrap(),
			signature_scheme: DigitalSignatureScheme::EcdsaSecp256k1
		});
	}: _(RawOrigin::Signed(caller.clone()), job_key.clone(), job_id.clone(), result)

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
			ttl: 100u32.into(),
			fallback: FallbackOptions::Destroy,
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType { participants: vec![caller.clone(), validator2, validator3].try_into().unwrap(), threshold: 2, permitted_caller: None, role_type : Default::default() }),
		};
		let _ = Pallet::<T>::submit_job(RawOrigin::Signed(caller.clone()).into(), job);
		let job_key: RoleType = RoleType::Tss(Default::default());
		let job_id: JobId = 0;
	}: _(RawOrigin::Signed(caller.clone()), job_key.clone(), job_id.clone(), caller.clone(), ValidatorOffenceType::Inactivity, vec![])

	// Benchmark extend_job_result_ttl function
	extend_job_result_ttl {
		let caller: T::AccountId = account("caller", 0, 0);
		let validator2: T::AccountId = account("caller", 0, 1);
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
		let job =  JobSubmissionOf::<T> {
			expiry: 100u32.into(),
			ttl: 100u32.into(),
			fallback: FallbackOptions::Destroy,
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType { participants: vec![caller.clone(), validator2].try_into().unwrap(), threshold: 1, permitted_caller: None, role_type : Default::default() }),
		};
		let _ = Pallet::<T>::submit_job(RawOrigin::Signed(caller.clone()).into(), job);
		let job_key: RoleType = RoleType::Tss(Default::default());
		let job_id: JobId = 0;
		let result = JobResult::DKGPhaseOne(DKGTSSKeySubmissionResult {
			signatures: vec![].try_into().unwrap(),
			threshold: 3,
			participants: vec![].try_into().unwrap(),
			key: vec![].try_into().unwrap(),
			signature_scheme: DigitalSignatureScheme::EcdsaSecp256k1
		});
		let _ = Pallet::<T>::submit_job_result(RawOrigin::Signed(caller.clone()).into(), job_key.clone(), job_id.clone(), result);
	}: _(RawOrigin::Signed(caller.clone()), RoleType::Tss(Default::default()), job_id.clone(), 10u32.into())
}

// Define the module and associated types for the benchmarks
impl_benchmark_test_suite!(
	Pallet,
	crate::mock::new_test_ext(vec![
		frame_benchmarking::account("caller", 0, 1),
		frame_benchmarking::account("caller", 0, 2)
	]),
	crate::mock::Runtime,
);
