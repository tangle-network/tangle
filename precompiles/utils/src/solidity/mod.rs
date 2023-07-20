// Copyright 2022 Webb Technologies Inc.
//
// This file is part of Utils package, originally developed by Purestake Inc.
// Utils package used in Tangle Network in terms of GPLv3.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Provides utilities for compatibility with Solidity tooling.

pub mod codec;
pub mod modifier;
pub mod revert;

pub use codec::{
	decode_arguments, decode_event_data, decode_return_value, encode_arguments, encode_event_data,
	encode_return_value, encode_with_selector, Codec,
};
