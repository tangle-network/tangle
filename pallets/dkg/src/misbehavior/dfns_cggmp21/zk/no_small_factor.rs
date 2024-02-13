//! ZK-proof for factoring of a RSA modulus. Called Пfac or Rfac in the CGGMP21
//! paper.
//!
//! ## Description
//!
//! A party P has a modulus `N = pq`. P wants to prove to a verifier V that p
//! and q are sufficiently large without disclosing p or q, with p and q each no
//! larger than `sqrt(N) * 2^l`, or equivalently no smaller than `sqrt(N) /
//! 2^l`
use crate::misbehavior::dfns_cggmp21::{integer, Integer};

use malachite_base::num::arithmetic::traits::{Mod, ModPow, UnsignedAbs};
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;

use super::paillier_blum_modulus::random_naturals_less_than;

/// Security parameters for proof. Choosing the values is a tradeoff between
/// speed and chance of rejecting a valid proof or accepting an invalid proof
#[derive(RuntimeDebug, Clone, Serialize, Deserialize)]
pub struct SecurityParams {
	/// l in paper, security parameter for bit size of plaintext: it needs to
	/// differ from sqrt(n) not more than by 2^l
	pub l: usize,
	/// Epsilon in paper, slackness parameter
	pub epsilon: usize,
	/// q in paper. Security parameter for challenge
	#[serde(with = "integer::serde")]
	pub q: Integer,
}

/// Public data that both parties know
#[derive(Debug, Clone, Copy)]
pub struct Data<'a> {
	/// N0 - rsa modulus
	pub n: &'a Integer,
	/// A number close to square root of n
	pub n_root: &'a Integer,
}

/// Private data of prover
#[derive(Debug, Clone, Copy)]
pub struct PrivateData<'a> {
	pub p: &'a Integer,
	pub q: &'a Integer,
}

/// Prover's data accompanying the commitment. Kept as state between rounds in
/// the interactive protocol.
#[derive(RuntimeDebug, Clone, Serialize, Deserialize)]
pub struct PrivateCommitment {
	#[serde(with = "integer::serde")]
	pub alpha: Integer,
	#[serde(with = "integer::serde")]
	pub beta: Integer,
	#[serde(with = "integer::serde")]
	pub mu: Integer,
	#[serde(with = "integer::serde")]
	pub nu: Integer,
	#[serde(with = "integer::serde")]
	pub r: Integer,
	#[serde(with = "integer::serde")]
	pub x: Integer,
	#[serde(with = "integer::serde")]
	pub y: Integer,
}

/// Prover's first message, obtained by [`interactive::commit`]
#[derive(RuntimeDebug, Clone, Serialize, Deserialize)]
pub struct Commitment {
	#[serde(with = "integer::serde")]
	pub p: Integer,
	#[serde(with = "integer::serde")]
	pub q: Integer,
	#[serde(with = "integer::serde")]
	pub a: Integer,
	#[serde(with = "integer::serde")]
	pub b: Integer,
	#[serde(with = "integer::serde")]
	pub t: Integer,
	#[serde(with = "integer::serde")]
	pub sigma: Integer,
}

/// Verifier's challenge to prover. Can be obtained deterministically by
/// [`non_interactive::challenge`] or randomly by [`interactive::challenge`]
pub type Challenge = Integer;

/// The ZK proof, computed by [`interactive::prove`]
#[derive(RuntimeDebug, Clone, Serialize, Deserialize)]
pub struct Proof {
	#[serde(with = "integer::serde")]
	pub z1: Integer,
	#[serde(with = "integer::serde")]
	pub z2: Integer,
	#[serde(with = "integer::serde")]
	pub w1: Integer,
	#[serde(with = "integer::serde")]
	pub w2: Integer,
	#[serde(with = "integer::serde")]
	pub v: Integer,
}

/// Witness that proof is invalid
#[derive(Debug)]
pub struct InvalidProof;

fn from_rng_pm<R: rand_core::RngCore>(range: &Integer, rng: &mut R) -> Integer {
	let range_twice = range.clone() << 1u32;
	let x = random_naturals_less_than(rng, range_twice.clone().unsigned_abs())
		.next()
		.unwrap();
	Integer::from(x) - range_twice
}

fn is_in_pm(x: &Integer, range: &Integer) -> bool {
	let minus_range = -range.clone();
	minus_range <= *x && x <= range
}

/// Returns `Err(err)` if `assertion` is false
pub fn fail_if<E>(err: E, assertion: bool) -> Result<(), E> {
	if assertion {
		Ok(())
	} else {
		Err(err)
	}
}

/// Returns `Err(err)` if `lhs != rhs`
pub fn fail_if_ne<T: PartialEq, E>(err: E, lhs: T, rhs: T) -> Result<(), E> {
	if lhs == rhs {
		Ok(())
	} else {
		Err(err)
	}
}

fn combine(x: &Integer, l: &Integer, le: &Integer, r: &Integer, re: &Integer) -> Integer {
	let l_to_le: Integer =
		l.unsigned_abs_ref().mod_pow(le.unsigned_abs_ref(), x.unsigned_abs()).into();
	let r_to_re: Integer =
		r.unsigned_abs_ref().mod_pow(re.unsigned_abs_ref(), x.unsigned_abs()).into();
	(l_to_le * r_to_re).mod_op(x)
}

/// Auxiliary data known to both prover and verifier
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Aux {
	/// ring-pedersen parameter
	#[serde(with = "integer::serde")]
	pub s: Integer,
	/// ring-pedersen parameter
	#[serde(with = "integer::serde")]
	pub t: Integer,
	/// N^ in paper
	#[serde(with = "integer::serde")]
	pub rsa_modulo: Integer,
}

