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
use crate::{mock::*, types::FeeInfo, Error, Event, FeeInfo as FeeInfoStorage};
use frame_support::{assert_noop, assert_ok};
use parity_scale_codec::Encode;
use sp_core::{crypto::ByteArray, ecdsa, keccak_256, sr25519};
use sp_io::crypto::{ecdsa_generate, ecdsa_sign_prehashed, sr25519_generate, sr25519_sign};
use tangle_primitives::jobs::{
	DKGTSSKeyRotationResult, DKGTSSKeySubmissionResult, DKGTSSSignatureResult,
	DigitalSignatureType, JobResult,
};

fn mock_pub_key_ecdsa() -> ecdsa::Public {
	ecdsa_generate(tangle_crypto_primitives::ROLE_KEY_TYPE, None)
}

fn mock_pub_key_sr25519() -> sr25519::Public {
	sr25519_generate(tangle_crypto_primitives::ROLE_KEY_TYPE, None)
}

fn mock_signature_ecdsa(pub_key: ecdsa::Public, role_key: ecdsa::Public) -> Vec<u8> {
	let msg = role_key.encode();
	let hash = keccak_256(&msg);
	let signature: ecdsa::Signature =
		ecdsa_sign_prehashed(tangle_crypto_primitives::ROLE_KEY_TYPE, &pub_key, &hash).unwrap();
	signature.encode()
}

fn mock_signature_sr25519(pub_key: sr25519::Public, role_key: sr25519::Public) -> Vec<u8> {
	let msg = role_key.to_vec().encode();
	let hash = keccak_256(&msg);
	let signature: sr25519::Signature =
		sr25519_sign(tangle_crypto_primitives::ROLE_KEY_TYPE, &pub_key, &hash).unwrap();
	// sanity check
	assert!(sp_io::crypto::sr25519_verify(&signature, &hash, &pub_key));
	signature.encode()
}

#[test]
fn set_fees_works() {
	new_test_ext().execute_with(|| {
		let new_fee = FeeInfo {
			base_fee: 10,
			dkg_validator_fee: 5,
			sig_validator_fee: 5,
			refresh_validator_fee: 5,
			storage_fee_per_byte: 1,
		};

		// Dispatch a signed extrinsic.
		assert_ok!(DKG::set_fee(RuntimeOrigin::signed(1), new_fee.clone()));

		assert_eq!(FeeInfoStorage::<Runtime>::get(), new_fee);
	});
}

#[test]
fn dkg_key_verifcation_works_for_ecdsa() {
	new_test_ext().execute_with(|| {
		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_type: DigitalSignatureType::Ecdsa,
			key: vec![].try_into().unwrap(),
			participants: vec![].try_into().unwrap(),
			signatures: vec![].try_into().unwrap(),
			threshold: 2,
		};

		// should fail for empty participants
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),
			Error::<Runtime>::NoParticipantsFound
		);

		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_type: DigitalSignatureType::Ecdsa,
			key: vec![].try_into().unwrap(),
			participants: vec![mock_pub_key_ecdsa().as_mut().to_vec().try_into().unwrap()]
				.try_into()
				.unwrap(),
			signatures: vec![].try_into().unwrap(),
			threshold: 2,
		};

		// should fail for empty keys/signatures
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),
			Error::<Runtime>::NoSignaturesFound
		);

		// setup key/signature
		let mut pub_key = mock_pub_key_ecdsa();
		let signature = mock_signature_ecdsa(pub_key, pub_key);

		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_type: DigitalSignatureType::Ecdsa,
			key: vec![].try_into().unwrap(),
			participants: vec![mock_pub_key_ecdsa().as_mut().to_vec().try_into().unwrap()]
				.try_into()
				.unwrap(),
			signatures: vec![signature.clone().try_into().unwrap()].try_into().unwrap(),
			threshold: 1,
		};

		// should fail for less than threshold
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),
			Error::<Runtime>::NotEnoughSigners
		);

		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_type: DigitalSignatureType::Ecdsa,
			key: pub_key.0.to_vec().try_into().unwrap(),
			participants: vec![pub_key.as_mut().to_vec().try_into().unwrap()].try_into().unwrap(),
			signatures: vec![
				signature.clone().try_into().unwrap(),
				signature.clone().try_into().unwrap(),
			]
			.try_into()
			.unwrap(),
			threshold: 1,
		};

		// should fail for duplicate signers
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),
			Error::<Runtime>::DuplicateSignature
		);

		// works correctly when all params as expected
		let mut participant_one = mock_pub_key_ecdsa();
		let mut participant_two = mock_pub_key_ecdsa();
		let signature_one = mock_signature_ecdsa(participant_one, participant_one);
		let signature_two = mock_signature_ecdsa(participant_two, participant_one);
		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_type: DigitalSignatureType::Ecdsa,
			key: participant_one.to_raw_vec().try_into().unwrap(),
			participants: vec![
				participant_one.as_mut().to_vec().try_into().unwrap(),
				participant_two.as_mut().to_vec().try_into().unwrap(),
			]
			.try_into()
			.unwrap(),
			signatures: vec![signature_two.try_into().unwrap(), signature_one.try_into().unwrap()]
				.try_into()
				.unwrap(),
			threshold: 1,
		};

		assert_ok!(DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),);
	});
}

