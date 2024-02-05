use sha2::{digest, Digest};

/// Pseudo-random generateur that obtains values by hashing the provided values
/// salted with an internal counter. The counter is prepended to conserve
/// entropy.
///
/// Useful when you want to deterministically but securely generate elliptic
/// curve points and scalars from some data
///
/// Having u64 counter means that the period of the sequence is 2^64 times
/// `Digest::OutputSize` bytes
pub struct HashRng<F, D: Digest> {
	hasher: F,
	counter: u64,
	buffer: digest::Output<D>,
	offset: usize,
}

impl<F, D: Digest> HashRng<F, D> {
	/// Create RNG from a function that will update and finalize a digest. Use it like this:
	/// ```ignore
	/// HashRng::new(|d| d.chain_update("my_values").finalize())
	/// ```
	pub fn new(hasher: F) -> Self
	where
		F: Fn(D) -> digest::Output<D>,
	{
		let d: D = D::new().chain_update(0u64.to_le_bytes());
		let buffer: digest::Output<D> = hasher(d);
		HashRng { hasher, counter: 1, offset: 0, buffer }
	}
}

impl<F, D> rand_core::RngCore for HashRng<F, D>
where
	D: Digest,
	F: Fn(D) -> digest::Output<D>,
{
	fn next_u32(&mut self) -> u32 {
		const SIZE: usize = std::mem::size_of::<u32>();
		// NOTE: careful with SIZE usage, otherwise it panics
		if self.offset + SIZE > self.buffer.len() {
			self.buffer = (self.hasher)(D::new().chain_update(self.counter.to_le_bytes()));
			self.counter = self.counter.wrapping_add(1);
			self.offset = 0;
		}
		let bytes = &self.buffer[self.offset..self.offset + SIZE];
		self.offset += SIZE;
		#[allow(clippy::expect_used)]
		let bytes: [u8; SIZE] = bytes.try_into().expect("Size mismatch");
		u32::from_le_bytes(bytes)
	}

	fn next_u64(&mut self) -> u64 {
		rand_core::impls::next_u64_via_u32(self)
	}

	fn fill_bytes(&mut self, dest: &mut [u8]) {
		rand_core::impls::fill_bytes_via_next(self, dest)
	}

	fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
		self.fill_bytes(dest);
		Ok(())
	}
}
