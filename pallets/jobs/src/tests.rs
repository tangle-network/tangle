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
#![cfg(test)]
use crate::mock_evm::{address_build, EIP1559UnsignedTransaction};

use super::*;
use hex::FromHex;
use mock::*;

use frame_support::{assert_noop, assert_ok};
use pallet_evm::{AddressMapping, HashedAddressMapping};
use sp_core::U256;
use sp_runtime::{traits::BlakeTwo256, AccountId32};
use sp_std::sync::Arc;
use tangle_primitives::jobs::{
	DKGJobType, DKGSignatureJobType, DKGSignatureResult, JobSubmission, JobType,
};

const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
const BOB: AccountId32 = AccountId32::new([2u8; 32]);
const CHARLIE: AccountId32 = AccountId32::new([3u8; 32]);
const DAVE: AccountId32 = AccountId32::new([4u8; 32]);
const EVE: AccountId32 = AccountId32::new([5u8; 32]);

const TEN: AccountId32 = AccountId32::new([10u8; 32]);
const TWENTY: AccountId32 = AccountId32::new([20u8; 32]);
const HUNDRED: AccountId32 = AccountId32::new([100u8; 32]);

use ethers::prelude::*;
use serde_json::Value;
use std::fs;

fn get_signing_rules_abi() -> (Value, Value) {
	let mut data: Value = serde_json::from_str(
		&fs::read_to_string("../../forge/out/SigningRules.sol/VotableSigningRules.json").unwrap(),
	)
	.unwrap();
	let abi = data["abi"].take();
	let bytecode = data["bytecode"]["object"].take();
	(abi, bytecode)
}

fn eip1559_signing_rules_creation_unsigned_transaction(
	bytecode: Vec<u8>,
) -> EIP1559UnsignedTransaction {
	EIP1559UnsignedTransaction {
		nonce: U256::zero(),
		max_priority_fee_per_gas: U256::from(1),
		max_fee_per_gas: U256::from(1),
		gas_limit: U256::from(0x100000),
		action: pallet_ethereum::TransactionAction::Create,
		value: U256::zero(),
		input: bytecode,
	}
}

fn eip1559_contract_call_unsigned_transaction(
	address: Address,
	data: Vec<u8>,
) -> EIP1559UnsignedTransaction {
	EIP1559UnsignedTransaction {
		nonce: U256::zero(),
		max_priority_fee_per_gas: U256::from(1),
		max_fee_per_gas: U256::from(1),
		gas_limit: U256::from(0x100000),
		action: pallet_ethereum::TransactionAction::Call(address),
		value: U256::zero(),
		input: data,
	}
}

#[test]
fn jobs_submission_e2e_works_for_dkg() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let submission = JobSubmission {
			expiry: 100,
			job_type: JobType::DKG(DKGJobType {
				participants: vec![HUNDRED, BOB, CHARLIE, DAVE, EVE],
				threshold: 3,
				permitted_caller: None,
			}),
		};

		// should fail with invalid validator
		assert_noop!(
			Jobs::submit_job(RuntimeOrigin::signed(ALICE), submission),
			Error::<Runtime>::InvalidValidator
		);

		let submission = JobSubmission {
			expiry: 100,
			job_type: JobType::DKG(DKGJobType {
				participants: vec![ALICE, BOB, CHARLIE, DAVE, EVE],
				threshold: 5,
				permitted_caller: None,
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
			job_type: JobType::DKG(DKGJobType {
				participants: vec![ALICE, BOB, CHARLIE, DAVE, EVE],
				threshold: 3,
				permitted_caller: None,
			}),
		};
		assert_noop!(
			Jobs::submit_job(RuntimeOrigin::signed(ALICE), submission),
			sp_runtime::TokenError::FundsUnavailable
		);

		let submission = JobSubmission {
			expiry: 100,
			job_type: JobType::DKG(DKGJobType {
				participants: vec![ALICE, BOB, CHARLIE, DAVE, EVE],
				threshold: 3,
				permitted_caller: Some(TEN),
			}),
		};
		assert_ok!(Jobs::submit_job(RuntimeOrigin::signed(TEN), submission));

		assert_eq!(Balances::free_balance(TEN), 100 - 5);

		// submit a solution for this job
		assert_ok!(Jobs::submit_job_result(
			RuntimeOrigin::signed(TEN),
			JobKey::DKG,
			0,
			JobResult::DKG(DKGResult {
				keys_and_signatures: vec![],
				threshold: 3,
				participants: vec![],
				key: vec![]
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
			job_type: JobType::DKGSignature(DKGSignatureJobType {
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
			job_type: JobType::DKGSignature(DKGSignatureJobType {
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
			JobResult::DKGSignature(DKGSignatureResult {
				signing_key: vec![],
				signature: vec![],
				data: vec![]
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

#[test]
fn test_signing_rules() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let pairs = (0..10).map(|i| address_build(i as u8)).collect::<Vec<_>>();
		let alice = &pairs[0];

		let (abi, bytecode) = get_signing_rules_abi();
		let stripped_bytecode = bytecode.as_str().unwrap().trim_start_matches("0x");
		let decoded = hex::decode(stripped_bytecode).unwrap();
		let signing_rules_create_tx = eip1559_signing_rules_creation_unsigned_transaction(decoded);
		let signed_tx = signing_rules_create_tx.sign(&alice.private_key, None);
		let res = Ethereum::execute(alice.address, &signed_tx, None);
		assert_ok!(res.clone());
		assert!(res.clone().unwrap().1.is_some());
		let signing_rules_address = res.unwrap().1.unwrap();

		let submission = JobSubmission {
			expiry: 100,
			job_type: JobType::DKG(DKGJobType {
				participants: vec![ALICE, BOB, CHARLIE, DAVE, EVE],
				threshold: 3,
				permitted_caller: Some(HashedAddressMapping::<BlakeTwo256>::into_account_id(
					signing_rules_address,
				)),
			}),
		};
		assert_ok!(Jobs::submit_job(RuntimeOrigin::signed(TEN), submission.clone()));

		abigen!(SigningRules, "../../forge/out/SigningRules.sol/VotableSigningRules.json");
		let (provider, _) = Provider::mocked();
		let client = Arc::new(provider);
		let contract = SigningRules::new(Address::from(signing_rules_address), client);

		let phase_1_job_id = [0u8; 32];
		let phase_1_job_details = submission.job_type.encode().into();
		let threshold = 3;
		let use_democracy = false;
		let voters = vec![
			pairs[0].address,
			pairs[1].address,
			pairs[2].address,
			pairs[3].address,
			pairs[4].address,
		];
		let expiry = 1000;
		let ethers_call: FunctionCall<_, _, _> = contract.initialize(
			phase_1_job_id,
			phase_1_job_details,
			threshold,
			use_democracy,
			voters,
			expiry,
		);
		let initialize_tx = eip1559_contract_call_unsigned_transaction(
			signing_rules_address,
			ethers_call.calldata().unwrap().to_vec(),
		);
		let signed_tx = initialize_tx.sign(&alice.private_key, None);
		let res = Ethereum::execute(alice.address, &signed_tx, None);
		assert_ok!(res.clone());
	});
}
