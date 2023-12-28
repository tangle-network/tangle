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
#![cfg(test)]
use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

use sp_runtime::AccountId32;

use tangle_primitives::jobs::{
	DKGTSSPhaseOneJobType, DKGTSSPhaseTwoJobType, DKGTSSSignatureResult, DkgKeyType, JobSubmission,
	JobType,
};

const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
const BOB: AccountId32 = AccountId32::new([2u8; 32]);
const CHARLIE: AccountId32 = AccountId32::new([3u8; 32]);
const DAVE: AccountId32 = AccountId32::new([4u8; 32]);
const EVE: AccountId32 = AccountId32::new([5u8; 32]);

const TEN: AccountId32 = AccountId32::new([10u8; 32]);
const TWENTY: AccountId32 = AccountId32::new([20u8; 32]);
const HUNDRED: AccountId32 = AccountId32::new([100u8; 32]);

#[test]
fn jobs_submission_e2e_works_for_dkg() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let submission = JobSubmission {
			expiry: 100,
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType {
				participants: vec![HUNDRED, BOB, CHARLIE, DAVE, EVE],
				threshold: 3,
				permitted_caller: None,
				key_type: DkgKeyType::Ecdsa,
			}),
		};

		// should fail with invalid validator
		assert_noop!(
			Jobs::submit_job(RuntimeOrigin::signed(ALICE), submission),
			Error::<Runtime>::InvalidValidator
		);

		let submission = JobSubmission {
			expiry: 100,
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType {
				participants: vec![ALICE, BOB, CHARLIE, DAVE, EVE],
				threshold: 5,
				permitted_caller: None,
				key_type: DkgKeyType::Ecdsa,
			}),
		};

		// should fail with invalid threshold
		assert_noop!(
			Jobs::submit_job(RuntimeOrigin::signed(ALICE), submission),
			Error::<Runtime>::InvalidJobParams
		);

		// should fail when caller has no balance
		let submission = JobSubmission {
			expiry: 100,
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType {
				participants: vec![ALICE, BOB, CHARLIE, DAVE, EVE],
				threshold: 3,
				permitted_caller: None,
				key_type: DkgKeyType::Ecdsa,
			}),
		};
		assert_noop!(
			Jobs::submit_job(RuntimeOrigin::signed(ALICE), submission),
			sp_runtime::TokenError::FundsUnavailable
		);

		let submission = JobSubmission {
			expiry: 100,
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType {
				participants: vec![ALICE, BOB, CHARLIE, DAVE, EVE],
				threshold: 3,
				permitted_caller: Some(TEN),
				key_type: DkgKeyType::Ecdsa,
			}),
		};
		assert_ok!(Jobs::submit_job(RuntimeOrigin::signed(TEN), submission));

		assert_eq!(Balances::free_balance(TEN), 100 - 5);

		// submit a solution for this job
		assert_ok!(Jobs::submit_job_result(
			RuntimeOrigin::signed(TEN),
			JobKey::DKG,
			0,
			JobResult::DKGPhaseOne(DKGTSSKeySubmissionResult {
				signatures: vec![],
				threshold: 3,
				participants: vec![],
				key: vec![],
				key_type: DkgKeyType::Ecdsa
			})
		));

		// ensure the job reward is distributed correctly
		for validator in [ALICE, BOB, CHARLIE, DAVE, EVE] {
			assert_eq!(ValidatorRewards::<Runtime>::get(validator), Some(1));
		}

		// ensure storage is correctly setup
		assert!(KnownResults::<Runtime>::get(JobKey::DKG, 0).is_some());
		assert!(SubmittedJobs::<Runtime>::get(JobKey::DKG, 0).is_none());

		// ---- use phase one solution in phase 2 signinig -------

		// another account cannot use solution
		let submission = JobSubmission {
			expiry: 100,
			job_type: JobType::DKGTSSPhaseTwo(DKGTSSPhaseTwoJobType {
				phase_one_id: 0,
				submission: vec![],
			}),
		};
		assert_noop!(
			Jobs::submit_job(RuntimeOrigin::signed(TWENTY), submission),
			Error::<Runtime>::InvalidJobParams
		);

		let submission = JobSubmission {
			expiry: 100,
			job_type: JobType::DKGTSSPhaseTwo(DKGTSSPhaseTwoJobType {
				phase_one_id: 0,
				submission: vec![],
			}),
		};
		assert_ok!(Jobs::submit_job(RuntimeOrigin::signed(TEN), submission));

		assert_eq!(Balances::free_balance(TEN), 100 - 25);

		// submit a solution for this job
		assert_ok!(Jobs::submit_job_result(
			RuntimeOrigin::signed(TEN),
			JobKey::DKGSignature,
			1,
			JobResult::DKGPhaseTwo(DKGTSSSignatureResult {
				signing_key: vec![],
				signature: vec![],
				data: vec![],
				key_type: DkgKeyType::Ecdsa
			})
		));

		// ensure the job reward is distributed correctly
		for validator in [ALICE, BOB, CHARLIE, DAVE, EVE] {
			assert_eq!(ValidatorRewards::<Runtime>::get(validator), Some(5));
		}

		// ensure storage is correctly setup
		assert!(KnownResults::<Runtime>::get(JobKey::DKG, 0).is_some());
		assert!(SubmittedJobs::<Runtime>::get(JobKey::DKG, 0).is_none());
	});
}

