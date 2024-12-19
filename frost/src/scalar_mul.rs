//! Non-adjacent form (NAF) implementations for fast batch scalar multiplcation

// We expect slicings in this module to never panic due to algorithmic
// constraints.
#![allow(clippy::indexing_slicing)]

use core::{
	borrow::Borrow,
	fmt::{Debug, Result},
	marker::PhantomData,
};

use alloc::vec::Vec;

use crate::{Ciphersuite, Element, Field, Group, Scalar};

/// Calculates the quotient of `self` and `rhs`, rounding the result towards positive infinity.
///
/// # Panics
///
/// This function will panic if `rhs` is 0 or the division results in overflow.
///
/// This function is similar to `div_ceil` that is [available on
/// Nightly](https://github.com/rust-lang/rust/issues/88581).
///
// TODO: remove this function and use `div_ceil()` instead when `int_roundings`
// is stabilized.
const fn div_ceil(lhs: usize, rhs: usize) -> usize {
	let d = lhs / rhs;
	let r = lhs % rhs;
	if r > 0 && rhs > 0 {
		d + 1
	} else {
		d
	}
}

/// A trait for transforming a scalar generic over a ciphersuite to a non-adjacent form (NAF).
pub trait NonAdjacentForm<C: Ciphersuite> {
	fn non_adjacent_form(&self, w: usize) -> Vec<i8>;
}

impl<C> NonAdjacentForm<C> for Scalar<C>
where
	C: Ciphersuite,
{
	/// Computes a width-(w) "Non-Adjacent Form" of this scalar.
	///
	/// Thanks to curve25519-dalek for the original implementation that informed this one.
	///
	/// # Safety
	///
	/// The full scalar field MUST fit in 256 bits in this implementation.
	fn non_adjacent_form(&self, w: usize) -> Vec<i8> {
		// required by the NAF definition
		debug_assert!(w >= 2);
		// required so that the NAF digits fit in i8
		debug_assert!(w <= 8);

		use byteorder::{ByteOrder, LittleEndian};

		let serialized_scalar = <<C::Group as Group>::Field>::little_endian_serialize(self);
		// The canonical serialization length of this `Scalar` in bytes.
		let serialization_len = serialized_scalar.as_ref().len();

		// Compute the size of the non-adjacent form from the number of bytes needed to serialize
		// `Scalar`s, plus 1 bit.
		//
		// The length of the NAF is at most one more than the bit length.
		let naf_length: usize = serialization_len * u8::BITS as usize + 1;

		// Safety:
		//
		// The max value of `naf_length` (the number of bits to represent the
		// scalar plus 1) _should_ have plenty of room in systems where usize is
		// greater than 8 bits (aka, not a u8). If you are able to compile this
		// code on a system with 8-bit pointers, well done, but this code will
		// probably not compute the right thing for you, use a 16-bit or above
		// system. Since the rest of this code uses u64's for limbs, we
		// recommend a 64-bit system.
		let mut naf = vec![0; naf_length];

		// Get the number of 64-bit limbs we need.
		let num_limbs: usize = div_ceil(naf_length, u64::BITS as usize);

		let mut x_u64 = vec![0u64; num_limbs];

		// This length needs to be 8*destination.len(), so pad out to length num_limbs * 8.
		let mut padded_le_serialized = vec![0u8; num_limbs * 8];

		padded_le_serialized[..serialization_len].copy_from_slice(serialized_scalar.as_ref());

		LittleEndian::read_u64_into(padded_le_serialized.as_ref(), &mut x_u64[0..num_limbs]);

		let width = 1 << w;
		let window_mask = width - 1;

		let mut pos = 0;
		let mut carry = 0;
		while pos < naf_length {
			// Construct a buffer of bits of the scalar, starting at bit `pos`
			let u64_idx = pos / 64;
			let bit_idx = pos % 64;

			let bit_buf: u64 = if bit_idx < 64 - w {
				// This window's bits are contained in a single u64
				x_u64[u64_idx] >> bit_idx
			} else {
				// Combine the current u64's bits with the bits from the next u64
				(x_u64[u64_idx] >> bit_idx) | (x_u64[1 + u64_idx] << (64 - bit_idx))
			};

			// Add the carry into the current window
			let window = carry + (bit_buf & window_mask);

			if window & 1 == 0 {
				// If the window value is even, preserve the carry and continue.
				// Why is the carry preserved?
				// If carry == 0 and window & 1 == 0, then the next carry should be 0
				// If carry == 1 and window & 1 == 0, then bit_buf & 1 == 1 so the next carry should be 1
				pos += 1;
				continue;
			}

			if window < width / 2 {
				carry = 0;
				naf[pos] = window as i8;
			} else {
				carry = 1;
				naf[pos] = (window as i8).wrapping_sub(width as i8);
			}

			pos += w;
		}

		naf
	}
}

