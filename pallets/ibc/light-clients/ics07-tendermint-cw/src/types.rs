// Copyright (C) 2022 ComposableFi.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq)]
pub struct Height {
	/// Previously known as "epoch"
	#[serde(default)]
	pub revision_number: u64,
	/// The height of a block
	pub revision_height: u64,
}

impl From<Height> for ibc::Height {
	fn from(value: Height) -> Self {
		Self { revision_number: value.revision_number, revision_height: value.revision_height }
	}
}
