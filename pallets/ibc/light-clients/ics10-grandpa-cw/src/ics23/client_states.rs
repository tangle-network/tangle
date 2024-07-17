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

use cosmwasm_std::Storage;

/// client_id => client_states
/// trie key path: "clients/{client_id}/clientState"
/// NOTE: the "clients/{client_id}" prefix is provided automatically by CosmWasm.
pub struct ClientStates<'a>(&'a mut dyn Storage);

impl<'a> ClientStates<'a> {
	pub fn new(storage: &'a mut dyn Storage) -> Self {
		ClientStates(storage)
	}

	pub fn key() -> Vec<u8> {
		let client_state_path = "clientState".to_string();
		client_state_path.into_bytes()
	}

	pub fn get(&self) -> Option<Vec<u8>> {
		ReadonlyClientStates::new(self.0).get()
	}

	pub fn get_prefixed(&self, prefix: &[u8]) -> Option<Vec<u8>> {
		ReadonlyClientStates::new(self.0).get_prefixed(prefix)
	}

	pub fn insert(&mut self, client_state: Vec<u8>) {
		self.0.set(&Self::key(), &client_state);
	}

	pub fn insert_prefixed(&mut self, client_state: Vec<u8>, prefix: &[u8]) {
		self.0.set(&[prefix, Self::key().as_slice()].concat(), &client_state);
	}

	pub fn contains_key(&self) -> bool {
		self.get().is_some()
	}
}

pub struct ReadonlyClientStates<'a>(&'a dyn Storage);

impl<'a> ReadonlyClientStates<'a> {
	pub fn new(storage: &'a dyn Storage) -> Self {
		ReadonlyClientStates(storage)
	}

	pub fn get(&self) -> Option<Vec<u8>> {
		self.0.get(&ClientStates::key())
	}

	pub fn get_prefixed(&self, prefix: &[u8]) -> Option<Vec<u8>> {
		self.0.get(&[prefix, ClientStates::key().as_slice()].concat())
	}

	pub fn contains_key(&self) -> bool {
		self.get().is_some()
	}
}
