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

use tangle_primitives::{
	jobs::{
		DKGTSSPhaseOneJobType, DKGTSSPhaseTwoJobType, DKGTSSSignatureResult, DigitalSignatureType,
		Groth16ProveRequest, Groth16System, HyperData, JobSubmission, JobType, RpcResponseJobsData,
		ZkSaaSCircuitResult, ZkSaaSPhaseOneJobType, ZkSaaSPhaseTwoJobType, ZkSaaSPhaseTwoRequest,
		ZkSaaSSystem,
	},
	roles::{RoleType, ThresholdSignatureRoleType, ZeroKnowledgeRoleType},
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

		let threshold_signature_role_type = ThresholdSignatureRoleType::TssGG20;
		let submission = JobSubmission {
			expiry: 10,
			ttl: 200,
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType {
				participants: vec![HUNDRED, BOB, CHARLIE, DAVE, EVE],
				threshold: 3,
				permitted_caller: None,
				role_type: threshold_signature_role_type,
			}),
		};

		// should fail with invalid validator
		assert_noop!(
			Jobs::submit_job(RuntimeOrigin::signed(ALICE), submission),
			Error::<Runtime>::InvalidValidator
		);

		let submission = JobSubmission {
			expiry: 10,
			ttl: 200,
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType {
				participants: vec![ALICE, BOB, CHARLIE, DAVE, EVE],
				threshold: 5,
				permitted_caller: None,
				role_type: threshold_signature_role_type,
			}),
		};

		// should fail with invalid threshold
		assert_noop!(
			Jobs::submit_job(RuntimeOrigin::signed(ALICE), submission),
			Error::<Runtime>::InvalidJobParams
		);

		// should fail when caller has no balance
		let submission = JobSubmission {
			expiry: 10,
			ttl: 200,
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType {
				participants: vec![ALICE, BOB, CHARLIE, DAVE, EVE],
				threshold: 3,
				permitted_caller: None,
				role_type: threshold_signature_role_type,
			}),
		};
		assert_noop!(
			Jobs::submit_job(RuntimeOrigin::signed(ALICE), submission),
			sp_runtime::TokenError::FundsUnavailable
		);

		let submission = JobSubmission {
			expiry: 10,
			ttl: 200,
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType {
				participants: vec![ALICE, BOB, CHARLIE, DAVE, EVE],
				threshold: 3,
				permitted_caller: Some(TEN),
				role_type: threshold_signature_role_type,
			}),
		};
		assert_ok!(Jobs::submit_job(RuntimeOrigin::signed(TEN), submission));

		assert_eq!(Balances::free_balance(TEN), 100 - 5);

		// submit a solution for this job
		assert_ok!(Jobs::submit_job_result(
			RuntimeOrigin::signed(TEN),
			RoleType::Tss(ThresholdSignatureRoleType::TssGG20),
			0,
			JobResult::DKGPhaseOne(DKGTSSKeySubmissionResult {
				signatures: vec![],
				threshold: 3,
				participants: vec![],
				key: vec![],
				signature_type: DigitalSignatureType::Ecdsa
			})
		));

		// ensure the job reward is distributed correctly
		for validator in [ALICE, BOB, CHARLIE, DAVE, EVE] {
			assert_eq!(ValidatorRewards::<Runtime>::get(validator), Some(1));
		}

		// ensure storage is correctly setup
		assert!(KnownResults::<Runtime>::get(
			RoleType::Tss(ThresholdSignatureRoleType::TssGG20),
			0
		)
		.is_some());
		assert!(SubmittedJobs::<Runtime>::get(
			RoleType::Tss(ThresholdSignatureRoleType::TssGG20),
			0
		)
		.is_none());

		// ---- use phase one solution in phase 2 signinig -------

		// another account cannot use solution
		let submission = JobSubmission {
			expiry: 10,
			ttl: 0,
			job_type: JobType::DKGTSSPhaseTwo(DKGTSSPhaseTwoJobType {
				phase_one_id: 0,
				submission: vec![],
				role_type: threshold_signature_role_type,
			}),
		};
		assert_noop!(
			Jobs::submit_job(RuntimeOrigin::signed(TWENTY), submission),
			Error::<Runtime>::InvalidJobParams
		);

		let submission = JobSubmission {
			expiry: 10,
			ttl: 0,
			job_type: JobType::DKGTSSPhaseTwo(DKGTSSPhaseTwoJobType {
				phase_one_id: 0,
				submission: vec![],
				role_type: threshold_signature_role_type,
			}),
		};
		assert_ok!(Jobs::submit_job(RuntimeOrigin::signed(TEN), submission));

		assert_eq!(Balances::free_balance(TEN), 100 - 25);

		// submit a solution for this job
		assert_ok!(Jobs::submit_job_result(
			RuntimeOrigin::signed(TEN),
			RoleType::Tss(threshold_signature_role_type),
			1,
			JobResult::DKGPhaseTwo(DKGTSSSignatureResult {
				signing_key: vec![],
				signature: vec![],
				data: vec![],
				signature_type: DigitalSignatureType::Ecdsa
			})
		));

		// ensure the job reward is distributed correctly
		for validator in [ALICE, BOB, CHARLIE, DAVE, EVE] {
			assert_eq!(ValidatorRewards::<Runtime>::get(validator), Some(5));
		}

		// ensure storage is correctly setup
		assert!(KnownResults::<Runtime>::get(
			RoleType::Tss(ThresholdSignatureRoleType::TssGG20),
			0
		)
		.is_some());
		assert!(SubmittedJobs::<Runtime>::get(
			RoleType::Tss(ThresholdSignatureRoleType::TssGG20),
			0
		)
		.is_none());
	});
}

