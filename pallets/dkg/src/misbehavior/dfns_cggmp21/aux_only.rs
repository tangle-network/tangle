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

use super::{zk::ring_pedersen_parameters as π_prm, DefaultDigest, Integer, SECURITY_BYTES};

#[derive(udigest::Digestable)]
#[udigest(tag = "dfns.cggmp21.aux_gen.tag")]
pub enum Tag<'a> {
	/// Tag that includes the prover index
	Indexed {
		party_index: u16,
		#[udigest(as_bytes)]
		sid: &'a [u8],
	},
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
	#[serde(with = "super::integer::serde")]
	pub N: Integer,
	/// $s_i$
	#[udigest(with = super::integer::encoding::integer)]
	#[serde(with = "super::integer::serde")]
	pub s: Integer,
	/// $t_i$
	#[udigest(with = super::integer::encoding::integer)]
	#[serde(with = "super::integer::serde")]
	pub t: Integer,
	/// $\hat \psi_i$
	pub params_proof: super::zk::ring_pedersen_parameters::Proof,
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

/// Given a KeyRefresh Round2 message, verify the misbehavior and return the result.
pub fn invalid_ring_pedersen_parameters<T: Config>(
	data: &MisbehaviorSubmission,
	round2: &SignedRoundMessage,
) -> DispatchResult {
	Pallet::<T>::ensure_signed_by_offender(round2, data.offender)?;
	let i = round2.sender;
	let job_id_bytes = data.job_id.to_be_bytes();
	let mix = keccak_256(AUX_GEN_EID);
	let eid_bytes = [&job_id_bytes[..], &mix[..]].concat();
	let parties_shared_state = DefaultDigest::new_with_prefix(DefaultDigest::digest(eid_bytes));
	let round2_msg = postcard::from_bytes::<MsgRound2>(&round2.message)
		.map_err(|_| Error::<T>::MalformedRoundMessage)?;
	if !super::validate_public_paillier_key_size(&round2_msg.N) {
		// Slash the offender!
		// TODO: add slashing logic
	}

	let data = π_prm::Data { N: &round2_msg.N, s: &round2_msg.s, t: &round2_msg.t };
	let proof = π_prm::verify(
		parties_shared_state.clone().chain_update(i.to_be_bytes()),
		data,
		&round2_msg.params_proof,
	);

	ensure!(proof.is_err(), Error::<T>::ValidRingPedersenParameters);

	// Slash the offender!
	// TODO: add slashing logic
	Ok(())
}
