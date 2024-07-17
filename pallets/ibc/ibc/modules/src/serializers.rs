// Copyright 2022 ComposableFi
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::prelude::*;
use serde::{
	de,
	ser::{self, Serialize, Serializer},
	Deserialize, Deserializer,
};
use subtle_encoding::{Encoding, Hex};

pub fn ser_hex_upper<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
	T: AsRef<[u8]>,
{
	let hex = Hex::upper_case().encode_to_string(data).map_err(ser::Error::custom)?;
	hex.serialize(serializer)
}

pub fn deser_hex_upper<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
	D: Deserializer<'de>,
	T: AsRef<[u8]>,
	T: From<Vec<u8>>,
{
	let hex = String::deserialize(deserializer)?;
	let bytes = Hex::upper_case().decode(hex.as_bytes()).map_err(de::Error::custom)?;
	Ok(bytes.into())
}

pub mod serde_string {
	use alloc::string::String;
	use core::{fmt::Display, str::FromStr};

	use serde::{de, Deserialize, Deserializer, Serializer};

	pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
	where
		T: Display,
		S: Serializer,
	{
		serializer.collect_str(value)
	}

	pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
	where
		T: FromStr,
		T::Err: Display,
		D: Deserializer<'de>,
	{
		String::deserialize(deserializer)?.parse().map_err(de::Error::custom)
	}
}