#[test]
fn jobs_rpc_tests() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let participants = vec![ALICE, BOB, CHARLIE, DAVE, EVE];

		let threshold_signature_role_type = ThresholdSignatureRoleType::TssGG20;
		let submission = JobSubmission {
			expiry: 10,
			ttl: 200,
			job_type: JobType::DKGTSSPhaseOne(DKGTSSPhaseOneJobType {
				participants: participants.clone(),
				threshold: 3,
				permitted_caller: Some(TEN),
				role_type: threshold_signature_role_type,
			}),
		};
		assert_ok!(Jobs::submit_job(RuntimeOrigin::signed(TEN), submission));

		let stored_job =
			SubmittedJobs::<Runtime>::get(RoleType::Tss(threshold_signature_role_type), 0).unwrap();
		let expected_rpc_response = RpcResponseJobsData {
			job_id: 0,
			job_type: stored_job.job_type,
			ttl: stored_job.ttl,
			expiry: stored_job.expiry,
		};

		// query jobs by validator should work
		for validator in participants {
			assert_eq!(
				Jobs::query_jobs_by_validator(validator),
				Some(vec![expected_rpc_response.clone()])
			);
		}

		assert_eq!(
			Jobs::query_job_by_id(RoleType::Tss(threshold_signature_role_type), 0),
			Some(expected_rpc_response)
		);
		assert_eq!(Jobs::query_next_job_id(), 1);

		// submit a solution for this job
		assert_ok!(Jobs::submit_job_result(
			RuntimeOrigin::signed(TEN),
			RoleType::Tss(ThresholdSignatureRoleType::TssGG20),
			0,
			JobResult::DKGPhaseOne(DKGTSSKeySubmissionResult {
				signatures: vec![],
				threshold: 3,
				participants: vec![],
				key: vec![],
				signature_type: DigitalSignatureType::Ecdsa
			})
		));

		assert_eq!(Jobs::query_job_by_id(RoleType::Tss(threshold_signature_role_type), 0), None);
		assert_eq!(Jobs::query_next_job_id(), 1);

		let expected_result =
			KnownResults::<Runtime>::get(RoleType::Tss(ThresholdSignatureRoleType::TssGG20), 0);
		assert_eq!(
			Jobs::query_job_result(RoleType::Tss(threshold_signature_role_type), 0),
			expected_result
		);

		let submission = JobSubmission {
			expiry: 10,
			ttl: 0,
			job_type: JobType::DKGTSSPhaseTwo(DKGTSSPhaseTwoJobType {
				phase_one_id: 0,
				submission: vec![],
				role_type: threshold_signature_role_type,
			}),
		};
		assert_ok!(Jobs::submit_job(RuntimeOrigin::signed(TEN), submission));

		let stored_job =
			SubmittedJobs::<Runtime>::get(RoleType::Tss(threshold_signature_role_type), 1).unwrap();
		let expected_rpc_response = RpcResponseJobsData {
			job_id: 1,
			job_type: stored_job.job_type,
			ttl: stored_job.ttl,
			expiry: stored_job.expiry,
		};

		assert_eq!(
			Jobs::query_job_by_id(RoleType::Tss(threshold_signature_role_type), 1),
			Some(expected_rpc_response)
		);
		assert_eq!(Jobs::query_next_job_id(), 2);

		let expected_result =
			KnownResults::<Runtime>::get(RoleType::Tss(ThresholdSignatureRoleType::TssGG20), 1);
		assert_eq!(
			Jobs::query_job_result(RoleType::Tss(threshold_signature_role_type), 1),
			expected_result
		);
	});
}

