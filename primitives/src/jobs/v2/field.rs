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
use alloc::string::String;
use frame_support::pallet_prelude::*;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;
use sp_runtime::traits::Get;
use sp_std::boxed::Box;

macro_rules! impl_from {
    ($from:ty, $variant:ident) => {
        impl<AccountId, MaxSize: Get<u32>> From<$from> for Field<AccountId, MaxSize> {
            fn from(val: $from) -> Self {
                Self::$variant(val)
            }
        }
    };

    ($from:ty, $variant:ident, $conv:expr) => {
        impl<AccountId, MaxSize: Get<u32>> From<$from> for Field<AccountId, MaxSize> {
            fn from(val: $from) -> Self {
                Self::$variant($conv(val))
            }
        }
    };

    ($( $from:ty => $variant:ident ),*) => {
        $( impl_from!($from, $variant); )*
    };
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(transparent))]
pub struct BoundedSring<S: Get<u32>>(pub BoundedVec<u8, S>);

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Clone, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Field<AccountId, MaxSize: Get<u32>> {
	/// Represents a field of null value.
	None,
	/// Represents a boolean.
	Bool(bool),
	/// Represents a u8 Number.
	Uint8(u8),
	/// Represents a i8 Number.
	Int8(i8),
	/// Represents a u16 Number.
	Uint16(u16),
	/// Represents a i16 Number.
	Int16(i16),
	/// Represents a u32 Number.
	Uint32(u32),
	/// Represents a i32 Number.
	Int32(i32),
	/// Represents a u64 Number.
	Uint64(u64),
	/// Represents a i64 Number.
	Int64(i64),
	/// Represents a UTF-8 string.
	String(BoundedSring<MaxSize>),
	/// Represents a Raw Bytes.
	Bytes(BoundedVec<u8, MaxSize>),
	/// Represents an array of values
	/// Fixed Length of values.
	Array(BoundedVec<Self, MaxSize>),
	/// Represents a list of values
	List(BoundedVec<Self, MaxSize>),
	/// A sepcial type for AccountId
	AccountId(AccountId),
}

impl<AccountId, MaxSize: Get<u32>> Field<AccountId, MaxSize> {
	/// Returns `true` if the field is [`None`].
	///
	/// [`None`]: Field::None
	#[must_use]
	pub fn is_none(&self) -> bool {
		matches!(self, Self::None)
	}

	/// Returns `true` if the field is [`Bool`].
	///
	/// [`Bool`]: Field::Bool
	#[must_use]
	pub fn is_bool(&self) -> bool {
		matches!(self, Self::Bool(..))
	}

	pub fn as_bool(&self) -> Option<&bool> {
		if let Self::Bool(v) = self {
			Some(v)
		} else {
			None
		}
	}

	pub fn try_into_bool(self) -> Result<bool, Self> {
		if let Self::Bool(v) = self {
			Ok(v)
		} else {
			Err(self)
		}
	}

	/// Returns `true` if the field is [`Uint8`].
	///
	/// [`Uint8`]: Field::Uint8
	#[must_use]
	pub fn is_uint8(&self) -> bool {
		matches!(self, Self::Uint8(..))
	}

	pub fn as_uint8(&self) -> Option<&u8> {
		if let Self::Uint8(v) = self {
			Some(v)
		} else {
			None
		}
	}

	pub fn try_into_uint8(self) -> Result<u8, Self> {
		if let Self::Uint8(v) = self {
			Ok(v)
		} else {
			Err(self)
		}
	}

	/// Returns `true` if the field is [`Int8`].
	///
	/// [`Int8`]: Field::Int8
	#[must_use]
	pub fn is_int8(&self) -> bool {
		matches!(self, Self::Int8(..))
	}

	pub fn as_int8(&self) -> Option<&i8> {
		if let Self::Int8(v) = self {
			Some(v)
		} else {
			None
		}
	}

	pub fn try_into_int8(self) -> Result<i8, Self> {
		if let Self::Int8(v) = self {
			Ok(v)
		} else {
			Err(self)
		}
	}

