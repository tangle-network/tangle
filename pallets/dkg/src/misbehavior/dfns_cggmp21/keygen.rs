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

use crate::*;
use digest::Digest;
use frame_support::ensure;
use generic_ec::{curves::Secp256k1, Curve, Point, Scalar};
use generic_ec_zkp::{polynomial::Polynomial, schnorr_pok};
use sp_core::RuntimeDebug;
use sp_io::hashing::keccak_256;
use sp_runtime::DispatchResult;
use sp_std::prelude::*;

use tangle_primitives::misbehavior::{
	dfns_cggmp21::KEYGEN_EID, MisbehaviorSubmission, SignedRoundMessage,
};

use super::{hashing_rng::HashRng, xor_array, DefaultDigest, SECURITY_BYTES};

#[derive(udigest::Digestable)]
#[udigest(tag = "dfns.cggmp21.keygen.threshold.tag")]
pub enum Tag<'a> {
	/// Tag that includes the prover index
	Indexed {
		party_index: u16,
		#[udigest(as_bytes)]
		sid: &'a [u8],
	},
}

#[derive(Clone)]
pub struct Rid<const N: usize>([u8; N]);

impl<const N: usize> AsRef<[u8]> for Rid<N> {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}

impl<const N: usize> AsMut<[u8]> for Rid<N> {
	fn as_mut(&mut self) -> &mut [u8] {
		&mut self.0
	}
}

impl<const N: usize> Default for Rid<N> {
	fn default() -> Self {
		Self([0u8; N])
	}
}

impl<const N: usize> hex::FromHex for Rid<N>
where
	[u8; N]: hex::FromHex,
{
	type Error = <[u8; N] as hex::FromHex>::Error;
	fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
		hex::FromHex::from_hex(hex).map(Self)
	}
}

/// Message from round 1
#[derive(Clone, RuntimeDebug, serde::Deserialize, udigest::Digestable)]
#[serde(bound = "")]
#[udigest(bound = "")]
#[udigest(tag = "dfns.cggmp21.keygen.threshold.round1")]
pub struct MsgRound1<D: Digest> {
	/// $V_i$
	#[udigest(as_bytes)]
	pub commitment: digest::Output<D>,
}

/// Message from round 2 broadcasted to everyone
#[serde_with::serde_as]
#[derive(Clone, serde::Deserialize, udigest::Digestable)]
#[serde(bound = "")]
#[udigest(bound = "")]
#[udigest(tag = "dfns.cggmp21.keygen.threshold.round1")]
#[allow(non_snake_case)]
pub struct MsgRound2Broad<E: Curve> {
	/// `rid_i`
	#[serde_as(as = "super::hex_or_bin::HexOrBin")]
	#[udigest(as_bytes)]
	pub rid: Rid<{ SECURITY_BYTES }>,
	/// $\vec S_i$
	pub F: Polynomial<Point<E>>,
	/// $A_i$
	pub sch_commit: schnorr_pok::Commit<E>,
	/// $u_i$
	#[serde(with = "hex::serde")]
	#[udigest(as_bytes)]
	pub decommit: [u8; SECURITY_BYTES],
}

/// Message from round 2 unicasted to each party
#[derive(Clone, RuntimeDebug, serde::Deserialize)]
#[serde(bound = "")]
pub struct MsgRound2Uni<E: Curve> {
	/// $\sigma_{i,j}$
	pub sigma: Scalar<E>,
}
/// Message from round 3
#[derive(Clone, serde::Deserialize)]
#[serde(bound = "")]
pub struct MsgRound3<E: Curve> {
	/// $\psi_i$
	pub sch_proof: schnorr_pok::Proof<E>,
}

/// Given a Keygen Round1 and Round2a messages, verify the misbehavior and return the result.
pub fn invalid_decommitment<T: Config>(
	data: &MisbehaviorSubmission,
	round1: &SignedRoundMessage,
	round2a: &SignedRoundMessage,
) -> DispatchResult {
	Pallet::<T>::ensure_signed_by_offender(round1, data.offender)?;
	Pallet::<T>::ensure_signed_by_offender(round2a, data.offender)?;
	ensure!(round1.sender == round2a.sender, Error::<T>::InvalidJustification);

	let job_id_bytes = data.job_id.to_be_bytes();
	let mix = keccak_256(KEYGEN_EID);
	let eid_bytes = [&job_id_bytes[..], &mix[..]].concat();
	let tag = udigest::Tag::<DefaultDigest>::new_structured(Tag::Indexed {
		party_index: round1.sender,
		sid: &eid_bytes[..],
	});

	let round1_msg = postcard::from_bytes::<MsgRound1<DefaultDigest>>(&round1.message)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;

	let round2_msg = postcard::from_bytes::<MsgRound2Broad<Secp256k1>>(&round2a.message)
		.map_err(|e| Error::<T>::MalformedRoundMessage)?;
	let hash_commit = tag.digest(round2_msg);

	ensure!(round1_msg.commitment != hash_commit, Error::<T>::ValidDecommitment);
	// Slash the offender!
	// TODO: add slashing logic
	Ok(())
}

