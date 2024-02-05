//! A `const fn` crc32 checksum implementation.
//!
//! # Examples
//!
//! ```
//! const BYTES: &[u8] = "The quick brown fox jumps over the lazy dog".as_bytes();
//! const CKSUM: u32 = const_crc32::crc32(BYTES);
//! assert_eq!(CKSUM, 0x414fa339_u32);
//! ```
/// used to generate up a [u32; 256] lookup table in `crc32`. this computes
/// the table on demand for a given "index" `i`
#[rustfmt::skip]
const fn table_fn(i: u32) -> u32 {
    let mut out = i;
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out
}
const fn get_table() -> [u32; 256] {
	let mut table: [u32; 256] = [0u32; 256];
	let mut i = 0;
	while i < 256 {
		table[i] = table_fn(i as u32);
		i += 1;
	}
	table
}
const TABLE: [u32; 256] = get_table();
/// A `const fn` crc32 checksum implementation.
///
/// Note: this is a naive implementation that should be expected to have poor performance
/// if used on dynamic data at runtime. Usage should generally be restricted to declaring
/// `const` variables based on `static` or `const` data available at build time.
pub const fn crc32(buf: &[u8]) -> u32 {
	crc32_seed(buf, 0)
}
/// Calculate crc32 checksum, using provided `seed` as the initial state, instead of the
/// default initial state of `0u32`.
///
/// # Examples
///
/// Calculating the checksum from several parts of a larger input:
///
/// ```
/// const BYTES: &[u8] = "The quick brown fox jumps over the lazy dog".as_bytes();
///
/// let mut cksum = 0u32;
///
/// cksum = const_crc32::crc32_seed(&BYTES[0..10], cksum);
/// cksum = const_crc32::crc32_seed(&BYTES[10..15], cksum);
/// cksum = const_crc32::crc32_seed(&BYTES[15..], cksum);
///
/// assert_eq!(cksum, const_crc32::crc32(BYTES));
/// ```
///
/// Using separate seeds for different kinds of data, to produce different checksums depending
/// on what kind of data the bytes represent:
///
/// ```
/// const THING_ONE_SEED: u32 = 0xbaaaaaad_u32;
/// const THING_TWO_SEED: u32 = 0x2bad2bad_u32;
///
/// let thing_one_bytes = "bump! thump!".as_bytes();
/// let thing_two_bytes = "thump! bump!".as_bytes();
///
/// let thing_one_cksum = const_crc32::crc32_seed(thing_one_bytes, THING_ONE_SEED);
/// let thing_two_cksum = const_crc32::crc32_seed(thing_two_bytes, THING_TWO_SEED);
///
/// assert_ne!(thing_one_cksum, thing_two_cksum);
/// ```
#[inline]
pub const fn crc32_seed(buf: &[u8], seed: u32) -> u32 {
	let mut out = !seed;
	let mut i = 0usize;
	while i < buf.len() {
		out = (out >> 8) ^ TABLE[((out & 0xff) ^ (buf[i] as u32)) as usize];
		i += 1;
	}
	!out
}
