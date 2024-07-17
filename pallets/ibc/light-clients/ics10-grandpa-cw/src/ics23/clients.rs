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
use ibc::core::ics24_host::identifier::ClientId;
use sp_std::prelude::*;

/// client_id => client_type
/// trie key path: "clients/{}/clientType"
pub struct Clients<'a>(&'a mut dyn Storage);

impl<'a> Clients<'a> {
	pub fn new(storage: &'a mut dyn Storage) -> Self {
		Clients(storage)
	}

	pub fn key(_client_id: ClientId) -> Vec<u8> {
		// let client_type_path = format!("{}", ClientTypePath(client_id));
		let client_type_path = "clientType".to_string();
		client_type_path.into_bytes()
	}

	pub fn get(&self, client_id: &ClientId) -> Option<Vec<u8>> {
		self.0.get(&Self::key(client_id.clone()))
	}

	pub fn insert(&mut self, client_id: ClientId, client_type: Vec<u8>) {
		self.0.set(&Self::key(client_id), &client_type);
	}

	pub fn contains_key(&self, client_id: &ClientId) -> bool {
		self.get(client_id).is_some()
	}
}

pub struct ReadonlyClients<'a>(&'a dyn Storage);

impl<'a> ReadonlyClients<'a> {
	pub fn new(storage: &'a dyn Storage) -> Self {
		ReadonlyClients(storage)
	}

	pub fn get(&self, client_id: &ClientId) -> Option<Vec<u8>> {
		self.0.get(&Clients::key(client_id.clone()))
	}

	pub fn contains_key(&self, client_id: &ClientId) -> bool {
		self.get(client_id).is_some()
	}
}
