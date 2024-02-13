//! ZK-proof of Paillier-Blum modulus. Called Пmod or Rmod in the CGGMP21 paper.
//!
//! ## Description
//! A party P has a modulus `N = pq`, with p and q being Blum primes, and
//! `gcd(N, phi(N)) = 1`. P wants to prove that those equalities about N hold,
//! without disclosing p and q.

use crate::misbehavior::dfns_cggmp21::{integer, Integer, M};

use malachite_base::num::{
	arithmetic::traits::{CeilingLogBase2, ModPowerOf2Assign, Pow, ShrRound},
	basic::{integers::PrimitiveInt, traits::Zero},
	conversion::traits::ExactFrom,
	logic::traits::BitConvertible,
};
use malachite_nz::{natural::Natural, platform::Limb};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;

/// Public data that both parties know: the Paillier-Blum modulus
#[derive(RuntimeDebug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Data {
	#[serde(with = "integer::serde")]
	pub n: Integer,
}

/// Private data of prover
#[derive(Clone)]
pub struct PrivateData {
	pub p: Integer,
	pub q: Integer,
}

/// Prover's first message, obtained by [`interactive::commit`]
#[derive(RuntimeDebug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Commitment {
	#[serde(with = "integer::serde")]
	pub w: Integer,
}

/// Verifier's challenge to prover. Can be obtained deterministically by
/// [`non_interactive::challenge`] or randomly by [`interactive::challenge`]
///
/// Consists of `M` singular challenges
#[derive(RuntimeDebug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Challenge {
	#[serde(with = "integer::serde_list")]
	pub ys: [Integer; M],
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

/// The ZK proof. Computed by [`interactive::prove`] or
/// [`non_interactive::prove`]. Consists of M proofs for each challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
	#[serde(with = "serde_with::As::<[serde_with::Same; M]>")]
	pub points: [ProofPoint; M],
}

/// Witness that proof is invalid
#[derive(Debug)]
pub struct InvalidProof;

/// The interactive version of the ZK proof. Should be completed in 3 rounds:
/// prover commits to data, verifier responds with a random challenge, and
/// prover gives proof with commitment and challenge.
pub mod interactive {

	use malachite_base::num::arithmetic::traits::{Mod, ModPow, Parity};
	use malachite_nz::natural::Natural;

	use super::{is_prime, Challenge, Commitment, Data, InvalidProof, Proof};

	/// Verify the proof. If this succeeds, the relation Rmod holds with chance
	/// `1/2^M`
	pub fn verify(
		data: &Data,
		commitment: &Commitment,
		challenge: &Challenge,
		proof: &Proof,
	) -> Result<(), InvalidProof> {
		if !is_prime(&data.n, 25) {
			return Err(InvalidProof)
		}
		if data.n.even() {
			return Err(InvalidProof)
		}
		for (point, y) in proof.points.iter().zip(challenge.ys.iter()) {
			if point
				.z
				.unsigned_abs_ref()
				.mod_pow(data.n.unsigned_abs_ref(), data.n.unsigned_abs_ref()) !=
				*y
			{
				return Err(InvalidProof)
			}
			let y = y.clone();
			let y = if point.a { &data.n - y } else { y };
			let y = if point.b { (y * &commitment.w).mod_op(&data.n) } else { y };
			if point
				.x
				.unsigned_abs_ref()
				.mod_pow(&Natural::from(4u32), data.n.unsigned_abs_ref()) !=
				y
			{
				return Err(InvalidProof)
			}
		}
		Ok(())
	}
}

/// The non-interactive version of proof. Completed in one round, for example
/// see the documentation of parent module.
pub mod non_interactive {
	use digest::{typenum::U32, Digest};
	use malachite_base::num::arithmetic::traits::UnsignedAbs;

	use crate::misbehavior::dfns_cggmp21::{hashing_rng::HashRng, integer::RugInteger, Integer, M};

	use super::{random_naturals_less_than, Challenge, Commitment, Data, InvalidProof, Proof};

	/// Verify the proof, deriving challenge independently from same data
	pub fn verify<D>(
		shared_state: D,
		data: &Data,
		commitment: &Commitment,
		proof: &Proof,
	) -> Result<(), InvalidProof>
	where
		D: Digest<OutputSize = U32> + Clone,
	{
		let challenge = challenge(shared_state, data, commitment);
		super::interactive::verify(data, commitment, &challenge, proof)
	}

