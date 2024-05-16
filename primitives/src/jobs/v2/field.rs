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

#[cfg(not(feature = "std"))]
use alloc::{string::String, string::ToString, vec::Vec};
use educe::Educe;
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;
use sp_std::boxed::Box;

use super::Constraints;

macro_rules! impl_from {
    ($from:ty, $variant:ident) => {
        impl<C: Constraints> From<$from> for Field<C> {
            fn from(val: $from) -> Self {
                Self::$variant(val)
            }
        }
    };

    ($from:ty, $variant:ident, $conv:expr) => {
        impl<C: Constraints> From<$from> for Field<C> {
            fn from(val: $from) -> Self {
                Self::$variant($conv(val))
            }
        }
    };

    ($( $from:ty => $variant:ident ),*) => {
        $( impl_from!($from, $variant); )*
    };
}

#[derive(Educe, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[educe(Debug(bound()), Clone(bound()), PartialEq(bound()), Eq)]
#[scale_info(skip_type_params(C))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(bound = ""))]
pub enum Field<C: Constraints> {
	/// Represents a field of null value.
	#[codec(index = 0)]
	None,
	/// Represents a boolean.
	#[codec(index = 1)]
	Bool(bool),
	/// Represents a u8 Number.
	#[codec(index = 2)]
	Uint8(u8),
	/// Represents a i8 Number.
	#[codec(index = 3)]
	Int8(i8),
	/// Represents a u16 Number.
	#[codec(index = 4)]
	Uint16(u16),
	/// Represents a i16 Number.
	#[codec(index = 5)]
	Int16(i16),
	/// Represents a u32 Number.
	#[codec(index = 6)]
	Uint32(u32),
	/// Represents a i32 Number.
	#[codec(index = 7)]
	Int32(i32),
	/// Represents a u64 Number.
	#[codec(index = 8)]
	Uint64(u64),
	/// Represents a i64 Number.
	#[codec(index = 9)]
	Int64(i64),
	/// Represents a UTF-8 string.
	#[codec(index = 10)]
	String(BoundedString<C::MaxFieldsSize>),
	/// Represents a Raw Bytes.
	#[codec(index = 11)]
	Bytes(BoundedVec<u8, C::MaxFieldsSize>),
	/// Represents an array of values
	/// Fixed Length of values.
	#[codec(index = 12)]
	Array(BoundedVec<Self, C::MaxFieldsSize>),
	/// Represents a list of values
	#[codec(index = 13)]
	List(BoundedVec<Self, C::MaxFieldsSize>),

	// NOTE: Special types starts from 100
	/// A sepcial type for AccountId
	#[codec(index = 100)]
	AccountId(C::AccountId),
}

impl_from! {
	bool => Bool,
	u8 => Uint8,
	i8 => Int8,
	u16 => Uint16,
	i16 => Int16,
	u32 => Uint32,
	i32 => Int32,
	u64 => Uint64,
	i64 => Int64,
	BoundedVec<u8, C::MaxFieldsSize> => Bytes,
	BoundedString<C::MaxFieldsSize> => String,
	BoundedVec<Self, C::MaxFieldsSize> => List
}

impl<C: Constraints, const N: usize> TryFrom<[Self; N]> for Field<C> {
	type Error = [Self; N];

	fn try_from(value: [Self; N]) -> Result<Self, Self::Error> {
		if N > <C::MaxFieldsSize as Get<u32>>::get() as usize {
			return Err(value);
		}
		let vec = value.to_vec().try_into().map_err(|_| value)?;
		Ok(Self::Array(vec))
	}
}

#[derive(Default, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum FieldType {
	/// A Field of `void` type.
	#[default]
	#[codec(index = 0)]
	Void,
	/// A Field of `bool` type.
	#[codec(index = 1)]
	Bool,
	/// A Field of `u8` type.
	#[codec(index = 2)]
	Uint8,
	/// A Field of `i8` type.
	#[codec(index = 3)]
	Int8,
	/// A Field of `u16` type.
	#[codec(index = 4)]
	Uint16,
	/// A Field of `i16` type.
	#[codec(index = 5)]
	Int16,
	/// A Field of `u32` type.
	#[codec(index = 6)]
	Uint32,
	/// A Field of `i32` type.
	#[codec(index = 7)]
	Int32,
	/// A Field of `u64` type.
	#[codec(index = 8)]
	Uint64,
	/// A Field of `i64` type.
	#[codec(index = 9)]
	Int64,
	/// A Field of `String` type.
	#[codec(index = 10)]
	String,
	/// A Field of `Vec<u8>` type.
	#[codec(index = 11)]
	Bytes,
	/// A Field of `Option<T>` type.
	#[codec(index = 12)]
	Optional(Box<Self>),
	/// An array of N items of type [`FieldType`].
	#[codec(index = 13)]
	Array(u64, Box<Self>),
	/// A List of items of type [`FieldType`].
	#[codec(index = 14)]
	List(Box<Self>),
	// NOTE: Special types starts from 100
	/// A special type for AccountId
	#[codec(index = 100)]
	AccountId,
}