/// A trait for variable-time multiscalar multiplication without precomputation.
///
/// Implement for a group element.
pub trait VartimeMultiscalarMul<C: Ciphersuite>: Clone {
	/// Given an iterator of public scalars and an iterator of
	/// `Option`s of group elements, compute either `Some(Q)`, where
	/// $$
	/// Q = c\_1 E\_1 + \cdots + c\_n E\_n,
	/// $$
	/// if all points were `Some(E_i)`, or else return `None`.
	fn optional_multiscalar_mul<I, J>(scalars: I, elements: J) -> Option<Self>
	where
		I: IntoIterator,
		I::Item: Borrow<Scalar<C>>,
		J: IntoIterator<Item = Option<Self>>;

	/// Given an iterator of public scalars and an iterator of
	/// public group elements, compute
	/// $$
	/// Q = c\_1 E\_1 + \cdots + c\_n E\_n,
	/// $$
	/// using variable-time operations.
	///
	/// It is an error to call this function with two iterators of different lengths.
	fn vartime_multiscalar_mul<I, J>(scalars: I, elements: J) -> Self
	where
		I: IntoIterator,
		I::Item: Borrow<Scalar<C>>,
		J: IntoIterator,
		J::Item: Borrow<Self>,
	{
		Self::optional_multiscalar_mul(
			scalars,
			elements.into_iter().map(|e| Some(e.borrow().clone())),
		)
		.expect("all elements should be Some")
	}
}

impl<C> VartimeMultiscalarMul<C> for Element<C>
where
	C: Ciphersuite,
{
	#[allow(clippy::comparison_chain)]
	fn optional_multiscalar_mul<I, J>(scalars: I, elements: J) -> Option<Element<C>>
	where
		I: IntoIterator,
		I::Item: Borrow<Scalar<C>>,
		J: IntoIterator<Item = Option<Element<C>>>,
	{
		let nafs: Vec<_> = scalars
			.into_iter()
			.map(|c| NonAdjacentForm::<C>::non_adjacent_form(c.borrow(), 5))
			.collect();

		let lookup_tables = elements
			.into_iter()
			.map(|P_opt| P_opt.map(|P| LookupTable5::<C, Element<C>>::from(&P)))
			.collect::<Option<Vec<_>>>()?;

		if nafs.len() != lookup_tables.len() {
			return None;
		}

		let mut r = <C::Group>::identity();

		// All NAFs will have the same size, so get it from the first
		if nafs.is_empty() {
			return Some(r);
		}
		let naf_length = nafs[0].len();

		for i in (0..naf_length).rev() {
			let mut t = r + r;

			for (naf, lookup_table) in nafs.iter().zip(lookup_tables.iter()) {
				if naf[i] > 0 {
					t = t + lookup_table.select(naf[i] as usize);
				} else if naf[i] < 0 {
					t = t - lookup_table.select(-naf[i] as usize);
				}
			}

			r = t;
		}

		Some(r)
	}
}

/// Holds odd multiples 1A, 3A, ..., 15A of a point A.
#[derive(Copy, Clone)]
pub(crate) struct LookupTable5<C, T> {
	pub(crate) bytes: [T; 8],
	pub(crate) _marker: PhantomData<C>,
}

impl<C: Ciphersuite, T: Copy> LookupTable5<C, T> {
	/// Given public, odd \\( x \\) with \\( 0 < x < 2^4 \\), return \\(xA\\).
	pub fn select(&self, x: usize) -> T {
		debug_assert_eq!(x & 1, 1);
		debug_assert!(x < 16);

		self.bytes[x / 2]
	}
}

impl<C: Ciphersuite, T: Debug> Debug for LookupTable5<C, T> {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> Result {
		write!(f, "LookupTable5({:?})", self.bytes)
	}
}

impl<'a, C> From<&'a Element<C>> for LookupTable5<C, Element<C>>
where
	C: Ciphersuite,
{
	fn from(A: &'a Element<C>) -> Self {
		let mut Ai = [*A; 8];
		let A2 = *A + *A;
		for i in 0..7 {
			Ai[i + 1] = A2 + Ai[i];
		}

		// Now Ai = [A, 3A, 5A, 7A, 9A, 11A, 13A, 15A]
		LookupTable5 { bytes: Ai, _marker: PhantomData }
	}
}
