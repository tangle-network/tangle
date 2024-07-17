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
use flex_error::{define_error, TraceError};

use crate::{
	applications::transfer,
	core::{ics02_client, ics03_connection, ics04_channel},
};

define_error! {
	#[derive(Debug, PartialEq, Eq)]
	Error {
		Ics02Client
			[ ics02_client::error::Error ]
			| _ | { "ICS02 client error" },

		Ics03Connection
			[ ics03_connection::error::Error ]
			| _ | { "ICS03 connection error" },

		Ics04Channel
			[ ics04_channel::error::Error ]
			| _ | { "ICS04 channel error" },

		Ics20FungibleTokenTransfer
			[ transfer::error::Error ]
			| _ | { "ICS20 fungible token transfer error" },

		UnknownMessageTypeUrl
			{ url: String }
			| e | { format_args!("unknown type URL {0}", e.url) },

		MalformedMessageBytes
			[ TraceError<tendermint_proto::Error> ]
			| _ | { "the message is malformed and cannot be decoded" },
	}
}
