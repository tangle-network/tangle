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

use cosmwasm_std::{Order, Storage};
use ibc::Height;

use crate::msg::GenesisMetadata;

pub struct ProcessedStates<'a>(&'a mut dyn Storage);

impl<'a> ProcessedStates<'a> {
	pub fn new(storage: &'a mut dyn Storage) -> Self {
		ProcessedStates(storage)
	}

	pub fn processed_time_key(height: Height, prefix: &mut Vec<u8>) -> Vec<u8> {
		prefix.append(&mut "consensusStates/".to_string().into_bytes());
		prefix.append(&mut format!("{height}").into_bytes());
		prefix.append(&mut "/processedTime".to_string().into_bytes());
		prefix.clone()
	}

	pub fn processed_height_key(height: Height, prefix: &mut Vec<u8>) -> Vec<u8> {
		prefix.append(&mut "consensusStates/".to_string().into_bytes());
		prefix.append(&mut format!("{height}").into_bytes());
		prefix.append(&mut "/processedHeight".to_string().into_bytes());
		prefix.clone()
	}

	pub fn iteration_key(height: Height, prefix: &mut Vec<u8>) -> Vec<u8> {
		prefix.append(&mut "iterateConsensusStates".to_string().into_bytes());
		prefix.append(&mut height.revision_number.to_be_bytes().to_vec());
		prefix.append(&mut height.revision_height.to_be_bytes().to_vec());
		prefix.clone()
	}

	pub fn get_processed_time(&self, height: Height, prefix: &mut Vec<u8>) -> Option<u64> {
		let full_key = Self::processed_time_key(height, prefix);
		self.0
			.get(&full_key)
			.map(|timestamp| u64::from_be_bytes(timestamp.try_into().unwrap()))
	}

	pub fn set_processed_time(&mut self, height: Height, timestamp: u64, prefix: &mut Vec<u8>) {
		let full_key = Self::processed_time_key(height, prefix);
		let time_vec: [u8; 8] = timestamp.to_be_bytes();
		self.0.set(&full_key, &time_vec);
	}

	pub fn get_processed_height(&self, height: Height, prefix: &mut Vec<u8>) -> Option<u64> {
		let full_key = Self::processed_height_key(height, prefix);
		self.0
			.get(&full_key)
			.map(|height| u64::from_be_bytes(height.try_into().unwrap()))
	}

	pub fn set_processed_height(
		&mut self,
		height: Height,
		processed_height: u64,
		prefix: &mut Vec<u8>,
	) {
		let full_key = Self::processed_height_key(height, prefix);
		let height_vec: [u8; 8] = processed_height.to_be_bytes();
		self.0.set(&full_key, &height_vec);
	}

	pub fn get_iteration_key(&self, height: Height, prefix: &mut Vec<u8>) -> Option<Height> {
		let full_key = Self::iteration_key(height, prefix);
		match self.0.get(&full_key) {
			Some(height) => match std::str::from_utf8(height.as_slice()) {
				Ok(height_str) => Some(Height::try_from(height_str).unwrap()),
				Err(_) => None,
			},
			None => None,
		}
	}

	pub fn set_iteration_key(&mut self, height: Height, prefix: &mut Vec<u8>) {
		let full_key = Self::iteration_key(height, prefix);
		let height_vec = format!("{height}").into_bytes();
		self.0.set(&full_key, &height_vec);
	}

	pub fn get_earliest_height(&mut self, current_height: Height) -> Option<Height> {
		let full_key = Self::iteration_key(current_height, &mut Vec::new());
		let start_key = "iterateConsensusStates".to_string().into_bytes();
		let mut iterator = self.0.range(Some(&start_key), Some(&full_key), Order::Ascending);
		match iterator.next() {
			Some((_, height)) => match std::str::from_utf8(height.as_slice()) {
				Ok(height_str) => Some(Height::try_from(height_str).unwrap()),
				Err(_) => None,
			},
			None => None,
		}
	}

