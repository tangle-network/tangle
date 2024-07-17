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

use super::types::{CryptoHash, LightClientBlockView, ValidatorStakeView};
use crate::client_def::{HostFunctionsTrait, NearClient};
use ibc::{
	core::{
		ics02_client::client_state::{ClientState, ClientType},
		ics24_host::identifier::ChainId,
	},
	prelude::*,
	Height,
};
use serde::{Deserialize, Serialize};
use std::{marker::PhantomData, time::Duration};
use tendermint_proto::Protobuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NearClientState<H> {
	chain_id: ChainId,
	head: LightClientBlockView,
	current_epoch: CryptoHash,
	next_epoch: CryptoHash,
	current_validators: Vec<ValidatorStakeView>,
	next_validators: Vec<ValidatorStakeView>,
	_phantom: PhantomData<H>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NearUpgradeOptions {}

impl<H: HostFunctionsTrait> NearClientState<H> {
	pub fn get_validators_by_epoch(
		&self,
		epoch_id: &CryptoHash,
	) -> Option<&Vec<ValidatorStakeView>> {
		if epoch_id == &self.current_epoch {
			Some(&self.current_validators)
		} else if epoch_id == &self.next_epoch {
			Some(&self.next_validators)
		} else {
			None
		}
	}

	pub fn get_head(&self) -> &LightClientBlockView {
		&self.head
	}
}

impl<H: HostFunctionsTrait> ClientState for NearClientState<H> {
	type UpgradeOptions = NearUpgradeOptions;
	type ClientDef = NearClient<H>;

	fn chain_id(&self) -> ChainId {
		self.chain_id.clone()
	}

	fn client_type(&self) -> ClientType {
		Self::client_type()
	}

	fn client_def(&self) -> Self::ClientDef {
		NearClient::default()
	}

	fn latest_height(&self) -> Height {
		self.head.get_height()
	}

	fn frozen_height(&self) -> Option<Height> {
		// TODO: validate this
		Some(self.head.get_height())
	}

	fn upgrade(
		self,
		_upgrade_height: Height,
		_upgrade_options: Self::UpgradeOptions,
		_chain_id: ChainId,
	) -> Self {
		// TODO: validate this -- not sure how to process the given parameters in this case
		self
	}

	fn expired(&self, _elapsed: Duration) -> bool {
		todo!()
	}

	fn encode_to_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error> {
		todo!("implement encoding")
	}
}

impl<H> NearClientState<H> {
	pub fn client_type() -> ClientType {
		"13-near".to_string()
	}
}

impl<H: HostFunctionsTrait> Protobuf<()> for NearClientState<H> {}

impl<H: HostFunctionsTrait> From<NearClientState<H>> for () {
	fn from(_: NearClientState<H>) -> Self {
		todo!()
	}
}

impl<H: HostFunctionsTrait> From<()> for NearClientState<H> {
	fn from(_: ()) -> Self {
		todo!()
	}
}