	/// Deterministically compute challenge based on prior known values in protocol
	pub fn challenge<D>(
		shared_state: D,
		Data { ref n }: &Data,
		commitment: &Commitment,
	) -> Challenge
	where
		D: Digest,
	{
		#[cfg(not(feature = "std"))]
		use ::alloc::vec::Vec;

		let shared_state = shared_state.finalize();
		let hash = |d: D| {
			d.chain_update(&shared_state)
				.chain_update(RugInteger::from(n).to_vec())
				.chain_update(RugInteger::from(&commitment.w).to_vec())
				.finalize()
		};
		let mut rng = HashRng::new(hash);
		// since we can't use Default and Integer isn't copy, we initialize
		// like this
		let ys = random_naturals_less_than(&mut rng, n.unsigned_abs())
			.take(M)
			.map(Integer::from)
			.collect::<Vec<_>>()
			.try_into()
			.expect("M is constant");
		Challenge { ys }
	}
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
				return Some(x)
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
pub fn random_naturals_less_than<'a, R: RngCore>(
	rng: &'a mut R,
	limit: Natural,
) -> RandomNaturalsLessThan<'a, R> {
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
		return Natural::ZERO
	}
	let l = usize::exact_from(
		bits.shr_round(Limb::LOG_WIDTH, malachite_base::rounding_modes::RoundingMode::Ceiling)
			.0,
	);
	#[cfg(not(feature = "std"))]
	let mut xs = ::alloc::vec::Vec::with_capacity(l);
	#[cfg(feature = "std")]
	let mut xs = ::std::vec::Vec::with_capacity(l);

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
		return
	}
	let new_size = usize::exact_from(
		pow.shr_round(Limb::LOG_WIDTH, malachite_base::rounding_modes::RoundingMode::Ceiling)
			.0,
	);
	if new_size > xs.len() {
		return
	}
	malachite_base::slices::slice_set_zero(&mut xs[new_size..]);
	let leftover_bits = pow & Limb::WIDTH_MASK;
	if leftover_bits != 0 {
		xs[new_size - 1].mod_power_of_2_assign(leftover_bits);
	}
}

/// Tests if the given integer within a string is prime
///
/// # Parameters
///
/// `n` is the integer to test its primality
pub fn is_prime(n: &Integer, reps: usize) -> bool {
	// Translated from
	// https://rosettacode.org/wiki/Miller%E2%80%93Rabin_primality_test#Perl

	if n < &Integer::from(2) {
		return false
	}

	if n == &Integer::from(2) || n == &Integer::from(3) || n == &Integer::from(5) {
		return true
	}

	if (n % Integer::from(2u32)) == Integer::ZERO {
		return false
	}

	let n_sub = n - Integer::from(1u32);
	let mut exponent = n_sub.clone();
	let mut trials = 0;

	while (&exponent % Integer::from(2u32)) == Integer::from(0u32) {
		exponent /= Integer::from(2);
		trials += 1;
	}

	'LOOP: for i in 1..reps {
		let mut result = bmodpow(&(Integer::from(2) + Integer::from(i)), &exponent, &n);

		if result == Integer::from(1) || result == n_sub {
			continue
		}

		for _ in 1..trials {
			result = result.pow(2) % n;

			if result == Integer::from(1) {
				return false
			}

			if result == n_sub {
				continue 'LOOP
			}
		}

		return false
	}

	true
}

fn bmodpow(base: &Integer, exponent: &Integer, modulus: &Integer) -> Integer {
	// Translated from
	// http://search.cpan.org/~pjacklam/Math-BigInt-1.999810/lib/Math/BigInt.pm#Arithmetic_methods

	if *base == Integer::from(0u32) {
		return match *exponent == Integer::from(0u32) {
			true => Integer::from(1u32),
			false => Integer::from(0u32),
		}
	}

	if *modulus == Integer::from(1u32) {
		return Integer::from(0u32)
	}

	let exponent_in_binary = exponent.to_bits_asc();
	let mut my_base = base.clone();
	let mut result = Integer::from(1u32);

	for next_bit in exponent_in_binary {
		if next_bit {
			result = (result * my_base.clone()) % modulus;
		}

		my_base = my_base.pow(2) % modulus;
	}

	result
}
