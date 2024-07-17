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

use super::error::Error;
use crate::prelude::*;
use core::{
	fmt::{Display, Formatter},
	str::FromStr,
};

use serde::{Deserialize, Serialize};

/// A string constant included in error acknowledgements.
/// NOTE: Changing this const is state machine breaking as acknowledgements are written into state
pub const ACK_ERR_STR: &str = "error handling packet on destination chain: see events for details";
pub const ACK_SUCCESS_B64: &str = "AQ==";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Acknowledgement {
	Result(String),
	Error(String),
}

impl Acknowledgement {
	pub fn success() -> Self {
		Self::Result(ACK_SUCCESS_B64.to_string())
	}

	pub fn from_error(err: Error) -> Self {
		Self::Error(err.to_string())
	}

	pub fn is_successful(&self) -> bool {
		!matches!(self, Self::Error(_))
	}

	pub fn into_result(self) -> Result<String, String> {
		match self {
			Self::Result(r) => Ok(r),
			Self::Error(e) => Err(e),
		}
	}
}

impl Display for Acknowledgement {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		serde_json::to_string(self)
			.map_err(|_| core::fmt::Error)
			.and_then(|s| write!(f, "{}", s))
	}
}

impl FromStr for Acknowledgement {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		serde_json::from_str(s)
			.map_err(|_e| Error::implementation_specific("could not parse acknowledgement".into()))
	}
}
