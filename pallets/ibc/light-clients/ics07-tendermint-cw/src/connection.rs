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

use crate::context::Context;
use ibc::{
	core::{
		ics03_connection::{connection::ConnectionEnd, context::ConnectionReader, error::Error},
		ics23_commitment::commitment::CommitmentPrefix,
		ics24_host::identifier::ConnectionId,
	},
	Height,
};
use ics07_tendermint::HostFunctionsProvider;
use std::time::Duration;

impl<'a, H: HostFunctionsProvider> ConnectionReader for Context<'a, H> {
	fn minimum_delay_period(&self) -> Duration {
		unimplemented!("minimum_delay_period")
	}

	fn connection_end(&self, _conn_id: &ConnectionId) -> Result<ConnectionEnd, Error> {
		Err(Error::implementation_specific(
			"'connection_end' is unavailable from the client".to_string(),
		))
	}

	fn host_oldest_height(&self) -> Height {
		unimplemented!("the method should be removed in the future");
	}

	#[allow(clippy::disallowed_methods)]
	fn commitment_prefix(&self) -> CommitmentPrefix {
		unimplemented!("'commitment_prefix' is unavailable from the client");
	}

	fn connection_counter(&self) -> Result<u64, Error> {
		Err(Error::implementation_specific(
			"'connection_counter' is unavailable from the client".to_string(),
		))
	}
}
