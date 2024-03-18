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

use crate::mock::*;

use frame_support::{assert_noop, assert_ok};
use generic_ec::coords::{Coordinate, HasAffineXAndParity, Parity};
use generic_ec::curves::Stark;
use generic_ec::Point;
use p256::ecdsa::signature::hazmat::PrehashSigner;
use p256::ecdsa::{SigningKey, VerifyingKey};
use pallet_dkg::{types::FeeInfo, Error, Event, FeeInfo as FeeInfoStorage};
use parity_scale_codec::Encode;
use rand_core::OsRng;
use sp_core::{crypto::ByteArray, ecdsa, keccak_256, sr25519};
use sp_io::crypto::{ecdsa_generate, ecdsa_sign_prehashed, sr25519_generate, sr25519_sign};
use starknet_crypto::FieldElement;
use tangle_primitives::jobs::{
	DKGTSSKeyRotationResult, DKGTSSKeySubmissionResult, DKGTSSSignatureResult,
	DigitalSignatureScheme, JobResult,
};

fn mock_pub_key_secp256k1_ecdsa() -> ecdsa::Public {
	ecdsa_generate(tangle_crypto_primitives::ROLE_KEY_TYPE, None)
}

fn mock_pub_key_sr25519() -> sr25519::Public {
	sr25519_generate(tangle_crypto_primitives::ROLE_KEY_TYPE, None)
}

fn mock_signature_secp256k1_ecdsa(pub_key: ecdsa::Public, role_key: ecdsa::Public) -> Vec<u8> {
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
			storage_fee_per_block: 0,
		};

		// Dispatch a signed extrinsic.
		assert_ok!(DKG::set_fee(RuntimeOrigin::signed(1), new_fee.clone()));

		assert_eq!(FeeInfoStorage::<Runtime>::get(), new_fee);
	});
}