	/// Returns `true` if the field is [`Uint16`].
	///
	/// [`Uint16`]: Field::Uint16
	#[must_use]
	pub fn is_uint16(&self) -> bool {
		matches!(self, Self::Uint16(..))
	}

	pub fn as_uint16(&self) -> Option<&u16> {
		if let Self::Uint16(v) = self {
			Some(v)
		} else {
			None
		}
	}

	pub fn try_into_uint16(self) -> Result<u16, Self> {
		if let Self::Uint16(v) = self {
			Ok(v)
		} else {
			Err(self)
		}
	}

	/// Returns `true` if the field is [`Int16`].
	///
	/// [`Int16`]: Field::Int16
	#[must_use]
	pub fn is_int16(&self) -> bool {
		matches!(self, Self::Int16(..))
	}

	pub fn as_int16(&self) -> Option<&i16> {
		if let Self::Int16(v) = self {
			Some(v)
		} else {
			None
		}
	}

	pub fn try_into_int16(self) -> Result<i16, Self> {
		if let Self::Int16(v) = self {
			Ok(v)
		} else {
			Err(self)
		}
	}

	/// Returns `true` if the field is [`Uint32`].
	///
	/// [`Uint32`]: Field::Uint32
	#[must_use]
	pub fn is_uint32(&self) -> bool {
		matches!(self, Self::Uint32(..))
	}

	pub fn as_uint32(&self) -> Option<&u32> {
		if let Self::Uint32(v) = self {
			Some(v)
		} else {
			None
		}
	}

	pub fn try_into_uint32(self) -> Result<u32, Self> {
		if let Self::Uint32(v) = self {
			Ok(v)
		} else {
			Err(self)
		}
	}

	/// Returns `true` if the field is [`Int32`].
	///
	/// [`Int32`]: Field::Int32
	#[must_use]
	pub fn is_int32(&self) -> bool {
		matches!(self, Self::Int32(..))
	}

	pub fn as_int32(&self) -> Option<&i32> {
		if let Self::Int32(v) = self {
			Some(v)
		} else {
			None
		}
	}

	pub fn try_into_int32(self) -> Result<i32, Self> {
		if let Self::Int32(v) = self {
			Ok(v)
		} else {
			Err(self)
		}
	}

	/// Returns `true` if the field is [`Uint64`].
	///
	/// [`Uint64`]: Field::Uint64
	#[must_use]
	pub fn is_uint64(&self) -> bool {
		matches!(self, Self::Uint64(..))
	}

	pub fn as_uint64(&self) -> Option<&u64> {
		if let Self::Uint64(v) = self {
			Some(v)
		} else {
			None
		}
	}

	pub fn try_into_uint64(self) -> Result<u64, Self> {
		if let Self::Uint64(v) = self {
			Ok(v)
		} else {
			Err(self)
		}
	}

	/// Returns `true` if the field is [`Int64`].
	///
	/// [`Int64`]: Field::Int64
	#[must_use]
	pub fn is_int64(&self) -> bool {
		matches!(self, Self::Int64(..))
	}

	pub fn as_int64(&self) -> Option<&i64> {
		if let Self::Int64(v) = self {
			Some(v)
		} else {
			None
		}
	}

	pub fn try_into_int64(self) -> Result<i64, Self> {
		if let Self::Int64(v) = self {
			Ok(v)
		} else {
			Err(self)
		}
	}

	/// Returns `true` if the field is [`String`].
	///
	/// [`String`]: Field::String
	#[must_use]
	pub fn is_string(&self) -> bool {
		matches!(self, Self::String(..))
	}

	pub fn as_string(&self) -> Option<&str> {
		if let Self::String(v) = self {
			core::str::from_utf8(&v.0).ok()
		} else {
			None
		}
	}

	pub fn try_into_string(self) -> Result<String, Self> {
		if let Self::String(ref v) = self {
			String::from_utf8(v.0.to_vec()).map_err(|_| self)
		} else {
			Err(self)
		}
	}

