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
use super::*;
use frame_support::traits::Currency;
use malachite::{
	num::{basic::traits::Zero, conversion::traits::FromStringBase},
	strings::ToLowerHexString,
};
use sp_core::RuntimeDebug;

pub type DefaultDigest = sha2::Sha256;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Hardcoded value for parameter $m$ of security level
///
/// Currently, [security parameter $m$](SecurityLevel::M) is hardcoded to this constant. We're going
/// to fix that once `feature(generic_const_exprs)` is stable.
pub const M: usize = 128;
pub const SECURITY_BITS: usize = 384;
pub const SECURITY_BYTES: usize = SECURITY_BITS / 8;
pub const EPSILON: usize = 230;
pub const ELL: usize = 2 * M;
pub const ELL_PRIME: usize = 848;

#[derive(Clone, RuntimeDebug, serde::Serialize, serde::Deserialize)]
pub struct Integer {
	pub radix: i32,
	pub value: String,
}

impl Integer {
	pub fn to_vec(&self) -> Vec<u8> {
		let v = malachite::Integer::from_string_base(self.radix as u8, &self.value).unwrap();
		// special case for zero
		if v == malachite::Integer::ZERO {
			return Vec::new()
		}
		let mut x = v.to_lower_hex_string();
		// fix odd length
		if x.len() % 2 != 0 {
			// add a leading zero
			x = format!("0{}", x);
		}

		let mut out = vec![0; x.len() / 2];
		hex::decode_to_slice(x, &mut out).unwrap();
		out
	}
}

/// Unambiguous encoding for different types for which it was not defined
pub mod encoding {
	pub fn integer<B: udigest::Buffer>(
		x: &super::Integer,
		encoder: udigest::encoding::EncodeValue<B>,
	) {
		encoder.encode_leaf().chain(x.to_vec());
	}

	pub fn integers_list<B: udigest::Buffer>(
		list: &[super::Integer],
		encoder: udigest::encoding::EncodeValue<B>,
	) {
		let mut encoder = encoder.encode_list();
		for x in list {
			integer(x, encoder.add_item())
		}
	}
}

pub mod keygen {
	use digest::Digest;
	use generic_ec::{Curve, Point, Scalar};
	use generic_ec_zkp::{polynomial::Polynomial, schnorr_pok};
	use sp_core::RuntimeDebug;

	#[derive(udigest::Digestable)]
	#[udigest(tag = "dfns.cggmp21.keygen.threshold.tag")]
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
	#[derive(Clone, serde::Deserialize, udigest::Digestable)]
	#[serde(bound = "")]
	#[udigest(bound = "")]
	#[udigest(tag = "dfns.cggmp21.keygen.threshold.round1")]
	#[allow(non_snake_case)]
	pub struct MsgRound2Broad<E: Curve> {
		/// `rid_i`
		#[serde(with = "hex::serde")]
		#[udigest(as_bytes)]
		pub rid: Vec<u8>,
		/// $\vec S_i$
		pub F: Polynomial<Point<E>>,
		/// $A_i$
		pub sch_commit: schnorr_pok::Commit<E>,
		/// $u_i$
		#[serde(with = "hex::serde")]
		#[udigest(as_bytes)]
		pub decommit: Vec<u8>,
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
}

pub mod aux_only {
	use digest::Digest;
	use sp_core::RuntimeDebug;

	use super::Integer;

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
	#[derive(Clone, RuntimeDebug, serde::Serialize, serde::Deserialize, udigest::Digestable)]
	pub struct ParamProof<const M: usize> {
		#[serde_as(as = "[_; M]")]
		#[udigest(with = super::encoding::integers_list)]
		pub commitment: [Integer; M],
		#[serde_as(as = "[_; M]")]
		#[udigest(with = super::encoding::integers_list)]
		pub zs: [Integer; M],
	}

	#[derive(Clone, RuntimeDebug, serde::Deserialize)]
	pub struct ProofPoint {
		pub x: super::Integer,
		pub a: bool,
		pub b: bool,
		pub z: super::Integer,
	}

	/// Message from round 1
	#[derive(Clone, RuntimeDebug, serde::Serialize, serde::Deserialize, udigest::Digestable)]
	#[udigest(tag = "dfns.cggmp21.aux_gen.round1")]
	#[udigest(bound = "")]
	#[serde(bound = "")]
	pub struct MsgRound1<D: Digest> {
		/// $V_i$
		#[udigest(as_bytes)]
		pub commitment: digest::Output<D>,
	}
	/// Message from round 2
	#[derive(Clone, RuntimeDebug, serde::Serialize, serde::Deserialize, udigest::Digestable)]
	#[udigest(tag = "dfns.cggmp21.aux_gen.round2")]
	#[udigest(bound = "")]
	#[serde(bound = "")]
	#[allow(non_snake_case)]
	pub struct MsgRound2<const M: usize> {
		/// $N_i$
		#[udigest(with = super::encoding::integer)]
		pub N: Integer,
		/// $s_i$
		#[udigest(with = super::encoding::integer)]
		pub s: Integer,
		/// $t_i$
		#[udigest(with = super::encoding::integer)]
		pub t: Integer,
		/// $\hat \psi_i$
		pub params_proof: ParamProof<M>,
		/// $\rho_i$
		#[serde(with = "hex")]
		#[udigest(as_bytes)]
		pub rho_bytes: Vec<u8>,
		/// $u_i$
		#[serde(with = "hex")]
		#[udigest(as_bytes)]
		pub decommit: Vec<u8>,
	}
}