#[test]
fn dkg_key_verification_works_for_bls() {
	new_test_ext().execute_with(|| {
		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_scheme: DigitalSignatureScheme::Bls381,
			key: vec![].try_into().unwrap(),
			participants: vec![].try_into().unwrap(),
			signatures: vec![].try_into().unwrap(),
			threshold: 2,
		};

		// Should fail for empty participants
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),
			Error::<Runtime>::NoParticipantsFound
		);

		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_scheme: DigitalSignatureScheme::Bls381,
			key: vec![].try_into().unwrap(),
			participants: vec![mock_pub_key_secp256k1_ecdsa()
				.as_mut()
				.to_vec()
				.try_into()
				.unwrap()]
			.try_into()
			.unwrap(),
			signatures: vec![].try_into().unwrap(),
			threshold: 2,
		};

		// Should fail for empty keys/signatures
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseOne(job_to_verify)),
			Error::<Runtime>::NoSignaturesFound
		);

		let mut pub_key = mock_pub_key_secp256k1_ecdsa();
		let signature = mock_signature_secp256k1_ecdsa(pub_key, pub_key);

		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_scheme: DigitalSignatureScheme::Bls381,
			key: vec![].try_into().unwrap(),
			participants: vec![mock_pub_key_secp256k1_ecdsa()
				.as_mut()
				.to_vec()
				.try_into()
				.unwrap()]
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
			signature_scheme: DigitalSignatureScheme::Bls381,
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
		let mut participant_one = mock_pub_key_secp256k1_ecdsa();
		let mut participant_two = mock_pub_key_secp256k1_ecdsa();
		let signature_one = mock_signature_secp256k1_ecdsa(participant_one, participant_one);
		let signature_two = mock_signature_secp256k1_ecdsa(participant_two, participant_one);
		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_scheme: DigitalSignatureScheme::Bls381,
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
fn dkg_key_verification_works_for_ecdsa() {
	new_test_ext().execute_with(|| {
		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_scheme: DigitalSignatureScheme::EcdsaSecp256k1,
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
			signature_scheme: DigitalSignatureScheme::EcdsaSecp256k1,
			key: vec![].try_into().unwrap(),
			participants: vec![mock_pub_key_secp256k1_ecdsa()
				.as_mut()
				.to_vec()
				.try_into()
				.unwrap()]
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

		let mut pub_key = mock_pub_key_secp256k1_ecdsa();
		let signature = mock_signature_secp256k1_ecdsa(pub_key, pub_key);

		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_scheme: DigitalSignatureScheme::EcdsaSecp256k1,
			key: vec![].try_into().unwrap(),
			participants: vec![mock_pub_key_secp256k1_ecdsa()
				.as_mut()
				.to_vec()
				.try_into()
				.unwrap()]
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
			signature_scheme: DigitalSignatureScheme::EcdsaSecp256k1,
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
		let mut participant_one = mock_pub_key_secp256k1_ecdsa();
		let mut participant_two = mock_pub_key_secp256k1_ecdsa();
		let signature_one = mock_signature_secp256k1_ecdsa(participant_one, participant_one);
		let signature_two = mock_signature_secp256k1_ecdsa(participant_two, participant_one);
		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_scheme: DigitalSignatureScheme::EcdsaSecp256k1,
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
		let mut participant_one = mock_pub_key_secp256k1_ecdsa();
		let mut participant_two = mock_pub_key_secp256k1_ecdsa();
		let signature_one = mock_signature_secp256k1_ecdsa(participant_one, participant_one);
		let signature_two = mock_signature_secp256k1_ecdsa(participant_two, participant_one);
		let job_to_verify = DKGTSSKeySubmissionResult {
			signature_scheme: DigitalSignatureScheme::EcdsaSecp256k1,
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
fn signature_verification_works_bls() {
	use snowbridge_milagro_bls::{PublicKey, SecretKey, Signature};

	const BLS_SECRET_KEY: [u8; 32] = [
		25, 192, 46, 5, 150, 93, 249, 180, 243, 38, 104, 158, 250, 226, 60, 6, 248, 5, 232, 52,
		111, 140, 82, 20, 226, 220, 135, 137, 186, 203, 181, 133,
	];

	const BLS_DATA_TO_SIGN: &[u8; 13] = b"Hello, world!";

	new_test_ext().execute_with(|| {
		let secret_key = SecretKey::from_bytes(&BLS_SECRET_KEY).unwrap();
		let pub_key = PublicKey::from_secret_key(&secret_key);
		let message = keccak_256(BLS_DATA_TO_SIGN);
		let signature = Signature::new(&message, &secret_key);

		let job_to_verify: DKGTSSSignatureResult<
			MaxDataLen,
			MaxKeyLen,
			MaxSignatureLen,
			MaxAdditionalParamsLen,
		> = DKGTSSSignatureResult {
			signature_scheme: DigitalSignatureScheme::Bls381,
			derivation_path: None,
			signature: signature.as_bytes().to_vec().try_into().unwrap(),
			verifying_key: pub_key.as_bytes().to_vec().try_into().unwrap(),
			data: BLS_DATA_TO_SIGN.to_vec().try_into().unwrap(),
		};

		// Should fail for an invalid public key
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseTwo(job_to_verify)),
			Error::<Runtime>::InvalidBlsPublicKey
		);

		let job_to_verify: DKGTSSSignatureResult<
			MaxDataLen,
			MaxKeyLen,
			MaxSignatureLen,
			MaxAdditionalParamsLen,
		> = DKGTSSSignatureResult {
			signature_scheme: DigitalSignatureScheme::Bls381,
			derivation_path: None,
			signature: signature.as_bytes()[..10].to_vec().try_into().unwrap(),
			data: BLS_DATA_TO_SIGN.to_vec().try_into().unwrap(),
			verifying_key: pub_key.as_uncompressed_bytes().to_vec().try_into().unwrap(),
		};

		// Should fail for an invalid signature
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseTwo(job_to_verify)),
			Error::<Runtime>::InvalidSignatureData
		);

		let job_to_verify: DKGTSSSignatureResult<
			MaxDataLen,
			MaxKeyLen,
			MaxSignatureLen,
			MaxAdditionalParamsLen,
		> = DKGTSSSignatureResult {
			signature_scheme: DigitalSignatureScheme::Bls381,
			derivation_path: None,
			signature: signature.as_bytes().to_vec().try_into().unwrap(),
			verifying_key: pub_key.as_uncompressed_bytes().to_vec().try_into().unwrap(),
			data: BLS_DATA_TO_SIGN.to_vec().try_into().unwrap(),
		};

		assert_ok!(DKG::verify(JobResult::DKGPhaseTwo(job_to_verify)));
	});
}

