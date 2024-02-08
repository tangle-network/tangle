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

use sp_core::RuntimeDebug;
use sp_std::prelude::*;

#[derive(Clone, RuntimeDebug, serde::Serialize, serde::Deserialize)]
pub struct Integer {
	pub radix: i32,
	#[cfg(not(feature = "std"))]
	pub value: ::alloc::string::String,
	#[cfg(feature = "std")]
	pub value: ::std::string::String,
}

impl Integer {
	pub fn to_vec(&self) -> Vec<u8> {
		if self.value == "0" {
			return Vec::new()
		}
		let mut x = self.value.clone();
		// fix odd length
		if x.len() % 2 != 0 {
			// add a leading zero
			x.insert(0, '0');
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
