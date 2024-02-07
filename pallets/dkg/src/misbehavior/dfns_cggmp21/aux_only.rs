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
use sp_core::RuntimeDebug;
use sp_io::hashing::keccak_256;
use sp_runtime::DispatchResult;
use sp_std::prelude::*;
use tangle_primitives::misbehavior::{
	dfns_cggmp21::{SignedRoundMessage, AUX_GEN_EID},
	MisbehaviorSubmission,
};

use super::{DefaultDigest, Integer, M, SECURITY_BYTES};

#[derive(udigest::Digestable)]
#[udigest(tag = "dfns.cggmp21.aux_gen.tag")]
pub enum Tag<'a> {
	/// Tag that includes the prover index
	Indexed {
		party_index: u16,
		#[udigest(as_bytes)]
		sid: &'a [u8],
	},
	/// Tag w/o party index
	Unindexed {
		#[udigest(as_bytes)]
		sid: &'a [u8],
	},
}

#[derive(Clone, RuntimeDebug, serde::Deserialize)]
pub struct Proof<const M: usize> {
	#[serde(with = "serde_with::As::<[serde_with::Same; M]>")]
	pub points: [ProofPoint; M],
}

/// The ZK proof. Computed by [`prove`].
///
/// Parameter `M` is security level. The probability of an adversary generating
/// a correct proof for incorrect data is $2^{-M}$. You can use M defined here
/// as [`SECURITY`]
#[serde_with::serde_as]
#[derive(Clone, RuntimeDebug, serde::Deserialize, udigest::Digestable)]
pub struct ParamProof<const M: usize> {
	#[serde_as(as = "[_; M]")]
	#[udigest(with = super::integer::encoding::integers_list)]
	pub commitment: [Integer; M],
	#[serde_as(as = "[_; M]")]
	#[udigest(with = super::integer::encoding::integers_list)]
	pub zs: [Integer; M],
}

#[derive(Clone, RuntimeDebug, serde::Deserialize)]
pub struct ProofPoint {
	pub x: Integer,
	pub a: bool,
	pub b: bool,
	pub z: Integer,
}

/// Message from round 1
#[derive(Clone, RuntimeDebug, serde::Deserialize, udigest::Digestable)]
#[udigest(tag = "dfns.cggmp21.aux_gen.round1")]
#[udigest(bound = "")]
#[serde(bound = "")]
pub struct MsgRound1<D: Digest> {
	/// $V_i$
	#[udigest(as_bytes)]
	pub commitment: digest::Output<D>,
}
/// Message from round 2
#[derive(Clone, RuntimeDebug, serde::Deserialize, udigest::Digestable)]
#[udigest(tag = "dfns.cggmp21.aux_gen.round2")]
#[udigest(bound = "")]
#[serde(bound = "")]
#[allow(non_snake_case)]
pub struct MsgRound2 {
	/// $N_i$
	#[udigest(with = super::integer::encoding::integer)]
	pub N: Integer,
	/// $s_i$
	#[udigest(with = super::integer::encoding::integer)]
	pub s: Integer,
	/// $t_i$
	#[udigest(with = super::integer::encoding::integer)]
	pub t: Integer,
	/// $\hat \psi_i$
	pub params_proof: ParamProof<M>,
	/// $\rho_i$
	#[serde(with = "hex")]
	#[udigest(as_bytes)]
	pub rho_bytes: [u8; SECURITY_BYTES],
	/// $u_i$
	#[serde(with = "hex")]
	#[udigest(as_bytes)]
	pub decommit: [u8; SECURITY_BYTES],
}

/// Given a KeyRefresh Round1 and Round2 messages, verify the misbehavior and return the result.
pub fn invalid_decommitment<T: Config>(
	data: &MisbehaviorSubmission,
	round1: &SignedRoundMessage,
	round2: &SignedRoundMessage,
) -> DispatchResult {
	Pallet::<T>::ensure_signed_by_offender(round1, data.offender)?;
	Pallet::<T>::ensure_signed_by_offender(round2, data.offender)?;
	ensure!(round1.sender == round2.sender, Error::<T>::InvalidJustification);

	let job_id_bytes = data.job_id.to_be_bytes();
	let mix = keccak_256(AUX_GEN_EID);
	let eid_bytes = [&job_id_bytes[..], &mix[..]].concat();
	let tag = udigest::Tag::<DefaultDigest>::new_structured(Tag::Indexed {
		party_index: round1.sender,
		sid: &eid_bytes[..],
	});

	let round1_msg = postcard::from_bytes::<MsgRound1<DefaultDigest>>(&round1.message)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;

	let round2_msg = postcard::from_bytes::<MsgRound2>(&round2.message)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;

	let hash_commit = tag.digest(round2_msg);

	ensure!(round1_msg.commitment != hash_commit, Error::<T>::ValidDecommitment);
	// Slash the offender!
	// TODO: add slashing logic
	Ok(())
}
