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
#![allow(non_snake_case)]

use crate::{
	mock::*,
	types::{DefaultDigest, Tag},
};
use dfns_cggmp21::{
	generic_ec::Point,
	keygen,
	security_level::{SecurityLevel, SecurityLevel128},
};

use frame_support::{assert_err, assert_ok};
use generic_ec::{curves::Secp256k1, SecretScalar};
use generic_ec_zkp::{polynomial::Polynomial, schnorr_pok};
use parity_scale_codec::Encode;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use sp_core::{ecdsa, keccak_256};
use sp_io::crypto::{ecdsa_generate, ecdsa_sign_prehashed};
use tangle_primitives::{
	misbehavior::{
		dfns_cggmp21::{DfnsCGGMP21Justification, KeygenAborted, SignedRoundMessage},
		DKGTSSJustification, MisbehaviorJustification, MisbehaviorSubmission,
	},
	roles::{RoleType, ThresholdSignatureRoleType},
};

fn pub_key() -> ecdsa::Public {
	ecdsa_generate(tangle_crypto_primitives::ROLE_KEY_TYPE, None)
}

fn sign(key: ecdsa::Public, msg: &[u8]) -> Vec<u8> {
	let hash = keccak_256(msg);
	let signature: ecdsa::Signature =
		ecdsa_sign_prehashed(tangle_crypto_primitives::ROLE_KEY_TYPE, &key, &hash).unwrap();
	signature.encode()
}

#[test]
fn submit_keygen_decommitment_should_work() {
	new_test_ext().execute_with(|| {
		let pub_key = pub_key();
		let offender = pub_key.0;
		let i = 2_u16;
		let n = 5_u16;
		let t = 3_u16;
		let job_id = 1_u64;
		let job_id_bytes = job_id.to_be_bytes();
		let mix = keccak_256(b"dnfs-cggmp21-keygen");
		let eid_bytes = [&job_id_bytes[..], &mix[..]].concat();
		let rng = &mut rand_chacha::ChaChaRng::from_seed(mix);
		let tag = udigest::Tag::<DefaultDigest>::new_structured(Tag::Indexed {
			party_index: i,
			sid: &eid_bytes[..],
		});

		let mut rid = <SecurityLevel128 as SecurityLevel>::Rid::default();
		rng.fill_bytes(rid.as_mut());

		let (_r, h) = schnorr_pok::prover_commits_ephemeral_secret::<Secp256k1, _>(rng);

		let f = Polynomial::<SecretScalar<Secp256k1>>::sample(rng, usize::from(t) - 1);
		let F = &f * &Point::generator();
		let my_decommitment: keygen::msg::threshold::MsgRound2Broad<_, SecurityLevel128> =
			keygen::msg::threshold::MsgRound2Broad {
				rid,
				F,
				sch_commit: h,
				decommit: {
					let mut nonce = <SecurityLevel128 as SecurityLevel>::Rid::default();
					rng.fill_bytes(nonce.as_mut());
					nonce
				},
			};
		let hash_commit = tag.digest(&my_decommitment);

		let my_commitment: keygen::msg::threshold::MsgRound1<DefaultDigest> =
			keygen::msg::threshold::MsgRound1 { commitment: hash_commit };

		let round1_msg = bincode2::serialize(&my_commitment).unwrap();
		let round2_msg = bincode2::serialize(&my_decommitment).unwrap();

		let msg_to_sign = [&i.to_be_bytes()[..], &round1_msg[..]].concat();
		let signature = sign(pub_key, &msg_to_sign);
		let round1_signed_message =
			SignedRoundMessage { sender: i, message: round1_msg, signature };

		let msg_to_sign = [&i.to_be_bytes()[..], &round2_msg[..]].concat();
		let signature = sign(pub_key, &msg_to_sign);
		let round2_signed_message =
			SignedRoundMessage { sender: i, message: round2_msg, signature };

		let submission = MisbehaviorSubmission {
			role_type: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
			offender,
			job_id,
			justification: MisbehaviorJustification::DKGTSS(DKGTSSJustification::DfnsCGGMP21(
				DfnsCGGMP21Justification::Keygen {
					n,
					t,
					reason: KeygenAborted::InvalidDecommitment {
						round1: round1_signed_message,
						round2a: round2_signed_message,
					},
				},
			)),
		};

		assert_err!(DfnsCGGMP21::verify(submission), crate::Error::<Runtime>::ValidDecommitment);
	});
}

