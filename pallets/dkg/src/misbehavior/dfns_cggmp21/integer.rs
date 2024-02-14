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

use malachite_base::num::conversion::traits::FromStringBase;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use sp_std::prelude::*;

#[cfg(not(feature = "std"))]
use ::alloc::string::{String, ToString};

#[derive(Clone, RuntimeDebug, ::serde::Deserialize, ::serde::Serialize)]
pub struct RugInteger {
	radix: i32,
	value: String,
}

impl RugInteger {
	/// Create a new `RugInteger` from a utf8 bytes and a radix.
	pub fn from_utf8_and_radix(v: &[u8], radix: i32) -> Result<Self, core::str::Utf8Error> {
		let value = core::str::from_utf8(v).map(|x| x.to_string())?;
		Ok(Self { radix, value })
	}

	/// Convert `RugInteger` to a `Vec<u8>`.
	pub fn to_vec(&self) -> Vec<u8> {
		if self.value == "0" || self.value.is_empty() {
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

impl<'a> From<&'a malachite_nz::integer::Integer> for RugInteger {
	fn from(x: &'a malachite_nz::integer::Integer) -> Self {
		use malachite_base::num::{
			conversion::traits::ToStringBase, logic::traits::SignificantBits,
		};
		let radix = if x.significant_bits() <= 32 { 10 } else { 16 };
		let value = x.to_string_base(radix as _);
		Self { radix, value }
	}
}

impl From<malachite_nz::integer::Integer> for RugInteger {
	fn from(x: malachite_nz::integer::Integer) -> Self {
		Self::from(&x)
	}
}

#[derive(RuntimeDebug, Default, Encode, Decode, TypeInfo)]
pub struct MalachiteIntegerError;

impl core::fmt::Display for MalachiteIntegerError {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		write!(f, "Could not convert from RugInteger to MalachiteInteger")
	}
}

impl TryFrom<RugInteger> for malachite_nz::integer::Integer {
	type Error = MalachiteIntegerError;
	fn try_from(x: RugInteger) -> Result<Self, Self::Error> {
		if x.radix < 2 || x.radix > 36 {
			return Err(MalachiteIntegerError)
		}
		let radix = x.radix as _;
		let value = x.value;
		malachite_nz::integer::Integer::from_string_base(radix, &value).ok_or(MalachiteIntegerError)
	}
}

/// Unambiguous encoding for different types for which it was not defined
pub mod encoding {
	pub fn integer<B: udigest::Buffer>(
		x: &malachite_nz::integer::Integer,
		encoder: udigest::encoding::EncodeValue<B>,
	) {
		let v = super::RugInteger::from(x);
		encoder.encode_leaf().chain(v.to_vec());
	}

	pub fn integers_list<B: udigest::Buffer>(
		list: &[malachite_nz::integer::Integer],
		encoder: udigest::encoding::EncodeValue<B>,
	) {
		let mut encoder = encoder.encode_list();
		for x in list {
			integer(x, encoder.add_item())
		}
	}
}

#[allow(unused)]
pub mod serde {
	use ::serde::{Deserialize, Deserializer, Serialize, Serializer};

	pub fn serialize<S: Serializer>(
		x: &malachite_nz::integer::Integer,
		serializer: S,
	) -> Result<S::Ok, S::Error> {
		let v = super::RugInteger::from(x);
		v.serialize(serializer)
	}

	pub fn deserialize<'de, D: Deserializer<'de>>(
		deserializer: D,
	) -> Result<malachite_nz::integer::Integer, D::Error> {
		let v = super::RugInteger::deserialize(deserializer)?;
		v.try_into().map_err(serde::de::Error::custom)
	}
}

#[allow(unused)]
pub mod serde_list {
	use crate::misbehavior::dfns_cggmp21::M;
	#[cfg(not(feature = "std"))]
	use ::alloc::vec::Vec;
	use ::serde::{Deserialize, Deserializer, Serialize, Serializer};
	use sp_runtime::DeserializeOwned;

	#[serde_with::serde_as]
	#[derive(Serialize, Deserialize)]
	struct FixedLengthArray<T, const N: usize>(#[serde_as(as = "[_; N]")] [T; N])
	where
		T: Serialize + DeserializeOwned;

	pub fn serialize<S: Serializer>(
		list: &[malachite_nz::integer::Integer],
		serializer: S,
	) -> Result<S::Ok, S::Error> {
		let out: [super::RugInteger; M] = list
			.iter()
			.map(super::RugInteger::from)
			.collect::<Vec<_>>()
			.try_into()
			.map_err(|_| serde::ser::Error::custom("Invalid integer list length"))?;
		out.serialize(serializer)
	}

	pub fn deserialize<'de, D: Deserializer<'de>>(
		deserializer: D,
	) -> Result<[malachite_nz::integer::Integer; M], D::Error> {
		FixedLengthArray::<super::RugInteger, M>::deserialize(deserializer)?
			.0
			.into_iter()
			.map(|x| x.try_into().map_err(serde::de::Error::custom))
			.collect::<Result<Vec<_>, _>>()?
			.try_into()
			.map_err(|_| serde::de::Error::custom("Invalid integer list length"))
	}
}

#[allow(unused)]
pub mod serde_vec {
	#[cfg(not(feature = "std"))]
	use ::alloc::vec::Vec;
	use ::serde::{Deserialize, Deserializer, Serialize, Serializer};

	pub fn serialize<S: Serializer>(
		list: &[malachite_nz::integer::Integer],
		serializer: S,
	) -> Result<S::Ok, S::Error> {
		list.iter()
			.map(super::RugInteger::from)
			.collect::<Vec<_>>()
			.serialize(serializer)
	}

	pub fn deserialize<'de, D: Deserializer<'de>>(
		deserializer: D,
	) -> Result<Vec<malachite_nz::integer::Integer>, D::Error> {
		Vec::<super::RugInteger>::deserialize(deserializer)?
			.into_iter()
			.map(|x| x.try_into().map_err(serde::de::Error::custom))
			.collect()
	}
}
