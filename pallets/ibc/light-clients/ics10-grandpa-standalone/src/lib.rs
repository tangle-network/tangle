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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::all)]

//! ICS-10: Grandpa IBC light client protocol implementation

extern crate alloc;

pub mod client_def;
pub mod client_message;
pub mod client_state;
pub mod consensus_state;
pub mod error;
pub mod proto;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