#[test]
fn dkg_key_verifcation_works_for_ecdsa_when_n_equals_t() {
	new_test_ext().execute_with(|| {
		let mut participant_one = mock_pub_key_ecdsa();
		let mut participant_two = mock_pub_key_ecdsa();
		let signature_one = mock_signature_ecdsa(participant_one, participant_one);
		let signature_two = mock_signature_ecdsa(participant_two, participant_one);
		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_type: DigitalSignatureType::Ecdsa,
			key: participant_one.to_raw_vec().try_into().unwrap(),
			participants: vec![
				participant_one.as_mut().to_vec().try_into().unwrap(),
				participant_two.as_mut().to_vec().try_into().unwrap(),
			]
			.try_into()
			.unwrap(),
			signatures: vec![signature_two.try_into().unwrap(), signature_one.try_into().unwrap()]
				.try_into()
				.unwrap(),
			threshold: 2,
		};

		assert_ok!(DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),);
	});
}

#[test]
fn dkg_key_verifcation_works_for_schnorr() {
	new_test_ext().execute_with(|| {
		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_type: DigitalSignatureType::SchnorrSr25519,
			key: mock_pub_key_sr25519().to_vec().try_into().unwrap(),
			participants: vec![].try_into().unwrap(),
			signatures: vec![].try_into().unwrap(),
			threshold: 2,
		};

		// should fail for empty participants
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),
			Error::<Runtime>::NoParticipantsFound
		);

		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_type: DigitalSignatureType::SchnorrSr25519,
			key: vec![].try_into().unwrap(),
			participants: vec![mock_pub_key_sr25519().as_mut().to_vec().try_into().unwrap()]
				.try_into()
				.unwrap(),
			signatures: vec![].try_into().unwrap(),
			threshold: 2,
		};

		// should fail for empty keys/signatures
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),
			Error::<Runtime>::NoSignaturesFound
		);

		// setup key/signature
		let mut pub_key = mock_pub_key_sr25519();
		let signature = mock_signature_sr25519(pub_key, pub_key);

		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_type: DigitalSignatureType::SchnorrSr25519,
			key: pub_key.to_vec().try_into().unwrap(),
			participants: vec![mock_pub_key_sr25519().as_mut().to_vec().try_into().unwrap()]
				.try_into()
				.unwrap(),
			signatures: vec![signature.clone().try_into().unwrap()].try_into().unwrap(),
			threshold: 1,
		};

		// should fail for less than threshold
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),
			Error::<Runtime>::NotEnoughSigners
		);

		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_type: DigitalSignatureType::SchnorrSr25519,
			key: pub_key.to_vec().try_into().unwrap(),
			participants: vec![pub_key.as_mut().to_vec().try_into().unwrap()].try_into().unwrap(),
			signatures: vec![
				signature.clone().try_into().unwrap(),
				signature.clone().try_into().unwrap(),
			]
			.try_into()
			.unwrap(),
			threshold: 1,
		};

		// should fail for duplicate signers
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),
			Error::<Runtime>::DuplicateSignature
		);

		// works correctly when all params as expected
		let mut participant_one = mock_pub_key_sr25519();
		let mut participant_two = mock_pub_key_sr25519();
		let signature_one = mock_signature_sr25519(participant_one, participant_one);
		let signature_two = mock_signature_sr25519(participant_two, participant_one);
		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_type: DigitalSignatureType::SchnorrSr25519,
			key: participant_one.to_raw_vec().try_into().unwrap(),
			participants: vec![
				participant_one.as_mut().to_vec().try_into().unwrap(),
				participant_two.as_mut().to_vec().try_into().unwrap(),
			]
			.try_into()
			.unwrap(),
			signatures: vec![signature_two.try_into().unwrap(), signature_one.try_into().unwrap()]
				.try_into()
				.unwrap(),
			threshold: 1,
		};

		assert_ok!(DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),);
	});
}

#[test]
fn dkg_key_verifcation_works_for_schnorr_when_n_equals_t() {
	new_test_ext().execute_with(|| {
		let mut participant_one = mock_pub_key_sr25519();
		let mut participant_two = mock_pub_key_sr25519();
		let signature_one = mock_signature_sr25519(participant_one, participant_one);
		let signature_two = mock_signature_sr25519(participant_two, participant_one);
		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_type: DigitalSignatureType::SchnorrSr25519,
			key: participant_one.to_raw_vec().try_into().unwrap(),
			participants: vec![
				participant_one.as_mut().to_vec().try_into().unwrap(),
				participant_two.as_mut().to_vec().try_into().unwrap(),
			]
			.try_into()
			.unwrap(),
			signatures: vec![signature_two.try_into().unwrap(), signature_one.try_into().unwrap()]
				.try_into()
				.unwrap(),
			threshold: 2,
		};

		assert_ok!(DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),);
	});
}