	pub fn remove_states_at_height(&mut self, height: Height) {
		let processed_time_key = Self::processed_time_key(height, &mut Vec::new());
		let processed_height_key = Self::processed_height_key(height, &mut Vec::new());
		let iteration_key = Self::iteration_key(height, &mut Vec::new());

		self.0.remove(&processed_time_key);
		self.0.remove(&processed_height_key);
		self.0.remove(&iteration_key)
	}
}

pub struct ReadonlyProcessedStates<'a>(&'a dyn Storage);

impl<'a> ReadonlyProcessedStates<'a> {
	pub fn new(storage: &'a dyn Storage) -> Self {
		ReadonlyProcessedStates(storage)
	}

	pub fn get_processed_time(&self, height: Height, prefix: &mut Vec<u8>) -> Option<u64> {
		let full_key = ProcessedStates::processed_time_key(height, prefix);
		self.0
			.get(&full_key)
			.map(|timestamp| u64::from_be_bytes(timestamp.try_into().unwrap()))
	}

	pub fn get_processed_height(&self, height: Height, prefix: &mut Vec<u8>) -> Option<u64> {
		let full_key = ProcessedStates::processed_height_key(height, prefix);
		self.0
			.get(&full_key)
			.map(|height| u64::from_be_bytes(height.try_into().unwrap()))
	}

	pub fn get_iteration_key(&self, height: Height, prefix: &mut Vec<u8>) -> Option<Height> {
		let full_key = ProcessedStates::iteration_key(height, prefix);
		match self.0.get(&full_key) {
			Some(height) => match std::str::from_utf8(height.as_slice()) {
				Ok(height_str) => Some(Height::try_from(height_str).unwrap()),
				Err(_) => None,
			},
			None => None,
		}
	}

	pub fn get_next_height(&self, height: Height) -> Option<Height> {
		let full_key = ProcessedStates::iteration_key(height, &mut Vec::new());
		let mut iterator = self.0.range(Some(&full_key), None, Order::Ascending);
		match iterator.next() {
			Some((_, height)) => match std::str::from_utf8(height.as_slice()) {
				Ok(height_str) => Some(Height::try_from(height_str).unwrap()),
				Err(_) => None,
			},
			None => None,
		}
	}

	pub fn get_prev_height(&self, height: Height) -> Option<Height> {
		let full_key = ProcessedStates::iteration_key(height, &mut Vec::new());
		let mut iterator = self.0.range(None, Some(&full_key), Order::Descending);
		match iterator.next() {
			Some((_, height)) => match std::str::from_utf8(height.as_slice()) {
				Ok(height_str) => Some(Height::try_from(height_str).unwrap()),
				Err(_) => None,
			},
			None => None,
		}
	}

	pub fn get_metadata(&self) -> Option<Vec<GenesisMetadata>> {
		let mut gm: Vec<GenesisMetadata> = Vec::<GenesisMetadata>::new();

		let start_key = "iterateConsensusStates".to_string().into_bytes();
		let iterator = self.0.range(Some(&start_key), None, Order::Ascending);
		for (_, height) in iterator {
			match std::str::from_utf8(height.as_slice()) {
				Ok(height_str) => {
					let height = Height::try_from(height_str).unwrap();
					let processed_height_key =
						ProcessedStates::processed_height_key(height, &mut Vec::new());
					gm.push(GenesisMetadata {
						key: processed_height_key.clone(),
						value: self.0.get(&processed_height_key).unwrap(),
					});
					let processed_time_key =
						ProcessedStates::processed_time_key(height, &mut Vec::new());
					gm.push(GenesisMetadata {
						key: processed_time_key.clone(),
						value: self.0.get(&processed_time_key).unwrap(),
					});
				},
				Err(_) => break,
			}
		}

		let iterator = self.0.range(Some(&start_key), None, Order::Ascending);
		for (key, height) in iterator {
			gm.push(GenesisMetadata { key, value: height });
		}
		Some(gm)
	}
}
