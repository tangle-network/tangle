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
use sha2::Digest;

use frame_support::{assert_err, assert_ok};
use generic_ec::{curves::Secp256k1, Scalar, SecretScalar};
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

fn sign_round_msg<M: serde::Serialize>(
	key: ecdsa::Public,
	sender: u16,
	msg: &M,
) -> SignedRoundMessage {
	let msg_bytes = bincode2::serialize(msg).unwrap();
	let sender_bytes = sender.to_be_bytes();
	let msg_to_sign = [&sender_bytes[..], &msg_bytes[..]].concat();
	let signature = sign(key, &msg_to_sign);
	SignedRoundMessage { sender, message: msg_bytes, signature }
}

#[test]
fn submit_keygen_decommitment_should_work() {
	new_test_ext().execute_with(|| {
		let i = 2_u16;
		let participants = (0..5).map(|_| pub_key()).collect::<Vec<_>>();
		let t = 3_u16;
		let offender = participants[usize::from(i)];
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

		let round1_signed_msg = sign_round_msg(offender, i, &my_commitment);
		let round2a_signed_msg = sign_round_msg(offender, i, &my_decommitment);

		let submission = MisbehaviorSubmission {
			role_type: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
			offender: offender.0,
			job_id,
			justification: MisbehaviorJustification::DKGTSS(DKGTSSJustification::DfnsCGGMP21(
				DfnsCGGMP21Justification::Keygen {
					participants: participants.iter().map(|p| p.0).collect(),
					t,
					reason: KeygenAborted::InvalidDecommitment {
						round1: round1_signed_msg,
						round2a: round2a_signed_msg,
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
		let i = 2_u16;
		let participants = (0..5).map(|_| pub_key()).collect::<Vec<_>>();
		let t = 3_u16;
		let offender = participants[usize::from(i)];
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

		let round1_signed_msg = sign_round_msg(offender, i, &my_commitment);
		let round2a_signed_msg = sign_round_msg(offender, i, &my_decommitment2);

		let submission = MisbehaviorSubmission {
			role_type: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
			offender: offender.0,
			job_id,
			justification: MisbehaviorJustification::DKGTSS(DKGTSSJustification::DfnsCGGMP21(
				DfnsCGGMP21Justification::Keygen {
					participants: participants.iter().map(|p| p.0).collect(),
					t,
					reason: KeygenAborted::InvalidDecommitment {
						round1: round1_signed_msg,
						round2a: round2a_signed_msg,
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
		let i = 2_u16;
		let participants = (0..5).map(|_| pub_key()).collect::<Vec<_>>();
		let t = 3_u16;
		let offender = participants[usize::from(i)];
		let job_id = 1_u64;
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
		let round2a_signed_msg = sign_round_msg(offender, i, &my_decommitment);

		let submission = MisbehaviorSubmission {
			role_type: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
			offender: offender.0,
			job_id,
			justification: MisbehaviorJustification::DKGTSS(DKGTSSJustification::DfnsCGGMP21(
				DfnsCGGMP21Justification::Keygen {
					participants: participants.iter().map(|p| p.0).collect(),
					t,
					reason: KeygenAborted::InvalidDataSize { round2a: round2a_signed_msg },
				},
			)),
		};

		assert_err!(DfnsCGGMP21::verify(submission), crate::Error::<Runtime>::ValidDataSize);
	});
}

#[test]
fn submit_keygen_invalid_decommitment_data_size_should_work() {
	new_test_ext().execute_with(|| {
		let i = 2_u16;
		let participants = (0..5).map(|_| pub_key()).collect::<Vec<_>>();
		let t = 3_u16;
		let offender = participants[usize::from(i)];
		let job_id = 1_u64;
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
		let round2a_signed_msg = sign_round_msg(offender, i, &my_decommitment);

		let submission = MisbehaviorSubmission {
			role_type: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
			offender: offender.0,
			job_id,
			justification: MisbehaviorJustification::DKGTSS(DKGTSSJustification::DfnsCGGMP21(
				DfnsCGGMP21Justification::Keygen {
					participants: participants.iter().map(|p| p.0).collect(),
					t: t + 1,
					reason: KeygenAborted::InvalidDataSize { round2a: round2a_signed_msg },
				},
			)),
		};

		assert_ok!(DfnsCGGMP21::verify(submission));
	});
}

#[test]
fn submit_keygen_feldman_verification_should_work() {
	new_test_ext().execute_with(|| {
		let i = 2_u16;
		let participants = (0..5).map(|_| pub_key()).collect::<Vec<_>>();
		let t = 3_u16;
		let offender = participants[usize::from(i)];
		let job_id = 1_u64;
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

		let my_sigma: keygen::msg::threshold::MsgRound2Uni<Secp256k1> =
			keygen::msg::threshold::MsgRound2Uni {
				sigma: {
					let x = Scalar::from(i + 1);
					f.value(&x)
				},
			};

		let round2a_signed_msg = sign_round_msg(offender, i, &my_decommitment);
		let round2b_signed_msg = sign_round_msg(offender, i, &my_sigma);

		let submission = MisbehaviorSubmission {
			role_type: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
			offender: offender.0,
			job_id,
			justification: MisbehaviorJustification::DKGTSS(DKGTSSJustification::DfnsCGGMP21(
				DfnsCGGMP21Justification::Keygen {
					participants: participants.iter().map(|p| p.0).collect(),
					t,
					reason: KeygenAborted::FeldmanVerificationFailed {
						round2a: round2a_signed_msg,
						round2b: round2b_signed_msg,
					},
				},
			)),
		};

		assert_err!(
			DfnsCGGMP21::verify(submission),
			crate::Error::<Runtime>::ValidFeldmanVerification
		);
	});
}

#[test]
fn submit_keygen_invalid_feldman_verification_should_work() {
	new_test_ext().execute_with(|| {
		let i = 2_u16;
		let participants = (0..5).map(|_| pub_key()).collect::<Vec<_>>();
		let t = 3_u16;
		let offender = participants[usize::from(i)];
		let job_id = 1_u64;
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

		let my_sigma: keygen::msg::threshold::MsgRound2Uni<Secp256k1> =
			keygen::msg::threshold::MsgRound2Uni {
				sigma: {
					// invalid value
					let x = Scalar::from(i + 1 + 5);
					f.value(&x)
				},
			};

		let round2a_signed_msg = sign_round_msg(offender, i, &my_decommitment);
		let round2b_signed_msg = sign_round_msg(offender, i, &my_sigma);

		let submission = MisbehaviorSubmission {
			role_type: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
			offender: offender.0,
			job_id,
			justification: MisbehaviorJustification::DKGTSS(DKGTSSJustification::DfnsCGGMP21(
				DfnsCGGMP21Justification::Keygen {
					participants: participants.iter().map(|p| p.0).collect(),
					t,
					reason: KeygenAborted::FeldmanVerificationFailed {
						round2a: round2a_signed_msg,
						round2b: round2b_signed_msg,
					},
				},
			)),
		};

		assert_ok!(DfnsCGGMP21::verify(submission));
	});
}

#[test]
fn submit_keygen_schnorr_proof_verification_should_work() {
	new_test_ext().execute_with(|| {
		let i = 2_u16;
		let participants = (0..5).map(|_| pub_key()).collect::<Vec<_>>();
		let n = participants.len() as u16;
		let t = 3_u16;
		let offender = participants[usize::from(i)];
		let job_id = 1_u64;
		let job_id_bytes = job_id.to_be_bytes();
		let mix = keccak_256(b"dnfs-cggmp21-keygen");
		let eid_bytes = [&job_id_bytes[..], &mix[..]].concat();
		let rng = &mut rand_chacha::ChaChaRng::from_seed(mix);

		let fp = (0..n)
			.map(|_| Polynomial::<SecretScalar<Secp256k1>>::sample(rng, usize::from(t) - 1))
			.collect::<Vec<_>>();
		let round2a_msgs = (0..n)
			.map(|j| {
				let mut rid = <SecurityLevel128 as SecurityLevel>::Rid::default();
				rng.fill_bytes(rid.as_mut());
				let (r, h) = schnorr_pok::prover_commits_ephemeral_secret::<Secp256k1, _>(rng);
				let f = &fp[usize::from(j)];
				let F = f * &Point::generator();
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
				(r, my_decommitment)
			})
			.collect::<Vec<_>>();

		let sigmas = (0..n)
			.map(|j| {
				let f = &fp[usize::from(j)];
				f.value(&Scalar::from(i + 1))
			})
			.collect::<Vec<Scalar<Secp256k1>>>();
		debug_assert_eq!(sigmas.len(), usize::from(n));

		let (r, my_decommitment) = &round2a_msgs[usize::from(i)];

		let rid = round2a_msgs
			.iter()
			.map(|(_, d)| &d.rid)
			.fold(<SecurityLevel128 as SecurityLevel>::Rid::default(), DfnsCGGMP21::xor_array);

		let polynomial_sum =
			round2a_msgs.iter().map(|(_, d)| &d.F).sum::<Polynomial<Point<Secp256k1>>>();

		let ys = (0..n)
			.map(|l| polynomial_sum.value(&Scalar::from(l + 1)))
			.collect::<Vec<Point<Secp256k1>>>();

		let mut sigma: Scalar<Secp256k1> = sigmas.iter().sum();
		let sigma = SecretScalar::new(&mut sigma);
		debug_assert_eq!(Point::generator() * &sigma, ys[usize::from(i)]);

		let challenge = {
			let hash = |d: DefaultDigest| {
				d.chain_update(&eid_bytes)
					.chain_update(i.to_be_bytes())
					.chain_update(rid.as_ref())
					.chain_update(ys[usize::from(i)].to_bytes(true)) // y_i
					.chain_update(my_decommitment.sch_commit.0.to_bytes(false)) // h
					.finalize()
			};
			let mut rng = paillier_zk::rng::HashRng::new(hash);
			Scalar::random(&mut rng)
		};
		let challenge = schnorr_pok::Challenge { nonce: challenge };

		let z = schnorr_pok::prove(r, &challenge, sigma);

		let round3_msg = keygen::msg::threshold::MsgRound3 { sch_proof: z };
		let round3_signed_msg = sign_round_msg(offender, i, &round3_msg);

		let signed_round2a_msgs = (0..n)
			.zip(participants.iter())
			.zip(round2a_msgs.iter())
			.map(|((i, key), (_, msg))| sign_round_msg(*key, i, msg))
			.collect::<Vec<_>>();

		let submission = MisbehaviorSubmission {
			role_type: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
			offender: offender.0,
			job_id,
			justification: MisbehaviorJustification::DKGTSS(DKGTSSJustification::DfnsCGGMP21(
				DfnsCGGMP21Justification::Keygen {
					participants: participants.iter().map(|p| p.0).collect(),
					t,
					reason: KeygenAborted::InvalidSchnorrProof {
						round2a: signed_round2a_msgs,
						round3: round3_signed_msg,
					},
				},
			)),
		};

		assert_err!(DfnsCGGMP21::verify(submission), crate::Error::<Runtime>::ValidSchnorrProof);
	});
}

#[test]
fn submit_keygen_invalid_schnorr_proof_verification_should_work() {
	new_test_ext().execute_with(|| {
		let i = 2_u16;
		let participants = (0..5).map(|_| pub_key()).collect::<Vec<_>>();
		let n = participants.len() as u16;
		let t = 3_u16;
		let offender = participants[usize::from(i)];
		let job_id = 1_u64;
		let job_id_bytes = job_id.to_be_bytes();
		let mix = keccak_256(b"dnfs-cggmp21-keygen");
		let eid_bytes = [&job_id_bytes[..], &mix[..]].concat();
		let rng = &mut rand_chacha::ChaChaRng::from_seed(mix);

		let fp = (0..n)
			.map(|_| Polynomial::<SecretScalar<Secp256k1>>::sample(rng, usize::from(t) - 1))
			.collect::<Vec<_>>();
		let round2a_msgs = (0..n)
			.map(|j| {
				let mut rid = <SecurityLevel128 as SecurityLevel>::Rid::default();
				rng.fill_bytes(rid.as_mut());
				let (r, h) = schnorr_pok::prover_commits_ephemeral_secret::<Secp256k1, _>(rng);
				let f = &fp[usize::from(j)];
				let F = f * &Point::generator();
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
				(r, my_decommitment)
			})
			.collect::<Vec<_>>();

		let sigmas = (0..n)
			.map(|j| {
				let f = &fp[usize::from(j)];
				f.value(&Scalar::from(i + 1))
			})
			.collect::<Vec<Scalar<Secp256k1>>>();
		debug_assert_eq!(sigmas.len(), usize::from(n));

		let (r, my_decommitment) = &round2a_msgs[usize::from(i)];

		let rid = round2a_msgs
			.iter()
			.map(|(_, d)| &d.rid)
			.fold(<SecurityLevel128 as SecurityLevel>::Rid::default(), DfnsCGGMP21::xor_array);

		let polynomial_sum =
			round2a_msgs.iter().map(|(_, d)| &d.F).sum::<Polynomial<Point<Secp256k1>>>();

		let ys = (0..n)
			.map(|l| polynomial_sum.value(&Scalar::from(l + 1)))
			.collect::<Vec<Point<Secp256k1>>>();

		let mut sigma: Scalar<Secp256k1> = sigmas.iter().sum();
		let sigma = SecretScalar::new(&mut sigma);
		debug_assert_eq!(Point::generator() * &sigma, ys[usize::from(i)]);

		let challenge = {
			let hash = |d: DefaultDigest| {
				d.chain_update(&eid_bytes)
					// commented intuentially to make the proof invalid
					// .chain_update(i.to_be_bytes())
					.chain_update(rid.as_ref())
					.chain_update(ys[usize::from(i)].to_bytes(true)) // y_i
					.chain_update(my_decommitment.sch_commit.0.to_bytes(false)) // h
					.finalize()
			};
			let mut rng = paillier_zk::rng::HashRng::new(hash);
			Scalar::random(&mut rng)
		};
		let challenge = schnorr_pok::Challenge { nonce: challenge };

		let z = schnorr_pok::prove(r, &challenge, sigma);

		let round3_msg = keygen::msg::threshold::MsgRound3 { sch_proof: z };
		let round3_signed_msg = sign_round_msg(offender, i, &round3_msg);

		let signed_round2a_msgs = (0..n)
			.zip(participants.iter())
			.zip(round2a_msgs.iter())
			.map(|((i, key), (_, msg))| sign_round_msg(*key, i, msg))
			.collect::<Vec<_>>();

		let submission = MisbehaviorSubmission {
			role_type: RoleType::Tss(ThresholdSignatureRoleType::DfnsCGGMP21Secp256k1),
			offender: offender.0,
			job_id,
			justification: MisbehaviorJustification::DKGTSS(DKGTSSJustification::DfnsCGGMP21(
				DfnsCGGMP21Justification::Keygen {
					participants: participants.iter().map(|p| p.0).collect(),
					t,
					reason: KeygenAborted::InvalidSchnorrProof {
						round2a: signed_round2a_msgs,
						round3: round3_signed_msg,
					},
				},
			)),
		};

		assert_ok!(DfnsCGGMP21::verify(submission));
	});
}