#[test]
fn signature_verification_works_secp256k1_ecdsa() {
	new_test_ext().execute_with(|| {
		let pub_key = mock_pub_key_secp256k1_ecdsa();
		let signature = mock_signature_secp256k1_ecdsa(pub_key, mock_pub_key_secp256k1_ecdsa());

		let job_to_verify = DKGTSSSignatureResult::<
			MaxDataLen,
			MaxKeyLen,
			MaxSignatureLen,
			MaxAdditionalParamsLen,
		> {
			signature_scheme: DigitalSignatureScheme::EcdsaSecp256k1,
			derivation_path: None,
			signature: signature.try_into().unwrap(),
			data: pub_key.to_raw_vec().try_into().unwrap(),
			verifying_key: pub_key.to_raw_vec().try_into().unwrap(),
		};

		// should fail for invalid keys
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseTwo(job_to_verify)),
			Error::<Runtime>::InvalidSignature
		);

		let signature = mock_signature_secp256k1_ecdsa(pub_key, pub_key);
		let job_to_verify = DKGTSSSignatureResult::<
			MaxDataLen,
			MaxKeyLen,
			MaxSignatureLen,
			MaxAdditionalParamsLen,
		> {
			signature_scheme: DigitalSignatureScheme::EcdsaSecp256k1,
			derivation_path: None,
			signature: signature.try_into().unwrap(),
			data: pub_key.to_raw_vec().try_into().unwrap(),
			verifying_key: pub_key.to_raw_vec().try_into().unwrap(),
		};

		// should work with correct params
		assert_ok!(DKG::verify(JobResult::DKGPhaseTwo(job_to_verify)));
	});
}

#[test]
fn signature_verification_works_sr25519_schnorr() {
	new_test_ext().execute_with(|| {
		let pub_key = mock_pub_key_sr25519();
		let signature = mock_signature_sr25519(pub_key, mock_pub_key_sr25519());

		let job_to_verify = DKGTSSSignatureResult::<
			MaxDataLen,
			MaxKeyLen,
			MaxSignatureLen,
			MaxAdditionalParamsLen,
		> {
			signature_scheme: DigitalSignatureScheme::SchnorrSr25519,
			derivation_path: None,
			signature: signature.try_into().unwrap(),
			data: pub_key.to_raw_vec().try_into().unwrap(),
			verifying_key: pub_key.to_raw_vec().try_into().unwrap(),
		};

		// should fail for invalid keys
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseTwo(job_to_verify)),
			Error::<Runtime>::InvalidSignature
		);

		let signature = mock_signature_sr25519(pub_key, pub_key);
		let job_to_verify = DKGTSSSignatureResult {
			signature_scheme: DigitalSignatureScheme::SchnorrSr25519,
			derivation_path: None,
			signature: signature.try_into().unwrap(),
			data: pub_key.to_raw_vec().try_into().unwrap(),
			verifying_key: pub_key.to_raw_vec().try_into().unwrap(),
		};

		// should work with correct params
		assert_ok!(DKG::verify(JobResult::DKGPhaseTwo(job_to_verify)));
	});
}

#[test]
fn signature_verification_works_secp256r1_ecdsa() {
	new_test_ext().execute_with(|| {
		let mut rng = OsRng;
		let secret_key = SigningKey::random(&mut rng);
		let public_key = VerifyingKey::from(&secret_key);
		let message = b"hello world";
		let prehash = keccak_256(message);
		let (signature, _) = secret_key.sign_prehash(&prehash).unwrap();

		let job_to_verify = DKGTSSSignatureResult {
			signature_scheme: DigitalSignatureScheme::EcdsaSecp256r1,
			derivation_path: None,
			signature: signature.to_vec().try_into().unwrap(),
			data: message.to_vec().try_into().unwrap(),
			verifying_key: public_key.to_sec1_bytes().to_vec().try_into().unwrap(),
		};

		// should work with correct params
		assert_ok!(DKG::verify(JobResult::DKGPhaseTwo(job_to_verify)));
	});
}