#[test]
fn submit_keygen_invalid_decommitment_should_work() {
	new_test_ext().execute_with(|| {
		let pub_key = pub_key();
		let offender = pub_key.0;
		let i = 2_u16;
		let n = 5_u16;
		let t = 3_u16;
		let job_id = 1_u64;
		let job_id_bytes = job_id.to_be_bytes();
		let mix = keccak_256(b"dnfs-cggmp21-keygen");
		let eid_bytes = [&job_id_bytes[..], &mix[..]].concat();
		let rng = &mut rand_chacha::ChaChaRng::from_seed(mix);
		let tag = udigest::Tag::<DefaultDigest>::new_structured(Tag::Indexed {
			party_index: i,
			sid: &eid_bytes[..],
		});

		let mut rid = <SecurityLevel128 as SecurityLevel>::Rid::default();
		rng.fill_bytes(rid.as_mut());

		let (_r, h) = schnorr_pok::prover_commits_ephemeral_secret::<Secp256k1, _>(rng);

		let f = Polynomial::<SecretScalar<Secp256k1>>::sample(rng, usize::from(t) - 1);
		let F = &f * &Point::generator();
		let my_decommitment: keygen::msg::threshold::MsgRound2Broad<_, SecurityLevel128> =
			keygen::msg::threshold::MsgRound2Broad {
				rid: rid.clone(),
				F: F.clone(),
				sch_commit: h.clone(),
				decommit: {
					let mut nonce = <SecurityLevel128 as SecurityLevel>::Rid::default();
					rng.fill_bytes(nonce.as_mut());
					nonce
				},
			};
		let hash_commit = tag.digest(my_decommitment);

		let my_commitment: keygen::msg::threshold::MsgRound1<DefaultDigest> =
			keygen::msg::threshold::MsgRound1 { commitment: hash_commit };

		// invalid decommitment
		let my_decommitment2: keygen::msg::threshold::MsgRound2Broad<_, SecurityLevel128> =
			keygen::msg::threshold::MsgRound2Broad {
				rid,
				F,
				sch_commit: h,
				decommit: Default::default(),
			};
		let round1_msg = bincode2::serialize(&my_commitment).unwrap();
		let round2_msg = bincode2::serialize(&my_decommitment2).unwrap();

		let msg_to_sign = [&i.to_be_bytes()[..], &round1_msg[..]].concat();
		let signature = sign(pub_key, &msg_to_sign);
		let round1_signed_message =
			SignedRoundMessage { sender: i, message: round1_msg, signature };

		let msg_to_sign = [&i.to_be_bytes()[..], &round2_msg[..]].concat();
		let signature = sign(pub_key, &msg_to_sign);
		let round2_signed_message =
			SignedRoundMessage { sender: i, message: round2_msg, signature };

		let submission = MisbehaviorSubmission {
			role_type: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
			offender,
			job_id,
			justification: MisbehaviorJustification::DKGTSS(DKGTSSJustification::DfnsCGGMP21(
				DfnsCGGMP21Justification::Keygen {
					n,
					t,
					reason: KeygenAborted::InvalidDecommitment {
						round1: round1_signed_message,
						round2a: round2_signed_message,
					},
				},
			)),
		};

		assert_ok!(DfnsCGGMP21::verify(submission));
	});
}

