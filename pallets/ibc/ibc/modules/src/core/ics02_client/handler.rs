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

//! This module implements the processing logic for ICS2 (client abstractions and functions) msgs.

use crate::{
	core::{
		ics02_client::{context::ClientTypes, error::Error, msgs::ClientMsg},
		ics26_routing::context::ReaderContext,
	},
	handler::HandlerOutput,
};
use core::fmt::Debug;

pub mod create_client;
pub mod update_client;
pub mod upgrade_client;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientResult<C: ClientTypes> {
	Create(create_client::Result<C>),
	Update(update_client::Result<C>),
	Upgrade(upgrade_client::Result<C>),
}

/// General entry point for processing any message related to ICS2 (client functions) protocols.
pub fn dispatch<Ctx>(
	ctx: &Ctx,
	msg: ClientMsg<Ctx>,
) -> Result<HandlerOutput<ClientResult<Ctx>>, Error>
where
	Ctx: ReaderContext,
{
	match msg {
		ClientMsg::CreateClient(msg) => create_client::process::<_>(ctx, msg),
		ClientMsg::UpdateClient(msg) => update_client::process::<_>(ctx, msg),
		ClientMsg::UpgradeClient(msg) => upgrade_client::process::<_>(ctx, msg),
	}
}