#[test]
fn jobs_submission_e2e_works_for_zksaas() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let dummy_system = ZkSaaSSystem::Groth16(Groth16System {
			circuit: HyperData::Raw(vec![]),
			num_inputs: 0,
			num_constraints: 0,
			proving_key: HyperData::Raw(vec![]),
			verifying_key: vec![],
			wasm: HyperData::Raw(vec![]),
		});

		let submission = JobSubmission {
			expiry: 10,
			ttl: 200,
			job_type: JobType::ZkSaaSPhaseOne(ZkSaaSPhaseOneJobType {
				permitted_caller: None,
				system: dummy_system.clone(),
				role_type: ZeroKnowledgeRoleType::ZkSaaSGroth16,
				participants: vec![HUNDRED, BOB, CHARLIE, DAVE, EVE],
			}),
		};

		// should fail with invalid validator
		assert_noop!(
			Jobs::submit_job(RuntimeOrigin::signed(ALICE), submission),
			Error::<Runtime>::InvalidValidator
		);

		// should fail when caller has no balance
		let submission = JobSubmission {
			expiry: 10,
			ttl: 200,
			job_type: JobType::ZkSaaSPhaseOne(ZkSaaSPhaseOneJobType {
				permitted_caller: None,
				system: dummy_system.clone(),
				role_type: ZeroKnowledgeRoleType::ZkSaaSGroth16,
				participants: vec![ALICE, BOB, CHARLIE, DAVE, EVE],
			}),
		};

		assert_noop!(
			Jobs::submit_job(RuntimeOrigin::signed(ALICE), submission),
			sp_runtime::TokenError::FundsUnavailable
		);

		let submission = JobSubmission {
			expiry: 10,
			ttl: 200,
			job_type: JobType::ZkSaaSPhaseOne(ZkSaaSPhaseOneJobType {
				permitted_caller: Some(TEN),
				system: dummy_system.clone(),
				role_type: ZeroKnowledgeRoleType::ZkSaaSGroth16,
				participants: vec![ALICE, BOB, CHARLIE, DAVE, EVE],
			}),
		};
		assert_ok!(Jobs::submit_job(RuntimeOrigin::signed(TEN), submission));

		assert_eq!(Balances::free_balance(TEN), 100 - 10);

		// submit a solution for this job
		assert_ok!(Jobs::submit_job_result(
			RuntimeOrigin::signed(TEN),
			RoleType::ZkSaaS(ZeroKnowledgeRoleType::ZkSaaSGroth16),
			0,
			JobResult::ZkSaaSPhaseOne(ZkSaaSCircuitResult { job_id: 0, participants: vec![] }),
		));

		// ensure the job reward is distributed correctly
		for validator in [ALICE, BOB, CHARLIE, DAVE, EVE] {
			assert_eq!(ValidatorRewards::<Runtime>::get(validator), Some(2));
		}

		// ensure storage is correctly setup
		assert!(KnownResults::<Runtime>::get(
			RoleType::ZkSaaS(ZeroKnowledgeRoleType::ZkSaaSGroth16),
			0
		)
		.is_some());
		assert!(SubmittedJobs::<Runtime>::get(
			RoleType::ZkSaaS(ZeroKnowledgeRoleType::ZkSaaSGroth16),
			0
		)
		.is_none());

		// ---- use phase one solution in phase 2 proving -------
		let dummy_req = ZkSaaSPhaseTwoRequest::Groth16(Groth16ProveRequest {
			public_input: vec![],
			a_shares: vec![],
			ax_shares: vec![],
			qap_shares: vec![],
		});
		// another account cannot use solution
		let submission = JobSubmission {
			expiry: 100,
			ttl: 200,
			job_type: JobType::ZkSaaSPhaseTwo(ZkSaaSPhaseTwoJobType {
				phase_one_id: 0,
				request: dummy_req.clone(),
				role_type: ZeroKnowledgeRoleType::ZkSaaSGroth16,
			}),
		};
		assert_noop!(
			Jobs::submit_job(RuntimeOrigin::signed(TWENTY), submission),
			Error::<Runtime>::InvalidJobParams
		);

		let submission = JobSubmission {
			expiry: 100,
			ttl: 200,
			job_type: JobType::ZkSaaSPhaseTwo(ZkSaaSPhaseTwoJobType {
				phase_one_id: 0,
				request: dummy_req,
				role_type: ZeroKnowledgeRoleType::ZkSaaSGroth16,
			}),
		};

		assert_ok!(Jobs::submit_job(RuntimeOrigin::signed(TEN), submission));

		assert_eq!(Balances::free_balance(TEN), 100 - 30);

		// ensure the job reward is distributed correctly
		for validator in [ALICE, BOB, CHARLIE, DAVE, EVE] {
			assert_eq!(ValidatorRewards::<Runtime>::get(validator), Some(2));
		}

		// ensure storage is correctly setup
		assert!(KnownResults::<Runtime>::get(
			RoleType::ZkSaaS(ZeroKnowledgeRoleType::ZkSaaSGroth16),
			1
		)
		.is_none());
		assert!(SubmittedJobs::<Runtime>::get(
			RoleType::ZkSaaS(ZeroKnowledgeRoleType::ZkSaaSGroth16),
			1
		)
		.is_some());
	});
}

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