#[test]
fn dkg_signature_verifcation_works_ecdsa() {
	new_test_ext().execute_with(|| {
		// setup key/signature
		let pub_key = mock_pub_key_ecdsa();
		let signature = mock_signature_ecdsa(pub_key, mock_pub_key_ecdsa());

		let job_to_verify = DKGTSSSignatureResult::<MaxDataLen, MaxKeyLen, MaxSignatureLen> {
			signature_type: DigitalSignatureType::Ecdsa,
			signature: signature.try_into().unwrap(),
			data: pub_key.to_raw_vec().try_into().unwrap(),
			signing_key: pub_key.to_raw_vec().try_into().unwrap(),
		};

		// should fail for invalid keys
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseTwo(job_to_verify)),
			Error::<Runtime>::SigningKeyMismatch
		);

		let signature = mock_signature_ecdsa(pub_key, pub_key);
		let job_to_verify = DKGTSSSignatureResult::<MaxDataLen, MaxKeyLen, MaxSignatureLen> {
			signature_type: DigitalSignatureType::Ecdsa,
			signature: signature.try_into().unwrap(),
			data: pub_key.to_raw_vec().try_into().unwrap(),
			signing_key: pub_key.to_raw_vec().try_into().unwrap(),
		};

		// should work with correct params
		assert_ok!(DKG::verify(JobResult::DKGPhaseTwo(job_to_verify)));
	});
}

#[test]
fn dkg_signature_verifcation_works_schnorr() {
	new_test_ext().execute_with(|| {
		// setup key/signature
		let pub_key = mock_pub_key_sr25519();
		let signature = mock_signature_sr25519(pub_key, mock_pub_key_sr25519());

		let job_to_verify = DKGTSSSignatureResult::<MaxDataLen, MaxKeyLen, MaxSignatureLen> {
			signature_type: DigitalSignatureType::SchnorrSr25519,
			signature: signature.try_into().unwrap(),
			data: pub_key.to_raw_vec().try_into().unwrap(),
			signing_key: pub_key.to_raw_vec().try_into().unwrap(),
		};

		// should fail for invalid keys
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseTwo(job_to_verify)),
			Error::<Runtime>::InvalidSignature
		);

		let signature = mock_signature_sr25519(pub_key, pub_key);
		let job_to_verify = DKGTSSSignatureResult {
			signature_type: DigitalSignatureType::SchnorrSr25519,
			signature: signature.try_into().unwrap(),
			data: pub_key.to_raw_vec().try_into().unwrap(),
			signing_key: pub_key.to_raw_vec().try_into().unwrap(),
		};

		// should work with correct params
		assert_ok!(DKG::verify(JobResult::DKGPhaseTwo(job_to_verify)));
	});
}

#[test]
fn dkg_key_rotation_works() {
	new_test_ext().execute_with(|| {
		// setup key/signature
		let curr_key = mock_pub_key_ecdsa();
		let new_key = mock_pub_key_ecdsa();
		let invalid_key = mock_pub_key_ecdsa();
		let signature = mock_signature_ecdsa(invalid_key, new_key);

		let job_to_verify = DKGTSSKeyRotationResult {
			signature_type: DigitalSignatureType::Ecdsa,
			signature: signature.try_into().unwrap(),
			key: curr_key.to_raw_vec().try_into().unwrap(),
			new_key: new_key.to_raw_vec().try_into().unwrap(),
			phase_one_id: 1,
			new_phase_one_id: 2,
		};

		// should fail for invalid keys
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseFour(job_to_verify)),
			Error::<Runtime>::SigningKeyMismatch
		);

		let signature = mock_signature_ecdsa(curr_key, new_key);

		let job_to_verify = DKGTSSKeyRotationResult {
			signature_type: DigitalSignatureType::Ecdsa,
			signature: signature.clone().try_into().unwrap(),
			key: curr_key.to_raw_vec().try_into().unwrap(),
			new_key: new_key.to_raw_vec().try_into().unwrap(),
			phase_one_id: 1,
			new_phase_one_id: 2,
		};
		// should work with correct params
		assert_ok!(DKG::verify(JobResult::DKGPhaseFour(job_to_verify)));
		// should emit KeyRotated event
		assert!(System::events().iter().any(|r| r.event ==
			RuntimeEvent::DKG(Event::KeyRotated {
				from_job_id: 1,
				to_job_id: 2,
				signature: signature.clone()
			})));
	});
}