impl Aux {
	/// Returns `s^x t^y mod rsa_modulo`
	pub fn combine(&self, x: &Integer, y: &Integer) -> Integer {
		combine(&self.rsa_modulo, &self.s, x, &self.t, y)
	}

	/// Returns `x^e mod rsa_modulo`
	pub fn pow_mod(&self, x: &Integer, e: &Integer) -> Integer {
		x.unsigned_abs_ref()
			.mod_pow(e.unsigned_abs_ref(), self.rsa_modulo.clone().unsigned_abs())
			.into()
	}
}

/// Interactive version of the proof
pub mod interactive {
	use malachite_base::num::arithmetic::traits::Mod;
	use rand_core::RngCore;

	use super::{
		combine, fail_if, fail_if_ne, Aux, Challenge, Commitment, Data, InvalidProof, Proof,
		SecurityParams,
	};

	use crate::misbehavior::dfns_cggmp21::Integer;

	/// Generate random challenge
	///
	/// `security` parameter is used to generate challenge in correct range
	pub fn challenge<R: RngCore>(security: &SecurityParams, rng: &mut R) -> Challenge {
		super::from_rng_pm(&security.q, rng)
	}

	/// Verify the proof
	pub fn verify(
		aux: &Aux,
		data: Data,
		commitment: &Commitment,
		security: &SecurityParams,
		challenge: &Challenge,
		proof: &Proof,
	) -> Result<(), InvalidProof> {
		// check 1
		{
			let lhs = aux.combine(&proof.z1, &proof.w1);
			let p_to_e = aux.pow_mod(&commitment.p, challenge);
			let rhs = (&commitment.a * p_to_e).mod_op(&aux.rsa_modulo);
			fail_if_ne(InvalidProof, lhs, rhs)?;
		}
		// check 2
		{
			let lhs = aux.combine(&proof.z2, &proof.w2);
			let q_to_e = aux.pow_mod(&commitment.q, challenge);
			let rhs = (&commitment.b * q_to_e).mod_op(&aux.rsa_modulo);
			fail_if_ne(InvalidProof, lhs, rhs)?;
		}
		// check 3
		{
			let r = aux.combine(data.n, &commitment.sigma);
			let q_to_z1 = aux.pow_mod(&commitment.q, &proof.z1);
			let t_to_v = aux.pow_mod(&aux.t, &proof.v);
			let lhs = (q_to_z1 * t_to_v).mod_op(&aux.rsa_modulo);
			let rhs = combine(&aux.rsa_modulo, &commitment.t, &Integer::from(1), &r, challenge);
			fail_if_ne(InvalidProof, lhs, rhs)?;
		}
		let range = (Integer::from(1) << (security.l + security.epsilon)) * data.n_root;
		// range check for z1
		fail_if(InvalidProof, super::is_in_pm(&proof.z1, &range))?;
		// range check for z2
		fail_if(InvalidProof, super::is_in_pm(&proof.z2, &range))?;

		Ok(())
	}
}

/// Non-interactive version of the proof
pub mod non_interactive {
	use digest::{typenum::U32, Digest};
	use sp_core::RuntimeDebug;

	use super::InvalidProof;
	pub use super::{Aux, Challenge, Data, PrivateData, SecurityParams};
	use crate::misbehavior::dfns_cggmp21::{hashing_rng::HashRng, integer::RugInteger};

	/// The ZK proof, computed by [`prove`]
	#[derive(RuntimeDebug, Clone, serde::Serialize, serde::Deserialize)]
	pub struct Proof {
		commitment: super::Commitment,
		proof: super::Proof,
	}

	/// Deterministically compute challenge based on prior known values in protocol
	pub fn challenge<D>(
		shared_state: D,
		aux: &Aux,
		data: Data,
		commitment: &super::Commitment,
		security: &SecurityParams,
	) -> Challenge
	where
		D: Digest,
	{
		let shared_state = shared_state.finalize();
		let hash = |d: D| {
			d.chain_update(&shared_state)
				.chain_update(RugInteger::from(&aux.s).to_vec())
				.chain_update(RugInteger::from(&aux.t).to_vec())
				.chain_update(RugInteger::from(&aux.rsa_modulo).to_vec())
				.chain_update(RugInteger::from(data.n).to_vec())
				.chain_update(RugInteger::from(data.n_root).to_vec())
				.chain_update(RugInteger::from(&commitment.p).to_vec())
				.chain_update(RugInteger::from(&commitment.q).to_vec())
				.chain_update(RugInteger::from(&commitment.a).to_vec())
				.chain_update(RugInteger::from(&commitment.b).to_vec())
				.chain_update(RugInteger::from(&commitment.t).to_vec())
				.chain_update(RugInteger::from(&commitment.sigma).to_vec())
				.finalize()
		};
		let mut rng = HashRng::new(hash);
		super::interactive::challenge(security, &mut rng)
	}

	/// Verify the proof, deriving challenge independently from same data
	pub fn verify<D>(
		shared_state: D,
		aux: &Aux,
		data: Data,
		security: &SecurityParams,
		proof: &Proof,
	) -> Result<(), InvalidProof>
	where
		D: Digest<OutputSize = U32>,
	{
		let challenge = challenge(shared_state, aux, data, &proof.commitment, security);
		super::interactive::verify(aux, data, &proof.commitment, security, &challenge, &proof.proof)
	}
}
