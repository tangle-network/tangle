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

//! These are definitions of messages that a relayer submits to a chain. Specific implementations of
//! these messages can be found, for instance, in ICS 07 for Tendermint-specific chains. A chain
//! handles these messages in two layers: first with the general ICS 02 client handler, which
//! subsequently calls into the chain-specific (e.g., ICS 07) client handler. See:
//! <https://github.com/cosmos/ibc/tree/master/spec/core/ics-002-client-semantics#create>.

use crate::core::ics02_client::{
	context::ClientTypes,
	msgs::{
		create_client::MsgCreateAnyClient, update_client::MsgUpdateAnyClient,
		upgrade_client::MsgUpgradeAnyClient,
	},
};

pub mod create_client;
pub mod update_client;
pub mod upgrade_client;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum ClientMsg<C: ClientTypes> {
	CreateClient(MsgCreateAnyClient<C>),
	UpdateClient(MsgUpdateAnyClient<C>),
	UpgradeClient(MsgUpgradeAnyClient<C>),
}
