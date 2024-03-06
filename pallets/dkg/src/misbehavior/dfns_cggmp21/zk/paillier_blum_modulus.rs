//! ZK-proof of Paillier-Blum modulus. Called Пmod or Rmod in the CGGMP21 paper.
//!
//! ## Description
//! A party P has a modulus `N = pq`, with p and q being Blum primes, and
//! `gcd(N, phi(N)) = 1`. P wants to prove that those equalities about N hold,
//! without disclosing p and q.

#[cfg(not(feature = "std"))]
use ::alloc::vec::Vec;

use crate::misbehavior::dfns_cggmp21::{
	hashing_rng::HashRng,
	integer::{self, RugInteger},
	Integer, M,
};

use digest::Digest;
use malachite_base::num::{
	arithmetic::traits::{
		CeilingLogBase2, Mod, ModPow, ModPowerOf2Assign, Parity, ShrRound, UnsignedAbs,
	},
	basic::{integers::PrimitiveInt, traits::Zero},
	conversion::traits::ExactFrom,
};
use malachite_nz::{natural::Natural, platform::Limb};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;

/// Public data that both parties know: the Paillier-Blum modulus
#[derive(RuntimeDebug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Data {
	/// N
	#[serde(with = "integer::serde")]
	pub n: Integer,
}

/// Prover's first message, obtained by [`interactive::commit`]
#[derive(RuntimeDebug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Commitment {
	#[serde(with = "integer::serde")]
	pub w: Integer,
}

/// A part of proof. Having enough of those guarantees security
#[derive(RuntimeDebug, Clone, Serialize, Deserialize)]
pub struct ProofPoint {
	#[serde(with = "integer::serde")]
	pub x: Integer,
	pub a: bool,
	pub b: bool,
	#[serde(with = "integer::serde")]
	pub z: Integer,
}

/// The ZK proof.
/// Consists of M proofs for each challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
	#[serde(with = "serde_with::As::<[serde_with::Same; M]>")]
	pub points: [ProofPoint; M],
}

/// Verify that `n` is prime.
pub fn verify_n_is_prime(data: &Data) -> bool {
	// N should not be prime.
	// TODO: check if this is correct.
	todo!("check if N = {} is prime or not.", data.n)
}

pub fn verify_n_is_even(data: &Data) -> bool {
	data.n.even()
}

pub fn verify_incorrect_nth_root<D>(
	i: usize,
	shared_state: D,
	data: &Data,
	proof: &Proof,
	commitment: &Commitment,
) -> bool
where
	D: Digest,
{
	// bounds check
	if i >= M {
		return false;
	}
	let ys = challenge_up_to(i, shared_state, data, commitment);
	let n = data.n.unsigned_abs_ref();
	let point = &proof.points[i];
	let y = &ys[i];
	point.z.unsigned_abs_ref().mod_pow(n, n).ne(y)
}

pub fn verify_incorrect_fourth_root<D>(
	i: usize,
	shared_state: D,
	data: &Data,
	proof: &Proof,
	commitment: &Commitment,
) -> bool
where
	D: Digest,
{
	// bounds check
	if i >= M {
		return false;
	}
	let ys = challenge_up_to(i, shared_state, data, commitment);
	let n = data.n.unsigned_abs_ref();
	let point = &proof.points[i];
	let y = ys[i].clone();
	let y = if point.a { &data.n - y } else { y };
	let y = if point.b { (y * &commitment.w).mod_op(&data.n) } else { y };
	point.x.unsigned_abs_ref().mod_pow(&Natural::from(4u32), n).ne(&y)
}

/// Deterministically compute challenge based on prior known values in protocol
pub fn challenge_up_to<D>(
	i: usize,
	shared_state: D,
	Data { ref n }: &Data,
	commitment: &Commitment,
) -> Vec<Integer>
where
	D: Digest,
{
	let shared_state = shared_state.finalize();
	let hash = |d: D| {
		d.chain_update(&shared_state)
			.chain_update(RugInteger::from(n).to_vec())
			.chain_update(RugInteger::from(&commitment.w).to_vec())
			.finalize()
	};
	let mut rng = HashRng::new(hash);
	random_naturals_less_than(&mut rng, n.unsigned_abs())
		.take(i)
		.map(Integer::from)
		.collect::<Vec<_>>()
}