pub fn field_element_from_be_hex(hex: &str) -> FieldElement {
	let decoded = hex::decode(hex.trim_start_matches("0x")).unwrap();

	if decoded.len() > 32 {
		panic!("hex string too long");
	}

	let mut buffer = [0u8; 32];
	buffer[(32 - decoded.len())..].copy_from_slice(&decoded[..]);

	FieldElement::from_bytes_be(&buffer).unwrap()
}

#[test]
fn signature_verification_works_stark_ecdsa() {
	new_test_ext().execute_with(|| {
		let private_key = field_element_from_be_hex(
			"0000000000000000000000000000000000000000000000000000000000000001",
		);
		let message = field_element_from_be_hex(
			"0000000000000000000000000000000000000000000000000000000000000002",
		);
		let k = field_element_from_be_hex(
			"0000000000000000000000000000000000000000000000000000000000000003",
		);
		let signature = starknet_crypto::sign(&private_key, &message, &k).unwrap();
		let public_key = starknet_crypto::get_public_key(&private_key);
		let public_key_point: Point<Stark> = Point::from_x_and_parity(
			&Coordinate::from_be_bytes(&public_key.to_bytes_be()).unwrap(),
			Parity::Odd,
		)
		.unwrap();
		let mut encoded_signature: [u8; 64] = [0u8; 64];
		encoded_signature[..32].copy_from_slice(&signature.r.to_bytes_be());
		encoded_signature[32..].copy_from_slice(&signature.s.to_bytes_be());

		let job_to_verify = DKGTSSSignatureResult {
			signature_scheme: DigitalSignatureScheme::EcdsaStark,
			derivation_path: None,
			signature: encoded_signature.to_vec().try_into().unwrap(),
			data: message.to_bytes_be().to_vec().try_into().unwrap(),
			verifying_key: public_key_point.to_bytes(true).to_vec().try_into().unwrap(),
		};

		// should work with correct params
		assert_ok!(DKG::verify(JobResult::DKGPhaseTwo(job_to_verify)));
	});
}

#[test]
fn dkg_key_rotation_works() {
	new_test_ext().execute_with(|| {
		let curr_key = mock_pub_key_secp256k1_ecdsa();
		let new_key = mock_pub_key_secp256k1_ecdsa();
		let invalid_key = mock_pub_key_secp256k1_ecdsa();
		let signature = mock_signature_secp256k1_ecdsa(invalid_key, new_key);

		let job_to_verify = DKGTSSKeyRotationResult {
			signature_scheme: DigitalSignatureScheme::EcdsaSecp256k1,
			derivation_path: None,
			signature: signature.try_into().unwrap(),
			key: curr_key.to_raw_vec().try_into().unwrap(),
			new_key: new_key.to_raw_vec().try_into().unwrap(),
			phase_one_id: 1,
			new_phase_one_id: 2,
		};

		// should fail for invalid keys
		assert_noop!(
			DKG::verify(JobResult::DKGPhaseFour(job_to_verify)),
			Error::<Runtime>::InvalidSignature
		);

		let signature = mock_signature_secp256k1_ecdsa(curr_key, new_key);

		let job_to_verify = DKGTSSKeyRotationResult {
			signature_scheme: DigitalSignatureScheme::EcdsaSecp256k1,
			derivation_path: None,
			signature: signature.clone().try_into().unwrap(),
			key: curr_key.to_raw_vec().try_into().unwrap(),
			new_key: new_key.to_raw_vec().try_into().unwrap(),
			phase_one_id: 1,
			new_phase_one_id: 2,
		};
		// should work with correct params
		assert_ok!(DKG::verify(JobResult::DKGPhaseFour(job_to_verify)));
		// should emit KeyRotated event
		assert!(System::events().iter().any(|r| r.event
			== RuntimeEvent::DKG(Event::KeyRotated {
				from_job_id: 1,
				to_job_id: 2,
				signature: signature.clone()
			})));
	});
}
