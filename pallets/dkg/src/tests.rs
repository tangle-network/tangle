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
use crate::{mock::*, types::FeeInfo, Error, FeeInfo as FeeInfoStorage};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use parity_scale_codec::Encode;
use sp_core::{crypto::ByteArray, ecdsa, keccak_256};
use sp_io::crypto::{ecdsa_generate, ecdsa_sign_prehashed};
use tangle_primitives::jobs::{DKGResult, DKGSignatureResult, DkgKeyType, JobResult};

/// Key type for DKG keys
pub const KEY_TYPE: sp_application_crypto::KeyTypeId = sp_application_crypto::KeyTypeId(*b"wdkg");

fn mock_pub_key() -> ecdsa::Public {
	ecdsa_generate(KEY_TYPE, None)
}

fn mock_signature(pub_key: ecdsa::Public, dkg_key: ecdsa::Public) -> (Vec<u8>, Vec<u8>) {
	let msg = dkg_key.encode();
	let hash = keccak_256(&msg);
	let signature: ecdsa::Signature = ecdsa_sign_prehashed(KEY_TYPE, &pub_key, &hash).unwrap();
	(msg, signature.encode())
}

#[test]
fn set_fees_works() {
	new_test_ext().execute_with(|| {
		let new_fee = FeeInfo {
			base_fee: 10,
			dkg_validator_fee: 5,
			sig_validator_fee: 5,
			refresh_validator_fee: 5,
		};

		// should fail for non update origin
		assert_noop!(DKG::set_fee(RuntimeOrigin::signed(10), new_fee.clone()), BadOrigin);

		// Dispatch a signed extrinsic.
		assert_ok!(DKG::set_fee(RuntimeOrigin::signed(1), new_fee.clone()));

		assert_eq!(FeeInfoStorage::<Runtime>::get(), new_fee);
	});
}

#[test]
fn dkg_key_verifcation_works() {
	new_test_ext().execute_with(|| {
		let job_to_verify = DKGResult {
			key_type: DkgKeyType::Ecdsa,
			key: vec![],
			participants: vec![],
			keys_and_signatures: vec![],
			threshold: 2,
		};

		// should fail for empty participants
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),
			Error::<Runtime>::NoParticipantsFound
		);

		let job_to_verify = DKGResult {
			key_type: DkgKeyType::Ecdsa,
			key: vec![],
			participants: vec![mock_pub_key().as_mut().to_vec()],
			keys_and_signatures: vec![],
			threshold: 2,
		};

		// should fail for empty keys/signatures
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),
			Error::<Runtime>::NoSignaturesFound
		);

		// setup key/signature
		let mut pub_key = mock_pub_key();
		let signature = mock_signature(pub_key, pub_key);

		let job_to_verify = DKGResult {
			key_type: DkgKeyType::Ecdsa,
			key: vec![],
			participants: vec![mock_pub_key().as_mut().to_vec()],
			keys_and_signatures: vec![signature.clone()],
			threshold: 2,
		};

		// should fail for less than threshold
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),
			Error::<Runtime>::NotEnoughSigners
		);

		let job_to_verify = DKGResult {
			key_type: DkgKeyType::Ecdsa,
			key: vec![],
			participants: vec![pub_key.as_mut().to_vec()],
			keys_and_signatures: vec![signature.clone(), signature.clone()],
			threshold: 2,
		};

		// should fail for duplicate signers
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),
			Error::<Runtime>::DuplicateSignature
		);

		let job_to_verify = DKGResult {
			key_type: DkgKeyType::Ecdsa,
			key: signature.1.clone(),
			participants: vec![pub_key.as_mut().to_vec()],
			keys_and_signatures: vec![signature.clone(), mock_signature(pub_key, mock_pub_key())],
			threshold: 2,
		};

		// should fail for signing different keys
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),
			Error::<Runtime>::InvalidSignatureData
		);

		// works correctly when all params as expected
		let mut participant_one = mock_pub_key();
		let mut participant_two = mock_pub_key();
		let signature_one = mock_signature(participant_one, participant_one);
		let signature_two = mock_signature(participant_two, participant_one);
		let job_to_verify = DKGResult {
			key_type: DkgKeyType::Ecdsa,
			key: participant_one.to_raw_vec(),
			participants: vec![
				participant_one.as_mut().to_vec(),
				participant_two.as_mut().to_vec(),
			],
			keys_and_signatures: vec![signature_two, signature_one],
			threshold: 1,
		};

		// should fail for signing different keys
		assert_ok!(DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),);
	});
}

#[test]
fn dkg_signature_verifcation_works() {
	new_test_ext().execute_with(|| {
		// setup key/signature
		let pub_key = mock_pub_key();
		let signature = mock_signature(pub_key, mock_pub_key());

		let job_to_verify: DKGSignatureResult = DKGSignatureResult {
			key_type: DkgKeyType::Ecdsa,
			signature: signature.1,
			data: pub_key.to_raw_vec(),
			signing_key: pub_key.to_raw_vec(),
		};

		// should fail for invalid keys
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseTwo(job_to_verify)),
			Error::<Runtime>::SigningKeyMismatch
		);

		let signature = mock_signature(pub_key, pub_key);
		let job_to_verify: DKGSignatureResult = DKGSignatureResult {
			key_type: DkgKeyType::Ecdsa,
			signature: signature.1,
			data: pub_key.to_raw_vec(),
			signing_key: pub_key.to_raw_vec(),
		};

		// should work with correct params
		assert_ok!(DKG::verify(JobResult::DKGPhaseTwo(job_to_verify)));
	});
}
