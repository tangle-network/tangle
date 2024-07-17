//! # AsBytes trait definition
//!
//! This module hosts the `AsBytes` trait, which is used by the AVL Tree to convert value to raw
//! bytes. This is helpful for making the AVL Tree generic over a wide range of data types for its
//! keys (the values still need to implement `Borrow<[u8]>), as long as they can be interpreted as
//! a slice of bytes.
//!
//! To add support for a new type in the AVL Tree, simply implement the `AsByte` trait for that
//! type.
use alloc::{string::String, vec::Vec};
use core::prelude::v1::*;

pub enum ByteSlice<'a> {
	Slice(&'a [u8]),
	Vector(Vec<u8>),
}

impl AsRef<[u8]> for ByteSlice<'_> {
	fn as_ref(&self) -> &[u8] {
		match self {
			ByteSlice::Slice(s) => s,
			ByteSlice::Vector(v) => v.as_slice(),
		}
	}
}

/// A trait for objects that can be interpreted as a slice of bytes.
pub trait AsBytes {
	fn as_bytes(&self) -> ByteSlice<'_>;
}

impl AsBytes for Vec<u8> {
	fn as_bytes(&self) -> ByteSlice<'_> {
		ByteSlice::Slice(self)
	}
}

impl AsBytes for [u8] {
	fn as_bytes(&self) -> ByteSlice<'_> {
		ByteSlice::Slice(self)
	}
}

impl AsBytes for str {
	fn as_bytes(&self) -> ByteSlice<'_> {
		ByteSlice::Slice(self.as_bytes())
	}
}

impl AsBytes for &str {
	fn as_bytes(&self) -> ByteSlice<'_> {
		ByteSlice::Slice((*self).as_bytes())
	}
}

impl AsBytes for String {
	fn as_bytes(&self) -> ByteSlice<'_> {
		ByteSlice::Slice(self.as_bytes())
	}
}

impl AsBytes for [u8; 1] {
	fn as_bytes(&self) -> ByteSlice<'_> {
		ByteSlice::Slice(self)
	}
}