	/// Returns `true` if the field is [`Bytes`].
	///
	/// [`Bytes`]: Field::Bytes
	#[must_use]
	pub fn is_bytes(&self) -> bool {
		matches!(self, Self::Bytes(..))
	}

	pub fn as_bytes(&self) -> Option<&BoundedVec<u8, MaxSize>> {
		if let Self::Bytes(v) = self {
			Some(v)
		} else {
			None
		}
	}

	pub fn try_into_bytes(self) -> Result<BoundedVec<u8, MaxSize>, Self> {
		if let Self::Bytes(v) = self {
			Ok(v)
		} else {
			Err(self)
		}
	}

	/// Returns `true` if the field is [`Array`].
	///
	/// [`Array`]: Field::Array
	#[must_use]
	pub fn is_array(&self) -> bool {
		matches!(self, Self::Array(..))
	}

	pub fn as_array(&self) -> Option<&BoundedVec<Self, MaxSize>> {
		if let Self::Array(v) = self {
			Some(v)
		} else {
			None
		}
	}

	pub fn try_into_array(self) -> Result<BoundedVec<Self, MaxSize>, Self> {
		if let Self::Array(v) = self {
			Ok(v)
		} else {
			Err(self)
		}
	}

	/// Returns `true` if the field is [`List`].
	///
	/// [`List`]: Field::List
	#[must_use]
	pub fn is_list(&self) -> bool {
		matches!(self, Self::List(..))
	}

	pub fn as_list(&self) -> Option<&BoundedVec<Self, MaxSize>> {
		if let Self::List(v) = self {
			Some(v)
		} else {
			None
		}
	}

	pub fn try_into_list(self) -> Result<BoundedVec<Self, MaxSize>, Self> {
		if let Self::List(v) = self {
			Ok(v)
		} else {
			Err(self)
		}
	}

	/// Returns `true` if the field is [`AccountId`].
	///
	/// [`AccountId`]: Field::AccountId
	#[must_use]
	pub fn is_account_id(&self) -> bool {
		matches!(self, Self::AccountId(..))
	}

	pub fn as_account_id(&self) -> Option<&AccountId> {
		if let Self::AccountId(v) = self {
			Some(v)
		} else {
			None
		}
	}

	pub fn try_into_account_id(self) -> Result<AccountId, Self> {
		if let Self::AccountId(v) = self {
			Ok(v)
		} else {
			Err(self)
		}
	}
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
	BoundedVec<u8, MaxSize> => Bytes,
	BoundedSring<MaxSize> => String,
	BoundedVec<Self, MaxSize> => List
}

