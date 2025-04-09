//! Performs batch Schnorr signature verification.
//!
//! Batch verification asks whether *all* signatures in some set are valid,
//! rather than asking whether *each* of them is valid. This allows sharing
//! computations among all signature verifications, performing less work overall
//! at the cost of higher latency (the entire batch must complete), complexity
//! of caller code (which must assemble a batch of signatures across
//! work-items), and loss of the ability to easily pinpoint failing signatures.

use rand_core::{CryptoRng, RngCore};

use crate::{Ciphersuite, Element, scalar_mul::VartimeMultiscalarMul, *};

/// A batch verification item.
///
/// This struct exists to allow batch processing to be decoupled from the
/// lifetime of the message. This is useful when using the batch verification
/// API in an async context.
#[derive(Clone, Debug)]
pub struct Item<C: Ciphersuite> {
	vk: VerifyingKey<C>,
	sig: Signature<C>,
	c: Challenge<C>,
}

impl<C> Item<C>
where
	C: Ciphersuite,
{
	/// Create a new batch [`Item`] from a [`VerifyingKey`], [`Signature`]
	/// and a message to be verified.
	pub fn new<M>(vk: VerifyingKey<C>, sig: Signature<C>, msg: M) -> Result<Self, Error<C>>
	where
		M: AsRef<[u8]>,
	{
		let (msg, sig, vk) = <C>::pre_verify(msg.as_ref(), &sig, &vk)?;
		let c = <C>::challenge(&sig.R, &vk, &msg)?;

		Ok(Self { vk: *vk, sig: *sig, c })
	}
}

impl<C> Item<C>
where
	C: Ciphersuite,
{
	/// Perform non-batched verification of this `Item`.
	///
	/// This is useful (in combination with `Item::clone`) for implementing
	/// fallback logic when batch verification fails. In contrast to
	/// [`VerifyingKey::verify`](crate::VerifyingKey::verify), which
	/// requires borrowing the message data, the `Item` type is unlinked
	/// from the lifetime of the message.
	pub fn verify_single(self) -> Result<(), Error<C>> {
		self.vk.verify_prehashed(self.c, &self.sig)
	}
}

/// A batch verification context.
pub struct Verifier<C: Ciphersuite> {
	/// Signature data queued for verification.
	signatures: Vec<Item<C>>,
}

impl<C> Verifier<C>
where
	C: Ciphersuite,
{
	/// Constructs a new batch verifier.
	pub fn new() -> Verifier<C> {
		Verifier::default()
	}

	/// Queues an Item for verification.
	pub fn queue<I: Into<Item<C>>>(&mut self, item: I) {
		self.signatures.push(item.into());
	}

	/// Performs batch verification, returning `Ok(())` if all signatures were
	/// valid and `Err` otherwise, or if the batch is empty.
	///
	/// The batch verification equation is:
	///
	/// h_G * -[sum(z_i * s_i)]P_G + sum(\[z_i\]R_i + [z_i * c_i]VK_i) = 0_G
	///
	/// which we split out into:
	///
	/// h_G * -[sum(z_i * s_i)]P_G + sum(\[z_i\]R_i) + sum([z_i * c_i]VK_i) =
	/// 0_G
	///
	/// so that we can use multiscalar multiplication speedups.
	///
	/// where for each signature i,
	/// - VK_i is the verification key;
	/// - R_i is the signature's R value;
	/// - s_i is the signature's s value;
	/// - c_i is the hash of the message and other data;
	/// - z_i is a random 128-bit Scalar;
	/// - h_G is the cofactor of the group;
	/// - P_G is the generator of the subgroup;
	///
	/// As follows elliptic curve scalar multiplication convention,
	/// scalar variables are lowercase and group point variables
	/// are uppercase. This does not exactly match the RedDSA
	/// notation in the [protocol specification §B.1][ps].
	///
	/// [ps]: https://zips.z.cash/protocol/protocol.pdf#reddsabatchverify
	pub fn verify<R: RngCore + CryptoRng>(self, mut rng: R) -> Result<(), Error<C>> {
		let n = self.signatures.len();

		if n == 0 {
			return Err(Error::InvalidSignature);
		}

		let mut VK_coeffs = Vec::with_capacity(n);
		let mut VKs = Vec::with_capacity(n);
		let mut R_coeffs = Vec::with_capacity(n);
		let mut Rs = Vec::with_capacity(n);
		let mut P_coeff_acc = <<C::Group as Group>::Field>::zero();

		for item in self.signatures.iter() {
			let z = item.sig.z;
			let R = item.sig.R;

			let blind = <<C::Group as Group>::Field>::random(&mut rng);

			let P_coeff = blind * z;
			P_coeff_acc = P_coeff_acc - P_coeff;

			R_coeffs.push(blind);
			Rs.push(R);

			VK_coeffs.push(<<C::Group as Group>::Field>::zero() + (blind * item.c.0));
			VKs.push(item.vk.to_element());
		}

		let scalars = core::iter::once(&P_coeff_acc).chain(VK_coeffs.iter()).chain(R_coeffs.iter());

		let basepoints = [C::Group::generator()];
		let points = basepoints.iter().chain(VKs.iter()).chain(Rs.iter());

		let check: Element<C> =
			VartimeMultiscalarMul::<C>::vartime_multiscalar_mul(scalars, points);

		if (check * <C::Group>::cofactor()) == <C::Group>::identity() {
			Ok(())
		} else {
			Err(Error::InvalidSignature)
		}
	}
}

impl<C> Default for Verifier<C>
where
	C: Ciphersuite,
{
	fn default() -> Self {
		Self { signatures: Vec::new() }
	}
}
