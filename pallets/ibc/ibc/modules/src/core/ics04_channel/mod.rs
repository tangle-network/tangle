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

//! ICS 04: Channel implementation that facilitates communication between
//! applications and the chains those applications are built upon.

pub mod channel;
pub mod context;
pub mod error;
pub mod events;

pub mod handler;
pub mod msgs;
pub mod packet;

pub mod commitment;
mod version;
pub use version::Version;