/// Uniformly generates random [`Natural`]s less than a positive limit.
#[derive(Debug)]
pub struct RandomNaturalsLessThan<'a, R: RngCore> {
	bits: u64,
	limit: Natural,
	rng: &'a mut R,
}

impl<'a, R: RngCore> Iterator for RandomNaturalsLessThan<'a, R> {
	type Item = Natural;

	fn next(&mut self) -> Option<Natural> {
		loop {
			let x = get_random_natural_with_up_to_bits(&mut self.rng, self.bits);
			if x < self.limit {
				return Some(x);
			}
		}
	}
}

/// Uniformly generates random [`Natural`]s less than a positive `limit`.
///
/// $$
/// P(x) = \\begin{cases}
///     \frac{1}{\\ell} & \text{if} \\quad x < \\ell, \\\\
///     0 & \\text{otherwise}.
/// \\end{cases}
/// $$
/// where $\ell$ is `limit`.
///
/// The output length is infinite.
///
/// # Expected complexity per iteration
/// $T(n) = O(n)$
///
/// $M(n) = O(n)$
///
/// where $T$ is time, $M$ is additional memory, and $n$ is `limit.significant_bits()`.
///
/// # Panics
/// Panics if `limit` is 0.
pub fn random_naturals_less_than<R: RngCore>(
	rng: &mut R,
	limit: Natural,
) -> RandomNaturalsLessThan<'_, R> {
	assert_ne!(limit, 0);
	RandomNaturalsLessThan { bits: limit.ceiling_log_base_2(), limit, rng }
}

/// Generates a random [`Natural`] with a given maximum bit length.
///
/// The [`Natural`] is chosen uniformly from $[0, 2^b)$; [`Natural`]s with bit lengths smaller than
/// the maximum may also be generated.
///
/// $$
/// P(n) = \\begin{cases}
///     \frac{1}{2^b} & \text{if} \\quad 0 \\leq n < 2^b, \\\\
///     0 & \\text{otherwise}.
/// \\end{cases}
/// $$
///
/// # Expected complexity
/// $T(n) = O(n)$
///
/// $M(n) = O(n)$
///
/// where $T$ is time, $M$ is additional memory, and `n` is `bits`.
pub fn get_random_natural_with_up_to_bits<R: RngCore>(rng: &mut R, bits: u64) -> Natural {
	if bits == 0 {
		return Natural::ZERO;
	}
	let l = usize::exact_from(
		bits.shr_round(Limb::LOG_WIDTH, malachite_base::rounding_modes::RoundingMode::Ceiling)
			.0,
	);
	let mut xs = Vec::with_capacity(l);

	for _ in 0..l {
		xs.push(rng.next_u32());
	}
	limbs_slice_mod_power_of_2_in_place(&mut xs, bits);
	Natural::from_owned_limbs_asc(xs)
}

// Interpreting a slice of `Limb`s as the limbs (in ascending order) of a `Natural`, writes the
// limbs of the `Natural` mod two raised to `pow` to the input slice. Equivalently, retains only the
// least-significant `pow` bits. If the upper limbs of the input slice are no longer needed, they
// are set to zero.
//
// # Worst-case complexity
// Constant time and additional memory.
//
// This is equivalent to `mpz_tdiv_r_2exp` from `mpz/tdiv_r_2exp.c`, GMP 6.2.1, where `in` is
// non-negative, `res == in`, and instead of possibly being truncated, the high limbs of `res` are
// possibly filled with zeros.
fn limbs_slice_mod_power_of_2_in_place(xs: &mut [Limb], pow: u64) {
	if pow == 0 {
		malachite_base::slices::slice_set_zero(xs);
		return;
	}
	let new_size = usize::exact_from(
		pow.shr_round(Limb::LOG_WIDTH, malachite_base::rounding_modes::RoundingMode::Ceiling)
			.0,
	);
	if new_size > xs.len() {
		return;
	}
	malachite_base::slices::slice_set_zero(&mut xs[new_size..]);
	let leftover_bits = pow & Limb::WIDTH_MASK;
	if leftover_bits != 0 {
		xs[new_size - 1].mod_power_of_2_assign(leftover_bits);
	}
}