// TODO : Integrate after zksaas pallet
// #[test]
// fn jobs_submission_e2e_works_for_zksaas() {
// 	new_test_ext().execute_with(|| {
// 		System::set_block_number(1);
//
// 		let submission = JobSubmission {
// 			expiry: 100,
// 			job_type: JobType::ZkSaasPhaseOne(ZkSaasPhaseOneJobType {
// 				participants: vec![100, 2, 3, 4, 5],
// 			}),
// 		};
//
// 		// should fail with invalid validator
// 		assert_noop!(
// 			Jobs::submit_job(RuntimeOrigin::signed(ALICE), submission),
// 			Error::<Runtime>::InvalidValidator
// 		);
//
// 		// should fail when caller has no balance
// 		let submission = JobSubmission {
// 			expiry: 100,
// 			job_type: JobType::ZkSaasPhaseOne(ZkSaasPhaseOneJobType {
// 				participants: vec![ALICE, BOB, CHARLIE, DAVE, EVE],
// 			}),
// 		};
// 		assert_noop!(
// 			Jobs::submit_job(RuntimeOrigin::signed(ALICE), submission),
// 			sp_runtime::TokenError::FundsUnavailable
// 		);
//
// 		let submission = JobSubmission {
// 			expiry: 100,
// 			job_type: JobType::ZkSaasPhaseOne(ZkSaasPhaseOneJobType {
// 				participants: vec![ALICE, BOB, CHARLIE, DAVE, EVE],
// 			}),
// 		};
// 		assert_ok!(Jobs::submit_job(RuntimeOrigin::signed(TEN), submission));
//
// 		assert_eq!(Balances::free_balance(TEN), 100 - 10);
//
// 		// submit a solution for this job
// 		assert_ok!(Jobs::submit_job_result(
// 			RuntimeOrigin::signed(TEN),
// 			JobKey::ZkSaasPhaseOne,
// 			0,
// 			vec![]
// 		));
//
// 		// ensure the job reward is distributed correctly
// 		for validator in [1, 2, 3, 4, 5] {
// 			assert_eq!(ValidatorRewards::<Runtime>::get(validator), Some(2));
// 		}
//
// 		// ensure storage is correctly setup
// 		assert!(KnownResults::<Runtime>::get(JobKey::ZkSaasPhaseOne, 0).is_some());
// 		assert!(SubmittedJobs::<Runtime>::get(JobKey::ZkSaasPhaseOne, 0).is_none());
//
// 		// ---- use phase one solution in phase 2 signinig -------
//
// 		// another account cannot use solution
// 		let submission = JobSubmission {
// 			expiry: 100,
// 			job_type: JobType::ZkSaasPhaseTwo(ZkSaasPhaseTwoJobType {
// 				phase_one_id: 0,
// 				submission: vec![],
// 			}),
// 		};
// 		assert_noop!(
// 			Jobs::submit_job(RuntimeOrigin::signed(TWENTY), submission),
// 			Error::<Runtime>::InvalidJobParams
// 		);
//
// 		let submission = JobSubmission {
// 			expiry: 100,
// 			job_type: JobType::ZkSaasPhaseTwo(ZkSaasPhaseTwoJobType {
// 				phase_one_id: 0,
// 				submission: vec![],
// 			}),
// 		};
// 		assert_ok!(Jobs::submit_job(RuntimeOrigin::signed(TEN), submission));
//
// 		assert_eq!(Balances::free_balance(TEN), 100 - 30);
//
// 		// ensure the job reward is distributed correctly
// 		for validator in [1, 2, 3, 4, 5] {
// 			assert_eq!(ValidatorRewards::<Runtime>::get(validator), Some(2));
// 		}
//
// 		// ensure storage is correctly setup
// 		assert!(KnownResults::<Runtime>::get(JobKey::ZkSaasPhaseOne, 0).is_some());
// 		assert!(SubmittedJobs::<Runtime>::get(JobKey::ZkSaasPhaseOne, 0).is_none());
// 	});
// }

// #[test]
// fn withdraw_validator_rewards_works() {
// 	new_test_ext().execute_with(|| {
// 		System::set_block_number(1);
//
// 		ValidatorRewards::<Runtime>::insert(1, 100);
// 		ValidatorRewards::<Runtime>::insert(2, 100);
// 		ValidatorRewards::<Runtime>::insert(3, 100);
//
// 		// can withdraw the reward by validator
// 		for validator in [1, 2, 3] {
// 			assert_ok!(Jobs::withdraw_rewards(RuntimeOrigin::signed(validator)));
// 			assert_eq!(ValidatorRewards::<Runtime>::get(validator), None);
// 		}
// 	});
// }
