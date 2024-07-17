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

extern crate alloc;
extern crate core;

mod channel;
mod client;
mod connection;
mod context;
pub mod contract;
mod error;
pub mod helpers;
pub mod ics23;
mod macros;
pub mod msg;
pub mod state;
mod types;

pub use crate::error::ContractError;

pub const CLIENT_STATE: &[u8] = b"client_state";
pub const STORAGE_PREFIX: &[u8] = b"";

pub type Bytes = Vec<u8>;