/// Given a Keygen t and Round2a messages, verify the misbehavior and return the result.
pub fn invalid_data_size<T: Config>(
	data: &MisbehaviorSubmission,
	t: u16,
	round2a: &SignedRoundMessage,
) -> DispatchResult {
	Pallet::<T>::ensure_signed_by_offender(round2a, data.offender)?;

	let round2a_msg = postcard::from_bytes::<MsgRound2Broad<Secp256k1>>(&round2a.message)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;

	ensure!(round2a_msg.F.degree() + 1 != usize::from(t), Error::<T>::ValidDataSize);
	// Slash the offender!
	// TODO: add slashing logic
	Ok(())
}

/// Given a Keygen Round2a and Round2b messages, verify the misbehavior and return the result.
pub fn feldman<T: Config>(
	data: &MisbehaviorSubmission,
	round2a: &SignedRoundMessage,
	round2b: &SignedRoundMessage,
) -> DispatchResult {
	Pallet::<T>::ensure_signed_by_offender(round2a, data.offender)?;
	Pallet::<T>::ensure_signed_by_offender(round2b, data.offender)?;
	ensure!(round2a.sender == round2b.sender, Error::<T>::InvalidJustification);
	let i = round2a.sender;

	let round2a_msg = postcard::from_bytes::<MsgRound2Broad<Secp256k1>>(&round2a.message)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;

	let round2b_msg = postcard::from_bytes::<MsgRound2Uni<Secp256k1>>(&round2b.message)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;

	let lhs = round2a_msg.F.value::<_, generic_ec::Point<_>>(&Scalar::from(i + 1));
	let rhs = generic_ec::Point::generator() * round2b_msg.sigma;
	let feldman_verification = lhs != rhs;
	ensure!(feldman_verification, Error::<T>::ValidFeldmanVerification);
	// Slash the offender!
	// TODO: add slashing logic
	Ok(())
}

pub fn schnorr_proof<T: Config>(
	data: &MisbehaviorSubmission,
	parties_including_offender: &[[u8; 33]],
	round2a: &[SignedRoundMessage],
	round3: &SignedRoundMessage,
) -> DispatchResult {
	let i = round3.sender;
	let n = parties_including_offender.len() as u16;
	Pallet::<T>::ensure_signed_by_offender(round3, data.offender)?;
	ensure!(round2a.len() == usize::from(n), Error::<T>::InvalidJustification);
	round2a
		.iter()
		.zip(parties_including_offender)
		.try_for_each(|(r, p)| Pallet::<T>::ensure_signed_by(r, *p))?;

	let decomm = round2a.get(usize::from(i)).ok_or(Error::<T>::InvalidJustification)?;
	// double-check
	Pallet::<T>::ensure_signed_by_offender(decomm, data.offender)?;

	let job_id_bytes = data.job_id.to_be_bytes();
	let mix = keccak_256(KEYGEN_EID);
	let eid_bytes = [&job_id_bytes[..], &mix[..]].concat();

	let round3_msg = postcard::from_bytes::<MsgRound3<Secp256k1>>(&round3.message)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;

	let round2a_msgs = round2a
		.iter()
		.map(|r| {
			postcard::from_bytes::<MsgRound2Broad<Secp256k1>>(&r.message)
				.map_err(|_| Error::<T>::MalformedRoundMessage)
		})
		.collect::<Result<Vec<_>, _>>()?;
	let round2a_msg = round2a_msgs.get(usize::from(i)).ok_or(Error::<T>::InvalidJustification)?;

	let rid = round2a_msgs.iter().map(|d| &d.rid).fold([0u8; SECURITY_BYTES], xor_array);

	let polynomial_sum = round2a_msgs.iter().map(|d| &d.F).sum::<Polynomial<Point<Secp256k1>>>();

	let ys = (0..n)
		.map(|l| polynomial_sum.value(&Scalar::from(l + 1)))
		.collect::<Vec<Point<Secp256k1>>>();

	let challenge = {
		let hash = |d: DefaultDigest| {
			d.chain_update(&eid_bytes)
				.chain_update(i.to_be_bytes())
				.chain_update(rid.as_slice())
				.chain_update(ys[usize::from(i)].to_bytes(true)) // y_i
				.chain_update(round2a_msg.sch_commit.0.to_bytes(false)) // h
				.finalize()
		};
		let mut rng = HashRng::new(hash);
		Scalar::random(&mut rng)
	};
	let challenge = schnorr_pok::Challenge { nonce: challenge };

	let proof =
		round3_msg
			.sch_proof
			.verify(&round2a_msg.sch_commit, &challenge, &ys[usize::from(i)]);

	ensure!(proof.is_err(), Error::<T>::ValidSchnorrProof);

	// TODO: add slashing logic
	// Slash the offender!
	Ok(())
}
