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

use grandpa_light_client_primitives::{RelayHostFunctions, StandaloneHostFunctions};
use sp_core::ed25519::{Public, Signature};
use sp_runtime::{
	app_crypto::RuntimePublic,
	traits::{BlakeTwo256, Header},
};
use std::fmt::Debug;

/// Only holds implementations for the relevant Host Functions for the verifier
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct HostFunctionsProvider;

impl light_client_common::HostFunctions for HostFunctionsProvider {
	type BlakeTwo256 = BlakeTwo256;
}

impl RelayHostFunctions for HostFunctionsProvider {
	type Header = sp_runtime::generic::Header<u32, BlakeTwo256>;

	fn ed25519_verify(sig: &Signature, msg: &[u8], pubkey: &Public) -> bool {
		pubkey.verify(&msg, sig)
	}

	fn insert_relay_header_hashes(_headers: &[<Self::Header as Header>::Hash]) {
		unimplemented!()
	}

	fn contains_relay_header_hash(_hash: <Self::Header as Header>::Hash) -> bool {
		unimplemented!()
	}
}

impl StandaloneHostFunctions for HostFunctionsProvider {
	type Header = sp_runtime::generic::Header<u32, BlakeTwo256>;

	fn ed25519_verify(sig: &Signature, msg: &[u8], pubkey: &Public) -> bool {
		pubkey.verify(&msg, sig)
	}

	fn insert_header_hashes(_headers: &[<Self::Header as Header>::Hash]) {
		unimplemented!()
	}

	fn contains_header_hash(_hash: <Self::Header as Header>::Hash) -> bool {
		unimplemented!()
	}
}
