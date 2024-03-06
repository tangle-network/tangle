//! Пprm or Rprm in the paper. Proof that s ⋮ t modulo N. Non-interactive
//! version only.
use crate::misbehavior::dfns_cggmp21::{integer, integer::RugInteger, Integer, M};
use digest::{typenum::U32, Digest};
use malachite_base::num::arithmetic::traits::{Mod, ModPow, UnsignedAbs};
use rand_core::{RngCore, SeedableRng};
use sp_core::RuntimeDebug;

struct Challenge {
	es: [bool; M],
}

/// Data to construct proof about
#[derive(Clone, Copy)]
pub struct Data<'a> {
	pub N: &'a Integer,
	pub s: &'a Integer,
	pub t: &'a Integer,
}

/// The ZK proof. Computed by [`prove`].
///
/// Parameter `M` is security level. The probability of an adversary generating
/// a correct proof for incorrect data is $2^{-M}$. You can use M defined here
/// as [`SECURITY`]
#[derive(Clone, RuntimeDebug, serde::Deserialize, udigest::Digestable)]
pub struct Proof {
	#[serde(with = "integer::serde_list")]
	#[udigest(with = integer::encoding::integers_list)]
	pub commitment: [Integer; M],
	#[serde(with = "integer::serde_list")]
	#[udigest(with = integer::encoding::integers_list)]
	pub zs: [Integer; M],
}

fn derive_challenge<D>(shared_state: D, data: Data, commitment: &[Integer; M]) -> Challenge
where
	D: Digest<OutputSize = U32>,
{
	let mut digest = shared_state
		.chain_update(RugInteger::from(data.N).to_vec())
		.chain_update(RugInteger::from(data.s).to_vec())
		.chain_update(RugInteger::from(data.t).to_vec());
	for a in commitment.iter() {
		digest.update(RugInteger::from(a).to_vec());
	}
	let seed = digest.finalize();
	let mut rng = rand_chacha::ChaCha20Rng::from_seed(seed.into());

	// generate bools by hand since we don't have rand
	let mut es = [false; M];
	let mut current = rng.next_u32();
	let mut bits_generated = 0;
	for e_ref in es.iter_mut() {
		if bits_generated == 32 {
			current = rng.next_u32();
			bits_generated = 0;
		}
		*e_ref = (current & 1) == 1;
		current >>= 1;
	}
	Challenge { es }
}

/// Verify the proof. Derives determenistic challenge based on `shared_state`
/// and `data`.
pub fn verify<D>(shared_state: D, data: Data, proof: &Proof) -> Result<(), InvalidProof>
where
	D: Digest<OutputSize = U32>,
{
	let challenge = derive_challenge(shared_state, data, &proof.commitment);
	for ((z, a), e) in proof.zs.iter().zip(&proof.commitment).zip(&challenge.es) {
		let lhs: Integer = data
			.t
			.unsigned_abs_ref()
			.mod_pow(z.unsigned_abs(), data.N.unsigned_abs_ref())
			.into();
		if *e {
			let rhs = (data.s * a).mod_op(data.N);
			if lhs != rhs {
				return Err(InvalidProof);
			}
		} else if lhs != *a {
			return Err(InvalidProof);
		}
	}
	Ok(())
}

/// Witness that proof is invalid
#[derive(Debug)]
pub struct InvalidProof;

#[cfg(test)]
pub mod original {
	use digest::{typenum::U32, Digest};
	use paillier_zk::{fast_paillier::utils, rug, Integer};
	use rand_core::{RngCore, SeedableRng};
	use serde::{Deserialize, Serialize};
	use serde_with::serde_as;

	/// Data to construct proof about
	#[derive(Clone, Copy)]
	pub struct Data<'a> {
		pub N: &'a Integer,
		pub s: &'a Integer,
		pub t: &'a Integer,
	}

	struct Challenge<const M: usize> {
		es: [bool; M],
	}

	/// The ZK proof. Computed by [`prove`].
	///
	/// Parameter `M` is security level. The probability of an adversary generating
	/// a correct proof for incorrect data is $2^{-M}$. You can use M defined here
	/// as [`SECURITY`]
	#[serde_as]
	#[derive(Clone, Serialize, Deserialize, udigest::Digestable)]
	pub struct Proof<const M: usize> {
		#[serde_as(as = "[_; M]")]
		#[udigest(with = encoding::integers_list)]
		pub commitment: [Integer; M],
		#[serde_as(as = "[_; M]")]
		#[udigest(with = encoding::integers_list)]
		pub zs: [Integer; M],
	}
	fn derive_challenge<const M: usize, D>(
		shared_state: D,
		data: Data,
		commitment: &[Integer; M],
	) -> Challenge<M>
	where
		D: Digest<OutputSize = U32>,
	{
		let order = rug::integer::Order::Msf;
		let mut digest = shared_state
			.chain_update(data.N.to_digits(order))
			.chain_update(data.s.to_digits(order))
			.chain_update(data.t.to_digits(order));
		for a in commitment.iter() {
			digest.update(a.to_digits(order));
		}
		let seed = digest.finalize();
		let mut rng = rand_chacha::ChaCha20Rng::from_seed(seed.into());

		// generate bools by hand since we don't have rand
		let mut es = [false; M];
		let mut current = rng.next_u32();
		let mut bits_generated = 0;
		for e_ref in es.iter_mut() {
			if bits_generated == 32 {
				current = rng.next_u32();
				bits_generated = 0;
			}
			*e_ref = (current & 1) == 1;
			current >>= 1;
		}
		Challenge { es }
	}

	/// Compute the proof for the given data, producing random commitment and
	/// deriving deterministic challenge based on `shared_state` and `data`
	///
	/// - `phi` - $φ(N) = (p-1)(q-1)$
	/// - `lambda` - λ such that $s = t^λ$
	pub fn prove<const M: usize, R, D>(
		shared_state: D,
		rng: &mut R,
		data: Data,
		phi: &Integer,
		lambda: &Integer,
	) -> Result<Proof<M>, ()>
	where
		D: Digest<OutputSize = U32>,
		R: RngCore,
	{
		let private_commitment =
			[(); M].map(|()| phi.random_below_ref(&mut utils::external_rand(rng)).into());
		let commitment = private_commitment
			.clone()
			.map(|a| data.t.pow_mod_ref(&a, data.N).map(|r| r.into()));
		// TODO: since array::try_map is not stable yet, we have to be hacky here
		let commitment = if commitment.iter().any(Option::is_none) {
			return Err(());
		} else {
			// We made sure that every item in the array is `Some(_)`
			#[allow(clippy::unwrap_used)]
			commitment.map(Option::unwrap)
		};

		let challenge: Challenge<M> = derive_challenge(shared_state, data, &commitment);

		let mut zs = private_commitment;
		for (z_ref, e) in zs.iter_mut().zip(&challenge.es) {
			if *e {
				*z_ref += lambda;
				z_ref.modulo_mut(phi);
			}
		}
		Ok(Proof { commitment, zs })
	}

	/// Unambiguous encoding for different types for which it was not defined
	pub mod encoding {
		use paillier_zk::rug;

		pub fn integer<B: udigest::Buffer>(
			x: &rug::Integer,
			encoder: udigest::encoding::EncodeValue<B>,
		) {
			encoder.encode_leaf().chain(x.to_digits(rug::integer::Order::Msf));
		}

		pub fn integers_list<B: udigest::Buffer>(
			list: &[rug::Integer],
			encoder: udigest::encoding::EncodeValue<B>,
		) {
			let mut encoder = encoder.encode_list();
			for x in list {
				integer(x, encoder.add_item())
			}
		}
	}
}