impl<C: Constraints> PartialEq<FieldType> for Field<C> {
	fn eq(&self, other: &FieldType) -> bool {
		match (self, other) {
			(Self::None, FieldType::Optional(_)) => true,
			(Self::Bool(_), FieldType::Bool) => true,
			(Self::Uint8(_), FieldType::Uint8) => true,
			(Self::Int8(_), FieldType::Int8) => true,
			(Self::Uint16(_), FieldType::Uint16) => true,
			(Self::Int16(_), FieldType::Int16) => true,
			(Self::Uint32(_), FieldType::Uint32) => true,
			(Self::Int32(_), FieldType::Int32) => true,
			(Self::Uint64(_), FieldType::Uint64) => true,
			(Self::Int64(_), FieldType::Int64) => true,
			(Self::String(_), FieldType::String) => true,
			(Self::Bytes(_), FieldType::Bytes) => true,
			(Self::Array(a), FieldType::Array(len, b)) => {
				a.len() == *len as usize && a.iter().all(|f| f.eq(b.as_ref()))
			},
			(Self::List(a), FieldType::List(b)) => a.iter().all(|f| f.eq(b.as_ref())),
			(Self::AccountId(_), FieldType::AccountId) => true,
			_ => false,
		}
	}
}

impl<C: Constraints> From<Field<C>> for FieldType {
	fn from(val: Field<C>) -> Self {
		match val {
			Field::None => FieldType::Optional(Box::new(FieldType::Void)),
			Field::Bool(_) => FieldType::Bool,
			Field::Uint8(_) => FieldType::Uint8,
			Field::Int8(_) => FieldType::Int8,
			Field::Uint16(_) => FieldType::Uint16,
			Field::Int16(_) => FieldType::Int16,
			Field::Uint32(_) => FieldType::Uint32,
			Field::Int32(_) => FieldType::Int32,
			Field::Uint64(_) => FieldType::Uint64,
			Field::Int64(_) => FieldType::Int64,
			Field::String(_) => FieldType::String,
			Field::Bytes(_) => FieldType::Bytes,
			Field::Array(a) => FieldType::Array(
				a.len() as u64,
				Box::new(a.first().cloned().map(Into::into).unwrap_or(FieldType::Void)),
			),
			Field::List(a) => FieldType::List(Box::new(
				a.first().cloned().map(Into::into).unwrap_or(FieldType::Void),
			)),
			Field::AccountId(_) => FieldType::AccountId,
		}
	}
}

impl<'a, C: Constraints> From<&'a Field<C>> for ethabi::Token {
	fn from(value: &'a Field<C>) -> Self {
		match value {
			Field::None => ethabi::Token::Tuple(Vec::new()),
			Field::Bool(val) => ethabi::Token::Bool(*val),
			Field::Uint8(val) => ethabi::Token::Uint((*val).into()),
			Field::Int8(val) => ethabi::Token::Int((*val).into()),
			Field::Uint16(val) => ethabi::Token::Uint((*val).into()),
			Field::Int16(val) => ethabi::Token::Int((*val).into()),
			Field::Uint32(val) => ethabi::Token::Uint((*val).into()),
			Field::Int32(val) => ethabi::Token::Int((*val).into()),
			Field::Uint64(val) => ethabi::Token::Uint((*val).into()),
			Field::Int64(val) => ethabi::Token::Int((*val).into()),
			Field::String(val) => ethabi::Token::String(val.to_string()),
			Field::Bytes(val) => ethabi::Token::FixedBytes(val.to_vec()),
			Field::Array(val) => ethabi::Token::Array(val.into_iter().map(Into::into).collect()),
			Field::List(val) => ethabi::Token::Array(val.into_iter().map(Into::into).collect()),
			Field::AccountId(val) => ethabi::Token::Bytes(val.encode()),
		}
	}
}