#[test]
fn submit_keygen_decommitment_data_size_should_work() {
	new_test_ext().execute_with(|| {
		let pub_key = pub_key();
		let offender = pub_key.0;
		let job_id = 1_u64;
		let i = 2_u16;
		let n = 5_u16;
		let t = 3_u16;
		let rng = &mut rand_chacha::ChaChaRng::from_seed([42; 32]);

		let mut rid = <SecurityLevel128 as SecurityLevel>::Rid::default();
		rng.fill_bytes(rid.as_mut());

		let (_r, h) = schnorr_pok::prover_commits_ephemeral_secret::<Secp256k1, _>(rng);

		let f = Polynomial::<SecretScalar<Secp256k1>>::sample(rng, usize::from(t) - 1);
		let F = &f * &Point::generator();
		let my_decommitment: keygen::msg::threshold::MsgRound2Broad<_, SecurityLevel128> =
			keygen::msg::threshold::MsgRound2Broad {
				rid,
				F,
				sch_commit: h,
				decommit: {
					let mut nonce = <SecurityLevel128 as SecurityLevel>::Rid::default();
					rng.fill_bytes(nonce.as_mut());
					nonce
				},
			};
		let round2_msg = bincode2::serialize(&my_decommitment).unwrap();

		let msg_to_sign = [&i.to_be_bytes()[..], &round2_msg[..]].concat();
		let signature = sign(pub_key, &msg_to_sign);
		let round2_signed_message =
			SignedRoundMessage { sender: i, message: round2_msg, signature };

		let submission = MisbehaviorSubmission {
			role_type: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
			offender,
			job_id,
			justification: MisbehaviorJustification::DKGTSS(DKGTSSJustification::DfnsCGGMP21(
				DfnsCGGMP21Justification::Keygen {
					n,
					t,
					reason: KeygenAborted::InvalidDataSize { round2a: round2_signed_message },
				},
			)),
		};

		assert_err!(DfnsCGGMP21::verify(submission), crate::Error::<Runtime>::ValidDataSize);
	});
}

#[test]
fn submit_keygen_invalid_decommitment_data_size_should_work() {
	new_test_ext().execute_with(|| {
		let pub_key = pub_key();
		let offender = pub_key.0;
		let job_id = 1_u64;
		let i = 2_u16;
		let n = 5_u16;
		let t = 3_u16;
		let rng = &mut rand_chacha::ChaChaRng::from_seed([42; 32]);

		let mut rid = <SecurityLevel128 as SecurityLevel>::Rid::default();
		rng.fill_bytes(rid.as_mut());

		let (_r, h) = schnorr_pok::prover_commits_ephemeral_secret::<Secp256k1, _>(rng);

		let f = Polynomial::<SecretScalar<Secp256k1>>::sample(rng, usize::from(t) - 1);
		let F = &f * &Point::generator();
		let my_decommitment: keygen::msg::threshold::MsgRound2Broad<_, SecurityLevel128> =
			keygen::msg::threshold::MsgRound2Broad {
				rid,
				F,
				sch_commit: h,
				decommit: {
					let mut nonce = <SecurityLevel128 as SecurityLevel>::Rid::default();
					rng.fill_bytes(nonce.as_mut());
					nonce
				},
			};
		let round2_msg = bincode2::serialize(&my_decommitment).unwrap();

		let msg_to_sign = [&i.to_be_bytes()[..], &round2_msg[..]].concat();
		let signature = sign(pub_key, &msg_to_sign);
		let round2_signed_message =
			SignedRoundMessage { sender: i, message: round2_msg, signature };

		let submission = MisbehaviorSubmission {
			role_type: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
			offender,
			job_id,
			justification: MisbehaviorJustification::DKGTSS(DKGTSSJustification::DfnsCGGMP21(
				DfnsCGGMP21Justification::Keygen {
					n,
					t: t + 1,
					reason: KeygenAborted::InvalidDataSize { round2a: round2_signed_message },
				},
			)),
		};

		assert_ok!(DfnsCGGMP21::verify(submission));
	});
}
