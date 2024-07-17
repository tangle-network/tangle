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

use super::types::CryptoHash;
use crate::client_state::NearClientState;
use flex_error::define_error;
use ibc::core::ics02_client::error::Error as Ics02Error;

define_error! {
	#[derive(Debug, PartialEq, Eq)]
	Error {
		InvalidEpoch
		{ epoch_id: CryptoHash }
		| _ | { "invalid epoch id" },
		HeightTooOld
		| _ | { format_args!(
			"height too old")
		},
		InvalidSignature
		| _ | { format_args!(
			"invalid signature")
		},
		InsufficientStakedAmount
		| _ | { format_args!(
			"insufficient staked amount")
		},
		SerializationError
		| _ | { format_args!(
			"serialization error")
		},
		UnavailableBlockProducers
		| _ | { format_args!(
			"unavailable block producers")
		},
	}
}

impl From<Error> for Ics02Error {
	fn from(e: Error) -> Self {
		Ics02Error::client_error(NearClientState::<()>::client_type().to_owned(), e.to_string())
	}
}
