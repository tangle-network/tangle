// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
//
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
//! Call filters for testnet

use super::*;

pub struct MainnetCallFilter;
impl Contains<RuntimeCall> for MainnetCallFilter {
	fn contains(call: &RuntimeCall) -> bool {
		let is_core_call = matches!(call, RuntimeCall::System(_) | RuntimeCall::Timestamp(_));
		if is_core_call {
			// always allow core call
			return true
		}

		let is_paused =
			pallet_transaction_pause::PausedTransactionFilter::<Runtime>::contains(call);
		if is_paused {
			// no paused call
			return false
		}

		let democracy_related = matches!(
			call,
			// Filter democracy proposals creation
			RuntimeCall::Democracy(_) |
			// disallow council
			RuntimeCall::Council(_)
		);

		// block all democracy calls in mainnet, TODO : this will be enabled later
		if democracy_related {
			return false
		}

		let light_client_related = matches!(
			call,
			// Filter light client calls
			RuntimeCall::Eth2Client(_)
		);

		// block all light client calls in mainnet, TODO : this will be enabled later
		if light_client_related {
			return false
		}

		// al other calls are allowed
		true
	}
}