impl<AccountId: Clone, MaxSize: Get<u32> + Clone, const N: usize> TryFrom<[Self; N]>
	for Field<AccountId, MaxSize>
{
	type Error = [Self; N];

	fn try_from(value: [Self; N]) -> Result<Self, Self::Error> {
		if N > MaxSize::get() as usize {
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
	Void,
	/// A Field of `bool` type.
	Bool,
	/// A Field of `u8` type.
	Uint8,
	/// A Field of `i8` type.
	Int8,
	/// A Field of `u16` type.
	Uint16,
	/// A Field of `i16` type.
	Int16,
	/// A Field of `u32` type.
	Uint32,
	/// A Field of `i32` type.
	Int32,
	/// A Field of `u64` type.
	Uint64,
	/// A Field of `i64` type.
	Int64,
	/// A Field of `String` type.
	String,
	/// A Field of `Vec<u8>` type.
	Bytes,
	/// A Field of `Option<T>` type.
	Optional(Box<Self>),
	/// An array of N items of type [`FieldType`].
	Array(u64, Box<Self>),
	/// A List of items of type [`FieldType`].
	List(Box<Self>),
	/// A special type for AccountId
	AccountId,
}

impl FieldType {
	/// Returns `true` if the field type is [`Bool`].
	///
	/// [`Bool`]: FieldType::Bool
	#[must_use]
	pub fn is_bool(&self) -> bool {
		matches!(self, Self::Bool)
	}

	/// Returns `true` if the field type is [`Uint8`].
	///
	/// [`Uint8`]: FieldType::Uint8
	#[must_use]
	pub fn is_uint8(&self) -> bool {
		matches!(self, Self::Uint8)
	}

	/// Returns `true` if the field type is [`Int8`].
	///
	/// [`Int8`]: FieldType::Int8
	#[must_use]
	pub fn is_int8(&self) -> bool {
		matches!(self, Self::Int8)
	}

	/// Returns `true` if the field type is [`Uint16`].
	///
	/// [`Uint16`]: FieldType::Uint16
	#[must_use]
	pub fn is_uint16(&self) -> bool {
		matches!(self, Self::Uint16)
	}

	/// Returns `true` if the field type is [`Int16`].
	///
	/// [`Int16`]: FieldType::Int16
	#[must_use]
	pub fn is_int16(&self) -> bool {
		matches!(self, Self::Int16)
	}

	/// Returns `true` if the field type is [`Uint32`].
	///
	/// [`Uint32`]: FieldType::Uint32
	#[must_use]
	pub fn is_uint32(&self) -> bool {
		matches!(self, Self::Uint32)
	}

	/// Returns `true` if the field type is [`Int32`].
	///
	/// [`Int32`]: FieldType::Int32
	#[must_use]
	pub fn is_int32(&self) -> bool {
		matches!(self, Self::Int32)
	}

	/// Returns `true` if the field type is [`Uint64`].
	///
	/// [`Uint64`]: FieldType::Uint64
	#[must_use]
	pub fn is_uint64(&self) -> bool {
		matches!(self, Self::Uint64)
	}

	/// Returns `true` if the field type is [`Int64`].
	///
	/// [`Int64`]: FieldType::Int64
	#[must_use]
	pub fn is_int64(&self) -> bool {
		matches!(self, Self::Int64)
	}

	/// Returns `true` if the field type is [`String`].
	///
	/// [`String`]: FieldType::String
	#[must_use]
	pub fn is_string(&self) -> bool {
		matches!(self, Self::String)
	}

	/// Returns `true` if the field type is [`Bytes`].
	///
	/// [`Bytes`]: FieldType::Bytes
	#[must_use]
	pub fn is_bytes(&self) -> bool {
		matches!(self, Self::Bytes)
	}

	/// Returns `true` if the field type is [`Optional`].
	///
	/// [`Optional`]: FieldType::Optional
	#[must_use]
	pub fn is_optional(&self) -> bool {
		matches!(self, Self::Optional(..))
	}

	/// Returns `true` if the field type is [`Array`] with the given length.
	///
	/// [`Array`]: FieldType::Array
	#[must_use]
	pub fn is_array(&self, len: usize) -> bool {
		matches!(self, Self::Array(l, ..) if *l == len as u64)
	}

	/// Returns `true` if the field type is [`List`].
	///
	/// [`List`]: FieldType::List
	#[must_use]
	pub fn is_list(&self) -> bool {
		matches!(self, Self::List(..))
	}

	/// Returns `true` if the field type is [`AccountId`].
	///
	/// [`AccountId`]: FieldType::AccountId
	#[must_use]
	pub fn is_account_id(&self) -> bool {
		matches!(self, Self::AccountId)
	}

	/// Returns `true` if the field type is [`Void`].
	///
	/// [`Void`]: FieldType::Void
	#[must_use]
	pub fn is_void(&self) -> bool {
		matches!(self, Self::Void)
	}
}

impl<AccountId, MaxSize: Get<u32>> PartialEq<FieldType> for Field<AccountId, MaxSize> {
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
				a.len() == *len as usize && a.iter().all(|f| f.eq(b))
			},
			(Self::List(a), FieldType::List(b)) => a.iter().all(|f| f.eq(b)),
			(Self::AccountId(_), FieldType::AccountId) => true,
			_ => false,
		}
	}
}

impl<AccountId: Clone, MaxSize: Get<u32> + Clone> From<Field<AccountId, MaxSize>> for FieldType {
	fn from(val: Field<AccountId, MaxSize>) -> Self {
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
