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

use ibc::{core::ics02_client::client_message::ClientMessage, Height};
use tendermint_proto::Protobuf;

use super::types::LightClientBlockView;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NearHeader {
	inner: LightClientBlockView,
}

impl NearHeader {
	pub fn get_light_client_block_view(&self) -> &LightClientBlockView {
		&self.inner
	}

	pub fn encode_to_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error> {
		unimplemented!()
	}

	pub fn height(&self) -> Height {
		todo!()
	}
}

impl Protobuf<()> for NearHeader {}

impl From<NearHeader> for () {
	fn from(_: NearHeader) -> Self {
		todo!()
	}
}

impl From<()> for NearHeader {
	fn from(_: ()) -> Self {
		todo!()
	}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NearClientMessage {
	Header(NearHeader),
}

impl ClientMessage for NearClientMessage {
	fn encode_to_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error> {
		todo!()
	}
}
