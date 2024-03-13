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
			return true;
		}

		let is_paused =
			<pallet_tx_pause::Pallet<Runtime> as Contains<RuntimeCall>>::contains(call);
		if is_paused {
			// no paused call
			return false;
		}

		match call {
			// Filter democracy proposals creation
			RuntimeCall::Democracy(_) |
			// disallow council
			RuntimeCall::Council(_) |
			// Block jobs and roles pallet
			RuntimeCall::Roles(_) |
			RuntimeCall::Jobs(_) => false,
			// al other calls are allowed
			_ =>true

		}
	}
}
