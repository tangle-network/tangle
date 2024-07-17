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

//! ICS 20: Token Transfer implementation allows for multi-chain denomination handling, which
//! constitutes a "fungible token transfer bridge module" between the IBC routing module and an
//! asset tracking module.
pub mod acknowledgement;
pub mod context;
pub mod denom;
pub mod error;
pub mod events;
pub mod msgs;
pub mod packet;
pub mod relay;

pub use denom::*;

/// Module identifier for the ICS20 application.
pub const MODULE_ID_STR: &str = "transfer";

/// The port identifier that the ICS20 applications
/// typically bind with.
pub const PORT_ID_STR: &str = "transfer";

/// ICS20 application current version.
pub const VERSION: &str = "ics20-1";
