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

use crate::{core::ics24_host::identifier::PortId, prelude::*};
use flex_error::define_error;

define_error! {
	#[derive(Debug, PartialEq, Eq, derive_more::From)]
	Error {
		UnknownPort
			{ port_id: PortId }
			| e | { format_args!("port '{0}' is unknown", e.port_id) },

		PortAlreadyBound
			{ port_id: PortId }
			| e | { format_args!("port '{0}' is already bound", e.port_id) },

		ModuleNotFound
			{ port_id: PortId }
			| e | { format_args!("could not retrieve module from port '{0}'", e.port_id) },

		ImplementationSpecific
			{ reason: String }
			| e | { format_args!("implementation specific error: {}", e.reason) },
	}
}