impl<C: Constraints> From<Field<C>> for ethabi::Token {
	fn from(value: Field<C>) -> Self {
		(&value).into()
	}
}

impl<C: Constraints> Field<C> {
	/// Convrts the field to a `ethabi::Token`.
	/// This is useful for converting the field to a type that can be used in an Ethereum transaction.
	pub fn into_ethabi_token(self) -> ethabi::Token {
		self.into()
	}

	/// Same as [`Self::into_ethabi_token`] but for references.
	pub fn to_ethabi_token(&self) -> ethabi::Token {
		self.into()
	}

	/// Encode the fields to ethabi bytes.
	pub fn encode_to_ethabi(fields: &[Self]) -> ethabi::Bytes {
		if fields.is_empty() {
			return Default::default();
		}
		let tokens: Vec<ethabi::Token> = fields.iter().map(Self::to_ethabi_token).collect();
		ethabi::encode(&tokens)
	}

	/// Encode the fields to ethabi tokens.
	pub fn to_ethabi(fields: &[Self]) -> Vec<ethabi::Token> {
		fields.iter().map(Self::to_ethabi_token).collect()
	}
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(S))]
#[cfg_attr(feature = "std", derive(Serialize), serde(transparent), serde(bound = ""))]
#[repr(transparent)]
pub struct BoundedString<S: Get<u32>>(pub(crate) BoundedVec<u8, S>);

impl<S: Get<u32>> Default for BoundedString<S> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<S: Get<u32>> Clone for BoundedString<S> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<S: Get<u32>> PartialEq for BoundedString<S> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

impl<S: Get<u32>> PartialOrd for BoundedString<S> {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<S: Get<u32>> Ord for BoundedString<S> {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

impl<S: Get<u32>> Eq for BoundedString<S> {}

impl<S: Get<u32>> TryFrom<String> for BoundedString<S> {
	type Error = String;
	fn try_from(value: String) -> Result<Self, Self::Error> {
		let bytes = value.as_bytes().to_vec().try_into().map_err(|_| value)?;
		Ok(Self(bytes))
	}
}

impl<S: Get<u32>> TryFrom<&str> for BoundedString<S> {
	type Error = String;
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		#[cfg(not(feature = "std"))]
		use alloc::string::ToString;

		Self::try_from(value.to_string())
	}
}

impl<S: Get<u32>> core::fmt::Display for BoundedString<S> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		core::fmt::Display::fmt(&core::str::from_utf8(&self.0).unwrap_or_default(), f)
	}
}

impl<S: Get<u32>> BoundedString<S> {
	/// Try to convert the bytes to a string slice.
	pub fn try_as_str(&self) -> Result<&str, core::str::Utf8Error> {
		core::str::from_utf8(&self.0)
	}

	/// Convert the bytes to a string slice.
	pub fn as_str(&self) -> &str {
		self.try_as_str().unwrap_or_default()
	}

	/// check if the string is empty.
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	/// Returns the length of the string.
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Check if the underlying bytes are valid utf8.
	pub fn is_utf8(&self) -> bool {
		core::str::from_utf8(&self.0).is_ok()
	}
}

#[cfg(feature = "std")]
impl<'de, S: Get<u32>> serde::Deserialize<'de> for BoundedString<S> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct StringVisitor<S: Get<u32>>(PhantomData<S>);

		impl<'de, S: Get<u32>> serde::de::Visitor<'de> for StringVisitor<S> {
			type Value = String;

			fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
				formatter.write_str("a string")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Self::visit_string(self, v.to_owned())
			}

			fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				let size = v.len();
				let max = match usize::try_from(S::get()) {
					Ok(n) => n,
					Err(_) => return Err(serde::de::Error::custom("can't convert to usize")),
				};
				if size > max {
					Err(serde::de::Error::invalid_length(
						size,
						&"string length is greater than the maximum allowed",
					))
				} else {
					Ok(v)
				}
			}
		}

		let visitor: StringVisitor<S> = StringVisitor(PhantomData);
		deserializer.deserialize_string(visitor).map(|v| {
			Ok(BoundedString::<S>(
				v.as_bytes().to_vec().try_into().expect("length checked in visitor"),
			))
		})?
	}
}
