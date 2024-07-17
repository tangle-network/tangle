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

use core::str::FromStr;

use crate::prelude::*;

use derive_more::Display;
use flex_error::define_error;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

define_error! {
	#[derive(Debug, PartialEq, Eq)]
	SignerError {
		EmptySigner
			| _ | { "signer cannot be empty" },
	}
}

#[derive(
	Clone,
	Debug,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	Serialize,
	Deserialize,
	Display,
	Encode,
	Decode,
	scale_info::TypeInfo,
)]
pub struct Signer(String);

impl FromStr for Signer {
	type Err = SignerError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let s = s.to_string();
		if s.trim().is_empty() {
			return Err(SignerError::empty_signer())
		}
		Ok(Self(s))
	}
}

impl AsRef<str> for Signer {
	fn as_ref(&self) -> &str {
		self.0.as_str()
	}
}
